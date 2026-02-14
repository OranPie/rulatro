use rulatro_core::{
    BlindKind, Card, ConsumableKind, EventBus, PackOpen, PackOption, Phase, RuleEffect, RunState,
    ScoreBreakdown, ScoreTables, ScoreTraceStep, ShopOfferRef,
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
    reroll_cost: i64,
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
        state: snapshot_state(&state.run, &state.content_signature),
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

fn snapshot_state(run: &RunState, content_signature: &str) -> UiState {
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
        reroll_cost: shop.reroll_cost,
    });
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
