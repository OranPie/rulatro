use crate::{LoadedMod, ModError};
use mlua::{Function, Lua, RegistryKey, Value};
use mlua::LuaSerdeExt;
use rulatro_core::{ActivationType, ModEffectBlock, ModHookContext, ModHookResult};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

struct LuaHook {
    mod_id: String,
    func: RegistryKey,
}

#[derive(Debug, Deserialize)]
struct LuaHookReturn {
    #[serde(default)]
    stop: bool,
    #[serde(default)]
    effects: Vec<ModEffectBlock>,
}

pub struct LuaRuntime {
    lua: Lua,
    hooks: Rc<RefCell<HashMap<ActivationType, Vec<LuaHook>>>>,
}

impl LuaRuntime {
    pub fn new() -> Result<Self, ModError> {
        let lua = Lua::new();
        let hooks = Rc::new(RefCell::new(HashMap::<ActivationType, Vec<LuaHook>>::new()));
        let api = lua
            .create_table()
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        let hooks_ref = hooks.clone();
        let register = lua
            .create_function_mut(move |lua, (trigger, func): (String, Function)| {
                let Some(trigger) = parse_trigger(&trigger) else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "unknown hook trigger {}",
                        trigger
                    )));
                };
                let key = lua.create_registry_value(func)?;
                let mod_id: String = lua.globals().get("__rulatro_mod_id").unwrap_or_else(|_| "unknown".to_string());
                hooks_ref
                    .borrow_mut()
                    .entry(trigger)
                    .or_default()
                    .push(LuaHook { mod_id, func: key });
                Ok(())
            })
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        let log = lua
            .create_function(|lua, msg: String| {
                let mod_id: Option<String> = lua.globals().get("__rulatro_mod_id").ok();
                if let Some(mod_id) = mod_id {
                    println!("[mod:{}] {}", mod_id, msg);
                } else {
                    println!("[mod] {}", msg);
                }
                Ok(())
            })
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        api.set("register_hook", register)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        api.set("log", log)
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        lua.globals()
            .set("rulatro", api)
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        Ok(Self { lua, hooks })
    }

    pub fn load_mod(&mut self, item: &LoadedMod) -> Result<(), ModError> {
        let Some(entry) = item.manifest.entry.as_ref() else {
            return Ok(());
        };
        let entry_path = item.root.join(entry);
        let source = fs::read_to_string(&entry_path)?;
        self.lua
            .globals()
            .set("__rulatro_mod_id", item.manifest.meta.id.clone())
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        let chunk = self.lua.load(&source).set_name(&item.manifest.meta.id);
        chunk
            .exec()
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        Ok(())
    }

    pub fn on_hook(&mut self, ctx: &ModHookContext<'_>) -> ModHookResult {
        let mut result = ModHookResult::default();
        let mut hooks_snapshot = {
            let mut hooks = self.hooks.borrow_mut();
            hooks.remove(&ctx.trigger).unwrap_or_default()
        };
        if hooks_snapshot.is_empty() {
            return result;
        }
        let ctx_value = match self.lua.to_value(ctx) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("[modding] failed to serialize hook context: {}", err);
                return result;
            }
        };
        for hook in hooks_snapshot.iter() {
            let func: Function = match self.lua.registry_value(&hook.func) {
                Ok(func) => func,
                Err(err) => {
                    eprintln!("[mod:{}] missing hook function: {}", hook.mod_id, err);
                    continue;
                }
            };
            let previous_mod: Option<String> = self.lua.globals().get("__rulatro_mod_id").ok();
            let _ = self
                .lua
                .globals()
                .set("__rulatro_mod_id", hook.mod_id.clone());
            let value: Value = match func.call(ctx_value.clone()) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("[mod:{}] hook error: {}", hook.mod_id, err);
                    if let Some(previous_mod) = previous_mod {
                        let _ = self.lua.globals().set("__rulatro_mod_id", previous_mod);
                    }
                    continue;
                }
            };
            if let Some(previous_mod) = previous_mod {
                let _ = self.lua.globals().set("__rulatro_mod_id", previous_mod);
            }
            if matches!(value, Value::Nil) {
                continue;
            }
            if let Ok(parsed) = self.lua.from_value::<LuaHookReturn>(value.clone()) {
                if parsed.stop {
                    result.stop = true;
                }
                if !parsed.effects.is_empty() {
                    result.effects.extend(parsed.effects);
                }
                continue;
            }
            if let Ok(block) = self.lua.from_value::<ModEffectBlock>(value) {
                result.effects.push(block);
            } else {
                eprintln!("[mod:{}] hook returned unsupported value", hook.mod_id);
            }
        }
        let mut hooks = self.hooks.borrow_mut();
        hooks
            .entry(ctx.trigger)
            .or_default()
            .extend(hooks_snapshot.drain(..));
        result
    }
}

fn parse_trigger(value: &str) -> Option<ActivationType> {
    let mut normalized = value.trim().to_ascii_lowercase();
    normalized.retain(|ch| ch.is_ascii_alphanumeric());
    match normalized.as_str() {
        "onplayed" | "played" => Some(ActivationType::OnPlayed),
        "onscoredpre" | "scoredpre" => Some(ActivationType::OnScoredPre),
        "onscored" | "scored" => Some(ActivationType::OnScored),
        "onheld" | "held" => Some(ActivationType::OnHeld),
        "independent" => Some(ActivationType::Independent),
        "onotherjokers" | "otherjokers" => Some(ActivationType::OnOtherJokers),
        "ondiscard" | "discard" => Some(ActivationType::OnDiscard),
        "ondiscardbatch" | "discardbatch" => Some(ActivationType::OnDiscardBatch),
        "oncarddestroyed" | "carddestroyed" => Some(ActivationType::OnCardDestroyed),
        "oncardadded" | "cardadded" => Some(ActivationType::OnCardAdded),
        "onroundend" | "roundend" => Some(ActivationType::OnRoundEnd),
        "onhandend" | "handend" => Some(ActivationType::OnHandEnd),
        "onblindstart" | "blindstart" => Some(ActivationType::OnBlindStart),
        "onblindfailed" | "blindfailed" => Some(ActivationType::OnBlindFailed),
        "onshopenter" | "shopenter" => Some(ActivationType::OnShopEnter),
        "onshopreroll" | "shopreroll" => Some(ActivationType::OnShopReroll),
        "onshopexit" | "shopexit" => Some(ActivationType::OnShopExit),
        "onpackopened" | "packopened" => Some(ActivationType::OnPackOpened),
        "onpackskipped" | "packskipped" => Some(ActivationType::OnPackSkipped),
        "onuse" | "use" => Some(ActivationType::OnUse),
        "onsell" | "sell" => Some(ActivationType::OnSell),
        "onanysell" | "anysell" => Some(ActivationType::OnAnySell),
        "onacquire" | "acquire" => Some(ActivationType::OnAcquire),
        "passive" => Some(ActivationType::Passive),
        _ => None,
    }
}
