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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ModContentSpec, ModManifest, ModMetadata};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rulatro_modding_loader_{}_{}_{}",
            std::process::id(),
            name,
            nanos
        ));
        let _ = fs::create_dir_all(&path);
        path
    }

    fn manifest_for(id: &str) -> ModManifest {
        ModManifest {
            meta: ModMetadata {
                id: id.to_string(),
                name: format!("{} name", id),
                version: "1.0.0".to_string(),
            },
            entry: None,
            content: None,
            overrides: Vec::new(),
            dependencies: Vec::new(),
            load_order: 0,
        }
    }

    macro_rules! valid_id_case {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                assert!(is_valid_id($value));
            }
        };
    }
    valid_id_case!(valid_id_0, "mod_0");
    valid_id_case!(valid_id_1, "mod_1");
    valid_id_case!(valid_id_2, "mod_2");
    valid_id_case!(valid_id_3, "mod_3");
    valid_id_case!(valid_id_4, "mod_4");
    valid_id_case!(valid_id_5, "mod_5");
    valid_id_case!(valid_id_6, "mod_6");
    valid_id_case!(valid_id_7, "mod_7");
    valid_id_case!(valid_id_8, "mod_8");
    valid_id_case!(valid_id_9, "mod_9");
    valid_id_case!(valid_id_10, "mod_10");
    valid_id_case!(valid_id_11, "mod_11");
    valid_id_case!(valid_id_12, "mod_12");
    valid_id_case!(valid_id_13, "mod_13");
    valid_id_case!(valid_id_14, "mod_14");
    valid_id_case!(valid_id_15, "mod_15");
    valid_id_case!(valid_id_16, "mod_16");
    valid_id_case!(valid_id_17, "mod_17");
    valid_id_case!(valid_id_18, "mod_18");
    valid_id_case!(valid_id_19, "mod_19");
    valid_id_case!(valid_id_20, "mod_20");
    valid_id_case!(valid_id_21, "mod_21");
    valid_id_case!(valid_id_22, "mod_22");
    valid_id_case!(valid_id_23, "mod_23");
    valid_id_case!(valid_id_24, "mod_24");
    valid_id_case!(valid_id_25, "mod_25");
    valid_id_case!(valid_id_26, "mod_26");
    valid_id_case!(valid_id_27, "mod_27");
    valid_id_case!(valid_id_28, "mod_28");
    valid_id_case!(valid_id_29, "mod_29");
    valid_id_case!(valid_id_30, "mod_30");
    valid_id_case!(valid_id_31, "mod_31");
    valid_id_case!(valid_id_32, "mod_32");
    valid_id_case!(valid_id_33, "mod_33");
    valid_id_case!(valid_id_34, "mod_34");
    valid_id_case!(valid_id_35, "mod_35");
    valid_id_case!(valid_id_36, "mod_36");
    valid_id_case!(valid_id_37, "mod_37");
    valid_id_case!(valid_id_38, "mod_38");
    valid_id_case!(valid_id_39, "mod_39");
    valid_id_case!(valid_id_40, "mod_40");
    valid_id_case!(valid_id_41, "mod_41");
    valid_id_case!(valid_id_42, "mod_42");
    valid_id_case!(valid_id_43, "mod_43");
    valid_id_case!(valid_id_44, "mod_44");
    valid_id_case!(valid_id_45, "mod_45");
    valid_id_case!(valid_id_46, "mod_46");
    valid_id_case!(valid_id_47, "mod_47");
    valid_id_case!(valid_id_48, "mod_48");
    valid_id_case!(valid_id_49, "mod_49");
    valid_id_case!(valid_id_50, "mod_50");
    valid_id_case!(valid_id_51, "mod_51");
    valid_id_case!(valid_id_52, "mod_52");
    valid_id_case!(valid_id_53, "mod_53");
    valid_id_case!(valid_id_54, "mod_54");
    valid_id_case!(valid_id_55, "mod_55");
    valid_id_case!(valid_id_56, "mod_56");
    valid_id_case!(valid_id_57, "mod_57");
    valid_id_case!(valid_id_58, "mod_58");
    valid_id_case!(valid_id_59, "mod_59");
    valid_id_case!(valid_id_60, "mod_60");
    valid_id_case!(valid_id_61, "mod_61");
    valid_id_case!(valid_id_62, "mod_62");
    valid_id_case!(valid_id_63, "mod_63");
    valid_id_case!(valid_id_64, "mod_64");
    valid_id_case!(valid_id_65, "mod_65");
    valid_id_case!(valid_id_66, "mod_66");
    valid_id_case!(valid_id_67, "mod_67");
    valid_id_case!(valid_id_68, "mod_68");
    valid_id_case!(valid_id_69, "mod_69");
    valid_id_case!(valid_id_70, "mod_70");
    valid_id_case!(valid_id_71, "mod_71");
    valid_id_case!(valid_id_72, "mod_72");
    valid_id_case!(valid_id_73, "mod_73");
    valid_id_case!(valid_id_74, "mod_74");
    valid_id_case!(valid_id_75, "mod_75");
    valid_id_case!(valid_id_76, "mod_76");
    valid_id_case!(valid_id_77, "mod_77");
    valid_id_case!(valid_id_78, "mod_78");
    valid_id_case!(valid_id_79, "mod_79");
    valid_id_case!(valid_id_80, "mod_80");
    valid_id_case!(valid_id_81, "mod_81");
    valid_id_case!(valid_id_82, "mod_82");
    valid_id_case!(valid_id_83, "mod_83");
    valid_id_case!(valid_id_84, "mod_84");
    valid_id_case!(valid_id_85, "mod_85");
    valid_id_case!(valid_id_86, "mod_86");
    valid_id_case!(valid_id_87, "mod_87");
    valid_id_case!(valid_id_88, "mod_88");
    valid_id_case!(valid_id_89, "mod_89");
    valid_id_case!(valid_id_90, "mod_90");
    valid_id_case!(valid_id_91, "mod_91");
    valid_id_case!(valid_id_92, "mod_92");
    valid_id_case!(valid_id_93, "mod_93");
    valid_id_case!(valid_id_94, "mod_94");
    valid_id_case!(valid_id_95, "mod_95");
    valid_id_case!(valid_id_96, "mod_96");
    valid_id_case!(valid_id_97, "mod_97");
    valid_id_case!(valid_id_98, "mod_98");
    valid_id_case!(valid_id_99, "mod_99");
    valid_id_case!(valid_id_100, "mod_100");
    valid_id_case!(valid_id_101, "mod_101");
    valid_id_case!(valid_id_102, "mod_102");
    valid_id_case!(valid_id_103, "mod_103");
    valid_id_case!(valid_id_104, "mod_104");
    valid_id_case!(valid_id_105, "mod_105");
    valid_id_case!(valid_id_106, "mod_106");
    valid_id_case!(valid_id_107, "mod_107");
    valid_id_case!(valid_id_108, "mod_108");
    valid_id_case!(valid_id_109, "mod_109");
    valid_id_case!(valid_id_110, "mod_110");
    valid_id_case!(valid_id_111, "mod_111");
    valid_id_case!(valid_id_112, "mod_112");
    valid_id_case!(valid_id_113, "mod_113");
    valid_id_case!(valid_id_114, "mod_114");
    valid_id_case!(valid_id_115, "mod_115");
    valid_id_case!(valid_id_116, "mod_116");
    valid_id_case!(valid_id_117, "mod_117");
    valid_id_case!(valid_id_118, "mod_118");
    valid_id_case!(valid_id_119, "mod_119");
    valid_id_case!(valid_id_120, "m0-x");
    valid_id_case!(valid_id_121, "m1-x");
    valid_id_case!(valid_id_122, "m2-x");
    valid_id_case!(valid_id_123, "m3-x");
    valid_id_case!(valid_id_124, "m4-x");
    valid_id_case!(valid_id_125, "m5-x");
    valid_id_case!(valid_id_126, "m6-x");
    valid_id_case!(valid_id_127, "m7-x");
    valid_id_case!(valid_id_128, "m8-x");
    valid_id_case!(valid_id_129, "m9-x");
    valid_id_case!(valid_id_130, "m10-x");
    valid_id_case!(valid_id_131, "m11-x");
    valid_id_case!(valid_id_132, "m12-x");
    valid_id_case!(valid_id_133, "m13-x");
    valid_id_case!(valid_id_134, "m14-x");
    valid_id_case!(valid_id_135, "m15-x");
    valid_id_case!(valid_id_136, "m16-x");
    valid_id_case!(valid_id_137, "m17-x");
    valid_id_case!(valid_id_138, "m18-x");
    valid_id_case!(valid_id_139, "m19-x");
    valid_id_case!(valid_id_140, "m20-x");
    valid_id_case!(valid_id_141, "m21-x");
    valid_id_case!(valid_id_142, "m22-x");
    valid_id_case!(valid_id_143, "m23-x");
    valid_id_case!(valid_id_144, "m24-x");
    valid_id_case!(valid_id_145, "m25-x");
    valid_id_case!(valid_id_146, "m26-x");
    valid_id_case!(valid_id_147, "m27-x");
    valid_id_case!(valid_id_148, "m28-x");
    valid_id_case!(valid_id_149, "m29-x");
    valid_id_case!(valid_id_150, "m30-x");
    valid_id_case!(valid_id_151, "m31-x");
    valid_id_case!(valid_id_152, "m32-x");
    valid_id_case!(valid_id_153, "m33-x");
    valid_id_case!(valid_id_154, "m34-x");
    valid_id_case!(valid_id_155, "m35-x");
    valid_id_case!(valid_id_156, "m36-x");
    valid_id_case!(valid_id_157, "m37-x");
    valid_id_case!(valid_id_158, "m38-x");
    valid_id_case!(valid_id_159, "m39-x");

    macro_rules! invalid_id_case {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                assert!(!is_valid_id($value));
            }
        };
    }
    invalid_id_case!(invalid_id_0, "");
    invalid_id_case!(invalid_id_1, "bad id 0");
    invalid_id_case!(invalid_id_2, "bad id 1");
    invalid_id_case!(invalid_id_3, "bad id 2");
    invalid_id_case!(invalid_id_4, "bad id 3");
    invalid_id_case!(invalid_id_5, "bad id 4");
    invalid_id_case!(invalid_id_6, "bad id 5");
    invalid_id_case!(invalid_id_7, "bad id 6");
    invalid_id_case!(invalid_id_8, "bad id 7");
    invalid_id_case!(invalid_id_9, "bad id 8");
    invalid_id_case!(invalid_id_10, "bad id 9");
    invalid_id_case!(invalid_id_11, "bad id 10");
    invalid_id_case!(invalid_id_12, "bad id 11");
    invalid_id_case!(invalid_id_13, "bad id 12");
    invalid_id_case!(invalid_id_14, "bad id 13");
    invalid_id_case!(invalid_id_15, "bad id 14");
    invalid_id_case!(invalid_id_16, "bad id 15");
    invalid_id_case!(invalid_id_17, "bad id 16");
    invalid_id_case!(invalid_id_18, "bad id 17");
    invalid_id_case!(invalid_id_19, "bad id 18");
    invalid_id_case!(invalid_id_20, "bad id 19");
    invalid_id_case!(invalid_id_21, "bad id 20");
    invalid_id_case!(invalid_id_22, "bad id 21");
    invalid_id_case!(invalid_id_23, "bad id 22");
    invalid_id_case!(invalid_id_24, "bad id 23");
    invalid_id_case!(invalid_id_25, "bad id 24");
    invalid_id_case!(invalid_id_26, "bad id 25");
    invalid_id_case!(invalid_id_27, "bad id 26");
    invalid_id_case!(invalid_id_28, "bad id 27");
    invalid_id_case!(invalid_id_29, "bad id 28");
    invalid_id_case!(invalid_id_30, "bad id 29");
    invalid_id_case!(invalid_id_31, "bad id 30");
    invalid_id_case!(invalid_id_32, "bad id 31");
    invalid_id_case!(invalid_id_33, "bad id 32");
    invalid_id_case!(invalid_id_34, "bad id 33");
    invalid_id_case!(invalid_id_35, "bad id 34");
    invalid_id_case!(invalid_id_36, "bad id 35");
    invalid_id_case!(invalid_id_37, "bad id 36");
    invalid_id_case!(invalid_id_38, "bad id 37");
    invalid_id_case!(invalid_id_39, "bad id 38");
    invalid_id_case!(invalid_id_40, "bad id 39");
    invalid_id_case!(invalid_id_41, "bad id 40");
    invalid_id_case!(invalid_id_42, "bad id 41");
    invalid_id_case!(invalid_id_43, "bad id 42");
    invalid_id_case!(invalid_id_44, "bad id 43");
    invalid_id_case!(invalid_id_45, "bad id 44");
    invalid_id_case!(invalid_id_46, "bad id 45");
    invalid_id_case!(invalid_id_47, "bad id 46");
    invalid_id_case!(invalid_id_48, "bad id 47");
    invalid_id_case!(invalid_id_49, "bad id 48");
    invalid_id_case!(invalid_id_50, "bad id 49");
    invalid_id_case!(invalid_id_51, "bad id 50");
    invalid_id_case!(invalid_id_52, "bad id 51");
    invalid_id_case!(invalid_id_53, "bad id 52");
    invalid_id_case!(invalid_id_54, "bad id 53");
    invalid_id_case!(invalid_id_55, "bad id 54");
    invalid_id_case!(invalid_id_56, "bad id 55");
    invalid_id_case!(invalid_id_57, "bad id 56");
    invalid_id_case!(invalid_id_58, "bad id 57");
    invalid_id_case!(invalid_id_59, "bad id 58");
    invalid_id_case!(invalid_id_60, "bad id 59");
    invalid_id_case!(invalid_id_61, "bad/id/0");
    invalid_id_case!(invalid_id_62, "bad/id/1");
    invalid_id_case!(invalid_id_63, "bad/id/2");
    invalid_id_case!(invalid_id_64, "bad/id/3");
    invalid_id_case!(invalid_id_65, "bad/id/4");
    invalid_id_case!(invalid_id_66, "bad/id/5");
    invalid_id_case!(invalid_id_67, "bad/id/6");
    invalid_id_case!(invalid_id_68, "bad/id/7");
    invalid_id_case!(invalid_id_69, "bad/id/8");
    invalid_id_case!(invalid_id_70, "bad/id/9");
    invalid_id_case!(invalid_id_71, "bad/id/10");
    invalid_id_case!(invalid_id_72, "bad/id/11");
    invalid_id_case!(invalid_id_73, "bad/id/12");
    invalid_id_case!(invalid_id_74, "bad/id/13");
    invalid_id_case!(invalid_id_75, "bad/id/14");
    invalid_id_case!(invalid_id_76, "bad/id/15");
    invalid_id_case!(invalid_id_77, "bad/id/16");
    invalid_id_case!(invalid_id_78, "bad/id/17");
    invalid_id_case!(invalid_id_79, "bad/id/18");
    invalid_id_case!(invalid_id_80, "bad/id/19");
    invalid_id_case!(invalid_id_81, "bad/id/20");
    invalid_id_case!(invalid_id_82, "bad/id/21");
    invalid_id_case!(invalid_id_83, "bad/id/22");
    invalid_id_case!(invalid_id_84, "bad/id/23");
    invalid_id_case!(invalid_id_85, "bad/id/24");
    invalid_id_case!(invalid_id_86, "bad/id/25");
    invalid_id_case!(invalid_id_87, "bad/id/26");
    invalid_id_case!(invalid_id_88, "bad/id/27");
    invalid_id_case!(invalid_id_89, "bad/id/28");
    invalid_id_case!(invalid_id_90, "bad/id/29");
    invalid_id_case!(invalid_id_91, "bad*0");
    invalid_id_case!(invalid_id_92, "bad*1");
    invalid_id_case!(invalid_id_93, "bad*2");
    invalid_id_case!(invalid_id_94, "bad*3");
    invalid_id_case!(invalid_id_95, "bad*4");
    invalid_id_case!(invalid_id_96, "bad*5");
    invalid_id_case!(invalid_id_97, "bad*6");
    invalid_id_case!(invalid_id_98, "bad*7");
    invalid_id_case!(invalid_id_99, "bad*8");
    invalid_id_case!(invalid_id_100, "bad*9");
    invalid_id_case!(invalid_id_101, "bad*10");
    invalid_id_case!(invalid_id_102, "bad*11");
    invalid_id_case!(invalid_id_103, "bad*12");
    invalid_id_case!(invalid_id_104, "bad*13");
    invalid_id_case!(invalid_id_105, "bad*14");
    invalid_id_case!(invalid_id_106, "bad*15");
    invalid_id_case!(invalid_id_107, "bad*16");
    invalid_id_case!(invalid_id_108, "bad*17");
    invalid_id_case!(invalid_id_109, "bad*18");
    invalid_id_case!(invalid_id_110, "bad*19");
    invalid_id_case!(invalid_id_111, "bad*20");
    invalid_id_case!(invalid_id_112, "bad*21");
    invalid_id_case!(invalid_id_113, "bad*22");
    invalid_id_case!(invalid_id_114, "bad*23");
    invalid_id_case!(invalid_id_115, "bad*24");
    invalid_id_case!(invalid_id_116, "bad*25");
    invalid_id_case!(invalid_id_117, "bad*26");
    invalid_id_case!(invalid_id_118, "bad*27");
    invalid_id_case!(invalid_id_119, "bad*28");
    invalid_id_case!(invalid_id_120, "bad*29");
    invalid_id_case!(invalid_id_121, "bad.0");
    invalid_id_case!(invalid_id_122, "bad.1");
    invalid_id_case!(invalid_id_123, "bad.2");
    invalid_id_case!(invalid_id_124, "bad.3");
    invalid_id_case!(invalid_id_125, "bad.4");
    invalid_id_case!(invalid_id_126, "bad.5");
    invalid_id_case!(invalid_id_127, "bad.6");
    invalid_id_case!(invalid_id_128, "bad.7");
    invalid_id_case!(invalid_id_129, "bad.8");
    invalid_id_case!(invalid_id_130, "bad.9");
    invalid_id_case!(invalid_id_131, "bad.10");
    invalid_id_case!(invalid_id_132, "bad.11");
    invalid_id_case!(invalid_id_133, "bad.12");
    invalid_id_case!(invalid_id_134, "bad.13");
    invalid_id_case!(invalid_id_135, "bad.14");
    invalid_id_case!(invalid_id_136, "bad.15");
    invalid_id_case!(invalid_id_137, "bad.16");
    invalid_id_case!(invalid_id_138, "bad.17");
    invalid_id_case!(invalid_id_139, "bad.18");
    invalid_id_case!(invalid_id_140, "bad.19");
    invalid_id_case!(invalid_id_141, "bad.20");
    invalid_id_case!(invalid_id_142, "bad.21");
    invalid_id_case!(invalid_id_143, "bad.22");
    invalid_id_case!(invalid_id_144, "bad.23");
    invalid_id_case!(invalid_id_145, "bad.24");
    invalid_id_case!(invalid_id_146, "bad.25");
    invalid_id_case!(invalid_id_147, "bad.26");
    invalid_id_case!(invalid_id_148, "bad.27");
    invalid_id_case!(invalid_id_149, "bad.28");
    invalid_id_case!(invalid_id_150, "bad.29");

    macro_rules! safe_path_case {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                assert!(is_safe_relative_path($value));
            }
        };
    }
    safe_path_case!(safe_path_0, "entry_0.lua");
    safe_path_case!(safe_path_1, "entry_1.lua");
    safe_path_case!(safe_path_2, "entry_2.lua");
    safe_path_case!(safe_path_3, "entry_3.lua");
    safe_path_case!(safe_path_4, "entry_4.lua");
    safe_path_case!(safe_path_5, "entry_5.lua");
    safe_path_case!(safe_path_6, "entry_6.lua");
    safe_path_case!(safe_path_7, "entry_7.lua");
    safe_path_case!(safe_path_8, "entry_8.lua");
    safe_path_case!(safe_path_9, "entry_9.lua");
    safe_path_case!(safe_path_10, "entry_10.lua");
    safe_path_case!(safe_path_11, "entry_11.lua");
    safe_path_case!(safe_path_12, "entry_12.lua");
    safe_path_case!(safe_path_13, "entry_13.lua");
    safe_path_case!(safe_path_14, "entry_14.lua");
    safe_path_case!(safe_path_15, "entry_15.lua");
    safe_path_case!(safe_path_16, "entry_16.lua");
    safe_path_case!(safe_path_17, "entry_17.lua");
    safe_path_case!(safe_path_18, "entry_18.lua");
    safe_path_case!(safe_path_19, "entry_19.lua");
    safe_path_case!(safe_path_20, "entry_20.lua");
    safe_path_case!(safe_path_21, "entry_21.lua");
    safe_path_case!(safe_path_22, "entry_22.lua");
    safe_path_case!(safe_path_23, "entry_23.lua");
    safe_path_case!(safe_path_24, "entry_24.lua");
    safe_path_case!(safe_path_25, "entry_25.lua");
    safe_path_case!(safe_path_26, "entry_26.lua");
    safe_path_case!(safe_path_27, "entry_27.lua");
    safe_path_case!(safe_path_28, "entry_28.lua");
    safe_path_case!(safe_path_29, "entry_29.lua");
    safe_path_case!(safe_path_30, "entry_30.lua");
    safe_path_case!(safe_path_31, "entry_31.lua");
    safe_path_case!(safe_path_32, "entry_32.lua");
    safe_path_case!(safe_path_33, "entry_33.lua");
    safe_path_case!(safe_path_34, "entry_34.lua");
    safe_path_case!(safe_path_35, "entry_35.lua");
    safe_path_case!(safe_path_36, "entry_36.lua");
    safe_path_case!(safe_path_37, "entry_37.lua");
    safe_path_case!(safe_path_38, "entry_38.lua");
    safe_path_case!(safe_path_39, "entry_39.lua");
    safe_path_case!(safe_path_40, "entry_40.lua");
    safe_path_case!(safe_path_41, "entry_41.lua");
    safe_path_case!(safe_path_42, "entry_42.lua");
    safe_path_case!(safe_path_43, "entry_43.lua");
    safe_path_case!(safe_path_44, "entry_44.lua");
    safe_path_case!(safe_path_45, "entry_45.lua");
    safe_path_case!(safe_path_46, "entry_46.lua");
    safe_path_case!(safe_path_47, "entry_47.lua");
    safe_path_case!(safe_path_48, "entry_48.lua");
    safe_path_case!(safe_path_49, "entry_49.lua");
    safe_path_case!(safe_path_50, "entry_50.lua");
    safe_path_case!(safe_path_51, "entry_51.lua");
    safe_path_case!(safe_path_52, "entry_52.lua");
    safe_path_case!(safe_path_53, "entry_53.lua");
    safe_path_case!(safe_path_54, "entry_54.lua");
    safe_path_case!(safe_path_55, "entry_55.lua");
    safe_path_case!(safe_path_56, "entry_56.lua");
    safe_path_case!(safe_path_57, "entry_57.lua");
    safe_path_case!(safe_path_58, "entry_58.lua");
    safe_path_case!(safe_path_59, "entry_59.lua");
    safe_path_case!(safe_path_60, "entry_60.lua");
    safe_path_case!(safe_path_61, "entry_61.lua");
    safe_path_case!(safe_path_62, "entry_62.lua");
    safe_path_case!(safe_path_63, "entry_63.lua");
    safe_path_case!(safe_path_64, "entry_64.lua");
    safe_path_case!(safe_path_65, "entry_65.lua");
    safe_path_case!(safe_path_66, "entry_66.lua");
    safe_path_case!(safe_path_67, "entry_67.lua");
    safe_path_case!(safe_path_68, "entry_68.lua");
    safe_path_case!(safe_path_69, "entry_69.lua");
    safe_path_case!(safe_path_70, "entry_70.lua");
    safe_path_case!(safe_path_71, "entry_71.lua");
    safe_path_case!(safe_path_72, "entry_72.lua");
    safe_path_case!(safe_path_73, "entry_73.lua");
    safe_path_case!(safe_path_74, "entry_74.lua");
    safe_path_case!(safe_path_75, "entry_75.lua");
    safe_path_case!(safe_path_76, "entry_76.lua");
    safe_path_case!(safe_path_77, "entry_77.lua");
    safe_path_case!(safe_path_78, "entry_78.lua");
    safe_path_case!(safe_path_79, "entry_79.lua");
    safe_path_case!(safe_path_80, "scripts/mod_0.lua");
    safe_path_case!(safe_path_81, "scripts/mod_1.lua");
    safe_path_case!(safe_path_82, "scripts/mod_2.lua");
    safe_path_case!(safe_path_83, "scripts/mod_3.lua");
    safe_path_case!(safe_path_84, "scripts/mod_4.lua");
    safe_path_case!(safe_path_85, "scripts/mod_5.lua");
    safe_path_case!(safe_path_86, "scripts/mod_6.lua");
    safe_path_case!(safe_path_87, "scripts/mod_7.lua");
    safe_path_case!(safe_path_88, "scripts/mod_8.lua");
    safe_path_case!(safe_path_89, "scripts/mod_9.lua");
    safe_path_case!(safe_path_90, "scripts/mod_10.lua");
    safe_path_case!(safe_path_91, "scripts/mod_11.lua");
    safe_path_case!(safe_path_92, "scripts/mod_12.lua");
    safe_path_case!(safe_path_93, "scripts/mod_13.lua");
    safe_path_case!(safe_path_94, "scripts/mod_14.lua");
    safe_path_case!(safe_path_95, "scripts/mod_15.lua");
    safe_path_case!(safe_path_96, "scripts/mod_16.lua");
    safe_path_case!(safe_path_97, "scripts/mod_17.lua");
    safe_path_case!(safe_path_98, "scripts/mod_18.lua");
    safe_path_case!(safe_path_99, "scripts/mod_19.lua");
    safe_path_case!(safe_path_100, "scripts/mod_20.lua");
    safe_path_case!(safe_path_101, "scripts/mod_21.lua");
    safe_path_case!(safe_path_102, "scripts/mod_22.lua");
    safe_path_case!(safe_path_103, "scripts/mod_23.lua");
    safe_path_case!(safe_path_104, "scripts/mod_24.lua");
    safe_path_case!(safe_path_105, "scripts/mod_25.lua");
    safe_path_case!(safe_path_106, "scripts/mod_26.lua");
    safe_path_case!(safe_path_107, "scripts/mod_27.lua");
    safe_path_case!(safe_path_108, "scripts/mod_28.lua");
    safe_path_case!(safe_path_109, "scripts/mod_29.lua");
    safe_path_case!(safe_path_110, "scripts/mod_30.lua");
    safe_path_case!(safe_path_111, "scripts/mod_31.lua");
    safe_path_case!(safe_path_112, "scripts/mod_32.lua");
    safe_path_case!(safe_path_113, "scripts/mod_33.lua");
    safe_path_case!(safe_path_114, "scripts/mod_34.lua");
    safe_path_case!(safe_path_115, "scripts/mod_35.lua");
    safe_path_case!(safe_path_116, "scripts/mod_36.lua");
    safe_path_case!(safe_path_117, "scripts/mod_37.lua");
    safe_path_case!(safe_path_118, "scripts/mod_38.lua");
    safe_path_case!(safe_path_119, "scripts/mod_39.lua");
    safe_path_case!(safe_path_120, "content/root_0");
    safe_path_case!(safe_path_121, "content/root_1");
    safe_path_case!(safe_path_122, "content/root_2");
    safe_path_case!(safe_path_123, "content/root_3");
    safe_path_case!(safe_path_124, "content/root_4");
    safe_path_case!(safe_path_125, "content/root_5");
    safe_path_case!(safe_path_126, "content/root_6");
    safe_path_case!(safe_path_127, "content/root_7");
    safe_path_case!(safe_path_128, "content/root_8");
    safe_path_case!(safe_path_129, "content/root_9");
    safe_path_case!(safe_path_130, "content/root_10");
    safe_path_case!(safe_path_131, "content/root_11");
    safe_path_case!(safe_path_132, "content/root_12");
    safe_path_case!(safe_path_133, "content/root_13");
    safe_path_case!(safe_path_134, "content/root_14");
    safe_path_case!(safe_path_135, "content/root_15");
    safe_path_case!(safe_path_136, "content/root_16");
    safe_path_case!(safe_path_137, "content/root_17");
    safe_path_case!(safe_path_138, "content/root_18");
    safe_path_case!(safe_path_139, "content/root_19");
    safe_path_case!(safe_path_140, "content/root_20");
    safe_path_case!(safe_path_141, "content/root_21");
    safe_path_case!(safe_path_142, "content/root_22");
    safe_path_case!(safe_path_143, "content/root_23");
    safe_path_case!(safe_path_144, "content/root_24");
    safe_path_case!(safe_path_145, "content/root_25");
    safe_path_case!(safe_path_146, "content/root_26");
    safe_path_case!(safe_path_147, "content/root_27");
    safe_path_case!(safe_path_148, "content/root_28");
    safe_path_case!(safe_path_149, "content/root_29");
    safe_path_case!(safe_path_150, "content/root_30");
    safe_path_case!(safe_path_151, "content/root_31");
    safe_path_case!(safe_path_152, "content/root_32");
    safe_path_case!(safe_path_153, "content/root_33");
    safe_path_case!(safe_path_154, "content/root_34");
    safe_path_case!(safe_path_155, "content/root_35");
    safe_path_case!(safe_path_156, "content/root_36");
    safe_path_case!(safe_path_157, "content/root_37");
    safe_path_case!(safe_path_158, "content/root_38");
    safe_path_case!(safe_path_159, "content/root_39");

    macro_rules! unsafe_path_case {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                assert!(!is_safe_relative_path($value));
            }
        };
    }
    unsafe_path_case!(unsafe_path_0, "../escape_0");
    unsafe_path_case!(unsafe_path_1, "../escape_1");
    unsafe_path_case!(unsafe_path_2, "../escape_2");
    unsafe_path_case!(unsafe_path_3, "../escape_3");
    unsafe_path_case!(unsafe_path_4, "../escape_4");
    unsafe_path_case!(unsafe_path_5, "../escape_5");
    unsafe_path_case!(unsafe_path_6, "../escape_6");
    unsafe_path_case!(unsafe_path_7, "../escape_7");
    unsafe_path_case!(unsafe_path_8, "../escape_8");
    unsafe_path_case!(unsafe_path_9, "../escape_9");
    unsafe_path_case!(unsafe_path_10, "../escape_10");
    unsafe_path_case!(unsafe_path_11, "../escape_11");
    unsafe_path_case!(unsafe_path_12, "../escape_12");
    unsafe_path_case!(unsafe_path_13, "../escape_13");
    unsafe_path_case!(unsafe_path_14, "../escape_14");
    unsafe_path_case!(unsafe_path_15, "../escape_15");
    unsafe_path_case!(unsafe_path_16, "../escape_16");
    unsafe_path_case!(unsafe_path_17, "../escape_17");
    unsafe_path_case!(unsafe_path_18, "../escape_18");
    unsafe_path_case!(unsafe_path_19, "../escape_19");
    unsafe_path_case!(unsafe_path_20, "../escape_20");
    unsafe_path_case!(unsafe_path_21, "../escape_21");
    unsafe_path_case!(unsafe_path_22, "../escape_22");
    unsafe_path_case!(unsafe_path_23, "../escape_23");
    unsafe_path_case!(unsafe_path_24, "../escape_24");
    unsafe_path_case!(unsafe_path_25, "../escape_25");
    unsafe_path_case!(unsafe_path_26, "../escape_26");
    unsafe_path_case!(unsafe_path_27, "../escape_27");
    unsafe_path_case!(unsafe_path_28, "../escape_28");
    unsafe_path_case!(unsafe_path_29, "../escape_29");
    unsafe_path_case!(unsafe_path_30, "../escape_30");
    unsafe_path_case!(unsafe_path_31, "../escape_31");
    unsafe_path_case!(unsafe_path_32, "../escape_32");
    unsafe_path_case!(unsafe_path_33, "../escape_33");
    unsafe_path_case!(unsafe_path_34, "../escape_34");
    unsafe_path_case!(unsafe_path_35, "../escape_35");
    unsafe_path_case!(unsafe_path_36, "../escape_36");
    unsafe_path_case!(unsafe_path_37, "../escape_37");
    unsafe_path_case!(unsafe_path_38, "../escape_38");
    unsafe_path_case!(unsafe_path_39, "../escape_39");
    unsafe_path_case!(unsafe_path_40, "../escape_40");
    unsafe_path_case!(unsafe_path_41, "../escape_41");
    unsafe_path_case!(unsafe_path_42, "../escape_42");
    unsafe_path_case!(unsafe_path_43, "../escape_43");
    unsafe_path_case!(unsafe_path_44, "../escape_44");
    unsafe_path_case!(unsafe_path_45, "../escape_45");
    unsafe_path_case!(unsafe_path_46, "../escape_46");
    unsafe_path_case!(unsafe_path_47, "../escape_47");
    unsafe_path_case!(unsafe_path_48, "../escape_48");
    unsafe_path_case!(unsafe_path_49, "../escape_49");
    unsafe_path_case!(unsafe_path_50, "/abs/path/0");
    unsafe_path_case!(unsafe_path_51, "/abs/path/1");
    unsafe_path_case!(unsafe_path_52, "/abs/path/2");
    unsafe_path_case!(unsafe_path_53, "/abs/path/3");
    unsafe_path_case!(unsafe_path_54, "/abs/path/4");
    unsafe_path_case!(unsafe_path_55, "/abs/path/5");
    unsafe_path_case!(unsafe_path_56, "/abs/path/6");
    unsafe_path_case!(unsafe_path_57, "/abs/path/7");
    unsafe_path_case!(unsafe_path_58, "/abs/path/8");
    unsafe_path_case!(unsafe_path_59, "/abs/path/9");
    unsafe_path_case!(unsafe_path_60, "/abs/path/10");
    unsafe_path_case!(unsafe_path_61, "/abs/path/11");
    unsafe_path_case!(unsafe_path_62, "/abs/path/12");
    unsafe_path_case!(unsafe_path_63, "/abs/path/13");
    unsafe_path_case!(unsafe_path_64, "/abs/path/14");
    unsafe_path_case!(unsafe_path_65, "/abs/path/15");
    unsafe_path_case!(unsafe_path_66, "/abs/path/16");
    unsafe_path_case!(unsafe_path_67, "/abs/path/17");
    unsafe_path_case!(unsafe_path_68, "/abs/path/18");
    unsafe_path_case!(unsafe_path_69, "/abs/path/19");
    unsafe_path_case!(unsafe_path_70, "/abs/path/20");
    unsafe_path_case!(unsafe_path_71, "/abs/path/21");
    unsafe_path_case!(unsafe_path_72, "/abs/path/22");
    unsafe_path_case!(unsafe_path_73, "/abs/path/23");
    unsafe_path_case!(unsafe_path_74, "/abs/path/24");
    unsafe_path_case!(unsafe_path_75, "/abs/path/25");
    unsafe_path_case!(unsafe_path_76, "/abs/path/26");
    unsafe_path_case!(unsafe_path_77, "/abs/path/27");
    unsafe_path_case!(unsafe_path_78, "/abs/path/28");
    unsafe_path_case!(unsafe_path_79, "/abs/path/29");
    unsafe_path_case!(unsafe_path_80, "/abs/path/30");
    unsafe_path_case!(unsafe_path_81, "/abs/path/31");
    unsafe_path_case!(unsafe_path_82, "/abs/path/32");
    unsafe_path_case!(unsafe_path_83, "/abs/path/33");
    unsafe_path_case!(unsafe_path_84, "/abs/path/34");
    unsafe_path_case!(unsafe_path_85, "/abs/path/35");
    unsafe_path_case!(unsafe_path_86, "/abs/path/36");
    unsafe_path_case!(unsafe_path_87, "/abs/path/37");
    unsafe_path_case!(unsafe_path_88, "/abs/path/38");
    unsafe_path_case!(unsafe_path_89, "/abs/path/39");
    unsafe_path_case!(unsafe_path_90, "/abs/path/40");
    unsafe_path_case!(unsafe_path_91, "/abs/path/41");
    unsafe_path_case!(unsafe_path_92, "/abs/path/42");
    unsafe_path_case!(unsafe_path_93, "/abs/path/43");
    unsafe_path_case!(unsafe_path_94, "/abs/path/44");
    unsafe_path_case!(unsafe_path_95, "/abs/path/45");
    unsafe_path_case!(unsafe_path_96, "/abs/path/46");
    unsafe_path_case!(unsafe_path_97, "/abs/path/47");
    unsafe_path_case!(unsafe_path_98, "/abs/path/48");
    unsafe_path_case!(unsafe_path_99, "/abs/path/49");
    unsafe_path_case!(unsafe_path_100, "mods/../../bad_0");
    unsafe_path_case!(unsafe_path_101, "mods/../../bad_1");
    unsafe_path_case!(unsafe_path_102, "mods/../../bad_2");
    unsafe_path_case!(unsafe_path_103, "mods/../../bad_3");
    unsafe_path_case!(unsafe_path_104, "mods/../../bad_4");
    unsafe_path_case!(unsafe_path_105, "mods/../../bad_5");
    unsafe_path_case!(unsafe_path_106, "mods/../../bad_6");
    unsafe_path_case!(unsafe_path_107, "mods/../../bad_7");
    unsafe_path_case!(unsafe_path_108, "mods/../../bad_8");
    unsafe_path_case!(unsafe_path_109, "mods/../../bad_9");
    unsafe_path_case!(unsafe_path_110, "mods/../../bad_10");
    unsafe_path_case!(unsafe_path_111, "mods/../../bad_11");
    unsafe_path_case!(unsafe_path_112, "mods/../../bad_12");
    unsafe_path_case!(unsafe_path_113, "mods/../../bad_13");
    unsafe_path_case!(unsafe_path_114, "mods/../../bad_14");
    unsafe_path_case!(unsafe_path_115, "mods/../../bad_15");
    unsafe_path_case!(unsafe_path_116, "mods/../../bad_16");
    unsafe_path_case!(unsafe_path_117, "mods/../../bad_17");
    unsafe_path_case!(unsafe_path_118, "mods/../../bad_18");
    unsafe_path_case!(unsafe_path_119, "mods/../../bad_19");
    unsafe_path_case!(unsafe_path_120, "mods/../../bad_20");
    unsafe_path_case!(unsafe_path_121, "mods/../../bad_21");
    unsafe_path_case!(unsafe_path_122, "mods/../../bad_22");
    unsafe_path_case!(unsafe_path_123, "mods/../../bad_23");
    unsafe_path_case!(unsafe_path_124, "mods/../../bad_24");
    unsafe_path_case!(unsafe_path_125, "mods/../../bad_25");
    unsafe_path_case!(unsafe_path_126, "mods/../../bad_26");
    unsafe_path_case!(unsafe_path_127, "mods/../../bad_27");
    unsafe_path_case!(unsafe_path_128, "mods/../../bad_28");
    unsafe_path_case!(unsafe_path_129, "mods/../../bad_29");

    #[test]
    fn validate_manifest_rejects_mismatched_directory() {
        let root = temp_root("mismatch");
        let mod_dir = root.join("wrong_name");
        fs::create_dir_all(&mod_dir).expect("mkdir");
        let manifest = manifest_for("expected_name");
        let err = validate_manifest(&mod_dir, &manifest).expect_err("should fail");
        assert!(matches!(err, ModError::InvalidManifest(_)));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn validate_manifest_accepts_existing_entry_and_content() {
        let root = temp_root("ok");
        let mod_dir = root.join("mod_ok");
        let scripts = mod_dir.join("scripts");
        let content = mod_dir.join("content");
        fs::create_dir_all(&scripts).expect("mkdir scripts");
        fs::create_dir_all(&content).expect("mkdir content");
        fs::write(scripts.join("main.lua"), "return true").expect("write entry");
        let mut manifest = manifest_for("mod_ok");
        manifest.entry = Some("scripts/main.lua".to_string());
        manifest.content = Some(ModContentSpec {
            root: "content".to_string(),
        });
        validate_manifest(&mod_dir, &manifest).expect("validate");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn load_all_handles_missing_root() {
        let root = temp_root("missing_root");
        let _ = fs::remove_dir_all(&root);
        let loader = FileSystemModLoader::new(&root);
        let loaded = loader.load_all().expect("load");
        assert!(loaded.is_empty());
    }

    #[test]
    fn load_all_detects_duplicate_mod_ids() {
        let root = temp_root("dup");
        let mod_a = root.join("same_a");
        let mod_b = root.join("same_b");
        fs::create_dir_all(&mod_a).expect("mkdir a");
        fs::create_dir_all(&mod_b).expect("mkdir b");
        fs::write(
            mod_a.join("mod.json"),
            r#"{"meta":{"id":"same","name":"A","version":"1"},"load_order":0}"#,
        )
        .expect("write a");
        fs::write(
            mod_b.join("mod.json"),
            r#"{"meta":{"id":"same","name":"B","version":"1"},"load_order":1}"#,
        )
        .expect("write b");
        let loader = FileSystemModLoader::new(&root);
        let err = loader.load_all().expect_err("duplicate should fail");
        assert!(matches!(
            err,
            ModError::InvalidManifest(_) | ModError::DuplicateMod(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn load_all_missing_dependency_fails() {
        let root = temp_root("dep");
        let mod_a = root.join("mod_a");
        fs::create_dir_all(&mod_a).expect("mkdir a");
        fs::write(mod_a.join("mod.json"), r#"{"meta":{"id":"mod_a","name":"A","version":"1"},"dependencies":[{"id":"missing_mod"}]}"#).expect("write");
        let loader = FileSystemModLoader::new(&root);
        let err = loader.load_all().expect_err("dependency should fail");
        assert!(matches!(err, ModError::MissingDependency(_)));
        let _ = fs::remove_dir_all(root);
    }
}
