use rulatro_core::CardConditionalRule;

const DEBUFF_RULES_JSON: &[u8] = include_bytes!("../../../assets/card_debuff_rules.json");
const DRAW_FACEDOWN_RULES_JSON: &[u8] =
    include_bytes!("../../../assets/card_draw_facedown_rules.json");

pub fn load_builtin_debuff_rules() -> Vec<CardConditionalRule> {
    serde_json::from_slice(DEBUFF_RULES_JSON)
        .expect("assets/card_debuff_rules.json must be valid JSON")
}

pub fn load_builtin_draw_facedown_rules() -> Vec<CardConditionalRule> {
    serde_json::from_slice(DRAW_FACEDOWN_RULES_JSON)
        .expect("assets/card_draw_facedown_rules.json must be valid JSON")
}
