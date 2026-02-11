use rulatro_core::{
    BlindKind, BlindOutcome, Card, ConsumableKind, Edition, Enhancement, EventBus, PackOpen,
    PackOption, Phase, Rank, RunState, Seal, ShopOfferRef, Suit,
};
use rulatro_data::{load_content_with_mods, load_game_config};
use rulatro_modding::ModManager;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--auto") {
        run_auto();
        return;
    }
    run_cui();
}

fn run_auto() {
    let mut events = EventBus::default();
    let config = load_game_config(Path::new("assets")).expect("load config");
    let modded = load_content_with_mods(Path::new("assets"), Path::new("mods"))
        .expect("load content");
    if !modded.mods.is_empty() {
        println!("mods loaded: {}", modded.mods.len());
        for item in &modded.mods {
            println!("mod: {}", item.manifest.meta.id);
        }
    }
    for warning in &modded.warnings {
        eprintln!("mod warning: {}", warning);
    }
    let mut runtime = ModManager::new();
    runtime
        .load_mods(&modded.mods)
        .expect("load mod runtime");
    let mut run = RunState::new(config, modded.content, 0xC0FFEE);
    run.set_mod_runtime(Some(Box::new(runtime)));
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut blinds_completed = 0;
    loop {
        run.prepare_hand(&mut events).expect("prepare hand");

        let play_count = run.hand.len().min(5);
        let indices: Vec<usize> = (0..play_count).collect();
        let breakdown = run
            .play_hand(&indices, &mut events)
            .expect("play hand");

        println!(
            "hand: {:?}, chips: {}, mult: {:.2}, total: {}",
            breakdown.hand,
            breakdown.total.chips,
            breakdown.total.mult,
            breakdown.total.total()
        );
        println!(
            "blind score: {} / target: {}",
            run.state.blind_score, run.state.target
        );

        if let Some(outcome) = run.blind_outcome() {
            println!("blind outcome: {:?}", outcome);
            match outcome {
                BlindOutcome::Cleared => {
                    blinds_completed += 1;
                    run.enter_shop(&mut events).expect("enter shop");
                    if run.reroll_shop(&mut events).is_ok() {
                        println!("shop rerolled");
                    }
                    if let Ok(purchase) = run.buy_shop_offer(ShopOfferRef::Pack(0), &mut events) {
                        if let Ok(open) = run.open_pack_purchase(&purchase, &mut events) {
                            let _ = run.choose_pack_options(&open, &[0], &mut events);
                            println!("opened pack with {} options", open.options.len());
                        }
                    } else if let Ok(purchase) =
                        run.buy_shop_offer(ShopOfferRef::Card(0), &mut events)
                    {
                        let _ = run.apply_purchase(&purchase);
                        println!("bought card 0");
                    }
                    run.leave_shop();
                    if run.start_next_blind(&mut events).is_err() {
                        break;
                    }
                }
                BlindOutcome::Failed => {
                    break;
                }
            }
        }

        if blinds_completed >= 2 {
            break;
        }
    }

    for event in events.drain() {
        println!("event: {:?}", event);
    }
}

fn run_cui() {
    let mut events = EventBus::default();
    let config = load_game_config(Path::new("assets")).expect("load config");
    let modded = load_content_with_mods(Path::new("assets"), Path::new("mods"))
        .expect("load content");
    if !modded.mods.is_empty() {
        println!("mods loaded: {}", modded.mods.len());
        for item in &modded.mods {
            println!("mod: {}", item.manifest.meta.id);
        }
    }
    for warning in &modded.warnings {
        eprintln!("mod warning: {}", warning);
    }
    let mut runtime = ModManager::new();
    runtime
        .load_mods(&modded.mods)
        .expect("load mod runtime");
    let mut run = RunState::new(config, modded.content, 0xC0FFEE);
    run.set_mod_runtime(Some(Box::new(runtime)));
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut open_pack: Option<PackOpen> = None;
    print_help();
    loop {
        if let Some(outcome) = run.blind_outcome() {
            println!("blind outcome: {:?}", outcome);
        }
        print_prompt(&run, open_pack.as_ref());
        let line = match read_line() {
            Some(line) => line,
            None => break,
        };
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        let mut parts = input.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();
        match cmd {
            "help" | "h" | "?" => print_help(),
            "quit" | "exit" => break,
            "state" | "s" => print_state(&run),
            "hand" => print_hand(&run),
            "deck" => print_deck(&run),
            "inv" | "inventory" => print_inventory(&run),
            "deal" | "d" => {
                match run.prepare_hand(&mut events) {
                    Ok(_) => println!("dealt hand"),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            "play" | "p" => {
                let indices = parse_indices(&args);
                match indices {
                    Some(indices) => match run.play_hand(&indices, &mut events) {
                        Ok(breakdown) => {
                            println!(
                                "scored: {:?} chips={} mult={:.2} total={}",
                                breakdown.hand,
                                breakdown.total.chips,
                                breakdown.total.mult,
                                breakdown.total.total()
                            );
                        }
                        Err(err) => println!("error: {err:?}"),
                    },
                    None => println!("usage: play <idx> <idx> ..."),
                }
            }
            "discard" | "x" => {
                let indices = parse_indices(&args);
                match indices {
                    Some(indices) => match run.discard(&indices, &mut events) {
                        Ok(_) => println!("discarded"),
                        Err(err) => println!("error: {err:?}"),
                    },
                    None => println!("usage: discard <idx> <idx> ..."),
                }
            }
            "shop" => match run.enter_shop(&mut events) {
                Ok(_) => print_shop(&run),
                Err(err) => println!("error: {err:?}"),
            },
            "leave" => {
                run.leave_shop();
                open_pack = None;
                println!("left shop");
            }
            "reroll" | "r" => match run.reroll_shop(&mut events) {
                Ok(_) => print_shop(&run),
                Err(err) => println!("error: {err:?}"),
            },
            "buy" => {
                if args.len() < 2 {
                    println!("usage: buy card|pack|voucher <index>");
                } else {
                    let kind = args[0];
                    let index = args[1].parse::<usize>().ok();
                    match (kind, index) {
                        ("card", Some(idx)) => {
                            match run.buy_shop_offer(ShopOfferRef::Card(idx), &mut events) {
                                Ok(purchase) => {
                                    if let Err(err) = run.apply_purchase(&purchase) {
                                        println!("error: {err:?}");
                                    } else {
                                        println!("bought card {idx}");
                                    }
                                }
                                Err(err) => println!("error: {err:?}"),
                            }
                        }
                        ("pack", Some(idx)) => {
                            match run.buy_shop_offer(ShopOfferRef::Pack(idx), &mut events) {
                                Ok(purchase) => match run.open_pack_purchase(&purchase, &mut events) {
                                    Ok(open) => {
                                        print_pack_open(&open, &run);
                                        open_pack = Some(open);
                                    }
                                    Err(err) => println!("error: {err:?}"),
                                },
                                Err(err) => println!("error: {err:?}"),
                            }
                        }
                        ("voucher", Some(idx)) => {
                            match run.buy_shop_offer(ShopOfferRef::Voucher(idx), &mut events) {
                                Ok(purchase) => {
                                    if let Err(err) = run.apply_purchase(&purchase) {
                                        println!("error: {err:?}");
                                    } else {
                                        println!("bought voucher {idx}");
                                    }
                                }
                                Err(err) => println!("error: {err:?}"),
                            }
                        }
                        _ => println!("usage: buy card|pack|voucher <index>"),
                    }
                }
            }
            "pick" => {
                if let Some(open) = open_pack.clone() {
                    let indices = parse_indices(&args);
                    match indices {
                        Some(indices) => match run.choose_pack_options(&open, &indices, &mut events) {
                            Ok(_) => {
                                println!("picked pack options");
                                open_pack = None;
                            }
                            Err(err) => println!("error: {err:?}"),
                        },
                        None => println!("usage: pick <idx> <idx> ..."),
                    }
                } else {
                    println!("no open pack");
                }
            }
            "skip" => {
                if let Some(open) = open_pack.clone() {
                    match run.skip_pack(&open, &mut events) {
                        Ok(_) => {
                            println!("skipped pack");
                            open_pack = None;
                        }
                        Err(err) => println!("error: {err:?}"),
                    }
                } else {
                    println!("no open pack");
                }
            }
            "use" => {
                if args.is_empty() {
                    println!("usage: use <consumable_index> [selected idxs]");
                    continue;
                }
                let idx = match args[0].parse::<usize>() {
                    Ok(idx) => idx,
                    Err(_) => {
                        println!("invalid index");
                        continue;
                    }
                };
                let selected = parse_indices(&args[1..]).unwrap_or_default();
                match run.use_consumable(idx, &selected, &mut events) {
                    Ok(_) => println!("consumable used"),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            "sell" => {
                if args.len() != 1 {
                    println!("usage: sell <joker_index>");
                    continue;
                }
                match args[0].parse::<usize>() {
                    Ok(idx) => match run.sell_joker(idx, &mut events) {
                        Ok(_) => println!("sold joker {idx}"),
                        Err(err) => println!("error: {err:?}"),
                    },
                    Err(_) => println!("invalid index"),
                }
            }
            "next" => {
                open_pack = None;
                match run.start_next_blind(&mut events) {
                    Ok(_) => println!("started next blind"),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            _ => println!("unknown command: {cmd} (type 'help')"),
        }
        drain_events(&mut events);
    }
}

fn print_help() {
    println!("commands:");
    println!("  help                show this help");
    println!("  state               show run state");
    println!("  hand                show current hand");
    println!("  deck                show deck sizes");
    println!("  inv                 show inventory");
    println!("  deal                draw to hand (phase Deal)");
    println!("  play <idx..>        play cards");
    println!("  discard <idx..>     discard cards");
    println!("  shop                enter shop");
    println!("  reroll              reroll shop");
    println!("  buy card|pack|voucher <idx>");
    println!("  pick <idx..>        pick open pack options");
    println!("  skip                skip open pack");
    println!("  use <idx> [sel..]   use consumable");
    println!("  sell <idx>          sell joker");
    println!("  leave               leave shop");
    println!("  next                start next blind");
    println!("  quit                exit");
    println!("tip: run with --auto for scripted demo");
}

fn print_prompt(run: &RunState, open_pack: Option<&PackOpen>) {
    let pack = if open_pack.is_some() { " pack-open" } else { "" };
    print!(
        "[ante {} {:?} {:?} money {} score {}/{}{}] > ",
        run.state.ante, run.state.blind, run.state.phase, run.state.money, run.state.blind_score,
        run.state.target, pack
    );
    let _ = io::stdout().flush();
}

fn read_line() -> Option<String> {
    let mut line = String::new();
    if io::stdin().read_line(&mut line).ok()? == 0 {
        return None;
    }
    Some(line)
}

fn print_state(run: &RunState) {
    println!(
        "ante {} blind {:?} phase {:?}",
        run.state.ante, run.state.blind, run.state.phase
    );
    println!(
        "target {} score {} hands {}/{} discards {}/{}",
        run.state.target,
        run.state.blind_score,
        run.state.hands_left,
        run.state.hands_max,
        run.state.discards_left,
        run.state.discards_max
    );
    println!(
        "money {} hand_size {}/{}",
        run.state.money, run.state.hand_size, run.state.hand_size_base
    );
    println!(
        "deck draw {} discard {}",
        run.deck.draw.len(),
        run.deck.discard.len()
    );
}

fn print_hand(run: &RunState) {
    println!("hand ({} cards):", run.hand.len());
    for (idx, card) in run.hand.iter().enumerate() {
        println!("{:>2}: {}", idx, format_card(card));
    }
}

fn print_deck(run: &RunState) {
    println!("draw pile: {}", run.deck.draw.len());
    println!("discard pile: {}", run.deck.discard.len());
}

fn print_inventory(run: &RunState) {
    println!("jokers ({}/{}):", run.inventory.jokers.len(), run.inventory.joker_capacity());
    for (idx, joker) in run.inventory.jokers.iter().enumerate() {
        let edition = joker.edition.map(edition_short).unwrap_or("");
        let suffix = if edition.is_empty() {
            "".to_string()
        } else {
            format!(" [{edition}]")
        };
        println!("{:>2}: {}{} ({:?})", idx, joker.id, suffix, joker.rarity);
    }
    println!(
        "consumables ({}/{}):",
        run.inventory.consumable_count(),
        run.inventory.consumable_slots
    );
    for (idx, item) in run.inventory.consumables.iter().enumerate() {
        let edition = item.edition.map(edition_short).unwrap_or("");
        let suffix = if edition.is_empty() {
            "".to_string()
        } else {
            format!(" [{edition}]")
        };
        println!("{:>2}: {} {:?}{}", idx, item.id, item.kind, suffix);
    }
}

fn print_shop(run: &RunState) {
    let Some(shop) = &run.shop else {
        println!("shop not available");
        return;
    };
    println!("shop cards:");
    for (idx, card) in shop.cards.iter().enumerate() {
        println!(
            "{:>2}: {:?} {} price {}",
            idx, card.kind, card.item_id, card.price
        );
    }
    println!("shop packs:");
    for (idx, pack) in shop.packs.iter().enumerate() {
        println!(
            "{:>2}: {:?} {:?} options {} picks {} price {}",
            idx, pack.kind, pack.size, pack.options, pack.picks, pack.price
        );
    }
    println!("vouchers: {}", shop.vouchers);
    println!("reroll cost: {}", shop.reroll_cost);
}

fn print_pack_open(open: &PackOpen, run: &RunState) {
    println!(
        "pack opened: {:?} {:?} (pick {})",
        open.offer.kind, open.offer.size, open.offer.picks
    );
    for (idx, option) in open.options.iter().enumerate() {
        match option {
            PackOption::Joker(id) => {
                let name = find_joker_name(run, id);
                println!("{:>2}: joker {} ({})", idx, id, name);
            }
            PackOption::Consumable(kind, id) => {
                let name = find_consumable_name(run, *kind, id);
                println!("{:>2}: {:?} {} ({})", idx, kind, id, name);
            }
            PackOption::PlayingCard(card) => {
                println!("{:>2}: card {}", idx, format_card(card));
            }
        }
    }
}

fn drain_events(events: &mut EventBus) {
    for event in events.drain() {
        println!("event: {:?}", event);
    }
}

fn parse_indices(args: &[&str]) -> Option<Vec<usize>> {
    if args.is_empty() {
        return None;
    }
    let mut indices = Vec::new();
    for arg in args {
        match arg.parse::<usize>() {
            Ok(idx) => indices.push(idx),
            Err(_) => return None,
        }
    }
    Some(indices)
}

fn format_card(card: &Card) -> String {
    if card.face_down {
        return "??".to_string();
    }
    let mut out = format!("{}{}", rank_short(card.rank), suit_short(card.suit));
    let mut tags = Vec::new();
    if let Some(enhancement) = card.enhancement {
        tags.push(enhancement_short(enhancement));
    }
    if let Some(edition) = card.edition {
        tags.push(edition_short(edition));
    }
    if let Some(seal) = card.seal {
        tags.push(seal_short(seal));
    }
    if card.bonus_chips != 0 {
        tags.push("Bonus");
    }
    if !tags.is_empty() {
        out.push_str(" [");
        out.push_str(&tags.join(","));
        out.push(']');
    }
    out
}

fn rank_short(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::King => "K",
        Rank::Queen => "Q",
        Rank::Jack => "J",
        Rank::Ten => "T",
        Rank::Nine => "9",
        Rank::Eight => "8",
        Rank::Seven => "7",
        Rank::Six => "6",
        Rank::Five => "5",
        Rank::Four => "4",
        Rank::Three => "3",
        Rank::Two => "2",
        Rank::Joker => "Jk",
    }
}

fn suit_short(suit: Suit) -> &'static str {
    match suit {
        Suit::Spades => "S",
        Suit::Hearts => "H",
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Wild => "W",
    }
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

fn edition_short(kind: Edition) -> &'static str {
    match kind {
        Edition::Foil => "Foil",
        Edition::Holographic => "Holo",
        Edition::Polychrome => "Poly",
        Edition::Negative => "Neg",
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

fn find_joker_name(run: &RunState, id: &str) -> String {
    run.content
        .jokers
        .iter()
        .find(|joker| joker.id == id)
        .map(|joker| joker.name.clone())
        .unwrap_or_else(|| "-".to_string())
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
        .unwrap_or_else(|| "-".to_string())
}
