use rulatro_core::{
    BlindKind, Card, ConsumableKind, Enhancement, Edition, EventBus, HandKind, JokerRarity,
    LastConsumable, PackKind, PackOffer, PackSize, Phase, Rank, RunError, RunState, ShopPurchase,
    Suit,
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
fn discard_rejects_invalid_count() {
    let mut run = new_run();
    run.state.phase = Phase::Play;
    run.state.discards_left = 1;
    run.hand = make_hand();
    let err = run.discard(&[0, 1, 2, 3, 4, 5], &mut EventBus::default());
    assert!(matches!(err, Err(RunError::InvalidCardCount)));
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
