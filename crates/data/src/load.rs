use crate::joker_dsl::{load_bosses_dsl, load_jokers_dsl, load_tags_dsl};
use crate::schema::{
    AnteRule, BlindRule, BossDef, ConsumableDef, Content, ContentPack, EconomyRule, GameConfig,
    HandRule, JokerDef, RankRule, ShopRule, TagDef,
};
use anyhow::{bail, Context};
use rulatro_modding::{FileSystemModLoader, LoadedMod};
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

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

    Ok(GameConfig {
        hands,
        ranks,
        blinds,
        antes,
        economy,
        shop,
    })
}

pub fn load_content(dir: &Path) -> anyhow::Result<Content> {
    let base = dir.join("content");
    load_content_dir(&base, false)
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

pub fn load_content_with_mods(
    assets_dir: &Path,
    mods_dir: &Path,
) -> anyhow::Result<ModLoadReport> {
    let mut content = load_content(assets_dir)?;
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
        let mod_content = load_content_dir(&item.root.join(&content_spec.root), true)
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

fn load_content_dir(base: &Path, allow_missing: bool) -> anyhow::Result<Content> {
    let jokers_path = base.join("jokers.dsl");
    let jokers = if jokers_path.exists() {
        load_jokers_dsl(&jokers_path).with_context(|| format!("parse {}", jokers_path.display()))?
    } else if allow_missing {
        Vec::new()
    } else {
        bail!("missing {}", jokers_path.display());
    };

    let bosses_path = base.join("bosses.dsl");
    let bosses = if bosses_path.exists() {
        load_bosses_dsl(&bosses_path).with_context(|| format!("parse {}", bosses_path.display()))?
    } else if allow_missing {
        Vec::new()
    } else {
        bail!("missing {}", bosses_path.display());
    };

    let tags_path = base.join("tags.dsl");
    let tags = if tags_path.exists() {
        load_tags_dsl(&tags_path).with_context(|| format!("parse {}", tags_path.display()))?
    } else if allow_missing {
        Vec::new()
    } else {
        bail!("missing {}", tags_path.display());
    };

    let tarots: Vec<ConsumableDef> = load_json_optional(&base.join("tarots.json"), allow_missing)?;
    let planets: Vec<ConsumableDef> = load_json_optional(&base.join("planets.json"), allow_missing)?;
    let spectrals: Vec<ConsumableDef> =
        load_json_optional(&base.join("spectrals.json"), allow_missing)?;

    Ok(Content {
        jokers,
        bosses,
        tags,
        tarots,
        planets,
        spectrals,
    })
}

fn load_json_optional<T: DeserializeOwned>(
    path: &Path,
    allow_missing: bool,
) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        if allow_missing {
            return Ok(Vec::new());
        }
        bail!("missing {}", path.display());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(value)
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
