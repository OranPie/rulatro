use crate::{LoadedMod, ModError};
use mlua::LuaSerdeExt;
use mlua::{Function, Lua, RegistryKey, Value};
use rulatro_core::{
    ActivationType, CardDebuffPatch, EffectOutput, FlowCtx, FlowMode, FlowPoint, HandEvalPatch,
    HandTypeOutput, ModActionResult, ModEffectBlock, ModEffectContext, ModHandEvalContext,
    ModHandResult, ModHookContext, ModHookPhase, ModHookResult, ScoreBasePatch, ShopParamsPatch,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

struct LuaHook {
    mod_id: String,
    func: RegistryKey,
}

struct LuaEffect {
    mod_id: String,
    func: RegistryKey,
}

/// A handler registered via `rulatro.register_flow(point, mode, fn, opts)`.
struct LuaFlowHandler {
    mod_id: String,
    func: RegistryKey,
    /// Higher = runs earlier in patch chain / wins Replace contests.
    priority: i32,
}

#[derive(Debug, Deserialize)]
struct LuaHookReturn {
    #[serde(default)]
    stop: bool,
    #[serde(default)]
    cancel_core: bool,
    #[serde(default)]
    effects: Vec<ModEffectBlock>,
}

pub struct LuaRuntime {
    lua: Lua,
    hooks: Rc<RefCell<HashMap<(ActivationType, ModHookPhase), Vec<LuaHook>>>>,
    /// Effects registered via `rulatro.register_effect(name, fn)`.
    effects: Rc<RefCell<HashMap<String, Vec<LuaEffect>>>>,
    /// EffectOp handlers registered via `rulatro.register_effect_op(name, fn)`.
    effect_ops: Rc<RefCell<HashMap<String, Vec<LuaEffect>>>>,
    /// Custom hand evaluators registered via `rulatro.register_hand(id, def)`.
    custom_hands: Rc<RefCell<Vec<LuaCustomHand>>>,
    /// Flow Kernel handlers registered via `rulatro.register_flow(point, mode, fn, opts)`.
    /// Sorted by priority desc at insertion time.
    flow_handlers: Rc<RefCell<HashMap<(FlowPoint, FlowMode), Vec<LuaFlowHandler>>>>,
}

struct LuaCustomHand {
    mod_id: String,
    id: String,
    eval_func: RegistryKey,
    /// Higher priority = checked first before standard evaluation.
    priority: i32,
}

impl LuaRuntime {
    pub fn new() -> Result<Self, ModError> {
        let lua = Lua::new();
        let hooks = Rc::new(RefCell::new(HashMap::<
            (ActivationType, ModHookPhase),
            Vec<LuaHook>,
        >::new()));
        let effects: Rc<RefCell<HashMap<String, Vec<LuaEffect>>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let effect_ops: Rc<RefCell<HashMap<String, Vec<LuaEffect>>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let custom_hands: Rc<RefCell<Vec<LuaCustomHand>>> = Rc::new(RefCell::new(Vec::new()));
        let flow_handlers: Rc<RefCell<HashMap<(FlowPoint, FlowMode), Vec<LuaFlowHandler>>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let api = lua
            .create_table()
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        // register_hook
        let hooks_ref = hooks.clone();
        let register = lua
            .create_function_mut(
                move |lua, (trigger, func, phase): (String, Function, Option<String>)| {
                    let Some(trigger) = parse_trigger(&trigger) else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "unknown hook trigger {}",
                            trigger
                        )));
                    };
                    let phase = parse_phase(phase.as_deref()).ok_or_else(|| {
                        mlua::Error::RuntimeError(
                            "invalid hook phase (expected pre/post)".to_string(),
                        )
                    })?;
                    let key = lua.create_registry_value(func)?;
                    let mod_id: String = lua
                        .globals()
                        .get("__rulatro_mod_id")
                        .unwrap_or_else(|_| "unknown".to_string());
                    hooks_ref
                        .borrow_mut()
                        .entry((trigger, phase))
                        .or_default()
                        .push(LuaHook { mod_id, func: key });
                    Ok(())
                },
            )
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        // register_effect(name, fn)
        let effects_ref = effects.clone();
        let register_effect = lua
            .create_function_mut(move |lua, (name, func): (String, Function)| {
                let key = lua.create_registry_value(func)?;
                let mod_id: String = lua
                    .globals()
                    .get("__rulatro_mod_id")
                    .unwrap_or_else(|_| "unknown".to_string());
                effects_ref
                    .borrow_mut()
                    .entry(name)
                    .or_default()
                    .push(LuaEffect { mod_id, func: key });
                Ok(())
            })
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        // register_hand(id, { eval=fn, priority=N })
        let hands_ref = custom_hands.clone();
        let register_hand = lua
            .create_function_mut(move |lua, (id, def): (String, mlua::Table)| {
                let eval_func: Function = def.get("eval").map_err(|_| {
                    mlua::Error::RuntimeError("register_hand: 'eval' function required".to_string())
                })?;
                let priority: i32 = def.get("priority").unwrap_or(50);
                let key = lua.create_registry_value(eval_func)?;
                let mod_id: String = lua
                    .globals()
                    .get("__rulatro_mod_id")
                    .unwrap_or_else(|_| "unknown".to_string());
                let mut hands = hands_ref.borrow_mut();
                hands.push(LuaCustomHand {
                    mod_id,
                    id,
                    eval_func: key,
                    priority,
                });
                hands.sort_by(|a, b| b.priority.cmp(&a.priority));
                Ok(())
            })
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        // register_flow(point, mode, fn, opts?) — Flow Kernel unified registration
        let flow_ref = flow_handlers.clone();
        let register_flow = lua
            .create_function_mut(
                move |lua,
                      (point_str, mode_str, func, opts): (
                    String,
                    String,
                    Function,
                    Option<mlua::Table>,
                )| {
                    let Some(point) = parse_flow_point(&point_str) else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "register_flow: unknown point '{}'",
                            point_str
                        )));
                    };
                    let Some(mode) = parse_flow_mode(&mode_str) else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "register_flow: unknown mode '{}' (expected patch/replace/around)",
                            mode_str
                        )));
                    };
                    let priority: i32 = opts
                        .as_ref()
                        .and_then(|t| t.get("priority").ok())
                        .unwrap_or(50);
                    let key = lua.create_registry_value(func)?;
                    let mod_id: String = lua
                        .globals()
                        .get("__rulatro_mod_id")
                        .unwrap_or_else(|_| "unknown".to_string());
                    let handler = LuaFlowHandler {
                        mod_id,
                        func: key,
                        priority,
                    };
                    let mut map = flow_ref.borrow_mut();
                    let list = map.entry((point, mode)).or_default();
                    // Insert sorted by priority desc (stable — equal priority preserves insertion order)
                    let pos = list.partition_point(|h| h.priority > priority);
                    list.insert(pos, handler);
                    Ok(())
                },
            )
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

        // register_effect_op(name, fn) — for EffectOp::Custom consumable effects
        let effect_ops_ref = effect_ops.clone();
        let register_effect_op = lua
            .create_function_mut(move |lua, (name, func): (String, Function)| {
                let key = lua.create_registry_value(func)?;
                let mod_id: String = lua
                    .globals()
                    .get("__rulatro_mod_id")
                    .unwrap_or_else(|_| "unknown".to_string());
                effect_ops_ref
                    .borrow_mut()
                    .entry(name)
                    .or_default()
                    .push(LuaEffect { mod_id, func: key });
                Ok(())
            })
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        api.set("register_hook", register)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        api.set("register_effect", register_effect)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        api.set("register_effect_op", register_effect_op)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        api.set("register_hand", register_hand)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        api.set("register_flow", register_flow)
            .map_err(|err| ModError::Runtime(err.to_string()))?;
        api.set("log", log)
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        lua.globals()
            .set("rulatro", api)
            .map_err(|err| ModError::Runtime(err.to_string()))?;

        Ok(Self {
            lua,
            hooks,
            effects,
            effect_ops,
            custom_hands,
            flow_handlers,
        })
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
            hooks.remove(&(ctx.trigger, ctx.phase)).unwrap_or_default()
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
                if parsed.cancel_core {
                    result.cancel_core = true;
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
            .entry((ctx.trigger, ctx.phase))
            .or_default()
            .extend(hooks_snapshot.drain(..));
        result
    }

    pub fn invoke_effect(
        &mut self,
        name: &str,
        target: Option<&str>,
        value: f64,
        ctx: &ModEffectContext<'_>,
    ) -> ModActionResult {
        let mut merged = ModActionResult::default();
        let effects_snapshot: Vec<(String, RegistryKey)> = {
            let effects = self.effects.borrow();
            let Some(list) = effects.get(name) else {
                return merged;
            };
            list.iter()
                .map(|e| {
                    (
                        e.mod_id.clone(),
                        self.lua
                            .create_registry_value(
                                self.lua.registry_value::<Function>(&e.func).unwrap(),
                            )
                            .unwrap(),
                    )
                })
                .collect()
        };

        let ctx_value = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(err) => {
                eprintln!(
                    "[modding] invoke_effect: failed to serialize context: {}",
                    err
                );
                return merged;
            }
        };

        for (mod_id, key) in &effects_snapshot {
            let func: Function = match self.lua.registry_value(key) {
                Ok(f) => f,
                Err(err) => {
                    eprintln!(
                        "[mod:{}] missing effect function '{}': {}",
                        mod_id, name, err
                    );
                    continue;
                }
            };
            let prev_mod: Option<String> = self.lua.globals().get("__rulatro_mod_id").ok();
            let _ = self.lua.globals().set("__rulatro_mod_id", mod_id.clone());
            let result: Value = match func.call((ctx_value.clone(), target.unwrap_or(""), value)) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("[mod:{}] effect '{}' error: {}", mod_id, name, err);
                    if let Some(p) = prev_mod {
                        let _ = self.lua.globals().set("__rulatro_mod_id", p);
                    }
                    continue;
                }
            };
            if let Some(p) = prev_mod {
                let _ = self.lua.globals().set("__rulatro_mod_id", p);
            }
            if matches!(result, Value::Nil) {
                continue;
            }
            match self.lua.from_value::<ModActionResult>(result) {
                Ok(r) => {
                    merged.add_chips += r.add_chips;
                    merged.add_mult += r.add_mult;
                    if r.mul_mult != 0.0 {
                        merged.mul_mult = if merged.mul_mult == 0.0 {
                            r.mul_mult
                        } else {
                            merged.mul_mult * r.mul_mult
                        };
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
                Err(err) => {
                    eprintln!("[mod:{}] effect '{}' bad return: {}", mod_id, name, err);
                }
            }
        }
        merged
    }

    pub fn evaluate_hand(&mut self, ctx: &ModHandEvalContext<'_>) -> Option<ModHandResult> {
        let ctx_value = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(err) => {
                eprintln!(
                    "[modding] evaluate_hand: failed to serialize context: {}",
                    err
                );
                return None;
            }
        };
        let hands: Vec<(String, String, RegistryKey)> = {
            let hands = self.custom_hands.borrow();
            hands
                .iter()
                .map(|h| {
                    (
                        h.mod_id.clone(),
                        h.id.clone(),
                        self.lua
                            .create_registry_value(
                                self.lua.registry_value::<Function>(&h.eval_func).unwrap(),
                            )
                            .unwrap(),
                    )
                })
                .collect()
        };
        for (mod_id, hand_id, key) in &hands {
            let func: Function = match self.lua.registry_value(key) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let prev_mod: Option<String> = self.lua.globals().get("__rulatro_mod_id").ok();
            let _ = self.lua.globals().set("__rulatro_mod_id", mod_id.clone());
            let result: Value = match func.call(ctx_value.clone()) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!(
                        "[mod:{}] evaluate_hand '{}' error: {}",
                        mod_id, hand_id, err
                    );
                    if let Some(p) = prev_mod {
                        let _ = self.lua.globals().set("__rulatro_mod_id", p);
                    }
                    continue;
                }
            };
            if let Some(p) = prev_mod {
                let _ = self.lua.globals().set("__rulatro_mod_id", p);
            }
            match result {
                Value::Nil | Value::Boolean(false) => continue,
                other => {
                    if let Ok(r) = self.lua.from_value::<ModHandResult>(other) {
                        return Some(r);
                    }
                    // If just `true` returned, use all cards as scoring
                    return Some(ModHandResult {
                        hand_id: hand_id.clone(),
                        scoring_indices: (0..ctx.cards.len()).collect(),
                        base_chips: None,
                        base_mult: None,
                        level_chips: None,
                        level_mult: None,
                    });
                }
            }
        }
        None
    }

    pub fn invoke_effect_op(&mut self, name: &str, value: f64, ctx: &ModEffectContext<'_>) -> bool {
        let ctx_value = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let handlers: Vec<(String, RegistryKey)> = {
            let ops = self.effect_ops.borrow();
            let Some(list) = ops.get(name) else {
                return false;
            };
            list.iter()
                .map(|e| {
                    (
                        e.mod_id.clone(),
                        self.lua
                            .create_registry_value(
                                self.lua.registry_value::<Function>(&e.func).unwrap(),
                            )
                            .unwrap(),
                    )
                })
                .collect()
        };
        let mut handled = false;
        for (mod_id, key) in &handlers {
            let func: Function = match self.lua.registry_value(key) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let prev_mod: Option<String> = self.lua.globals().get("__rulatro_mod_id").ok();
            let _ = self.lua.globals().set("__rulatro_mod_id", mod_id.clone());
            match func.call::<_, Value>((ctx_value.clone(), value)) {
                Ok(_) => handled = true,
                Err(err) => {
                    eprintln!("[mod:{}] effect_op '{}' error: {}", mod_id, name, err);
                }
            }
            if let Some(p) = prev_mod {
                let _ = self.lua.globals().set("__rulatro_mod_id", p);
            }
        }
        handled
    }

    // ── Flow Kernel public methods ────────────────────────────────────────────

    /// Snapshot flow handlers for a point+mode, creating fresh registry copies.
    fn snapshot_flow(&self, point: FlowPoint, mode: FlowMode) -> Vec<(String, RegistryKey)> {
        let map = self.flow_handlers.borrow();
        let Some(list) = map.get(&(point, mode)) else {
            return Vec::new();
        };
        list.iter()
            .filter_map(|h| {
                let func: Function = self.lua.registry_value(&h.func).ok()?;
                let key = self.lua.create_registry_value(func).ok()?;
                Some((h.mod_id.clone(), key))
            })
            .collect()
    }

    pub fn flow_hand_eval_patch(
        &mut self,
        base: HandEvalPatch,
        ctx: &FlowCtx<'_>,
    ) -> HandEvalPatch {
        let snapshot = self.snapshot_flow(FlowPoint::HandEval, FlowMode::Patch);
        if snapshot.is_empty() {
            return base;
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return base,
        };
        run_patch(&self.lua, &snapshot, base, &ctx_val, "hand_eval")
    }

    pub fn flow_card_debuff_patch(
        &mut self,
        base: CardDebuffPatch,
        ctx: &FlowCtx<'_>,
    ) -> CardDebuffPatch {
        let snapshot = self.snapshot_flow(FlowPoint::CardDebuff, FlowMode::Patch);
        if snapshot.is_empty() {
            return base;
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return base,
        };
        run_patch(&self.lua, &snapshot, base, &ctx_val, "card_debuff")
    }

    pub fn flow_score_base_patch(
        &mut self,
        base: ScoreBasePatch,
        ctx: &FlowCtx<'_>,
    ) -> ScoreBasePatch {
        let snapshot = self.snapshot_flow(FlowPoint::ScoreBase, FlowMode::Patch);
        if snapshot.is_empty() {
            return base;
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return base,
        };
        run_patch(&self.lua, &snapshot, base, &ctx_val, "score_base")
    }

    pub fn flow_shop_params_patch(
        &mut self,
        base: ShopParamsPatch,
        ctx: &FlowCtx<'_>,
    ) -> ShopParamsPatch {
        let snapshot = self.snapshot_flow(FlowPoint::ShopParams, FlowMode::Patch);
        if snapshot.is_empty() {
            return base;
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return base,
        };
        run_patch(&self.lua, &snapshot, base, &ctx_val, "shop_params")
    }

    pub fn flow_hand_type_replace(&mut self, ctx: &FlowCtx<'_>) -> Option<HandTypeOutput> {
        let snapshot = self.snapshot_flow(FlowPoint::HandType, FlowMode::Replace);
        if snapshot.is_empty() {
            return None;
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return None,
        };
        for (mod_id, key) in &snapshot {
            let func: Function = match self.lua.registry_value(key) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let prev: Option<String> = self.lua.globals().get("__rulatro_mod_id").ok();
            let _ = self.lua.globals().set("__rulatro_mod_id", mod_id.clone());
            let result: Value = match func.call::<_, Value>(ctx_val.clone()) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("[mod:{}] flow hand_type replace error: {}", mod_id, err);
                    if let Some(p) = prev {
                        let _ = self.lua.globals().set("__rulatro_mod_id", p);
                    }
                    continue;
                }
            };
            if let Some(p) = prev {
                let _ = self.lua.globals().set("__rulatro_mod_id", p);
            }
            if matches!(result, Value::Nil | Value::Boolean(false)) {
                continue;
            }
            if let Ok(out) = self.lua.from_value::<HandTypeOutput>(result) {
                return Some(out);
            }
        }
        None
    }

    pub fn flow_joker_effect(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        let name = ctx.effect_name.unwrap_or("");
        let snapshot = self.snapshot_flow(FlowPoint::JokerEffect, FlowMode::Patch);
        if snapshot.is_empty() {
            return EffectOutput::default();
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return EffectOutput::default(),
        };
        run_effect_output_patch(&self.lua, &snapshot, ctx_val, name, "joker_effect")
    }

    pub fn flow_consumable_effect(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        let name = ctx.effect_name.unwrap_or("");
        let snapshot = self.snapshot_flow(FlowPoint::ConsumableEffect, FlowMode::Patch);
        if snapshot.is_empty() {
            return EffectOutput::default();
        }
        let ctx_val = match self.lua.to_value(ctx) {
            Ok(v) => v,
            Err(_) => return EffectOutput::default(),
        };
        run_effect_output_patch(&self.lua, &snapshot, ctx_val, name, "consumable_effect")
    }
}

/// Run a chain of Patch handlers; each handler sees the accumulated patch and returns updated patch.
fn run_patch<T>(
    lua: &Lua,
    handlers: &[(String, RegistryKey)],
    mut current: T,
    ctx_val: &Value,
    label: &str,
) -> T
where
    T: Serialize + for<'de> serde::Deserialize<'de>,
{
    for (mod_id, key) in handlers {
        let patch_val = match lua.to_value(&current) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("[mod:{}] flow_{} patch serialize: {}", mod_id, label, err);
                continue;
            }
        };
        let func: Function = match lua.registry_value(key) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let prev: Option<String> = lua.globals().get("__rulatro_mod_id").ok();
        let _ = lua.globals().set("__rulatro_mod_id", mod_id.clone());
        let result: Value = match func.call::<_, Value>((patch_val, ctx_val.clone())) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("[mod:{}] flow_{} error: {}", mod_id, label, err);
                if let Some(p) = prev {
                    let _ = lua.globals().set("__rulatro_mod_id", p);
                }
                continue;
            }
        };
        if let Some(p) = prev {
            let _ = lua.globals().set("__rulatro_mod_id", p);
        }
        if matches!(result, Value::Nil) {
            continue;
        }
        match lua.from_value::<T>(result) {
            Ok(new_patch) => {
                current = new_patch;
            }
            Err(err) => {
                eprintln!("[mod:{}] flow_{} bad return: {}", mod_id, label, err);
            }
        }
    }
    current
}

/// Run effect-output handlers for JokerEffect / ConsumableEffect flow points.
fn run_effect_output_patch(
    lua: &Lua,
    handlers: &[(String, RegistryKey)],
    ctx_val: Value,
    _name: &str,
    label: &str,
) -> EffectOutput {
    let mut merged = EffectOutput::default();
    for (mod_id, key) in handlers {
        let func: Function = match lua.registry_value(key) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let prev: Option<String> = lua.globals().get("__rulatro_mod_id").ok();
        let _ = lua.globals().set("__rulatro_mod_id", mod_id.clone());
        let result: Value = match func.call::<_, Value>(ctx_val.clone()) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("[mod:{}] flow_{} error: {}", mod_id, label, err);
                if let Some(p) = prev {
                    let _ = lua.globals().set("__rulatro_mod_id", p);
                }
                continue;
            }
        };
        if let Some(p) = prev {
            let _ = lua.globals().set("__rulatro_mod_id", p);
        }
        if matches!(result, Value::Nil) {
            continue;
        }
        match lua.from_value::<EffectOutput>(result) {
            Ok(out) => {
                let stop = out.stop;
                merged.merge_from(out);
                if stop {
                    break;
                }
            }
            Err(err) => {
                eprintln!("[mod:{}] flow_{} bad return: {}", mod_id, label, err);
            }
        }
    }
    merged
}

fn parse_flow_point(s: &str) -> Option<FlowPoint> {
    let norm: String = s.trim().to_ascii_lowercase();
    // Check lifecycle: prefix first
    if let Some(rest) = norm.strip_prefix("lifecycle:") {
        return parse_trigger(rest).map(FlowPoint::Lifecycle);
    }
    match norm.as_str() {
        "hand_eval" | "handeval" => Some(FlowPoint::HandEval),
        "card_debuff" | "carddebuff" => Some(FlowPoint::CardDebuff),
        "score_base" | "scorebase" => Some(FlowPoint::ScoreBase),
        "shop_params" | "shopparams" => Some(FlowPoint::ShopParams),
        "hand_type" | "handtype" => Some(FlowPoint::HandType),
        "joker_effect" | "jokereffect" => Some(FlowPoint::JokerEffect),
        "consumable_effect" | "consumableeffect" => Some(FlowPoint::ConsumableEffect),
        _ => None,
    }
}

fn parse_flow_mode(s: &str) -> Option<FlowMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "patch" => Some(FlowMode::Patch),
        "replace" => Some(FlowMode::Replace),
        "around" => Some(FlowMode::Around),
        _ => None,
    }
}

fn parse_phase(value: Option<&str>) -> Option<ModHookPhase> {
    let Some(raw) = value else {
        return Some(ModHookPhase::Post);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" | "post" | "postcore" => Some(ModHookPhase::Post),
        "pre" | "precore" | "before" => Some(ModHookPhase::Pre),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hook_phase_aliases() {
        assert_eq!(parse_phase(None), Some(ModHookPhase::Post));
        assert_eq!(parse_phase(Some("post")), Some(ModHookPhase::Post));
        assert_eq!(parse_phase(Some("pre")), Some(ModHookPhase::Pre));
        assert_eq!(parse_phase(Some("before")), Some(ModHookPhase::Pre));
        assert_eq!(parse_phase(Some("bad")), None);
    }
}
