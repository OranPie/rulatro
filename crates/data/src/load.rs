use crate::card_modifier_defs::load_builtin_card_modifiers;
use crate::joker_dsl::{
    load_boss_mixin_refs, load_bosses_dsl_with_locale, load_joker_mixin_refs,
    load_jokers_dsl_with_locale, load_tag_mixin_refs, load_tags_dsl_with_locale,
    parse_effect_dsl_line,
};
use crate::schema::{
    AnteRule, BlindRule, BossDef, CardAttrRules, ConsumableDef, ConsumableKind, Content,
    ContentPack, EconomyRule, EffectBlock, GameConfig, HandRule, JokerDef, RankRule, ShopRule,
    TagDef,
};
use anyhow::{bail, Context};
use rulatro_core::{ActionOp, ActionOpKind, HandKind, JokerEffect};
use rulatro_modding::{FileSystemModLoader, LoadedMod};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const CONSUMABLE_MIXINS_FILE: &str = "consumable_mixins.json";
const NAMED_EFFECT_MIXINS_FILE: &str = "named_effect_mixins.json";
const CARD_ATTRIBUTES_FILE: &str = "card_attributes.json";

pub fn load_content_pack(path: &Path) -> anyhow::Result<ContentPack> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pack = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(pack)
}

pub fn load_game_config(dir: &Path) -> anyhow::Result<GameConfig> {
    let hands: Vec<HandRule> = load_json(dir.join("hands.json"))?;
    let ranks: Vec<RankRule> = load_json(dir.join("ranks.json"))?;
    let blinds: Vec<BlindRule> = load_json(dir.join("blinds.json"))?;
    let antes: Vec<AnteRule> = load_json(dir.join("antes.json"))?;
    let economy: EconomyRule = load_json(dir.join("economy.json"))?;
    let shop: ShopRule = load_json(dir.join("shop.json"))?;
    let card_attrs_path = dir.join(CARD_ATTRIBUTES_FILE);
    let card_attrs: CardAttrRules = if card_attrs_path.exists() {
        load_json(card_attrs_path)?
    } else {
        CardAttrRules::default()
    };

    Ok(GameConfig {
        hands,
        ranks,
        blinds,
        antes,
        economy,
        shop,
        card_attrs,
    })
}

pub fn load_content(dir: &Path) -> anyhow::Result<Content> {
    load_content_with_locale(dir, None)
}

pub fn load_content_with_locale(dir: &Path, locale: Option<&str>) -> anyhow::Result<Content> {
    let base = dir.join("content");
    load_content_dir(&base, false, locale)
}

#[derive(Debug)]
pub struct ModLoadReport {
    pub content: Content,
    pub mods: Vec<LoadedMod>,
    pub warnings: Vec<String>,
}

pub fn load_mods(dir: &Path) -> anyhow::Result<Vec<LoadedMod>> {
    let loader = FileSystemModLoader::new(dir);
    loader
        .load_all()
        .map_err(|err| anyhow::anyhow!(err.to_string()))
}

pub fn load_content_with_mods(assets_dir: &Path, mods_dir: &Path) -> anyhow::Result<ModLoadReport> {
    load_content_with_mods_locale(assets_dir, mods_dir, None)
}

pub fn load_content_with_mods_locale(
    assets_dir: &Path,
    mods_dir: &Path,
    locale: Option<&str>,
) -> anyhow::Result<ModLoadReport> {
    let mut content = load_content_with_locale(assets_dir, locale)?;

    // Populate card_modifiers from the embedded JSON, resolved against the
    // game's card attribute balance values.
    let card_attrs_path = assets_dir.join(CARD_ATTRIBUTES_FILE);
    let card_attrs: CardAttrRules = if card_attrs_path.exists() {
        load_json(card_attrs_path)?
    } else {
        CardAttrRules::default()
    };
    content.card_modifiers = load_builtin_card_modifiers(&card_attrs);

    let mods = load_mods(mods_dir)?;
    if mods.is_empty() {
        return Ok(ModLoadReport {
            content,
            mods,
            warnings: Vec::new(),
        });
    }
    let mut origins = ContentOrigins::from_content(&content);
    let warnings = Vec::new();
    for item in &mods {
        let overrides = parse_overrides(&item.manifest.overrides)?;
        validate_overrides(&overrides, &origins, &item.manifest.meta.id)?;
        let Some(content_spec) = item.manifest.content.as_ref() else {
            continue;
        };
        let mod_content = load_content_dir(&item.root.join(&content_spec.root), true, locale)
            .with_context(|| format!("load mod content {}", item.manifest.meta.id))?;
        merge_content(
            &mut content,
            &mut origins,
            mod_content,
            &item.manifest.meta.id,
            &overrides,
        )?;
    }
    Ok(ModLoadReport {
        content,
        mods,
        warnings,
    })
}

fn load_content_dir(
    base: &Path,
    allow_missing: bool,
    locale: Option<&str>,
) -> anyhow::Result<Content> {
    let locale = normalize_locale(locale);
    let named_mixins =
        load_named_effect_mixins(&base.join(NAMED_EFFECT_MIXINS_FILE), allow_missing)?;
    let jokers_path = base.join("jokers.dsl");
    let jokers = if jokers_path.exists() {
        let mut defs = load_jokers_dsl_with_locale(&jokers_path, Some(&locale))
            .with_context(|| format!("parse {}", jokers_path.display()))?;
        let refs = load_joker_mixin_refs(&jokers_path)
            .with_context(|| format!("parse {}", jokers_path.display()))?;
        apply_named_effect_mixins(
            &jokers_path,
            NamedContentKind::Joker,
            &mut defs,
            &refs,
            &named_mixins,
        )?;
        defs
    } else if allow_missing {
        Vec::new()
    } else {
        bail!("missing {}", jokers_path.display());
    };

    let bosses_path = base.join("bosses.dsl");
    let bosses = if bosses_path.exists() {
        let mut defs = load_bosses_dsl_with_locale(&bosses_path, Some(&locale))
            .with_context(|| format!("parse {}", bosses_path.display()))?;
        let refs = load_boss_mixin_refs(&bosses_path)
            .with_context(|| format!("parse {}", bosses_path.display()))?;
        apply_named_effect_mixins(
            &bosses_path,
            NamedContentKind::Boss,
            &mut defs,
            &refs,
            &named_mixins,
        )?;
        defs
    } else if allow_missing {
        Vec::new()
    } else {
        bail!("missing {}", bosses_path.display());
    };

    let tags_path = base.join("tags.dsl");
    let tags = if tags_path.exists() {
        let mut defs = load_tags_dsl_with_locale(&tags_path, Some(&locale))
            .with_context(|| format!("parse {}", tags_path.display()))?;
        let refs = load_tag_mixin_refs(&tags_path)
            .with_context(|| format!("parse {}", tags_path.display()))?;
        apply_named_effect_mixins(
            &tags_path,
            NamedContentKind::Tag,
            &mut defs,
            &refs,
            &named_mixins,
        )?;
        defs
    } else if allow_missing {
        Vec::new()
    } else {
        bail!("missing {}", tags_path.display());
    };

    let consumable_mixins =
        load_consumable_mixins(&base.join(CONSUMABLE_MIXINS_FILE), allow_missing)?;

    let tarots = load_consumables_optional(
        &base.join("tarots.json"),
        allow_missing,
        &locale,
        &consumable_mixins,
    )?;
    let planets = load_consumables_optional(
        &base.join("planets.json"),
        allow_missing,
        &locale,
        &consumable_mixins,
    )?;
    let spectrals = load_consumables_optional(
        &base.join("spectrals.json"),
        allow_missing,
        &locale,
        &consumable_mixins,
    )?;

    Ok(Content {
        jokers,
        bosses,
        tags,
        tarots,
        planets,
        spectrals,
        card_modifiers: Vec::new(),
    })
}

#[derive(Debug, Clone, Deserialize)]
struct RawConsumableDef {
    id: String,
    name: String,
    kind: ConsumableKind,
    #[serde(default)]
    hand: Option<HandKind>,
    effects: Vec<EffectBlock>,
    #[serde(default)]
    mixins: Vec<String>,
    #[serde(default, alias = "i18n", alias = "locales")]
    names: HashMap<String, String>,
    #[serde(default)]
    skip_last_consumable: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct RawConsumableMixin {
    id: String,
    #[serde(default)]
    kinds: Vec<ConsumableKind>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    effects: Vec<EffectBlock>,
}

#[derive(Debug, Default, Clone)]
struct ConsumableMixinRegistry {
    by_id: HashMap<String, RawConsumableMixin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NamedContentKind {
    Joker,
    Tag,
    Boss,
}

impl NamedContentKind {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "joker" | "jokers" => Some(Self::Joker),
            "tag" | "tags" => Some(Self::Tag),
            "boss" | "bosses" => Some(Self::Boss),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Joker => "joker",
            Self::Tag => "tag",
            Self::Boss => "boss",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawNamedEffectMixinDoc {
    id: String,
    #[serde(default)]
    kinds: Vec<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    effects: Vec<String>,
}

#[derive(Debug, Clone)]
struct RawNamedEffectMixin {
    kinds: Vec<NamedContentKind>,
    requires: Vec<String>,
    effects: Vec<JokerEffect>,
}

#[derive(Debug, Default, Clone)]
struct NamedEffectMixinRegistry {
    by_id: HashMap<String, RawNamedEffectMixin>,
}

fn detect_cycle_ids<'a, F>(
    ids: impl Iterator<Item = &'a str>,
    mut deps_for: F,
) -> Option<Vec<String>>
where
    F: FnMut(&str) -> Vec<String>,
{
    fn visit<F>(
        id: &str,
        deps_for: &mut F,
        visiting: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Option<Vec<String>>
    where
        F: FnMut(&str) -> Vec<String>,
    {
        if visited.contains(id) {
            return None;
        }
        if let Some(index) = visiting.iter().position(|value| value == id) {
            let mut cycle = visiting[index..].to_vec();
            cycle.push(id.to_string());
            return Some(cycle);
        }
        visiting.push(id.to_string());
        for dep in deps_for(id) {
            if let Some(cycle) = visit(&dep, deps_for, visiting, visited) {
                return Some(cycle);
            }
        }
        let _ = visiting.pop();
        visited.insert(id.to_string());
        None
    }

    let mut visiting = Vec::new();
    let mut visited = HashSet::new();
    for id in ids {
        if let Some(cycle) = visit(id, &mut deps_for, &mut visiting, &mut visited) {
            return Some(cycle);
        }
    }
    None
}

fn load_named_effect_mixins(
    path: &Path,
    allow_missing: bool,
) -> anyhow::Result<NamedEffectMixinRegistry> {
    if !path.exists() {
        if !allow_missing {
            // Named mixins are optional for base content too.
        }
        return Ok(NamedEffectMixinRegistry::default());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let values: Vec<RawNamedEffectMixinDoc> =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let mut by_id = HashMap::new();
    for mixin in values {
        let id = mixin.id.trim();
        if id.is_empty() {
            bail!("named mixin id cannot be empty in {}", path.display());
        }
        if mixin.requires.iter().any(|value| value.trim().is_empty()) {
            bail!(
                "named mixin {} has empty requires entry in {}",
                id,
                path.display()
            );
        }
        if by_id.contains_key(id) {
            bail!("duplicate named mixin {} in {}", id, path.display());
        }
        let mut kinds = Vec::new();
        for raw_kind in &mixin.kinds {
            let kind = NamedContentKind::parse(raw_kind).ok_or_else(|| {
                anyhow::anyhow!(
                    "named mixin {} has invalid kind '{}' in {}",
                    id,
                    raw_kind,
                    path.display()
                )
            })?;
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
        let mut effects = Vec::new();
        for (index, line) in mixin.effects.iter().enumerate() {
            if line.trim().is_empty() {
                bail!(
                    "named mixin {} has empty effects[{}] in {}",
                    id,
                    index,
                    path.display()
                );
            }
            let parsed = parse_effect_dsl_line(line).with_context(|| {
                format!(
                    "parse named mixin {} effects[{}] in {}",
                    id,
                    index,
                    path.display()
                )
            })?;
            effects.push(parsed);
        }
        by_id.insert(
            id.to_string(),
            RawNamedEffectMixin {
                kinds,
                requires: mixin.requires,
                effects,
            },
        );
    }
    for (id, mixin) in &by_id {
        for dep in &mixin.requires {
            if !by_id.contains_key(dep) {
                bail!(
                    "named mixin {} requires unknown mixin {} in {}",
                    id,
                    dep,
                    path.display()
                );
            }
        }
    }
    if let Some(cycle) = detect_cycle_ids(by_id.keys().map(|value| value.as_str()), |id| {
        by_id
            .get(id)
            .map(|mixin| mixin.requires.clone())
            .unwrap_or_default()
    }) {
        bail!(
            "named mixin dependency cycle in {}: {}",
            path.display(),
            cycle.join(" -> ")
        );
    }
    Ok(NamedEffectMixinRegistry { by_id })
}

fn load_consumable_mixins(
    path: &Path,
    allow_missing: bool,
) -> anyhow::Result<ConsumableMixinRegistry> {
    if !path.exists() {
        if !allow_missing {
            // Mixins are optional for base content as well.
        }
        return Ok(ConsumableMixinRegistry::default());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let values: Vec<RawConsumableMixin> =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let mut by_id = HashMap::new();
    for mixin in values {
        let id = mixin.id.trim();
        if id.is_empty() {
            bail!("mixin id cannot be empty in {}", path.display());
        }
        if mixin.requires.iter().any(|value| value.trim().is_empty()) {
            bail!(
                "mixin {} has empty requires entry in {}",
                id,
                path.display()
            );
        }
        if by_id.contains_key(id) {
            bail!("duplicate mixin {} in {}", id, path.display());
        }
        by_id.insert(
            id.to_string(),
            RawConsumableMixin {
                id: id.to_string(),
                ..mixin
            },
        );
    }
    for (id, mixin) in &by_id {
        for dep in &mixin.requires {
            if !by_id.contains_key(dep) {
                bail!(
                    "mixin {} requires unknown mixin {} in {}",
                    id,
                    dep,
                    path.display()
                );
            }
        }
    }
    if let Some(cycle) = detect_cycle_ids(by_id.keys().map(|value| value.as_str()), |id| {
        by_id
            .get(id)
            .map(|mixin| mixin.requires.clone())
            .unwrap_or_default()
    }) {
        bail!(
            "consumable mixin dependency cycle in {}: {}",
            path.display(),
            cycle.join(" -> ")
        );
    }
    Ok(ConsumableMixinRegistry { by_id })
}

fn load_consumables_optional(
    path: &Path,
    allow_missing: bool,
    locale: &str,
    mixins: &ConsumableMixinRegistry,
) -> anyhow::Result<Vec<ConsumableDef>> {
    if !path.exists() {
        if allow_missing {
            return Ok(Vec::new());
        }
        bail!("missing {}", path.display());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut values: Vec<RawConsumableDef> =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let mut out = Vec::with_capacity(values.len());
    for item in values.drain(..) {
        let effects = resolve_consumable_effects(path, &item, mixins)?;
        out.push(ConsumableDef {
            id: item.id,
            name: localize_name(&item.name, &item.names, locale),
            kind: item.kind,
            hand: item.hand,
            effects,
            skip_last_consumable: item.skip_last_consumable,
        });
    }
    Ok(out)
}

fn resolve_consumable_effects(
    path: &Path,
    raw: &RawConsumableDef,
    mixins: &ConsumableMixinRegistry,
) -> anyhow::Result<Vec<EffectBlock>> {
    if raw.mixins.is_empty() {
        return Ok(raw.effects.clone());
    }

    fn visit_mixin(
        root_path: &Path,
        def: &RawConsumableDef,
        mixin_id: &str,
        mixins: &ConsumableMixinRegistry,
        visiting: &mut Vec<String>,
        visited: &mut HashSet<String>,
        resolved: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        if visited.contains(mixin_id) {
            return Ok(());
        }
        if visiting.iter().any(|value| value == mixin_id) {
            let mut cycle = visiting.clone();
            cycle.push(mixin_id.to_string());
            bail!(
                "mixin cycle for consumable {} in {}: {}",
                def.id,
                root_path.display(),
                cycle.join(" -> ")
            );
        }
        let Some(mixin) = mixins.by_id.get(mixin_id) else {
            bail!(
                "consumable {} references unknown mixin {} in {}",
                def.id,
                mixin_id,
                root_path.display()
            );
        };
        if !mixin.kinds.is_empty() && !mixin.kinds.iter().any(|kind| *kind == def.kind) {
            bail!(
                "mixin {} does not support kind {:?} for consumable {} in {}",
                mixin_id,
                def.kind,
                def.id,
                root_path.display()
            );
        }
        visiting.push(mixin_id.to_string());
        for dep in &mixin.requires {
            visit_mixin(root_path, def, dep, mixins, visiting, visited, resolved)?;
        }
        let _ = visiting.pop();
        if visited.insert(mixin_id.to_string()) {
            resolved.push(mixin_id.to_string());
        }
        Ok(())
    }

    let mut visiting = Vec::new();
    let mut visited = HashSet::new();
    let mut resolved = Vec::new();
    for mixin in &raw.mixins {
        visit_mixin(
            path,
            raw,
            mixin,
            mixins,
            &mut visiting,
            &mut visited,
            &mut resolved,
        )?;
    }

    let mut effects = Vec::new();
    for mixin_id in resolved {
        let mixin = mixins
            .by_id
            .get(&mixin_id)
            .ok_or_else(|| anyhow::anyhow!("missing mixin {}", mixin_id))?;
        effects.extend(mixin.effects.iter().cloned());
    }
    effects.extend(raw.effects.iter().cloned());
    Ok(effects)
}

trait HasEffects {
    fn effects_mut(&mut self) -> &mut Vec<JokerEffect>;
}

impl HasEffects for JokerDef {
    fn effects_mut(&mut self) -> &mut Vec<JokerEffect> {
        &mut self.effects
    }
}

impl HasEffects for TagDef {
    fn effects_mut(&mut self) -> &mut Vec<JokerEffect> {
        &mut self.effects
    }
}

impl HasEffects for BossDef {
    fn effects_mut(&mut self) -> &mut Vec<JokerEffect> {
        &mut self.effects
    }
}

fn apply_named_effect_mixins<T: HasId + HasEffects>(
    path: &Path,
    kind: NamedContentKind,
    defs: &mut [T],
    mixin_refs: &HashMap<String, Vec<String>>,
    mixins: &NamedEffectMixinRegistry,
) -> anyhow::Result<()> {
    for def in defs {
        let Some(refs) = mixin_refs.get(def.id()) else {
            continue;
        };
        if refs.is_empty() {
            continue;
        }
        let own_effects = std::mem::take(def.effects_mut());
        let effects = resolve_named_effects(path, def.id(), kind, refs, &own_effects, mixins)?;
        *def.effects_mut() = effects;
    }
    Ok(())
}

fn resolve_named_effects(
    path: &Path,
    def_id: &str,
    kind: NamedContentKind,
    refs: &[String],
    own_effects: &[JokerEffect],
    mixins: &NamedEffectMixinRegistry,
) -> anyhow::Result<Vec<JokerEffect>> {
    fn visit_mixin(
        root_path: &Path,
        def_id: &str,
        kind: NamedContentKind,
        mixin_id: &str,
        mixins: &NamedEffectMixinRegistry,
        visiting: &mut Vec<String>,
        visited: &mut HashSet<String>,
        resolved: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        if visited.contains(mixin_id) {
            return Ok(());
        }
        if visiting.iter().any(|value| value == mixin_id) {
            let mut cycle = visiting.clone();
            cycle.push(mixin_id.to_string());
            bail!(
                "named mixin cycle for {} {} in {}: {}",
                kind.label(),
                def_id,
                root_path.display(),
                cycle.join(" -> ")
            );
        }
        let Some(mixin) = mixins.by_id.get(mixin_id) else {
            bail!(
                "{} {} references unknown named mixin {} in {}",
                kind.label(),
                def_id,
                mixin_id,
                root_path.display()
            );
        };
        if !mixin.kinds.is_empty() && !mixin.kinds.contains(&kind) {
            bail!(
                "named mixin {} does not support {} {} in {}",
                mixin_id,
                kind.label(),
                def_id,
                root_path.display()
            );
        }
        visiting.push(mixin_id.to_string());
        for dep in &mixin.requires {
            visit_mixin(
                root_path, def_id, kind, dep, mixins, visiting, visited, resolved,
            )?;
        }
        let _ = visiting.pop();
        if visited.insert(mixin_id.to_string()) {
            resolved.push(mixin_id.to_string());
        }
        Ok(())
    }

    let mut visiting = Vec::new();
    let mut visited = HashSet::new();
    let mut resolved = Vec::new();
    for mixin in refs {
        visit_mixin(
            path,
            def_id,
            kind,
            mixin,
            mixins,
            &mut visiting,
            &mut visited,
            &mut resolved,
        )?;
    }

    let mut effects = Vec::new();
    for mixin_id in resolved {
        let mixin = mixins
            .by_id
            .get(&mixin_id)
            .ok_or_else(|| anyhow::anyhow!("missing named mixin {}", mixin_id))?;
        effects.extend(mixin.effects.iter().cloned());
    }
    effects.extend(own_effects.iter().cloned());
    Ok(effects)
}

fn localize_name(base: &str, names: &HashMap<String, String>, locale: &str) -> String {
    let locale = normalize_locale(Some(locale));
    if locale == "en_US" {
        return base.to_string();
    }
    for (key, value) in names {
        if normalize_locale(Some(key)) == locale {
            return value.clone();
        }
    }
    base.to_string()
}

pub fn normalize_locale(locale: Option<&str>) -> String {
    let raw = locale.unwrap_or("en_US").trim();
    if raw.is_empty() {
        return "en_US".to_string();
    }
    let lowered = raw.replace('-', "_").to_ascii_lowercase();
    match lowered.as_str() {
        "zh" | "zh_cn" | "zh_hans" | "zh_hans_cn" => "zh_CN".to_string(),
        "en" | "en_us" => "en_US".to_string(),
        _ => raw.replace('-', "_"),
    }
}

fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OverrideKind {
    Joker,
    Tag,
    Boss,
    Tarot,
    Planet,
    Spectral,
}

impl OverrideKind {
    fn from_str(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "joker" | "jokers" => Some(Self::Joker),
            "tag" | "tags" => Some(Self::Tag),
            "boss" | "bosses" => Some(Self::Boss),
            "tarot" | "tarots" => Some(Self::Tarot),
            "planet" | "planets" => Some(Self::Planet),
            "spectral" | "spectrals" => Some(Self::Spectral),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Joker => "joker",
            Self::Tag => "tag",
            Self::Boss => "boss",
            Self::Tarot => "tarot",
            Self::Planet => "planet",
            Self::Spectral => "spectral",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OverrideKey {
    kind: OverrideKind,
    id: String,
}

fn parse_overrides(list: &[String]) -> anyhow::Result<HashSet<OverrideKey>> {
    let mut out = HashSet::new();
    for entry in list {
        let (kind, id) = entry
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("invalid override {}", entry))?;
        let kind = OverrideKind::from_str(kind)
            .ok_or_else(|| anyhow::anyhow!("invalid override kind {}", entry))?;
        let id = id.trim();
        if id.is_empty() {
            bail!("invalid override {}", entry);
        }
        out.insert(OverrideKey {
            kind,
            id: id.to_string(),
        });
    }
    Ok(out)
}

struct ContentOrigins {
    jokers: HashMap<String, String>,
    tags: HashMap<String, String>,
    bosses: HashMap<String, String>,
    tarots: HashMap<String, String>,
    planets: HashMap<String, String>,
    spectrals: HashMap<String, String>,
}

impl ContentOrigins {
    fn from_content(content: &Content) -> Self {
        Self {
            jokers: content
                .jokers
                .iter()
                .map(|item| (item.id.clone(), "base".to_string()))
                .collect(),
            tags: content
                .tags
                .iter()
                .map(|item| (item.id.clone(), "base".to_string()))
                .collect(),
            bosses: content
                .bosses
                .iter()
                .map(|item| (item.id.clone(), "base".to_string()))
                .collect(),
            tarots: content
                .tarots
                .iter()
                .map(|item| (item.id.clone(), "base".to_string()))
                .collect(),
            planets: content
                .planets
                .iter()
                .map(|item| (item.id.clone(), "base".to_string()))
                .collect(),
            spectrals: content
                .spectrals
                .iter()
                .map(|item| (item.id.clone(), "base".to_string()))
                .collect(),
        }
    }

    fn origin(&self, key: &OverrideKey) -> Option<&str> {
        match key.kind {
            OverrideKind::Joker => self.jokers.get(&key.id).map(|value| value.as_str()),
            OverrideKind::Tag => self.tags.get(&key.id).map(|value| value.as_str()),
            OverrideKind::Boss => self.bosses.get(&key.id).map(|value| value.as_str()),
            OverrideKind::Tarot => self.tarots.get(&key.id).map(|value| value.as_str()),
            OverrideKind::Planet => self.planets.get(&key.id).map(|value| value.as_str()),
            OverrideKind::Spectral => self.spectrals.get(&key.id).map(|value| value.as_str()),
        }
    }
}

fn validate_overrides(
    overrides: &HashSet<OverrideKey>,
    origins: &ContentOrigins,
    mod_id: &str,
) -> anyhow::Result<()> {
    for key in overrides {
        let Some(origin) = origins.origin(key) else {
            bail!(
                "mod {} overrides missing {} {}",
                mod_id,
                key.kind.label(),
                key.id
            );
        };
        if origin != "base" {
            bail!(
                "mod {} cannot override {} {} from {}",
                mod_id,
                key.kind.label(),
                key.id,
                origin
            );
        }
    }
    Ok(())
}

trait HasId {
    fn id(&self) -> &str;
}

impl HasId for JokerDef {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for TagDef {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for BossDef {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for ConsumableDef {
    fn id(&self) -> &str {
        &self.id
    }
}

fn merge_content(
    base: &mut Content,
    origins: &mut ContentOrigins,
    extra: Content,
    mod_id: &str,
    overrides: &HashSet<OverrideKey>,
) -> anyhow::Result<()> {
    for item in extra.jokers {
        merge_list(
            &mut base.jokers,
            &mut origins.jokers,
            item,
            OverrideKind::Joker,
            mod_id,
            overrides,
        )?;
    }
    for item in extra.tags {
        merge_list(
            &mut base.tags,
            &mut origins.tags,
            item,
            OverrideKind::Tag,
            mod_id,
            overrides,
        )?;
    }
    for item in extra.bosses {
        merge_list(
            &mut base.bosses,
            &mut origins.bosses,
            item,
            OverrideKind::Boss,
            mod_id,
            overrides,
        )?;
    }
    for item in extra.tarots {
        merge_list(
            &mut base.tarots,
            &mut origins.tarots,
            item,
            OverrideKind::Tarot,
            mod_id,
            overrides,
        )?;
    }
    for item in extra.planets {
        merge_list(
            &mut base.planets,
            &mut origins.planets,
            item,
            OverrideKind::Planet,
            mod_id,
            overrides,
        )?;
    }
    for item in extra.spectrals {
        merge_list(
            &mut base.spectrals,
            &mut origins.spectrals,
            item,
            OverrideKind::Spectral,
            mod_id,
            overrides,
        )?;
    }
    Ok(())
}

fn merge_list<T: HasId>(
    list: &mut Vec<T>,
    origins: &mut HashMap<String, String>,
    item: T,
    kind: OverrideKind,
    mod_id: &str,
    overrides: &HashSet<OverrideKey>,
) -> anyhow::Result<()> {
    let id = item.id().to_string();
    if let Some(origin) = origins.get(&id) {
        let allowed = overrides.contains(&OverrideKey {
            kind,
            id: id.clone(),
        });
        if allowed && origin == "base" {
            if let Some(index) = list.iter().position(|entry| entry.id() == id) {
                list[index] = item;
            } else {
                list.push(item);
            }
            origins.insert(id, mod_id.to_string());
            return Ok(());
        }
        bail!(
            "mod {} duplicate {} {} from {}",
            mod_id,
            kind.label(),
            id,
            origin
        );
    }
    list.push(item);
    origins.insert(id, mod_id.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulatro_core::{Action, ActionOp, ActivationType, Condition, EffectOp, Expr};
    use std::collections::BTreeMap;

    fn block_add_money(value: i64) -> EffectBlock {
        EffectBlock {
            trigger: ActivationType::OnUse,
            conditions: vec![Condition::Always],
            effects: vec![EffectOp::AddMoney(value)],
        }
    }

    fn joker_add_money(value: f64) -> JokerEffect {
        JokerEffect {
            trigger: ActivationType::OnUse,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(value),
            }],
        }
    }

    fn first_joker_money(effect: &JokerEffect) -> Option<f64> {
        let action = effect.actions.first()?;
        if action.op != ActionOpKind::Builtin(ActionOp::AddMoney) {
            return None;
        }
        if let Expr::Number(value) = action.value.clone() {
            return Some(value);
        }
        None
    }

    fn raw_consumable(kind: ConsumableKind, mixins: &[&str], own_money: i64) -> RawConsumableDef {
        RawConsumableDef {
            id: "sample".to_string(),
            name: "Sample".to_string(),
            kind,
            hand: None,
            effects: vec![block_add_money(own_money)],
            mixins: mixins.iter().map(|value| value.to_string()).collect(),
            names: HashMap::new(),
            skip_last_consumable: false,
        }
    }

    #[test]
    fn resolves_consumable_mixins_with_dependencies() {
        let mut registry = ConsumableMixinRegistry::default();
        registry.by_id.insert(
            "base".to_string(),
            RawConsumableMixin {
                id: "base".to_string(),
                kinds: Vec::new(),
                requires: Vec::new(),
                effects: vec![block_add_money(1)],
            },
        );
        registry.by_id.insert(
            "extra".to_string(),
            RawConsumableMixin {
                id: "extra".to_string(),
                kinds: Vec::new(),
                requires: vec!["base".to_string()],
                effects: vec![block_add_money(2)],
            },
        );

        let raw = raw_consumable(ConsumableKind::Tarot, &["extra"], 3);
        let effects =
            resolve_consumable_effects(Path::new("test"), &raw, &registry).expect("resolve");
        assert_eq!(effects.len(), 3);
        match effects[0].effects.first() {
            Some(EffectOp::AddMoney(value)) => assert_eq!(*value, 1),
            other => panic!("unexpected first effect: {:?}", other),
        }
        match effects[1].effects.first() {
            Some(EffectOp::AddMoney(value)) => assert_eq!(*value, 2),
            other => panic!("unexpected second effect: {:?}", other),
        }
        match effects[2].effects.first() {
            Some(EffectOp::AddMoney(value)) => assert_eq!(*value, 3),
            other => panic!("unexpected third effect: {:?}", other),
        }
    }

    #[test]
    fn rejects_kind_mismatch_mixin() {
        let mut registry = ConsumableMixinRegistry::default();
        registry.by_id.insert(
            "tarot_only".to_string(),
            RawConsumableMixin {
                id: "tarot_only".to_string(),
                kinds: vec![ConsumableKind::Tarot],
                requires: Vec::new(),
                effects: vec![block_add_money(1)],
            },
        );
        let raw = raw_consumable(ConsumableKind::Planet, &["tarot_only"], 3);
        let err = resolve_consumable_effects(Path::new("test"), &raw, &registry)
            .expect_err("kind mismatch must fail");
        assert!(err.to_string().contains("does not support kind"));
    }

    #[test]
    fn rejects_mixin_cycle() {
        let mut registry = ConsumableMixinRegistry::default();
        registry.by_id.insert(
            "a".to_string(),
            RawConsumableMixin {
                id: "a".to_string(),
                kinds: Vec::new(),
                requires: vec!["b".to_string()],
                effects: vec![block_add_money(1)],
            },
        );
        registry.by_id.insert(
            "b".to_string(),
            RawConsumableMixin {
                id: "b".to_string(),
                kinds: Vec::new(),
                requires: vec!["a".to_string()],
                effects: vec![block_add_money(2)],
            },
        );
        let raw = raw_consumable(ConsumableKind::Spectral, &["a"], 3);
        let err = resolve_consumable_effects(Path::new("test"), &raw, &registry)
            .expect_err("cycle must fail");
        assert!(err.to_string().contains("mixin cycle"));
    }

    #[test]
    fn resolves_named_mixins_with_dependencies() {
        let mut registry = NamedEffectMixinRegistry::default();
        registry.by_id.insert(
            "base".to_string(),
            RawNamedEffectMixin {
                kinds: Vec::new(),
                requires: Vec::new(),
                effects: vec![joker_add_money(1.0)],
            },
        );
        registry.by_id.insert(
            "extra".to_string(),
            RawNamedEffectMixin {
                kinds: vec![NamedContentKind::Joker],
                requires: vec!["base".to_string()],
                effects: vec![joker_add_money(2.0)],
            },
        );

        let effects = resolve_named_effects(
            Path::new("test"),
            "sample_joker",
            NamedContentKind::Joker,
            &["extra".to_string()],
            &[joker_add_money(3.0)],
            &registry,
        )
        .expect("resolve");
        assert_eq!(effects.len(), 3);
        assert_eq!(first_joker_money(&effects[0]), Some(1.0));
        assert_eq!(first_joker_money(&effects[1]), Some(2.0));
        assert_eq!(first_joker_money(&effects[2]), Some(3.0));
    }

    #[test]
    fn rejects_named_mixin_kind_mismatch() {
        let mut registry = NamedEffectMixinRegistry::default();
        registry.by_id.insert(
            "tag_only".to_string(),
            RawNamedEffectMixin {
                kinds: vec![NamedContentKind::Tag],
                requires: Vec::new(),
                effects: vec![joker_add_money(1.0)],
            },
        );
        let err = resolve_named_effects(
            Path::new("test"),
            "sample_joker",
            NamedContentKind::Joker,
            &["tag_only".to_string()],
            &[joker_add_money(3.0)],
            &registry,
        )
        .expect_err("kind mismatch must fail");
        assert!(err.to_string().contains("does not support"));
    }

    #[test]
    fn rejects_named_mixin_cycle() {
        let mut registry = NamedEffectMixinRegistry::default();
        registry.by_id.insert(
            "a".to_string(),
            RawNamedEffectMixin {
                kinds: Vec::new(),
                requires: vec!["b".to_string()],
                effects: vec![joker_add_money(1.0)],
            },
        );
        registry.by_id.insert(
            "b".to_string(),
            RawNamedEffectMixin {
                kinds: Vec::new(),
                requires: vec!["a".to_string()],
                effects: vec![joker_add_money(2.0)],
            },
        );
        let err = resolve_named_effects(
            Path::new("test"),
            "sample_tag",
            NamedContentKind::Tag,
            &["a".to_string()],
            &[joker_add_money(3.0)],
            &registry,
        )
        .expect_err("cycle must fail");
        assert!(err.to_string().contains("named mixin cycle"));
    }

    #[test]
    fn detects_dependency_cycle_ids() {
        let mut graph = BTreeMap::new();
        graph.insert("a".to_string(), vec!["b".to_string()]);
        graph.insert("b".to_string(), vec!["c".to_string()]);
        graph.insert("c".to_string(), vec!["a".to_string()]);
        let cycle = detect_cycle_ids(graph.keys().map(|value| value.as_str()), |id| {
            graph.get(id).cloned().unwrap_or_default()
        })
        .expect("cycle must be found");
        assert!(cycle.len() >= 3);
        assert!(cycle.first() == cycle.last());
    }

    #[test]
    fn allows_acyclic_dependency_ids() {
        let mut graph = BTreeMap::new();
        graph.insert("base".to_string(), Vec::new());
        graph.insert("child".to_string(), vec!["base".to_string()]);
        let cycle = detect_cycle_ids(graph.keys().map(|value| value.as_str()), |id| {
            graph.get(id).cloned().unwrap_or_default()
        });
        assert!(cycle.is_none());
    }
}
