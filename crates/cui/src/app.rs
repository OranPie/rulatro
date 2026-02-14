use crate::persistence::{
    compute_content_signature, default_state_path, load_state_file, save_state_file, SavedAction,
};
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rulatro_core::{
    BlindKind, BlindOutcome, Card, ConsumableKind, Event, EventBus, PackOpen, PackOption, Phase,
    RunError, RunState, ShopOfferRef, ShopPurchase,
};
use rulatro_data::{load_content_with_mods_locale, load_game_config, normalize_locale};
use rulatro_modding::ModManager;
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

pub const DEFAULT_RUN_SEED: u64 = 0xC0FFEE;
const MAX_EVENT_LOG: usize = 200;

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
        startup_notes.push(format!("mods loaded: {}", modded.mods.len()));
        for item in &modded.mods {
            startup_notes.push(format!(
                "  - {} {}",
                item.manifest.meta.id, item.manifest.meta.version
            ));
        }
    }
    for warning in &modded.warnings {
        startup_notes.push(format!("warning: {warning}"));
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
            status_line: locale.text("ready", "ready").to_string(),
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
            FocusPane::Hand => self.locale.text("Hand", "Hand"),
            FocusPane::Shop => {
                if self.open_pack.is_some() {
                    self.locale.text("Pack", "Pack")
                } else {
                    self.locale.text("Shop", "Shop")
                }
            }
            FocusPane::Inventory => self.locale.text("Inventory", "Inventory"),
            FocusPane::Events => self.locale.text("Events", "Events"),
        }
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
                .text("select pack options", "select pack options")
                .to_string();
        }
        if let Some(outcome) = self.run.blind_outcome() {
            return match outcome {
                BlindOutcome::Cleared => {
                    if self.run.state.phase == Phase::Shop {
                        self.locale
                            .text("buy/reroll/leave", "buy/reroll/leave")
                            .to_string()
                    } else {
                        self.locale
                            .text("enter shop or next", "enter shop or next")
                            .to_string()
                    }
                }
                BlindOutcome::Failed => self
                    .locale
                    .text("start next blind", "start next blind")
                    .to_string(),
            };
        }
        match self.run.state.phase {
            Phase::Deal => self.locale.text("deal", "deal").to_string(),
            Phase::Play => self.locale.text("play/discard", "play/discard").to_string(),
            Phase::Shop => self
                .locale
                .text("buy/reroll/leave", "buy/reroll/leave")
                .to_string(),
            Phase::Setup | Phase::Score | Phase::Cleanup => {
                self.locale.text("next blind", "next blind").to_string()
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
            rows.push(ShopRow {
                offer: ShopOfferRef::Card(idx),
                label: format!("C{idx} {:?} {} ${}", card.kind, card.item_id, card.price),
            });
        }
        for (idx, pack) in shop.packs.iter().enumerate() {
            rows.push(ShopRow {
                offer: ShopOfferRef::Pack(idx),
                label: format!(
                    "P{idx} {:?}/{:?} opt:{} pick:{} ${}",
                    pack.kind, pack.size, pack.options, pack.picks, pack.price
                ),
            });
        }
        for idx in 0..shop.vouchers {
            rows.push(ShopRow {
                offer: ShopOfferRef::Voucher(idx),
                label: format!(
                    "V{idx} {} ${}",
                    self.locale.text("voucher", "voucher"),
                    self.run.config.shop.prices.voucher
                ),
            });
        }
        rows
    }

    pub fn inventory_rows(&self) -> Vec<InventoryRow> {
        let mut rows = Vec::new();
        for (idx, joker) in self.run.inventory.jokers.iter().enumerate() {
            rows.push(InventoryRow {
                kind: InventoryRowKind::Joker(idx),
                label: format!("J{idx} {} ({})", joker.id, self.find_joker_name(&joker.id)),
            });
        }
        for (idx, item) in self.run.inventory.consumables.iter().enumerate() {
            rows.push(InventoryRow {
                kind: InventoryRowKind::Consumable(idx),
                label: format!(
                    "C{idx} {} ({}) {:?}",
                    item.id,
                    self.find_consumable_name(item.kind, &item.id),
                    item.kind
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
                self.push_status("path prompt cancelled");
            }
            KeyCode::Enter => {
                let resolved =
                    resolve_prompt_path(self.path_prompt_input.trim(), default_state_path());
                self.path_prompt_mode = None;
                self.path_prompt_input.clear();
                let Ok(path) = resolved else {
                    self.push_status("save path unavailable");
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
                "saved {} actions to {}",
                self.recorded_actions.len(),
                path.display()
            )),
            Err(err) => self.push_status(format!("save failed: {err}")),
        }
    }

    fn load_from_path(&mut self, path: PathBuf) {
        let saved = match load_state_file(&path) {
            Ok(saved) => saved,
            Err(err) => {
                self.push_status(format!("load failed: {err}"));
                return;
            }
        };
        let (mut restored_run, restored_signature, _notes) =
            match build_run_with_seed(self.locale, saved.seed) {
                Ok(bundle) => bundle,
                Err(err) => {
                    self.push_status(format!("load failed: {err}"));
                    return;
                }
            };
        if !saved.content_signature.is_empty() && saved.content_signature != restored_signature {
            self.push_status(format!(
                "load failed: content signature mismatch (saved={} current={})",
                saved.content_signature, restored_signature
            ));
            return;
        }
        let mut restored_events = EventBus::default();
        if let Err(err) = restored_run.start_blind(1, BlindKind::Small, &mut restored_events) {
            self.push_status(format!("load failed: {err}"));
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
                self.push_status(format!("load failed: {err}"));
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
            "loaded {} actions from {}",
            self.recorded_actions.len(),
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
                self.push_status(self.locale.text("dealt hand", "dealt hand"));
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
            self.push_status(self.locale.text("no card selected", "no card selected"));
            return;
        }
        match self.run.play_hand(&indices, &mut self.events) {
            Ok(breakdown) => {
                self.push_status(format!(
                    "{} {:?} = {}",
                    self.locale.text("played", "played"),
                    breakdown.hand,
                    breakdown.total.total()
                ));
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
            self.push_status(self.locale.text("no card selected", "no card selected"));
            return;
        }
        let result = self.run.discard(&indices, &mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("discarded", "discarded"));
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
                self.push_status(self.locale.text("blind skipped", "blind skipped"));
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
                self.push_status(self.locale.text("started next blind", "started next blind"));
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
            self.push_status(self.locale.text("left shop", "left shop"));
            self.record_action("leave_shop", Vec::new(), None);
            self.flush_events();
            self.normalize_cursors();
            return;
        }
        let result = self.run.enter_shop(&mut self.events);
        match result {
            Ok(_) => {
                self.push_status(self.locale.text("entered shop", "entered shop"));
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
                self.push_status(self.locale.text("shop rerolled", "shop rerolled"));
                self.record_action("reroll", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn buy_selected_offer(&mut self) {
        if self.open_pack.is_some() {
            self.push_status(self.locale.text(
                "pack is open, pick/skip first",
                "pack is open, pick/skip first",
            ));
            return;
        }
        let Some(offer) = self.current_shop_offer() else {
            self.push_status(
                self.locale
                    .text("no shop offer selected", "no shop offer selected"),
            );
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
                                self.push_status(self.locale.text("pack opened", "pack opened"));
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
                            self.push_status(
                                self.locale.text("purchase complete", "purchase complete"),
                            );
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
            self.push_status(self.locale.text("no open pack", "no open pack"));
            return;
        };
        let picks = self.selected_pack_indices();
        if picks.is_empty() {
            self.push_status(
                self.locale
                    .text("no pack option selected", "no pack option selected"),
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
                self.push_status(self.locale.text("pack applied", "pack applied"));
                self.record_action("pick_pack", picks, None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn skip_pack(&mut self) {
        let Some(open) = self.open_pack.clone() else {
            self.push_status(self.locale.text("no open pack", "no open pack"));
            return;
        };
        match self.run.skip_pack(&open, &mut self.events) {
            Ok(_) => {
                self.open_pack = None;
                self.selected_pack.clear();
                self.push_status(self.locale.text("pack skipped", "pack skipped"));
                self.record_action("skip_pack", Vec::new(), None);
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn use_selected_consumable(&mut self) {
        let Some(kind) = self.current_inventory_kind() else {
            self.push_status(self.locale.text("inventory is empty", "inventory is empty"));
            return;
        };
        let InventoryRowKind::Consumable(index) = kind else {
            self.push_status(
                self.locale
                    .text("focus a consumable first", "focus a consumable first"),
            );
            return;
        };
        let selected = self.explicit_selected_hand_indices();
        match self.run.use_consumable(index, &selected, &mut self.events) {
            Ok(_) => {
                self.push_status(self.locale.text("consumable used", "consumable used"));
                self.record_action("use_consumable", selected, Some(index.to_string()));
            }
            Err(err) => self.push_error(err),
        }
        self.flush_events();
        self.normalize_cursors();
    }

    pub fn sell_selected_joker(&mut self) {
        let Some(kind) = self.current_inventory_kind() else {
            self.push_status(self.locale.text("inventory is empty", "inventory is empty"));
            return;
        };
        let InventoryRowKind::Joker(index) = kind else {
            self.push_status(
                self.locale
                    .text("focus a joker first", "focus a joker first"),
            );
            return;
        };
        match self.run.sell_joker(index, &mut self.events) {
            Ok(_) => {
                self.push_status(self.locale.text("joker sold", "joker sold"));
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
        format!("{marker} {index:>2}: {}", format_card(card))
    }

    pub fn pack_option_label(&self, index: usize, option: &PackOption) -> String {
        let marker = if self.selected_pack.contains(&index) {
            "*"
        } else {
            " "
        };
        let body = match option {
            PackOption::Joker(id) => format!("Joker {id} ({})", self.find_joker_name(id)),
            PackOption::Consumable(kind, id) => format!(
                "Consumable {id} ({}) {:?}",
                self.find_consumable_name(*kind, id),
                kind
            ),
            PackOption::PlayingCard(card) => format!("Card {}", format_card(card)),
        };
        format!("{marker} {index:>2}: {body}")
    }

    pub fn push_status(&mut self, value: impl Into<String>) {
        self.status_line = value.into();
    }

    pub fn push_error(&mut self, err: RunError) {
        self.status_line = format!("{}: {err}", self.locale.text("error", "error"));
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

fn format_event(_locale: UiLocale, event: &Event) -> String {
    match event {
        Event::BlindStarted {
            ante,
            blind,
            target,
            hands,
            discards,
        } => format!("blind started A{ante} {blind:?} target {target} H{hands}/D{discards}"),
        Event::BlindSkipped { ante, blind, tag } => {
            format!(
                "blind skipped A{ante} {blind:?} tag {}",
                tag.as_deref().unwrap_or("-")
            )
        }
        Event::HandDealt { count } => format!("dealt {count}"),
        Event::HandScored {
            hand,
            chips,
            mult,
            total,
        } => format!("scored {hand:?}: {chips} x{mult:.2} = {total}"),
        Event::ShopEntered {
            offers,
            reroll_cost,
            reentered,
        } => format!(
            "shop entered offers {offers} reroll {reroll_cost}{}",
            if *reentered { " reentered" } else { "" }
        ),
        Event::ShopRerolled {
            offers,
            reroll_cost,
            cost,
            money,
        } => format!("shop rerolled offers {offers} cost {cost} next {reroll_cost} money {money}"),
        Event::ShopBought { offer, cost, money } => {
            format!("shop bought {offer:?} cost {cost} money {money}")
        }
        Event::PackOpened {
            kind,
            options,
            picks,
        } => format!("pack {kind:?} options {options} picks {picks}"),
        Event::PackChosen { picks } => format!("pack chosen {picks}"),
        Event::JokerSold {
            id,
            sell_value,
            money,
        } => format!("joker sold {id} value {sell_value} money {money}"),
        Event::BlindCleared {
            score,
            reward,
            money,
        } => format!("blind cleared score {score} reward {reward} money {money}"),
        Event::BlindFailed { score } => format!("blind failed score {score}"),
    }
}

pub fn format_card(card: &Card) -> String {
    if card.face_down {
        return "??".to_string();
    }
    let mut out = format!("{}{}", rank_short(card.rank), suit_short(card.suit));
    let mut tags = Vec::new();
    if let Some(enhancement) = card.enhancement {
        tags.push(format!("{enhancement:?}"));
    }
    if let Some(edition) = card.edition {
        tags.push(format!("{edition:?}"));
    }
    if let Some(seal) = card.seal {
        tags.push(format!("{seal:?}"));
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
