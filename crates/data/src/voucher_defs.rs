use anyhow::Context;
use rulatro_core::VoucherDef;

const BUILTIN_JSON: &[u8] = include_bytes!("../../../assets/vouchers.json");

/// Load voucher definitions from the embedded `assets/vouchers.json`.
pub fn load_builtin_vouchers() -> Vec<VoucherDef> {
    serde_json::from_slice(BUILTIN_JSON).expect("built-in vouchers.json must be valid")
}

/// Parse `json_bytes` as a `Vec<VoucherDef>`.
pub fn load_vouchers(json_bytes: &[u8]) -> anyhow::Result<Vec<VoucherDef>> {
    serde_json::from_slice(json_bytes).context("parse vouchers JSON")
}
