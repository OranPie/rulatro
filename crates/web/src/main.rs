use rulatro_core::{
    BlindKind, BlindOutcome, Card, ConsumableKind, EventBus, PackOpen, PackOption, Phase, RunState,
    ShopOfferKind, ShopOfferRef,
};
use rulatro_data::{load_content_with_mods, load_game_config};
use rulatro_modding::ModManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn main() {
    let server = Server::http("0.0.0.0:7878").expect("start server");
    println!("Rulatro web server on http://localhost:7878");
    let state = Arc::new(Mutex::new(AppState::new()));
    for request in server.incoming_requests() {
        let state = state.clone();
        if let Err(err) = handle_request(request, state) {
            eprintln!("request error: {err}");
        }
    }
}

struct AppState {
    run: RunState,
    events: EventBus,
    open_pack: Option<PackOpen>,
}

impl AppState {
    fn new() -> Self {
        let config = load_game_config(Path::new("assets")).expect("load config");
        let modded = load_content_with_mods(Path::new("assets"), Path::new("mods"))
            .expect("load content");
        let mut runtime = ModManager::new();
        runtime
            .load_mods(&modded.mods)
            .expect("load mod runtime");
        let mut run = RunState::new(config, modded.content, 0xC0FFEE);
        run.set_mod_runtime(Some(Box::new(runtime)));
        Self {
            run,
            events: EventBus::default(),
            open_pack: None,
        }
    }
}

#[derive(Serialize)]
struct ApiResponse {
    ok: bool,
    error: Option<String>,
    state: UiState,
    events: Vec<rulatro_core::Event>,
    open_pack: Option<UiPackOpen>,
}

#[derive(Serialize)]
struct UiState {
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
    jokers: Vec<UiJoker>,
    consumables: Vec<UiConsumable>,
    shop: Option<UiShop>,
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
    rarity: rulatro_core::JokerRarity,
    edition: Option<rulatro_core::Edition>,
    buy_price: i64,
}

#[derive(Serialize)]
struct UiConsumable {
    id: String,
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
    Joker(String),
    Consumable { kind: ConsumableKind, id: String },
    PlayingCard(UiCard),
}

#[derive(Serialize)]
struct UiHandLevel {
    hand: rulatro_core::HandKind,
    level: u32,
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
            respond_with_file(&mut request, web_path("index.html"), "text/html; charset=utf-8")?;
        }
        (&Method::Get, "/app.js") => {
            respond_with_file(&mut request, web_path("app.js"), "application/javascript")?;
        }
        (&Method::Get, "/styles.css") => {
            respond_with_file(&mut request, web_path("styles.css"), "text/css; charset=utf-8")?;
        }
        (&Method::Get, "/api/state") => {
            let mut guard = state.lock().unwrap();
            let response = build_response(&mut *guard, None);
            respond_json(&mut request, response)?;
        }
        (&Method::Post, "/api/action") => {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body)?;
            let action: ActionRequest = serde_json::from_str(&body)?;
            let mut guard = state.lock().unwrap();
            let err = apply_action(&mut *guard, action);
            let response = build_response(&mut *guard, err);
            respond_json(&mut request, response)?;
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

fn respond_with_file(
    request: &mut tiny_http::Request,
    path: PathBuf,
    content_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    let header = Header::from_bytes(&b"Content-Type"[..], content_type)?;
    let response = Response::from_data(content).with_header(header);
    request.respond(response)?;
    Ok(())
}

fn respond_json(
    request: &mut tiny_http::Request,
    response: ApiResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_vec_pretty(&response)?;
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])?;
    request.respond(Response::from_data(body).with_header(header))?;
    Ok(())
}

fn build_response(state: &mut AppState, err: Option<String>) -> ApiResponse {
    let events: Vec<_> = state.events.drain().collect();
    ApiResponse {
        ok: err.is_none(),
        error: err,
        state: snapshot_state(&state.run),
        events,
        open_pack: state.open_pack.as_ref().map(snapshot_open_pack),
    }
}

fn snapshot_state(run: &RunState) -> UiState {
    let hand = run.hand.iter().map(snapshot_card).collect();
    let jokers = run
        .inventory
        .jokers
        .iter()
        .map(|joker| UiJoker {
            id: joker.id.clone(),
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
    UiState {
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
        jokers,
        consumables,
        shop,
        tags: run.state.tags.clone(),
        duplicate_next_tag: run.state.duplicate_next_tag,
        duplicate_tag_exclude: run.state.duplicate_tag_exclude.clone(),
        hand_levels,
    }
}

fn snapshot_open_pack(open: &PackOpen) -> UiPackOpen {
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
                PackOption::Joker(id) => UiPackOption::Joker(id.clone()),
                PackOption::Consumable(kind, id) => UiPackOption::Consumable {
                    kind: *kind,
                    id: id.clone(),
                },
                PackOption::PlayingCard(card) => UiPackOption::PlayingCard(snapshot_card(card)),
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

fn apply_action(state: &mut AppState, req: ActionRequest) -> Option<String> {
    let run = &mut state.run;
    let events = &mut state.events;
    match req.action.as_str() {
        "reset" => {
            *state = AppState::new();
            None
        }
        "start_blind" => {
            let ante = req
                .target
                .as_deref()
                .and_then(|value| value.parse::<u8>().ok())
                .unwrap_or(1);
            run.start_blind(ante, run.state.blind, events)
                .map_err(|err| format!("{err:?}"))
                .err()
        }
        "deal" => run
            .prepare_hand(events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        "play" => run
            .play_hand(&req.indices, events)
            .map_err(|err| format!("{err:?}"))
            .map(|_| ())
            .err(),
        "discard" => run
            .discard(&req.indices, events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        "enter_shop" => run
            .enter_shop(events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        "leave_shop" => {
            run.leave_shop();
            state.open_pack = None;
            None
        }
        "reroll" => run
            .reroll_shop(events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        "buy_card" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            handle_purchase(run, events, ShopOfferRef::Card(idx), state)
        }
        "buy_pack" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            handle_purchase(run, events, ShopOfferRef::Pack(idx), state)
        }
        "buy_voucher" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            handle_purchase(run, events, ShopOfferRef::Voucher(idx), state)
        }
        "open_pack" => {
            if let Some(open) = state.open_pack.as_ref() {
                let _ = open;
                return None;
            }
            if let Some(open) = state.open_pack.take() {
                let _ = open;
            }
            if let Some(shop) = run.shop.as_ref() {
                if shop.packs.is_empty() {
                    return Some("no packs available".to_string());
                }
            }
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Pack(0), events)
                .map_err(|err| format!("{err:?}"))?;
            match run.open_pack_purchase(&purchase, events) {
                Ok(open) => {
                    state.open_pack = Some(open);
                    None
                }
                Err(err) => Some(format!("{err:?}")),
            }
        }
        "pick_pack" => {
            if let Some(open) = state.open_pack.clone() {
                match run.choose_pack_options(&open, &req.indices, events) {
                    Ok(_) => {
                        state.open_pack = None;
                        None
                    }
                    Err(err) => Some(format!("{err:?}")),
                }
            } else {
                Some("no open pack".to_string())
            }
        }
        "skip_pack" => {
            if let Some(open) = state.open_pack.clone() {
                match run.skip_pack(&open, events) {
                    Ok(_) => {
                        state.open_pack = None;
                        None
                    }
                    Err(err) => Some(format!("{err:?}")),
                }
            } else {
                Some("no open pack".to_string())
            }
        }
        "use_consumable" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            run.use_consumable(idx, &req.indices, events)
                .map_err(|err| format!("{err:?}"))
                .err()
        }
        "sell_joker" => {
            let idx = match index(req.target) {
                Ok(idx) => idx,
                Err(err) => return Some(err),
            };
            run.sell_joker(idx, events)
                .map_err(|err| format!("{err:?}"))
                .err()
        }
        "next_blind" => run
            .start_next_blind(events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        "start_next" => run
            .start_next_blind(events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        "start" => run
            .start_blind(run.state.ante, run.state.blind, events)
            .map_err(|err| format!("{err:?}"))
            .err(),
        _ => Some("unknown action".to_string()),
    }
}

fn handle_purchase(
    run: &mut RunState,
    events: &mut EventBus,
    offer: ShopOfferRef,
    state: &mut AppState,
) -> Option<String> {
    let purchase = run
        .buy_shop_offer(offer, events)
        .map_err(|err| format!("{err:?}"))?;
    match purchase {
        rulatro_core::ShopPurchase::Pack(_) => match run.open_pack_purchase(&purchase, events) {
            Ok(open) => {
                state.open_pack = Some(open);
                None
            }
            Err(err) => Some(format!("{err:?}")),
        },
        _ => run
            .apply_purchase(&purchase)
            .map_err(|err| format!("{err:?}"))
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
