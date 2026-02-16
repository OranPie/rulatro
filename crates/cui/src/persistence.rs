use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const SAVE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAction {
    pub action: String,
    #[serde(default)]
    pub indices: Vec<usize>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRunState {
    pub version: u32,
    pub locale: String,
    pub seed: u64,
    #[serde(default)]
    pub content_signature: String,
    pub actions: Vec<SavedAction>,
}

#[derive(Debug, Clone)]
pub struct AutoPerformScript {
    pub locale: Option<String>,
    pub seed: Option<u64>,
    pub actions: Vec<SavedAction>,
}

#[derive(Debug, Clone, Deserialize)]
struct AutoPerformScriptFile {
    #[serde(default)]
    locale: Option<String>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    actions: Vec<SavedAction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AutoPerformPayload {
    SavedRunState(SavedRunState),
    Script(AutoPerformScriptFile),
    Actions(Vec<SavedAction>),
}

pub fn default_state_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RULATRO_SAVE") {
        return Some(PathBuf::from(path));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".rulatro_cli_state.json"))
}

pub fn save_state_file(
    locale_code: &str,
    seed: u64,
    content_signature: &str,
    actions: &[SavedAction],
    path: &Path,
) -> Result<(), String> {
    let payload = SavedRunState {
        version: SAVE_SCHEMA_VERSION,
        locale: locale_code.to_string(),
        seed,
        content_signature: content_signature.to_string(),
        actions: actions.to_vec(),
    };
    let body = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    fs::write(path, body).map_err(|err| err.to_string())
}

pub fn load_state_file(path: &Path) -> Result<SavedRunState, String> {
    let body = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let payload: SavedRunState = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    if payload.version != SAVE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported save version {} (expected {})",
            payload.version, SAVE_SCHEMA_VERSION
        ));
    }
    Ok(payload)
}

pub fn load_auto_perform_file(path: &Path) -> Result<AutoPerformScript, String> {
    let body = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let payload: AutoPerformPayload = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    let script = match payload {
        AutoPerformPayload::SavedRunState(saved) => {
            if saved.version != SAVE_SCHEMA_VERSION {
                return Err(format!(
                    "unsupported save version {} (expected {})",
                    saved.version, SAVE_SCHEMA_VERSION
                ));
            }
            AutoPerformScript {
                locale: Some(saved.locale),
                seed: Some(saved.seed),
                actions: saved.actions,
            }
        }
        AutoPerformPayload::Script(script) => AutoPerformScript {
            locale: script.locale,
            seed: script.seed,
            actions: script.actions,
        },
        AutoPerformPayload::Actions(actions) => AutoPerformScript {
            locale: None,
            seed: None,
            actions,
        },
    };
    Ok(script)
}

#[derive(Clone, Copy)]
struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

fn hash_dir_tree(base: &Path, rel: &Path, hasher: &mut Fnv64) -> Result<(), String> {
    let path = base.join(rel);
    if !path.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(&path)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_name = entry.file_name();
        let rel_path = if rel.as_os_str().is_empty() {
            PathBuf::from(&file_name)
        } else {
            rel.join(&file_name)
        };
        let entry_path = entry.path();
        if entry_path.is_dir() {
            hasher.update(b"D");
            hasher.update(rel_path.to_string_lossy().as_bytes());
            hasher.update(&[0]);
            hash_dir_tree(base, &rel_path, hasher)?;
        } else if entry_path.is_file() {
            hasher.update(b"F");
            hasher.update(rel_path.to_string_lossy().as_bytes());
            hasher.update(&[0]);
            let bytes = fs::read(&entry_path).map_err(|err| err.to_string())?;
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        }
    }
    Ok(())
}

pub fn compute_content_signature(locale_code: &str) -> Result<String, String> {
    let mut hasher = Fnv64::new();
    hasher.update(b"rulatro-save-signature-v1");
    hasher.update(locale_code.as_bytes());
    hash_dir_tree(Path::new("assets"), Path::new(""), &mut hasher)?;
    hash_dir_tree(Path::new("mods"), Path::new(""), &mut hasher)?;
    Ok(format!("{:016x}", hasher.finish()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn save_load_roundtrip() {
        let file = unique_temp_file();
        let actions = vec![
            SavedAction {
                action: "deal".to_string(),
                indices: Vec::new(),
                target: None,
            },
            SavedAction {
                action: "play".to_string(),
                indices: vec![0, 2, 4],
                target: None,
            },
        ];
        save_state_file("en_US", 42, "abc123", &actions, &file).expect("save");
        let loaded = load_state_file(&file).expect("load");
        assert_eq!(loaded.version, SAVE_SCHEMA_VERSION);
        assert_eq!(loaded.locale, "en_US");
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.content_signature, "abc123");
        assert_eq!(loaded.actions.len(), actions.len());
        assert_eq!(loaded.actions[1].indices, vec![0, 2, 4]);
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn load_auto_perform_from_actions_array() {
        let file = unique_temp_file();
        let body = r#"
[
  {"action":"deal"},
  {"action":"play","indices":[0,1,2]}
]
"#;
        std::fs::write(&file, body).expect("write");
        let loaded = load_auto_perform_file(&file).expect("load auto");
        assert_eq!(loaded.seed, None);
        assert_eq!(loaded.locale, None);
        assert_eq!(loaded.actions.len(), 2);
        assert_eq!(loaded.actions[1].indices, vec![0, 1, 2]);
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn load_auto_perform_from_script_object() {
        let file = unique_temp_file();
        let body = r#"
{
  "locale":"zh_CN",
  "seed":99,
  "actions":[{"action":"deal"}]
}
"#;
        std::fs::write(&file, body).expect("write");
        let loaded = load_auto_perform_file(&file).expect("load auto");
        assert_eq!(loaded.seed, Some(99));
        assert_eq!(loaded.locale.as_deref(), Some("zh_CN"));
        assert_eq!(loaded.actions.len(), 1);
        let _ = std::fs::remove_file(file);
    }

    fn unique_temp_file() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "rulatro_cui_persistence_test_{}_{}.json",
            std::process::id(),
            nanos
        ))
    }
}
