use crate::{LoadedMod, ModError};
use rulatro_core::{ModHookContext, ModHookResult};

pub struct WasmRuntime {
    _mods: Vec<LoadedMod>,
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self { _mods: Vec::new() }
    }

    pub fn load_mod(&mut self, item: &LoadedMod) -> Result<(), ModError> {
        let entry = item
            .manifest
            .entry
            .as_ref()
            .map(|value| value.as_str())
            .unwrap_or("unknown");
        Err(ModError::RuntimeUnavailable(format!(
            "wasm runtime not implemented ({})",
            entry
        )))
    }

    pub fn on_hook(&mut self, _ctx: &ModHookContext<'_>) -> ModHookResult {
        ModHookResult::default()
    }
}
