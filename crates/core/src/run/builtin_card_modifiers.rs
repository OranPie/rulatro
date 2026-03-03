use crate::{
    Action, ActionOp, ActionOpKind, ActivationType, CardAttrRules, CardModifierDef,
    CardModifierKind, Expr, JokerEffect,
};

fn scored_action(op: ActionOp, value: f64) -> Action {
    Action {
        op: ActionOpKind::Builtin(op),
        target: None,
        value: Expr::Number(value),
    }
}

/// Build the built-in card modifier definitions from the game's `CardAttrRules`.
///
/// All numerical values are resolved from `card_attrs` at startup so that
/// operator expressions in effects are plain `Expr::Number` literals.
pub(super) fn build_builtin_card_modifiers(card_attrs: &CardAttrRules) -> Vec<CardModifierDef> {
    let mut defs = Vec::new();

    // ─── Enhancements (scored) ────────────────────────────────────────────────

    // Bonus: flat chips on score
    let bonus = card_attrs.enhancement("bonus");
    if bonus.chips != 0 {
        defs.push(CardModifierDef::scored(
            CardModifierKind::Enhancement,
            "bonus",
            scored_action(ActionOp::AddChips, bonus.chips as f64),
        ));
    }

    // Mult: flat mult on score
    let mult = card_attrs.enhancement("mult");
    if mult.mult_add != 0.0 {
        defs.push(CardModifierDef::scored(
            CardModifierKind::Enhancement,
            "mult",
            scored_action(ActionOp::AddMult, mult.mult_add),
        ));
    }

    // Glass: multiply mult on score + probabilistic destruction
    let glass = card_attrs.enhancement("glass");
    {
        let effects = if glass.mult_mul != 0.0 {
            vec![JokerEffect {
                trigger: ActivationType::OnScored,
                when: Expr::Bool(true),
                actions: vec![scored_action(ActionOp::MultiplyMult, glass.mult_mul)],
            }]
        } else {
            vec![]
        };
        defs.push(CardModifierDef {
            kind: CardModifierKind::Enhancement,
            id: "glass".into(),
            effects,
            destroy_odds: glass.destroy_odds,
            lucky_mult_odds: 0,
            lucky_mult_add: 0.0,
            lucky_money_odds: 0,
            lucky_money_add: 0,
            retrigger_count: 0,
            phase: crate::ModifierPhase::Pre,
        });
    }

    // Stone: flat chips on score (Stone cards are scored even without rank/suit)
    let stone = card_attrs.enhancement("stone");
    if stone.chips != 0 {
        defs.push(CardModifierDef::scored(
            CardModifierKind::Enhancement,
            "stone",
            scored_action(ActionOp::AddChips, stone.chips as f64),
        ));
    }

    // Lucky: probabilistic mult and/or money on score
    let lucky = card_attrs.enhancement("lucky");
    if lucky.prob_mult_odds > 0 || lucky.prob_money_odds > 0 {
        defs.push(CardModifierDef {
            kind: CardModifierKind::Enhancement,
            id: "lucky".into(),
            effects: vec![],
            destroy_odds: 0,
            lucky_mult_odds: lucky.prob_mult_odds,
            lucky_mult_add: lucky.prob_mult_add,
            lucky_money_odds: lucky.prob_money_odds,
            lucky_money_add: lucky.prob_money_add,
            retrigger_count: 0,
            phase: crate::ModifierPhase::Pre,
        });
    }

    // ─── Enhancements (held) ──────────────────────────────────────────────────

    // Steel: multiply mult when held in hand
    let steel = card_attrs.enhancement("steel");
    if steel.mult_mul_held != 0.0 {
        defs.push(CardModifierDef::held(
            CardModifierKind::Enhancement,
            "steel",
            Action {
                op: ActionOpKind::Builtin(ActionOp::MultiplyMult),
                target: None,
                value: Expr::Number(steel.mult_mul_held),
            },
        ));
    }

    // Gold enhancement: add money at round end for held cards
    let gold_enh = card_attrs.seal("gold"); // uses seal.money_held for Gold enhancement
    if gold_enh.money_held != 0 {
        defs.push(CardModifierDef::simple(
            CardModifierKind::Enhancement,
            "gold",
            vec![JokerEffect {
                trigger: ActivationType::OnRoundEnd,
                when: Expr::Bool(true),
                actions: vec![scored_action(
                    ActionOp::AddMoney,
                    gold_enh.money_held as f64,
                )],
            }],
        ));
    }

    // ─── Editions (scored) ───────────────────────────────────────────────────

    // Foil: flat chips on score
    let foil = card_attrs.edition("foil");
    if foil.chips != 0 {
        defs.push(CardModifierDef::scored(
            CardModifierKind::Edition,
            "foil",
            scored_action(ActionOp::AddChips, foil.chips as f64),
        ));
    }

    // Holographic: flat mult on score
    let holographic = card_attrs.edition("holographic");
    if holographic.mult_add != 0.0 {
        defs.push(CardModifierDef::scored(
            CardModifierKind::Edition,
            "holographic",
            scored_action(ActionOp::AddMult, holographic.mult_add),
        ));
    }

    // Polychrome: multiply mult on score (post-phase — applied after main scoring)
    let polychrome = card_attrs.edition("polychrome");
    if polychrome.mult_mul != 0.0 {
        let mut def = CardModifierDef::scored(
            CardModifierKind::Edition,
            "polychrome",
            scored_action(ActionOp::MultiplyMult, polychrome.mult_mul),
        );
        def.phase = crate::ModifierPhase::Post;
        defs.push(def);
    }

    // ─── Seals (scored + side-effects) ───────────────────────────────────────

    // Gold seal: add money when scored
    let gold_seal = card_attrs.seal("gold");
    if gold_seal.money_scored != 0 {
        defs.push(CardModifierDef::scored(
            CardModifierKind::Seal,
            "gold",
            scored_action(ActionOp::AddMoney, gold_seal.money_scored as f64),
        ));
    }

    // Red seal: retrigger cards (count=2 means score twice)
    {
        let mut def = CardModifierDef::simple(CardModifierKind::Seal, "red", vec![]);
        def.retrigger_count = 2;
        defs.push(def);
    }

    // Purple seal: grant random tarot on discard
    defs.push(CardModifierDef::simple(
        CardModifierKind::Seal,
        "purple",
        vec![JokerEffect {
            trigger: ActivationType::OnDiscard,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::GrantRandomConsumable),
                target: Some("tarot".to_string()),
                value: Expr::String("tarot".to_string()),
            }],
        }],
    ));

    // Blue seal: grant planet for hand at round end
    defs.push(CardModifierDef::simple(
        CardModifierKind::Seal,
        "blue",
        vec![JokerEffect {
            trigger: ActivationType::OnRoundEnd,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::GrantPlanetForHand),
                target: None,
                value: Expr::Number(0.0),
            }],
        }],
    ));

    defs
}
