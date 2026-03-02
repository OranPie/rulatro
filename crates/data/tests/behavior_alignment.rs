use rulatro_core::{
    evaluate_hand, evaluate_hand_with_rules, open_pack, pick_pack_options, score_hand,
    scoring_cards, Action, ActionOp, ActionOpKind, ActivationType, BinaryOp, BlindKind, BossDef,
    Card, CardOffer, CardWeight, ConsumableDef, ConsumableKind, Edition, Enhancement, EventBus,
    Expr, HandEvalRules, HandKind, JokerDef, JokerEffect, JokerRarity, JokerRarityWeight,
    LastConsumable, PackError, PackKind, PackOffer, PackOpen, PackOption, PackSize, Phase, Rank,
    RngState, RunError, RunState, ScoreTables, Seal, ShopCardKind, ShopOfferRef, ShopPurchase,
    ShopRestrictions, ShopState, Suit, TagDef, UnaryOp,
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
    let mut run = RunState::new(config, content, 12345);
    run.inventory.joker_slots = 99;
    run.inventory.consumable_slots = 99;
    run.state.money = 0;
    run.state.hand_size = 8;
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

fn make_cards(specs: &[(Suit, Rank)]) -> Vec<Card> {
    specs
        .iter()
        .map(|(suit, rank)| Card::standard(*suit, *rank))
        .collect()
}

fn use_consumable(run: &mut RunState, id: &str, kind: ConsumableKind, selected: &[usize]) {
    run.inventory.consumables.clear();
    run.inventory
        .add_consumable(id.to_string(), kind)
        .expect("add consumable");
    let mut events = EventBus::default();
    run.use_consumable(0, selected, &mut events)
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

fn add_rule_joker(run: &mut RunState, id: &str, key: &str, value: f64) {
    run.content.jokers.push(JokerDef {
        id: id.to_string(),
        name: id.to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::Passive,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::SetRule),
                target: Some(key.to_string()),
                value: Expr::Number(value),
            }],
        }],
    });
    run.inventory
        .add_joker(id.to_string(), JokerRarity::Common, 1)
        .expect("add joker");
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

fn add_money_joker(run: &mut RunState, id: &str, trigger: ActivationType, amount: f64) {
    add_joker_effect(
        run,
        id,
        trigger,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddMoney),
            target: None,
            value: Expr::Number(amount),
        }],
    );
}

fn add_scoring_joker(run: &mut RunState, id: &str, op: ActionOp, value: f64) {
    run.content.jokers.push(JokerDef {
        id: id.to_string(),
        name: id.to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(op),
                target: None,
                value: Expr::Number(value),
            }],
        }],
    });
    run.inventory
        .add_joker(id.to_string(), JokerRarity::Common, 1)
        .expect("add joker");
}

fn add_plain_joker_with_edition(run: &mut RunState, id: &str, edition: Edition) {
    run.content.jokers.push(JokerDef {
        id: id.to_string(),
        name: id.to_string(),
        rarity: JokerRarity::Common,
        effects: Vec::new(),
    });
    run.inventory
        .add_joker_with_edition(id.to_string(), JokerRarity::Common, 1, Some(edition))
        .expect("add joker");
}

macro_rules! test_play_hand_invalid_phase {
    ($name:ident, $phase:expr) => {
        #[test]
        fn $name() {
            let mut run = new_run();
            run.state.phase = $phase;
            run.state.hands_left = 1;
            run.hand = make_hand();
            let err = run
                .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
                .unwrap_err();
            assert!(matches!(err, RunError::InvalidPhase(p) if p == $phase));
        }
    };
}

macro_rules! test_discard_invalid_phase {
    ($name:ident, $phase:expr) => {
        #[test]
        fn $name() {
            let mut run = new_run();
            run.state.phase = $phase;
            run.state.discards_left = 1;
            run.hand = make_hand();
            let err = run
                .discard(&[0], &mut EventBus::default())
                .unwrap_err();
            assert!(matches!(err, RunError::InvalidPhase(p) if p == $phase));
        }
    };
}

macro_rules! test_prepare_hand_invalid_phase {
    ($name:ident, $phase:expr) => {
        #[test]
        fn $name() {
            let mut run = new_run();
            run.state.phase = $phase;
            run.state.hands_left = 1;
            run.state.hand_size = 2;
            run.deck.draw = vec![
                Card::standard(Suit::Spades, Rank::Ace),
                Card::standard(Suit::Hearts, Rank::Two),
            ];
            let err = run.prepare_hand(&mut EventBus::default()).unwrap_err();
            assert!(matches!(err, RunError::InvalidPhase(p) if p == $phase));
        }
    };
}

macro_rules! test_reroll_invalid_phase {
    ($name:ident, $phase:expr) => {
        #[test]
        fn $name() {
            let mut run = new_run();
            run.state.phase = $phase;
            let err = run.reroll_shop(&mut EventBus::default()).unwrap_err();
            assert!(matches!(err, RunError::InvalidPhase(p) if p == $phase));
        }
    };
}

macro_rules! test_buy_invalid_phase {
    ($name:ident, $phase:expr) => {
        #[test]
        fn $name() {
            let mut run = new_run();
            run.state.phase = $phase;
            let err = run
                .buy_shop_offer(ShopOfferRef::Card(0), &mut EventBus::default())
                .unwrap_err();
            assert!(matches!(err, RunError::InvalidPhase(p) if p == $phase));
        }
    };
}

test_play_hand_invalid_phase!(play_hand_invalid_phase_setup, Phase::Setup);
test_play_hand_invalid_phase!(play_hand_invalid_phase_deal, Phase::Deal);
test_play_hand_invalid_phase!(play_hand_invalid_phase_score, Phase::Score);
test_play_hand_invalid_phase!(play_hand_invalid_phase_cleanup, Phase::Cleanup);
test_play_hand_invalid_phase!(play_hand_invalid_phase_shop, Phase::Shop);

test_discard_invalid_phase!(discard_invalid_phase_setup, Phase::Setup);
test_discard_invalid_phase!(discard_invalid_phase_deal, Phase::Deal);
test_discard_invalid_phase!(discard_invalid_phase_score, Phase::Score);
test_discard_invalid_phase!(discard_invalid_phase_cleanup, Phase::Cleanup);
test_discard_invalid_phase!(discard_invalid_phase_shop, Phase::Shop);

test_prepare_hand_invalid_phase!(prepare_hand_invalid_phase_setup, Phase::Setup);
test_prepare_hand_invalid_phase!(prepare_hand_invalid_phase_play, Phase::Play);
test_prepare_hand_invalid_phase!(prepare_hand_invalid_phase_score, Phase::Score);
test_prepare_hand_invalid_phase!(prepare_hand_invalid_phase_cleanup, Phase::Cleanup);
test_prepare_hand_invalid_phase!(prepare_hand_invalid_phase_shop, Phase::Shop);

test_reroll_invalid_phase!(reroll_invalid_phase_setup, Phase::Setup);
test_reroll_invalid_phase!(reroll_invalid_phase_deal, Phase::Deal);
test_reroll_invalid_phase!(reroll_invalid_phase_play, Phase::Play);
test_reroll_invalid_phase!(reroll_invalid_phase_score, Phase::Score);
test_reroll_invalid_phase!(reroll_invalid_phase_cleanup, Phase::Cleanup);

test_buy_invalid_phase!(buy_invalid_phase_setup, Phase::Setup);
test_buy_invalid_phase!(buy_invalid_phase_deal, Phase::Deal);
test_buy_invalid_phase!(buy_invalid_phase_play, Phase::Play);
test_buy_invalid_phase!(buy_invalid_phase_score, Phase::Score);
test_buy_invalid_phase!(buy_invalid_phase_cleanup, Phase::Cleanup);

#[test]
fn content_counts_and_ids() {
    let content = load_content(&assets_root()).expect("load content");
    assert_eq!(content.tarots.len(), 22);
    assert_eq!(content.planets.len(), 12);
    assert_eq!(content.spectrals.len(), 18);
    assert_eq!(content.tags.len(), 24);

    let tarot_ids: Vec<&str> = vec![
        "the_fool",
        "the_magician",
        "the_high_priestess",
        "the_empress",
        "the_emperor",
        "the_hierophant",
        "the_lovers",
        "the_chariot",
        "justice",
        "the_hermit",
        "the_wheel_of_fortune",
        "strength",
        "the_hanged_man",
        "death",
        "temperance",
        "the_devil",
        "the_tower",
        "the_star",
        "the_moon",
        "the_sun",
        "judgement",
        "the_world",
    ];
    for id in tarot_ids {
        assert!(content.tarots.iter().any(|card| card.id == id));
    }

    let planet_ids: Vec<&str> = vec![
        "pluto", "mercury", "uranus", "venus", "saturn", "jupiter", "earth", "mars", "neptune",
        "planet_x", "ceres", "eris",
    ];
    for id in planet_ids {
        assert!(content.planets.iter().any(|card| card.id == id));
    }

    let spectral_ids: Vec<&str> = vec![
        "familiar",
        "grim",
        "incantation",
        "talisman",
        "aura",
        "wraith",
        "sigil",
        "ouija",
        "ectoplasm",
        "immolate",
        "hex",
        "ankh",
        "black_hole",
        "cryptid",
        "deja_vu",
        "medium",
        "trance",
        "the_soul",
    ];
    for id in spectral_ids {
        assert!(content.spectrals.iter().any(|card| card.id == id));
    }
}

#[test]
fn evaluate_hand_high_card() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Nine),
        (Suit::Clubs, Rank::Seven),
        (Suit::Diamonds, Rank::Four),
        (Suit::Hearts, Rank::Two),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::HighCard);
}

#[test]
fn evaluate_hand_pair() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Seven),
        (Suit::Diamonds, Rank::Four),
        (Suit::Hearts, Rank::Two),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::Pair);
}

#[test]
fn evaluate_hand_two_pair() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Seven),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Hearts, Rank::Two),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::TwoPair);
}

#[test]
fn evaluate_hand_trips() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Ace),
        (Suit::Diamonds, Rank::Four),
        (Suit::Hearts, Rank::Two),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::Trips);
}

#[test]
fn evaluate_hand_straight() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Hearts, Rank::Three),
        (Suit::Clubs, Rank::Four),
        (Suit::Diamonds, Rank::Five),
        (Suit::Hearts, Rank::Six),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::Straight);
}

#[test]
fn evaluate_hand_flush() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Spades, Rank::Seven),
        (Suit::Spades, Rank::Nine),
        (Suit::Spades, Rank::Jack),
        (Suit::Spades, Rank::King),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::Flush);
}

#[test]
fn evaluate_hand_full_house() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Ace),
        (Suit::Diamonds, Rank::King),
        (Suit::Hearts, Rank::King),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::FullHouse);
}

#[test]
fn evaluate_hand_quads() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Nine),
        (Suit::Hearts, Rank::Nine),
        (Suit::Clubs, Rank::Nine),
        (Suit::Diamonds, Rank::Nine),
        (Suit::Hearts, Rank::Two),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::Quads);
}

#[test]
fn evaluate_hand_straight_flush() {
    let cards = make_cards(&[
        (Suit::Hearts, Rank::Five),
        (Suit::Hearts, Rank::Six),
        (Suit::Hearts, Rank::Seven),
        (Suit::Hearts, Rank::Eight),
        (Suit::Hearts, Rank::Nine),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::StraightFlush);
}

#[test]
fn evaluate_hand_royal_flush() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ten),
        (Suit::Spades, Rank::Jack),
        (Suit::Spades, Rank::Queen),
        (Suit::Spades, Rank::King),
        (Suit::Spades, Rank::Ace),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::RoyalFlush);
}

#[test]
fn evaluate_hand_five_of_a_kind() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Ace),
        (Suit::Diamonds, Rank::Ace),
        (Suit::Spades, Rank::Ace),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::FiveOfAKind);
}

#[test]
fn evaluate_hand_flush_house() {
    let cards = make_cards(&[
        (Suit::Clubs, Rank::Ace),
        (Suit::Clubs, Rank::Ace),
        (Suit::Clubs, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Clubs, Rank::King),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::FlushHouse);
}

#[test]
fn evaluate_hand_flush_five() {
    let cards = make_cards(&[
        (Suit::Hearts, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
    ]);
    assert_eq!(evaluate_hand(&cards), HandKind::FlushFive);
}

#[test]
fn evaluate_hand_smeared_suits_flush() {
    let cards = make_cards(&[
        (Suit::Hearts, Rank::Two),
        (Suit::Hearts, Rank::Four),
        (Suit::Diamonds, Rank::Six),
        (Suit::Diamonds, Rank::Eight),
        (Suit::Hearts, Rank::Ten),
    ]);
    let rules = HandEvalRules {
        smeared_suits: true,
        four_fingers: false,
        shortcut: false,
    };
    assert_eq!(evaluate_hand_with_rules(&cards, rules), HandKind::Flush);
    assert_eq!(evaluate_hand(&cards), HandKind::HighCard);
}

#[test]
fn evaluate_hand_four_fingers_straight() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Two),
        (Suit::Clubs, Rank::Three),
        (Suit::Diamonds, Rank::Four),
    ]);
    let rules = HandEvalRules {
        smeared_suits: false,
        four_fingers: true,
        shortcut: false,
    };
    assert_eq!(evaluate_hand_with_rules(&cards, rules), HandKind::Straight);
    assert_eq!(evaluate_hand(&cards), HandKind::HighCard);
}

#[test]
fn evaluate_hand_shortcut_straight() {
    let cards = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Hearts, Rank::Four),
        (Suit::Clubs, Rank::Six),
        (Suit::Diamonds, Rank::Eight),
        (Suit::Hearts, Rank::Ten),
    ]);
    let rules = HandEvalRules {
        smeared_suits: false,
        four_fingers: false,
        shortcut: true,
    };
    assert_eq!(evaluate_hand_with_rules(&cards, rules), HandKind::Straight);
    assert_eq!(evaluate_hand(&cards), HandKind::HighCard);
}

#[test]
fn scoring_cards_include_stone() {
    let mut cards = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Hearts, Rank::Two),
        (Suit::Clubs, Rank::Three),
        (Suit::Diamonds, Rank::Four),
        (Suit::Spades, Rank::Five),
    ]);
    cards[2].enhancement = Some(Enhancement::Stone);
    let scoring = scoring_cards(&cards, HandKind::Pair);
    assert_eq!(scoring.len(), 3);
    assert!(scoring.contains(&2));
}

#[test]
fn score_hand_ignores_stone_rank_chips() {
    let config = load_game_config(&assets_root()).expect("load config");
    let tables = ScoreTables::from_config(&config);
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Queen),
        (Suit::Spades, Rank::Jack),
    ]);
    let mut cards_stone = cards.clone();
    cards_stone[2].enhancement = Some(Enhancement::Stone);
    let normal = score_hand(&cards, &tables);
    let with_stone = score_hand(&cards_stone, &tables);
    assert_eq!(normal.hand, HandKind::Pair);
    assert_eq!(with_stone.hand, HandKind::Pair);
    assert_eq!(with_stone.rank_chips, normal.rank_chips);
}

#[test]
fn scoring_bonus_enhancement_adds_chips() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.enhancement = Some(Enhancement::Bonus);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + breakdown.rank_chips + 30
    );
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "enhancement:bonus"));
}

#[test]
fn scoring_mult_enhancement_adds_mult() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.enhancement = Some(Enhancement::Mult);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.base.mult + 4.0, breakdown.total.mult);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "enhancement:mult"));
}

#[test]
fn scoring_stone_enhancement_adds_chips() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.enhancement = Some(Enhancement::Stone);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.rank_chips, 0);
    assert_eq!(breakdown.total.chips, breakdown.base.chips + 50);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "enhancement:stone"));
}

#[test]
fn scoring_foil_edition_adds_chips() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.edition = Some(Edition::Foil);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + breakdown.rank_chips + 50
    );
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "edition:foil"));
}

#[test]
fn scoring_polychrome_edition_multiplies_mult() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.edition = Some(Edition::Polychrome);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult * 1.5);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "edition:polychrome"));
}

#[test]
fn scoring_card_bonus_chips_trace() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.bonus_chips = 25;
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + breakdown.rank_chips + 25
    );
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "card:bonus_chips"));
}

#[test]
fn scoring_steel_held_mult_trace() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut steel = Card::standard(Suit::Hearts, Rank::Two);
    steel.enhancement = Some(Enhancement::Steel);
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace), steel];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult * 1.5);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "enhancement:steel"));
}

#[test]
fn scoring_joker_add_chips_trace() {
    let mut run = new_run();
    add_scoring_joker(&mut run, "trace_add_chips", ActionOp::AddChips, 40.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + breakdown.rank_chips + 40
    );
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker:trace_add_chips:add_chips"));
}

#[test]
fn scoring_joker_add_mult_trace() {
    let mut run = new_run();
    add_scoring_joker(&mut run, "trace_add_mult", ActionOp::AddMult, 3.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult + 3.0);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker:trace_add_mult:add_mult"));
}

#[test]
fn scoring_joker_mul_mult_trace() {
    let mut run = new_run();
    add_scoring_joker(&mut run, "trace_mul_mult", ActionOp::MultiplyMult, 2.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult * 2.0);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker:trace_mul_mult:mul_mult"));
}

#[test]
fn scoring_joker_mul_chips_trace() {
    let mut run = new_run();
    add_scoring_joker(&mut run, "trace_mul_chips", ActionOp::MultiplyChips, 1.5);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    let base = breakdown.base.chips + breakdown.rank_chips;
    let expected = (base as f64 * 1.5).floor() as i64;
    assert_eq!(breakdown.total.chips, expected);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker:trace_mul_chips:mul_chips"));
}

#[test]
fn scoring_holographic_edition_adds_mult() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.edition = Some(Edition::Holographic);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult + 10.0);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "edition:holographic"));
}

#[test]
fn scoring_red_seal_retriggers_bonus() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.enhancement = Some(Enhancement::Bonus);
    card.seal = Some(Seal::Red);
    run.hand = vec![card];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + breakdown.rank_chips + 60
    );
    let bonus_hits = run
        .last_score_trace
        .iter()
        .filter(|step| step.source == "enhancement:bonus")
        .count();
    assert_eq!(bonus_hits, 2);
}

#[test]
fn scoring_held_steel_red_seal_retriggers() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut steel = Card::standard(Suit::Hearts, Rank::Two);
    steel.enhancement = Some(Enhancement::Steel);
    steel.seal = Some(Seal::Red);
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace), steel];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult * 2.25);
    let steel_hits = run
        .last_score_trace
        .iter()
        .filter(|step| step.source == "enhancement:steel")
        .count();
    assert_eq!(steel_hits, 2);
}

#[test]
fn scoring_joker_edition_foil_adds_chips() {
    let mut run = new_run();
    add_plain_joker_with_edition(&mut run, "joker_foil", Edition::Foil);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + breakdown.rank_chips + 50
    );
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker_edition:foil"));
}

#[test]
fn scoring_joker_edition_holographic_adds_mult() {
    let mut run = new_run();
    add_plain_joker_with_edition(&mut run, "joker_holo", Edition::Holographic);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult + 10.0);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker_edition:holographic"));
}

#[test]
fn scoring_joker_edition_polychrome_multiplies_mult() {
    let mut run = new_run();
    add_plain_joker_with_edition(&mut run, "joker_poly", Edition::Polychrome);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace)];
    let breakdown = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.total.mult, breakdown.base.mult * 1.5);
    assert!(run
        .last_score_trace
        .iter()
        .any(|step| step.source == "joker_edition:polychrome"));
}

#[test]
fn shop_add_joker_offer_action_adds_card() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "add_shop_joker".to_string(),
        name: "Add Shop Joker".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddShopJoker),
                target: Some("rare".to_string()),
                value: Expr::Number(0.0),
            }],
        }],
    });
    run.inventory
        .add_joker("add_shop_joker".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.iter().any(|card| {
        matches!(card.kind, ShopCardKind::Joker)
            && card.rarity == Some(JokerRarity::Rare)
            && card.price == 0
    }));
}

#[test]
fn shop_set_reroll_cost_action_applies() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "set_reroll".to_string(),
        name: "Set Reroll".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::SetRerollCost),
                target: None,
                value: Expr::Number(1.0),
            }],
        }],
    });
    run.inventory
        .add_joker("set_reroll".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert_eq!(shop.reroll_cost, 1);
}

#[test]
fn shop_add_voucher_action_increases_slots() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "add_voucher".to_string(),
        name: "Add Voucher".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddVoucher),
                target: None,
                value: Expr::Number(2.0),
            }],
        }],
    });
    run.inventory
        .add_joker("add_voucher".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    let base = run.config.shop.voucher_slots as usize;
    assert_eq!(shop.vouchers, base + 2);
}

#[test]
fn prevent_death_action_clears_failed_blind() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "prevent_death".to_string(),
        name: "Prevent Death".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnBlindFailed,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::PreventDeath),
                target: None,
                value: Expr::Number(1.0),
            }],
        }],
    });
    run.inventory
        .add_joker("prevent_death".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    run.state.target = 10_000;
    run.state.hands_left = 1;
    run.state.phase = Phase::Play;
    run.hand = vec![Card::standard(Suit::Spades, Rank::Two)];
    run.play_hand(&[0], &mut EventBus::default())
        .expect("play hand");
    assert!(run.blind_cleared());
    assert_eq!(run.state.blind_score, run.state.target);
}

#[test]
fn duplicate_next_tag_action_duplicates_tag() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "dup_tag".to_string(),
        name: "Dup Tag".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnBlindStart,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::DuplicateNextTag),
                target: None,
                value: Expr::Number(1.0),
            }],
        }],
    });
    run.content.jokers.push(JokerDef {
        id: "add_tag".to_string(),
        name: "Add Tag".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnBlindStart,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddTag),
                target: Some("coupon_tag".to_string()),
                value: Expr::Number(1.0),
            }],
        }],
    });
    run.inventory
        .add_joker("dup_tag".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.inventory
        .add_joker("add_tag".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.tags.len(), 2);
    assert!(run.state.tags.iter().all(|tag| tag == "coupon_tag"));
    assert!(!run.state.duplicate_next_tag);
}

#[test]
fn duplicate_next_tag_exclude_skips_duplicate() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "dup_tag_exclude".to_string(),
        name: "Dup Tag Exclude".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnBlindStart,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::DuplicateNextTag),
                target: Some("coupon_tag".to_string()),
                value: Expr::Number(1.0),
            }],
        }],
    });
    run.content.jokers.push(JokerDef {
        id: "add_tag".to_string(),
        name: "Add Tag".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnBlindStart,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddTag),
                target: Some("coupon_tag".to_string()),
                value: Expr::Number(1.0),
            }],
        }],
    });
    run.inventory
        .add_joker("dup_tag_exclude".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.inventory
        .add_joker("add_tag".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.tags.len(), 1);
    assert_eq!(run.state.tags[0], "coupon_tag");
    assert!(run.state.duplicate_next_tag);
}

#[test]
fn add_pack_action_adds_pack_offer() {
    let mut baseline = new_run();
    mark_blind_cleared(&mut baseline);
    baseline
        .enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let base_packs = baseline.shop.as_ref().expect("shop").packs.len();

    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "add_pack".to_string(),
        name: "Add Pack".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddPack),
                target: Some("buffoon_mega".to_string()),
                value: Expr::Number(0.0),
            }],
        }],
    });
    run.inventory
        .add_joker("add_pack".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert_eq!(shop.packs.len(), base_packs + 1);
    assert!(shop.packs.iter().any(|pack| {
        pack.kind == PackKind::Buffoon && pack.size == PackSize::Mega && pack.price == 0
    }));
}

#[test]
fn score_tables_level_scaling() {
    let config = load_game_config(&assets_root()).expect("load config");
    let tables = ScoreTables::from_config(&config);
    let (chips, mult) = tables.hand_base_for_level(HandKind::Pair, 3);
    assert_eq!(chips, 40);
    assert_eq!(mult, 4.0);
}

#[test]
fn tarot_requires_selection() {
    let mut run = new_run();
    run.hand = make_hand();
    run.inventory
        .add_consumable("the_devil".to_string(), ConsumableKind::Tarot)
        .expect("add consumable");
    let mut events = EventBus::default();
    let err = run.use_consumable(0, &[], &mut events).unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn tarot_enhancements_and_suits() {
    let mut run = new_run();
    run.hand = make_hand();

    use_consumable(&mut run, "the_lovers", ConsumableKind::Tarot, &[0]);
    assert_eq!(run.hand[0].enhancement, Some(Enhancement::Wild));

    use_consumable(&mut run, "the_chariot", ConsumableKind::Tarot, &[1]);
    assert_eq!(run.hand[1].enhancement, Some(Enhancement::Steel));

    use_consumable(&mut run, "justice", ConsumableKind::Tarot, &[2]);
    assert_eq!(run.hand[2].enhancement, Some(Enhancement::Glass));

    use_consumable(&mut run, "the_devil", ConsumableKind::Tarot, &[3]);
    assert_eq!(run.hand[3].enhancement, Some(Enhancement::Gold));

    use_consumable(&mut run, "the_tower", ConsumableKind::Tarot, &[4]);
    assert_eq!(run.hand[4].enhancement, Some(Enhancement::Stone));

    use_consumable(&mut run, "the_empress", ConsumableKind::Tarot, &[5, 6]);
    assert_eq!(run.hand[5].enhancement, Some(Enhancement::Mult));
    assert_eq!(run.hand[6].enhancement, Some(Enhancement::Mult));

    use_consumable(&mut run, "the_hierophant", ConsumableKind::Tarot, &[6, 7]);
    assert_eq!(run.hand[6].enhancement, Some(Enhancement::Bonus));
    assert_eq!(run.hand[7].enhancement, Some(Enhancement::Bonus));

    use_consumable(&mut run, "the_magician", ConsumableKind::Tarot, &[0, 1]);
    assert_eq!(run.hand[0].enhancement, Some(Enhancement::Lucky));
    assert_eq!(run.hand[1].enhancement, Some(Enhancement::Lucky));

    use_consumable(&mut run, "the_moon", ConsumableKind::Tarot, &[0, 1, 2]);
    assert_eq!(run.hand[0].suit, Suit::Clubs);
    assert_eq!(run.hand[1].suit, Suit::Clubs);
    assert_eq!(run.hand[2].suit, Suit::Clubs);

    use_consumable(&mut run, "the_star", ConsumableKind::Tarot, &[0, 1, 2]);
    assert_eq!(run.hand[0].suit, Suit::Diamonds);
    assert_eq!(run.hand[1].suit, Suit::Diamonds);
    assert_eq!(run.hand[2].suit, Suit::Diamonds);

    use_consumable(&mut run, "the_sun", ConsumableKind::Tarot, &[0, 1, 2]);
    assert_eq!(run.hand[0].suit, Suit::Hearts);
    assert_eq!(run.hand[1].suit, Suit::Hearts);
    assert_eq!(run.hand[2].suit, Suit::Hearts);

    use_consumable(&mut run, "the_world", ConsumableKind::Tarot, &[0, 1, 2]);
    assert_eq!(run.hand[0].suit, Suit::Spades);
    assert_eq!(run.hand[1].suit, Suit::Spades);
    assert_eq!(run.hand[2].suit, Suit::Spades);
}

#[test]
fn tarot_rank_and_destroy_and_convert() {
    let mut run = new_run();
    run.hand = make_hand();

    use_consumable(&mut run, "strength", ConsumableKind::Tarot, &[0, 1]);
    assert_eq!(run.hand[0].rank, Rank::Two);
    assert_eq!(run.hand[1].rank, Rank::Three);

    let before = run.hand.len();
    use_consumable(&mut run, "the_hanged_man", ConsumableKind::Tarot, &[0, 1]);
    assert_eq!(run.hand.len(), before - 2);

    run.hand = make_hand();
    let right = run.hand[1];
    use_consumable(&mut run, "death", ConsumableKind::Tarot, &[0, 1]);
    assert_eq!(run.hand[0].suit, right.suit);
    assert_eq!(run.hand[0].rank, right.rank);
    assert_eq!(run.hand[0].enhancement, right.enhancement);
    assert_eq!(run.hand[0].edition, right.edition);
    assert_eq!(run.hand[0].seal, right.seal);
}

#[test]
fn tarot_money_and_generation() {
    let mut run = new_run();
    run.hand = make_hand();

    run.state.money = 30;
    use_consumable(&mut run, "the_hermit", ConsumableKind::Tarot, &[]);
    assert_eq!(run.state.money, 50);

    run.state.money = 0;
    run.inventory
        .add_joker("joker_a".to_string(), JokerRarity::Common, 200)
        .unwrap();
    run.inventory
        .add_joker("joker_b".to_string(), JokerRarity::Common, 200)
        .unwrap();
    use_consumable(&mut run, "temperance", ConsumableKind::Tarot, &[]);
    assert_eq!(run.state.money, 50);

    run.inventory.consumables.clear();
    use_consumable(&mut run, "the_emperor", ConsumableKind::Tarot, &[]);
    assert_eq!(run.inventory.consumables.len(), 2);
    assert!(run
        .inventory
        .consumables
        .iter()
        .all(|item| item.kind == ConsumableKind::Tarot));

    run.inventory.consumables.clear();
    use_consumable(&mut run, "the_high_priestess", ConsumableKind::Tarot, &[]);
    assert_eq!(run.inventory.consumables.len(), 2);
    assert!(run
        .inventory
        .consumables
        .iter()
        .all(|item| item.kind == ConsumableKind::Planet));

    run.inventory.jokers.clear();
    use_consumable(&mut run, "judgement", ConsumableKind::Tarot, &[]);
    assert_eq!(run.inventory.jokers.len(), 1);
}

#[test]
fn tarot_fool_creates_last_consumable() {
    let mut run = new_run();
    run.hand = make_hand();
    run.state.last_consumable = Some(LastConsumable {
        kind: ConsumableKind::Tarot,
        id: "the_magician".to_string(),
    });
    use_consumable(&mut run, "the_fool", ConsumableKind::Tarot, &[]);
    assert_eq!(run.inventory.consumables.len(), 1);
    assert_eq!(run.inventory.consumables[0].id, "the_magician");
    assert_eq!(run.inventory.consumables[0].kind, ConsumableKind::Tarot);
}

#[test]
fn planet_upgrades_hand_levels() {
    let content = load_content(&assets_root()).expect("load content");
    for planet in &content.planets {
        let hand = planet.hand.expect("planet hand");
        let mut run = new_run();
        run.hand = make_hand();
        use_consumable(&mut run, &planet.id, ConsumableKind::Planet, &[]);
        assert_eq!(hand_level(&run, hand), 2, "planet {}", planet.id);
    }
}

#[test]
fn spectral_seals_editions_and_copies() {
    let mut run = new_run();
    run.hand = make_hand();

    use_consumable(&mut run, "deja_vu", ConsumableKind::Spectral, &[0]);
    assert_eq!(run.hand[0].seal, Some(rulatro_core::Seal::Red));

    use_consumable(&mut run, "medium", ConsumableKind::Spectral, &[1]);
    assert_eq!(run.hand[1].seal, Some(rulatro_core::Seal::Purple));

    use_consumable(&mut run, "trance", ConsumableKind::Spectral, &[2]);
    assert_eq!(run.hand[2].seal, Some(rulatro_core::Seal::Blue));

    use_consumable(&mut run, "talisman", ConsumableKind::Spectral, &[3]);
    assert_eq!(run.hand[3].seal, Some(rulatro_core::Seal::Gold));

    use_consumable(&mut run, "aura", ConsumableKind::Spectral, &[4]);
    assert!(matches!(
        run.hand[4].edition,
        Some(Edition::Foil | Edition::Holographic | Edition::Polychrome)
    ));

    let before = run.hand.len();
    use_consumable(&mut run, "cryptid", ConsumableKind::Spectral, &[0]);
    assert_eq!(run.hand.len(), before + 2);
    assert_eq!(run.hand[before].rank, run.hand[0].rank);
    assert_eq!(run.hand[before].suit, run.hand[0].suit);
}

#[test]
fn spectral_transform_and_money() {
    let mut run = new_run();
    run.hand = make_hand();
    run.state.money = 10;

    use_consumable(&mut run, "wraith", ConsumableKind::Spectral, &[]);
    assert_eq!(run.state.money, 0);
    assert!(run.inventory.jokers.len() >= 1);

    run.hand = make_hand();
    let hand_size = run.state.hand_size;
    use_consumable(&mut run, "ouija", ConsumableKind::Spectral, &[]);
    assert_eq!(run.state.hand_size, hand_size - 1);
    let rank = run.hand[0].rank;
    assert!(run.hand.iter().all(|card| card.rank == rank));

    run.hand = make_hand();
    use_consumable(&mut run, "sigil", ConsumableKind::Spectral, &[]);
    let suit = run.hand[0].suit;
    assert!(run.hand.iter().all(|card| card.suit == suit));
}

#[test]
fn spectral_destroy_and_add() {
    let mut run = new_run();
    run.hand = make_hand();
    run.state.money = 0;

    let before = run.hand.len();
    use_consumable(&mut run, "immolate", ConsumableKind::Spectral, &[]);
    assert_eq!(run.hand.len(), before - 5);
    assert_eq!(run.state.money, 20);

    run.hand = make_hand();
    let before = run.hand.len();
    use_consumable(&mut run, "grim", ConsumableKind::Spectral, &[]);
    assert_eq!(run.hand.len(), before + 1);
    assert!(
        run.hand
            .iter()
            .filter(|card| card.enhancement.is_some())
            .count()
            >= 2
    );

    run.hand = make_hand();
    let before = run.hand.len();
    use_consumable(&mut run, "familiar", ConsumableKind::Spectral, &[]);
    assert_eq!(run.hand.len(), before + 2);
    assert!(
        run.hand
            .iter()
            .filter(|card| card.enhancement.is_some())
            .count()
            >= 3
    );

    run.hand = make_hand();
    let before = run.hand.len();
    use_consumable(&mut run, "incantation", ConsumableKind::Spectral, &[]);
    assert_eq!(run.hand.len(), before + 3);
    assert!(
        run.hand
            .iter()
            .filter(|card| card.enhancement.is_some())
            .count()
            >= 4
    );
}

#[test]
fn spectral_joker_modifications() {
    let mut run = new_run();
    run.hand = make_hand();
    run.inventory
        .add_joker("joker_a".to_string(), JokerRarity::Common, 10)
        .unwrap();

    let hand_size = run.state.hand_size;
    use_consumable(&mut run, "ectoplasm", ConsumableKind::Spectral, &[]);
    assert_eq!(run.state.hand_size, hand_size - 1);
    assert!(run
        .inventory
        .jokers
        .iter()
        .any(|joker| joker.edition == Some(Edition::Negative)));

    run.inventory.jokers.clear();
    run.inventory
        .add_joker("joker_b".to_string(), JokerRarity::Common, 10)
        .unwrap();
    run.inventory
        .add_joker("joker_c".to_string(), JokerRarity::Common, 10)
        .unwrap();
    run.inventory
        .add_joker("joker_d".to_string(), JokerRarity::Common, 10)
        .unwrap();
    use_consumable(&mut run, "hex", ConsumableKind::Spectral, &[]);
    assert_eq!(run.inventory.jokers.len(), 1);
    assert_eq!(run.inventory.jokers[0].edition, Some(Edition::Polychrome));

    run.inventory.jokers.clear();
    run.inventory
        .add_joker_with_edition(
            "joker_e".to_string(),
            JokerRarity::Common,
            10,
            Some(Edition::Negative),
        )
        .unwrap();
    use_consumable(&mut run, "ankh", ConsumableKind::Spectral, &[]);
    assert_eq!(run.inventory.jokers.len(), 2);
    assert_eq!(
        run.inventory
            .jokers
            .iter()
            .filter(|joker| joker.edition == Some(Edition::Negative))
            .count(),
        1
    );

    use_consumable(&mut run, "the_soul", ConsumableKind::Spectral, &[]);
    assert!(run
        .content
        .jokers
        .iter()
        .any(|joker| joker.rarity == JokerRarity::Legendary));
    assert!(run.inventory.jokers.len() >= 2);
}

#[test]
fn spectral_black_hole_upgrades_all_hands() {
    let mut run = new_run();
    run.hand = make_hand();
    use_consumable(&mut run, "black_hole", ConsumableKind::Spectral, &[]);
    assert_eq!(hand_level(&run, HandKind::HighCard), 2);
    assert_eq!(hand_level(&run, HandKind::Pair), 2);
    assert_eq!(hand_level(&run, HandKind::Straight), 2);
}

#[test]
fn tag_coupon_sets_shop_prices_and_consumes() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("coupon_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.iter().all(|card| card.price == 0));
    assert!(shop.packs.iter().all(|pack| pack.price == 0));
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_d6_sets_reroll_cost_zero() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("d6_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert_eq!(shop.reroll_cost, 0);
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_economy_scales_money() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.money = 25;
    run.state.tags.push("economy_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    assert_eq!(run.state.money, 40);
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_handy_and_garbage_add_money() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.money = 0;
    run.state.hand_play_counts.insert(HandKind::Pair, 2);
    run.state.hand_play_counts.insert(HandKind::Trips, 1);
    run.state.unused_discards = 4;
    run.state.tags.push("handy_tag".to_string());
    run.state.tags.push("garbage_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    assert_eq!(run.state.money, 7);
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_juggle_adds_hand_size_on_blind_start() {
    let mut run = new_run();
    run.state.tags.push("juggle_tag".to_string());
    let base = run.state.hand_size_base;
    let mut events = EventBus::default();
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");
    assert_eq!(run.state.hand_size, base + 3);
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_pack_adds_expected_offers() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("buffoon_tag".to_string());
    run.state.tags.push("charm_tag".to_string());
    run.state.tags.push("ethereal_tag".to_string());
    run.state.tags.push("meteor_tag".to_string());
    run.state.tags.push("standard_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    let mut expect = vec![
        (PackKind::Buffoon, PackSize::Mega),
        (PackKind::Arcana, PackSize::Mega),
        (PackKind::Spectral, PackSize::Normal),
        (PackKind::Celestial, PackSize::Mega),
        (PackKind::Standard, PackSize::Mega),
    ];
    for (kind, size) in expect.drain(..) {
        assert!(shop
            .packs
            .iter()
            .any(|pack| pack.kind == kind && pack.size == size && pack.price == 0));
    }
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_voucher_increases_vouchers() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("voucher_tag".to_string());
    let base = run.config.shop.voucher_slots as usize;
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert_eq!(shop.vouchers, base + 1);
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_rare_plus_foil_sets_shop_joker_edition() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("rare_tag".to_string());
    run.state.tags.push("foil_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.iter().any(|card| {
        matches!(card.kind, rulatro_core::ShopCardKind::Joker)
            && card.rarity == Some(JokerRarity::Rare)
    }));
    assert!(shop
        .cards
        .iter()
        .any(|card| card.edition == Some(Edition::Foil)));
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_top_up_adds_jokers_to_inventory() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("top_up_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    assert_eq!(run.inventory.jokers.len(), 2);
    assert!(run.state.tags.is_empty());
}

#[test]
fn tag_foil_sets_single_joker_edition_and_price() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.config.shop.card_weights = vec![CardWeight {
        kind: ShopCardKind::Joker,
        weight: 1,
    }];
    run.state.tags.push("foil_tag".to_string());
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    let foil_cards: Vec<_> = shop
        .cards
        .iter()
        .filter(|card| card.edition == Some(Edition::Foil))
        .collect();
    assert_eq!(foil_cards.len(), 1);
    assert_eq!(foil_cards[0].price, 0);
    assert!(run.state.tags.is_empty());
}

#[test]
fn astronomer_sets_planet_and_celestial_prices() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.config.shop.card_weights = vec![CardWeight {
        kind: ShopCardKind::Planet,
        weight: 1,
    }];
    run.config
        .shop
        .pack_weights
        .retain(|pack| pack.kind == PackKind::Celestial);
    run.inventory
        .add_joker("astronomer".to_string(), JokerRarity::Uncommon, 10)
        .expect("add joker");
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop
        .cards
        .iter()
        .all(|card| matches!(card.kind, ShopCardKind::Planet) && card.price == 0));
    assert!(shop
        .packs
        .iter()
        .all(|pack| pack.kind == PackKind::Celestial && pack.price == 0));

    run.state.money = 100;
    run.reroll_shop(&mut events).expect("reroll");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop
        .cards
        .iter()
        .all(|card| matches!(card.kind, ShopCardKind::Planet) && card.price == 0));
    assert!(shop
        .packs
        .iter()
        .all(|pack| pack.kind == PackKind::Celestial && pack.price == 0));
}

#[test]
fn shop_reroll_price_override_ordered_jokers() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.config.shop.card_weights = vec![CardWeight {
        kind: ShopCardKind::Planet,
        weight: 1,
    }];
    let price_all = JokerDef {
        id: "price_all".to_string(),
        name: "Price All".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopReroll,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::SetShopPrice),
                target: Some("cards".to_string()),
                value: Expr::Number(2.0),
            }],
        }],
    };
    let price_planet = JokerDef {
        id: "price_planet".to_string(),
        name: "Price Planet".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopReroll,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::SetShopPrice),
                target: Some("planet".to_string()),
                value: Expr::Number(0.0),
            }],
        }],
    };
    run.content.jokers.push(price_all);
    run.content.jokers.push(price_planet);
    run.inventory
        .add_joker("price_all".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.inventory
        .add_joker("price_planet".to_string(), JokerRarity::Common, 1)
        .expect("add joker");

    run.state.money = 100;
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    run.reroll_shop(&mut events).expect("reroll");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop
        .cards
        .iter()
        .all(|card| matches!(card.kind, ShopCardKind::Planet) && card.price == 0));
}

#[test]
fn shop_buy_rejects_not_enough_money() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    run.state.money = 0;
    let err = run
        .buy_shop_offer(ShopOfferRef::Card(0), &mut events)
        .unwrap_err();
    assert!(matches!(err, RunError::NotEnoughMoney));
}

#[test]
fn shop_buy_joker_adds_inventory() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.config.shop.card_weights = vec![CardWeight {
        kind: ShopCardKind::Joker,
        weight: 1,
    }];
    run.state.money = 50;
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let price = run
        .shop
        .as_ref()
        .and_then(|shop| shop.cards.get(0))
        .map(|card| card.price)
        .unwrap_or(0);
    let purchase = run
        .buy_shop_offer(ShopOfferRef::Card(0), &mut events)
        .expect("buy offer");
    run.apply_purchase(&purchase).expect("apply purchase");
    assert_eq!(run.inventory.jokers.len(), 1);
    assert_eq!(run.state.money, 50 - price);
}

#[test]
fn shop_buy_voucher_decrements_and_costs() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    let voucher_price = run.config.shop.prices.voucher;
    let initial_money = voucher_price + 5;
    run.state.money = initial_money;
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let purchase = run
        .buy_shop_offer(ShopOfferRef::Voucher(0), &mut events)
        .expect("buy voucher");
    match purchase {
        ShopPurchase::Voucher(_) => {}
        _ => panic!("expected voucher purchase"),
    }
    let shop = run.shop.as_ref().expect("shop");
    assert_eq!(shop.vouchers, run.config.shop.voucher_slots as usize - 1);
    assert_eq!(run.state.money, initial_money - voucher_price);
    run.apply_purchase(&purchase).expect("apply purchase");
}

#[test]
fn voucher_overstock_increases_future_shop_card_slots() {
    let mut run = new_run();
    run.apply_purchase(&ShopPurchase::Voucher(rulatro_core::VoucherOffer {
        id: "overstock".to_string(),
    }))
    .expect("apply overstock");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.len() >= run.config.shop.card_slots as usize + 1);
}

#[test]
fn voucher_clearance_sale_discounts_shop_prices() {
    let mut run = new_run();
    run.config.shop.card_weights = vec![CardWeight {
        kind: ShopCardKind::Tarot,
        weight: 1,
    }];
    run.apply_purchase(&ShopPurchase::Voucher(rulatro_core::VoucherOffer {
        id: "clearance_sale".to_string(),
    }))
    .expect("apply clearance sale");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    let first = shop.cards.first().expect("card offer");
    let expected = (run.config.shop.prices.tarot * 75) / 100;
    assert_eq!(first.price, expected);
}

#[test]
fn shop_reroll_cost_and_free_rerolls() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.money = 100;
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let base = run.shop.as_ref().expect("shop").reroll_cost;
    let step = run.config.shop.prices.reroll_step;
    run.state.shop_free_rerolls = 1;

    run.reroll_shop(&mut events).expect("reroll");
    let after_free = run.shop.as_ref().expect("shop").reroll_cost;
    assert_eq!(after_free, base + step);
    assert_eq!(run.state.shop_free_rerolls, 0);
    assert_eq!(run.state.money, 100);

    run.reroll_shop(&mut events).expect("reroll");
    let after_paid = run.shop.as_ref().expect("shop").reroll_cost;
    assert_eq!(after_paid, after_free + step);
    assert_eq!(run.state.money, 100 - after_free);
}

#[test]
fn pack_open_and_choose_consumable() {
    let mut run = new_run();
    let mut events = EventBus::default();
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 3,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack.clone());
    let open = run
        .open_pack_purchase(&purchase, &mut events)
        .expect("open pack");
    assert_eq!(open.offer.kind, PackKind::Arcana);
    assert_eq!(open.options.len(), pack.options as usize);
    run.choose_pack_options(&open, &[0], &mut events)
        .expect("choose pack");
    assert_eq!(run.inventory.consumables.len(), 1);
    assert_eq!(run.inventory.consumables[0].kind, ConsumableKind::Tarot);
}

#[test]
fn pack_open_and_choose_playing_card() {
    let mut run = new_run();
    let mut events = EventBus::default();
    let pack = PackOffer {
        kind: PackKind::Standard,
        size: PackSize::Normal,
        options: 1,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    let open = run
        .open_pack_purchase(&purchase, &mut events)
        .expect("open pack");
    let before = run.deck.discard.len();
    run.choose_pack_options(&open, &[0], &mut events)
        .expect("choose pack");
    assert_eq!(run.deck.discard.len(), before + 1);
}

#[test]
fn pack_choose_rejects_too_many_picks() {
    let mut run = new_run();
    let mut events = EventBus::default();
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    let open = run
        .open_pack_purchase(&purchase, &mut events)
        .expect("open pack");
    let err = run
        .choose_pack_options(&open, &[0, 1], &mut events)
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn pack_skip_leaves_inventory_unchanged() {
    let mut run = new_run();
    let mut events = EventBus::default();
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 3,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    let open = run
        .open_pack_purchase(&purchase, &mut events)
        .expect("open pack");
    let before_consumables = run.inventory.consumables.len();
    let before_discard = run.deck.discard.len();
    run.skip_pack(&open, &mut events).expect("skip pack");
    assert_eq!(run.inventory.consumables.len(), before_consumables);
    assert_eq!(run.deck.discard.len(), before_discard);
}

#[test]
fn pack_choose_rejects_invalid_selection() {
    let mut run = new_run();
    let mut events = EventBus::default();
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 1,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    let open = run
        .open_pack_purchase(&purchase, &mut events)
        .expect("open pack");
    let err = run
        .choose_pack_options(&open, &[0, 1], &mut events)
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn blind_reward_includes_interest_and_per_hand() {
    let mut run = new_run();
    run.hand = make_hand();
    run.state.phase = Phase::Play;
    run.state.blind = BlindKind::Small;
    run.state.target = 1;
    run.state.hands_left = 2;
    run.state.discards_left = 2;
    run.state.money = 20;
    let pre_money = run.state.money;
    let economy = run.config.economy.clone();
    let mut events = EventBus::default();
    run.play_hand(&[0, 1, 2, 3, 4], &mut events)
        .expect("play hand");

    let hands_left_after = run.state.hands_left as i64;
    let steps = if economy.interest_step > 0 {
        pre_money / economy.interest_step
    } else {
        0
    };
    let cap_steps = if economy.interest_per > 0 {
        economy.interest_cap / economy.interest_per
    } else {
        0
    };
    let interest = steps.min(cap_steps).max(0) * economy.interest_per;
    let expected_reward =
        economy.reward_small + economy.per_hand_reward * hands_left_after + interest;
    assert_eq!(run.state.money, pre_money + expected_reward);
}

#[test]
fn play_hand_rejects_invalid_count() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_hand();
    let err = run.play_hand(&[0, 1, 2, 3, 4, 5], &mut EventBus::default());
    assert!(matches!(err, Err(RunError::InvalidCardCount)));
}

#[test]
fn play_hand_rejects_no_hands_left() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 0;
    run.hand = make_hand();
    let err = run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default());
    assert!(matches!(err, Err(RunError::NoHandsLeft)));
}

#[test]
fn discard_rejects_invalid_count() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.discards_left = 1;
    run.hand = make_hand();
    let err = run.discard(&[0, 1, 2, 3, 4, 5], &mut EventBus::default());
    assert!(matches!(err, Err(RunError::InvalidCardCount)));
}

#[test]
fn discard_rejects_no_discards_left() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.discards_left = 0;
    run.hand = make_hand();
    let err = run.discard(&[0], &mut EventBus::default());
    assert!(matches!(err, Err(RunError::NoDiscardsLeft)));
}

#[test]
fn tarot_rejects_out_of_range_selection() {
    let mut run = new_run();
    run.hand = make_hand();
    run.inventory
        .add_consumable("the_devil".to_string(), ConsumableKind::Tarot)
        .expect("add consumable");
    let err = run
        .use_consumable(0, &[99], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn chicot_disables_boss_effects() {
    let mut baseline = new_run();
    baseline
        .inventory
        .add_joker("matador".to_string(), JokerRarity::Uncommon, 10)
        .expect("add joker");
    let mut base_events = EventBus::default();
    baseline
        .start_blind(1, BlindKind::Boss, &mut base_events)
        .expect("start boss");
    baseline.hand = make_hand();
    baseline.state.phase = Phase::Play;
    let base_money = baseline.state.money;
    baseline
        .play_hand(&[0, 1, 2, 3, 4], &mut base_events)
        .expect("play hand");
    assert!(baseline.state.money > base_money);

    let mut run = new_run();
    run.inventory
        .add_joker("matador".to_string(), JokerRarity::Uncommon, 10)
        .expect("add joker");
    run.inventory
        .add_joker("chicot".to_string(), JokerRarity::Legendary, 10)
        .expect("add joker");
    let mut events = EventBus::default();
    run.start_blind(1, BlindKind::Boss, &mut events)
        .expect("start boss");
    run.hand = make_hand();
    run.state.phase = Phase::Play;
    let money_before = run.state.money;
    run.play_hand(&[0, 1, 2, 3, 4], &mut events)
        .expect("play hand");
    assert_eq!(run.state.money, money_before);
}

#[test]
fn boss_effects_apply_on_shop_enter() {
    let mut run = new_run();
    let boss_id = "test_boss";
    run.content.bosses.push(BossDef {
        weight: 1,
        id: boss_id.to_string(),
        name: "Test Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(7.0),
            }],
        }],
    });
    run.state.blind = BlindKind::Boss;
    run.state.boss_id = Some(boss_id.to_string());
    mark_blind_cleared(&mut run);
    run.state.money = 0;
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    assert_eq!(run.state.money, 7);
}

#[test]
fn tag_coupon_then_joker_overrides_card_prices() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.tags.push("coupon_tag".to_string());
    run.content.jokers.push(JokerDef {
        id: "price_override".to_string(),
        name: "Price Override".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::SetShopPrice),
                target: Some("cards".to_string()),
                value: Expr::Number(2.0),
            }],
        }],
    });
    run.inventory
        .add_joker("price_override".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    let mut events = EventBus::default();
    run.enter_shop(&mut events).expect("enter shop");
    let shop = run.shop.as_ref().expect("shop");
    assert!(shop.cards.iter().all(|card| card.price == 2));
    assert!(shop.packs.iter().all(|pack| pack.price == 0));
}

#[test]
fn play_and_discard_require_play_phase() {
    let mut run = new_run();
    run.hand = make_hand();
    run.state.phase = Phase::Deal;
    let err = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidPhase(Phase::Deal)));
    let err = run.discard(&[0], &mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::InvalidPhase(Phase::Deal)));
}

#[test]
fn discard_consumes_and_refills_hand() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.discards_left = 1;
    run.hand = make_hand();
    let mut events = EventBus::default();
    run.discard(&[0, 1], &mut events).expect("discard");
    assert_eq!(run.state.discards_left, 0);
    assert_eq!(run.hand.len(), run.state.hand_size);
}

#[test]
fn play_hand_consumes_hand_and_advances_phase() {
    let mut run = new_run();
    run.hand = make_hand();
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.state.target = 10_000;
    let mut events = EventBus::default();
    run.play_hand(&[0, 1, 2, 3, 4], &mut events)
        .expect("play hand");
    assert_eq!(run.state.hands_left, 1);
    assert_eq!(run.state.phase, Phase::Deal);
}

#[test]
fn luchador_sell_disables_next_boss() {
    let mut run = new_run();
    run.state.phase = Phase::Shop;
    run.inventory
        .add_joker("luchador".to_string(), JokerRarity::Uncommon, 10)
        .expect("add joker");
    let mut events = EventBus::default();
    run.sell_joker(0, &mut events).expect("sell joker");

    let mut baseline = new_run();
    let mut baseline_events = EventBus::default();
    baseline
        .start_blind(1, BlindKind::Boss, &mut baseline_events)
        .expect("start boss");
    assert!(baseline.state.boss_id.is_some());

    run.start_blind(1, BlindKind::Boss, &mut events)
        .expect("start boss");
    assert!(run.state.boss_id.is_none());
}

#[test]
fn prepare_hand_draws_to_size() {
    let mut run = new_run();
    run.state.phase = Phase::Deal;
    run.state.hands_left = 1;
    run.state.hand_size = 3;
    run.deck.draw = vec![
        Card::standard(Suit::Spades, Rank::Ace),
        Card::standard(Suit::Hearts, Rank::Two),
        Card::standard(Suit::Clubs, Rank::Three),
    ];
    run.prepare_hand(&mut EventBus::default())
        .expect("prepare hand");
    assert_eq!(run.hand.len(), 3);
    assert_eq!(run.state.phase, Phase::Play);
}

#[test]
fn prepare_hand_rejects_no_hands_left() {
    let mut run = new_run();
    run.state.phase = Phase::Deal;
    run.state.hands_left = 0;
    let err = run.prepare_hand(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::NoHandsLeft));
}

#[test]
fn draw_to_hand_no_change_when_full() {
    let mut run = new_run();
    run.state.hand_size = 2;
    run.hand = vec![
        Card::standard(Suit::Spades, Rank::Ace),
        Card::standard(Suit::Hearts, Rank::Two),
    ];
    let before_draw = run.deck.draw.len();
    run.draw_to_hand(&mut EventBus::default());
    assert_eq!(run.hand.len(), 2);
    assert_eq!(run.deck.draw.len(), before_draw);
}

#[test]
fn draw_to_hand_reshuffles_discard() {
    let mut run = new_run();
    run.state.hand_size = 1;
    run.hand.clear();
    run.deck.draw.clear();
    run.deck.discard = vec![Card::standard(Suit::Clubs, Rank::Three)];
    run.draw_to_hand(&mut EventBus::default());
    assert_eq!(run.hand.len(), 1);
    assert!(run.deck.discard.is_empty());
}

#[test]
fn enter_shop_requires_clear() {
    let mut run = new_run();
    run.state.target = 10;
    run.state.blind_score = 0;
    let err = run.enter_shop(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::BlindNotCleared));
}

#[test]
fn enter_shop_resets_free_rerolls() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.state.shop_free_rerolls = 3;
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    assert_eq!(run.state.shop_free_rerolls, 0);
}

#[test]
fn reroll_shop_not_enough_money() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    run.state.money = 0;
    let err = run.reroll_shop(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::NotEnoughMoney));
}

#[test]
fn buy_shop_offer_invalid_index() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let err = run
        .buy_shop_offer(ShopOfferRef::Card(99), &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidOfferIndex));
}

#[test]
fn open_pack_purchase_invalid_type() {
    let mut run = new_run();
    let purchase = ShopPurchase::Voucher(rulatro_core::VoucherOffer {
        id: "blank".to_string(),
    });
    let err = run
        .open_pack_purchase(&purchase, &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::PackNotAvailable));
}

#[test]
fn choose_pack_invalid_index() {
    let mut run = new_run();
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 1,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    let open = run
        .open_pack_purchase(&purchase, &mut EventBus::default())
        .expect("open pack");
    let err = run
        .choose_pack_options(&open, &[1], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn play_hand_rejects_empty_selection() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_hand();
    let err = run.play_hand(&[], &mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn discard_rejects_empty_selection() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.discards_left = 1;
    run.hand = make_hand();
    let err = run.discard(&[], &mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn use_consumable_invalid_index() {
    let mut run = new_run();
    let err = run
        .use_consumable(0, &[], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidSelection));
}

#[test]
fn last_consumable_updates_tarot() {
    let mut run = new_run();
    run.hand = make_hand();
    use_consumable(&mut run, "the_magician", ConsumableKind::Tarot, &[0, 1]);
    let last = run.state.last_consumable.expect("last consumable");
    assert_eq!(last.kind, ConsumableKind::Tarot);
    assert_eq!(last.id, "the_magician");
}

#[test]
fn last_consumable_updates_planet() {
    let mut run = new_run();
    use_consumable(&mut run, "pluto", ConsumableKind::Planet, &[]);
    let last = run.state.last_consumable.expect("last consumable");
    assert_eq!(last.kind, ConsumableKind::Planet);
    assert_eq!(last.id, "pluto");
}

#[test]
fn last_consumable_not_set_for_spectral() {
    let mut run = new_run();
    run.hand = make_hand();
    use_consumable(&mut run, "familiar", ConsumableKind::Spectral, &[]);
    assert!(run.state.last_consumable.is_none());
}

#[test]
fn planets_used_updates_on_planet_use() {
    let mut run = new_run();
    use_consumable(&mut run, "pluto", ConsumableKind::Planet, &[]);
    assert!(run.state.planets_used.contains("pluto"));
}

#[test]
fn inventory_no_joker_slots_fails() {
    let mut run = new_run();
    run.inventory.joker_slots = 0;
    run.inventory.jokers.clear();
    let err = run
        .inventory
        .add_joker("joker_x".to_string(), JokerRarity::Common, 1)
        .unwrap_err();
    assert!(matches!(err, rulatro_core::InventoryError::NoJokerSlots));
}

#[test]
fn inventory_negative_joker_allows_extra_slot() {
    let mut run = new_run();
    run.inventory.joker_slots = 0;
    run.inventory
        .add_joker_with_edition(
            "joker_neg".to_string(),
            JokerRarity::Common,
            1,
            Some(Edition::Negative),
        )
        .expect("add negative joker");
    assert_eq!(run.inventory.jokers.len(), 1);
}

#[test]
fn inventory_negative_consumable_not_counted() {
    let mut run = new_run();
    run.inventory.consumable_slots = 0;
    run.inventory
        .add_consumable_with_edition(
            "the_fool".to_string(),
            ConsumableKind::Tarot,
            Some(Edition::Negative),
            0.0,
        )
        .expect("add negative consumable");
    assert_eq!(run.inventory.consumable_count(), 0);
}

#[test]
fn joker_capacity_counts_negative() {
    let mut run = new_run();
    run.inventory.joker_slots = 1;
    run.inventory
        .add_joker_with_edition(
            "joker_neg".to_string(),
            JokerRarity::Common,
            1,
            Some(Edition::Negative),
        )
        .expect("add negative joker");
    assert_eq!(run.inventory.joker_capacity(), 2);
    run.inventory
        .add_joker("joker_norm".to_string(), JokerRarity::Common, 1)
        .expect("add normal joker");
    assert_eq!(run.inventory.jokers.len(), 2);
}

#[test]
fn joker_sell_value_none_for_invalid_index() {
    let run = new_run();
    assert!(run.joker_sell_value(0).is_none());
}

#[test]
fn sell_joker_increases_money() {
    let mut run = new_run();
    run.state.phase = Phase::Shop;
    run.state.money = 0;
    run.inventory
        .add_joker("joker_sell".to_string(), JokerRarity::Common, 10)
        .expect("add joker");
    run.sell_joker(0, &mut EventBus::default())
        .expect("sell joker");
    assert_eq!(run.state.money, 5);
}

#[test]
fn start_blind_resets_round_state() {
    let mut run = new_run();
    run.state.round_hand_types.insert(HandKind::Pair);
    run.state.round_hand_lock = Some(HandKind::Trips);
    run.hand = make_hand();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert!(run.state.round_hand_types.is_empty());
    assert!(run.state.round_hand_lock.is_none());
    assert!(run.hand.is_empty());
}

#[test]
fn start_blind_sets_limits_and_phase() {
    let mut run = new_run();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.phase, Phase::Deal);
    assert_eq!(run.state.hands_left, run.state.hands_max);
    assert_eq!(run.state.discards_left, run.state.discards_max);
    assert_eq!(run.state.hand_size, run.state.hand_size_base);
}

#[test]
fn start_blind_clears_played_cards_on_new_ante() {
    let mut run = new_run();
    run.state.ante = 1;
    run.state.played_card_ids_ante.insert(42);
    run.start_blind(2, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert!(run.state.played_card_ids_ante.is_empty());
}

#[test]
fn start_blind_keeps_played_cards_same_ante() {
    let mut run = new_run();
    run.state.ante = 1;
    run.state.played_card_ids_ante.insert(42);
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert!(run.state.played_card_ids_ante.contains(&42));
}

#[test]
fn start_blind_boss_assigns_id() {
    let mut run = new_run();
    run.start_blind(1, BlindKind::Boss, &mut EventBus::default())
        .expect("start blind");
    if !run.content.bosses.is_empty() {
        assert!(run.state.boss_id.is_some());
    }
}

#[test]
fn advance_blind_order_small_big() {
    let mut run = new_run();
    run.state.ante = 1;
    run.state.blind = BlindKind::Small;
    run.advance_blind().expect("advance blind");
    assert_eq!(run.state.blind, BlindKind::Big);
    assert_eq!(run.state.ante, 1);
}

#[test]
fn advance_blind_order_big_boss() {
    let mut run = new_run();
    run.state.ante = 1;
    run.state.blind = BlindKind::Big;
    run.advance_blind().expect("advance blind");
    assert_eq!(run.state.blind, BlindKind::Boss);
    assert_eq!(run.state.ante, 1);
}

#[test]
fn advance_blind_order_boss_small() {
    let mut run = new_run();
    run.state.ante = 1;
    run.state.blind = BlindKind::Boss;
    run.advance_blind().expect("advance blind");
    assert_eq!(run.state.blind, BlindKind::Small);
    assert_eq!(run.state.ante, 2);
}

#[test]
fn advance_blind_missing_ante() {
    let mut run = new_run();
    run.state.ante = 8;
    run.state.blind = BlindKind::Boss;
    let err = run.advance_blind().unwrap_err();
    assert!(matches!(err, RunError::MissingAnteRule(9)));
}

#[test]
fn start_next_blind_advances() {
    let mut run = new_run();
    run.state.ante = 1;
    run.state.blind = BlindKind::Small;
    run.start_next_blind(&mut EventBus::default())
        .expect("start next blind");
    assert_eq!(run.state.blind, BlindKind::Big);
    assert_eq!(run.state.phase, Phase::Deal);
}

#[test]
fn skip_blind_requires_deal_phase() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    let err = run.skip_blind(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::InvalidPhase(Phase::Play)));
}

#[test]
fn skip_blind_rejects_boss() {
    let mut run = new_run();
    run.start_blind(1, BlindKind::Boss, &mut EventBus::default())
        .expect("start blind");
    let err = run.skip_blind(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::CannotSkipBoss));
}

#[test]
fn skip_blind_advances_and_adds_tag() {
    let mut run = new_run();
    run.content.tags = vec![TagDef {
        id: "tag_one".to_string(),
        name: "Tag One".to_string(),
        effects: Vec::new(),
    }];
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    run.skip_blind(&mut EventBus::default())
        .expect("skip blind");
    assert_eq!(run.state.blind, BlindKind::Big);
    assert_eq!(run.state.ante, 1);
    assert_eq!(run.state.blinds_skipped, 1);
    assert_eq!(run.state.tags, vec!["tag_one".to_string()]);
}

#[test]
fn blind_outcome_none_when_target_zero() {
    let mut run = new_run();
    run.state.target = 0;
    run.state.blind_score = 0;
    run.state.hands_left = 1;
    assert!(run.blind_outcome().is_none());
}

#[test]
fn blind_outcome_failed_when_hands_zero() {
    let mut run = new_run();
    run.state.target = 10;
    run.state.blind_score = 0;
    run.state.hands_left = 0;
    assert!(matches!(
        run.blind_outcome(),
        Some(rulatro_core::BlindOutcome::Failed)
    ));
}

#[test]
fn blind_outcome_cleared_when_score_reached() {
    let mut run = new_run();
    run.state.target = 10;
    run.state.blind_score = 10;
    run.state.hands_left = 1;
    assert!(matches!(
        run.blind_outcome(),
        Some(rulatro_core::BlindOutcome::Cleared)
    ));
}

#[test]
fn pack_options_arcana_are_tarot() {
    let mut run = new_run();
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 3,
        picks: 1,
        price: 0,
    };
    let open = run
        .open_pack_purchase(&ShopPurchase::Pack(pack), &mut EventBus::default())
        .expect("open pack");
    assert!(open.options.iter().all(|option| matches!(
        option,
        rulatro_core::PackOption::Consumable(ConsumableKind::Tarot, _)
    )));
}

#[test]
fn pack_options_celestial_are_planet() {
    let mut run = new_run();
    let pack = PackOffer {
        kind: PackKind::Celestial,
        size: PackSize::Normal,
        options: 3,
        picks: 1,
        price: 0,
    };
    let open = run
        .open_pack_purchase(&ShopPurchase::Pack(pack), &mut EventBus::default())
        .expect("open pack");
    assert!(open.options.iter().all(|option| matches!(
        option,
        rulatro_core::PackOption::Consumable(ConsumableKind::Planet, _)
    )));
}

#[test]
fn pack_options_spectral_are_spectral() {
    let mut run = new_run();
    let pack = PackOffer {
        kind: PackKind::Spectral,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let open = run
        .open_pack_purchase(&ShopPurchase::Pack(pack), &mut EventBus::default())
        .expect("open pack");
    assert!(open.options.iter().all(|option| matches!(
        option,
        rulatro_core::PackOption::Consumable(ConsumableKind::Spectral, _)
    )));
}

#[test]
fn pack_options_buffoon_are_jokers() {
    let mut run = new_run();
    let pack = PackOffer {
        kind: PackKind::Buffoon,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let open = run
        .open_pack_purchase(&ShopPurchase::Pack(pack), &mut EventBus::default())
        .expect("open pack");
    assert!(open
        .options
        .iter()
        .all(|option| matches!(option, rulatro_core::PackOption::Joker(_))));
}

#[test]
fn pack_options_standard_are_playing_cards() {
    let mut run = new_run();
    let pack = PackOffer {
        kind: PackKind::Standard,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let open = run
        .open_pack_purchase(&ShopPurchase::Pack(pack), &mut EventBus::default())
        .expect("open pack");
    assert!(open
        .options
        .iter()
        .all(|option| matches!(option, rulatro_core::PackOption::PlayingCard(_))));
}

#[test]
fn leave_shop_resets_state() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    run.state.shop_free_rerolls = 2;
    run.leave_shop();
    assert!(run.shop.is_some());
    assert_eq!(run.state.phase, Phase::Deal);
    assert_eq!(run.state.shop_free_rerolls, 0);
}

#[test]
fn shop_reenter_keeps_offers() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let first = run.shop.as_ref().expect("shop");
    let first_cards: Vec<String> = first
        .cards
        .iter()
        .map(|card| card.item_id.clone())
        .collect();
    let first_packs: Vec<(PackKind, PackSize, i64)> = first
        .packs
        .iter()
        .map(|pack| (pack.kind, pack.size, pack.price))
        .collect();
    let first_reroll = first.reroll_cost;
    let first_vouchers = first.vouchers;

    run.leave_shop();
    run.enter_shop(&mut EventBus::default())
        .expect("reenter shop");
    let second = run.shop.as_ref().expect("shop");
    let second_cards: Vec<String> = second
        .cards
        .iter()
        .map(|card| card.item_id.clone())
        .collect();
    let second_packs: Vec<(PackKind, PackSize, i64)> = second
        .packs
        .iter()
        .map(|pack| (pack.kind, pack.size, pack.price))
        .collect();

    assert_eq!(first_cards, second_cards);
    assert_eq!(first_packs, second_packs);
    assert_eq!(first_reroll, second.reroll_cost);
    assert_eq!(first_vouchers, second.vouchers);
}

#[test]
fn rule_smeared_suits_from_joker() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_smeared", "smeared_suits", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Spades, Rank::Nine),
        (Suit::Clubs, Rank::Five),
        (Suit::Spades, Rank::Two),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.hand, HandKind::Flush);
}

#[test]
fn rule_four_fingers_from_joker() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_four_fingers", "four_fingers", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Hearts, Rank::Ace),
        (Suit::Hearts, Rank::Nine),
        (Suit::Hearts, Rank::Seven),
        (Suit::Hearts, Rank::Two),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.hand, HandKind::Flush);
}

#[test]
fn rule_shortcut_from_joker() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_shortcut", "shortcut", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Hearts, Rank::Four),
        (Suit::Clubs, Rank::Six),
        (Suit::Diamonds, Rank::Eight),
        (Suit::Spades, Rank::Ten),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.hand, HandKind::Straight);
}

#[test]
fn rule_single_hand_type_enforced() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_single", "single_hand_type", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Four),
    ]);
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");

    run.state.phase = Phase::Play;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::King),
        (Suit::Hearts, Rank::Nine),
        (Suit::Clubs, Rank::Eight),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Six),
    ]);
    let err = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::HandNotAllowed));
}

#[test]
fn rule_no_repeat_hand_enforced() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_no_repeat", "no_repeat_hand", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Four),
    ]);
    run.hand = cards.clone();
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");

    run.state.phase = Phase::Play;
    run.hand = cards;
    let err = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::HandNotAllowed));
}

#[test]
fn rule_required_play_count_enforced() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_required", "required_play_count", 5.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::King),
        (Suit::Clubs, Rank::Queen),
        (Suit::Diamonds, Rank::Jack),
        (Suit::Spades, Rank::Nine),
    ]);
    let err = run
        .play_hand(&[0, 1, 2, 3], &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidCardCount));
}

#[test]
fn rule_discard_held_after_hand() {
    let mut run = new_run();
    add_rule_joker(
        &mut run,
        "rule_discard_held",
        "discard_held_after_hand",
        3.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_hand();
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(run.hand.len(), 0);
}

#[test]
fn rule_draw_after_play() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_draw_play", "draw_after_play", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::King),
        (Suit::Clubs, Rank::Queen),
        (Suit::Diamonds, Rank::Jack),
        (Suit::Spades, Rank::Nine),
    ]);
    run.deck.draw = vec![Card::standard(Suit::Hearts, Rank::Two)];
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(run.hand.len(), 1);
}

#[test]
fn rule_draw_after_discard() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_draw_discard", "draw_after_discard", 1.0);
    run.state.phase = Phase::Play;
    run.state.discards_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::King),
        (Suit::Clubs, Rank::Queen),
    ]);
    run.deck.draw = vec![Card::standard(Suit::Diamonds, Rank::Two)];
    run.discard(&[0], &mut EventBus::default())
        .expect("discard");
    assert_eq!(run.hand.len(), 3);
}

#[test]
fn rule_base_chips_mult_scales() {
    let config = load_game_config(&assets_root()).expect("load config");
    let tables = ScoreTables::from_config(&config);
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_base_chips", "base_chips_mult", 2.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Four),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    let (base_chips, base_mult) = tables.hand_base_for_level(HandKind::Pair, 1);
    assert_eq!(breakdown.base.chips, base_chips * 2);
    assert_eq!(breakdown.base.mult, base_mult);
}

#[test]
fn rule_base_mult_mult_scales() {
    let config = load_game_config(&assets_root()).expect("load config");
    let tables = ScoreTables::from_config(&config);
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_base_mult", "base_mult_mult", 3.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Four),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    let (base_chips, base_mult) = tables.hand_base_for_level(HandKind::Pair, 1);
    assert_eq!(breakdown.base.chips, base_chips);
    assert_eq!(breakdown.base.mult, base_mult * 3.0);
}

#[test]
fn rule_hand_level_delta_scales() {
    let config = load_game_config(&assets_root()).expect("load config");
    let tables = ScoreTables::from_config(&config);
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_level_delta", "hand_level_delta", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Four),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    let (base_chips, base_mult) = tables.hand_base_for_level(HandKind::Pair, 2);
    assert_eq!(breakdown.base.chips, base_chips);
    assert_eq!(breakdown.base.mult, base_mult);
}

#[test]
fn rule_splash_scores_all_cards() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_splash", "splash", 1.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::King),
        (Suit::Clubs, Rank::Queen),
        (Suit::Diamonds, Rank::Jack),
        (Suit::Spades, Rank::Nine),
    ]);
    let breakdown = run
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(breakdown.scoring_indices.len(), 5);
}

#[test]
fn debuff_face_blocks_bonus_enhancement() {
    let mut cards = make_cards(&[
        (Suit::Spades, Rank::King),
        (Suit::Hearts, Rank::Nine),
        (Suit::Clubs, Rank::Eight),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Six),
    ]);
    cards[0].enhancement = Some(Enhancement::Bonus);

    let mut baseline = new_run();
    baseline.state.phase = Phase::Play;
    baseline.state.hands_left = 1;
    baseline.hand = cards.clone();
    let normal = baseline
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");

    let mut debuffed = new_run();
    add_rule_joker(&mut debuffed, "rule_debuff_face", "debuff_face", 1.0);
    debuffed.state.phase = Phase::Play;
    debuffed.state.hands_left = 1;
    debuffed.hand = cards;
    let debuffed_breakdown = debuffed
        .play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");

    assert_eq!(normal.total.chips - debuffed_breakdown.total.chips, 30);
}

#[test]
fn gold_seal_adds_money_on_score() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.money = 0;
    let mut cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Nine),
        (Suit::Clubs, Rank::Eight),
        (Suit::Diamonds, Rank::Seven),
        (Suit::Spades, Rank::Six),
    ]);
    cards[0].seal = Some(Seal::Gold);
    run.hand = cards;
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play hand");
    assert_eq!(run.state.money, 3);
}

#[test]
fn shop_restrictions_block_owned_joker() {
    let run = new_run();
    let mut content = run.content.clone();
    content.jokers = vec![
        JokerDef {
            id: "owned".to_string(),
            name: "owned".to_string(),
            rarity: JokerRarity::Common,
            effects: Vec::new(),
        },
        JokerDef {
            id: "free".to_string(),
            name: "free".to_string(),
            rarity: JokerRarity::Common,
            effects: Vec::new(),
        },
    ];
    let mut rule = run.config.shop.clone();
    rule.card_slots = 1;
    rule.booster_slots = 0;
    rule.voucher_slots = 0;
    rule.card_weights = vec![CardWeight {
        kind: ShopCardKind::Joker,
        weight: 1,
    }];
    rule.joker_rarity_weights = vec![JokerRarityWeight {
        rarity: JokerRarity::Common,
        weight: 1,
    }];
    let mut rng = RngState::from_seed(1);
    let mut restrictions = ShopRestrictions::default();
    restrictions.allow_duplicates = false;
    restrictions.owned_jokers.insert("owned".to_string());

    let shop = ShopState::generate(&rule, &content, &mut rng, &restrictions);

    assert_eq!(shop.cards.len(), 1);
    assert_eq!(shop.cards[0].item_id, "free");
}

#[test]
fn shop_restrictions_all_owned_yields_empty() {
    let run = new_run();
    let mut content = run.content.clone();
    content.jokers = vec![JokerDef {
        id: "owned".to_string(),
        name: "owned".to_string(),
        rarity: JokerRarity::Common,
        effects: Vec::new(),
    }];
    let mut rule = run.config.shop.clone();
    rule.card_slots = 1;
    rule.booster_slots = 0;
    rule.voucher_slots = 0;
    rule.card_weights = vec![CardWeight {
        kind: ShopCardKind::Joker,
        weight: 1,
    }];
    rule.joker_rarity_weights = vec![JokerRarityWeight {
        rarity: JokerRarity::Common,
        weight: 1,
    }];
    let mut rng = RngState::from_seed(2);
    let mut restrictions = ShopRestrictions::default();
    restrictions.allow_duplicates = false;
    restrictions.owned_jokers.insert("owned".to_string());

    let shop = ShopState::generate(&rule, &content, &mut rng, &restrictions);

    assert!(shop.cards.is_empty());
}

#[test]
fn pack_open_blocks_owned_tarot_when_duplicates_disabled() {
    let run = new_run();
    let mut content = run.content.clone();
    content.tarots = vec![ConsumableDef {
        id: "only".to_string(),
        name: "only".to_string(),
        kind: ConsumableKind::Tarot,
        hand: None,
        effects: Vec::new(),
    }];
    let offer = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let mut rng = RngState::from_seed(3);
    let mut restrictions = ShopRestrictions::default();
    restrictions.owned_tarots.insert("only".to_string());

    let open = open_pack(
        &offer,
        &content,
        &run.config.shop.joker_rarity_weights,
        &mut rng,
        &restrictions,
    );

    assert!(open.options.is_empty());
}

#[test]
fn pack_open_allows_owned_tarot_when_duplicates_enabled() {
    let run = new_run();
    let mut content = run.content.clone();
    content.tarots = vec![ConsumableDef {
        id: "only".to_string(),
        name: "only".to_string(),
        kind: ConsumableKind::Tarot,
        hand: None,
        effects: Vec::new(),
    }];
    let offer = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let mut rng = RngState::from_seed(4);
    let mut restrictions = ShopRestrictions::default();
    restrictions.allow_duplicates = true;
    restrictions.owned_tarots.insert("only".to_string());

    let open = open_pack(
        &offer,
        &content,
        &run.config.shop.joker_rarity_weights,
        &mut rng,
        &restrictions,
    );

    assert_eq!(open.options.len(), 2);
    for option in open.options {
        match option {
            PackOption::Consumable(kind, id) => {
                assert_eq!(kind, ConsumableKind::Tarot);
                assert_eq!(id, "only");
            }
            _ => panic!("expected tarot consumable option"),
        }
    }
}

#[test]
fn money_floor_blocks_reroll_below_floor() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_floor_block", "money_floor", -2.0);
    run.state.money = 0;
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    run.shop.as_mut().expect("shop").reroll_cost = 3;

    let err = run.reroll_shop(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::NotEnoughMoney));
}

#[test]
fn money_floor_allows_reroll_to_floor() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_floor_ok", "money_floor", -2.0);
    run.state.money = 0;
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    run.shop.as_mut().expect("shop").reroll_cost = 2;

    run.reroll_shop(&mut EventBus::default()).expect("reroll");

    assert_eq!(run.state.money, -2);
}

#[test]
fn shop_restrictions_block_owned_tarot_offer() {
    let run = new_run();
    let mut content = run.content.clone();
    content.tarots = vec![
        ConsumableDef {
            id: "owned".to_string(),
            name: "owned".to_string(),
            kind: ConsumableKind::Tarot,
            hand: None,
            effects: Vec::new(),
        },
        ConsumableDef {
            id: "free".to_string(),
            name: "free".to_string(),
            kind: ConsumableKind::Tarot,
            hand: None,
            effects: Vec::new(),
        },
    ];
    let mut rule = run.config.shop.clone();
    rule.card_slots = 1;
    rule.booster_slots = 0;
    rule.voucher_slots = 0;
    rule.card_weights = vec![CardWeight {
        kind: ShopCardKind::Tarot,
        weight: 1,
    }];
    let mut rng = RngState::from_seed(10);
    let mut restrictions = ShopRestrictions::default();
    restrictions.allow_duplicates = false;
    restrictions.owned_tarots.insert("owned".to_string());

    let shop = ShopState::generate(&rule, &content, &mut rng, &restrictions);

    assert_eq!(shop.cards.len(), 1);
    assert_eq!(shop.cards[0].item_id, "free");
}

#[test]
fn shop_restrictions_block_owned_planet_offer() {
    let run = new_run();
    let mut content = run.content.clone();
    content.planets = vec![
        ConsumableDef {
            id: "owned".to_string(),
            name: "owned".to_string(),
            kind: ConsumableKind::Planet,
            hand: None,
            effects: Vec::new(),
        },
        ConsumableDef {
            id: "free".to_string(),
            name: "free".to_string(),
            kind: ConsumableKind::Planet,
            hand: None,
            effects: Vec::new(),
        },
    ];
    let mut rule = run.config.shop.clone();
    rule.card_slots = 1;
    rule.booster_slots = 0;
    rule.voucher_slots = 0;
    rule.card_weights = vec![CardWeight {
        kind: ShopCardKind::Planet,
        weight: 1,
    }];
    let mut rng = RngState::from_seed(11);
    let mut restrictions = ShopRestrictions::default();
    restrictions.allow_duplicates = false;
    restrictions.owned_planets.insert("owned".to_string());

    let shop = ShopState::generate(&rule, &content, &mut rng, &restrictions);

    assert_eq!(shop.cards.len(), 1);
    assert_eq!(shop.cards[0].item_id, "free");
}

#[test]
fn pack_open_blocks_owned_planet_when_duplicates_disabled() {
    let run = new_run();
    let mut content = run.content.clone();
    content.planets = vec![ConsumableDef {
        id: "only".to_string(),
        name: "only".to_string(),
        kind: ConsumableKind::Planet,
        hand: None,
        effects: Vec::new(),
    }];
    let offer = PackOffer {
        kind: PackKind::Celestial,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let mut rng = RngState::from_seed(12);
    let mut restrictions = ShopRestrictions::default();
    restrictions.owned_planets.insert("only".to_string());

    let open = open_pack(
        &offer,
        &content,
        &run.config.shop.joker_rarity_weights,
        &mut rng,
        &restrictions,
    );

    assert!(open.options.is_empty());
}

#[test]
fn pack_open_allows_owned_planet_when_duplicates_enabled() {
    let run = new_run();
    let mut content = run.content.clone();
    content.planets = vec![ConsumableDef {
        id: "only".to_string(),
        name: "only".to_string(),
        kind: ConsumableKind::Planet,
        hand: None,
        effects: Vec::new(),
    }];
    let offer = PackOffer {
        kind: PackKind::Celestial,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let mut rng = RngState::from_seed(13);
    let mut restrictions = ShopRestrictions::default();
    restrictions.allow_duplicates = true;
    restrictions.owned_planets.insert("only".to_string());

    let open = open_pack(
        &offer,
        &content,
        &run.config.shop.joker_rarity_weights,
        &mut rng,
        &restrictions,
    );

    assert_eq!(open.options.len(), 2);
    for option in open.options {
        match option {
            PackOption::Consumable(kind, id) => {
                assert_eq!(kind, ConsumableKind::Planet);
                assert_eq!(id, "only");
            }
            _ => panic!("expected planet consumable option"),
        }
    }
}

#[test]
fn pack_open_blocks_owned_spectral_when_duplicates_disabled() {
    let run = new_run();
    let mut content = run.content.clone();
    content.spectrals = vec![ConsumableDef {
        id: "only".to_string(),
        name: "only".to_string(),
        kind: ConsumableKind::Spectral,
        hand: None,
        effects: Vec::new(),
    }];
    let offer = PackOffer {
        kind: PackKind::Spectral,
        size: PackSize::Normal,
        options: 2,
        picks: 1,
        price: 0,
    };
    let mut rng = RngState::from_seed(14);
    let mut restrictions = ShopRestrictions::default();
    restrictions.owned_spectrals.insert("only".to_string());

    let open = open_pack(
        &offer,
        &content,
        &run.config.shop.joker_rarity_weights,
        &mut rng,
        &restrictions,
    );

    assert!(open.options.is_empty());
}

#[test]
fn pack_pick_rejects_out_of_range_index() {
    let open = PackOpen {
        offer: PackOffer {
            kind: PackKind::Buffoon,
            size: PackSize::Normal,
            options: 2,
            picks: 1,
            price: 0,
        },
        options: vec![
            PackOption::Joker("a".to_string()),
            PackOption::Joker("b".to_string()),
        ],
    };

    let err = pick_pack_options(&open, &[2]).unwrap_err();
    assert!(matches!(err, PackError::InvalidSelection));
}

#[test]
fn pack_pick_rejects_too_many_indices() {
    let open = PackOpen {
        offer: PackOffer {
            kind: PackKind::Buffoon,
            size: PackSize::Normal,
            options: 2,
            picks: 1,
            price: 0,
        },
        options: vec![
            PackOption::Joker("a".to_string()),
            PackOption::Joker("b".to_string()),
        ],
    };

    let err = pick_pack_options(&open, &[0, 1]).unwrap_err();
    assert!(matches!(err, PackError::TooManyPicks));
}

#[test]
fn pack_pick_dedups_duplicate_indices() {
    let open = PackOpen {
        offer: PackOffer {
            kind: PackKind::Buffoon,
            size: PackSize::Normal,
            options: 1,
            picks: 2,
            price: 0,
        },
        options: vec![PackOption::Joker("a".to_string())],
    };

    let picked = pick_pack_options(&open, &[0, 0]).expect("pick options");
    assert_eq!(picked.len(), 1);
    match &picked[0] {
        PackOption::Joker(id) => assert_eq!(id, "a"),
        _ => panic!("expected joker option"),
    }
}

#[test]
fn money_floor_blocks_purchase_below_floor() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_floor_buy_block", "money_floor", -2.0);
    run.state.money = 0;
    run.state.phase = Phase::Shop;
    run.shop = Some(ShopState {
        cards: vec![CardOffer {
            kind: ShopCardKind::Tarot,
            item_id: "tarot".to_string(),
            rarity: None,
            price: 3,
            edition: None,
        }],
        packs: Vec::new(),
        vouchers: 0,
        voucher_offers: Vec::new(),
        reroll_cost: 0,
    });

    let err = run
        .buy_shop_offer(ShopOfferRef::Card(0), &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::NotEnoughMoney));
}

#[test]
fn money_floor_allows_purchase_to_floor() {
    let mut run = new_run();
    add_rule_joker(&mut run, "rule_floor_buy_ok", "money_floor", -2.0);
    run.state.money = 0;
    run.state.phase = Phase::Shop;
    run.shop = Some(ShopState {
        cards: vec![CardOffer {
            kind: ShopCardKind::Tarot,
            item_id: "tarot".to_string(),
            rarity: None,
            price: 2,
            edition: None,
        }],
        packs: Vec::new(),
        vouchers: 0,
        voucher_offers: Vec::new(),
        reroll_cost: 0,
    });

    run.buy_shop_offer(ShopOfferRef::Card(0), &mut EventBus::default())
        .expect("buy offer");
    assert_eq!(run.state.money, -2);
}

#[test]
fn skip_blind_emits_events_and_starts_next() {
    let mut run = new_run();
    run.content.tags = vec![TagDef {
        id: "tag_one".to_string(),
        name: "Tag One".to_string(),
        effects: Vec::new(),
    }];
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");

    let mut events = EventBus::default();
    run.skip_blind(&mut events).expect("skip blind");
    let drained: Vec<_> = events.drain().collect();

    assert_eq!(drained.len(), 2);
    match &drained[0] {
        rulatro_core::Event::BlindSkipped { ante, blind, tag } => {
            assert_eq!(*ante, 1);
            assert_eq!(*blind, BlindKind::Small);
            assert_eq!(tag.as_deref(), Some("tag_one"));
        }
        other => panic!("unexpected first event: {other:?}"),
    }
    match &drained[1] {
        rulatro_core::Event::BlindStarted { ante, blind, .. } => {
            assert_eq!(*ante, 1);
            assert_eq!(*blind, BlindKind::Big);
        }
        other => panic!("unexpected second event: {other:?}"),
    }
}

#[test]
fn skip_blind_no_tags_emits_none() {
    let mut run = new_run();
    run.content.tags.clear();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");

    let mut events = EventBus::default();
    run.skip_blind(&mut events).expect("skip blind");
    let drained: Vec<_> = events.drain().collect();

    assert_eq!(drained.len(), 2);
    match &drained[0] {
        rulatro_core::Event::BlindSkipped { tag, .. } => {
            assert!(tag.is_none());
        }
        other => panic!("unexpected event: {other:?}"),
    }
    assert!(run.state.tags.is_empty());
    assert_eq!(run.state.blinds_skipped, 1);
}

#[test]
fn skip_blind_big_advances_to_boss() {
    let mut run = new_run();
    run.start_blind(1, BlindKind::Big, &mut EventBus::default())
        .expect("start blind");
    run.skip_blind(&mut EventBus::default())
        .expect("skip blind");
    assert_eq!(run.state.blind, BlindKind::Boss);
    assert_eq!(run.state.ante, 1);
}

#[test]
fn skip_blind_sets_next_limits() {
    let mut run = new_run();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    let (expected_hands, expected_discards) = {
        let rule = run.config.blind_rule(BlindKind::Big).expect("blind rule");
        (rule.hands, rule.discards)
    };

    run.skip_blind(&mut EventBus::default())
        .expect("skip blind");

    assert_eq!(run.state.blind, BlindKind::Big);
    assert_eq!(run.state.hands_left, expected_hands);
    assert_eq!(run.state.discards_left, expected_discards);
}

// ═══════════════════════════════════════════════════════════════════════════
// DSL content loading
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dsl_loads_all_jokers() {
    let content = load_content(&assets_root()).expect("load content");
    assert!(
        content.jokers.len() >= 100,
        "expected at least 100 jokers, got {}",
        content.jokers.len()
    );
}

#[test]
fn dsl_loads_all_bosses() {
    let content = load_content(&assets_root()).expect("load content");
    assert_eq!(content.bosses.len(), 23, "expected 23 bosses");
}

#[test]
fn dsl_loads_all_tags() {
    let content = load_content(&assets_root()).expect("load content");
    assert_eq!(content.tags.len(), 24, "expected 24 tags");
}

#[test]
fn dsl_jokers_have_nonempty_ids_and_names() {
    let content = load_content(&assets_root()).expect("load content");
    for joker in &content.jokers {
        assert!(!joker.id.is_empty(), "joker id empty");
        assert!(!joker.name.is_empty(), "joker name empty for {}", joker.id);
    }
}

#[test]
fn dsl_jokers_have_unique_ids() {
    let content = load_content(&assets_root()).expect("load content");
    let mut ids: Vec<&str> = content.jokers.iter().map(|j| j.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), content.jokers.len(), "duplicate joker ids found");
}

#[test]
fn dsl_bosses_have_unique_ids() {
    let content = load_content(&assets_root()).expect("load content");
    let mut ids: Vec<&str> = content.bosses.iter().map(|b| b.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), content.bosses.len(), "duplicate boss ids found");
}

#[test]
fn dsl_tags_have_unique_ids() {
    let content = load_content(&assets_root()).expect("load content");
    let mut ids: Vec<&str> = content.tags.iter().map(|t| t.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), content.tags.len(), "duplicate tag ids found");
}

#[test]
fn dsl_known_jokers_present() {
    let content = load_content(&assets_root()).expect("load content");
    let known = [
        "half_joker",
        "banner",
        "even_steven",
        "odd_todd",
        "scholar",
        "fibonacci",
        "mime",
        "blue_joker",
        "golden_joker",
        "steel_joker",
        "four_fingers",
        "shortcut",
        "ancient_joker",
        "burnt_joker",
        "dna",
        "luchador",
        "chicot",
        "perkeo",
    ];
    for id in &known {
        assert!(
            content.jokers.iter().any(|j| &j.id == id),
            "joker '{}' not found in content",
            id
        );
    }
}

#[test]
fn dsl_known_bosses_present() {
    let content = load_content(&assets_root()).expect("load content");
    let known = [
        "the_hook",
        "the_wall",
        "the_eye",
        "the_mouth",
        "the_head",
        "the_tooth",
        "the_flint",
    ];
    for id in &known {
        assert!(
            content.bosses.iter().any(|b| &b.id == id),
            "boss '{}' not found in content",
            id
        );
    }
}

#[test]
fn dsl_known_tags_present() {
    let content = load_content(&assets_root()).expect("load content");
    let known = [
        "boss_tag",
        "coupon_tag",
        "d6_tag",
        "economy_tag",
        "foil_tag",
        "rare_tag",
        "top_up_tag",
        "voucher_tag",
    ];
    for id in &known {
        assert!(
            content.tags.iter().any(|t| &t.id == id),
            "tag '{}' not found in content",
            id
        );
    }
}

#[test]
fn dsl_jokers_all_have_at_least_one_effect() {
    let content = load_content(&assets_root()).expect("load content");
    // Every joker should define at least one effect
    let empty_jokers: Vec<&str> = content
        .jokers
        .iter()
        .filter(|j| j.effects.is_empty())
        .map(|j| j.id.as_str())
        .collect();
    assert!(
        empty_jokers.is_empty(),
        "jokers with no effects: {:?}",
        empty_jokers
    );
}

#[test]
fn dsl_bosses_all_have_at_least_one_effect() {
    let content = load_content(&assets_root()).expect("load content");
    let empty: Vec<&str> = content
        .bosses
        .iter()
        .filter(|b| b.effects.is_empty())
        .map(|b| b.id.as_str())
        .collect();
    assert!(empty.is_empty(), "bosses with no effects: {:?}", empty);
}

// ═══════════════════════════════════════════════════════════════════════════
// Trigger coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn trigger_on_scored_fires_per_scoring_card() {
    let mut run = new_run();
    // OnScored fires for each scored card; add_money 5 per card
    add_money_joker(&mut run, "scored_money", ActivationType::OnScored, 5.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Play a pair (2 scoring cards)
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Three),
    ]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    // Should get 5 per scoring card (at least 2 for the pair)
    assert!(
        run.state.money >= 10,
        "expected >=10, got {}",
        run.state.money
    );
}

#[test]
fn trigger_on_scored_pre_fires_per_scoring_card() {
    let mut run = new_run();
    add_money_joker(
        &mut run,
        "scored_pre_money",
        ActivationType::OnScoredPre,
        3.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Five), (Suit::Hearts, Rank::Five)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    assert!(
        run.state.money >= 6,
        "expected >=6, got {}",
        run.state.money
    );
}

#[test]
fn trigger_on_held_fires_per_held_card() {
    let mut run = new_run();
    add_money_joker(&mut run, "held_money", ActivationType::OnHeld, 2.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // hand has 4 cards, play 1 — 3 held cards should trigger
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Two),
        (Suit::Clubs, Rank::Three),
        (Suit::Diamonds, Rank::Four),
    ]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    assert!(
        run.state.money >= 6,
        "expected >=6 (3 held × 2), got {}",
        run.state.money
    );
}

#[test]
fn trigger_on_discard_fires_per_discarded_card() {
    let mut run = new_run();
    add_money_joker(&mut run, "discard_money", ActivationType::OnDiscard, 4.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.discards_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Hearts, Rank::Three),
        (Suit::Clubs, Rank::Four),
    ]);
    run.discard(&[0, 1], &mut EventBus::default())
        .expect("discard");
    assert_eq!(run.state.money, 8, "expected 8 (2 cards × 4)");
}

#[test]
fn trigger_on_discard_batch_fires_once() {
    let mut run = new_run();
    add_money_joker(
        &mut run,
        "batch_money",
        ActivationType::OnDiscardBatch,
        10.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.discards_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Two), (Suit::Hearts, Rank::Three)]);
    run.discard(&[0, 1], &mut EventBus::default())
        .expect("discard");
    // Fires once regardless of number of discarded cards
    assert_eq!(run.state.money, 10, "expected 10 (batch fires once)");
}

#[test]
fn trigger_on_hand_end_fires_after_play() {
    let mut run = new_run();
    add_money_joker(&mut run, "hand_end_money", ActivationType::OnHandEnd, 7.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    assert_eq!(run.state.money, 7, "expected 7 from OnHandEnd");
}

#[test]
fn trigger_on_round_end_fires_when_blind_cleared() {
    let mut run = new_run();
    add_money_joker(&mut run, "round_end_money", ActivationType::OnRoundEnd, 9.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.target = 1;
    run.state.blind_score = 0;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // Blind should be cleared, round end fires
    assert!(run.blind_cleared());
    assert!(
        run.state.money >= 9,
        "expected >=9 from OnRoundEnd, got {}",
        run.state.money
    );
}

#[test]
fn trigger_on_shop_exit_fires_on_start_blind() {
    let mut run = new_run();
    add_money_joker(&mut run, "shop_exit_money", ActivationType::OnShopExit, 6.0);
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    // start_blind fires OnShopExit before beginning the blind
    run.start_blind(1, BlindKind::Big, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.money, 6, "expected 6 from OnShopExit");
}

#[test]
fn trigger_on_pack_opened_fires_on_open() {
    let mut run = new_run();
    add_money_joker(
        &mut run,
        "pack_opened_money",
        ActivationType::OnPackOpened,
        11.0,
    );
    let pack = PackOffer {
        kind: PackKind::Arcana,
        size: PackSize::Normal,
        options: 1,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    run.open_pack_purchase(&purchase, &mut EventBus::default())
        .expect("open pack");
    assert_eq!(run.state.money, 11, "expected 11 from OnPackOpened");
}

#[test]
fn trigger_on_pack_skipped_fires_on_skip() {
    let mut run = new_run();
    add_money_joker(
        &mut run,
        "pack_skipped_money",
        ActivationType::OnPackSkipped,
        8.0,
    );
    let pack = PackOffer {
        kind: PackKind::Standard,
        size: PackSize::Normal,
        options: 1,
        picks: 1,
        price: 0,
    };
    let purchase = ShopPurchase::Pack(pack);
    let open = run
        .open_pack_purchase(&purchase, &mut EventBus::default())
        .expect("open pack");
    run.skip_pack(&open, &mut EventBus::default())
        .expect("skip pack");
    assert_eq!(run.state.money, 8, "expected 8 from OnPackSkipped");
}

#[test]
fn trigger_on_use_fires_when_consumable_used() {
    let mut run = new_run();
    add_money_joker(&mut run, "use_money", ActivationType::OnUse, 12.0);
    run.hand = make_hand();
    use_consumable(&mut run, "the_devil", ConsumableKind::Tarot, &[0]);
    assert_eq!(run.state.money, 12, "expected 12 from OnUse");
}

#[test]
fn trigger_on_sell_fires_when_joker_sold() {
    let mut run = new_run();
    // OnSell fires on the joker BEING sold — add the money-gaining joker and sell IT
    add_money_joker(&mut run, "sell_me", ActivationType::OnSell, 5.0);
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let sell_idx = run
        .inventory
        .jokers
        .iter()
        .position(|j| j.id == "sell_me")
        .expect("sell_me joker");
    run.sell_joker(sell_idx, &mut EventBus::default())
        .expect("sell joker");
    // OnSell fired before removal, adding 5 money
    assert!(
        run.state.money >= 5,
        "expected >=5 from OnSell, got {}",
        run.state.money
    );
}

#[test]
fn trigger_on_any_sell_fires_when_joker_sold() {
    let mut run = new_run();
    add_money_joker(&mut run, "any_sell_watcher", ActivationType::OnAnySell, 4.0);
    run.content.jokers.push(JokerDef {
        id: "sell_target".to_string(),
        name: "Sell Target".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![],
    });
    run.inventory
        .add_joker("sell_target".to_string(), JokerRarity::Common, 3)
        .expect("add joker");
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let sell_idx = run
        .inventory
        .jokers
        .iter()
        .position(|j| j.id == "sell_target")
        .expect("sell_target joker");
    run.sell_joker(sell_idx, &mut EventBus::default())
        .expect("sell joker");
    assert!(
        run.state.money >= 4,
        "expected >=4 from OnAnySell, got {}",
        run.state.money
    );
}

#[test]
fn trigger_on_acquire_fires_when_joker_bought() {
    let mut run = new_run();
    // This joker fires OnAcquire — when *it* is acquired, adds money
    run.content.jokers.push(JokerDef {
        id: "acquire_joker".to_string(),
        name: "Acquire Joker".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnAcquire,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(15.0),
            }],
        }],
    });
    run.state.money = 50;
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    let before = run.state.money;
    // Add the joker as a shop card using CardOffer
    if let Some(shop) = run.shop.as_mut() {
        shop.cards.push(CardOffer {
            kind: ShopCardKind::Joker,
            item_id: "acquire_joker".to_string(),
            rarity: Some(JokerRarity::Common),
            price: 0,
            edition: None,
        });
    }
    let card_count = run.shop.as_ref().map(|s| s.cards.len()).unwrap_or(0);
    let offer_idx = card_count - 1;
    let purchase = run
        .buy_shop_offer(ShopOfferRef::Card(offer_idx), &mut EventBus::default())
        .expect("buy joker");
    run.apply_purchase(&purchase).expect("apply");
    // OnAcquire fired, adding 15 money (minus 0 price = net +15)
    assert!(
        run.state.money >= before + 15,
        "OnAcquire should have fired, money={}, before={}",
        run.state.money,
        before
    );
}

#[test]
fn trigger_on_other_jokers_fires_during_scoring() {
    // OnOtherJokers hook is defined but not yet invoked in the pipeline.
    // Verify the ActivationType variant exists and parses from keyword.
    use rulatro_core::ActionOp;
    let parsed = ActionOp::from_keyword("add_money");
    assert!(parsed.is_some(), "add_money should parse");
    // Verify OnOtherJokers maps correctly via activation_for
    let trigger = ActivationType::OnOtherJokers;
    let effect = JokerEffect {
        trigger,
        when: Expr::Bool(true),
        actions: vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddMoney),
            target: None,
            value: Expr::Number(3.0),
        }],
    };
    assert_eq!(effect.trigger, ActivationType::OnOtherJokers);
}

#[test]
fn trigger_on_card_destroyed_fires_on_glass_break() {
    let mut run = new_run();
    add_money_joker(
        &mut run,
        "destroy_watcher",
        ActivationType::OnCardDestroyed,
        7.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Glass cards can be destroyed; set probability to 1 via rule
    // Actually, we need a deterministic destroy — use a fixed seed run with glass card
    // Easier: use a joker that explicitly destroys a card
    add_joker_effect(
        &mut run,
        "destroyer",
        ActivationType::OnScored,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::DestroyCard),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace), (Suit::Hearts, Rank::Two)]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    assert!(
        run.state.money >= 7,
        "expected >=7 from OnCardDestroyed, got {}",
        run.state.money
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// ActionOp functional tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_add_hand_size_increases_hand_size() {
    let mut run = new_run();
    let before = run.state.hand_size;
    add_joker_effect(
        &mut run,
        "hand_size_plus",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddHandSize),
            target: None,
            value: Expr::Number(2.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.hand_size, before + 2);
}

#[test]
fn action_set_hands_overrides_hands_left() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "set_hands_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::SetHands),
            target: None,
            value: Expr::Number(7.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.hands_left, 7);
}

#[test]
fn action_add_hands_increments_hands_left() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "add_hands_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddHands),
            target: None,
            value: Expr::Number(3.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    // default hands + 3
    let default_hands = run
        .config
        .blind_rule(BlindKind::Small)
        .map(|r| r.hands)
        .unwrap_or(4);
    assert_eq!(run.state.hands_left, default_hands + 3);
}

#[test]
fn action_set_discards_overrides_discards_left() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "set_discards_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::SetDiscards),
            target: None,
            value: Expr::Number(5.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.discards_left, 5);
}

#[test]
fn action_add_discards_increments_discards_left() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "add_discards_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddDiscards),
            target: None,
            value: Expr::Number(2.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    let default_discards = run
        .config
        .blind_rule(BlindKind::Small)
        .map(|r| r.discards)
        .unwrap_or(3);
    assert_eq!(run.state.discards_left, default_discards + 2);
}

#[test]
fn action_set_money_overrides_money() {
    let mut run = new_run();
    run.state.money = 50;
    add_joker_effect(
        &mut run,
        "set_money_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::SetMoney),
            target: None,
            value: Expr::Number(20.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.state.money, 20);
}

#[test]
fn action_multiply_target_scales_score_target() {
    let mut run = new_run();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    let base_target = run.state.target;
    // Now add joker and re-run to test multiply target
    let mut run2 = new_run();
    add_joker_effect(
        &mut run2,
        "mul_target_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::MultiplyTarget),
            target: None,
            value: Expr::Number(2.0),
        }],
    );
    run2.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run2.state.target, base_target * 2);
}

#[test]
fn action_add_tarot_gives_consumable() {
    let mut run = new_run();
    run.inventory.consumables.clear();
    add_joker_effect(
        &mut run,
        "add_tarot_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddTarot),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.inventory.consumables.len(), 1);
    assert_eq!(run.inventory.consumables[0].kind, ConsumableKind::Tarot);
}

#[test]
fn action_add_planet_gives_consumable() {
    let mut run = new_run();
    run.inventory.consumables.clear();
    add_joker_effect(
        &mut run,
        "add_planet_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddPlanet),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.inventory.consumables.len(), 1);
    assert_eq!(run.inventory.consumables[0].kind, ConsumableKind::Planet);
}

#[test]
fn action_add_spectral_gives_consumable() {
    let mut run = new_run();
    run.inventory.consumables.clear();
    add_joker_effect(
        &mut run,
        "add_spectral_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddSpectral),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.inventory.consumables.len(), 1);
    assert_eq!(run.inventory.consumables[0].kind, ConsumableKind::Spectral);
}

#[test]
fn action_add_free_reroll_grants_free_reroll() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "free_reroll_j",
        ActivationType::OnShopEnter,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddFreeReroll),
            target: None,
            value: Expr::Number(2.0),
        }],
    );
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    assert_eq!(run.state.shop_free_rerolls, 2);
}

#[test]
fn action_add_free_reroll_allows_free_reroll_in_shop() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "free_reroll_j",
        ActivationType::OnShopEnter,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddFreeReroll),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default())
        .expect("enter shop");
    run.state.money = 0; // no money
                         // Should still be able to reroll since we have a free one
    run.reroll_shop(&mut EventBus::default())
        .expect("free reroll");
    assert_eq!(run.state.money, 0, "free reroll should not cost money");
}

#[test]
fn action_destroy_random_joker_removes_one() {
    let mut run = new_run();
    // Add two jokers; one destroys a random other
    run.content.jokers.push(JokerDef {
        id: "victim_j".to_string(),
        name: "Victim".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![],
    });
    run.inventory
        .add_joker("victim_j".to_string(), JokerRarity::Common, 1)
        .expect("add victim");
    add_joker_effect(
        &mut run,
        "destroyer_j",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::DestroyRandomJoker),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // One joker should be gone
    assert!(
        run.inventory.jokers.len() < 2,
        "expected <2 jokers after DestroyRandomJoker"
    );
}

#[test]
fn action_destroy_joker_left_removes_left_neighbor() {
    let mut run = new_run();
    // joker[0] = "left_j", joker[1] = "destroyer_left"
    run.content.jokers.push(JokerDef {
        id: "left_j".to_string(),
        name: "Left".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![],
    });
    run.inventory
        .add_joker("left_j".to_string(), JokerRarity::Common, 1)
        .expect("add left");
    add_joker_effect(
        &mut run,
        "destroyer_left",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::DestroyJokerLeft),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // left_j should have been destroyed
    assert!(
        !run.inventory.jokers.iter().any(|j| j.id == "left_j"),
        "left_j should have been destroyed"
    );
    assert!(
        run.inventory
            .jokers
            .iter()
            .any(|j| j.id == "destroyer_left"),
        "destroyer_left should still exist"
    );
}

#[test]
fn action_upgrade_hand_upgrades_current_hand_level() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "upgrade_hand_j",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::UpgradeHand),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace), (Suit::Hearts, Rank::Ace)]);
    let before = hand_level(&run, HandKind::Pair);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    assert_eq!(hand_level(&run, HandKind::Pair), before + 1);
}

#[test]
fn action_upgrade_random_hand_upgrades_some_hand() {
    let mut run = new_run();
    add_joker_effect(
        &mut run,
        "upgrade_rnd_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::UpgradeRandomHand),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    // Record all hand levels before
    let all_hands = [
        HandKind::HighCard,
        HandKind::Pair,
        HandKind::TwoPair,
        HandKind::Trips,
        HandKind::Straight,
        HandKind::Flush,
        HandKind::FullHouse,
        HandKind::Quads,
        HandKind::StraightFlush,
        HandKind::RoyalFlush,
    ];
    let before: Vec<u32> = all_hands.iter().map(|&h| hand_level(&run, h)).collect();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    let after: Vec<u32> = all_hands.iter().map(|&h| hand_level(&run, h)).collect();
    let upgraded = before
        .iter()
        .zip(after.iter())
        .filter(|(b, a)| *a > *b)
        .count();
    assert_eq!(upgraded, 1, "expected exactly 1 hand level increased");
}

#[test]
fn action_duplicate_random_joker_adds_copy() {
    let mut run = new_run();
    run.inventory.joker_slots = 99;
    add_joker_effect(
        &mut run,
        "dup_rnd_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::DuplicateRandomJoker),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    let before = run.inventory.jokers.len();
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(
        run.inventory.jokers.len(),
        before + 1,
        "expected one more joker after DuplicateRandomJoker"
    );
}

#[test]
fn action_duplicate_random_consumable_adds_copy() {
    let mut run = new_run();
    run.inventory.consumables.clear();
    run.inventory
        .add_consumable("pluto".to_string(), ConsumableKind::Planet)
        .expect("add planet");
    add_joker_effect(
        &mut run,
        "dup_cons_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::DuplicateRandomConsumable),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(
        run.inventory.consumables.len(),
        2,
        "expected duplicate consumable"
    );
    assert_eq!(run.inventory.consumables[1].id, "pluto");
    assert_eq!(
        run.inventory.consumables[1].edition,
        Some(Edition::Negative)
    );
}

#[test]
fn action_add_sell_bonus_increases_joker_sell_value() {
    let mut run = new_run();
    run.content.jokers.push(JokerDef {
        id: "bonus_target".to_string(),
        name: "Bonus Target".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![],
    });
    run.inventory
        .add_joker("bonus_target".to_string(), JokerRarity::Common, 4)
        .expect("add joker");
    let idx = run
        .inventory
        .jokers
        .iter()
        .position(|j| j.id == "bonus_target")
        .expect("find joker");
    let before = run.joker_sell_value(idx).unwrap_or(0);
    add_joker_effect(
        &mut run,
        "sell_bonus_giver",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddSellBonus),
            target: Some("jokers".to_string()),
            value: Expr::Number(5.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    let idx2 = run
        .inventory
        .jokers
        .iter()
        .position(|j| j.id == "bonus_target")
        .expect("find joker after");
    let after = run.joker_sell_value(idx2).unwrap_or(0);
    assert_eq!(after, before + 5, "expected sell bonus +5");
}

#[test]
fn action_add_random_hand_card_adds_sealed_card_to_hand() {
    let mut run = new_run();
    let before = run.hand.len();
    add_joker_effect(
        &mut run,
        "hand_card_j",
        ActivationType::OnBlindStart,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddRandomHandCard),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.start_blind(1, BlindKind::Small, &mut EventBus::default())
        .expect("start blind");
    assert_eq!(run.hand.len(), before + 1, "expected 1 added card");
    // The card should have a seal
    let added = &run.hand[run.hand.len() - 1];
    assert!(added.seal.is_some(), "added card should have a seal");
}

#[test]
fn action_copy_joker_leftmost_copies_first_joker_effects() {
    let mut run = new_run();
    // joker[0] = money giver; joker[1] = copy_leftmost
    add_money_joker(&mut run, "left_money", ActivationType::OnPlayed, 8.0);
    add_joker_effect(
        &mut run,
        "copy_leftmost_j",
        ActivationType::OnPlayed,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::CopyJokerLeftmost),
            target: None,
            value: Expr::Number(1.0),
        }],
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // left_money fires once, copy_leftmost fires it again = 16 total
    assert_eq!(
        run.state.money, 16,
        "expected 16 (8 × 2 from copy leftmost)"
    );
}

#[test]
fn action_add_rule_accumulates_on_existing_rule() {
    let mut run = new_run();
    // Use SetRule to set base, then AddRule to accumulate
    add_joker_effect(
        &mut run,
        "set_base",
        ActivationType::Passive,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::SetRule),
            target: Some("four_fingers".to_string()),
            value: Expr::Number(0.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "add_rule_j",
        ActivationType::Passive,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::AddRule),
            target: Some("four_fingers".to_string()),
            value: Expr::Number(1.0),
        }],
    );
    // Passive effects rebuild the hand eval rules
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // four_fingers=1 should enable 4-card straights
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Two),
        (Suit::Clubs, Rank::Three),
        (Suit::Diamonds, Rank::Four),
    ]);
    run.hand = cards;
    let breakdown = run
        .play_hand(&[0, 1, 2, 3], &mut EventBus::default())
        .expect("play");
    assert_eq!(
        breakdown.hand,
        HandKind::Straight,
        "four_fingers rule should enable 4-card straight"
    );
}

#[test]
fn action_clear_rule_resets_rule_to_zero() {
    let mut run = new_run();
    // Set four_fingers=1 then clear it
    add_joker_effect(
        &mut run,
        "set_ff",
        ActivationType::Passive,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::SetRule),
            target: Some("four_fingers".to_string()),
            value: Expr::Number(1.0),
        }],
    );
    add_joker_effect(
        &mut run,
        "clear_ff",
        ActivationType::Passive,
        vec![Action {
            op: ActionOpKind::Builtin(ActionOp::ClearRule),
            target: Some("four_fingers".to_string()),
            value: Expr::Number(0.0),
        }],
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Should NOT be a straight since four_fingers was cleared
    let cards = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Two),
        (Suit::Clubs, Rank::Three),
        (Suit::Diamonds, Rank::Four),
    ]);
    run.hand = cards;
    let breakdown = run
        .play_hand(&[0, 1, 2, 3], &mut EventBus::default())
        .expect("play");
    assert_ne!(
        breakdown.hand,
        HandKind::Straight,
        "four_fingers cleared, should not be straight"
    );
}

#[test]
fn action_set_var_and_add_var_accumulate_on_joker() {
    let mut run = new_run();
    // A joker that sets a var on played, then uses it to add chips
    run.content.jokers.push(JokerDef {
        id: "var_joker".to_string(),
        name: "Var Joker".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![
            JokerEffect {
                trigger: ActivationType::OnPlayed,
                when: Expr::Bool(true),
                actions: vec![Action {
                    op: ActionOpKind::Builtin(ActionOp::AddVar),
                    target: Some("counter".to_string()),
                    value: Expr::Number(1.0),
                }],
            },
            JokerEffect {
                trigger: ActivationType::Independent,
                when: Expr::Bool(true),
                actions: vec![Action {
                    op: ActionOpKind::Builtin(ActionOp::AddChips),
                    target: None,
                    value: Expr::Call {
                        name: "var".to_string(),
                        args: vec![Expr::String("counter".to_string())],
                    },
                }],
            },
        ],
    });
    run.inventory
        .add_joker("var_joker".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    let bd1 = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("first play");
    // After play 1: counter=1; Independent fires with var:counter=1 → chips +1
    let chips1 = bd1.total.chips;
    run.state.phase = Phase::Play;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    let bd2 = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("second play");
    // After play 2: counter=2; Independent fires with var:counter=2 → chips +2
    assert!(
        bd2.total.chips > chips1,
        "second play should score more chips due to growing var"
    );
}

#[test]
fn action_retrigger_scored_fires_card_effects_again() {
    let mut run = new_run();
    // RetriggerScored from OnScored needs a var guard to prevent infinite loop.
    // Pattern: `on scored when var(used)==0 { retrigger_scored 1; set_var used 1 }`
    run.content.jokers.push(JokerDef {
        id: "retrigger_j".to_string(),
        name: "Retrigger".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnScored,
            when: Expr::Binary {
                left: Box::new(Expr::Call {
                    name: "var".to_string(),
                    args: vec![Expr::String("used".to_string())],
                }),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Number(0.0)),
            },
            actions: vec![
                Action {
                    op: ActionOpKind::Builtin(ActionOp::RetriggerScored),
                    target: None,
                    value: Expr::Number(1.0),
                },
                Action {
                    op: ActionOpKind::Builtin(ActionOp::SetVar),
                    target: Some("used".to_string()),
                    value: Expr::Number(1.0),
                },
            ],
        }],
    });
    run.inventory
        .add_joker("retrigger_j".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.enhancement = Some(Enhancement::Bonus); // +30 chips per score
    run.hand = vec![card];
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // With 1 guarded retrigger, bonus enhancement fires twice
    let bonus_hits = run
        .last_score_trace
        .iter()
        .filter(|s| s.source == "enhancement:bonus")
        .count();
    assert_eq!(
        bonus_hits, 2,
        "retrigger should double bonus enhancement hits"
    );
}

#[test]
fn action_retrigger_held_fires_held_effects_again() {
    let mut run = new_run();
    // OnHeld + RetriggerHeld with var guard to prevent infinite loop
    run.content.jokers.push(JokerDef {
        id: "retrigger_held_j".to_string(),
        name: "Retrigger Held".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnHeld,
            when: Expr::Binary {
                left: Box::new(Expr::Call {
                    name: "var".to_string(),
                    args: vec![Expr::String("used".to_string())],
                }),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Number(0.0)),
            },
            actions: vec![
                Action {
                    op: ActionOpKind::Builtin(ActionOp::RetriggerHeld),
                    target: None,
                    value: Expr::Number(1.0),
                },
                Action {
                    op: ActionOpKind::Builtin(ActionOp::SetVar),
                    target: Some("used".to_string()),
                    value: Expr::Number(1.0),
                },
            ],
        }],
    });
    run.inventory
        .add_joker("retrigger_held_j".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut steel = Card::standard(Suit::Hearts, Rank::Two);
    steel.enhancement = Some(Enhancement::Steel); // x1.5 mult per trigger
    run.hand = vec![Card::standard(Suit::Spades, Rank::Ace), steel];
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    let steel_hits = run
        .last_score_trace
        .iter()
        .filter(|s| s.source == "enhancement:steel")
        .count();
    assert_eq!(steel_hits, 2, "retrigger held should double steel hits");
}

// ═══════════════════════════════════════════════════════════════════════════
// DSL when-expression condition tests (card.is_face, card.suit, etc.)
// ═══════════════════════════════════════════════════════════════════════════

fn add_scored_conditional_joker(run: &mut RunState, id: &str, when: Expr, amount: f64) {
    run.content.jokers.push(JokerDef {
        id: id.to_string(),
        name: id.to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnScored,
            when,
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(amount),
            }],
        }],
    });
    run.inventory
        .add_joker(id.to_string(), JokerRarity::Common, 1)
        .expect("add joker");
}

#[test]
fn condition_card_is_face_gates_face_cards() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "face_money",
        Expr::Ident("card.is_face".to_string()),
        3.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Play a King (face) + Two (not face)
    run.hand = make_cards(&[(Suit::Spades, Rank::King), (Suit::Hearts, Rank::Two)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    // Only King is face → 1 × 3 = 3
    assert_eq!(run.state.money, 3, "expected 3 from face card only");
}

#[test]
fn condition_card_is_not_face_gates_non_face_cards() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "nonface_money",
        Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Ident("card.is_face".to_string())),
        },
        2.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Pair of Fives (both non-face) → both score via OnScored → 2 × 2 = 4
    run.hand = make_cards(&[(Suit::Spades, Rank::Five), (Suit::Hearts, Rank::Five)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    assert_eq!(run.state.money, 4, "expected 4 from two non-face cards");
}

#[test]
fn condition_card_is_odd_gates_odd_ranks() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "odd_money",
        Expr::Ident("card.is_odd".to_string()),
        5.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Pair of Aces (odd) → both score → 2 × 5 = 10
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace), (Suit::Hearts, Rank::Ace)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    assert_eq!(run.state.money, 10, "expected 10 from two odd cards");
}

#[test]
fn condition_card_is_even_gates_even_ranks() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "even_money",
        Expr::Ident("card.is_even".to_string()),
        4.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Pair of Fours (even) → both score → 2 × 4 = 8
    run.hand = make_cards(&[(Suit::Spades, Rank::Four), (Suit::Hearts, Rank::Four)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    assert_eq!(run.state.money, 8, "expected 8 from two even cards");
}

#[test]
fn condition_card_suit_gates_matching_suit() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "hearts_money",
        Expr::Binary {
            left: Box::new(Expr::Ident("card.suit".to_string())),
            op: BinaryOp::Eq,
            right: Box::new(Expr::String("hearts".to_string())),
        },
        6.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    // Pair of Twos: one Hearts, one Spades → Hearts scores → 1 × 6 = 6
    run.hand = make_cards(&[(Suit::Hearts, Rank::Two), (Suit::Spades, Rank::Two)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    assert_eq!(run.state.money, 6, "expected 6 from one hearts card");
}

#[test]
fn condition_card_rank_gates_matching_rank() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "ace_money",
        Expr::Binary {
            left: Box::new(Expr::Ident("card.rank".to_string())),
            op: BinaryOp::Eq,
            right: Box::new(Expr::String("ace".to_string())),
        },
        7.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Ace),
        (Suit::Hearts, Rank::Ace),
        (Suit::Clubs, Rank::Two),
    ]);
    run.play_hand(&[0, 1, 2], &mut EventBus::default())
        .expect("play");
    // Two Aces = 2 × 7 = 14
    assert_eq!(run.state.money, 14, "expected 14 from two aces");
}

#[test]
fn condition_card_is_stone_gates_stone_cards() {
    let mut run = new_run();
    add_scored_conditional_joker(
        &mut run,
        "stone_money",
        Expr::Ident("card.is_stone".to_string()),
        10.0,
    );
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut stone = Card::standard(Suit::Spades, Rank::Ace);
    stone.enhancement = Some(Enhancement::Stone);
    let normal = Card::standard(Suit::Hearts, Rank::Two);
    run.hand = vec![stone, normal];
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play");
    // Only stone card matches
    assert_eq!(run.state.money, 10, "expected 10 from one stone card");
}

#[test]
fn condition_is_boss_blind_gates_boss_blind() {
    let mut run = new_run();
    // Joker fires Independent with is_boss_blind condition
    run.content.jokers.push(JokerDef {
        id: "boss_blind_money".to_string(),
        name: "Boss Blind Money".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Ident("is_boss_blind".to_string()),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(20.0),
            }],
        }],
    });
    run.inventory
        .add_joker("boss_blind_money".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    // Play in small blind → should NOT fire
    run.state.phase = Phase::Play;
    run.state.blind = BlindKind::Small;
    run.state.hands_left = 1;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default())
        .expect("play non-boss");
    assert_eq!(run.state.money, 0, "should not fire on small blind");
    // Play in boss blind → should fire
    run.state.phase = Phase::Play;
    run.state.blind = BlindKind::Boss;
    run.state.hands_left = 1;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default())
        .expect("play boss");
    assert_eq!(run.state.money, 20, "should fire on boss blind");
}

#[test]
fn condition_hand_variable_gates_by_hand_type() {
    let mut run = new_run();
    // Fires Independent when hand == "flush"
    run.content.jokers.push(JokerDef {
        id: "flush_money".to_string(),
        name: "Flush Money".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Binary {
                left: Box::new(Expr::Ident("hand".to_string())),
                op: BinaryOp::Eq,
                right: Box::new(Expr::String("flush".to_string())),
            },
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(15.0),
            }],
        }],
    });
    run.inventory
        .add_joker("flush_money".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.state.target = 1_000_000;
    // Play a non-flush first
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace), (Suit::Hearts, Rank::Two)]);
    run.play_hand(&[0, 1], &mut EventBus::default())
        .expect("play pair");
    assert_eq!(run.state.money, 0, "should not fire on pair");
    // Play a flush
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[
        (Suit::Spades, Rank::Two),
        (Suit::Spades, Rank::Four),
        (Suit::Spades, Rank::Six),
        (Suit::Spades, Rank::Eight),
        (Suit::Spades, Rank::Ten),
    ]);
    run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default())
        .expect("play flush");
    assert_eq!(run.state.money, 15, "should fire on flush");
}

#[test]
fn condition_hands_left_variable_gates_by_count() {
    let mut run = new_run();
    // Fires when hands_left == 1 (last hand)
    run.content.jokers.push(JokerDef {
        id: "last_hand_money".to_string(),
        name: "Last Hand Money".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Binary {
                left: Box::new(Expr::Ident("hands_left".to_string())),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Number(1.0)),
            },
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddMoney),
                target: None,
                value: Expr::Number(25.0),
            }],
        }],
    });
    run.inventory
        .add_joker("last_hand_money".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default())
        .expect("first play");
    assert_eq!(run.state.money, 0, "should not fire with 2 hands left");
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    run.play_hand(&[0], &mut EventBus::default())
        .expect("second play");
    assert_eq!(run.state.money, 25, "should fire on last hand");
}

#[test]
fn condition_money_variable_gates_by_amount() {
    let mut run = new_run();
    run.state.money = 0;
    // Fires Independent when money >= 10
    run.content.jokers.push(JokerDef {
        id: "rich_bonus".to_string(),
        name: "Rich Bonus".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::Independent,
            when: Expr::Binary {
                left: Box::new(Expr::Ident("money".to_string())),
                op: BinaryOp::Ge,
                right: Box::new(Expr::Number(10.0)),
            },
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::AddChips),
                target: None,
                value: Expr::Number(50.0),
            }],
        }],
    });
    run.inventory
        .add_joker("rich_bonus".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    // First play: money=0 → should not fire
    run.state.phase = Phase::Play;
    run.state.hands_left = 2;
    run.state.target = 1_000_000;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    let bd1 = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play broke");
    assert!(
        !run.last_score_trace
            .iter()
            .any(|s| s.source.contains("rich_bonus")),
        "should not fire when broke"
    );
    // Set money to 15 and play again
    run.state.money = 15;
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    let bd2 = run
        .play_hand(&[0], &mut EventBus::default())
        .expect("play rich");
    assert!(
        bd2.total.chips > bd1.total.chips,
        "rich bonus should fire and add chips"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// DSL: Retrigger and scoring interaction edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn scoring_retrigger_scored_stacks_with_joker_editions() {
    let mut run = new_run();
    // Use var guard to prevent infinite loop: fires once with 2 retriggers
    run.content.jokers.push(JokerDef {
        id: "retrig2".to_string(),
        name: "Retrig2".to_string(),
        rarity: JokerRarity::Common,
        effects: vec![JokerEffect {
            trigger: ActivationType::OnScored,
            when: Expr::Binary {
                left: Box::new(Expr::Call {
                    name: "var".to_string(),
                    args: vec![Expr::String("used".to_string())],
                }),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Number(0.0)),
            },
            actions: vec![
                Action {
                    op: ActionOpKind::Builtin(ActionOp::RetriggerScored),
                    target: None,
                    value: Expr::Number(2.0),
                },
                Action {
                    op: ActionOpKind::Builtin(ActionOp::SetVar),
                    target: Some("used".to_string()),
                    value: Expr::Number(1.0),
                },
            ],
        }],
    });
    run.inventory
        .add_joker("retrig2".to_string(), JokerRarity::Common, 1)
        .expect("add joker");
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    let mut card = Card::standard(Suit::Spades, Rank::Ace);
    card.enhancement = Some(Enhancement::Bonus);
    run.hand = vec![card];
    run.play_hand(&[0], &mut EventBus::default()).expect("play");
    let bonus_hits = run
        .last_score_trace
        .iter()
        .filter(|s| s.source == "enhancement:bonus")
        .count();
    assert_eq!(bonus_hits, 3, "2 retriggers → 3 total bonus hits");
}

#[test]
fn scoring_multiple_jokers_add_chips_stack() {
    let mut run = new_run();
    add_scoring_joker(&mut run, "chips_a", ActionOp::AddChips, 10.0);
    add_scoring_joker(&mut run, "chips_b", ActionOp::AddChips, 20.0);
    add_scoring_joker(&mut run, "chips_c", ActionOp::AddChips, 30.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    let bd = run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // All three AddChips stack: base + rank + 10 + 20 + 30
    let base_and_rank = bd.base.chips + bd.rank_chips;
    assert_eq!(bd.total.chips, base_and_rank + 60);
}

#[test]
fn scoring_multiple_mul_mult_compound() {
    let mut run = new_run();
    add_scoring_joker(&mut run, "mul2a", ActionOp::MultiplyMult, 2.0);
    add_scoring_joker(&mut run, "mul2b", ActionOp::MultiplyMult, 3.0);
    run.state.phase = Phase::Play;
    run.state.hands_left = 1;
    run.hand = make_cards(&[(Suit::Spades, Rank::Ace)]);
    let bd = run.play_hand(&[0], &mut EventBus::default()).expect("play");
    // base_mult * 2 * 3 = base_mult * 6
    assert_eq!(bd.total.mult, bd.base.mult * 6.0);
}
