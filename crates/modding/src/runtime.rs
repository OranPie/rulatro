use crate::{LoadedMod, ModError};
use rulatro_core::{
    CardDebuffPatch, EffectOutput, FlowCtx, HandEvalPatch, HandTypeOutput, ModActionResult,
    ModEffectContext, ModHandEvalContext, ModHandResult, ModHookContext, ModHookResult, ModRuntime,
    ScoreBasePatch, ShopParamsPatch,
};

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

    fn invoke_effect(
        &mut self,
        name: &str,
        target: Option<&str>,
        value: f64,
        ctx: &ModEffectContext<'_>,
    ) -> ModActionResult {
        let mut merged = ModActionResult::default();
        #[cfg(feature = "mod_lua")]
        if let Some(runtime) = self.lua.as_mut() {
            let r = runtime.invoke_effect(name, target, value, ctx);
            merged.add_chips += r.add_chips;
            merged.add_mult += r.add_mult;
            if r.mul_mult != 0.0 {
                merged.mul_mult =
                    if merged.mul_mult == 0.0 { r.mul_mult } else { merged.mul_mult * r.mul_mult };
            }
            if r.mul_chips != 0.0 {
                merged.mul_chips = if merged.mul_chips == 0.0 {
                    r.mul_chips
                } else {
                    merged.mul_chips * r.mul_chips
                };
            }
            merged.add_money += r.add_money;
            merged.set_rules.extend(r.set_rules);
            merged.add_rules.extend(r.add_rules);
            merged.set_vars.extend(r.set_vars);
            merged.add_vars.extend(r.add_vars);
        }
        merged
    }

    fn evaluate_hand(&mut self, ctx: &ModHandEvalContext<'_>) -> Option<ModHandResult> {
        #[cfg(feature = "mod_lua")]
        if let Some(runtime) = self.lua.as_mut() {
            if let Some(result) = runtime.evaluate_hand(ctx) {
                return Some(result);
            }
        }
        None
    }

    fn invoke_effect_op(&mut self, name: &str, value: f64, ctx: &ModEffectContext<'_>) -> bool {
        #[cfg(feature = "mod_lua")]
        if let Some(runtime) = self.lua.as_mut() {
            if runtime.invoke_effect_op(name, value, ctx) {
                return true;
            }
        }
        false
    }

    // ── Flow Kernel delegations ───────────────────────────────────────────────

    fn flow_hand_eval_patch(&mut self, base: HandEvalPatch, ctx: &FlowCtx<'_>) -> HandEvalPatch {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            return rt.flow_hand_eval_patch(base, ctx);
        }
        base
    }

    fn flow_card_debuff_patch(&mut self, base: CardDebuffPatch, ctx: &FlowCtx<'_>) -> CardDebuffPatch {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            return rt.flow_card_debuff_patch(base, ctx);
        }
        base
    }

    fn flow_score_base_patch(&mut self, base: ScoreBasePatch, ctx: &FlowCtx<'_>) -> ScoreBasePatch {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            return rt.flow_score_base_patch(base, ctx);
        }
        base
    }

    fn flow_shop_params_patch(&mut self, base: ShopParamsPatch, ctx: &FlowCtx<'_>) -> ShopParamsPatch {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            return rt.flow_shop_params_patch(base, ctx);
        }
        base
    }

    fn flow_hand_type_replace(&mut self, ctx: &FlowCtx<'_>) -> Option<HandTypeOutput> {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            if let Some(out) = rt.flow_hand_type_replace(ctx) {
                return Some(out);
            }
        }
        None
    }

    fn flow_joker_effect(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            return rt.flow_joker_effect(ctx);
        }
        EffectOutput::default()
    }

    fn flow_consumable_effect(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        #[cfg(feature = "mod_lua")]
        if let Some(rt) = self.lua.as_mut() {
            return rt.flow_consumable_effect(ctx);
        }
        EffectOutput::default()
    }
}
