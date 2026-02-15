use rulatro_core::{
    format_joker_effect_compact, voucher_by_id, BlindKind, Card, ConsumableKind, EventBus,
    PackOpen, PackOption, Phase, RuleEffect, RunState, ScoreBreakdown, ScoreTables, ScoreTraceStep,
    ShopOfferRef,
};
use rulatro_data::{load_content_with_mods_locale, load_game_config, normalize_locale};
use rulatro_modding::ModManager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server, StatusCode};

const DEFAULT_RUN_SEED: u64 = 0xC0FFEE;

fn main() {
    let locale = parse_locale_from_args();
    let server = Server::http("0.0.0.0:7878").expect("start server");
    println!(
        "Rulatro web server on http://localhost:7878 (lang: {})",
        locale
    );
    let state = Arc::new(Mutex::new(AppState::new(&locale)));
    for request in server.incoming_requests() {
        let state = state.clone();
        if let Err(err) = handle_request(request, state) {
            eprintln!("request error: {err}");
        }
    }
}

struct AppState {
    locale: String,
    content_signature: String,
    run: RunState,
    events: EventBus,
    open_pack: Option<PackOpen>,
    last_breakdown: Option<ScoreBreakdown>,
    last_played: Vec<Card>,
}

impl AppState {
    fn new(locale: &str) -> Self {
        Self::new_with_seed(locale, DEFAULT_RUN_SEED)
    }

    fn new_with_seed(locale: &str, seed: u64) -> Self {
        let config = load_game_config(Path::new("assets")).expect("load config");
        let modded =
            load_content_with_mods_locale(Path::new("assets"), Path::new("mods"), Some(locale))
                .expect("load content");
        let mut runtime = ModManager::new();
        runtime.load_mods(&modded.mods).expect("load mod runtime");
        let mut run = RunState::new(config, modded.content, seed);
        run.set_mod_runtime(Some(Box::new(runtime)));
        let content_signature =
            compute_content_signature(locale).unwrap_or_else(|_| "".to_string());
        Self {
            locale: locale.to_string(),
            content_signature,
            run,
            events: EventBus::default(),
            open_pack: None,
            last_breakdown: None,
            last_played: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct ApiResponse {
    locale: String,
    ok: bool,
    error: Option<String>,
    state: UiState,
    events: Vec<rulatro_core::Event>,
    open_pack: Option<UiPackOpen>,
    last_breakdown: Option<UiScoreBreakdown>,
}

#[derive(Serialize)]
struct UiState {
    seed: u64,
    content_signature: String,
    ante: u8,
    blind: BlindKind,
    phase: Phase,
    money: i64,
    score: i64,
    target: i64,
    hands_left: u8,
    discards_left: u8,
    hands_max: u8,
    discards_max: u8,
    hand_size: usize,
    hand_size_base: usize,
    hand: Vec<UiCard>,
    deck_draw: usize,
    deck_discard: usize,
    rank_chips: Vec<UiRankChip>,
    jokers: Vec<UiJoker>,
    consumables: Vec<UiConsumable>,
    shop: Option<UiShop>,
    blinds_skipped: u32,
    tags: Vec<String>,
    duplicate_next_tag: bool,
    duplicate_tag_exclude: Option<String>,
    hand_levels: Vec<UiHandLevel>,
    boss_id: Option<String>,
    boss_name: Option<String>,
    boss_disabled: bool,
    boss_effects: Vec<String>,
    active_vouchers: Vec<String>,
}

#[derive(Serialize)]
struct UiCard {
    id: u32,
    suit: rulatro_core::Suit,
    rank: rulatro_core::Rank,
    enhancement: Option<rulatro_core::Enhancement>,
    edition: Option<rulatro_core::Edition>,
    seal: Option<rulatro_core::Seal>,
    bonus_chips: i64,
    face_down: bool,
}

#[derive(Serialize)]
struct UiJoker {
    id: String,
    name: String,
    rarity: rulatro_core::JokerRarity,
    edition: Option<rulatro_core::Edition>,
    buy_price: i64,
}

#[derive(Serialize)]
struct UiConsumable {
    id: String,
    name: String,
    kind: ConsumableKind,
    edition: Option<rulatro_core::Edition>,
}

#[derive(Serialize)]
struct UiShop {
    cards: Vec<UiShopCard>,
    packs: Vec<UiShopPack>,
    vouchers: usize,
    voucher_offers: Vec<UiVoucherOffer>,
    voucher_price: i64,
    reroll_cost: i64,
}

#[derive(Serialize)]
struct UiVoucherOffer {
    id: String,
    name: String,
    effect: String,
}

#[derive(Serialize)]
struct UiShopCard {
    kind: rulatro_core::ShopCardKind,
    item_id: String,
    name: String,
    rarity: Option<rulatro_core::JokerRarity>,
    price: i64,
    edition: Option<rulatro_core::Edition>,
}

#[derive(Serialize)]
struct UiShopPack {
    kind: rulatro_core::PackKind,
    size: rulatro_core::PackSize,
    options: u8,
    picks: u8,
    price: i64,
}

#[derive(Serialize)]
struct UiPackOpen {
    offer: UiShopPack,
    options: Vec<UiPackOption>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value")]
enum UiPackOption {
    Joker {
        id: String,
        name: String,
    },
    Consumable {
        kind: ConsumableKind,
        id: String,
        name: String,
    },
    PlayingCard(UiCard),
}

#[derive(Serialize)]
struct UiHandLevel {
    hand: rulatro_core::HandKind,
    level: u32,
}

#[derive(Serialize)]
struct UiRankChip {
    rank: rulatro_core::Rank,
    chips: i64,
}

#[derive(Serialize)]
struct UiScoreBreakdown {
    hand: rulatro_core::HandKind,
    base_chips: i64,
    base_mult: f64,
    rank_chips: i64,
    scoring_indices: Vec<usize>,
    total_chips: i64,
    total_mult: f64,
    total_score: i64,
    played_cards: Vec<UiCard>,
    scoring_cards: Vec<UiScoringCard>,
    steps: Vec<UiScoreStep>,
}

#[derive(Serialize)]
struct UiScoringCard {
    index: usize,
    card: UiCard,
    chips: i64,
}

#[derive(Serialize)]
struct UiScoreStep {
    source: String,
    effect: String,
    before_chips: i64,
    before_mult: f64,
    after_chips: i64,
    after_mult: f64,
}

#[derive(Deserialize)]
struct ActionRequest {
    action: String,
    #[serde(default)]
    indices: Vec<usize>,
    #[serde(default)]
    target: Option<String>,
}

fn handle_request(
    mut request: tiny_http::Request,
    state: Arc<Mutex<AppState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = request.url().to_string();
    match (request.method(), url.as_str()) {
        (&Method::Get, "/") => {
            respond_with_file(request, web_path("index.html"), "text/html; charset=utf-8")?;
        }
        (&Method::Get, "/app.js") => {
            respond_with_file(request, web_path("app.js"), "application/javascript")?;
        }
        (&Method::Get, "/styles.css") => {
            respond_with_file(request, web_path("styles.css"), "text/css; charset=utf-8")?;
        }
        (&Method::Get, "/api/state") => {
            let mut guard = state.lock().unwrap();
            let response = build_response(&mut *guard, None);
            respond_json(request, response)?;
        }
        (&Method::Post, "/api/action") => {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body)?;
            let action: ActionRequest = serde_json::from_str(&body)?;
            let mut guard = state.lock().unwrap();
            let err = apply_action(&mut *guard, action);
            let response = build_response(&mut *guard, err);
            respond_json(request, response)?;
        }
        _ => {
            let response = Response::empty(StatusCode(404));
            request.respond(response)?;
        }
    }
    Ok(())
}

fn web_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("web")
        .join(file)
}

fn parse_locale_from_args() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut locale = std::env::var("RULATRO_LANG").ok();
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--lang" | "-l" => {
                if let Some(value) = args.get(idx + 1) {
                    locale = Some(value.clone());
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }
    normalize_locale(locale.as_deref())
}

#[derive(Clone, Copy)]
struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

fn hash_dir_tree(base: &Path, rel: &Path, hasher: &mut Fnv64) -> Result<(), String> {
    let path = base.join(rel);
    if !path.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(&path)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_name = entry.file_name();
        let rel_path = if rel.as_os_str().is_empty() {
            PathBuf::from(&file_name)
        } else {
            rel.join(&file_name)
        };
        let entry_path = entry.path();
        if entry_path.is_dir() {
            hasher.update(b"D");
            hasher.update(rel_path.to_string_lossy().as_bytes());
            hasher.update(&[0]);
            hash_dir_tree(base, &rel_path, hasher)?;
        } else if entry_path.is_file() {
            hasher.update(b"F");
            hasher.update(rel_path.to_string_lossy().as_bytes());
            hasher.update(&[0]);
            let bytes = fs::read(&entry_path).map_err(|err| err.to_string())?;
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        }
    }
    Ok(())
}

fn compute_content_signature(locale: &str) -> Result<String, String> {
    let mut hasher = Fnv64::new();
    hasher.update(b"rulatro-save-signature-v1");
    hasher.update(locale.as_bytes());
    hash_dir_tree(Path::new("assets"), Path::new(""), &mut hasher)?;
    hash_dir_tree(Path::new("mods"), Path::new(""), &mut hasher)?;
    Ok(format!("{:016x}", hasher.finish()))
}

fn respond_with_file(
    request: tiny_http::Request,
    path: PathBuf,
    content_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    let header = Header::from_bytes(&b"Content-Type"[..], content_type)
        .expect("valid static content-type header");
    let response = Response::from_data(content).with_header(header);
    request.respond(response)?;
    Ok(())
}

fn respond_json(
    request: tiny_http::Request,
    response: ApiResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_vec_pretty(&response)?;
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .expect("valid json content-type header");
    request.respond(Response::from_data(body).with_header(header))?;
    Ok(())
}

fn build_response(state: &mut AppState, err: Option<String>) -> ApiResponse {
    let events: Vec<_> = state.events.drain().collect();
    ApiResponse {
        locale: state.locale.clone(),
        ok: err.is_none(),
        error: err,
        state: snapshot_state(&state.run, &state.content_signature, &state.locale),
        events,
        open_pack: state
            .open_pack
            .as_ref()
            .map(|open| snapshot_open_pack(&state.run, open)),
        last_breakdown: state.last_breakdown.as_ref().map(|breakdown| {
            snapshot_breakdown(
                breakdown,
                &state.last_played,
                &state.run.tables,
                &state.run.last_score_trace,
            )
        }),
    }
}

fn snapshot_state(run: &RunState, content_signature: &str, locale: &str) -> UiState {
    let zh_cn = normalize_locale(Some(locale)) == "zh_CN";
    let hand = run.hand.iter().map(snapshot_card).collect();
    let jokers = run
        .inventory
        .jokers
        .iter()
        .map(|joker| UiJoker {
            id: joker.id.clone(),
            name: find_joker_name(run, &joker.id),
            rarity: joker.rarity,
            edition: joker.edition,
            buy_price: joker.buy_price,
        })
        .collect();
    let consumables = run
        .inventory
        .consumables
        .iter()
        .map(|item| UiConsumable {
            id: item.id.clone(),
            name: find_consumable_name(run, item.kind, &item.id),
            kind: item.kind,
            edition: item.edition,
        })
        .collect();
    let shop = run.shop.as_ref().map(|shop| UiShop {
        cards: shop
            .cards
            .iter()
            .map(|card| UiShopCard {
                kind: card.kind,
                item_id: card.item_id.clone(),
                name: match card.kind {
                    rulatro_core::ShopCardKind::Joker => find_joker_name(run, &card.item_id),
                    rulatro_core::ShopCardKind::Tarot => {
                        find_consumable_name(run, ConsumableKind::Tarot, &card.item_id)
                    }
                    rulatro_core::ShopCardKind::Planet => {
                        find_consumable_name(run, ConsumableKind::Planet, &card.item_id)
                    }
                },
                rarity: card.rarity,
                price: card.price,
                edition: card.edition,
            })
            .collect(),
        packs: shop
            .packs
            .iter()
            .map(|pack| UiShopPack {
                kind: pack.kind,
                size: pack.size,
                options: pack.options,
                picks: pack.picks,
                price: pack.price,
            })
            .collect(),
        vouchers: shop.vouchers,
        voucher_offers: shop
            .voucher_offers
            .iter()
            .map(|offer| {
                if let Some(def) = voucher_by_id(&offer.id) {
                    UiVoucherOffer {
                        id: offer.id.clone(),
                        name: def.name(zh_cn).to_string(),
                        effect: def.effect_text(zh_cn).to_string(),
                    }
                } else {
                    UiVoucherOffer {
                        id: offer.id.clone(),
                        name: offer.id.clone(),
                        effect: String::new(),
                    }
                }
            })
            .collect(),
        voucher_price: run.config.shop.prices.voucher,
        reroll_cost: shop.reroll_cost,
    });
    let boss_id = run.state.boss_id.clone();
    let boss_name = run.current_boss().map(|boss| boss.name.clone());
    let boss_disabled = run.boss_effects_disabled();
    let boss_effects = if boss_disabled {
        Vec::new()
    } else {
        run.current_boss()
            .map(|boss| {
                boss.effects
                    .iter()
                    .map(format_joker_effect_compact)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let mut hand_levels: Vec<_> = rulatro_core::HandKind::ALL
        .iter()
        .map(|kind| UiHandLevel {
            hand: *kind,
            level: run.state.hand_levels.get(kind).copied().unwrap_or(1),
        })
        .collect();
    hand_levels.sort_by(|a, b| a.hand.id().cmp(b.hand.id()));
    let rank_chips = run
        .config
        .ranks
        .iter()
        .map(|rule| UiRankChip {
            rank: rule.rank,
            chips: rule.chips,
        })
        .collect();
    UiState {
        seed: run.rng.seed(),
        content_signature: content_signature.to_string(),
        ante: run.state.ante,
        blind: run.state.blind,
        phase: run.state.phase,
        money: run.state.money,
        score: run.state.blind_score,
        target: run.state.target,
        hands_left: run.state.hands_left,
        discards_left: run.state.discards_left,
        hands_max: run.state.hands_max,
        discards_max: run.state.discards_max,
        hand_size: run.state.hand_size,
        hand_size_base: run.state.hand_size_base,
        hand,
        deck_draw: run.deck.draw.len(),
        deck_discard: run.deck.discard.len(),
        rank_chips,
        jokers,
        consumables,
        shop,
        blinds_skipped: run.state.blinds_skipped,
        tags: run.state.tags.clone(),
        duplicate_next_tag: run.state.duplicate_next_tag,
        duplicate_tag_exclude: run.state.duplicate_tag_exclude.clone(),
        hand_levels,
        boss_id,
        boss_name,
        boss_disabled,
        boss_effects,
        active_vouchers: run.active_voucher_summaries(zh_cn),
    }
}

fn snapshot_open_pack(run: &RunState, open: &PackOpen) -> UiPackOpen {
    UiPackOpen {
        offer: UiShopPack {
            kind: open.offer.kind,
            size: open.offer.size,
            options: open.offer.options,
            picks: open.offer.picks,
            price: open.offer.price,
        },
        options: open
            .options
            .iter()
            .map(|option| match option {
                PackOption::Joker(id) => UiPackOption::Joker {
                    id: id.clone(),
                    name: find_joker_name(run, id),
                },
                PackOption::Consumable(kind, id) => UiPackOption::Consumable {
                    kind: *kind,
                    id: id.clone(),
                    name: find_consumable_name(run, *kind, id),
                },
                PackOption::PlayingCard(card) => UiPackOption::PlayingCard(snapshot_card(card)),
            })
            .collect(),
    }
}

fn snapshot_breakdown(
    breakdown: &ScoreBreakdown,
    played: &[Card],
    tables: &ScoreTables,
    trace: &[ScoreTraceStep],
) -> UiScoreBreakdown {
    let played_cards: Vec<UiCard> = played.iter().map(snapshot_card).collect();
    let mut scoring_cards = Vec::new();
    for &idx in &breakdown.scoring_indices {
        if let Some(card) = played.get(idx) {
            let chips = if card.is_stone() {
                0
            } else {
                tables.rank_chips(card.rank)
            };
            scoring_cards.push(UiScoringCard {
                index: idx,
                card: snapshot_card(card),
                chips,
            });
        }
    }
    UiScoreBreakdown {
        hand: breakdown.hand,
        base_chips: breakdown.base.chips,
        base_mult: breakdown.base.mult,
        rank_chips: breakdown.rank_chips,
        scoring_indices: breakdown.scoring_indices.clone(),
        total_chips: breakdown.total.chips,
        total_mult: breakdown.total.mult,
        total_score: breakdown.total.total(),
        played_cards,
        scoring_cards,
        steps: trace
            .iter()
            .map(|step| UiScoreStep {
                source: step.source.clone(),
                effect: format_rule_effect(&step.effect),
                before_chips: step.before.chips,
                before_mult: step.before.mult,
                after_chips: step.after.chips,
                after_mult: step.after.mult,
            })
            .collect(),
    }
}

fn snapshot_card(card: &Card) -> UiCard {
    UiCard {
        id: card.id,
        suit: card.suit,
        rank: card.rank,
        enhancement: card.enhancement,
        edition: card.edition,
        seal: card.seal,
        bonus_chips: card.bonus_chips,
        face_down: card.face_down,
    }
}

fn format_rule_effect(effect: &RuleEffect) -> String {
    match effect {
        RuleEffect::AddChips(value) => format!("+{} chips", value),
        RuleEffect::AddMult(value) => format!("+{:.2} mult", value),
        RuleEffect::MultiplyMult(value) => format!("×{:.2} mult", value),
        RuleEffect::MultiplyChips(value) => format!("×{:.2} chips", value),
    }
}

fn apply_action(state: &mut AppState, req: ActionRequest) -> Option<String> {
    match req.action.as_str() {
        "reset" => {
            let locale = state.locale.clone();
            let seed = req
                .target
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or_else(|| state.run.rng.seed());
            *state = AppState::new_with_seed(&locale, seed);
            None
        }
        "start_blind" => {
            let ante = req
                .target
                .as_deref()
                .and_then(|value| value.parse::<u8>().ok())
                .unwrap_or(1);
            state.last_breakdown = None;
            state.last_played.clear();
            state
                .run
                .start_blind(ante, state.run.state.blind, &mut state.events)
                .map_err(|err| err.to_string())
                .err()
        }
        "deal" => state
            .run
            .prepare_hand(&mut state.events)
            .map_err(|err| err.to_string())
            .err(),
        "play" => {
            let preview = match collect_played_preview(&state.run.hand, &req.indices) {
                Ok(cards) => cards,
                Err(err) => return Some(err),
            };
            match state.run.play_hand(&req.indices, &mut state.events) {
                Ok(breakdown) => {
                    state.last_breakdown = Some(breakdown);
                    state.last_played = preview;
                    None
                }
                Err(err) => Some(err.to_string()),
            }
        }
        "discard" => state
            .run
            .discard(&req.indices, &mut state.events)
            .map_err(|err| err.to_string())
            .err(),
        "enter_shop" => state
            .run
            .enter_shop(&mut state.events)
            .map_err(|err| err.to_string())
            .err(),
        "leave_shop" => {
            state.run.leave_shop();
            state.open_pack = None;
            None
        }
        "reroll" => state
            .run
            .reroll_shop(&mut state.events)
            .map_err(|err| err.to_string())
            .err(),
        "buy_card" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            handle_purchase(state, ShopOfferRef::Card(idx))
        }
        "buy_pack" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            handle_purchase(state, ShopOfferRef::Pack(idx))
        }
        "buy_voucher" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            handle_purchase(state, ShopOfferRef::Voucher(idx))
        }
        "open_pack" => {
            if let Some(open) = state.open_pack.as_ref() {
                let _ = open;
                return None;
            }
            if let Some(open) = state.open_pack.take() {
                let _ = open;
            }
            if let Some(shop) = state.run.shop.as_ref() {
                if shop.packs.is_empty() {
                    return Some("no packs available".to_string());
                }
            }
            let purchase = match state
                .run
                .buy_shop_offer(ShopOfferRef::Pack(0), &mut state.events)
            {
                Ok(purchase) => purchase,
                Err(err) => return Some(err.to_string()),
            };
            match state.run.open_pack_purchase(&purchase, &mut state.events) {
                Ok(open) => {
                    state.open_pack = Some(open);
                    None
                }
                Err(err) => Some(err.to_string()),
            }
        }
        "pick_pack" => {
            if let Some(open) = state.open_pack.clone() {
                match state
                    .run
                    .choose_pack_options(&open, &req.indices, &mut state.events)
                {
                    Ok(_) => {
                        state.open_pack = None;
                        None
                    }
                    Err(err) => Some(err.to_string()),
                }
            } else {
                Some("no open pack".to_string())
            }
        }
        "skip_pack" => {
            if let Some(open) = state.open_pack.clone() {
                match state.run.skip_pack(&open, &mut state.events) {
                    Ok(_) => {
                        state.open_pack = None;
                        None
                    }
                    Err(err) => Some(err.to_string()),
                }
            } else {
                Some("no open pack".to_string())
            }
        }
        "skip_blind" => {
            state.last_breakdown = None;
            state.last_played.clear();
            state
                .run
                .skip_blind(&mut state.events)
                .map_err(|err| err.to_string())
                .err()
        }
        "use_consumable" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            state
                .run
                .use_consumable(idx, &req.indices, &mut state.events)
                .map_err(|err| err.to_string())
                .err()
        }
        "sell_joker" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            state
                .run
                .sell_joker(idx, &mut state.events)
                .map_err(|err| err.to_string())
                .err()
        }
        "next_blind" => {
            state.last_breakdown = None;
            state.last_played.clear();
            state
                .run
                .start_next_blind(&mut state.events)
                .map_err(|err| err.to_string())
                .err()
        }
        "start_next" => {
            state.last_breakdown = None;
            state.last_played.clear();
            state
                .run
                .start_next_blind(&mut state.events)
                .map_err(|err| err.to_string())
                .err()
        }
        "start" => {
            state.last_breakdown = None;
            state.last_played.clear();
            state
                .run
                .start_blind(
                    state.run.state.ante,
                    state.run.state.blind,
                    &mut state.events,
                )
                .map_err(|err| err.to_string())
                .err()
        }
        _ => Some("unknown action".to_string()),
    }
}

fn handle_purchase(state: &mut AppState, offer: ShopOfferRef) -> Option<String> {
    let purchase = match state.run.buy_shop_offer(offer, &mut state.events) {
        Ok(purchase) => purchase,
        Err(err) => return Some(err.to_string()),
    };
    match purchase {
        rulatro_core::ShopPurchase::Pack(_) => {
            match state.run.open_pack_purchase(&purchase, &mut state.events) {
                Ok(open) => {
                    state.open_pack = Some(open);
                    None
                }
                Err(err) => Some(err.to_string()),
            }
        }
        _ => state
            .run
            .apply_purchase(&purchase)
            .map_err(|err| err.to_string())
            .err(),
    }
}

fn index(target: Option<String>) -> Result<usize, String> {
    target
        .as_deref()
        .ok_or_else(|| "missing target index".to_string())?
        .parse::<usize>()
        .map_err(|_| "invalid index".to_string())
}

fn collect_played_preview(hand: &[Card], indices: &[usize]) -> Result<Vec<Card>, String> {
    if indices.is_empty() {
        return Err("no cards selected".to_string());
    }
    let mut unique = indices.to_vec();
    unique.sort_unstable();
    unique.dedup();
    if unique.iter().any(|&idx| idx >= hand.len()) {
        return Err("invalid card index".to_string());
    }
    unique.sort_unstable_by(|a, b| b.cmp(a));
    let mut picked = Vec::with_capacity(unique.len());
    for idx in unique {
        picked.push(hand[idx]);
    }
    Ok(picked)
}

fn find_joker_name(run: &RunState, id: &str) -> String {
    run.content
        .jokers
        .iter()
        .find(|joker| joker.id == id)
        .map(|joker| joker.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn find_consumable_name(run: &RunState, kind: ConsumableKind, id: &str) -> String {
    let list = match kind {
        ConsumableKind::Tarot => &run.content.tarots,
        ConsumableKind::Planet => &run.content.planets,
        ConsumableKind::Spectral => &run.content.spectrals,
    };
    list.iter()
        .find(|card| card.id == id)
        .map(|card| card.name.clone())
        .unwrap_or_else(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulatro_core::{Card, Rank, RuleEffect, Suit};

    fn sample_hand() -> Vec<Card> {
        vec![
            Card {
                id: 1,
                suit: Suit::Spades,
                rank: Rank::Ace,
                enhancement: None,
                edition: None,
                seal: None,
                bonus_chips: 0,
                face_down: false,
            },
            Card {
                id: 2,
                suit: Suit::Hearts,
                rank: Rank::King,
                enhancement: None,
                edition: None,
                seal: None,
                bonus_chips: 0,
                face_down: false,
            },
            Card {
                id: 3,
                suit: Suit::Clubs,
                rank: Rank::Queen,
                enhancement: None,
                edition: None,
                seal: None,
                bonus_chips: 0,
                face_down: false,
            },
            Card {
                id: 4,
                suit: Suit::Diamonds,
                rank: Rank::Jack,
                enhancement: None,
                edition: None,
                seal: None,
                bonus_chips: 0,
                face_down: false,
            },
            Card {
                id: 5,
                suit: Suit::Spades,
                rank: Rank::Ten,
                enhancement: None,
                edition: None,
                seal: None,
                bonus_chips: 0,
                face_down: false,
            },
            Card {
                id: 6,
                suit: Suit::Hearts,
                rank: Rank::Nine,
                enhancement: None,
                edition: None,
                seal: None,
                bonus_chips: 0,
                face_down: false,
            },
        ]
    }

    macro_rules! index_ok_case {
        ($name:ident, $value:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(index(Some($value.to_string())).expect("idx"), $expected);
            }
        };
    }
    index_ok_case!(index_ok_0, "0", 0);
    index_ok_case!(index_ok_1, "1", 1);
    index_ok_case!(index_ok_2, "2", 2);
    index_ok_case!(index_ok_3, "3", 3);
    index_ok_case!(index_ok_4, "4", 4);
    index_ok_case!(index_ok_5, "5", 5);
    index_ok_case!(index_ok_6, "6", 6);
    index_ok_case!(index_ok_7, "7", 7);
    index_ok_case!(index_ok_8, "8", 8);
    index_ok_case!(index_ok_9, "9", 9);
    index_ok_case!(index_ok_10, "10", 10);
    index_ok_case!(index_ok_11, "11", 11);
    index_ok_case!(index_ok_12, "12", 12);
    index_ok_case!(index_ok_13, "13", 13);
    index_ok_case!(index_ok_14, "14", 14);
    index_ok_case!(index_ok_15, "15", 15);
    index_ok_case!(index_ok_16, "16", 16);
    index_ok_case!(index_ok_17, "17", 17);
    index_ok_case!(index_ok_18, "18", 18);
    index_ok_case!(index_ok_19, "19", 19);
    index_ok_case!(index_ok_20, "20", 20);
    index_ok_case!(index_ok_21, "21", 21);
    index_ok_case!(index_ok_22, "22", 22);
    index_ok_case!(index_ok_23, "23", 23);
    index_ok_case!(index_ok_24, "24", 24);
    index_ok_case!(index_ok_25, "25", 25);
    index_ok_case!(index_ok_26, "26", 26);
    index_ok_case!(index_ok_27, "27", 27);
    index_ok_case!(index_ok_28, "28", 28);
    index_ok_case!(index_ok_29, "29", 29);
    index_ok_case!(index_ok_30, "30", 30);
    index_ok_case!(index_ok_31, "31", 31);
    index_ok_case!(index_ok_32, "32", 32);
    index_ok_case!(index_ok_33, "33", 33);
    index_ok_case!(index_ok_34, "34", 34);
    index_ok_case!(index_ok_35, "35", 35);
    index_ok_case!(index_ok_36, "36", 36);
    index_ok_case!(index_ok_37, "37", 37);
    index_ok_case!(index_ok_38, "38", 38);
    index_ok_case!(index_ok_39, "39", 39);
    index_ok_case!(index_ok_40, "40", 40);
    index_ok_case!(index_ok_41, "41", 41);
    index_ok_case!(index_ok_42, "42", 42);
    index_ok_case!(index_ok_43, "43", 43);
    index_ok_case!(index_ok_44, "44", 44);
    index_ok_case!(index_ok_45, "45", 45);
    index_ok_case!(index_ok_46, "46", 46);
    index_ok_case!(index_ok_47, "47", 47);
    index_ok_case!(index_ok_48, "48", 48);
    index_ok_case!(index_ok_49, "49", 49);
    index_ok_case!(index_ok_50, "50", 50);
    index_ok_case!(index_ok_51, "51", 51);
    index_ok_case!(index_ok_52, "52", 52);
    index_ok_case!(index_ok_53, "53", 53);
    index_ok_case!(index_ok_54, "54", 54);
    index_ok_case!(index_ok_55, "55", 55);
    index_ok_case!(index_ok_56, "56", 56);
    index_ok_case!(index_ok_57, "57", 57);
    index_ok_case!(index_ok_58, "58", 58);
    index_ok_case!(index_ok_59, "59", 59);
    index_ok_case!(index_ok_60, "60", 60);
    index_ok_case!(index_ok_61, "61", 61);
    index_ok_case!(index_ok_62, "62", 62);
    index_ok_case!(index_ok_63, "63", 63);
    index_ok_case!(index_ok_64, "64", 64);
    index_ok_case!(index_ok_65, "65", 65);
    index_ok_case!(index_ok_66, "66", 66);
    index_ok_case!(index_ok_67, "67", 67);
    index_ok_case!(index_ok_68, "68", 68);
    index_ok_case!(index_ok_69, "69", 69);
    index_ok_case!(index_ok_70, "70", 70);
    index_ok_case!(index_ok_71, "71", 71);
    index_ok_case!(index_ok_72, "72", 72);
    index_ok_case!(index_ok_73, "73", 73);
    index_ok_case!(index_ok_74, "74", 74);
    index_ok_case!(index_ok_75, "75", 75);
    index_ok_case!(index_ok_76, "76", 76);
    index_ok_case!(index_ok_77, "77", 77);
    index_ok_case!(index_ok_78, "78", 78);
    index_ok_case!(index_ok_79, "79", 79);
    index_ok_case!(index_ok_80, "80", 80);
    index_ok_case!(index_ok_81, "81", 81);
    index_ok_case!(index_ok_82, "82", 82);
    index_ok_case!(index_ok_83, "83", 83);
    index_ok_case!(index_ok_84, "84", 84);
    index_ok_case!(index_ok_85, "85", 85);
    index_ok_case!(index_ok_86, "86", 86);
    index_ok_case!(index_ok_87, "87", 87);
    index_ok_case!(index_ok_88, "88", 88);
    index_ok_case!(index_ok_89, "89", 89);
    index_ok_case!(index_ok_90, "90", 90);
    index_ok_case!(index_ok_91, "91", 91);
    index_ok_case!(index_ok_92, "92", 92);
    index_ok_case!(index_ok_93, "93", 93);
    index_ok_case!(index_ok_94, "94", 94);
    index_ok_case!(index_ok_95, "95", 95);
    index_ok_case!(index_ok_96, "96", 96);
    index_ok_case!(index_ok_97, "97", 97);
    index_ok_case!(index_ok_98, "98", 98);
    index_ok_case!(index_ok_99, "99", 99);
    index_ok_case!(index_ok_100, "100", 100);
    index_ok_case!(index_ok_101, "101", 101);
    index_ok_case!(index_ok_102, "102", 102);
    index_ok_case!(index_ok_103, "103", 103);
    index_ok_case!(index_ok_104, "104", 104);
    index_ok_case!(index_ok_105, "105", 105);
    index_ok_case!(index_ok_106, "106", 106);
    index_ok_case!(index_ok_107, "107", 107);
    index_ok_case!(index_ok_108, "108", 108);
    index_ok_case!(index_ok_109, "109", 109);
    index_ok_case!(index_ok_110, "110", 110);
    index_ok_case!(index_ok_111, "111", 111);
    index_ok_case!(index_ok_112, "112", 112);
    index_ok_case!(index_ok_113, "113", 113);
    index_ok_case!(index_ok_114, "114", 114);
    index_ok_case!(index_ok_115, "115", 115);
    index_ok_case!(index_ok_116, "116", 116);
    index_ok_case!(index_ok_117, "117", 117);
    index_ok_case!(index_ok_118, "118", 118);
    index_ok_case!(index_ok_119, "119", 119);
    index_ok_case!(index_ok_120, "120", 120);
    index_ok_case!(index_ok_121, "121", 121);
    index_ok_case!(index_ok_122, "122", 122);
    index_ok_case!(index_ok_123, "123", 123);
    index_ok_case!(index_ok_124, "124", 124);
    index_ok_case!(index_ok_125, "125", 125);
    index_ok_case!(index_ok_126, "126", 126);
    index_ok_case!(index_ok_127, "127", 127);
    index_ok_case!(index_ok_128, "128", 128);
    index_ok_case!(index_ok_129, "129", 129);
    index_ok_case!(index_ok_130, "130", 130);
    index_ok_case!(index_ok_131, "131", 131);
    index_ok_case!(index_ok_132, "132", 132);
    index_ok_case!(index_ok_133, "133", 133);
    index_ok_case!(index_ok_134, "134", 134);
    index_ok_case!(index_ok_135, "135", 135);
    index_ok_case!(index_ok_136, "136", 136);
    index_ok_case!(index_ok_137, "137", 137);
    index_ok_case!(index_ok_138, "138", 138);
    index_ok_case!(index_ok_139, "139", 139);
    index_ok_case!(index_ok_140, "140", 140);
    index_ok_case!(index_ok_141, "141", 141);
    index_ok_case!(index_ok_142, "142", 142);
    index_ok_case!(index_ok_143, "143", 143);
    index_ok_case!(index_ok_144, "144", 144);
    index_ok_case!(index_ok_145, "145", 145);
    index_ok_case!(index_ok_146, "146", 146);
    index_ok_case!(index_ok_147, "147", 147);
    index_ok_case!(index_ok_148, "148", 148);
    index_ok_case!(index_ok_149, "149", 149);
    index_ok_case!(index_ok_150, "150", 150);
    index_ok_case!(index_ok_151, "151", 151);
    index_ok_case!(index_ok_152, "152", 152);
    index_ok_case!(index_ok_153, "153", 153);
    index_ok_case!(index_ok_154, "154", 154);
    index_ok_case!(index_ok_155, "155", 155);
    index_ok_case!(index_ok_156, "156", 156);
    index_ok_case!(index_ok_157, "157", 157);
    index_ok_case!(index_ok_158, "158", 158);
    index_ok_case!(index_ok_159, "159", 159);
    index_ok_case!(index_ok_160, "160", 160);
    index_ok_case!(index_ok_161, "161", 161);
    index_ok_case!(index_ok_162, "162", 162);
    index_ok_case!(index_ok_163, "163", 163);
    index_ok_case!(index_ok_164, "164", 164);
    index_ok_case!(index_ok_165, "165", 165);
    index_ok_case!(index_ok_166, "166", 166);
    index_ok_case!(index_ok_167, "167", 167);
    index_ok_case!(index_ok_168, "168", 168);
    index_ok_case!(index_ok_169, "169", 169);
    index_ok_case!(index_ok_170, "170", 170);
    index_ok_case!(index_ok_171, "171", 171);
    index_ok_case!(index_ok_172, "172", 172);
    index_ok_case!(index_ok_173, "173", 173);
    index_ok_case!(index_ok_174, "174", 174);
    index_ok_case!(index_ok_175, "175", 175);
    index_ok_case!(index_ok_176, "176", 176);
    index_ok_case!(index_ok_177, "177", 177);
    index_ok_case!(index_ok_178, "178", 178);
    index_ok_case!(index_ok_179, "179", 179);

    macro_rules! index_err_case {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                let out = index($value);
                assert!(out.is_err());
            }
        };
    }
    index_err_case!(index_err_0, None);
    index_err_case!(index_err_1, Some("".to_string()));
    index_err_case!(index_err_2, Some("bad0".to_string()));
    index_err_case!(index_err_3, Some("bad1".to_string()));
    index_err_case!(index_err_4, Some("bad2".to_string()));
    index_err_case!(index_err_5, Some("bad3".to_string()));
    index_err_case!(index_err_6, Some("bad4".to_string()));
    index_err_case!(index_err_7, Some("bad5".to_string()));
    index_err_case!(index_err_8, Some("bad6".to_string()));
    index_err_case!(index_err_9, Some("bad7".to_string()));
    index_err_case!(index_err_10, Some("bad8".to_string()));
    index_err_case!(index_err_11, Some("bad9".to_string()));
    index_err_case!(index_err_12, Some("bad10".to_string()));
    index_err_case!(index_err_13, Some("bad11".to_string()));
    index_err_case!(index_err_14, Some("bad12".to_string()));
    index_err_case!(index_err_15, Some("bad13".to_string()));
    index_err_case!(index_err_16, Some("bad14".to_string()));
    index_err_case!(index_err_17, Some("bad15".to_string()));
    index_err_case!(index_err_18, Some("bad16".to_string()));
    index_err_case!(index_err_19, Some("bad17".to_string()));
    index_err_case!(index_err_20, Some("bad18".to_string()));
    index_err_case!(index_err_21, Some("bad19".to_string()));
    index_err_case!(index_err_22, Some("bad20".to_string()));
    index_err_case!(index_err_23, Some("bad21".to_string()));
    index_err_case!(index_err_24, Some("bad22".to_string()));
    index_err_case!(index_err_25, Some("bad23".to_string()));
    index_err_case!(index_err_26, Some("bad24".to_string()));
    index_err_case!(index_err_27, Some("bad25".to_string()));
    index_err_case!(index_err_28, Some("bad26".to_string()));
    index_err_case!(index_err_29, Some("bad27".to_string()));
    index_err_case!(index_err_30, Some("bad28".to_string()));
    index_err_case!(index_err_31, Some("bad29".to_string()));
    index_err_case!(index_err_32, Some("bad30".to_string()));
    index_err_case!(index_err_33, Some("bad31".to_string()));
    index_err_case!(index_err_34, Some("bad32".to_string()));
    index_err_case!(index_err_35, Some("bad33".to_string()));
    index_err_case!(index_err_36, Some("bad34".to_string()));
    index_err_case!(index_err_37, Some("bad35".to_string()));
    index_err_case!(index_err_38, Some("bad36".to_string()));
    index_err_case!(index_err_39, Some("bad37".to_string()));
    index_err_case!(index_err_40, Some("bad38".to_string()));
    index_err_case!(index_err_41, Some("bad39".to_string()));
    index_err_case!(index_err_42, Some("-0".to_string()));
    index_err_case!(index_err_43, Some("-1".to_string()));
    index_err_case!(index_err_44, Some("-2".to_string()));
    index_err_case!(index_err_45, Some("-3".to_string()));
    index_err_case!(index_err_46, Some("-4".to_string()));
    index_err_case!(index_err_47, Some("-5".to_string()));
    index_err_case!(index_err_48, Some("-6".to_string()));
    index_err_case!(index_err_49, Some("-7".to_string()));
    index_err_case!(index_err_50, Some("-8".to_string()));
    index_err_case!(index_err_51, Some("-9".to_string()));
    index_err_case!(index_err_52, Some("-10".to_string()));
    index_err_case!(index_err_53, Some("-11".to_string()));
    index_err_case!(index_err_54, Some("-12".to_string()));
    index_err_case!(index_err_55, Some("-13".to_string()));
    index_err_case!(index_err_56, Some("-14".to_string()));
    index_err_case!(index_err_57, Some("-15".to_string()));
    index_err_case!(index_err_58, Some("-16".to_string()));
    index_err_case!(index_err_59, Some("-17".to_string()));
    index_err_case!(index_err_60, Some("-18".to_string()));
    index_err_case!(index_err_61, Some("-19".to_string()));
    index_err_case!(index_err_62, Some("0.5".to_string()));
    index_err_case!(index_err_63, Some("1.5".to_string()));
    index_err_case!(index_err_64, Some("2.5".to_string()));
    index_err_case!(index_err_65, Some("3.5".to_string()));
    index_err_case!(index_err_66, Some("4.5".to_string()));
    index_err_case!(index_err_67, Some("5.5".to_string()));
    index_err_case!(index_err_68, Some("6.5".to_string()));
    index_err_case!(index_err_69, Some("7.5".to_string()));
    index_err_case!(index_err_70, Some("8.5".to_string()));
    index_err_case!(index_err_71, Some("9.5".to_string()));
    index_err_case!(index_err_72, Some("10.5".to_string()));
    index_err_case!(index_err_73, Some("11.5".to_string()));
    index_err_case!(index_err_74, Some("12.5".to_string()));
    index_err_case!(index_err_75, Some("13.5".to_string()));
    index_err_case!(index_err_76, Some("14.5".to_string()));
    index_err_case!(index_err_77, Some("15.5".to_string()));
    index_err_case!(index_err_78, Some("16.5".to_string()));
    index_err_case!(index_err_79, Some("17.5".to_string()));
    index_err_case!(index_err_80, Some("18.5".to_string()));
    index_err_case!(index_err_81, Some("19.5".to_string()));

    macro_rules! add_chips_case {
        ($name:ident, $value:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(format_rule_effect(&RuleEffect::AddChips($value)), $expected);
            }
        };
    }
    add_chips_case!(rule_add_chips_0, 0, "+0 chips");
    add_chips_case!(rule_add_chips_1, 1, "+1 chips");
    add_chips_case!(rule_add_chips_2, 2, "+2 chips");
    add_chips_case!(rule_add_chips_3, 3, "+3 chips");
    add_chips_case!(rule_add_chips_4, 4, "+4 chips");
    add_chips_case!(rule_add_chips_5, 5, "+5 chips");
    add_chips_case!(rule_add_chips_6, 6, "+6 chips");
    add_chips_case!(rule_add_chips_7, 7, "+7 chips");
    add_chips_case!(rule_add_chips_8, 8, "+8 chips");
    add_chips_case!(rule_add_chips_9, 9, "+9 chips");
    add_chips_case!(rule_add_chips_10, 10, "+10 chips");
    add_chips_case!(rule_add_chips_11, 11, "+11 chips");
    add_chips_case!(rule_add_chips_12, 12, "+12 chips");
    add_chips_case!(rule_add_chips_13, 13, "+13 chips");
    add_chips_case!(rule_add_chips_14, 14, "+14 chips");
    add_chips_case!(rule_add_chips_15, 15, "+15 chips");
    add_chips_case!(rule_add_chips_16, 16, "+16 chips");
    add_chips_case!(rule_add_chips_17, 17, "+17 chips");
    add_chips_case!(rule_add_chips_18, 18, "+18 chips");
    add_chips_case!(rule_add_chips_19, 19, "+19 chips");
    add_chips_case!(rule_add_chips_20, 20, "+20 chips");
    add_chips_case!(rule_add_chips_21, 21, "+21 chips");
    add_chips_case!(rule_add_chips_22, 22, "+22 chips");
    add_chips_case!(rule_add_chips_23, 23, "+23 chips");
    add_chips_case!(rule_add_chips_24, 24, "+24 chips");
    add_chips_case!(rule_add_chips_25, 25, "+25 chips");
    add_chips_case!(rule_add_chips_26, 26, "+26 chips");
    add_chips_case!(rule_add_chips_27, 27, "+27 chips");
    add_chips_case!(rule_add_chips_28, 28, "+28 chips");
    add_chips_case!(rule_add_chips_29, 29, "+29 chips");
    add_chips_case!(rule_add_chips_30, 30, "+30 chips");
    add_chips_case!(rule_add_chips_31, 31, "+31 chips");
    add_chips_case!(rule_add_chips_32, 32, "+32 chips");
    add_chips_case!(rule_add_chips_33, 33, "+33 chips");
    add_chips_case!(rule_add_chips_34, 34, "+34 chips");
    add_chips_case!(rule_add_chips_35, 35, "+35 chips");
    add_chips_case!(rule_add_chips_36, 36, "+36 chips");
    add_chips_case!(rule_add_chips_37, 37, "+37 chips");
    add_chips_case!(rule_add_chips_38, 38, "+38 chips");
    add_chips_case!(rule_add_chips_39, 39, "+39 chips");

    macro_rules! add_mult_case {
        ($name:ident, $value:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let _ = $expected;
                assert_eq!(
                    format_rule_effect(&RuleEffect::AddMult($value)),
                    format!("+{:.2} mult", $value)
                );
            }
        };
    }
    add_mult_case!(rule_add_mult_0, 0.25, "+0.25 mult");
    add_mult_case!(rule_add_mult_1, 0.5, "+0.5 mult");
    add_mult_case!(rule_add_mult_2, 0.75, "+0.75 mult");
    add_mult_case!(rule_add_mult_3, 1.0, "+1.0 mult");
    add_mult_case!(rule_add_mult_4, 1.25, "+1.2 mult");
    add_mult_case!(rule_add_mult_5, 1.5, "+1.5 mult");
    add_mult_case!(rule_add_mult_6, 1.75, "+1.8 mult");
    add_mult_case!(rule_add_mult_7, 2.0, "+2.0 mult");
    add_mult_case!(rule_add_mult_8, 2.25, "+2.2 mult");
    add_mult_case!(rule_add_mult_9, 2.5, "+2.5 mult");
    add_mult_case!(rule_add_mult_10, 2.75, "+2.8 mult");
    add_mult_case!(rule_add_mult_11, 3.0, "+3.0 mult");
    add_mult_case!(rule_add_mult_12, 3.25, "+3.2 mult");
    add_mult_case!(rule_add_mult_13, 3.5, "+3.5 mult");
    add_mult_case!(rule_add_mult_14, 3.75, "+3.8 mult");
    add_mult_case!(rule_add_mult_15, 4.0, "+4.0 mult");
    add_mult_case!(rule_add_mult_16, 4.25, "+4.2 mult");
    add_mult_case!(rule_add_mult_17, 4.5, "+4.5 mult");
    add_mult_case!(rule_add_mult_18, 4.75, "+4.8 mult");
    add_mult_case!(rule_add_mult_19, 5.0, "+5.0 mult");
    add_mult_case!(rule_add_mult_20, 5.25, "+5.2 mult");
    add_mult_case!(rule_add_mult_21, 5.5, "+5.5 mult");
    add_mult_case!(rule_add_mult_22, 5.75, "+5.8 mult");
    add_mult_case!(rule_add_mult_23, 6.0, "+6.0 mult");
    add_mult_case!(rule_add_mult_24, 6.25, "+6.2 mult");
    add_mult_case!(rule_add_mult_25, 6.5, "+6.5 mult");
    add_mult_case!(rule_add_mult_26, 6.75, "+6.8 mult");
    add_mult_case!(rule_add_mult_27, 7.0, "+7.0 mult");
    add_mult_case!(rule_add_mult_28, 7.25, "+7.2 mult");
    add_mult_case!(rule_add_mult_29, 7.5, "+7.5 mult");
    add_mult_case!(rule_add_mult_30, 7.75, "+7.8 mult");
    add_mult_case!(rule_add_mult_31, 8.0, "+8.0 mult");
    add_mult_case!(rule_add_mult_32, 8.25, "+8.2 mult");
    add_mult_case!(rule_add_mult_33, 8.5, "+8.5 mult");
    add_mult_case!(rule_add_mult_34, 8.75, "+8.8 mult");
    add_mult_case!(rule_add_mult_35, 9.0, "+9.0 mult");
    add_mult_case!(rule_add_mult_36, 9.25, "+9.2 mult");
    add_mult_case!(rule_add_mult_37, 9.5, "+9.5 mult");
    add_mult_case!(rule_add_mult_38, 9.75, "+9.8 mult");
    add_mult_case!(rule_add_mult_39, 10.0, "+1e+01 mult");

    macro_rules! mul_mult_case {
        ($name:ident, $value:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let _ = $expected;
                assert_eq!(
                    format_rule_effect(&RuleEffect::MultiplyMult($value)),
                    format!("×{:.2} mult", $value)
                );
            }
        };
    }
    mul_mult_case!(rule_mul_mult_0, 1.1, "×1.1 mult");
    mul_mult_case!(rule_mul_mult_1, 1.2, "×1.2 mult");
    mul_mult_case!(rule_mul_mult_2, 1.3, "×1.3 mult");
    mul_mult_case!(rule_mul_mult_3, 1.4, "×1.4 mult");
    mul_mult_case!(rule_mul_mult_4, 1.5, "×1.5 mult");
    mul_mult_case!(rule_mul_mult_5, 1.6, "×1.6 mult");
    mul_mult_case!(rule_mul_mult_6, 1.7, "×1.7 mult");
    mul_mult_case!(rule_mul_mult_7, 1.8, "×1.8 mult");
    mul_mult_case!(rule_mul_mult_8, 1.9, "×1.9 mult");
    mul_mult_case!(rule_mul_mult_9, 2.0, "×2.0 mult");
    mul_mult_case!(rule_mul_mult_10, 2.1, "×2.1 mult");
    mul_mult_case!(rule_mul_mult_11, 2.2, "×2.2 mult");
    mul_mult_case!(rule_mul_mult_12, 2.3, "×2.3 mult");
    mul_mult_case!(rule_mul_mult_13, 2.4, "×2.4 mult");
    mul_mult_case!(rule_mul_mult_14, 2.5, "×2.5 mult");
    mul_mult_case!(rule_mul_mult_15, 2.6, "×2.6 mult");
    mul_mult_case!(rule_mul_mult_16, 2.7, "×2.7 mult");
    mul_mult_case!(rule_mul_mult_17, 2.8, "×2.8 mult");
    mul_mult_case!(rule_mul_mult_18, 2.9, "×2.9 mult");
    mul_mult_case!(rule_mul_mult_19, 3.0, "×3.0 mult");
    mul_mult_case!(rule_mul_mult_20, 3.1, "×3.1 mult");
    mul_mult_case!(rule_mul_mult_21, 3.2, "×3.2 mult");
    mul_mult_case!(rule_mul_mult_22, 3.3, "×3.3 mult");
    mul_mult_case!(rule_mul_mult_23, 3.4, "×3.4 mult");
    mul_mult_case!(rule_mul_mult_24, 3.5, "×3.5 mult");
    mul_mult_case!(rule_mul_mult_25, 3.6, "×3.6 mult");
    mul_mult_case!(rule_mul_mult_26, 3.7, "×3.7 mult");
    mul_mult_case!(rule_mul_mult_27, 3.8, "×3.8 mult");
    mul_mult_case!(rule_mul_mult_28, 3.9, "×3.9 mult");
    mul_mult_case!(rule_mul_mult_29, 4.0, "×4.0 mult");
    mul_mult_case!(rule_mul_mult_30, 4.1, "×4.1 mult");
    mul_mult_case!(rule_mul_mult_31, 4.2, "×4.2 mult");
    mul_mult_case!(rule_mul_mult_32, 4.3, "×4.3 mult");
    mul_mult_case!(rule_mul_mult_33, 4.4, "×4.4 mult");
    mul_mult_case!(rule_mul_mult_34, 4.5, "×4.5 mult");
    mul_mult_case!(rule_mul_mult_35, 4.6, "×4.6 mult");
    mul_mult_case!(rule_mul_mult_36, 4.7, "×4.7 mult");
    mul_mult_case!(rule_mul_mult_37, 4.8, "×4.8 mult");
    mul_mult_case!(rule_mul_mult_38, 4.9, "×4.9 mult");
    mul_mult_case!(rule_mul_mult_39, 5.0, "×5.0 mult");

    macro_rules! mul_chips_case {
        ($name:ident, $value:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let _ = $expected;
                assert_eq!(
                    format_rule_effect(&RuleEffect::MultiplyChips($value)),
                    format!("×{:.2} chips", $value)
                );
            }
        };
    }
    mul_chips_case!(rule_mul_chips_0, 1.1, "×1.1 chips");
    mul_chips_case!(rule_mul_chips_1, 1.2, "×1.2 chips");
    mul_chips_case!(rule_mul_chips_2, 1.3, "×1.3 chips");
    mul_chips_case!(rule_mul_chips_3, 1.4, "×1.4 chips");
    mul_chips_case!(rule_mul_chips_4, 1.5, "×1.5 chips");
    mul_chips_case!(rule_mul_chips_5, 1.6, "×1.6 chips");
    mul_chips_case!(rule_mul_chips_6, 1.7, "×1.7 chips");
    mul_chips_case!(rule_mul_chips_7, 1.8, "×1.8 chips");
    mul_chips_case!(rule_mul_chips_8, 1.9, "×1.9 chips");
    mul_chips_case!(rule_mul_chips_9, 2.0, "×2.0 chips");
    mul_chips_case!(rule_mul_chips_10, 2.1, "×2.1 chips");
    mul_chips_case!(rule_mul_chips_11, 2.2, "×2.2 chips");
    mul_chips_case!(rule_mul_chips_12, 2.3, "×2.3 chips");
    mul_chips_case!(rule_mul_chips_13, 2.4, "×2.4 chips");
    mul_chips_case!(rule_mul_chips_14, 2.5, "×2.5 chips");
    mul_chips_case!(rule_mul_chips_15, 2.6, "×2.6 chips");
    mul_chips_case!(rule_mul_chips_16, 2.7, "×2.7 chips");
    mul_chips_case!(rule_mul_chips_17, 2.8, "×2.8 chips");
    mul_chips_case!(rule_mul_chips_18, 2.9, "×2.9 chips");
    mul_chips_case!(rule_mul_chips_19, 3.0, "×3.0 chips");
    mul_chips_case!(rule_mul_chips_20, 3.1, "×3.1 chips");
    mul_chips_case!(rule_mul_chips_21, 3.2, "×3.2 chips");
    mul_chips_case!(rule_mul_chips_22, 3.3, "×3.3 chips");
    mul_chips_case!(rule_mul_chips_23, 3.4, "×3.4 chips");
    mul_chips_case!(rule_mul_chips_24, 3.5, "×3.5 chips");
    mul_chips_case!(rule_mul_chips_25, 3.6, "×3.6 chips");
    mul_chips_case!(rule_mul_chips_26, 3.7, "×3.7 chips");
    mul_chips_case!(rule_mul_chips_27, 3.8, "×3.8 chips");
    mul_chips_case!(rule_mul_chips_28, 3.9, "×3.9 chips");
    mul_chips_case!(rule_mul_chips_29, 4.0, "×4.0 chips");
    mul_chips_case!(rule_mul_chips_30, 4.1, "×4.1 chips");
    mul_chips_case!(rule_mul_chips_31, 4.2, "×4.2 chips");
    mul_chips_case!(rule_mul_chips_32, 4.3, "×4.3 chips");
    mul_chips_case!(rule_mul_chips_33, 4.4, "×4.4 chips");
    mul_chips_case!(rule_mul_chips_34, 4.5, "×4.5 chips");
    mul_chips_case!(rule_mul_chips_35, 4.6, "×4.6 chips");
    mul_chips_case!(rule_mul_chips_36, 4.7, "×4.7 chips");
    mul_chips_case!(rule_mul_chips_37, 4.8, "×4.8 chips");
    mul_chips_case!(rule_mul_chips_38, 4.9, "×4.9 chips");
    mul_chips_case!(rule_mul_chips_39, 5.0, "×5.0 chips");

    macro_rules! preview_case {
        ($name:ident, [$($input:expr),*], [$($expected:expr),*]) => {
            #[test]
            fn $name() {
                let hand = sample_hand();
                let out = collect_played_preview(&hand, &[$($input),*]).expect("preview");
                let ids: Vec<u32> = out.into_iter().map(|c| c.id).collect();
                let want: Vec<u32> = vec![$($expected),*].into_iter().map(|i| i as u32 + 1).collect();
                assert_eq!(ids, want);
            }
        };
    }
    preview_case!(preview_case_0, [0], [0]);
    preview_case!(preview_case_1, [1], [1]);
    preview_case!(preview_case_2, [0, 1], [1, 0]);
    preview_case!(preview_case_3, [1, 0], [1, 0]);
    preview_case!(preview_case_4, [2, 0], [2, 0]);
    preview_case!(preview_case_5, [0, 0, 1], [1, 0]);
    preview_case!(preview_case_6, [3, 1, 2], [3, 2, 1]);
    preview_case!(preview_case_7, [4, 3, 2, 1, 0], [4, 3, 2, 1, 0]);

    #[test]
    fn preview_rejects_empty_indices() {
        let hand = sample_hand();
        assert!(collect_played_preview(&hand, &[]).is_err());
    }

    #[test]
    fn preview_rejects_out_of_range_index() {
        let hand = sample_hand();
        assert!(collect_played_preview(&hand, &[999]).is_err());
    }

    #[test]
    fn web_path_points_to_web_assets() {
        let path = web_path("index.html");
        assert!(path.ends_with("web/index.html"));
    }
}
