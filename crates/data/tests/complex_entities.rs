use rulatro_core::{
    Action, ActionOp, ActivationType, BlindKind, BossDef, Card, ConsumableKind, Edition,
    Enhancement, EventBus, Expr, HandKind, JokerDef, JokerEffect, JokerRarity, Phase, Rank,
    RunState, Seal, ShopOfferRef, Suit,
};
use rulatro_data::{load_content, load_game_config};
use std::path::PathBuf;

fn assets_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
}

fn new_run() -> RunState {
    let config = load_game_config(&assets_root()).expect("load config");
    let content = load_content(&assets_root()).expect("load content");
    let mut run = RunState::new(config, content, 7);
    run.inventory.joker_slots = 99;
    run.inventory.consumable_slots = 99;
    run
}

fn make_hand() -> Vec<Card> {
    vec![
        Card::standard(Suit::Spades, Rank::Ace),
        Card::standard(Suit::Hearts, Rank::Two),
        Card::standard(Suit::Clubs, Rank::Three),
        Card::standard(Suit::Diamonds, Rank::Four),
        Card::standard(Suit::Spades, Rank::Five),
        Card::standard(Suit::Hearts, Rank::Six),
        Card::standard(Suit::Clubs, Rank::Seven),
        Card::standard(Suit::Diamonds, Rank::Eight),
    ]
}

fn prepare_play_state(run: &mut RunState) {
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.state.target = 1_000_000;
    run.state.blind_score = 0;
    run.hand = make_hand();
}

fn use_consumable(run: &mut RunState, id: &str, kind: ConsumableKind, selected: &[usize]) {
    run.inventory.consumables.clear();
    run.inventory
        .add_consumable(id.to_string(), kind)
        .expect("add consumable");
    run.use_consumable(0, selected, &mut EventBus::default())
        .expect("use consumable");
}

fn hand_level(run: &RunState, kind: HandKind) -> u32 {
    let key = rulatro_core::level_kind(kind);
    run.state.hand_levels.get(&key).copied().unwrap_or(1)
}

fn mark_blind_cleared(run: &mut RunState) {
    run.state.target = 1;
    run.state.blind_score = 1;
}

fn add_joker_effect(run: &mut RunState, id: &str, trigger: ActivationType, actions: Vec<Action>) {
    run.content.jokers.push(JokerDef {
        id: id.to_string(),
        name: id.to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger,
            when: Expr::Bool(true),
            actions,
        }],
    });
    run.inventory
        .add_joker(id.to_string(), JokerRarity::Common, 1)
        .expect("add joker");
}

#[test]
fn complex_joker_scoring_trace_preserves_action_order() {
    let mut baseline = new_run();
    prepare_play_state(&mut baseline);
    baseline
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("baseline play");
    let baseline_score = baseline.state.blind_score;

    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "j_add_chips",
        ActivationType::Independent,
        vec![Action {
            op: ActionOp::AddChips,
            target: None,
            value: Expr::Number(10.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "j_mul_chips",
        ActivationType::Independent,
        vec![Action {
            op: ActionOp::MultiplyChips,
            target: None,
            value: Expr::Number(2.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "j_add_mult",
        ActivationType::Independent,
        vec![Action {
            op: ActionOp::AddMult,
            target: None,
            value: Expr::Number(1.5),
        }],
    );
    prepare_play_state(&mut run);
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("stacked play");
    assert!(run.state.blind_score > baseline_score);

    let trace = &run.last_score_trace;
    let add_pos = trace
        .iter()
        .position(|step| step.source == "joker:j_add_chips:add_chips")
        .expect("add chips trace");
    let mul_chip_pos = trace
        .iter()
        .position(|step| step.source == "joker:j_mul_chips:mul_chips")
        .expect("mul chips trace");
    let add_mult_pos = trace
        .iter()
        .position(|step| step.source == "joker:j_add_mult:add_mult")
        .expect("add mult trace");
    assert!(add_pos < mul_chip_pos);
    assert!(mul_chip_pos < add_mult_pos);
}

#[test]
fn complex_joker_disable_boss_and_destroy_self_blocks_boss_scoring_bonus() {
    let boss_id = "complex_money_boss";
    let mut baseline = new_run();
    baseline.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Complex Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(17.0),
            }],
        }],
    });
    prepare_play_state(&mut baseline);
    baseline.state.blind = BlindKind::Boss;
    baseline.state.boss_id = Some(boss_id.to_string());
    baseline
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("baseline boss play");
    assert_eq!(baseline.state.money, 17);

    let mut run = new_run();
    run.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Complex Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(17.0),
            }],
        }],
    });
    add_joker_effect(
        &mut run,
        "boss_off_switch",
        ActivationType::OnPlayed,
        vec![
            Action {
                op: ActionOp::DisableBoss,
                target: None,
                value: Expr::Number(1.0),
            },
            Action {
                op: ActionOp::DestroySelf,
                target: None,
                value: Expr::Number(1.0),
            },
        ],
    );
    prepare_play_state(&mut run);
    run.state.blind = BlindKind::Boss;
    run.state.boss_id = Some(boss_id.to_string());
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("boss off play");
    assert_eq!(run.state.money, 0);
    assert!(run.boss_effects_disabled());
    assert!(run.inventory.jokers.is_empty());
}

#[test]
fn complex_tarot_death_copies_full_card_metadata() {
    let mut run = new_run();
    run.hand = make_hand();
    run.hand[1].enhancement = Some(Enhancement::Gold);
    run.hand[1].edition = Some(Edition::Foil);
    run.hand[1].seal = Some(Seal::Blue);
    run.hand[1].bonus_chips = 42;

    use_consumable(&mut run, "death", ConsumableKind::Tarot, &[0, 1]);
    assert_eq!(run.hand[0].suit, run.hand[1].suit);
    assert_eq!(run.hand[0].rank, run.hand[1].rank);
    assert_eq!(run.hand[0].enhancement, run.hand[1].enhancement);
    assert_eq!(run.hand[0].edition, run.hand[1].edition);
    assert_eq!(run.hand[0].seal, run.hand[1].seal);
    assert_eq!(run.hand[0].bonus_chips, run.hand[1].bonus_chips);
}

#[test]
fn complex_tarot_fool_replays_last_planet_and_keeps_last_consumable() {
    let mut run = new_run();
    let planet_hand = run
        .content
        .planets
        .iter()
        .find(|item| item.id == "pluto")
        .and_then(|item| item.hand)
        .expect("pluto hand");

    use_consumable(&mut run, "pluto", ConsumableKind::Planet, &[]);
    assert_eq!(hand_level(&run, planet_hand), 2);

    use_consumable(&mut run, "the_fool", ConsumableKind::Tarot, &[]);
    assert_eq!(run.inventory.consumables.len(), 1);
    assert_eq!(run.inventory.consumables[0].id, "pluto");
    assert_eq!(run.inventory.consumables[0].kind, ConsumableKind::Planet);
    let last = run.state.last_consumable.clone().expect("last consumable");
    assert_eq!(last.id, "pluto");
    assert_eq!(last.kind, ConsumableKind::Planet);

    run.use_consumable(0, &[], &mut EventBus::default())
        .expect("reuse pluto");
    assert_eq!(hand_level(&run, planet_hand), 3);
}

#[test]
fn complex_spectral_black_hole_then_planet_stacks_hand_levels() {
    let mut run = new_run();
    let planet_hand = run
        .content
        .planets
        .iter()
        .find(|item| item.id == "pluto")
        .and_then(|item| item.hand)
        .expect("pluto hand");
    use_consumable(&mut run, "black_hole", ConsumableKind::Spectral, &[]);
    assert_eq!(hand_level(&run, HandKind::Pair), 2);
    assert_eq!(hand_level(&run, planet_hand), 2);

    use_consumable(&mut run, "pluto", ConsumableKind::Planet, &[]);
    assert_eq!(hand_level(&run, planet_hand), 3);
    assert_eq!(hand_level(&run, HandKind::Pair), 2);
}

#[test]
fn complex_spectral_ectoplasm_grants_extra_joker_capacity() {
    let mut run = new_run();
    run.hand = make_hand();
    run.inventory.joker_slots = 1;
    run.inventory.jokers.clear();
    run.inventory
        .add_joker("base_joker".to_string(), JokerRarity::Common, 5)
        .expect("add base joker");
    assert_eq!(run.inventory.joker_capacity(), 1);

    use_consumable(&mut run, "ectoplasm", ConsumableKind::Spectral, &[]);
    assert_eq!(run.inventory.jokers.len(), 1);
    assert_eq!(run.inventory.joker_capacity(), 2);
    assert!(run
        .inventory
        .jokers
        .iter()
        .any(|joker| joker.edition == Some(Edition::Negative)));
    run.inventory
        .add_joker("extra_joker".to_string(), JokerRarity::Common, 5)
        .expect("extra joker now fits");
    assert_eq!(run.inventory.jokers.len(), 2);
}

#[test]
fn complex_tags_apply_before_joker_shop_overrides() {
    let mut run = new_run();
    run.state.tags.push("coupon_tag".to_string());
    run.state.tags.push("d6_tag".to_string());
    add_joker_effect(
        &mut run,
        "shop_override",
        ActivationType::OnShopEnter,
        vec![
            Action {
                op: ActionOp::SetShopPrice,
                target: Some("cards".to_string()),
                value: Expr::Number(4.0),
            },
            Action {
                op: ActionOp::SetRerollCost,
                target: None,
                value: Expr::Number(6.0),
            },
        ],
    );
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.iter().all(|card| card.price == 4));
    assert!(shop.packs.iter().all(|pack| pack.price == 0));
    assert_eq!(shop.reroll_cost, 6);
    assert!(run.state.tags.is_empty());
}

#[test]
fn complex_duplicate_next_tag_stacks_with_coupon_effect() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "dup_tag",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOp::DuplicateNextTag,
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "add_coupon",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOp::AddTag,
            target: Some("coupon_tag".to_string()),
            value: Expr::Number(1.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.tags.len(), 2);
    assert!(run.state.tags.iter().all(|item| item == "coupon_tag"));

    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.iter().all(|card| card.price == 0));
    assert!(shop.packs.iter().all(|pack| pack.price == 0));
    assert!(run.state.tags.is_empty());
}

#[test]
fn complex_boss_shop_bonus_blocked_after_disable_during_play() {
    let boss_id = "shop_money_boss";
    let mut baseline = new_run();
    baseline.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Shop Bonus Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(9.0),
            }],
        }],
    });
    baseline.state.blind = BlindKind::Boss;
    baseline.state.boss_id = Some(boss_id.to_string());
    mark_blind_cleared(&mut baseline);
    baseline
        .enter_shop(&mut EventBus::default())
        .expect("baseline enter shop");
    assert_eq!(baseline.state.money, 9);

    let mut run = new_run();
    run.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Shop Bonus Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(9.0),
            }],
        }],
    });
    add_joker_effect(
        &mut run,
        "disable_in_play",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOp::DisableBoss,
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    prepare_play_state(&mut run);
    run.state.blind = BlindKind::Boss;
    run.state.boss_id = Some(boss_id.to_string());
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("disable boss in play");
    assert!(run.boss_effects_disabled());
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop with disabled boss");
    assert_eq!(run.state.money, 0);
}

#[test]
fn complex_boss_effect_requires_boss_blind_even_if_boss_id_is_set() {
    let boss_id = "blind_specific_boss";
    let mut run = new_run();
    run.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Blind Specific Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(11.0),
            }],
        }],
    });
    prepare_play_state(&mut run);
    run.state.blind = BlindKind::Big;
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play non-boss blind without boss id");
    assert_eq!(run.state.money, 0);

    let mut run_with_id = new_run();
    run_with_id.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Blind Specific Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(11.0),
            }],
        }],
    });
    prepare_play_state(&mut run_with_id);
    run_with_id.state.blind = BlindKind::Big;
    run_with_id.state.boss_id = Some(boss_id.to_string());
    run_with_id
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play non-boss blind with boss id");
    assert_eq!(run_with_id.state.money, 0);

    let mut run_boss = new_run();
    run_boss.content.bosses.push(BossDef {
        id: boss_id.to_string(),
        name: "Blind Specific Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
                target: None,
                value: Expr::Number(11.0),
            }],
        }],
    });
    prepare_play_state(&mut run_boss);
    run_boss.state.blind = BlindKind::Boss;
    run_boss.state.boss_id = Some(boss_id.to_string());
    run_boss
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play boss blind");
    assert_eq!(run_boss.state.money, 11);
}

#[test]
fn complex_buy_flow_after_tag_and_joker_price_override() {
    let mut run = new_run();
    run.state.money = 20;
    run.state.tags.push("coupon_tag".to_string());
    add_joker_effect(
        &mut run,
        "card_price_three",
        ActivationType::OnShopEnter,
        vec![Action {
            op: ActionOp::SetShopPrice,
            target: Some("cards".to_string()),
            value: Expr::Number(3.0),
        }],
    );
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    let card_count = shop.cards.len();
    assert!(card_count > 0);
    assert!(shop.cards.iter().all(|offer| offer.price == 3));
    let before_money = run.state.money;
    let purchase = run
        .buy_shop_offer(ShopOfferRef::Card(0), &mut EventBus::default())
        .expect("buy first card");
    run.apply_purchase(&purchase).expect("apply purchase");
    assert_eq!(run.state.money, before_money - 3);
}

#[test]
fn complex_joker_copy_right_replays_neighbor_action_once() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "copy_right",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOp::CopyJokerRight,
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "money_neighbor",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOp::AddMoney,
            target: None,
            value: Expr::Number(5.0),
        }],
    );
    prepare_play_state(&mut run);
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play with copy-right joker");
    assert_eq!(run.state.money, 10);
    assert_eq!(run.inventory.jokers.len(), 2);
}

#[test]
fn complex_joker_destroy_right_skips_neighbor_effect_same_trigger() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "destroy_right",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOp::DestroyJokerRight,
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "money_victim",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOp::AddMoney,
            target: None,
            value: Expr::Number(25.0),
        }],
    );
    prepare_play_state(&mut run);
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play with destroy-right joker");
    assert_eq!(run.state.money, 0);
    assert_eq!(run.inventory.jokers.len(), 1);
    assert_eq!(run.inventory.jokers[0].id, "destroy_right");
}

#[test]
fn complex_tarot_strength_wraps_rank_cycle() {
    let mut run = new_run();
    run.hand = make_hand();
    run.hand[0].rank = Rank::King;
    run.hand[1].rank = Rank::Ace;
    use_consumable(&mut run, "strength", ConsumableKind::Tarot, &[0, 1]);
    assert_eq!(run.hand[0].rank, Rank::Ace);
    assert_eq!(run.hand[1].rank, Rank::Two);
}

#[test]
fn complex_boss_disable_in_shop_applies_to_next_boss_start() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "shop_disable_boss",
        ActivationType::OnShopEnter,
        vec![Action {
            op: ActionOp::DisableBoss,
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop with disable boss");
    run.start_blind(1, BlindKind::Boss, &mut EventBus::default())
        .expect("start next boss blind");
    assert!(run.boss_effects_disabled());
    assert!(run.state.boss_id.is_none());
}
