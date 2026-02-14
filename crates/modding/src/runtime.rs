use crate::{LoadedMod, ModError};
use rulatro_core::{ModHookContext, ModHookResult, ModRuntime};

#[cfg(feature = "mod_lua")]
mod runtime_lua;
#[cfg(feature = "mod_lua")]
pub use runtime_lua::LuaRuntime;

#[cfg(feature = "mod_wasm")]
mod runtime_wasm;
#[cfg(feature = "mod_wasm")]
pub use runtime_wasm::WasmRuntime;

pub struct ModManager {
    mods: Vec<LoadedMod>,
    #[cfg(feature = "mod_lua")]
    lua: Option<LuaRuntime>,
    #[cfg(feature = "mod_wasm")]
    wasm: Option<WasmRuntime>,
}

impl ModManager {
    pub fn new() -> Self {
        Self {
            mods: Vec::new(),
            #[cfg(feature = "mod_lua")]
            lua: None,
            #[cfg(feature = "mod_wasm")]
            wasm: None,
        }
    }

    pub fn load_mods(&mut self, mods: &[LoadedMod]) -> Result<(), ModError> {
        self.mods = mods.to_vec();
        for item in mods {
            let Some(entry) = item.manifest.entry.as_ref() else {
                continue;
            };
            let ext = entry.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
            match ext.as_str() {
                "lua" => {
                    #[cfg(feature = "mod_lua")]
                    {
                        if self.lua.is_none() {
                            self.lua = Some(LuaRuntime::new()?);
                        }
                        if let Some(runtime) = self.lua.as_mut() {
                            runtime.load_mod(item)?;
                        }
                    }
                    #[cfg(not(feature = "mod_lua"))]
                    {
                        return Err(ModError::RuntimeUnavailable("lua".to_string()));
                    }
                }
                "wasm" => {
                    #[cfg(feature = "mod_wasm")]
                    {
                        if self.wasm.is_none() {
                            self.wasm = Some(WasmRuntime::new());
                        }
                        if let Some(runtime) = self.wasm.as_mut() {
                            runtime.load_mod(item)?;
                        }
                    }
                    #[cfg(not(feature = "mod_wasm"))]
                    {
                        return Err(ModError::RuntimeUnavailable("wasm".to_string()));
                    }
                }
                other => {
                    return Err(ModError::InvalidManifest(format!(
                        "unsupported entry extension {}",
                        other
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Default for ModManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModRuntime for ModManager {
    fn on_hook(&mut self, ctx: &ModHookContext<'_>) -> ModHookResult {
        let mut result = ModHookResult::default();
        #[cfg(feature = "mod_lua")]
        if let Some(runtime) = self.lua.as_mut() {
            result.merge(runtime.on_hook(ctx));
        }
        #[cfg(feature = "mod_wasm")]
        if let Some(runtime) = self.wasm.as_mut() {
            result.merge(runtime.on_hook(ctx));
        }
        result
    }
}
