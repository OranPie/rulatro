use crate::{LoadedMod, ModError};
use rulatro_core::{ModEffectBlock, ModHookContext, ModHookResult};
use serde::Deserialize;
use wasmtime::{Engine, Instance, Memory, Module, Store, TypedFunc};

#[derive(Debug, Deserialize)]
struct WasmHookReturn {
    #[serde(default)]
    stop: bool,
    #[serde(default)]
    cancel_core: bool,
    #[serde(default)]
    effects: Vec<ModEffectBlock>,
}

struct WasmMod {
    mod_id: String,
    store: Store<()>,
    memory: Memory,
    alloc: TypedFunc<i32, i32>,
    on_hook: TypedFunc<(i32, i32), i64>,
    dealloc: Option<TypedFunc<(i32, i32), ()>>,
}

pub struct WasmRuntime {
    engine: Engine,
    mods: Vec<WasmMod>,
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            mods: Vec::new(),
        }
    }

    pub fn load_mod(&mut self, item: &LoadedMod) -> Result<(), ModError> {
        let entry = item
            .manifest
            .entry
            .as_ref()
            .map(|value| value.as_str())
            .unwrap_or("unknown");
        let entry_path = item.root.join(entry);
        let module = Module::from_file(&self.engine, &entry_path)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| ModError::Runtime("wasm export memory is required".to_string()))?;
        let alloc = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .map_err(|err| ModError::Runtime(format!("wasm export alloc missing: {}", err)))?;
        let on_hook = instance
            .get_typed_func::<(i32, i32), i64>(&mut store, "on_hook")
            .map_err(|err| ModError::Runtime(format!("wasm export on_hook missing: {}", err)))?;
        let dealloc = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "dealloc")
            .ok();
        self.mods.push(WasmMod {
            mod_id: item.manifest.meta.id.clone(),
            store,
            memory,
            alloc,
            on_hook,
            dealloc,
        });
        Ok(())
    }

    pub fn on_hook(&mut self, ctx: &ModHookContext<'_>) -> ModHookResult {
        let mut merged = ModHookResult::default();
        let input = match serde_json::to_vec(ctx) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("[modding] failed to serialize wasm hook context: {}", err);
                return merged;
            }
        };

        for item in &mut self.mods {
            let in_len: i32 = match i32::try_from(input.len()) {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("[mod:{}] hook payload too large", item.mod_id);
                    continue;
                }
            };
            let in_ptr = match item.alloc.call(&mut item.store, in_len) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("[mod:{}] alloc failed: {}", item.mod_id, err);
                    continue;
                }
            };
            if in_ptr < 0 {
                eprintln!("[mod:{}] alloc returned negative pointer", item.mod_id);
                continue;
            }
            if let Err(err) = item.memory.write(&mut item.store, in_ptr as usize, &input) {
                eprintln!("[mod:{}] write memory failed: {}", item.mod_id, err);
                continue;
            }

            let out_packed = match item.on_hook.call(&mut item.store, (in_ptr, in_len)) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("[mod:{}] on_hook failed: {}", item.mod_id, err);
                    continue;
                }
            };
            if let Some(dealloc) = &item.dealloc {
                let _ = dealloc.call(&mut item.store, (in_ptr, in_len));
            }

            let (out_ptr, out_len) = unpack_ptr_len(out_packed);
            if out_len == 0 {
                continue;
            }
            let mut output = vec![0u8; out_len];
            if let Err(err) = item.memory.read(&mut item.store, out_ptr, &mut output) {
                eprintln!("[mod:{}] read memory failed: {}", item.mod_id, err);
                continue;
            }
            if let Some(dealloc) = &item.dealloc {
                let _ = dealloc.call(&mut item.store, (out_ptr as i32, out_len as i32));
            }

            match parse_hook_return(&output) {
                Ok(value) => merged.merge(value),
                Err(err) => eprintln!("[mod:{}] invalid hook return: {}", item.mod_id, err),
            }
        }
        merged
    }
}

fn unpack_ptr_len(value: i64) -> (usize, usize) {
    let raw = value as u64;
    let ptr = (raw >> 32) as u32 as usize;
    let len = (raw & 0xffff_ffff) as u32 as usize;
    (ptr, len)
}

fn parse_hook_return(bytes: &[u8]) -> Result<ModHookResult, serde_json::Error> {
    let parsed: Result<WasmHookReturn, serde_json::Error> = serde_json::from_slice(bytes);
    if let Ok(value) = parsed {
        return Ok(ModHookResult {
            stop: value.stop,
            cancel_core: value.cancel_core,
            effects: value.effects,
        });
    }
    let single: ModEffectBlock = serde_json::from_slice(bytes)?;
    Ok(ModHookResult {
        stop: false,
        cancel_core: false,
        effects: vec![single],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpacks_ptr_len() {
        let packed = ((7u64 << 32) | 13u64) as i64;
        let (ptr, len) = unpack_ptr_len(packed);
        assert_eq!(ptr, 7);
        assert_eq!(len, 13);
    }
}
