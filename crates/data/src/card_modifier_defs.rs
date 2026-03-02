use anyhow::Context;
use rulatro_core::{
    Action, ActionOp, ActionOpKind, ActivationType, CardAttrRules, CardModifierDef,
    CardModifierKind, Expr, JokerEffect,
};
use serde::Deserialize;

const BUILTIN_JSON: &[u8] = include_bytes!("../card_modifiers.json");

/// Load card modifier definitions from the embedded `card_modifiers.json`, resolving
/// all value expressions against the provided `CardAttrRules`.
pub fn load_builtin_card_modifiers(card_attrs: &CardAttrRules) -> Vec<CardModifierDef> {
    load_card_modifiers(BUILTIN_JSON, card_attrs)
        .expect("built-in card_modifiers.json must be valid")
}

/// Parse `json_bytes` as a `Vec<RawCardModifierDef>` and resolve all value expressions
/// against `card_attrs`, producing the equivalent of what
/// `build_builtin_card_modifiers` generates from Rust.
pub fn load_card_modifiers(
    json_bytes: &[u8],
    card_attrs: &CardAttrRules,
) -> anyhow::Result<Vec<CardModifierDef>> {
    let raws: Vec<RawCardModifierDef> =
        serde_json::from_slice(json_bytes).context("parse card_modifiers JSON")?;
    raws.iter()
        .map(|raw| resolve_modifier(raw, card_attrs))
        .collect()
}

// ── Deserialization types ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RawCardModifierDef {
    kind: String,
    id: String,
    effects: Vec<RawEffect>,
    #[serde(default)]
    destroy_odds: Option<String>,
    #[serde(default)]
    lucky_mult_odds: Option<String>,
    #[serde(default)]
    lucky_mult_add: Option<String>,
    #[serde(default)]
    lucky_money_odds: Option<String>,
    #[serde(default)]
    lucky_money_add: Option<String>,
}

#[derive(Deserialize)]
struct RawEffect {
    trigger: String,
    // `when` is always `true` for built-in card modifiers; we accept but ignore it.
    #[allow(dead_code)]
    #[serde(default)]
    when: serde_json::Value,
    actions: Vec<RawAction>,
}

#[derive(Deserialize)]
struct RawAction {
    op: String,
    value: String,
}

// ── Resolver ─────────────────────────────────────────────────────────────────

fn resolve_modifier(
    raw: &RawCardModifierDef,
    card_attrs: &CardAttrRules,
) -> anyhow::Result<CardModifierDef> {
    let kind = parse_kind(&raw.kind)?;

    let mut effects = Vec::new();
    for raw_effect in &raw.effects {
        let trigger = parse_trigger(&raw_effect.trigger)?;
        let mut actions = Vec::new();
        for raw_action in &raw_effect.actions {
            let op = parse_op(&raw_action.op)?;
            actions.push(Action {
                op: ActionOpKind::Builtin(op),
                target: None,
                value: Expr::Lookup(raw_action.value.clone()),
            });
        }
        if !actions.is_empty() {
            effects.push(JokerEffect {
                trigger,
                when: Expr::Bool(true),
                actions,
            });
        }
    }

    let resolve_u32 = |expr: &Option<String>| -> u32 {
        expr.as_deref()
            .map(|e| resolve_expr(e, &raw.id, card_attrs) as u32)
            .unwrap_or(0)
    };
    let resolve_f64 = |expr: &Option<String>| -> f64 {
        expr.as_deref()
            .map(|e| resolve_expr(e, &raw.id, card_attrs))
            .unwrap_or(0.0)
    };
    let resolve_i64 = |expr: &Option<String>| -> i64 {
        expr.as_deref()
            .map(|e| resolve_expr(e, &raw.id, card_attrs) as i64)
            .unwrap_or(0)
    };

    Ok(CardModifierDef {
        kind,
        id: raw.id.clone(),
        effects,
        destroy_odds: resolve_u32(&raw.destroy_odds),
        lucky_mult_odds: resolve_u32(&raw.lucky_mult_odds),
        lucky_mult_add: resolve_f64(&raw.lucky_mult_add),
        lucky_money_odds: resolve_u32(&raw.lucky_money_odds),
        lucky_money_add: resolve_i64(&raw.lucky_money_add),
    })
}

fn parse_kind(s: &str) -> anyhow::Result<CardModifierKind> {
    match s {
        "enhancement" => Ok(CardModifierKind::Enhancement),
        "edition" => Ok(CardModifierKind::Edition),
        "seal" => Ok(CardModifierKind::Seal),
        _ => Err(anyhow::anyhow!("unknown card modifier kind: {s}")),
    }
}

fn parse_trigger(s: &str) -> anyhow::Result<ActivationType> {
    match s {
        "on_scored" => Ok(ActivationType::OnScored),
        "on_held" => Ok(ActivationType::OnHeld),
        _ => Err(anyhow::anyhow!("unknown trigger: {s}")),
    }
}

fn parse_op(s: &str) -> anyhow::Result<ActionOp> {
    ActionOp::from_keyword(s).ok_or_else(|| anyhow::anyhow!("unknown action op: {s}"))
}

/// Resolve a dot-notation expression like `"enhancement.chips"` to an `f64` value.
/// Delegates to [`CardAttrRules::resolve_lookup`].
fn resolve_expr(expr: &str, id: &str, card_attrs: &CardAttrRules) -> f64 {
    card_attrs.resolve_lookup(expr, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_json_is_valid() {
        let card_attrs = CardAttrRules::default();
        let defs = load_builtin_card_modifiers(&card_attrs);
        assert!(!defs.is_empty(), "should load at least one modifier");
    }

    #[test]
    fn builtin_default_values_match_known_constants() {
        let card_attrs = CardAttrRules::default();
        let defs = load_builtin_card_modifiers(&card_attrs);

        let find = |kind, id| -> &CardModifierDef {
            defs.iter()
                .find(|d| d.kind == kind && d.id == id)
                .unwrap_or_else(|| panic!("missing modifier {id}"))
        };

        // Bonus: add_chips using dynamic lookup key
        let bonus = find(CardModifierKind::Enhancement, "bonus");
        assert_eq!(bonus.effects.len(), 1);
        assert_eq!(bonus.effects[0].trigger, ActivationType::OnScored);
        assert_eq!(bonus.effects[0].actions.len(), 1);
        assert!(matches!(
            &bonus.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "enhancement.chips"
        ));

        // Mult: add_mult using dynamic lookup key
        let mult = find(CardModifierKind::Enhancement, "mult");
        assert!(matches!(
            &mult.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "enhancement.mult"
        ));

        // Glass: mul_mult using dynamic lookup key; destroy_odds is a static struct field
        // resolved at load time (not an action value subject to dynamic lookup)
        let glass = find(CardModifierKind::Enhancement, "glass");
        assert_eq!(glass.destroy_odds, 4);
        assert_eq!(glass.effects.len(), 1);
        assert!(matches!(
            &glass.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "enhancement.x_mult"
        ));

        // Steel: mul_mult (held) using dynamic lookup key
        let steel = find(CardModifierKind::Enhancement, "steel");
        assert_eq!(steel.effects[0].trigger, ActivationType::OnHeld);
        assert!(matches!(
            &steel.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "enhancement.x_mult_held"
        ));

        // Lucky: probabilistic fields (lucky_mult_odds, lucky_mult_add, etc.) are static
        // struct fields resolved at load time — they are not action values in the effects
        // list and are unaffected by the Expr::Lookup change.
        let lucky = find(CardModifierKind::Enhancement, "lucky");
        assert_eq!(lucky.effects.len(), 0);
        assert_eq!(lucky.lucky_mult_odds, 5);
        assert!((lucky.lucky_mult_add - 20.0).abs() < 1e-9);
        assert_eq!(lucky.lucky_money_odds, 15);
        assert_eq!(lucky.lucky_money_add, 20);

        // Polychrome: mul_mult using dynamic lookup key
        let poly = find(CardModifierKind::Edition, "polychrome");
        assert!(matches!(
            &poly.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "edition.x_mult"
        ));

        // Gold seal: add_money using dynamic lookup key
        let gold = find(CardModifierKind::Seal, "gold");
        assert_eq!(gold.effects[0].trigger, ActivationType::OnScored);
        assert!(matches!(
            &gold.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "seal.money_scored"
        ));
    }

    #[test]
    fn custom_card_attrs_resolve_at_eval_time() {
        use rulatro_core::{EditionDef, EnhancementDef};
        use std::collections::HashMap;

        let mut enhancements = HashMap::new();
        enhancements.insert(
            "bonus".to_string(),
            EnhancementDef {
                chips: 99,
                ..Default::default()
            },
        );
        let mut editions = HashMap::new();
        editions.insert(
            "foil".to_string(),
            EditionDef {
                chips: 77,
                ..Default::default()
            },
        );
        let card_attrs = CardAttrRules {
            enhancements,
            editions,
            seals: HashMap::new(),
        };
        let defs = load_builtin_card_modifiers(&card_attrs);

        // Action values are now Expr::Lookup — not pre-resolved at load time.
        let bonus = defs
            .iter()
            .find(|d| d.kind == CardModifierKind::Enhancement && d.id == "bonus")
            .expect("bonus modifier");
        assert!(matches!(
            &bonus.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "enhancement.chips"
        ));

        let foil = defs
            .iter()
            .find(|d| d.kind == CardModifierKind::Edition && d.id == "foil")
            .expect("foil modifier");
        assert!(matches!(
            &foil.effects[0].actions[0].value,
            Expr::Lookup(k) if k == "edition.chips"
        ));

        // Verify that resolve_lookup returns the custom values at evaluation time.
        assert_eq!(
            card_attrs.resolve_lookup("enhancement.chips", "bonus"),
            99.0
        );
        assert_eq!(card_attrs.resolve_lookup("edition.chips", "foil"), 77.0);
    }
}
