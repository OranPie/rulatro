use crate::persistence::{
    compute_content_signature, default_state_path, load_state_file, save_state_file, SavedAction,
};
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rulatro_core::{
    voucher_by_id, BlindKind, BlindOutcome, Card, ConsumableKind, Edition, EffectBlock, EffectOp,
    Enhancement, Event, EventBus, PackOpen, PackOption, Phase, RankFilter, RuleEffect, RunError,
    RunState, ScoreBreakdown, Seal, ShopOfferRef, ShopPurchase,
};
use rulatro_data::{load_content_with_mods_locale, load_game_config, normalize_locale};
use rulatro_modding::ModManager;
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

pub const DEFAULT_RUN_SEED: u64 = 0xC0FFEE;
const MAX_EVENT_LOG: usize = 200;
const MAX_TRACE_LINES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLocale {
    EnUs,
    ZhCn,
}

impl UiLocale {
    pub fn from_opt(value: Option<&str>) -> Self {
        let normalized = normalize_locale(value);
        if normalized == "zh_CN" {
            Self::ZhCn
        } else {
            Self::EnUs
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::EnUs => "en_US",
            Self::ZhCn => "zh_CN",
        }
    }

    pub fn text<'a>(self, en: &'a str, zh: &'a str) -> &'a str {
        if matches!(self, Self::ZhCn) {
            zh
        } else {
            en
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Hand,
    Shop,
    Inventory,
    Events,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathPromptMode {
    Save,
    Load,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryRowKind {
    Joker(usize),
    Consumable(usize),
}

#[derive(Debug, Clone)]
pub struct InventoryRow {
    pub kind: InventoryRowKind,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct ShopRow {
    pub offer: ShopOfferRef,
    pub label: String,
}

pub struct App {
    pub locale: UiLocale,
    pub seed: u64,
    pub content_signature: String,
    pub recorded_actions: Vec<SavedAction>,
    pub run: RunState,
    pub events: EventBus,
    pub open_pack: Option<PackOpen>,
    pub focus: FocusPane,
    pub hand_cursor: usize,
    pub shop_cursor: usize,
    pub inventory_cursor: usize,
    pub pack_cursor: usize,
    pub selected_hand: BTreeSet<usize>,
    pub selected_pack: BTreeSet<usize>,
    pub event_log: VecDeque<String>,
    pub status_line: String,
    pub show_help: bool,
    pub path_prompt_mode: Option<PathPromptMode>,
    pub path_prompt_input: String,
    pub should_quit: bool,
}

fn build_run_with_seed(locale: UiLocale, seed: u64) -> Result<(RunState, String, Vec<String>)> {
    let config = load_game_config(Path::new("assets")).context("load config")?;
    let modded =
        load_content_with_mods_locale(Path::new("assets"), Path::new("mods"), Some(locale.code()))
            .context("load content")?;
    let mut runtime = ModManager::new();
    runtime
        .load_mods(&modded.mods)
        .map_err(|err| anyhow::anyhow!(err.to_string()))
        .context("load mod runtime")?;
    let mut run = RunState::new(config, modded.content, seed);
    run.set_mod_runtime(Some(Box::new(runtime)));
    let content_signature =
        compute_content_signature(locale.code()).unwrap_or_else(|_| String::new());
    let mut startup_notes = Vec::new();
    if !modded.mods.is_empty() {
        startup_notes.push(format!(
            "{}: {}",
            locale.text("mods loaded", "已加载模组"),
            modded.mods.len()
        ));
        for item in &modded.mods {
            startup_notes.push(format!(
                "  - {} {}",
                item.manifest.meta.id, item.manifest.meta.version
            ));
        }
    }
    for warning in &modded.warnings {
        startup_notes.push(format!("{}: {warning}", locale.text("warning", "警告")));
    }
    Ok((run, content_signature, startup_notes))
}

impl App {
    pub fn bootstrap(locale: UiLocale, seed: u64) -> Result<Self> {
        let (run, content_signature, startup_notes) = build_run_with_seed(locale, seed)?;

        let mut app = Self {
            locale,
            seed,
            content_signature,
            recorded_actions: Vec::new(),
            run,
            events: EventBus::default(),
            open_pack: None,
            focus: FocusPane::Hand,
            hand_cursor: 0,
            shop_cursor: 0,
            inventory_cursor: 0,
            pack_cursor: 0,
            selected_hand: BTreeSet::new(),
            selected_pack: BTreeSet::new(),
            event_log: VecDeque::new(),
            status_line: locale.text("ready", "已就绪").to_string(),
            show_help: false,
            path_prompt_mode: None,
            path_prompt_input: String::new(),
            should_quit: false,
        };

        app.run
            .start_blind(1, BlindKind::Small, &mut app.events)
            .map_err(|err| anyhow::anyhow!(err.to_string()))
            .context("start blind")?;

        for line in startup_notes {
            app.push_event_line(line);
        }

        app.flush_events();
        app.normalize_cursors();
        Ok(app)
    }

    pub fn on_tick(&mut self) {}

    pub fn focus_label(&self, pane: FocusPane) -> &'static str {
        match pane {
            FocusPane::Hand => self.locale.text("Hand", "手牌"),
            FocusPane::Shop => {
                if self.open_pack.is_some() {
                    self.locale.text("Pack", "卡包")
                } else {
                    self.locale.text("Shop", "商店")
                }
            }
            FocusPane::Inventory => self.locale.text("Inventory", "库存"),
            FocusPane::Events => self.locale.text("Events", "事件"),
        }
    }

    pub fn phase_label(&self, phase: Phase) -> &'static str {
        phase_label(self.locale, phase)
    }

    pub fn blind_label(&self, blind: BlindKind) -> &'static str {
        blind_label(self.locale, blind)
    }

    pub fn blind_outcome_label(&self) -> &'static str {
        match self.run.blind_outcome() {
            Some(BlindOutcome::Cleared) => self.locale.text("Cleared", "已通过"),
            Some(BlindOutcome::Failed) => self.locale.text("Failed", "失败"),
            None => self.locale.text("In Progress", "进行中"),
        }
    }

    pub fn boss_status_label(&self) -> String {
        if self.run.state.blind != BlindKind::Boss {
            return self.locale.text("none", "无").to_string();
        }
        if self.run.boss_effects_disabled() {
            return self.locale.text("disabled", "已禁用").to_string();
        }
        match self.run.current_boss() {
            Some(boss) => format!("{} ({})", boss.name, boss.id),
            None => self.locale.text("pending", "待定").to_string(),
        }
    }

    pub fn boss_effect_lines(&self, max_lines: usize) -> Vec<String> {
        if self.run.state.blind != BlindKind::Boss || self.run.boss_effects_disabled() {
            return Vec::new();
        }
        let effects = self.run.current_boss_effect_summaries();
        if effects.is_empty() {
            return Vec::new();
        }
        let total = effects.len();
        let mut lines = effects.into_iter().take(max_lines).collect::<Vec<_>>();
        if total > max_lines {
            lines.push(self.locale.text("...", "...").to_string());
        }
        lines
    }

    pub fn active_voucher_lines(&self, max_lines: usize) -> Vec<String> {
        let mut rows = self
            .run
            .active_voucher_summaries(matches!(self.locale, UiLocale::ZhCn));
        if rows.is_empty() {
            return rows;
        }
        if rows.len() > max_lines {
            rows.truncate(max_lines);
            rows.push(self.locale.text("...", "...").to_string());
        }
        rows
    }

    pub fn cycle_focus(&mut self, forward: bool) {
        self.focus = match (self.focus, forward) {
            (FocusPane::Hand, true) => FocusPane::Shop,
            (FocusPane::Shop, true) => FocusPane::Inventory,
            (FocusPane::Inventory, true) => FocusPane::Events,
            (FocusPane::Events, true) => FocusPane::Hand,
            (FocusPane::Hand, false) => FocusPane::Events,
            (FocusPane::Shop, false) => FocusPane::Hand,
            (FocusPane::Inventory, false) => FocusPane::Shop,
            (FocusPane::Events, false) => FocusPane::Inventory,
        };
    }

    pub fn move_cursor(&mut self, down: bool) {
        match self.focus {
            FocusPane::Hand => {
                let len = self.hand_len();
                move_index(&mut self.hand_cursor, len, down);
            }
            FocusPane::Shop => {
                if self.open_pack.is_some() {
                    let len = self.pack_len();
                    move_index(&mut self.pack_cursor, len, down);
                } else {
                    let len = self.shop_rows().len();
                    move_index(&mut self.shop_cursor, len, down);
                }
            }
            FocusPane::Inventory => {
                let len = self.inventory_rows().len();
                move_index(&mut self.inventory_cursor, len, down);
            }
            FocusPane::Events => {}
        }
    }

    pub fn select_number(&mut self, index: usize) {
        match self.focus {
            FocusPane::Hand => {
                let len = self.hand_len();
                if len == 0 {
                    self.push_status(self.locale.text("hand is empty", "手牌为空"));
                    return;
                }
                if index >= len {
                    self.push_status(format!(
                        "{} {} {}",
                        self.locale.text("index out of range:", "索引超出范围："),
                        index,
                        self.locale.text("(hand)", "（手牌）")
                    ));
                    return;
                }
                self.hand_cursor = index;
                toggle_set(&mut self.selected_hand, index);
                self.push_status(format!(
                    "{} {}",
                    self.locale.text("hand select", "手牌选择"),
                    index
                ));
            }
            FocusPane::Shop => {
                if self.open_pack.is_some() {
                    let len = self.pack_len();
                    if len == 0 {
                        self.push_status(
                            self.locale
                                .text("pack has no options", "当前卡包没有可选项"),
                        );
                        return;
                    }
                    if index >= len {
                        self.push_status(format!(
                            "{} {} {}",
                            self.locale.text("index out of range:", "索引超出范围："),
                            index,
                            self.locale.text("(pack)", "（卡包）")
                        ));
                        return;
                    }
                    self.pack_cursor = index;
                    toggle_set(&mut self.selected_pack, index);
                    self.push_status(format!(
                        "{} {}",
                        self.locale.text("pack select", "卡包选择"),
                        index
                    ));
                    return;
                }
                let len = self.shop_rows().len();
                if len == 0 {
                    self.push_status(self.locale.text("shop has no offers", "商店没有商品"));
                    return;
                }
                if index >= len {
                    self.push_status(format!(
                        "{} {} {}",
                        self.locale.text("index out of range:", "索引超出范围："),
                        index,
                        self.locale.text("(shop)", "（商店）")
                    ));
                    return;
                }
                self.shop_cursor = index;
                self.push_status(format!(
                    "{} {}",
                    self.locale.text("shop focus", "商店焦点"),
                    index
                ));
            }
            FocusPane::Inventory => {
                let len = self.inventory_rows().len();
                if len == 0 {
                    self.push_status(self.locale.text("inventory is empty", "库存为空"));
                    return;
                }
                if index >= len {
                    self.push_status(format!(
                        "{} {} {}",
                        self.locale.text("index out of range:", "索引超出范围："),
                        index,
                        self.locale.text("(inventory)", "（库存）")
                    ));
                    return;
                }
                self.inventory_cursor = index;
                self.push_status(format!(
                    "{} {}",
                    self.locale.text("inventory focus", "库存焦点"),
                    index
                ));
            }
            FocusPane::Events => {
                self.push_status(self.locale.text(
                    "number select unavailable in events",
                    "事件面板不支持数字选择",
                ));
            }
        }
    }

    pub fn toggle_focused_selection(&mut self) {
        match self.focus {
            FocusPane::Hand => {
                let len = self.hand_len();
                if len == 0 {
                    return;
                }
                let idx = self.hand_cursor.min(len - 1);
                toggle_set(&mut self.selected_hand, idx);
            }
            FocusPane::Shop => {
                let len = self.pack_len();
                if self.open_pack.is_none() || len == 0 {
                    return;
                }
                let idx = self.pack_cursor.min(len - 1);
                toggle_set(&mut self.selected_pack, idx);
            }
            FocusPane::Inventory | FocusPane::Events => {}
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_hand.clear();
        self.selected_pack.clear();
    }

    pub fn next_hint(&self) -> String {
        if self.open_pack.is_some() {
            return self
                .locale
                .text("select pack options", "选择卡包选项")
                .to_string();
        }
        if let Some(outcome) = self.run.blind_outcome() {
            return match outcome {
                BlindOutcome::Cleared => {
                    if self.run.state.phase == Phase::Shop {
                        self.locale
                            .text("buy/reroll/leave", "购买/刷新/离开")
                            .to_string()
                    } else {
                        self.locale
                            .text("enter shop or next", "进商店或下一盲注")
                            .to_string()
                    }
                }
                BlindOutcome::Failed => self
                    .locale
                    .text("start next blind", "开始下一盲注")
                    .to_string(),
            };
        }
        match self.run.state.phase {
            Phase::Deal => self.locale.text("deal", "发牌").to_string(),
            Phase::Play => self.locale.text("play/discard", "出牌/弃牌").to_string(),
            Phase::Shop => self
                .locale
                .text("buy/reroll/leave", "购买/刷新/离开")
                .to_string(),
            Phase::Setup | Phase::Score | Phase::Cleanup => {
                self.locale.text("next blind", "下一盲注").to_string()
            }
        }
    }

    pub fn hand_len(&self) -> usize {
        self.run.hand.len()
    }

    pub fn pack_len(&self) -> usize {
        self.open_pack
            .as_ref()
            .map(|open| open.options.len())
            .unwrap_or(0)
    }

    pub fn shop_rows(&self) -> Vec<ShopRow> {
        let mut rows = Vec::new();
        let Some(shop) = self.run.shop.as_ref() else {
            return rows;
        };
        for (idx, card) in shop.cards.iter().enumerate() {
            let item_name = match card.kind {
                rulatro_core::ShopCardKind::Joker => self.find_joker_name(&card.item_id),
                rulatro_core::ShopCardKind::Tarot => {
                    self.find_consumable_name(ConsumableKind::Tarot, &card.item_id)
                }
                rulatro_core::ShopCardKind::Planet => {
                    self.find_consumable_name(ConsumableKind::Planet, &card.item_id)
                }
            };
            let rarity = card
                .rarity
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "-".to_string());
            let edition = card
                .edition
                .map(edition_short)
                .unwrap_or_else(|| "-".to_string());
            let effect = match card.kind {
                rulatro_core::ShopCardKind::Joker => String::new(),
                rulatro_core::ShopCardKind::Tarot => {
                    self.consumable_effect_summary(ConsumableKind::Tarot, &card.item_id, 2)
                }
                rulatro_core::ShopCardKind::Planet => {
                    self.consumable_effect_summary(ConsumableKind::Planet, &card.item_id, 2)
                }
            };
            rows.push(ShopRow {
                offer: ShopOfferRef::Card(idx),
                label: format!(
                    "C{idx} {} {} ({}) {} ${} {} {} {} {}{}",
                    shop_card_kind_label(self.locale, card.kind),
                    card.item_id,
                    item_name,
                    self.locale.text("price", "价格"),
                    card.price,
                    self.locale.text("rarity", "稀有度"),
                    rarity,
                    self.locale.text("edition", "版本"),
                    edition,
                    if effect.is_empty() {
                        String::new()
                    } else {
                        format!(" {} {}", self.locale.text("effect", "效果"), effect)
                    }
                ),
            });
        }
        for (idx, pack) in shop.packs.iter().enumerate() {
            rows.push(ShopRow {
                offer: ShopOfferRef::Pack(idx),
                label: format!(
                    "P{idx} {:?}/{:?} {}:{} {}:{} ${}",
                    pack.kind,
                    pack.size,
                    self.locale.text("options", "选项"),
                    pack.options,
                    self.locale.text("pick", "可选"),
                    pack.picks,
                    pack.price
                ),
            });
        }
        for (idx, offer) in shop.voucher_offers.iter().enumerate() {
            let (name, effect) = if let Some(def) = voucher_by_id(&offer.id) {
                (
                    def.name(matches!(self.locale, UiLocale::ZhCn)).to_string(),
                    def.effect_text(matches!(self.locale, UiLocale::ZhCn))
                        .to_string(),
                )
            } else {
                (offer.id.clone(), String::new())
            };
            rows.push(ShopRow {
                offer: ShopOfferRef::Voucher(idx),
                label: format!(
                    "V{idx} {} ({}) ${} {}",
                    self.locale.text("voucher", "优惠券"),
                    name,
                    self.run.config.shop.prices.voucher,
                    effect
                ),
            });
        }
        rows
    }

    pub fn inventory_rows(&self) -> Vec<InventoryRow> {
        let mut rows = Vec::new();
        for (idx, joker) in self.run.inventory.jokers.iter().enumerate() {
            let edition = joker
                .edition
                .map(edition_short)
                .unwrap_or_else(|| "-".to_string());
            rows.push(InventoryRow {
                kind: InventoryRowKind::Joker(idx),
                label: format!(
                    "J{idx} {} ({}) {} {:?} {} {}",
                    joker.id,
                    self.find_joker_name(&joker.id),
                    self.locale.text("rarity", "稀有度"),
                    joker.rarity,
                    self.locale.text("edition", "版本"),
                    edition
                ),
            });
        }
        for (idx, item) in self.run.inventory.consumables.iter().enumerate() {
            let edition = item
                .edition
                .map(edition_short)
                .unwrap_or_else(|| "-".to_string());
            let effect = self.consumable_effect_summary(item.kind, &item.id, 2);
            rows.push(InventoryRow {
                kind: InventoryRowKind::Consumable(idx),
                label: format!(
                    "C{idx} {} ({}) {} {} {} {}{}",
                    item.id,
                    self.find_consumable_name(item.kind, &item.id),
                    self.locale.text("type", "类型"),
                    consumable_kind_label(self.locale, item.kind),
                    self.locale.text("edition", "版本"),
                    edition,
                    if effect.is_empty() {
                        String::new()
                    } else {
                        format!(" {} {}", self.locale.text("effect", "效果"), effect)
                    }
                ),
            });
        }
        rows
    }

    pub fn current_inventory_kind(&self) -> Option<InventoryRowKind> {
        let rows = self.inventory_rows();
        if rows.is_empty() {
            None
        } else {
            rows.get(self.inventory_cursor.min(rows.len() - 1))
                .map(|row| row.kind)
        }
    }

    pub fn current_shop_offer(&self) -> Option<ShopOfferRef> {
        let rows = self.shop_rows();
        if rows.is_empty() {
            None
        } else {
            rows.get(self.shop_cursor.min(rows.len() - 1))
                .map(|row| row.offer)
        }
    }

    pub fn selected_hand_indices(&self) -> Vec<usize> {
        selected_or_cursor(&self.selected_hand, self.hand_cursor, self.hand_len())
    }

    pub fn explicit_selected_hand_indices(&self) -> Vec<usize> {
        self.selected_hand
            .iter()
            .copied()
            .filter(|idx| *idx < self.hand_len())
            .collect()
    }

    pub fn selected_pack_indices(&self) -> Vec<usize> {
        selected_or_cursor(&self.selected_pack, self.pack_cursor, self.pack_len())
    }

    pub fn open_save_prompt(&mut self) {
        self.path_prompt_mode = Some(PathPromptMode::Save);
        self.path_prompt_input.clear();
    }

    pub fn open_load_prompt(&mut self) {
        self.path_prompt_mode = Some(PathPromptMode::Load);
        self.path_prompt_input.clear();
    }

    pub fn prompt_default_path_hint(&self) -> String {
        default_state_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn handle_path_prompt_key(&mut self, key: KeyEvent) -> bool {
        let Some(mode) = self.path_prompt_mode else {
            return false;
        };
        match key.code {
            KeyCode::Esc => {
                self.path_prompt_mode = None;
                self.path_prompt_input.clear();
                self.push_status(self.locale.text("path prompt cancelled", "已取消路径输入"));
            }
            KeyCode::Enter => {
                let resolved =
                    resolve_prompt_path(self.path_prompt_input.trim(), default_state_path());
                self.path_prompt_mode = None;
                self.path_prompt_input.clear();
                let Ok(path) = resolved else {
                    self.push_status(self.locale.text("save path unavailable", "保存路径不可用"));
                    return true;
                };
                match mode {
                    PathPromptMode::Save => self.save_to_path(path),
                    PathPromptMode::Load => self.load_from_path(path),
                }
            }
            KeyCode::Backspace => {
                self.path_prompt_input.pop();
            }
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    self.path_prompt_input.push(ch);
                }
            }
            _ => {}
        }
        true
    }

    fn save_to_path(&mut self, path: PathBuf) {
        match save_state_file(
            self.locale.code(),
            self.seed,
            &self.content_signature,
            &self.recorded_actions,
            &path,
        ) {
            Ok(_) => self.push_status(format!(
                "{} {} {} {}",
                self.locale.text("saved", "已保存"),
                self.recorded_actions.len(),
                self.locale.text("actions to", "条动作到"),
                path.display()
            )),
            Err(err) => self.push_status(format!(
                "{}: {err}",
                self.locale.text("save failed", "保存失败")
            )),
        }
    }

    fn load_from_path(&mut self, path: PathBuf) {
        let saved = match load_state_file(&path) {
            Ok(saved) => saved,
            Err(err) => {
                self.push_status(format!(
                    "{}: {err}",
                    self.locale.text("load failed", "读取失败")
                ));
                return;
            }
        };
        let (mut restored_run, restored_signature, _notes) =
            match build_run_with_seed(self.locale, saved.seed) {
                Ok(bundle) => bundle,
                Err(err) => {
                    self.push_status(format!(
                        "{}: {err}",
                        self.locale.text("load failed", "读取失败")
                    ));
                    return;
                }
            };
        if !saved.content_signature.is_empty() && saved.content_signature != restored_signature {
            self.push_status(format!(
                "{}: {} ({}={} {}={})",
                self.locale.text("load failed", "读取失败"),
                self.locale
                    .text("content signature mismatch", "内容签名不一致"),
                self.locale.text("saved", "存档"),
                saved.content_signature,
                self.locale.text("current", "当前"),
                restored_signature
            ));
            return;
        }
        let mut restored_events = EventBus::default();
        if let Err(err) = restored_run.start_blind(1, BlindKind::Small, &mut restored_events) {
            self.push_status(format!(
                "{}: {err}",
                self.locale.text("load failed", "读取失败")
            ));
            return;
        }
        let mut restored_open_pack: Option<PackOpen> = None;
        for action in &saved.actions {
            if let Err(err) = apply_saved_action(
                &mut restored_run,
                &mut restored_events,
                &mut restored_open_pack,
                action,
            ) {
                self.push_status(format!(
                    "{}: {err}",
                    self.locale.text("load failed", "读取失败")
                ));
                return;
            }
        }
        self.run = restored_run;
        self.events = restored_events;
        self.open_pack = restored_open_pack;
        self.seed = saved.seed;
        self.content_signature = restored_signature;
        self.recorded_actions = saved.actions;
        self.clear_selection();
        self.normalize_cursors();
        self.push_status(format!(
            "{} {} {} {}",
            self.locale.text("loaded", "已读取"),
            self.recorded_actions.len(),
            self.locale.text("actions from", "条动作自"),
            path.display()
        ));
        self.flush_events();
    }

    fn record_action(&mut self, action: &str, indices: Vec<usize>, target: Option<String>) {
        self.recorded_actions.push(SavedAction {
            action: action.to_string(),
            indices,
            target,
        });
    }

    pub fn auto_perform_actions(&mut self, actions: &[SavedAction]) -> Result<(), String> {
        for (idx, action) in actions.iter().enumerate() {
            if let Err(err) =
                apply_saved_action(&mut self.run, &mut self.events, &mut self.open_pack, action)
            {
                return Err(format!("auto step {} failed: {}", idx + 1, err));
            }
        }
        self.recorded_actions.extend(actions.iter().cloned());
        self.clear_selection();
        self.normalize_cursors();
        self.flush_events();
        self.push_status(format!(
            "{} {} {}",
            self.locale.text("auto performed", "自动执行"),
            actions.len(),
            self.locale.text("actions from json", "条JSON动作")
        ));
        Ok(())
    }

    pub fn activate_primary(&mut self) {
        if self.show_help {
            self.show_help = false;
            return;
        }
        if self.open_pack.is_some() {
            self.focus = FocusPane::Shop;
            if self.selected_pack.is_empty() {
                self.toggle_focused_selection();
            } else {
                self.pick_pack_selected();
            }
            return;
        }
        match self.focus {
            FocusPane::Hand => match self.run.state.phase {
                Phase::Deal => self.deal(),
                Phase::Play => {
                    if self.selected_hand.is_empty() {
                        self.toggle_focused_selection();
                    } else {
                        self.play_selected();
                    }
                }
                _ => {}
            },
            FocusPane::Shop => {
                if self.run.state.phase == Phase::Shop {
                    self.buy_selected_offer();
                } else if matches!(self.run.blind_outcome(), Some(BlindOutcome::Cleared)) {
                    self.enter_or_leave_shop();
                }
            }
            FocusPane::Inventory => {
                if let Some(kind) = self.current_inventory_kind() {
                    match kind {
                        InventoryRowKind::Joker(_) => self.sell_selected_joker(),
                        InventoryRowKind::Consumable(_) => self.use_selected_consumable(),
                    }
                }
            }
            FocusPane::Events => {}
        }
    }

    pub fn deal(&mut self) {
        match self.run.prepare_hand(&mut self.events) {
            Ok(_) => {
                self.push_status(self.locale.text("dealt hand", "已发牌"));
                self.record_action("deal", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn play_selected(&mut self) {
        let indices = self.selected_hand_indices();
        if indices.is_empty() {
            self.push_status(self.locale.text("no card selected", "未选择卡牌"));
            return;
        }
        match self.run.play_hand(&indices, &mut self.events) {
            Ok(breakdown) => {
                self.push_status(format!(
                    "{} {}: {:?}={} {}",
                    self.locale.text("played", "已出牌"),
                    self.locale.text("result", "结果"),
                    breakdown.hand,
                    breakdown.total.total(),
                    self.locale.text("points", "分")
                ));
                self.push_breakdown_lines(&breakdown);
                self.record_action("play", indices, None);
                self.selected_hand.clear();
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn discard_selected(&mut self) {
        let indices = self.selected_hand_indices();
        if indices.is_empty() {
            self.push_status(self.locale.text("no card selected", "未选择卡牌"));
            return;
        }
        let result = self.run.discard(&indices, &mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("discarded", "已弃牌"));
                self.record_action("discard", indices, None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
        self.selected_hand.clear();
    }

    pub fn skip_blind(&mut self) {
        self.open_pack = None;
        let result = self.run.skip_blind(&mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("blind skipped", "已跳过盲注"));
                self.record_action("skip_blind", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
        self.clear_selection();
    }

    pub fn next_blind(&mut self) {
        self.open_pack = None;
        let result = self.run.start_next_blind(&mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("started next blind", "已开始下一盲注"));
                self.record_action("next_blind", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
        self.clear_selection();
    }

    pub fn enter_or_leave_shop(&mut self) {
        if self.run.state.phase == Phase::Shop {
            self.run.leave_shop();
            self.open_pack = None;
            self.push_status(self.locale.text("left shop", "已离开商店"));
            self.record_action("leave_shop", Vec::new(), None);
            self.flush_events();
            self.normalize_cursors();
            return;
        }
        let result = self.run.enter_shop(&mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("entered shop", "已进入商店"));
                self.record_action("enter_shop", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
        self.focus = FocusPane::Shop;
    }

    pub fn reroll_shop(&mut self) {
        let result = self.run.reroll_shop(&mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("shop rerolled", "商店已刷新"));
                self.record_action("reroll", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn buy_selected_offer(&mut self) {
        if self.open_pack.is_some() {
            self.push_status(
                self.locale
                    .text("pack is open, pick/skip first", "卡包已打开，请先选择/跳过"),
            );
            return;
        }
        let Some(offer) = self.current_shop_offer() else {
            self.push_status(self.locale.text("no shop offer selected", "未选择商店商品"));
            return;
        };
        let action_target = offer_index(offer);
        let action_name = offer_action_name(offer);
        match self.run.buy_shop_offer(offer, &mut self.events) {
            Ok(purchase) => {
                let result = match &purchase {
                    ShopPurchase::Pack(_) => {
                        match self.run.open_pack_purchase(&purchase, &mut self.events) {
                            Ok(open) => {
                                self.open_pack = Some(open);
                                self.focus = FocusPane::Shop;
                                self.selected_pack.clear();
                                self.push_status(self.locale.text("pack opened", "已打开卡包"));
                                Ok(())
                            }
                            Err(err) => Err(err),
                        }
                    }
                    _ => self.run.apply_purchase(&purchase),
                };
                match result {
                    Ok(_) => {
                        if !matches!(purchase, ShopPurchase::Pack(_)) {
                            self.push_status(self.locale.text("purchase complete", "购买完成"));
                        }
                        self.record_action(
                            action_name,
                            Vec::new(),
                            Some(action_target.to_string()),
                        );
                    }
                    Err(err) => self.push_error(err),
                }
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn pick_pack_selected(&mut self) {
        let Some(open) = self.open_pack.clone() else {
            self.push_status(self.locale.text("no open pack", "当前没有打开的卡包"));
            return;
        };
        let picks = self.selected_pack_indices();
        if picks.is_empty() {
            self.push_status(
                self.locale
                    .text("no pack option selected", "未选择卡包选项"),
            );
            return;
        }
        match self
            .run
            .choose_pack_options(&open, &picks, &mut self.events)
        {
            Ok(_) => {
                self.open_pack = None;
                self.selected_pack.clear();
                self.push_status(self.locale.text("pack applied", "卡包效果已应用"));
                self.record_action("pick_pack", picks, None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn skip_pack(&mut self) {
        let Some(open) = self.open_pack.clone() else {
            self.push_status(self.locale.text("no open pack", "当前没有打开的卡包"));
            return;
        };
        match self.run.skip_pack(&open, &mut self.events) {
            Ok(_) => {
                self.open_pack = None;
                self.selected_pack.clear();
                self.push_status(self.locale.text("pack skipped", "已跳过卡包"));
                self.record_action("skip_pack", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn use_selected_consumable(&mut self) {
        let Some(kind) = self.current_inventory_kind() else {
            self.push_status(self.locale.text("inventory is empty", "库存为空"));
            return;
        };
        let InventoryRowKind::Consumable(index) = kind else {
            self.push_status(
                self.locale
                    .text("focus a consumable first", "请先选中消耗牌"),
            );
            return;
        };
        let selected = self.explicit_selected_hand_indices();
        match self.run.use_consumable(index, &selected, &mut self.events) {
            Ok(_) => {
                self.push_status(self.locale.text("consumable used", "已使用消耗牌"));
                self.record_action("use_consumable", selected, Some(index.to_string()));
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn sell_selected_joker(&mut self) {
        let Some(kind) = self.current_inventory_kind() else {
            self.push_status(self.locale.text("inventory is empty", "库存为空"));
            return;
        };
        let InventoryRowKind::Joker(index) = kind else {
            self.push_status(self.locale.text("focus a joker first", "请先选中小丑"));
            return;
        };
        match self.run.sell_joker(index, &mut self.events) {
            Ok(_) => {
                self.push_status(self.locale.text("joker sold", "已出售小丑"));
                self.record_action("sell_joker", Vec::new(), Some(index.to_string()));
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn normalize_cursors(&mut self) {
        let hand_len = self.hand_len();
        let pack_len = self.pack_len();
        let shop_len = self.shop_rows().len();
        let inventory_len = self.inventory_rows().len();
        clamp_index(&mut self.hand_cursor, hand_len);
        if self.open_pack.is_some() {
            clamp_index(&mut self.pack_cursor, pack_len);
        } else {
            clamp_index(&mut self.shop_cursor, shop_len);
        }
        clamp_index(&mut self.inventory_cursor, inventory_len);
        self.selected_hand.retain(|idx| *idx < hand_len);
        self.selected_pack.retain(|idx| *idx < pack_len);
    }

    pub fn card_label(&self, index: usize, card: &Card) -> String {
        let marker = if self.selected_hand.contains(&index) {
            "*"
        } else {
            " "
        };
        let value = card_value(card, &self.run.tables);
        let detail = card_detail(card);
        format!(
            "{marker} {index:>2}: {:<16} {}:{:<4} {}",
            format_card(card),
            self.locale.text("val", "值"),
            value,
            detail
        )
    }

    pub fn pack_option_label(&self, index: usize, option: &PackOption) -> String {
        let marker = if self.selected_pack.contains(&index) {
            "*"
        } else {
            " "
        };
        let body = match option {
            PackOption::Joker(id) => format!(
                "{} {id} ({})",
                self.locale.text("Joker", "小丑"),
                self.find_joker_name(id)
            ),
            PackOption::Consumable(kind, id) => format!(
                "{} {id} ({}) {} {}{}",
                self.locale.text("Consumable", "消耗牌"),
                self.find_consumable_name(*kind, id),
                self.locale.text("type", "类型"),
                consumable_kind_label(self.locale, *kind),
                {
                    let effect = self.consumable_effect_summary(*kind, id, 2);
                    if effect.is_empty() {
                        String::new()
                    } else {
                        format!(" {} {}", self.locale.text("effect", "效果"), effect)
                    }
                }
            ),
            PackOption::PlayingCard(card) => {
                format!("{} {}", self.locale.text("Card", "卡牌"), format_card(card))
            }
        };
        format!("{marker} {index:>2}: {body}")
    }

    pub fn push_status(&mut self, value: impl Into<String>) {
        self.status_line = value.into();
    }

    pub fn push_error(&mut self, err: RunError) {
        self.status_line = format!("{}: {err}", self.locale.text("error", "错误"));
    }

    fn push_breakdown_lines(&mut self, breakdown: &ScoreBreakdown) {
        self.push_event_line(format!(
            "{}: {:?}",
            self.locale.text("score hand", "计分牌型"),
            breakdown.hand
        ));
        self.push_event_line(format!(
            "{}: {:?}",
            self.locale.text("scoring indices", "计分索引"),
            breakdown.scoring_indices
        ));
        self.push_event_line(format!(
            "{}: {} {} + {} {} = {}",
            self.locale.text("chips", "筹码"),
            self.locale.text("base", "基础"),
            breakdown.base.chips,
            self.locale.text("rank", "牌面"),
            breakdown.rank_chips,
            breakdown.total.chips
        ));
        self.push_event_line(format!(
            "{}: {:.2}",
            self.locale.text("mult", "倍率"),
            breakdown.total.mult
        ));
        self.push_event_line(format!(
            "{}: {}",
            self.locale.text("total score", "总分"),
            breakdown.total.total()
        ));
        let trace = self.run.last_score_trace.clone();
        if trace.is_empty() {
            self.push_event_line(
                self.locale
                    .text("effect steps: none", "效果步骤：无")
                    .to_string(),
            );
            return;
        }
        self.push_event_line(format!(
            "{}: {}",
            self.locale.text("effect steps", "效果步骤"),
            trace.len()
        ));
        for (idx, step) in trace.iter().take(MAX_TRACE_LINES).enumerate() {
            self.push_event_line(format!(
                "#{:02} {} | {} | {}x{:.2} -> {}x{:.2}",
                idx + 1,
                step.source,
                format_rule_effect(self.locale, &step.effect),
                step.before.chips,
                step.before.mult,
                step.after.chips,
                step.after.mult
            ));
        }
        if trace.len() > MAX_TRACE_LINES {
            self.push_event_line(format!(
                "... {} {}",
                trace.len() - MAX_TRACE_LINES,
                self.locale.text("more effect steps", "条效果未展示")
            ));
        }
    }

    fn flush_events(&mut self) {
        let drained: Vec<_> = self.events.drain().collect();
        for event in drained {
            self.push_event_line(format_event(self.locale, &event));
        }
    }

    fn push_event_line(&mut self, line: String) {
        if self.event_log.len() >= MAX_EVENT_LOG {
            let _ = self.event_log.pop_front();
        }
        self.event_log.push_back(line);
    }

    fn find_joker_name(&self, id: &str) -> String {
        self.run
            .content
            .jokers
            .iter()
            .find(|joker| joker.id == id)
            .map(|joker| joker.name.clone())
            .unwrap_or_else(|| "-".to_string())
    }

    fn find_consumable_name(&self, kind: ConsumableKind, id: &str) -> String {
        let list = match kind {
            ConsumableKind::Tarot => &self.run.content.tarots,
            ConsumableKind::Planet => &self.run.content.planets,
            ConsumableKind::Spectral => &self.run.content.spectrals,
        };
        list.iter()
            .find(|card| card.id == id)
            .map(|card| card.name.clone())
            .unwrap_or_else(|| "-".to_string())
    }

    fn find_consumable_def(
        &self,
        kind: ConsumableKind,
        id: &str,
    ) -> Option<&rulatro_core::ConsumableDef> {
        let list = match kind {
            ConsumableKind::Tarot => &self.run.content.tarots,
            ConsumableKind::Planet => &self.run.content.planets,
            ConsumableKind::Spectral => &self.run.content.spectrals,
        };
        list.iter().find(|card| card.id == id)
    }

    fn consumable_effect_summary(
        &self,
        kind: ConsumableKind,
        id: &str,
        max_parts: usize,
    ) -> String {
        let Some(def) = self.find_consumable_def(kind, id) else {
            return String::new();
        };
        summarize_effect_blocks(self.locale, &def.effects, def.hand, max_parts)
    }
}

fn move_index(value: &mut usize, len: usize, down: bool) {
    if len == 0 {
        *value = 0;
        return;
    }
    if down {
        *value = (*value + 1) % len;
    } else if *value == 0 {
        *value = len - 1;
    } else {
        *value -= 1;
    }
}

fn resolve_prompt_path(input: &str, default_path: Option<PathBuf>) -> Result<PathBuf, String> {
    if input.is_empty() {
        return default_path.ok_or_else(|| "save path unavailable".to_string());
    }
    Ok(PathBuf::from(input))
}

fn apply_saved_action(
    run: &mut RunState,
    events: &mut EventBus,
    open_pack: &mut Option<PackOpen>,
    action: &SavedAction,
) -> Result<(), String> {
    match action.action.as_str() {
        "deal" => run.prepare_hand(events).map_err(|err| err.to_string())?,
        "play" => {
            run.play_hand(&action.indices, events)
                .map_err(|err| err.to_string())?;
        }
        "discard" => run
            .discard(&action.indices, events)
            .map_err(|err| err.to_string())?,
        "skip_blind" => run.skip_blind(events).map_err(|err| err.to_string())?,
        "enter_shop" => run.enter_shop(events).map_err(|err| err.to_string())?,
        "leave_shop" => {
            run.leave_shop();
            *open_pack = None;
        }
        "reroll" => run.reroll_shop(events).map_err(|err| err.to_string())?,
        "buy_card" => {
            let idx = parse_saved_index(action.target.as_deref())?;
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Card(idx), events)
                .map_err(|err| err.to_string())?;
            run.apply_purchase(&purchase)
                .map_err(|err| err.to_string())?;
        }
        "buy_pack" => {
            let idx = parse_saved_index(action.target.as_deref())?;
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Pack(idx), events)
                .map_err(|err| err.to_string())?;
            let open = run
                .open_pack_purchase(&purchase, events)
                .map_err(|err| err.to_string())?;
            *open_pack = Some(open);
        }
        "buy_voucher" => {
            let idx = parse_saved_index(action.target.as_deref())?;
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Voucher(idx), events)
                .map_err(|err| err.to_string())?;
            run.apply_purchase(&purchase)
                .map_err(|err| err.to_string())?;
        }
        "pick_pack" => {
            let open = open_pack
                .clone()
                .ok_or_else(|| "no open pack".to_string())?;
            run.choose_pack_options(&open, &action.indices, events)
                .map_err(|err| err.to_string())?;
            *open_pack = None;
        }
        "skip_pack" => {
            let open = open_pack
                .clone()
                .ok_or_else(|| "no open pack".to_string())?;
            run.skip_pack(&open, events)
                .map_err(|err| err.to_string())?;
            *open_pack = None;
        }
        "use_consumable" => {
            let idx = parse_saved_index(action.target.as_deref())?;
            run.use_consumable(idx, &action.indices, events)
                .map_err(|err| err.to_string())?;
        }
        "sell_joker" => {
            let idx = parse_saved_index(action.target.as_deref())?;
            run.sell_joker(idx, events).map_err(|err| err.to_string())?;
        }
        "next_blind" => {
            run.start_next_blind(events)
                .map_err(|err| err.to_string())?;
            *open_pack = None;
        }
        _ => return Err(format!("unknown saved action '{}'", action.action)),
    }
    Ok(())
}

fn parse_saved_index(value: Option<&str>) -> Result<usize, String> {
    value
        .ok_or_else(|| "missing target index".to_string())?
        .parse::<usize>()
        .map_err(|_| "invalid index".to_string())
}

fn offer_index(offer: ShopOfferRef) -> usize {
    match offer {
        ShopOfferRef::Card(idx) | ShopOfferRef::Pack(idx) | ShopOfferRef::Voucher(idx) => idx,
    }
}

fn offer_action_name(offer: ShopOfferRef) -> &'static str {
    match offer {
        ShopOfferRef::Card(_) => "buy_card",
        ShopOfferRef::Pack(_) => "buy_pack",
        ShopOfferRef::Voucher(_) => "buy_voucher",
    }
}

fn clamp_index(value: &mut usize, len: usize) {
    if len == 0 {
        *value = 0;
    } else if *value >= len {
        *value = len - 1;
    }
}

fn toggle_set(set: &mut BTreeSet<usize>, value: usize) {
    if set.contains(&value) {
        let _ = set.remove(&value);
    } else {
        set.insert(value);
    }
}

fn selected_or_cursor(selected: &BTreeSet<usize>, cursor: usize, len: usize) -> Vec<usize> {
    if len == 0 {
        return Vec::new();
    }
    let mut out: Vec<usize> = selected.iter().copied().filter(|idx| *idx < len).collect();
    if out.is_empty() {
        out.push(cursor.min(len - 1));
    }
    out
}

fn format_event(locale: UiLocale, event: &Event) -> String {
    match event {
        Event::BlindStarted {
            ante,
            blind,
            target,
            hands,
            discards,
        } => format!(
            "{} A{ante} {} {} {target} H{hands}/D{discards}",
            locale.text("blind started", "盲注开始"),
            blind_label(locale, *blind),
            locale.text("target", "目标")
        ),
        Event::BlindSkipped { ante, blind, tag } => {
            format!(
                "{} A{ante} {} {} {}",
                locale.text("blind skipped", "盲注已跳过"),
                blind_label(locale, *blind),
                locale.text("tag", "标签"),
                tag.as_deref().unwrap_or(locale.text("-", "无"))
            )
        }
        Event::HandDealt { count } => format!("{} {count}", locale.text("dealt", "已发牌")),
        Event::HandScored {
            hand,
            chips,
            mult,
            total,
        } => format!(
            "{} {hand:?}: {chips} x{mult:.2} = {total}",
            locale.text("scored", "计分")
        ),
        Event::ShopEntered {
            offers,
            reroll_cost,
            reentered,
        } => format!(
            "{} {} {offers} {} {reroll_cost}{}",
            locale.text("shop entered", "进入商店"),
            locale.text("offers", "商品数"),
            locale.text("reroll", "刷新价"),
            if *reentered {
                locale.text(" reentered", "（重复进入）")
            } else {
                ""
            }
        ),
        Event::ShopRerolled {
            offers,
            reroll_cost,
            cost,
            money,
        } => format!(
            "{} {} {offers} {} {cost} {} {reroll_cost} {} {money}",
            locale.text("shop rerolled", "商店刷新"),
            locale.text("offers", "商品数"),
            locale.text("cost", "花费"),
            locale.text("next", "下次"),
            locale.text("money", "金钱")
        ),
        Event::ShopBought { offer, cost, money } => format!(
            "{} {} {} {cost} {} {money}",
            locale.text("shop bought", "商店购买"),
            shop_offer_label(locale, *offer),
            locale.text("cost", "花费"),
            locale.text("money", "金钱")
        ),
        Event::PackOpened {
            kind,
            options,
            picks,
        } => format!(
            "{} {kind:?} {} {options} {} {picks}",
            locale.text("pack opened", "卡包打开"),
            locale.text("options", "选项"),
            locale.text("pick", "可选")
        ),
        Event::PackChosen { picks } => {
            format!("{} {picks}", locale.text("pack chosen", "卡包已选择"))
        }
        Event::JokerSold {
            id,
            sell_value,
            money,
        } => format!(
            "{} {id} {} {sell_value} {} {money}",
            locale.text("joker sold", "出售小丑"),
            locale.text("value", "价值"),
            locale.text("money", "金钱")
        ),
        Event::BlindCleared {
            score,
            reward,
            money,
        } => format!(
            "{} {} {score} {} {reward} {} {money}",
            locale.text("blind cleared", "盲注通过"),
            locale.text("score", "分数"),
            locale.text("reward", "奖励"),
            locale.text("money", "金钱")
        ),
        Event::BlindFailed { score } => format!(
            "{} {} {score}",
            locale.text("blind failed", "盲注失败"),
            locale.text("score", "分数")
        ),
    }
}

pub(crate) fn phase_label(locale: UiLocale, phase: Phase) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match phase {
            Phase::Setup => "准备",
            Phase::Deal => "发牌",
            Phase::Play => "出牌",
            Phase::Score => "计分",
            Phase::Cleanup => "清理",
            Phase::Shop => "商店",
        }
    } else {
        match phase {
            Phase::Setup => "Setup",
            Phase::Deal => "Deal",
            Phase::Play => "Play",
            Phase::Score => "Score",
            Phase::Cleanup => "Cleanup",
            Phase::Shop => "Shop",
        }
    }
}

pub(crate) fn blind_label(locale: UiLocale, blind: BlindKind) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match blind {
            BlindKind::Small => "小盲",
            BlindKind::Big => "大盲",
            BlindKind::Boss => "Boss",
        }
    } else {
        match blind {
            BlindKind::Small => "Small",
            BlindKind::Big => "Big",
            BlindKind::Boss => "Boss",
        }
    }
}

fn consumable_kind_label(locale: UiLocale, kind: ConsumableKind) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match kind {
            ConsumableKind::Tarot => "塔罗",
            ConsumableKind::Planet => "星球",
            ConsumableKind::Spectral => "幻灵",
        }
    } else {
        match kind {
            ConsumableKind::Tarot => "Tarot",
            ConsumableKind::Planet => "Planet",
            ConsumableKind::Spectral => "Spectral",
        }
    }
}

fn shop_card_kind_label(locale: UiLocale, kind: rulatro_core::ShopCardKind) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match kind {
            rulatro_core::ShopCardKind::Joker => "小丑",
            rulatro_core::ShopCardKind::Tarot => "塔罗",
            rulatro_core::ShopCardKind::Planet => "星球",
        }
    } else {
        match kind {
            rulatro_core::ShopCardKind::Joker => "Joker",
            rulatro_core::ShopCardKind::Tarot => "Tarot",
            rulatro_core::ShopCardKind::Planet => "Planet",
        }
    }
}

fn shop_offer_label(locale: UiLocale, offer: rulatro_core::ShopOfferKind) -> String {
    match offer {
        rulatro_core::ShopOfferKind::Card(kind) => {
            format!("{} {:?}", locale.text("card", "卡牌"), kind)
        }
        rulatro_core::ShopOfferKind::Pack(kind, size) => {
            format!("{} {:?}/{:?}", locale.text("pack", "卡包"), kind, size)
        }
        rulatro_core::ShopOfferKind::Voucher => locale.text("voucher", "优惠券").to_string(),
    }
}

fn summarize_effect_blocks(
    locale: UiLocale,
    blocks: &[EffectBlock],
    hand_hint: Option<rulatro_core::HandKind>,
    max_parts: usize,
) -> String {
    let mut parts = Vec::new();
    for block in blocks {
        for op in &block.effects {
            parts.push(summarize_effect_op(locale, op));
        }
    }
    if parts.is_empty() {
        if let Some(hand) = hand_hint {
            return format!(
                "{} {}",
                locale.text("upgrade", "升级"),
                hand_label(locale, hand)
            );
        }
        return String::new();
    }
    let display = if max_parts == 0 {
        parts.len()
    } else {
        parts.len().min(max_parts)
    };
    let mut out = parts[..display].join(" | ");
    if parts.len() > display {
        out.push_str(&format!(
            " {}{}",
            locale.text("+", "+"),
            parts.len() - display
        ));
    }
    out
}

fn summarize_effect_op(locale: UiLocale, op: &EffectOp) -> String {
    match op {
        EffectOp::Score(effect) => format_rule_effect(locale, effect),
        EffectOp::AddMoney(value) => format!("{}${value}", locale.text("+", "+")),
        EffectOp::SetMoney(value) => format!("{}={value}", locale.text("money", "金钱")),
        EffectOp::DoubleMoney { cap } => format!(
            "{} x2 ({} {cap})",
            locale.text("money", "金钱"),
            locale.text("cap", "上限")
        ),
        EffectOp::AddMoneyFromJokers { cap } => format!(
            "{} ({} {cap})",
            locale.text("joker money", "小丑转钱"),
            locale.text("cap", "上限")
        ),
        EffectOp::AddHandSize(value) => format!(
            "{}{}",
            locale.text("hand size ", "手牌上限 "),
            signed_int(*value)
        ),
        EffectOp::UpgradeHand { hand, amount } => {
            format!(
                "{} {} +{}",
                locale.text("upgrade", "升级"),
                hand_label(locale, *hand),
                amount
            )
        }
        EffectOp::UpgradeAllHands { amount } => {
            format!("{} +{}", locale.text("all hands", "全部牌型"), amount)
        }
        EffectOp::AddRandomConsumable { kind, count } => {
            format!("+{} {}", count, consumable_kind_label(locale, *kind))
        }
        EffectOp::AddJoker { rarity, count } => {
            format!("+{} {}({:?})", count, locale.text("joker", "小丑"), rarity)
        }
        EffectOp::AddRandomJoker { count } => {
            format!("+{} {}", count, locale.text("random joker", "随机小丑"))
        }
        EffectOp::RandomJokerEdition { editions, chance } => format!(
            "{} {:?} {} {:.0}%",
            locale.text("joker edition", "小丑版本"),
            editions,
            locale.text("chance", "概率"),
            chance * 100.0
        ),
        EffectOp::SetRandomJokerEdition { edition } => format!(
            "{} {}",
            locale.text("set joker edition", "设置小丑版本"),
            edition_short(*edition)
        ),
        EffectOp::SetRandomJokerEditionDestroyOthers { edition } => format!(
            "{} {}",
            locale.text("set edition destroy others", "设置版本并销毁其他小丑"),
            edition_short(*edition)
        ),
        EffectOp::DuplicateRandomJokerDestroyOthers { remove_negative } => format!(
            "{}{}",
            locale.text("duplicate random joker", "复制随机小丑"),
            if *remove_negative {
                locale.text(" (remove negative)", "（移除负片）")
            } else {
                ""
            }
        ),
        EffectOp::EnhanceSelected { enhancement, count } => format!(
            "{}{} {} {}",
            locale.text("selected", "选中"),
            count,
            locale.text("add", "加成"),
            enhancement_short(*enhancement)
        ),
        EffectOp::AddEditionToSelected { editions, count } => format!(
            "{}{} {} {:?}",
            locale.text("selected", "选中"),
            count,
            locale.text("edition", "版本"),
            editions
        ),
        EffectOp::AddSealToSelected { seal, count } => {
            format!(
                "{}{} {}",
                locale.text("selected", "选中"),
                count,
                seal_short(*seal)
            )
        }
        EffectOp::ConvertSelectedSuit { suit, count } => format!(
            "{}{} {} {}",
            locale.text("selected", "选中"),
            count,
            locale.text("to suit", "改为花色"),
            suit_short(*suit)
        ),
        EffectOp::IncreaseSelectedRank { count, delta } => format!(
            "{}{} {} {}",
            locale.text("selected", "选中"),
            count,
            locale.text("rank", "点数"),
            signed_int(*delta as i64)
        ),
        EffectOp::DestroySelected { count } => format!(
            "{}{} {}",
            locale.text("destroy", "销毁"),
            count,
            locale.text("selected", "选中")
        ),
        EffectOp::DestroyRandomInHand { count } => format!(
            "{}{} {}",
            locale.text("destroy", "销毁"),
            count,
            locale.text("in hand random", "张手牌（随机）")
        ),
        EffectOp::CopySelected { count } => format!(
            "{}{} {}",
            locale.text("copy", "复制"),
            count,
            locale.text("selected", "选中")
        ),
        EffectOp::ConvertLeftIntoRight => locale
            .text("left card turns right card", "左牌变为右牌")
            .to_string(),
        EffectOp::ConvertHandToRandomRank => locale
            .text("hand to random rank", "手牌变为随机点数")
            .to_string(),
        EffectOp::ConvertHandToRandomSuit => locale
            .text("hand to random suit", "手牌变为随机花色")
            .to_string(),
        EffectOp::AddRandomEnhancedCards { count, filter } => format!(
            "+{} {} ({})",
            count,
            locale.text("enhanced cards", "增强牌"),
            rank_filter_label(locale, *filter)
        ),
        EffectOp::CreateLastConsumable { exclude } => {
            if let Some(exclude) = exclude {
                format!(
                    "{} {} ({exclude})",
                    locale.text("repeat last consumable", "重复上一个消耗牌"),
                    locale.text("exclude", "排除")
                )
            } else {
                locale
                    .text("repeat last consumable", "重复上一个消耗牌")
                    .to_string()
            }
        }
        EffectOp::RetriggerScored(times) => format!(
            "{} {}",
            locale.text("retrigger scored", "重触发计分牌"),
            signed_int(*times)
        ),
        EffectOp::RetriggerHeld(times) => format!(
            "{} {}",
            locale.text("retrigger held", "重触发留手牌"),
            signed_int(*times)
        ),
        EffectOp::Custom { name, .. } => format!("[{}]", name),
    }
    .replace("  ", " ")
    .trim()
    .to_string()
}

fn hand_label(locale: UiLocale, hand: rulatro_core::HandKind) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match hand {
            rulatro_core::HandKind::HighCard => "高牌",
            rulatro_core::HandKind::Pair => "对子",
            rulatro_core::HandKind::TwoPair => "两对",
            rulatro_core::HandKind::Trips => "三条",
            rulatro_core::HandKind::Straight => "顺子",
            rulatro_core::HandKind::Flush => "同花",
            rulatro_core::HandKind::FullHouse => "葫芦",
            rulatro_core::HandKind::Quads => "四条",
            rulatro_core::HandKind::StraightFlush => "同花顺",
            rulatro_core::HandKind::RoyalFlush => "皇家同花顺",
            rulatro_core::HandKind::FiveOfAKind => "五条",
            rulatro_core::HandKind::FlushHouse => "同花葫芦",
            rulatro_core::HandKind::FlushFive => "同花五条",
            rulatro_core::HandKind::Custom(_) => "自定义",
        }
    } else {
        match hand {
            rulatro_core::HandKind::HighCard => "HighCard",
            rulatro_core::HandKind::Pair => "Pair",
            rulatro_core::HandKind::TwoPair => "TwoPair",
            rulatro_core::HandKind::Trips => "Trips",
            rulatro_core::HandKind::Straight => "Straight",
            rulatro_core::HandKind::Flush => "Flush",
            rulatro_core::HandKind::FullHouse => "FullHouse",
            rulatro_core::HandKind::Quads => "Quads",
            rulatro_core::HandKind::StraightFlush => "StraightFlush",
            rulatro_core::HandKind::RoyalFlush => "RoyalFlush",
            rulatro_core::HandKind::FiveOfAKind => "FiveOfAKind",
            rulatro_core::HandKind::FlushHouse => "FlushHouse",
            rulatro_core::HandKind::FlushFive => "FlushFive",
            rulatro_core::HandKind::Custom(_) => "Custom",
        }
    }
}

fn rank_filter_label(locale: UiLocale, filter: RankFilter) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match filter {
            RankFilter::Any => "任意",
            RankFilter::Face => "人头牌",
            RankFilter::Ace => "A",
            RankFilter::Numbered => "数字牌",
        }
    } else {
        match filter {
            RankFilter::Any => "Any",
            RankFilter::Face => "Face",
            RankFilter::Ace => "Ace",
            RankFilter::Numbered => "Numbered",
        }
    }
}

fn signed_int(value: i64) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

fn format_rule_effect(locale: UiLocale, effect: &RuleEffect) -> String {
    match effect {
        RuleEffect::AddChips(value) => {
            format!("{}{}", locale.text("+chips ", "+筹码 "), value)
        }
        RuleEffect::AddMult(value) => {
            format!(
                "{}{}",
                locale.text("+mult ", "+倍率 "),
                format!("{value:.2}")
            )
        }
        RuleEffect::MultiplyMult(value) => {
            format!(
                "{}{}",
                locale.text("xmult ", "倍率x"),
                format!("{value:.2}")
            )
        }
        RuleEffect::MultiplyChips(value) => {
            format!(
                "{}{}",
                locale.text("xchips ", "筹码x"),
                format!("{value:.2}")
            )
        }
    }
}

pub fn format_card(card: &Card) -> String {
    if card.face_down {
        return "??".to_string();
    }
    let mut out = format!("{}{}", rank_short(card.rank), suit_short(card.suit));
    let mut tags = Vec::new();
    if let Some(enhancement) = card.enhancement {
        tags.push(enhancement_short(enhancement).to_string());
    }
    if let Some(edition) = card.edition {
        tags.push(edition_short(edition).to_string());
    }
    if let Some(seal) = card.seal {
        tags.push(seal_short(seal).to_string());
    }
    if card.bonus_chips != 0 {
        tags.push(format!("+{}", card.bonus_chips));
    }
    if !tags.is_empty() {
        out.push_str(" [");
        out.push_str(&tags.join(","));
        out.push(']');
    }
    out
}

fn card_value(card: &Card, tables: &rulatro_core::ScoreTables) -> i64 {
    if card.is_stone() {
        return 0;
    }
    tables.rank_chips(card.rank) + card.bonus_chips
}

fn card_detail(card: &Card) -> String {
    if card.face_down {
        return "face_down".to_string();
    }
    let mut tags = Vec::new();
    tags.push(format!("{:?}{:?}", card.rank, card.suit));
    if let Some(enhancement) = card.enhancement {
        tags.push(format!("enh={}", enhancement_short(enhancement)));
    }
    if let Some(edition) = card.edition {
        tags.push(format!("ed={}", edition_short(edition)));
    }
    if let Some(seal) = card.seal {
        tags.push(format!("seal={}", seal_short(seal)));
    }
    if card.bonus_chips != 0 {
        tags.push(format!("bonus={}", card.bonus_chips));
    }
    tags.join(" ")
}

fn enhancement_short(kind: Enhancement) -> &'static str {
    match kind {
        Enhancement::Bonus => "Bonus",
        Enhancement::Mult => "Mult",
        Enhancement::Wild => "Wild",
        Enhancement::Glass => "Glass",
        Enhancement::Steel => "Steel",
        Enhancement::Stone => "Stone",
        Enhancement::Lucky => "Lucky",
        Enhancement::Gold => "Gold",
    }
}

fn edition_short(kind: Edition) -> String {
    match kind {
        Edition::Foil => "Foil".to_string(),
        Edition::Holographic => "Holo".to_string(),
        Edition::Polychrome => "Poly".to_string(),
        Edition::Negative => "Neg".to_string(),
    }
}

fn seal_short(kind: Seal) -> &'static str {
    match kind {
        Seal::Red => "R",
        Seal::Blue => "B",
        Seal::Gold => "G",
        Seal::Purple => "P",
    }
}

fn rank_short(rank: rulatro_core::Rank) -> &'static str {
    match rank {
        rulatro_core::Rank::Ace => "A",
        rulatro_core::Rank::King => "K",
        rulatro_core::Rank::Queen => "Q",
        rulatro_core::Rank::Jack => "J",
        rulatro_core::Rank::Ten => "T",
        rulatro_core::Rank::Nine => "9",
        rulatro_core::Rank::Eight => "8",
        rulatro_core::Rank::Seven => "7",
        rulatro_core::Rank::Six => "6",
        rulatro_core::Rank::Five => "5",
        rulatro_core::Rank::Four => "4",
        rulatro_core::Rank::Three => "3",
        rulatro_core::Rank::Two => "2",
        rulatro_core::Rank::Joker => "Jk",
    }
}

fn suit_short(suit: rulatro_core::Suit) -> &'static str {
    match suit {
        rulatro_core::Suit::Spades => "S",
        rulatro_core::Suit::Hearts => "H",
        rulatro_core::Suit::Clubs => "C",
        rulatro_core::Suit::Diamonds => "D",
        rulatro_core::Suit::Wild => "W",
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_prompt_path;
    use std::path::PathBuf;

    #[test]
    fn resolve_prompt_path_prefers_user_input() {
        let out = resolve_prompt_path("custom/save.json", Some(PathBuf::from("default.json")))
            .expect("path");
        assert_eq!(out, PathBuf::from("custom/save.json"));
    }

    #[test]
    fn resolve_prompt_path_uses_default_for_empty_input() {
        let out = resolve_prompt_path("", Some(PathBuf::from("default.json"))).expect("path");
        assert_eq!(out, PathBuf::from("default.json"));
    }

    #[test]
    fn resolve_prompt_path_errors_without_default() {
        let out = resolve_prompt_path("", None);
        assert!(out.is_err());
    }
}
