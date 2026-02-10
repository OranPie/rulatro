use crate::api::ModManifest;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
}

#[derive(Debug, Clone)]
pub struct LoadedMod {
    pub manifest: ModManifest,
    pub path: String,
}

pub trait ModLoader {
    type Error;

    fn load_manifest(&self, path: &str) -> Result<LoadedMod, Self::Error>;
}
