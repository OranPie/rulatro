use crate::schema::{
    AnteRule, BlindRule, ConsumableDef, Content, ContentPack, EconomyRule, GameConfig, HandRule,
    RankRule, ShopRule,
};
use crate::joker_dsl::load_jokers_dsl;
use anyhow::Context;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

pub fn load_content_pack(path: &Path) -> anyhow::Result<ContentPack> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pack = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(pack)
}

pub fn load_game_config(dir: &Path) -> anyhow::Result<GameConfig> {
    let hands: Vec<HandRule> = load_json(dir.join("hands.json"))?;
    let ranks: Vec<RankRule> = load_json(dir.join("ranks.json"))?;
    let blinds: Vec<BlindRule> = load_json(dir.join("blinds.json"))?;
    let antes: Vec<AnteRule> = load_json(dir.join("antes.json"))?;
    let economy: EconomyRule = load_json(dir.join("economy.json"))?;
    let shop: ShopRule = load_json(dir.join("shop.json"))?;

    Ok(GameConfig {
        hands,
        ranks,
        blinds,
        antes,
        economy,
        shop,
    })
}

pub fn load_content(dir: &Path) -> anyhow::Result<Content> {
    let base = dir.join("content");
    let jokers_path = base.join("jokers.dsl");
    let jokers = load_jokers_dsl(&jokers_path)
        .with_context(|| format!("parse {}", jokers_path.display()))?;
    let tarots: Vec<ConsumableDef> = load_json(base.join("tarots.json"))?;
    let planets: Vec<ConsumableDef> = load_json(base.join("planets.json"))?;
    let spectrals: Vec<ConsumableDef> = load_json(base.join("spectrals.json"))?;

    Ok(Content {
        jokers,
        tarots,
        planets,
        spectrals,
    })
}

fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(value)
}
