use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModContentSpec {
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub meta: ModMetadata,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub content: Option<ModContentSpec>,
    #[serde(default)]
    pub overrides: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    #[serde(default)]
    pub load_order: i32,
}
