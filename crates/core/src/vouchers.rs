use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoucherEffect {
    None,
    AddShopCardSlots(u8),
    AddConsumableSlots(u8),
    AddJokerSlots(u8),
    AddHandsPerRound(u8),
    AddDiscardsPerRound(u8),
    AddHandSizeBase(u8),
    AddTarotWeight(u32),
    AddPlanetWeight(u32),
    ReduceRerollBase(i64),
    SetShopDiscountPercent(u8),
}

/// Voucher definition. Instances are loaded from `assets/vouchers.json` by
/// `rulatro-data` into [`Content::vouchers`]. Use [`Content::voucher_by_id`]
/// or iterate `content.vouchers` instead of any free function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherDef {
    pub id: String,
    pub name_en: String,
    pub name_zh: String,
    pub effect_en: String,
    pub effect_zh: String,
    pub effect: VoucherEffect,
}

impl VoucherDef {
    pub fn name(&self, zh_cn: bool) -> &str {
        if zh_cn {
            &self.name_zh
        } else {
            &self.name_en
        }
    }

    pub fn effect_text(&self, zh_cn: bool) -> &str {
        if zh_cn {
            &self.effect_zh
        } else {
            &self.effect_en
        }
    }
}
