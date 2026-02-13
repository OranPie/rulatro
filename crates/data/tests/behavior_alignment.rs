use rulatro_core::{
    Action, ActionOp, ActivationType, BlindKind, BossDef, Card, CardWeight, ConsumableKind,
    Enhancement, Edition, Seal, EventBus, Expr, HandKind, JokerDef, JokerEffect, JokerRarity,
    LastConsumable, PackKind, PackOffer, PackSize, Phase, Rank, RunError, RunState, ScoreTables,
    ShopCardKind, ShopOfferRef, ShopPurchase, Suit, evaluate_hand, evaluate_hand_with_rules,
    scoring_cards, score_hand, HandEvalRules,
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
                op: ActionOp::SetRule,
                target: Some(key.to_string()),
                value: Expr::Number(value),
            }],
        }],
    });
    run.inventory
        .add_joker(id.to_string(), JokerRarity::Common, 1)
        .expect("add joker");
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
                op,
                target: None,
                value: Expr::Number(value),
            }],
        }],
    });
    run.inventory
        .add_joker(id.to_string(), JokerRarity::Common, 1)
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
        "pluto",
        "mercury",
        "uranus",
        "venus",
        "saturn",
        "jupiter",
        "earth",
        "mars",
        "neptune",
        "planet_x",
        "ceres",
        "eris",
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
    assert_eq!(
        breakdown.total.chips,
        breakdown.base.chips + 50
    );
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
    use_consumable(
        &mut run,
        "the_high_priestess",
        ConsumableKind::Tarot,
        &[],
    );
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
    assert!(run
        .hand
        .iter()
        .filter(|card| card.enhancement.is_some())
        .count()
        >= 2);

    run.hand = make_hand();
    let before = run.hand.len();
    use_consumable(&mut run, "familiar", ConsumableKind::Spectral, &[]);
    assert_eq!(run.hand.len(), before + 2);
    assert!(run
        .hand
        .iter()
        .filter(|card| card.enhancement.is_some())
        .count()
        >= 3);

    run.hand = make_hand();
    let before = run.hand.len();
    use_consumable(&mut run, "incantation", ConsumableKind::Spectral, &[]);
    assert_eq!(run.hand.len(), before + 3);
    assert!(run
        .hand
        .iter()
        .filter(|card| card.enhancement.is_some())
        .count()
        >= 4);
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
    assert!(
        run.content
            .jokers
            .iter()
            .any(|joker| joker.rarity == JokerRarity::Legendary)
    );
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
    assert!(shop.cards.iter().any(|card| card.edition == Some(Edition::Foil)));
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
                op: ActionOp::SetShopPrice,
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
                op: ActionOp::SetShopPrice,
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
        ShopPurchase::Voucher => {}
        _ => panic!("expected voucher purchase"),
    }
    let shop = run.shop.as_ref().expect("shop");
    assert_eq!(shop.vouchers, run.config.shop.voucher_slots as usize - 1);
    assert_eq!(run.state.money, initial_money - voucher_price);
    run.apply_purchase(&purchase).expect("apply purchase");
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
        id: boss_id.to_string(),
        name: "Test Boss".to_string(),
        effects: vec![JokerEffect {
            trigger: ActivationType::OnShopEnter,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOp::AddMoney,
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
                op: ActionOp::SetShopPrice,
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
    let err = run.play_hand(&[0, 1, 2, 3, 4], &mut EventBus::default()).unwrap_err();
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
    run.enter_shop(&mut EventBus::default()).expect("enter shop");
    assert_eq!(run.state.shop_free_rerolls, 0);
}

#[test]
fn reroll_shop_not_enough_money() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default()).expect("enter shop");
    run.state.money = 0;
    let err = run.reroll_shop(&mut EventBus::default()).unwrap_err();
    assert!(matches!(err, RunError::NotEnoughMoney));
}

#[test]
fn buy_shop_offer_invalid_index() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default()).expect("enter shop");
    let err = run
        .buy_shop_offer(ShopOfferRef::Card(99), &mut EventBus::default())
        .unwrap_err();
    assert!(matches!(err, RunError::InvalidOfferIndex));
}

#[test]
fn open_pack_purchase_invalid_type() {
    let mut run = new_run();
    let purchase = ShopPurchase::Voucher;
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
    let err = run.use_consumable(0, &[], &mut EventBus::default()).unwrap_err();
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
    assert!(matches!(run.blind_outcome(), Some(rulatro_core::BlindOutcome::Failed)));
}

#[test]
fn blind_outcome_cleared_when_score_reached() {
    let mut run = new_run();
    run.state.target = 10;
    run.state.blind_score = 10;
    run.state.hands_left = 1;
    assert!(matches!(run.blind_outcome(), Some(rulatro_core::BlindOutcome::Cleared)));
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
    assert!(open.options.iter().all(|option| matches!(
        option,
        rulatro_core::PackOption::Joker(_)
    )));
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
    assert!(open.options.iter().all(|option| matches!(
        option,
        rulatro_core::PackOption::PlayingCard(_)
    )));
}

#[test]
fn leave_shop_resets_state() {
    let mut run = new_run();
    mark_blind_cleared(&mut run);
    run.enter_shop(&mut EventBus::default()).expect("enter shop");
    run.state.shop_free_rerolls = 2;
    run.leave_shop();
    assert!(run.shop.is_none());
    assert_eq!(run.state.phase, Phase::Deal);
    assert_eq!(run.state.shop_free_rerolls, 0);
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
    add_rule_joker(&mut run, "rule_discard_held", "discard_held_after_hand", 3.0);
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
