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

#[derive(Debug, Clone, Copy)]
pub struct VoucherDef {
    pub id: &'static str,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub effect_en: &'static str,
    pub effect_zh: &'static str,
    pub effect: VoucherEffect,
}

impl VoucherDef {
    pub fn name(self, zh_cn: bool) -> &'static str {
        if zh_cn {
            self.name_zh
        } else {
            self.name_en
        }
    }

    pub fn effect_text(self, zh_cn: bool) -> &'static str {
        if zh_cn {
            self.effect_zh
        } else {
            self.effect_en
        }
    }
}

const VOUCHERS: &[VoucherDef] = &[
    VoucherDef {
        id: "overstock",
        name_en: "Overstock",
        name_zh: "超额库存",
        effect_en: "+1 shop card slot",
        effect_zh: "商店卡牌槽位 +1",
        effect: VoucherEffect::AddShopCardSlots(1),
    },
    VoucherDef {
        id: "overstock_plus",
        name_en: "Overstock Plus",
        name_zh: "超额库存+",
        effect_en: "+1 shop card slot",
        effect_zh: "商店卡牌槽位 +1",
        effect: VoucherEffect::AddShopCardSlots(1),
    },
    VoucherDef {
        id: "clearance_sale",
        name_en: "Clearance Sale",
        name_zh: "清仓特卖",
        effect_en: "cards/packs 25% off",
        effect_zh: "卡牌/卡包 25% 折扣",
        effect: VoucherEffect::SetShopDiscountPercent(25),
    },
    VoucherDef {
        id: "liquidation",
        name_en: "Liquidation",
        name_zh: "甩卖",
        effect_en: "cards/packs 50% off",
        effect_zh: "卡牌/卡包 50% 折扣",
        effect: VoucherEffect::SetShopDiscountPercent(50),
    },
    VoucherDef {
        id: "hone",
        name_en: "Hone",
        name_zh: "抛光",
        effect_en: "editions appear more often (todo)",
        effect_zh: "版本出现率提升（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "glow_up",
        name_en: "Glow Up",
        name_zh: "闪亮升级",
        effect_en: "editions appear much more often (todo)",
        effect_zh: "版本出现率大幅提升（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "reroll_surplus",
        name_en: "Reroll Surplus",
        name_zh: "刷新富余",
        effect_en: "reroll base cost -2",
        effect_zh: "刷新基础费用 -2",
        effect: VoucherEffect::ReduceRerollBase(2),
    },
    VoucherDef {
        id: "reroll_glut",
        name_en: "Reroll Glut",
        name_zh: "刷新过剩",
        effect_en: "reroll base cost -2",
        effect_zh: "刷新基础费用 -2",
        effect: VoucherEffect::ReduceRerollBase(2),
    },
    VoucherDef {
        id: "crystal_ball",
        name_en: "Crystal Ball",
        name_zh: "水晶球",
        effect_en: "+1 consumable slot",
        effect_zh: "消耗牌槽位 +1",
        effect: VoucherEffect::AddConsumableSlots(1),
    },
    VoucherDef {
        id: "omen_globe",
        name_en: "Omen Globe",
        name_zh: "预兆之球",
        effect_en: "spectral shop behavior (todo)",
        effect_zh: "灵异牌商店效果（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "telescope",
        name_en: "Telescope",
        name_zh: "望远镜",
        effect_en: "celestial pack targeting (todo)",
        effect_zh: "星球包定向效果（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "observatory",
        name_en: "Observatory",
        name_zh: "天文台",
        effect_en: "planet scaling effect (todo)",
        effect_zh: "星球缩放效果（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "grabber",
        name_en: "Grabber",
        name_zh: "抓手",
        effect_en: "+1 hand each round",
        effect_zh: "每回合出牌次数 +1",
        effect: VoucherEffect::AddHandsPerRound(1),
    },
    VoucherDef {
        id: "nacho_tong",
        name_en: "Nacho Tong",
        name_zh: "玉米片夹",
        effect_en: "+1 hand each round",
        effect_zh: "每回合出牌次数 +1",
        effect: VoucherEffect::AddHandsPerRound(1),
    },
    VoucherDef {
        id: "wasteful",
        name_en: "Wasteful",
        name_zh: "挥霍",
        effect_en: "+1 discard each round",
        effect_zh: "每回合弃牌次数 +1",
        effect: VoucherEffect::AddDiscardsPerRound(1),
    },
    VoucherDef {
        id: "recyclomancy",
        name_en: "Recyclomancy",
        name_zh: "回收术",
        effect_en: "+1 discard each round",
        effect_zh: "每回合弃牌次数 +1",
        effect: VoucherEffect::AddDiscardsPerRound(1),
    },
    VoucherDef {
        id: "tarot_merchant",
        name_en: "Tarot Merchant",
        name_zh: "塔罗商人",
        effect_en: "tarot offers more frequent",
        effect_zh: "塔罗牌商品出现率提升",
        effect: VoucherEffect::AddTarotWeight(6),
    },
    VoucherDef {
        id: "tarot_tycoon",
        name_en: "Tarot Tycoon",
        name_zh: "塔罗大亨",
        effect_en: "tarot offers much more frequent",
        effect_zh: "塔罗牌商品出现率大幅提升",
        effect: VoucherEffect::AddTarotWeight(20),
    },
    VoucherDef {
        id: "planet_merchant",
        name_en: "Planet Merchant",
        name_zh: "行星商人",
        effect_en: "planet offers more frequent",
        effect_zh: "行星牌商品出现率提升",
        effect: VoucherEffect::AddPlanetWeight(6),
    },
    VoucherDef {
        id: "planet_tycoon",
        name_en: "Planet Tycoon",
        name_zh: "行星大亨",
        effect_en: "planet offers much more frequent",
        effect_zh: "行星牌商品出现率大幅提升",
        effect: VoucherEffect::AddPlanetWeight(20),
    },
    VoucherDef {
        id: "seed_money",
        name_en: "Seed Money",
        name_zh: "种子资金",
        effect_en: "interest cap increase (todo)",
        effect_zh: "利息上限提升（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "money_tree",
        name_en: "Money Tree",
        name_zh: "摇钱树",
        effect_en: "interest cap increase (todo)",
        effect_zh: "利息上限提升（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "blank",
        name_en: "Blank",
        name_zh: "空白",
        effect_en: "no direct effect",
        effect_zh: "无直接效果",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "antimatter",
        name_en: "Antimatter",
        name_zh: "反物质",
        effect_en: "+1 joker slot",
        effect_zh: "小丑槽位 +1",
        effect: VoucherEffect::AddJokerSlots(1),
    },
    VoucherDef {
        id: "magic_trick",
        name_en: "Magic Trick",
        name_zh: "魔术技巧",
        effect_en: "playing cards in shop (todo)",
        effect_zh: "商店出现扑克牌（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "illusion",
        name_en: "Illusion",
        name_zh: "幻象",
        effect_en: "shop playing card modifiers (todo)",
        effect_zh: "商店扑克牌词缀（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "hieroglyph",
        name_en: "Hieroglyph",
        name_zh: "象形文字",
        effect_en: "ante manipulation (todo)",
        effect_zh: "底注变化效果（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "petroglyph",
        name_en: "Petroglyph",
        name_zh: "岩画",
        effect_en: "ante manipulation (todo)",
        effect_zh: "底注变化效果（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "directors_cut",
        name_en: "Director's Cut",
        name_zh: "导演剪辑",
        effect_en: "boss reroll unlock (todo)",
        effect_zh: "Boss 刷新解锁（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "retcon",
        name_en: "Retcon",
        name_zh: "设定重写",
        effect_en: "boss reroll upgrade (todo)",
        effect_zh: "Boss 刷新升级（待实现）",
        effect: VoucherEffect::None,
    },
    VoucherDef {
        id: "paint_brush",
        name_en: "Paint Brush",
        name_zh: "画笔",
        effect_en: "+1 hand size",
        effect_zh: "手牌上限 +1",
        effect: VoucherEffect::AddHandSizeBase(1),
    },
    VoucherDef {
        id: "palette",
        name_en: "Palette",
        name_zh: "调色板",
        effect_en: "+1 hand size",
        effect_zh: "手牌上限 +1",
        effect: VoucherEffect::AddHandSizeBase(1),
    },
];

pub fn all_vouchers() -> &'static [VoucherDef] {
    VOUCHERS
}

pub fn voucher_by_id(id: &str) -> Option<VoucherDef> {
    VOUCHERS.iter().copied().find(|voucher| voucher.id == id)
}
