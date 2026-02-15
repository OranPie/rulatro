use rulatro_core::{
    format_action_compact, format_expr_compact, format_joker_effect_compact, Action, ActionOp,
    ActivationType, BinaryOp, Expr, JokerEffect, UnaryOp,
};

macro_rules! keyword_case {
    ($name:ident, $keyword:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(ActionOp::from_keyword($keyword), Some($expected));
        }
    };
}

keyword_case!(keyword_add_chips, "add_chips", ActionOp::AddChips);
keyword_case!(keyword_add_mult, "add_mult", ActionOp::AddMult);
keyword_case!(keyword_mul_mult, "mul_mult", ActionOp::MultiplyMult);
keyword_case!(
    keyword_multiply_mult,
    "multiply_mult",
    ActionOp::MultiplyMult
);
keyword_case!(keyword_mul_chips, "mul_chips", ActionOp::MultiplyChips);
keyword_case!(
    keyword_multiply_chips,
    "multiply_chips",
    ActionOp::MultiplyChips
);
keyword_case!(keyword_add_money, "add_money", ActionOp::AddMoney);
keyword_case!(keyword_set_money, "set_money", ActionOp::SetMoney);
keyword_case!(keyword_money_set, "money_set", ActionOp::SetMoney);
keyword_case!(
    keyword_add_hand_size,
    "add_hand_size",
    ActionOp::AddHandSize
);
keyword_case!(
    keyword_retrigger_scored,
    "retrigger_scored",
    ActionOp::RetriggerScored
);
keyword_case!(
    keyword_retrigger_held,
    "retrigger_held",
    ActionOp::RetriggerHeld
);
keyword_case!(
    keyword_add_stone_card,
    "add_stone_card",
    ActionOp::AddStoneCard
);
keyword_case!(
    keyword_add_card_bonus,
    "add_card_bonus",
    ActionOp::AddCardBonus
);
keyword_case!(
    keyword_add_card_chips,
    "add_card_chips",
    ActionOp::AddCardBonus
);
keyword_case!(keyword_card_bonus, "card_bonus", ActionOp::AddCardBonus);
keyword_case!(
    keyword_set_card_enhancement,
    "set_card_enhancement",
    ActionOp::SetCardEnhancement
);
keyword_case!(
    keyword_set_enhancement,
    "set_enhancement",
    ActionOp::SetCardEnhancement
);
keyword_case!(
    keyword_card_enhancement,
    "card_enhancement",
    ActionOp::SetCardEnhancement
);
keyword_case!(
    keyword_clear_card_enhancement,
    "clear_card_enhancement",
    ActionOp::ClearCardEnhancement
);
keyword_case!(
    keyword_remove_card_enhancement,
    "remove_card_enhancement",
    ActionOp::ClearCardEnhancement
);
keyword_case!(
    keyword_clear_enhancement,
    "clear_enhancement",
    ActionOp::ClearCardEnhancement
);
keyword_case!(keyword_destroy_card, "destroy_card", ActionOp::DestroyCard);
keyword_case!(
    keyword_destroy_current_card,
    "destroy_current_card",
    ActionOp::DestroyCard
);
keyword_case!(
    keyword_copy_played_card,
    "copy_played_card",
    ActionOp::CopyPlayedCard
);
keyword_case!(keyword_copy_card, "copy_card", ActionOp::CopyPlayedCard);
keyword_case!(
    keyword_copy_scoring_card,
    "copy_scoring_card",
    ActionOp::CopyPlayedCard
);
keyword_case!(keyword_add_hands, "add_hands", ActionOp::AddHands);
keyword_case!(keyword_add_discards, "add_discards", ActionOp::AddDiscards);
keyword_case!(keyword_set_discards, "set_discards", ActionOp::SetDiscards);
keyword_case!(keyword_add_tarot, "add_tarot", ActionOp::AddTarot);
keyword_case!(keyword_add_planet, "add_planet", ActionOp::AddPlanet);
keyword_case!(keyword_add_spectral, "add_spectral", ActionOp::AddSpectral);
keyword_case!(
    keyword_add_free_reroll,
    "add_free_reroll",
    ActionOp::AddFreeReroll
);
keyword_case!(
    keyword_set_shop_price,
    "set_shop_price",
    ActionOp::SetShopPrice
);
keyword_case!(keyword_shop_price, "shop_price", ActionOp::SetShopPrice);
keyword_case!(keyword_add_joker, "add_joker", ActionOp::AddJoker);
keyword_case!(
    keyword_add_random_joker,
    "add_random_joker",
    ActionOp::AddJoker
);
keyword_case!(
    keyword_destroy_random_joker,
    "destroy_random_joker",
    ActionOp::DestroyRandomJoker
);
keyword_case!(
    keyword_destroy_joker_random,
    "destroy_joker_random",
    ActionOp::DestroyRandomJoker
);
keyword_case!(
    keyword_destroy_joker_right,
    "destroy_joker_right",
    ActionOp::DestroyJokerRight
);
keyword_case!(
    keyword_destroy_right_joker,
    "destroy_right_joker",
    ActionOp::DestroyJokerRight
);
keyword_case!(
    keyword_destroy_joker_left,
    "destroy_joker_left",
    ActionOp::DestroyJokerLeft
);
keyword_case!(
    keyword_destroy_left_joker,
    "destroy_left_joker",
    ActionOp::DestroyJokerLeft
);
keyword_case!(keyword_destroy_self, "destroy_self", ActionOp::DestroySelf);
keyword_case!(keyword_upgrade_hand, "upgrade_hand", ActionOp::UpgradeHand);
keyword_case!(
    keyword_duplicate_random_joker,
    "duplicate_random_joker",
    ActionOp::DuplicateRandomJoker
);
keyword_case!(
    keyword_dup_random_joker,
    "dup_random_joker",
    ActionOp::DuplicateRandomJoker
);
keyword_case!(
    keyword_duplicate_random_consumable,
    "duplicate_random_consumable",
    ActionOp::DuplicateRandomConsumable
);
keyword_case!(
    keyword_dup_random_consumable,
    "dup_random_consumable",
    ActionOp::DuplicateRandomConsumable
);
keyword_case!(
    keyword_add_sell_bonus,
    "add_sell_bonus",
    ActionOp::AddSellBonus
);
keyword_case!(keyword_sell_bonus, "sell_bonus", ActionOp::AddSellBonus);
keyword_case!(keyword_disable_boss, "disable_boss", ActionOp::DisableBoss);
keyword_case!(keyword_boss_disable, "boss_disable", ActionOp::DisableBoss);
keyword_case!(
    keyword_add_random_hand_card,
    "add_random_hand_card",
    ActionOp::AddRandomHandCard
);
keyword_case!(
    keyword_add_hand_card,
    "add_hand_card",
    ActionOp::AddRandomHandCard
);
keyword_case!(
    keyword_copy_joker_right,
    "copy_joker_right",
    ActionOp::CopyJokerRight
);
keyword_case!(
    keyword_copy_right_joker,
    "copy_right_joker",
    ActionOp::CopyJokerRight
);
keyword_case!(
    keyword_copy_joker_leftmost,
    "copy_joker_leftmost",
    ActionOp::CopyJokerLeftmost
);
keyword_case!(
    keyword_copy_leftmost_joker,
    "copy_leftmost_joker",
    ActionOp::CopyJokerLeftmost
);
keyword_case!(
    keyword_prevent_death,
    "prevent_death",
    ActionOp::PreventDeath
);
keyword_case!(keyword_survive, "survive", ActionOp::PreventDeath);
keyword_case!(keyword_add_tag, "add_tag", ActionOp::AddTag);
keyword_case!(keyword_tag, "tag", ActionOp::AddTag);
keyword_case!(
    keyword_duplicate_next_tag,
    "duplicate_next_tag",
    ActionOp::DuplicateNextTag
);
keyword_case!(
    keyword_dup_next_tag,
    "dup_next_tag",
    ActionOp::DuplicateNextTag
);
keyword_case!(keyword_add_pack, "add_pack", ActionOp::AddPack);
keyword_case!(
    keyword_add_booster_pack,
    "add_booster_pack",
    ActionOp::AddPack
);
keyword_case!(
    keyword_add_shop_joker,
    "add_shop_joker",
    ActionOp::AddShopJoker
);
keyword_case!(keyword_shop_joker, "shop_joker", ActionOp::AddShopJoker);
keyword_case!(keyword_add_voucher, "add_voucher", ActionOp::AddVoucher);
keyword_case!(keyword_voucher_add, "voucher_add", ActionOp::AddVoucher);
keyword_case!(
    keyword_set_reroll_cost,
    "set_reroll_cost",
    ActionOp::SetRerollCost
);
keyword_case!(keyword_reroll_cost, "reroll_cost", ActionOp::SetRerollCost);
keyword_case!(
    keyword_set_shop_joker_edition,
    "set_shop_joker_edition",
    ActionOp::SetShopJokerEdition
);
keyword_case!(
    keyword_shop_joker_edition,
    "shop_joker_edition",
    ActionOp::SetShopJokerEdition
);
keyword_case!(keyword_reroll_boss, "reroll_boss", ActionOp::RerollBoss);
keyword_case!(keyword_boss_reroll, "boss_reroll", ActionOp::RerollBoss);
keyword_case!(
    keyword_upgrade_random_hand,
    "upgrade_random_hand",
    ActionOp::UpgradeRandomHand
);
keyword_case!(
    keyword_upgrade_hand_random,
    "upgrade_hand_random",
    ActionOp::UpgradeRandomHand
);
keyword_case!(keyword_set_hands, "set_hands", ActionOp::SetHands);
keyword_case!(keyword_hands_set, "hands_set", ActionOp::SetHands);
keyword_case!(keyword_set_hands_left, "set_hands_left", ActionOp::SetHands);
keyword_case!(keyword_mul_target, "mul_target", ActionOp::MultiplyTarget);
keyword_case!(
    keyword_multiply_target,
    "multiply_target",
    ActionOp::MultiplyTarget
);
keyword_case!(keyword_target_mult, "target_mult", ActionOp::MultiplyTarget);
keyword_case!(keyword_set_rule, "set_rule", ActionOp::SetRule);
keyword_case!(keyword_rule_set, "rule_set", ActionOp::SetRule);
keyword_case!(keyword_add_rule, "add_rule", ActionOp::AddRule);
keyword_case!(keyword_rule_add, "rule_add", ActionOp::AddRule);
keyword_case!(keyword_clear_rule, "clear_rule", ActionOp::ClearRule);
keyword_case!(keyword_rule_clear, "rule_clear", ActionOp::ClearRule);
keyword_case!(keyword_set_var, "set_var", ActionOp::SetVar);
keyword_case!(keyword_add_var, "add_var", ActionOp::AddVar);

#[test]
fn keyword_unknown_is_none() {
    assert_eq!(ActionOp::from_keyword("unknown_action"), None);
}

macro_rules! target_case {
    ($name:ident, $variant:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!($variant.requires_target(), $expected);
        }
    };
}

target_case!(requires_target_addchips, ActionOp::AddChips, false);
target_case!(requires_target_addmult, ActionOp::AddMult, false);
target_case!(requires_target_multiplymult, ActionOp::MultiplyMult, false);
target_case!(
    requires_target_multiplychips,
    ActionOp::MultiplyChips,
    false
);
target_case!(requires_target_addmoney, ActionOp::AddMoney, false);
target_case!(requires_target_setmoney, ActionOp::SetMoney, false);
target_case!(requires_target_addhandsize, ActionOp::AddHandSize, false);
target_case!(
    requires_target_retriggerscored,
    ActionOp::RetriggerScored,
    false
);
target_case!(
    requires_target_retriggerheld,
    ActionOp::RetriggerHeld,
    false
);
target_case!(requires_target_addstonecard, ActionOp::AddStoneCard, false);
target_case!(requires_target_addcardbonus, ActionOp::AddCardBonus, false);
target_case!(
    requires_target_setcardenhancement,
    ActionOp::SetCardEnhancement,
    true
);
target_case!(
    requires_target_clearcardenhancement,
    ActionOp::ClearCardEnhancement,
    false
);
target_case!(requires_target_destroycard, ActionOp::DestroyCard, false);
target_case!(
    requires_target_copyplayedcard,
    ActionOp::CopyPlayedCard,
    false
);
target_case!(requires_target_addhands, ActionOp::AddHands, false);
target_case!(requires_target_adddiscards, ActionOp::AddDiscards, false);
target_case!(requires_target_setdiscards, ActionOp::SetDiscards, false);
target_case!(requires_target_addtarot, ActionOp::AddTarot, false);
target_case!(requires_target_addplanet, ActionOp::AddPlanet, false);
target_case!(requires_target_addspectral, ActionOp::AddSpectral, false);
target_case!(
    requires_target_addfreereroll,
    ActionOp::AddFreeReroll,
    false
);
target_case!(requires_target_setshopprice, ActionOp::SetShopPrice, true);
target_case!(requires_target_addjoker, ActionOp::AddJoker, true);
target_case!(
    requires_target_destroyrandomjoker,
    ActionOp::DestroyRandomJoker,
    false
);
target_case!(
    requires_target_destroyjokerright,
    ActionOp::DestroyJokerRight,
    false
);
target_case!(
    requires_target_destroyjokerleft,
    ActionOp::DestroyJokerLeft,
    false
);
target_case!(requires_target_destroyself, ActionOp::DestroySelf, false);
target_case!(requires_target_upgradehand, ActionOp::UpgradeHand, false);
target_case!(
    requires_target_duplicaterandomjoker,
    ActionOp::DuplicateRandomJoker,
    false
);
target_case!(
    requires_target_duplicaterandomconsumable,
    ActionOp::DuplicateRandomConsumable,
    false
);
target_case!(requires_target_addsellbonus, ActionOp::AddSellBonus, true);
target_case!(requires_target_disableboss, ActionOp::DisableBoss, false);
target_case!(
    requires_target_addrandomhandcard,
    ActionOp::AddRandomHandCard,
    false
);
target_case!(
    requires_target_copyjokerright,
    ActionOp::CopyJokerRight,
    false
);
target_case!(
    requires_target_copyjokerleftmost,
    ActionOp::CopyJokerLeftmost,
    false
);
target_case!(requires_target_preventdeath, ActionOp::PreventDeath, false);
target_case!(requires_target_addtag, ActionOp::AddTag, true);
target_case!(
    requires_target_duplicatenexttag,
    ActionOp::DuplicateNextTag,
    true
);
target_case!(requires_target_addpack, ActionOp::AddPack, true);
target_case!(requires_target_addshopjoker, ActionOp::AddShopJoker, true);
target_case!(requires_target_addvoucher, ActionOp::AddVoucher, false);
target_case!(
    requires_target_setrerollcost,
    ActionOp::SetRerollCost,
    false
);
target_case!(
    requires_target_setshopjokeredition,
    ActionOp::SetShopJokerEdition,
    true
);
target_case!(requires_target_rerollboss, ActionOp::RerollBoss, false);
target_case!(
    requires_target_upgraderandomhand,
    ActionOp::UpgradeRandomHand,
    false
);
target_case!(requires_target_sethands, ActionOp::SetHands, false);
target_case!(
    requires_target_multiplytarget,
    ActionOp::MultiplyTarget,
    false
);
target_case!(requires_target_setrule, ActionOp::SetRule, true);
target_case!(requires_target_addrule, ActionOp::AddRule, true);
target_case!(requires_target_clearrule, ActionOp::ClearRule, true);
target_case!(requires_target_setvar, ActionOp::SetVar, true);
target_case!(requires_target_addvar, ActionOp::AddVar, true);

macro_rules! activation_case {
    ($name:ident, $variant:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let effect = JokerEffect {
                trigger: $variant,
                when: Expr::Bool(true),
                actions: Vec::new(),
            };
            assert_eq!(
                format_joker_effect_compact(&effect),
                format!("on {}", $expected)
            );
        }
    };
}

activation_case!(activation_played, ActivationType::OnPlayed, "played");
activation_case!(
    activation_scored_pre,
    ActivationType::OnScoredPre,
    "scored_pre"
);
activation_case!(activation_scored, ActivationType::OnScored, "scored");
activation_case!(activation_held, ActivationType::OnHeld, "held");
activation_case!(
    activation_independent,
    ActivationType::Independent,
    "independent"
);
activation_case!(
    activation_other_jokers,
    ActivationType::OnOtherJokers,
    "other_jokers"
);
activation_case!(activation_discard, ActivationType::OnDiscard, "discard");
activation_case!(
    activation_discard_batch,
    ActivationType::OnDiscardBatch,
    "discard_batch"
);
activation_case!(
    activation_card_destroyed,
    ActivationType::OnCardDestroyed,
    "card_destroyed"
);
activation_case!(
    activation_card_added,
    ActivationType::OnCardAdded,
    "card_added"
);
activation_case!(
    activation_round_end,
    ActivationType::OnRoundEnd,
    "round_end"
);
activation_case!(activation_hand_end, ActivationType::OnHandEnd, "hand_end");
activation_case!(
    activation_blind_start,
    ActivationType::OnBlindStart,
    "blind_start"
);
activation_case!(
    activation_blind_failed,
    ActivationType::OnBlindFailed,
    "blind_failed"
);
activation_case!(
    activation_shop_enter,
    ActivationType::OnShopEnter,
    "shop_enter"
);
activation_case!(
    activation_shop_reroll,
    ActivationType::OnShopReroll,
    "shop_reroll"
);
activation_case!(
    activation_shop_exit,
    ActivationType::OnShopExit,
    "shop_exit"
);
activation_case!(
    activation_pack_opened,
    ActivationType::OnPackOpened,
    "pack_opened"
);
activation_case!(
    activation_pack_skipped,
    ActivationType::OnPackSkipped,
    "pack_skipped"
);
activation_case!(activation_use, ActivationType::OnUse, "use");
activation_case!(activation_sell, ActivationType::OnSell, "sell");
activation_case!(activation_any_sell, ActivationType::OnAnySell, "any_sell");
activation_case!(activation_acquire, ActivationType::OnAcquire, "acquire");
activation_case!(activation_passive, ActivationType::Passive, "passive");

macro_rules! expr_number_case {
    ($name:ident, $value:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(format_expr_compact(&Expr::Number($value)), $expected);
        }
    };
}

expr_number_case!(expr_number_0, 0.0, "0");
expr_number_case!(expr_number_1, 1.0, "1");
expr_number_case!(expr_number_2, 2.0, "2");
expr_number_case!(expr_number_3, 3.0, "3");
expr_number_case!(expr_number_10, 10.0, "10");
expr_number_case!(expr_number_100, 100.0, "100");
expr_number_case!(expr_number_minus_1, -1.0, "-1");
expr_number_case!(expr_number_minus_5, -5.0, "-5");
expr_number_case!(expr_number_half, 0.5, "0.5");
expr_number_case!(expr_number_quarter, 0.25, "0.25");
expr_number_case!(expr_number_three_quarters, 0.75, "0.75");
expr_number_case!(expr_number_one_and_quarter, 1.25, "1.25");
expr_number_case!(expr_number_one_and_half, 1.5, "1.5");
expr_number_case!(expr_number_one_and_1250, 1.125, "1.125");
expr_number_case!(expr_number_pi_trim, 3.14159, "3.1416");
expr_number_case!(expr_number_tiny_trim, 2.5, "2.5");
expr_number_case!(expr_number_negative_fraction, -2.125, "-2.125");
expr_number_case!(expr_number_negative_round, -9.87654, "-9.8765");
expr_number_case!(expr_number_large_fraction, 12345.5, "12345.5");
expr_number_case!(expr_number_large_round, 12345.67891, "12345.6789");

#[test]
fn expr_formats_string_ident_and_call() {
    assert_eq!(
        format_expr_compact(&Expr::String("abc".to_string())),
        "\"abc\""
    );
    assert_eq!(format_expr_compact(&Expr::Ident("var".to_string())), "var");
    let call = Expr::Call {
        name: "foo".to_string(),
        args: vec![Expr::Number(1.0), Expr::Bool(false)],
    };
    assert_eq!(format_expr_compact(&call), "foo(1, false)");
}

macro_rules! binary_case {
    ($name:ident, $op:expr, $symbol:expr) => {
        #[test]
        fn $name() {
            let expr = Expr::Binary {
                left: Box::new(Expr::Number(1.0)),
                op: $op,
                right: Box::new(Expr::Number(2.0)),
            };
            assert_eq!(format_expr_compact(&expr), format!("1 {} 2", $symbol));
        }
    };
}

binary_case!(binary_or, BinaryOp::Or, "||");
binary_case!(binary_and, BinaryOp::And, "&&");
binary_case!(binary_eq, BinaryOp::Eq, "==");
binary_case!(binary_ne, BinaryOp::Ne, "!=");
binary_case!(binary_lt, BinaryOp::Lt, "<");
binary_case!(binary_le, BinaryOp::Le, "<=");
binary_case!(binary_gt, BinaryOp::Gt, ">");
binary_case!(binary_ge, BinaryOp::Ge, ">=");
binary_case!(binary_add, BinaryOp::Add, "+");
binary_case!(binary_sub, BinaryOp::Sub, "-");
binary_case!(binary_mul, BinaryOp::Mul, "*");
binary_case!(binary_div, BinaryOp::Div, "/");

#[test]
fn expr_wraps_nested_binary_children() {
    let inner = Expr::Binary {
        left: Box::new(Expr::Number(1.0)),
        op: BinaryOp::Add,
        right: Box::new(Expr::Number(2.0)),
    };
    let outer = Expr::Binary {
        left: Box::new(inner),
        op: BinaryOp::Mul,
        right: Box::new(Expr::Number(3.0)),
    };
    assert_eq!(format_expr_compact(&outer), "(1 + 2) * 3");
}

#[test]
fn expr_unary_formats() {
    let neg = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Expr::Number(2.0)),
    };
    assert_eq!(format_expr_compact(&neg), "-2");
    let not_expr = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(Expr::Bool(true)),
    };
    assert_eq!(format_expr_compact(&not_expr), "!true");
}

macro_rules! action_case {
    ($name:ident, $value:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let action = Action {
                op: ActionOp::AddMoney,
                target: Some("bank".to_string()),
                value: Expr::Number($value),
            };
            assert_eq!(format_action_compact(&action), $expected);
        }
    };
}

action_case!(action_value_0, 0.0, "add_money bank 0");
action_case!(action_value_1, 1.0, "add_money bank 1");
action_case!(action_value_2, 2.0, "add_money bank 2");
action_case!(action_value_3, 3.0, "add_money bank 3");
action_case!(action_value_4, 4.0, "add_money bank 4");
action_case!(action_value_5, 5.0, "add_money bank 5");
action_case!(action_value_6, 6.0, "add_money bank 6");
action_case!(action_value_7, 7.0, "add_money bank 7");
action_case!(action_value_8, 8.0, "add_money bank 8");
action_case!(action_value_9, 9.0, "add_money bank 9");

#[test]
fn action_without_target_omits_target_text() {
    let action = Action {
        op: ActionOp::AddMoney,
        target: None,
        value: Expr::Number(7.0),
    };
    assert_eq!(format_action_compact(&action), "add_money 7");
}

#[test]
fn effect_with_condition_formats_compact_text() {
    let effect = JokerEffect {
        trigger: ActivationType::OnScored,
        when: Expr::Binary {
            left: Box::new(Expr::Ident("x".to_string())),
            op: BinaryOp::Gt,
            right: Box::new(Expr::Number(3.0)),
        },
        actions: vec![Action {
            op: ActionOp::SetVar,
            target: Some("score".to_string()),
            value: Expr::Number(10.0),
        }],
    };
    assert_eq!(
        format_joker_effect_compact(&effect),
        "on scored when x > 3 { set_var score 10 }"
    );
}
