use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub meta: ModMetadata,
    pub entry: String,
}

/// Runtime-agnostic mod API surface. Implementations (Lua, WASM, etc.) live later.
pub trait ModRuntime {
    type Error;

    fn load_entry(&mut self, manifest: &ModManifest) -> Result<(), Self::Error>;
}
