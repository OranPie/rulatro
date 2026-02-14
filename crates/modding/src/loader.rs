use crate::api::ModManifest;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("invalid mod id: {0}")]
    InvalidId(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("missing dependency: {0}")]
    MissingDependency(String),
    #[error("duplicate mod id: {0}")]
    DuplicateMod(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("runtime unavailable: {0}")]
    RuntimeUnavailable(String),
}

#[derive(Debug, Clone)]
pub struct LoadedMod {
    pub manifest: ModManifest,
    pub root: PathBuf,
}

pub struct FileSystemModLoader {
    root: PathBuf,
}

impl FileSystemModLoader {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load_all(&self) -> Result<Vec<LoadedMod>, ModError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut mods = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("mod.json");
            if !manifest_path.exists() {
                continue;
            }
            let manifest = load_manifest(&manifest_path)?;
            validate_manifest(&path, &manifest)?;
            mods.push(LoadedMod {
                manifest,
                root: path,
            });
        }
        mods.sort_by(|a, b| {
            (a.manifest.load_order, a.manifest.meta.id.clone())
                .cmp(&(b.manifest.load_order, b.manifest.meta.id.clone()))
        });
        let mut seen = HashSet::new();
        for item in &mods {
            if !seen.insert(item.manifest.meta.id.clone()) {
                return Err(ModError::DuplicateMod(item.manifest.meta.id.clone()));
            }
        }
        let ids: HashSet<String> = mods
            .iter()
            .map(|item| item.manifest.meta.id.clone())
            .collect();
        for item in &mods {
            for dep in &item.manifest.dependencies {
                if !ids.contains(&dep.id) {
                    return Err(ModError::MissingDependency(format!(
                        "{} requires {}",
                        item.manifest.meta.id, dep.id
                    )));
                }
            }
        }
        Ok(mods)
    }
}

fn load_manifest(path: &Path) -> Result<ModManifest, ModError> {
    let raw = fs::read_to_string(path)?;
    let manifest: ModManifest =
        serde_json::from_str(&raw).map_err(|err| ModError::InvalidManifest(err.to_string()))?;
    Ok(manifest)
}

fn validate_manifest(root: &Path, manifest: &ModManifest) -> Result<(), ModError> {
    let id = manifest.meta.id.trim();
    if !is_valid_id(id) {
        return Err(ModError::InvalidId(id.to_string()));
    }
    if let Some(dir_name) = root.file_name().and_then(|name| name.to_str()) {
        if dir_name != id {
            return Err(ModError::InvalidManifest(format!(
                "mod id {} does not match directory {}",
                id, dir_name
            )));
        }
    }
    if let Some(entry) = manifest.entry.as_ref() {
        if !is_safe_relative_path(entry) {
            return Err(ModError::InvalidPath(entry.to_string()));
        }
        let entry_path = root.join(entry);
        if !entry_path.exists() {
            return Err(ModError::InvalidManifest(format!(
                "missing entry {}",
                entry
            )));
        }
    }
    if let Some(content) = manifest.content.as_ref() {
        if !is_safe_relative_path(&content.root) {
            return Err(ModError::InvalidPath(content.root.clone()));
        }
        let content_path = root.join(&content.root);
        if !content_path.exists() {
            return Err(ModError::InvalidManifest(format!(
                "missing content root {}",
                content.root
            )));
        }
    }
    Ok(())
}

fn is_valid_id(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn is_safe_relative_path(path: &str) -> bool {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return false;
    }
    for component in candidate.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    true
}
