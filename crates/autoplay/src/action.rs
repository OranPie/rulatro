use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AutoAction {
    Deal,
    Play { indices: Vec<usize> },
    Discard { indices: Vec<usize> },
    SkipBlind,
    EnterShop,
    LeaveShop,
    RerollShop,
    BuyCard { index: usize },
    BuyPack { index: usize },
    BuyVoucher { index: usize },
    PickPack { indices: Vec<usize> },
    SkipPack,
    UseConsumable { index: usize, selected: Vec<usize> },
    SellJoker { index: usize },
    NextBlind,
}

impl AutoAction {
    pub fn stable_key(&self) -> String {
        match self {
            Self::Deal => "deal".to_string(),
            Self::Play { indices } => format!("play:{indices:?}"),
            Self::Discard { indices } => format!("discard:{indices:?}"),
            Self::SkipBlind => "skip_blind".to_string(),
            Self::EnterShop => "enter_shop".to_string(),
            Self::LeaveShop => "leave_shop".to_string(),
            Self::RerollShop => "reroll_shop".to_string(),
            Self::BuyCard { index } => format!("buy_card:{index}"),
            Self::BuyPack { index } => format!("buy_pack:{index}"),
            Self::BuyVoucher { index } => format!("buy_voucher:{index}"),
            Self::PickPack { indices } => format!("pick_pack:{indices:?}"),
            Self::SkipPack => "skip_pack".to_string(),
            Self::UseConsumable { index, selected } => {
                format!("use_consumable:{index}:{selected:?}")
            }
            Self::SellJoker { index } => format!("sell_joker:{index}"),
            Self::NextBlind => "next_blind".to_string(),
        }
    }

    pub fn short_label(&self) -> String {
        match self {
            Self::Deal => "deal".to_string(),
            Self::Play { indices } => format!("play {indices:?}"),
            Self::Discard { indices } => format!("discard {indices:?}"),
            Self::SkipBlind => "skip_blind".to_string(),
            Self::EnterShop => "enter_shop".to_string(),
            Self::LeaveShop => "leave_shop".to_string(),
            Self::RerollShop => "reroll_shop".to_string(),
            Self::BuyCard { index } => format!("buy_card {index}"),
            Self::BuyPack { index } => format!("buy_pack {index}"),
            Self::BuyVoucher { index } => format!("buy_voucher {index}"),
            Self::PickPack { indices } => format!("pick_pack {indices:?}"),
            Self::SkipPack => "skip_pack".to_string(),
            Self::UseConsumable { index, selected } => {
                format!("use_consumable {index} {selected:?}")
            }
            Self::SellJoker { index } => format!("sell_joker {index}"),
            Self::NextBlind => "next_blind".to_string(),
        }
    }
}
