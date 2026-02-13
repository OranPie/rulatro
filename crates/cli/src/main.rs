use rulatro_core::{
    BlindKind, BlindOutcome, Card, ConsumableKind, Edition, Enhancement, Event, EventBus, PackOpen,
    PackOption, Phase, Rank, RunError, RunState, ScoreBreakdown, ScoreTables, ScoreTraceStep, Seal,
    ShopOfferRef, Suit,
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
    let modded =
        load_content_with_mods(Path::new("assets"), Path::new("mods")).expect("load content");
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
    runtime.load_mods(&modded.mods).expect("load mod runtime");
    let mut run = RunState::new(config, modded.content, 0xC0FFEE);
    run.set_mod_runtime(Some(Box::new(runtime)));
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut blinds_completed = 0;
    loop {
        run.prepare_hand(&mut events).expect("prepare hand");

        let play_count = run.hand.len().min(5);
        let indices: Vec<usize> = (0..play_count).collect();
        let breakdown = run.play_hand(&indices, &mut events).expect("play hand");

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
    let modded =
        load_content_with_mods(Path::new("assets"), Path::new("mods")).expect("load content");
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
    runtime.load_mods(&modded.mods).expect("load mod runtime");
    let mut run = RunState::new(config, modded.content, 0xC0FFEE);
    run.set_mod_runtime(Some(Box::new(runtime)));
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut open_pack: Option<PackOpen> = None;
    print_help();
    loop {
        let mut show_flow = false;
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
            "status" => print_summary(&run),
            "hand" => print_hand(&run),
            "deck" => print_deck(&run),
            "levels" => print_levels(&run),
            "tags" => print_tags(&run),
            "inv" | "inventory" => print_inventory(&run),
            "reward" => print_reward(&run),
            "summary" => print_summary(&run),
            "board" | "overview" | "ls" => print_overview(&run, open_pack.as_ref()),
            "data" | "ref" => print_reference(),
            "deal" | "d" => {
                show_flow = true;
                match run.prepare_hand(&mut events) {
                    Ok(_) => println!("dealt hand"),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            "play" | "p" => {
                show_flow = true;
                let indices = parse_indices(&args);
                match indices {
                    Some(indices) => {
                        println!("selected indices: {:?}", indices);
                        let preview = collect_played_cards(&run.hand, &indices).ok();
                        match run.play_hand(&indices, &mut events) {
                            Ok(breakdown) => {
                                print_score_breakdown(
                                    &breakdown,
                                    preview.as_deref(),
                                    &run.tables,
                                    &run.last_score_trace,
                                );
                            }
                            Err(err) => println!("error: {err:?}"),
                        }
                    }
                    None => println!("usage: play <idx> <idx> ..."),
                }
            }
            "discard" | "x" => {
                show_flow = true;
                let indices = parse_indices(&args);
                match indices {
                    Some(indices) => match run.discard(&indices, &mut events) {
                        Ok(_) => println!("discarded"),
                        Err(err) => println!("error: {err:?}"),
                    },
                    None => println!("usage: discard <idx> <idx> ..."),
                }
            }
            "skip" | "skip_blind" => {
                show_flow = true;
                match run.skip_blind(&mut events) {
                    Ok(_) => println!("blind skipped"),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            "shop" | "sh" => {
                show_flow = true;
                match run.enter_shop(&mut events) {
                    Ok(_) => print_shop(&run),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            "leave" => {
                show_flow = true;
                run.leave_shop();
                open_pack = None;
                println!("left shop");
            }
            "reroll" | "r" => {
                show_flow = true;
                match run.reroll_shop(&mut events) {
                    Ok(_) => print_shop(&run),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            "buy" => {
                show_flow = true;
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
                                Ok(purchase) => {
                                    match run.open_pack_purchase(&purchase, &mut events) {
                                        Ok(open) => {
                                            print_pack_open(&open, &run);
                                            open_pack = Some(open);
                                        }
                                        Err(err) => println!("error: {err:?}"),
                                    }
                                }
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
            "pack" => {
                if let Some(open) = open_pack.as_ref() {
                    print_pack_open(open, &run);
                } else {
                    println!("no open pack");
                }
            }
            "edit" => {
                if args.is_empty() {
                    println!(
                        "usage: edit <idx..> enh=<kind|none> ed=<kind|none> seal=<kind|none> bonus=<n|+n|-n> face_down=<0|1>"
                    );
                    continue;
                }
                match parse_edit_args(&args) {
                    Ok((indices, edits)) => {
                        match apply_card_edits(&mut run.hand, &indices, edits) {
                            Ok(_) => println!("edited cards: {:?}", indices),
                            Err(err) => println!("error: {err}"),
                        }
                    }
                    Err(err) => println!("error: {err}"),
                }
            }
            "pick" => {
                show_flow = true;
                if let Some(open) = open_pack.clone() {
                    let indices = parse_indices(&args);
                    match indices {
                        Some(indices) => {
                            match run.choose_pack_options(&open, &indices, &mut events) {
                                Ok(_) => {
                                    println!("picked pack options");
                                    open_pack = None;
                                }
                                Err(err) => println!("error: {err:?}"),
                            }
                        }
                        None => println!("usage: pick <idx> <idx> ..."),
                    }
                } else {
                    println!("no open pack");
                }
            }
            "skip_pack" | "sp" => {
                show_flow = true;
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
            "peek" => {
                if args.is_empty() {
                    println!("usage: peek draw|discard [count]");
                } else {
                    let target = args[0];
                    let count = args
                        .get(1)
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(5);
                    match target {
                        "draw" => print_peek(&run.deck.draw, count, "draw"),
                        "discard" => print_peek(&run.deck.discard, count, "discard"),
                        _ => println!("usage: peek draw|discard [count]"),
                    }
                }
            }
            "use" => {
                show_flow = true;
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
                show_flow = true;
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
            "next" | "n" => {
                show_flow = true;
                open_pack = None;
                match run.start_next_blind(&mut events) {
                    Ok(_) => println!("started next blind"),
                    Err(err) => println!("error: {err:?}"),
                }
            }
            _ => println!("unknown command: {cmd} (type 'help')"),
        }
        drain_events(&mut events);
        if show_flow {
            print_flow_summary(&run, open_pack.as_ref());
        }
    }
}

fn print_help() {
    println!("Commands:");
    println!("  help|h|?                 show help");
    println!("  quit|exit                exit");
    println!();
    println!("View:");
    println!("  summary|status           one-line run status");
    println!("  state|s                  detailed run state");
    println!("  board|overview|ls        full current view (state+hand+inv+shop+pack)");
    println!("  hand                      show hand table");
    println!("  deck                      show draw/discard sizes");
    println!("  levels                    show hand levels");
    println!("  tags                      show tags");
    println!("  inv|inventory             show jokers and consumables");
    println!("  reward                    estimate clear reward");
    println!("  data|ref                  print enhancement/joker/consumable reference");
    println!();
    println!("Run:");
    println!("  deal|d                    draw hand (Deal phase)");
    println!("  play|p <idx..>            play cards");
    println!("  discard|x <idx..>         discard cards");
    println!("  skip|skip_blind           skip current blind (Small/Big only)");
    println!("  next|n                    start next blind");
    println!();
    println!("Shop / Pack:");
    println!("  shop|sh                   enter shop");
    println!("  reroll|r                  reroll shop");
    println!("  buy card|pack|voucher <idx>");
    println!("  leave                     leave shop");
    println!("  pack                      show open pack options");
    println!("  pick <idx..>              pick pack options");
    println!("  skip_pack|sp              skip open pack");
    println!();
    println!("Debug / Edit:");
    println!("  use <consumable_idx> [sel..]");
    println!("  sell <joker_idx>");
    println!("  edit <idx..> enh=.. ed=.. seal=.. bonus=.. face_down=..");
    println!("  peek draw|discard [n]");
    println!("note: indices support comma and ranges (e.g. 0,2-4 7)");
    println!("tip: actions print a flow summary automatically");
    println!("tip: run with --auto for scripted demo");
}

fn print_reference() {
    println!("Enhancements:");
    println!("  Bonus +30 chips (scored)");
    println!("  Mult +4 mult (scored)");
    println!("  Glass x2 mult (scored), 1/4 break");
    println!("  Stone +50 chips (scored), no rank/suit");
    println!("  Lucky 1/5 +20 mult, 1/15 +$20");
    println!("  Steel x1.5 mult (held)");
    println!("  Gold +$3 end of round (held)");
    println!("  Wild counts as any suit");
    println!(
        "Seals: Red retrigger; Gold +$3 scored; Blue planet on round end; Purple tarot on discard"
    );
    println!(
        "Editions: Foil +50 chips; Holo +10 mult; Polychrome x1.5 mult; Negative +1 joker slot"
    );
    println!();
    println!("Joker DSL triggers (on ...): played, scored_pre, scored, held, independent,");
    println!("  discard, discard_batch, card_destroyed, card_added, round_end, hand_end,");
    println!("  blind_start, blind_failed, shop_enter, shop_reroll, shop_exit,");
    println!("  pack_opened, pack_skipped, use, sell, any_sell, acquire, passive");
    println!("Common DSL condition identifiers:");
    println!("  hand, blind, ante, blind_score, target, money, hands_left, discards_left,");
    println!("  played_count, scoring_count, held_count, deck_count,");
    println!("  card.rank, card.suit, card.enhancement, card.edition, card.seal,");
    println!("  card.is_face/odd/even/stone/wild, consumable.kind/id");
    println!("Common DSL functions:");
    println!("  contains(hand, HandKind), count(scope,target), count_joker(name/id),");
    println!("  count_rarity(rarity), suit_match(suit|id), hand_count(hand), var(key),");
    println!("  roll(n), rand(min,max), min/max/floor/ceil/pow");
    println!();
    println!("Consumable effects:");
    println!("  EnhanceSelected/AddEditionToSelected/AddSealToSelected");
    println!("  ConvertSelectedSuit/IncreaseSelectedRank/DestroySelected/CopySelected");
    println!("  AddRandomConsumable/AddJoker/AddRandomJoker/UpgradeHand/UpgradeAllHands");
    println!("  AddMoney/SetMoney/DoubleMoney/AddMoneyFromJokers");
    println!("Selection rules: selection required for *Selected/*LeftIntoRight ops;");
    println!("  indices refer to current hand.");
}

fn print_prompt(run: &RunState, open_pack: Option<&PackOpen>) {
    let pack = if open_pack.is_some() { " PK" } else { "" };
    print!(
        "[A{} {} {} ${} {}/{} H{}/{} D{}/{} SK{}{}] > ",
        run.state.ante,
        blind_short(run.state.blind),
        phase_short(run.state.phase),
        run.state.money,
        run.state.blind_score,
        run.state.target,
        run.state.hands_left,
        run.state.hands_max,
        run.state.discards_left,
        run.state.discards_max,
        run.state.blinds_skipped,
        pack
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
    println!("blinds skipped {}", run.state.blinds_skipped);
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

fn print_levels(run: &RunState) {
    println!("hand levels:");
    for kind in rulatro_core::HandKind::ALL {
        let level = run.state.hand_levels.get(&kind).copied().unwrap_or(1);
        println!("  {:?}: {}", kind, level);
    }
}

fn print_tags(run: &RunState) {
    if run.state.tags.is_empty() {
        println!("tags: none");
    } else {
        println!("tags: {}", run.state.tags.join(", "));
    }
    if run.state.duplicate_next_tag {
        if let Some(exclude) = &run.state.duplicate_tag_exclude {
            println!("duplicate next tag (excluding {exclude})");
        } else {
            println!("duplicate next tag");
        }
    }
}

fn print_reward(run: &RunState) {
    if run.state.target <= 0 {
        println!("reward: blind not started");
        return;
    }
    let economy = &run.config.economy;
    let base = match run.state.blind {
        BlindKind::Small => economy.reward_small,
        BlindKind::Big => economy.reward_big,
        BlindKind::Boss => economy.reward_boss,
    };
    let interest = estimate_interest(run);
    let reward = base + economy.per_hand_reward * run.state.hands_left as i64 + interest;
    println!("reward estimate: {}", reward);
}

fn print_summary(run: &RunState) {
    println!(
        "ante {} {:?} {:?} money {} score {}/{} hands {}/{} discards {}/{} skipped {}",
        run.state.ante,
        run.state.blind,
        run.state.phase,
        run.state.money,
        run.state.blind_score,
        run.state.target,
        run.state.hands_left,
        run.state.hands_max,
        run.state.discards_left,
        run.state.discards_max,
        run.state.blinds_skipped
    );
}

fn print_flow_summary(run: &RunState, open_pack: Option<&PackOpen>) {
    let pack = if open_pack.is_some() {
        " | pack open"
    } else {
        ""
    };
    println!(
        "=> A{} {} {} | ${} | {}/{} | hands {}/{} discards {}/{} | skipped {}{}",
        run.state.ante,
        blind_short(run.state.blind),
        phase_short(run.state.phase),
        run.state.money,
        run.state.blind_score,
        run.state.target,
        run.state.hands_left,
        run.state.hands_max,
        run.state.discards_left,
        run.state.discards_max,
        run.state.blinds_skipped,
        pack
    );
}

fn print_overview(run: &RunState, open_pack: Option<&PackOpen>) {
    print_summary(run);
    print_tags(run);
    print_hand(run);
    print_inventory(run);
    if run.shop.is_some() {
        print_shop(run);
    }
    if let Some(open) = open_pack {
        print_pack_open(open, run);
    }
}

fn print_hand(run: &RunState) {
    println!("hand ({} cards):", run.hand.len());
    println!(" idx  card          value  detail");
    for (idx, card) in run.hand.iter().enumerate() {
        let value = card_value(card, &run.tables);
        println!(
            "{:>4}  {:<12}  {:>5}  {}",
            idx,
            format_card(card),
            value,
            card_detail(card)
        );
    }
}

fn print_deck(run: &RunState) {
    println!("draw pile: {}", run.deck.draw.len());
    println!("discard pile: {}", run.deck.discard.len());
}

fn print_inventory(run: &RunState) {
    println!(
        "jokers ({}/{}):",
        run.inventory.jokers.len(),
        run.inventory.joker_capacity()
    );
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
    println!(
        "shop: cards {} packs {} vouchers {} reroll {}",
        shop.cards.len(),
        shop.packs.len(),
        shop.vouchers,
        shop.reroll_cost
    );
    println!("cards:");
    for (idx, card) in shop.cards.iter().enumerate() {
        let rarity = card
            .rarity
            .map(|value| format!("{value:?}"))
            .unwrap_or_else(|| "-".to_string());
        let edition = card.edition.map(edition_short).unwrap_or("-");
        println!(
            "  {:>2}: {:<10?} {:<22} price {:>3} rarity {:<8} edition {}",
            idx, card.kind, card.item_id, card.price, rarity, edition
        );
    }
    println!("packs:");
    for (idx, pack) in shop.packs.iter().enumerate() {
        println!(
            "  {:>2}: {:<9?} {:<6?} options {:>2} picks {:>2} price {:>3}",
            idx, pack.kind, pack.size, pack.options, pack.picks, pack.price
        );
    }
    println!("vouchers: {}", shop.vouchers);
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

fn print_peek(cards: &[Card], count: usize, label: &str) {
    if cards.is_empty() {
        println!("{label}: empty");
        return;
    }
    let total = cards.len();
    let start = total.saturating_sub(count);
    println!("{label} top {}/{}:", total - start, total);
    for (offset, card) in cards[start..].iter().rev().enumerate() {
        let index = total - 1 - offset;
        println!("{:>2}: {}", index, format_card(card));
    }
}

fn drain_events(events: &mut EventBus) {
    for event in events.drain() {
        println!("event: {}", format_event(&event));
    }
}

fn parse_indices(args: &[&str]) -> Option<Vec<usize>> {
    parse_indices_result(args).ok()
}

fn parse_indices_result(args: &[&str]) -> Result<Vec<usize>, String> {
    if args.is_empty() {
        return Err("missing indices".to_string());
    }
    let mut indices = Vec::new();
    for arg in args {
        for part in arg.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((start, end)) = part.split_once('-') {
                let start = start
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "invalid range start".to_string())?;
                let end = end
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "invalid range end".to_string())?;
                if start > end {
                    return Err("range start larger than end".to_string());
                }
                for idx in start..=end {
                    indices.push(idx);
                }
            } else {
                let idx = part
                    .parse::<usize>()
                    .map_err(|_| format!("invalid index '{part}'"))?;
                indices.push(idx);
            }
        }
    }
    if indices.is_empty() {
        return Err("missing indices".to_string());
    }
    Ok(indices)
}

fn collect_played_cards(hand: &[Card], indices: &[usize]) -> Result<Vec<Card>, RunError> {
    if indices.is_empty() {
        return Err(RunError::InvalidSelection);
    }
    let mut unique = indices.to_vec();
    unique.sort_unstable();
    unique.dedup();
    if unique.iter().any(|&idx| idx >= hand.len()) {
        return Err(RunError::InvalidSelection);
    }
    unique.sort_unstable_by(|a, b| b.cmp(a));
    let mut picked = Vec::with_capacity(unique.len());
    for idx in unique {
        picked.push(hand[idx]);
    }
    Ok(picked)
}

#[derive(Debug, Clone, Copy)]
enum BonusEdit {
    Set(i64),
    Add(i64),
}

#[derive(Debug, Clone)]
struct CardEdits {
    enhancement: Option<Option<Enhancement>>,
    edition: Option<Option<Edition>>,
    seal: Option<Option<Seal>>,
    bonus: Option<BonusEdit>,
    face_down: Option<bool>,
}

fn parse_edit_args(args: &[&str]) -> Result<(Vec<usize>, CardEdits), String> {
    let mut index_tokens = Vec::new();
    let mut edits = CardEdits {
        enhancement: None,
        edition: None,
        seal: None,
        bonus: None,
        face_down: None,
    };

    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim();
            match key.as_str() {
                "enh" | "enhancement" => {
                    edits.enhancement = Some(parse_optional_enhancement(value)?);
                }
                "ed" | "edition" => {
                    edits.edition = Some(parse_optional_edition(value)?);
                }
                "seal" => {
                    edits.seal = Some(parse_optional_seal(value)?);
                }
                "bonus" => {
                    edits.bonus = Some(parse_bonus_edit(value)?);
                }
                "face" | "face_down" => {
                    edits.face_down = Some(parse_bool(value)?);
                }
                _ => return Err(format!("unknown edit key '{key}'")),
            }
        } else {
            index_tokens.push(*arg);
        }
    }

    let indices = parse_indices_result(&index_tokens)?;
    Ok((indices, edits))
}

fn apply_card_edits(hand: &mut [Card], indices: &[usize], edits: CardEdits) -> Result<(), String> {
    if indices.is_empty() {
        return Err("missing indices".to_string());
    }
    for &idx in indices {
        if idx >= hand.len() {
            return Err(format!("index {idx} out of range"));
        }
    }
    for &idx in indices {
        let card = &mut hand[idx];
        if let Some(enh) = edits.enhancement {
            card.enhancement = enh;
        }
        if let Some(edition) = edits.edition {
            card.edition = edition;
        }
        if let Some(seal) = edits.seal {
            card.seal = seal;
        }
        if let Some(bonus) = edits.bonus {
            match bonus {
                BonusEdit::Set(value) => card.bonus_chips = value,
                BonusEdit::Add(delta) => card.bonus_chips = card.bonus_chips.saturating_add(delta),
            }
        }
        if let Some(face_down) = edits.face_down {
            card.face_down = face_down;
        }
    }
    Ok(())
}

fn parse_optional_enhancement(value: &str) -> Result<Option<Enhancement>, String> {
    if is_none(value) {
        return Ok(None);
    }
    parse_enhancement(value).map(Some)
}

fn parse_enhancement(value: &str) -> Result<Enhancement, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "bonus" => Ok(Enhancement::Bonus),
        "mult" => Ok(Enhancement::Mult),
        "wild" => Ok(Enhancement::Wild),
        "glass" => Ok(Enhancement::Glass),
        "steel" => Ok(Enhancement::Steel),
        "stone" => Ok(Enhancement::Stone),
        "lucky" => Ok(Enhancement::Lucky),
        "gold" => Ok(Enhancement::Gold),
        _ => Err(format!("invalid enhancement '{value}'")),
    }
}

fn parse_optional_edition(value: &str) -> Result<Option<Edition>, String> {
    if is_none(value) {
        return Ok(None);
    }
    parse_edition(value).map(Some)
}

fn parse_edition(value: &str) -> Result<Edition, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "foil" => Ok(Edition::Foil),
        "holo" | "holographic" => Ok(Edition::Holographic),
        "poly" | "polychrome" => Ok(Edition::Polychrome),
        "neg" | "negative" => Ok(Edition::Negative),
        _ => Err(format!("invalid edition '{value}'")),
    }
}

fn parse_optional_seal(value: &str) -> Result<Option<Seal>, String> {
    if is_none(value) {
        return Ok(None);
    }
    parse_seal(value).map(Some)
}

fn parse_seal(value: &str) -> Result<Seal, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "red" => Ok(Seal::Red),
        "blue" => Ok(Seal::Blue),
        "gold" => Ok(Seal::Gold),
        "purple" => Ok(Seal::Purple),
        _ => Err(format!("invalid seal '{value}'")),
    }
}

fn parse_bonus_edit(value: &str) -> Result<BonusEdit, String> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix('+') {
        let amount = rest
            .parse::<i64>()
            .map_err(|_| "invalid bonus delta".to_string())?;
        return Ok(BonusEdit::Add(amount));
    }
    if let Some(rest) = value.strip_prefix('-') {
        let amount = rest
            .parse::<i64>()
            .map_err(|_| "invalid bonus delta".to_string())?;
        return Ok(BonusEdit::Add(-amount));
    }
    let amount = value
        .parse::<i64>()
        .map_err(|_| "invalid bonus value".to_string())?;
    Ok(BonusEdit::Set(amount))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean '{value}'")),
    }
}

fn is_none(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "none" | "null" | "clear"
    )
}

fn print_score_breakdown(
    breakdown: &ScoreBreakdown,
    played: Option<&[Card]>,
    tables: &ScoreTables,
    trace: &[ScoreTraceStep],
) {
    println!("hand: {:?}", breakdown.hand);
    if let Some(cards) = played {
        println!("played cards (order used):");
        for (idx, card) in cards.iter().enumerate() {
            println!("  {:>2}: {}", idx, format_card(card));
        }
    }
    println!("scoring indices: {:?}", breakdown.scoring_indices);
    println!(
        "base: chips={} mult={:.2}",
        breakdown.base.chips, breakdown.base.mult
    );
    if let Some(cards) = played {
        let mut rank_total = 0i64;
        println!("rank chips breakdown:");
        for &idx in &breakdown.scoring_indices {
            if let Some(card) = cards.get(idx) {
                let chips = if card.is_stone() {
                    0
                } else {
                    tables.rank_chips(card.rank)
                };
                rank_total += chips;
                println!("  {:>2}: {} => {}", idx, format_card(card), chips);
            }
        }
        println!("rank chips total: {}", rank_total);
    } else {
        println!("rank chips total: {}", breakdown.rank_chips);
    }
    println!(
        "chips: base {} + rank {} = {} (before effects)",
        breakdown.base.chips,
        breakdown.rank_chips,
        breakdown.base.chips + breakdown.rank_chips
    );
    println!(
        "final: chips={} mult={:.2} score={}",
        breakdown.total.chips,
        breakdown.total.mult,
        breakdown.total.total()
    );

    if trace.is_empty() {
        println!("effect steps: none");
    } else {
        println!("effect steps:");
        for (idx, step) in trace.iter().enumerate() {
            println!(
                "  {:>2}. {} | {:?} | {}×{:.2} -> {}×{:.2}",
                idx + 1,
                step.source,
                step.effect,
                step.before.chips,
                step.before.mult,
                step.after.chips,
                step.after.mult
            );
        }
    }
}

fn estimate_interest(run: &RunState) -> i64 {
    let economy = &run.config.economy;
    if economy.interest_step <= 0 || economy.interest_per <= 0 {
        return 0;
    }
    let steps = (run.state.money / economy.interest_step).max(0);
    let cap_steps = if economy.interest_per > 0 {
        economy.interest_cap / economy.interest_per
    } else {
        0
    };
    let capped = steps.min(cap_steps);
    capped * economy.interest_per
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

fn card_value(card: &Card, tables: &ScoreTables) -> i64 {
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

fn phase_short(phase: Phase) -> &'static str {
    match phase {
        Phase::Setup => "Setup",
        Phase::Deal => "Deal",
        Phase::Play => "Play",
        Phase::Score => "Score",
        Phase::Cleanup => "Clean",
        Phase::Shop => "Shop",
    }
}

fn blind_short(blind: BlindKind) -> &'static str {
    match blind {
        BlindKind::Small => "Small",
        BlindKind::Big => "Big",
        BlindKind::Boss => "Boss",
    }
}

fn format_event(event: &Event) -> String {
    match event {
        Event::BlindStarted {
            ante,
            blind,
            target,
            hands,
            discards,
        } => format!(
            "blind started: ante {ante} {blind:?} target {target} hands {hands} discards {discards}"
        ),
        Event::BlindSkipped { ante, blind, tag } => format!(
            "blind skipped: ante {ante} {blind:?} tag {}",
            tag.as_deref().unwrap_or("none")
        ),
        Event::HandDealt { count } => format!("hand dealt: {count} cards"),
        Event::HandScored {
            hand,
            chips,
            mult,
            total,
        } => format!("hand scored: {hand:?} {chips}x{mult:.2} = {total}"),
        Event::ShopEntered {
            offers,
            reroll_cost,
            reentered,
        } => format!(
            "shop entered: offers {offers} reroll {reroll_cost}{}",
            if *reentered { " (reenter)" } else { "" }
        ),
        Event::ShopRerolled {
            offers,
            reroll_cost,
            cost,
            money,
        } => {
            format!("shop rerolled: offers {offers} reroll {reroll_cost} cost {cost} money {money}")
        }
        Event::ShopBought { offer, cost, money } => {
            format!("shop bought: {offer:?} cost {cost} money {money}")
        }
        Event::PackOpened {
            kind,
            options,
            picks,
        } => format!("pack opened: {kind:?} options {options} picks {picks}"),
        Event::PackChosen { picks } => format!("pack chosen: {picks}"),
        Event::JokerSold {
            id,
            sell_value,
            money,
        } => format!("joker sold: {id} value {sell_value} money {money}"),
        Event::BlindCleared {
            score,
            reward,
            money,
        } => {
            format!("blind cleared: score {score} reward {reward} money {money}")
        }
        Event::BlindFailed { score } => format!("blind failed: score {score}"),
    }
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
