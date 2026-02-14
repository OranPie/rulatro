const state = {
  locale: "en_US",
  selectedHand: new Set(),
  selectedShopCard: null,
  selectedShopPack: null,
  selectedVoucher: null,
  selectedJoker: null,
  selectedConsumable: null,
  selectedPackOptions: new Set(),
  logLines: [],
  sortKey: "none",
  sortDir: "desc",
  rankChipMap: new Map(),
  lastSnapshot: null,
  openPackKey: null,
  quickBuy: true,
  pendingAction: null,
  actionHistory: [],
  scoreSections: {
    played: true,
    scoring: true,
    steps: false,
  },
  highlightScoring: true,
};

const UI_TEXT = {
  en_US: {
    subtitle: "Web frontend (API connected)",
    status_hint_prefix: "Next: ",
    hand_title: "Hand",
    scoring_title: "Scoring",
    shop_title: "Shop",
    inventory_title: "Inventory",
    pack_title: "Pack",
    run_data_title: "Run Data",
    log_title: "Log",
    reference_title: "Reference",
    controls_title: "Controls",
    blind_group: "Blind",
    shop_group: "Shop & Pack",
    system_group: "Inventory & System",
    start_btn: "Start Blind",
    deal_btn: "Deal",
    play_btn: "Play Selected",
    discard_btn: "Discard Selected",
    clear_hand_btn: "Clear Hand Selection",
    skip_blind_btn: "Skip Blind",
    next_blind_btn: "Next Blind",
    enter_shop_btn: "Enter Shop",
    reroll_btn: "Reroll",
    buy_selected_btn: "Buy Selected",
    leave_shop_btn: "Leave Shop",
    pick_pack_btn: "Pick Pack Selection",
    skip_pack_btn: "Skip Pack",
    clear_pack_btn: "Clear Pack Selection",
    use_consumable_btn: "Use Consumable",
    sell_joker_btn: "Sell Joker",
    undo_pending_btn: "Undo Pending",
    save_local_btn: "Save Local",
    load_local_btn: "Load Local",
    clear_local_btn: "Clear Local Save",
    clear_log_btn: "Clear Log",
    reset_btn: "Reset",
    next_action_setup: "Start blind.",
    next_action_shop: "Buy, reroll, or leave shop.",
    next_action_cleared: "Blind cleared - enter shop or go next blind.",
    next_action_deal: "Deal a hand.",
    next_action_deal_skip: "Deal a hand (or skip blind).",
    next_action_play: "Play selected cards or discard.",
    next_action_pack: "Pick pack options or skip the pack.",
    next_action_generic: "Review state and continue.",
    no_shop: "Shop closed.",
    no_pack: "No open pack.",
    no_score: "No scoring yet.",
    no_pack_inline: "No open pack.",
    quick_buy_on: "Quick buy: ON (click to buy, Shift+click to select)",
    quick_buy_off: "Quick buy: OFF (click to select, Buy Selected confirms)",
    selected_none: "none",
    selected_word: "Selected",
    picks_word: "Picks",
    options_word: "Options",
    too_many: "too many",
    log_entries: "Entries",
    quick_buy_on_btn: "Quick Buy: On",
    quick_buy_off_btn: "Quick Buy: Off",
    buy_in: "in",
    undo_key: "Undo (Z)",
    select_consumable: "select a consumable",
    select_joker: "select a joker",
    shop_unavailable: "shop not available",
    resolve_pack_first: "resolve open pack first",
    select_shop_first: "select a shop item first",
    max_hand_selected: "max 5 hand cards can be selected",
    pack_pick_limit: "pack allows up to {count} pick(s)",
    resolve_pack_before_buy: "resolve open pack before buying another offer",
    saved_local_ok: "saved state to browser storage",
    loaded_local_ok: "loaded state from browser storage",
    cleared_local_ok: "cleared browser save",
    no_local_save: "no local save found",
    load_local_failed: "failed to load local save",
    load_local_step_failed: "restore failed at step {step}: {error}",
    load_local_sig_mismatch:
      "local save does not match current content/mods (saved {saved}, current {current})",
    unknown: "Unknown",
    boss_none: "none",
    boss_disabled_state: "disabled",
    boss_effects_none: "Boss effects: none",
    boss_effects_title: "Boss effects",
    active_vouchers_none: "Active vouchers: none",
    active_vouchers_title: "Active vouchers",
  },
  zh_CN: {
    subtitle: "Web 前端（已连接 API）",
    status_hint_prefix: "下一步：",
    hand_title: "手牌",
    scoring_title: "计分",
    shop_title: "商店",
    inventory_title: "背包",
    pack_title: "卡包",
    run_data_title: "运行数据",
    log_title: "日志",
    reference_title: "参考",
    controls_title: "控制",
    blind_group: "盲注流程",
    shop_group: "商店与卡包",
    system_group: "背包与系统",
    start_btn: "开始盲注",
    deal_btn: "发牌",
    play_btn: "出选中牌",
    discard_btn: "弃选中牌",
    clear_hand_btn: "清空手牌选择",
    skip_blind_btn: "跳过盲注",
    next_blind_btn: "下一盲注",
    enter_shop_btn: "进入商店",
    reroll_btn: "刷新",
    buy_selected_btn: "购买已选",
    leave_shop_btn: "离开商店",
    pick_pack_btn: "确认卡包选择",
    skip_pack_btn: "跳过卡包",
    clear_pack_btn: "清空卡包选择",
    use_consumable_btn: "使用消耗牌",
    sell_joker_btn: "出售小丑",
    undo_pending_btn: "撤销待执行",
    save_local_btn: "保存到浏览器",
    load_local_btn: "从浏览器读取",
    clear_local_btn: "清除浏览器存档",
    clear_log_btn: "清空日志",
    reset_btn: "重置",
    next_action_setup: "开始盲注。",
    next_action_shop: "购买、刷新，或离开商店。",
    next_action_cleared: "盲注已通过 - 进入商店或下一盲注。",
    next_action_deal: "发一手牌。",
    next_action_deal_skip: "发一手牌（或跳过盲注）。",
    next_action_play: "出已选牌，或弃牌。",
    next_action_pack: "选择卡包选项，或跳过卡包。",
    next_action_generic: "检查状态后继续。",
    no_shop: "商店未开启。",
    no_pack: "没有打开的卡包。",
    no_score: "尚无计分。",
    no_pack_inline: "没有打开的卡包。",
    quick_buy_on: "快速购买：开（点击即买，Shift+点击仅选择）",
    quick_buy_off: "快速购买：关（点击选择，使用 Buy Selected 确认）",
    selected_none: "无",
    selected_word: "已选",
    picks_word: "可选",
    options_word: "选项",
    too_many: "超出上限",
    log_entries: "日志条数",
    quick_buy_on_btn: "快速购买：开",
    quick_buy_off_btn: "快速购买：关",
    buy_in: "后执行",
    undo_key: "撤销 (Z)",
    select_consumable: "请先选择消耗牌",
    select_joker: "请先选择小丑",
    shop_unavailable: "商店不可用",
    resolve_pack_first: "请先处理当前打开的卡包",
    select_shop_first: "请先选择商店商品",
    max_hand_selected: "最多只能选择 5 张手牌",
    pack_pick_limit: "该卡包最多可选 {count} 个",
    resolve_pack_before_buy: "请先处理打开的卡包，再购买其他商品",
    saved_local_ok: "已保存状态到浏览器存储",
    loaded_local_ok: "已从浏览器存储恢复状态",
    cleared_local_ok: "已清除浏览器存档",
    no_local_save: "未找到浏览器存档",
    load_local_failed: "读取浏览器存档失败",
    load_local_step_failed: "恢复在第 {step} 步失败：{error}",
    load_local_sig_mismatch:
      "本地存档与当前内容/模组不一致（存档 {saved}，当前 {current}）",
    unknown: "未知",
    boss_none: "无",
    boss_disabled_state: "已禁用",
    boss_effects_none: "Boss 效果：无",
    boss_effects_title: "Boss 效果",
    active_vouchers_none: "已激活优惠券：无",
    active_vouchers_title: "已激活优惠券",
  },
};

const LOCAL_SAVE_KEY = "rulatro.web.state.v1";
const MAX_SAVED_ACTIONS = 1500;

const elements = {
  status: document.getElementById("status"),
  statusHint: document.getElementById("status-hint"),
  hand: document.getElementById("hand"),
  handSummary: document.getElementById("hand-summary"),
  shopCards: document.getElementById("shop-cards"),
  shopPacks: document.getElementById("shop-packs"),
  shopVouchers: document.getElementById("shop-vouchers"),
  shopSummary: document.getElementById("shop-summary"),
  invJokers: document.getElementById("inv-jokers"),
  invConsumables: document.getElementById("inv-consumables"),
  invSummary: document.getElementById("inv-summary"),
  packOptions: document.getElementById("pack-options"),
  packSummary: document.getElementById("pack-summary"),
  scoreBreakdown: document.getElementById("score-breakdown"),
  levels: document.getElementById("levels"),
  tags: document.getElementById("tags"),
  log: document.getElementById("log"),
  logSummary: document.getElementById("log-summary"),
  toastArea: document.getElementById("toast-area"),
  sortKey: document.getElementById("sort-key"),
  sortDir: document.getElementById("sort-dir"),
  bossEffects: document.getElementById("boss-effects"),
  activeVouchers: document.getElementById("active-vouchers"),
};

const QUICK_BUY_DELAY_MS = 2000;

const buttons = document.querySelectorAll("[data-action]");
const actionButtons = {};
buttons.forEach((button) => {
  actionButtons[button.dataset.action] = button;
  button.addEventListener("click", () => handleAction(button.dataset.action));
});

const shortcutHints = {
  start: "S",
  deal: "D",
  play: "P",
  discard: "X",
  clear_hand: "C",
  enter_shop: "O",
  reroll: "R",
  buy_selected: "B",
  leave_shop: "L",
  toggle_quick_buy: "Q",
  pick_pack: "K",
  skip_pack: "Y",
  clear_pack: "Shift+C",
  use_consumable: "U",
  sell_joker: "J",
  next_blind: "N",
  skip_blind: "G",
  reset: "Shift+R",
  undo_pending: "Z",
};

Object.entries(shortcutHints).forEach(([action, key]) => {
  const button = actionButtons[action];
  if (button) {
    button.title = `Shortcut: ${key}`;
  }
});

function tr(key, vars = {}) {
  const table = UI_TEXT[state.locale] || UI_TEXT.en_US;
  const base = table[key] ?? UI_TEXT.en_US[key] ?? key;
  return base.replace(/\{(\w+)\}/g, (_, name) =>
    Object.prototype.hasOwnProperty.call(vars, name) ? String(vars[name]) : ""
  );
}

function applyStaticTranslations() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.dataset.i18n;
    el.textContent = tr(key);
  });
}

function shouldRecordAction(action) {
  return action !== "reset";
}

function recordAction(action, indices = [], target = null) {
  if (!shouldRecordAction(action)) {
    state.actionHistory = [];
    persistLocalState();
    return;
  }
  state.actionHistory.push({
    action,
    indices: Array.isArray(indices) ? [...indices] : [],
    target: target ?? null,
  });
  if (state.actionHistory.length > MAX_SAVED_ACTIONS) {
    state.actionHistory = state.actionHistory.slice(
      state.actionHistory.length - MAX_SAVED_ACTIONS
    );
  }
  persistLocalState();
}

function currentUiPrefs() {
  return {
    quickBuy: state.quickBuy,
    sortKey: state.sortKey,
    sortDir: state.sortDir,
  };
}

function persistLocalState() {
  try {
    const seed =
      state.lastSnapshot &&
      state.lastSnapshot.state &&
      Number.isInteger(state.lastSnapshot.state.seed)
        ? state.lastSnapshot.state.seed
        : null;
    const contentSignature =
      state.lastSnapshot &&
      state.lastSnapshot.state &&
      typeof state.lastSnapshot.state.content_signature === "string"
        ? state.lastSnapshot.state.content_signature
        : null;
    const payload = {
      version: 1,
      locale: state.locale,
      seed,
      contentSignature,
      actions: state.actionHistory,
      prefs: currentUiPrefs(),
      savedAt: new Date().toISOString(),
    };
    localStorage.setItem(LOCAL_SAVE_KEY, JSON.stringify(payload));
  } catch (_err) {
    // ignore storage errors in restricted/private mode
  }
}

function readLocalSave() {
  try {
    const raw = localStorage.getItem(LOCAL_SAVE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    return parsed;
  } catch (_err) {
    return null;
  }
}

function applySavedPrefs(saved) {
  const prefs = saved?.prefs;
  if (!prefs || typeof prefs !== "object") {
    return;
  }
  if (prefs.quickBuy != null) {
    state.quickBuy = Boolean(prefs.quickBuy);
  }
  if (prefs.sortKey === "none" || prefs.sortKey === "rank" || prefs.sortKey === "value") {
    state.sortKey = prefs.sortKey;
    elements.sortKey.value = prefs.sortKey;
  }
  if (prefs.sortDir === "asc" || prefs.sortDir === "desc") {
    state.sortDir = prefs.sortDir;
    elements.sortDir.textContent = state.sortDir === "asc" ? "Asc" : "Desc";
  }
}

function clearLocalSave() {
  try {
    localStorage.removeItem(LOCAL_SAVE_KEY);
  } catch (_err) {
    // ignore
  }
  state.actionHistory = [];
  pushLog(tr("cleared_local_ok"));
}

async function restoreFromLocalSave() {
  const saved = readLocalSave();
  if (!saved || !Array.isArray(saved.actions)) {
    pushLog(tr("no_local_save"));
    return;
  }
  const currentSignature =
    state.lastSnapshot &&
    state.lastSnapshot.state &&
    typeof state.lastSnapshot.state.content_signature === "string"
      ? state.lastSnapshot.state.content_signature
      : null;
  if (
    typeof saved.contentSignature === "string" &&
    saved.contentSignature &&
    currentSignature &&
    saved.contentSignature !== currentSignature
  ) {
    pushLog(
      tr("load_local_sig_mismatch", {
        saved: saved.contentSignature,
        current: currentSignature,
      })
    );
    return;
  }
  const actions = saved.actions
    .filter((step) => step && typeof step.action === "string")
    .map((step) => ({
      action: step.action,
      indices: Array.isArray(step.indices)
        ? step.indices.filter((value) => Number.isInteger(value) && value >= 0)
        : [],
      target: step.target == null ? null : String(step.target),
    }));
  cancelPendingAction();
  applySavedPrefs(saved);
  state.actionHistory = [];
  const seed =
    Number.isInteger(saved.seed) && saved.seed >= 0 ? String(saved.seed) : null;
  await callAction("reset", { target: seed }, { record: false });
  for (let idx = 0; idx < actions.length; idx += 1) {
    const step = actions[idx];
    const result = await callAction(
      step.action,
      { indices: step.indices, target: step.target },
      { record: true }
    );
    if (!result.ok) {
      pushLog(tr("load_local_step_failed", { step: idx + 1, error: result.error || tr("unknown") }));
      return;
    }
  }
  pushLog(tr("loaded_local_ok"));
}

elements.sortKey.addEventListener("change", () => {
  state.sortKey = elements.sortKey.value;
  persistLocalState();
  if (state.lastSnapshot) {
    render(state.lastSnapshot);
  }
});

elements.sortDir.addEventListener("click", () => {
  state.sortDir = state.sortDir === "asc" ? "desc" : "asc";
  elements.sortDir.textContent = state.sortDir === "asc" ? "Asc" : "Desc";
  persistLocalState();
  if (state.lastSnapshot) {
    render(state.lastSnapshot);
  }
});

async function fetchState() {
  const res = await fetch("/api/state");
  const data = await res.json();
  render(data);
}

async function callAction(action, payload = {}, options = {}) {
  const record = options.record !== false;
  const body = {
    action,
    indices: payload.indices || [],
    target: payload.target ?? null,
  };
  const res = await fetch("/api/action", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = await res.json();
  if (!data.ok) {
    pushLog(`error: ${data.error}`);
  } else if (record) {
    recordAction(action, body.indices, body.target);
  }
  render(data);
  return data;
}

function handleAction(action) {
  if (actionButtons[action] && actionButtons[action].disabled) {
    return;
  }
  if (action !== "undo_pending") {
    cancelPendingAction();
  }
  switch (action) {
    case "start":
      callAction("start");
      break;
    case "deal":
      callAction("deal");
      break;
    case "play":
      callAction("play", { indices: Array.from(state.selectedHand).sort() });
      break;
    case "discard":
      callAction("discard", { indices: Array.from(state.selectedHand).sort() });
      break;
    case "enter_shop":
      callAction("enter_shop");
      break;
    case "leave_shop":
      callAction("leave_shop");
      break;
    case "reroll":
      callAction("reroll");
      break;
    case "use_consumable":
      if (state.selectedConsumable == null) {
        pushLog(tr("select_consumable"));
        break;
      }
      callAction("use_consumable", {
        target: String(state.selectedConsumable),
        indices: Array.from(state.selectedHand).sort(),
      });
      break;
    case "sell_joker":
      if (state.selectedJoker == null) {
        pushLog(tr("select_joker"));
        break;
      }
      callAction("sell_joker", { target: String(state.selectedJoker) });
      break;
    case "pick_pack":
      callAction("pick_pack", {
        indices: Array.from(state.selectedPackOptions).sort(),
      });
      break;
    case "skip_pack":
      callAction("skip_pack");
      break;
    case "skip_blind":
      callAction("skip_blind");
      break;
    case "next_blind":
      callAction("next_blind");
      break;
    case "reset":
      callAction("reset");
      break;
    case "save_local":
      persistLocalState();
      pushLog(tr("saved_local_ok"));
      if (state.lastSnapshot) {
        updateControls(state.lastSnapshot);
      }
      break;
    case "load_local":
      restoreFromLocalSave().catch((err) => {
        pushLog(`${tr("load_local_failed")}: ${err}`);
      });
      break;
    case "clear_local":
      clearLocalSave();
      if (state.lastSnapshot) {
        updateControls(state.lastSnapshot);
      }
      break;
    case "clear_log":
      state.logLines = [];
      renderLog();
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
      break;
    case "clear_hand":
      state.selectedHand.clear();
      if (state.lastSnapshot) {
        render(state.lastSnapshot);
      }
      break;
    case "clear_pack":
      state.selectedPackOptions.clear();
      if (state.lastSnapshot) {
        render(state.lastSnapshot);
      }
      break;
    case "toggle_quick_buy":
      state.quickBuy = !state.quickBuy;
      persistLocalState();
      updateQuickBuyButton();
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
      break;
    case "buy_selected":
      handleBuySelected();
      break;
    case "undo_pending":
      cancelPendingAction();
      break;
    default:
      break;
  }
}

function setActionEnabled(action, enabled) {
  const button = actionButtons[action];
  if (!button) return;
  button.disabled = !enabled;
}

function updateControls(snapshot) {
  if (!snapshot) return;
  const run = snapshot.state;
  const phase = snapshot.state.phase;
  const hasOpenPack = snapshot.open_pack != null;
  const blindCleared = isBlindCleared(run);
  const hasHandSelection = state.selectedHand.size > 0 && state.selectedHand.size <= 5;
  const hasPackSelection = state.selectedPackOptions.size > 0;
  const hasConsumable = state.selectedConsumable != null;
  const hasJoker = state.selectedJoker != null;
  const hasShopSelection =
    state.selectedShopCard != null ||
    state.selectedShopPack != null ||
    state.selectedVoucher != null;
  const packPickLimit = snapshot.open_pack?.offer?.picks ?? 0;
  const validPackSelection =
    hasPackSelection &&
    state.selectedPackOptions.size <= packPickLimit &&
    state.selectedPackOptions.size > 0;
  setActionEnabled("start", phase === "Setup");
  setActionEnabled("deal", phase === "Deal" && !hasOpenPack);
  setActionEnabled("play", phase === "Play" && hasHandSelection && !hasOpenPack);
  setActionEnabled(
    "discard",
    phase === "Play" &&
      hasHandSelection &&
      run.discards_left > 0 &&
      !hasOpenPack
  );
  setActionEnabled("enter_shop", phase !== "Shop" && blindCleared && !hasOpenPack);
  setActionEnabled("leave_shop", phase === "Shop" && !hasOpenPack);
  setActionEnabled("reroll", phase === "Shop" && !hasOpenPack);
  setActionEnabled("buy_selected", phase === "Shop" && hasShopSelection && !hasOpenPack);
  setActionEnabled("pick_pack", hasOpenPack && validPackSelection);
  setActionEnabled("skip_pack", hasOpenPack);
  setActionEnabled(
    "skip_blind",
    phase === "Deal" && run.blind !== "Boss" && !hasOpenPack
  );
  setActionEnabled("next_blind", blindCleared && !hasOpenPack);
  setActionEnabled("use_consumable", hasConsumable && !hasOpenPack);
  setActionEnabled("sell_joker", hasJoker && !hasOpenPack);
  setActionEnabled("undo_pending", state.pendingAction != null);
  setActionEnabled("clear_log", state.logLines.length > 0);
  setActionEnabled("save_local", true);
  setActionEnabled("load_local", readLocalSave() != null);
  setActionEnabled("clear_local", readLocalSave() != null);
  setActionEnabled("reset", true);
  setActionEnabled("clear_hand", state.selectedHand.size > 0);
  setActionEnabled("clear_pack", state.selectedPackOptions.size > 0);
}

function render(data) {
  if (data.locale && data.locale !== state.locale) {
    state.locale = data.locale;
  }
  applyStaticTranslations();
  state.lastSnapshot = data;
  state.rankChipMap = new Map(
    (data.state.rank_chips || []).map((entry) => [entry.rank, entry.chips])
  );
  if (data.events && data.events.length > 0) {
    data.events.forEach((event) => pushLog(formatEvent(event)));
  }
  renderStatus(data);
  renderHand(data.state, data.last_breakdown);
  renderShop(data.state.shop);
  renderInventory(data.state);
  renderPack(data.open_pack);
  renderScore(data.last_breakdown);
  renderBoss(data.state);
  renderVouchers(data.state);
  renderLevels(data.state.hand_levels);
  renderTags(data.state.tags, data.state.duplicate_next_tag, data.state.duplicate_tag_exclude);
  updateSummaries(data);
  updateControls(data);
  updateQuickBuyButton();
  renderLog();
}

function renderStatus(snapshot) {
  const run = snapshot.state;
  const fields = elements.status.querySelectorAll("[data-field]");
  fields.forEach((el) => {
    const key = el.dataset.field;
    const value = run[key];
    if (key === "boss_name") {
      if (run.blind !== "Boss") {
        el.textContent = tr("boss_none");
      } else if (run.boss_disabled) {
        el.textContent = tr("boss_disabled_state");
      } else {
        el.textContent = value || run.boss_id || "-";
      }
      return;
    }
    if (key === "boss_disabled") {
      el.textContent = run.blind === "Boss" ? (run.boss_disabled ? "yes" : "no") : "-";
      return;
    }
    el.textContent = value ?? "-";
  });
  if (elements.statusHint) {
    elements.statusHint.textContent = tr("status_hint_prefix") + nextActionHint(snapshot);
  }
}

function renderBoss(run) {
  if (!elements.bossEffects) return;
  elements.bossEffects.innerHTML = "";
  if (run.blind !== "Boss") {
    elements.bossEffects.textContent = tr("boss_effects_none");
    return;
  }
  if (run.boss_disabled) {
    elements.bossEffects.textContent = `${tr("boss_effects_title")}: ${tr("boss_disabled_state")}`;
    return;
  }
  if (!Array.isArray(run.boss_effects) || run.boss_effects.length === 0) {
    elements.bossEffects.textContent = tr("boss_effects_none");
    return;
  }
  const title = document.createElement("div");
  title.className = "badge";
  title.textContent = `${tr("boss_effects_title")}: ${run.boss_name || run.boss_id || "-"}`;
  elements.bossEffects.appendChild(title);
  run.boss_effects.forEach((effect) => {
    const row = document.createElement("div");
    row.className = "badge";
    row.textContent = effect;
    elements.bossEffects.appendChild(row);
  });
}

function renderVouchers(run) {
  if (!elements.activeVouchers) return;
  elements.activeVouchers.innerHTML = "";
  if (!Array.isArray(run.active_vouchers) || run.active_vouchers.length === 0) {
    elements.activeVouchers.textContent = tr("active_vouchers_none");
    return;
  }
  const title = document.createElement("div");
  title.className = "badge";
  title.textContent = tr("active_vouchers_title");
  elements.activeVouchers.appendChild(title);
  run.active_vouchers.forEach((entry) => {
    const row = document.createElement("div");
    row.className = "badge";
    row.textContent = entry;
    elements.activeVouchers.appendChild(row);
  });
}

function updateSummaries(snapshot) {
  const run = snapshot.state;
  const handCount = run.hand.length;
  elements.handSummary.textContent = `Selected: ${state.selectedHand.size}/5 | Hand: ${handCount} | Hands: ${run.hands_left}/${run.hands_max} | Discards: ${run.discards_left}/${run.discards_max} | Skipped: ${run.blinds_skipped}`;

  if (!run.shop) {
    elements.shopSummary.textContent = tr("no_shop");
  } else {
    let selected = tr("selected_none");
    if (state.selectedShopCard != null) {
      selected = `card #${state.selectedShopCard}`;
    } else if (state.selectedShopPack != null) {
      selected = `pack #${state.selectedShopPack}`;
    } else if (state.selectedVoucher != null) {
      selected = `voucher #${state.selectedVoucher}`;
    }
    const quickHint = state.quickBuy ? tr("quick_buy_on") : tr("quick_buy_off");
    const voucherCount = Array.isArray(run.shop.voucher_offers)
      ? run.shop.voucher_offers.length
      : run.shop.vouchers;
    elements.shopSummary.textContent = `Cards: ${run.shop.cards.length} | Packs: ${run.shop.packs.length} | Vouchers: ${voucherCount} ($${run.shop.voucher_price}) | Reroll: ${run.shop.reroll_cost} | ${tr("selected_word")}: ${selected} | ${quickHint}`;
  }

  elements.invSummary.textContent = `Jokers: ${run.jokers.length} | Consumables: ${run.consumables.length} | Selected: ${
    state.selectedJoker != null ? `joker #${state.selectedJoker}` : state.selectedConsumable != null ? `consumable #${state.selectedConsumable}` : tr("selected_none")
  }`;

  if (!snapshot.open_pack) {
    elements.packSummary.textContent = tr("no_pack");
  } else {
    const picks = snapshot.open_pack.offer.picks;
    const selectionState =
      state.selectedPackOptions.size > picks
        ? `${tr("selected_word")}: ${state.selectedPackOptions.size}/${picks} (${tr("too_many")})`
        : `${tr("selected_word")}: ${state.selectedPackOptions.size}/${picks}`;
    elements.packSummary.textContent = `${tr("options_word")}: ${snapshot.open_pack.options.length} | ${tr("picks_word")}: ${picks} | ${selectionState}`;
  }

  elements.logSummary.textContent = `${tr("log_entries")}: ${state.logLines.length}`;
  elements.logSummary.title =
    "Shortcuts: S D P X O R L B Q K Y U J N G Shift+R Z (Shift+C clears pack)";
}

function renderHand(run, breakdown) {
  const hand = run.hand;
  elements.hand.innerHTML = "";
  state.selectedHand.forEach((idx) => {
    if (idx >= hand.length) {
      state.selectedHand.delete(idx);
    }
  });
  let scoringSet = new Set();
  let scoringChips = new Map();
  if (
    state.highlightScoring &&
    breakdown &&
    Array.isArray(breakdown.scoring_indices) &&
    breakdown.scoring_indices.every((idx) => idx < hand.length)
  ) {
    scoringSet = new Set(breakdown.scoring_indices);
    if (Array.isArray(breakdown.scoring_cards)) {
      breakdown.scoring_cards.forEach((entry) => {
        scoringChips.set(entry.index, entry.chips);
      });
    }
  }
  const entries = hand.map((card, idx) => ({ card, idx }));
  const sorted = sortHandEntries(entries);
  sorted.forEach(({ card, idx }) => {
    const el = document.createElement("div");
    el.className = "card";
    el.tabIndex = 0;
    if (state.selectedHand.has(idx)) {
      el.classList.add("selected");
    }
    if (scoringSet.has(idx)) {
      el.classList.add("scoring");
    }
    const scoreValue = scoringChips.get(idx);
    el.title = formatCardTooltip(card, idx);
    if (scoreValue != null) {
      el.title += " | scoring: " + scoreValue;
    }
    el.innerHTML = `
      <div class="title">${formatCard(card)}</div>
      <div class="meta">${formatCardMeta(card, idx, scoreValue)}</div>
    `;
    el.addEventListener("click", () => toggleHandSelection(idx, el));
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        toggleHandSelection(idx, el);
      }
    });
    elements.hand.appendChild(el);
  });
}

function sortHandEntries(entries) {
  if (state.sortKey === "none") {
    return entries;
  }
  const dir = state.sortDir === "asc" ? 1 : -1;
  const sorted = [...entries];
  if (state.sortKey === "rank") {
    sorted.sort((a, b) => {
      const left = rankValue(a.card.rank);
      const right = rankValue(b.card.rank);
      if (left === right) return (a.idx - b.idx) * dir;
      return (left - right) * dir;
    });
  } else if (state.sortKey === "value") {
    sorted.sort((a, b) => {
      const left = cardValue(a.card);
      const right = cardValue(b.card);
      if (left === right) return (a.idx - b.idx) * dir;
      return (left - right) * dir;
    });
  }
  return sorted;
}

function renderShop(shop) {
  elements.shopCards.innerHTML = "";
  elements.shopPacks.innerHTML = "";
  elements.shopVouchers.innerHTML = "";

  if (!shop) {
    state.selectedShopCard = null;
    state.selectedShopPack = null;
    state.selectedVoucher = null;
    return;
  }
  if (state.selectedShopCard != null && state.selectedShopCard >= shop.cards.length) {
    state.selectedShopCard = null;
  }
  if (state.selectedShopPack != null && state.selectedShopPack >= shop.packs.length) {
    state.selectedShopPack = null;
  }
  const voucherOffers = Array.isArray(shop.voucher_offers)
    ? shop.voucher_offers
    : Array.from({ length: shop.vouchers || 0 }, (_, idx) => ({
        id: `voucher_${idx}`,
        name: "Voucher",
        effect: "",
      }));
  if (state.selectedVoucher != null && state.selectedVoucher >= voucherOffers.length) {
    state.selectedVoucher = null;
  }

  shop.cards.forEach((card, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.tabIndex = 0;
    if (state.selectedShopCard === idx) {
      el.classList.add("selected");
    }
    el.title = `price ${card.price}${card.rarity ? ` | rarity ${card.rarity}` : ""}${
      card.edition ? ` | edition ${card.edition}` : ""
    }`;
    const label = card.name ? `${card.name} / ${card.item_id}` : card.item_id;
    el.innerHTML = `[${idx}] ${card.kind} ${label} (${card.price})`;
    const clickHandler = (event) => {
      state.selectedShopCard = idx;
      state.selectedShopPack = null;
      state.selectedVoucher = null;
      highlightSelection(elements.shopCards, idx);
      clearSelection(elements.shopPacks);
      clearSelection(elements.shopVouchers);
      if (state.lastSnapshot?.open_pack) {
        pushLog(tr("resolve_pack_before_buy"));
        return;
      }
      if (
        state.quickBuy &&
        !event.shiftKey &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey
      ) {
        scheduleAction(
          "buy_card",
          { target: String(idx) },
          `Buy card #${idx} (${card.name || card.item_id})`
        );
      }
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    };
    el.addEventListener("click", clickHandler);
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        clickHandler(event);
      }
    });
    elements.shopCards.appendChild(el);
  });

  shop.packs.forEach((pack, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.tabIndex = 0;
    if (state.selectedShopPack === idx) {
      el.classList.add("selected");
    }
    el.title = `options ${pack.options} | picks ${pack.picks} | price ${pack.price}`;
    el.innerHTML = `[${idx}] ${pack.kind} ${pack.size} (pick ${pack.picks}) ${pack.price}`;
    const clickHandler = (event) => {
      state.selectedShopPack = idx;
      state.selectedShopCard = null;
      state.selectedVoucher = null;
      highlightSelection(elements.shopPacks, idx);
      clearSelection(elements.shopCards);
      clearSelection(elements.shopVouchers);
      if (state.lastSnapshot?.open_pack) {
        pushLog(tr("resolve_pack_before_buy"));
        return;
      }
      if (
        state.quickBuy &&
        !event.shiftKey &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey
      ) {
        scheduleAction(
          "buy_pack",
          { target: String(idx) },
          `Buy pack #${idx} (${pack.kind} ${pack.size})`
        );
      }
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    };
    el.addEventListener("click", clickHandler);
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        clickHandler(event);
      }
    });
    elements.shopPacks.appendChild(el);
  });

  voucherOffers.forEach((voucher, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.tabIndex = 0;
    if (state.selectedVoucher === idx) {
      el.classList.add("selected");
    }
    const suffix = voucher.effect ? ` - ${voucher.effect}` : "";
    el.textContent = `[${idx}] ${voucher.name || voucher.id} ($${shop.voucher_price})${suffix}`;
    const clickHandler = (event) => {
      state.selectedVoucher = idx;
      state.selectedShopCard = null;
      state.selectedShopPack = null;
      highlightSelection(elements.shopVouchers, idx);
      clearSelection(elements.shopCards);
      clearSelection(elements.shopPacks);
      if (state.lastSnapshot?.open_pack) {
        pushLog(tr("resolve_pack_before_buy"));
        return;
      }
      if (
        state.quickBuy &&
        !event.shiftKey &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey
      ) {
        scheduleAction("buy_voucher", { target: String(idx) }, `Buy voucher #${idx}`);
      }
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    };
    el.addEventListener("click", clickHandler);
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        clickHandler(event);
      }
    });
    elements.shopVouchers.appendChild(el);
  });
}

function renderInventory(run) {
  elements.invJokers.innerHTML = "";
  elements.invConsumables.innerHTML = "";
  if (state.selectedJoker != null && state.selectedJoker >= run.jokers.length) {
    state.selectedJoker = null;
  }
  if (state.selectedConsumable != null && state.selectedConsumable >= run.consumables.length) {
    state.selectedConsumable = null;
  }

  run.jokers.forEach((joker, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.tabIndex = 0;
    if (state.selectedJoker === idx) {
      el.classList.add("selected");
    }
    el.title = `rarity ${joker.rarity}${joker.edition ? ` | edition ${joker.edition}` : ""}`;
    el.innerHTML = `[${idx}] ${joker.name || joker.id} (${joker.rarity})`;
    el.addEventListener("click", () => {
      state.selectedJoker = idx;
      state.selectedConsumable = null;
      highlightSelection(elements.invJokers, idx);
      clearSelection(elements.invConsumables);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    });
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        state.selectedJoker = idx;
        state.selectedConsumable = null;
        highlightSelection(elements.invJokers, idx);
        clearSelection(elements.invConsumables);
        if (state.lastSnapshot) {
          updateSummaries(state.lastSnapshot);
          updateControls(state.lastSnapshot);
        }
      }
    });
    elements.invJokers.appendChild(el);
  });

  run.consumables.forEach((consumable, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.tabIndex = 0;
    if (state.selectedConsumable === idx) {
      el.classList.add("selected");
    }
    el.title = consumable.edition ? `edition ${consumable.edition}` : "";
    el.innerHTML = `[${idx}] ${consumable.kind} ${consumable.name || consumable.id}`;
    el.addEventListener("click", () => {
      state.selectedConsumable = idx;
      state.selectedJoker = null;
      highlightSelection(elements.invConsumables, idx);
      clearSelection(elements.invJokers);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    });
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        state.selectedConsumable = idx;
        state.selectedJoker = null;
        highlightSelection(elements.invConsumables, idx);
        clearSelection(elements.invJokers);
        if (state.lastSnapshot) {
          updateSummaries(state.lastSnapshot);
          updateControls(state.lastSnapshot);
        }
      }
    });
    elements.invConsumables.appendChild(el);
  });
}

function renderPack(openPack) {
  elements.packOptions.innerHTML = "";
  if (!openPack) {
    state.selectedPackOptions.clear();
    state.openPackKey = null;
    const empty = document.createElement("div");
    empty.textContent = tr("no_pack_inline");
    elements.packOptions.appendChild(empty);
    return;
  }
  const packKey = `${openPack.offer.kind}|${openPack.offer.size}|${openPack.offer.picks}|${openPack.options.length}`;
  if (state.openPackKey !== packKey) {
    state.selectedPackOptions.clear();
    state.openPackKey = packKey;
  }
  state.selectedPackOptions.forEach((idx) => {
    if (idx >= openPack.options.length) {
      state.selectedPackOptions.delete(idx);
    }
  });
  openPack.options.forEach((option, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.tabIndex = 0;
    el.textContent = `[${idx}] ${formatPackOption(option)}`;
    el.addEventListener("click", () => togglePackSelection(idx, openPack.offer.picks, el));
    el.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        togglePackSelection(idx, openPack.offer.picks, el);
      }
    });
    if (state.selectedPackOptions.has(idx)) {
      el.classList.add("selected");
    }
    elements.packOptions.appendChild(el);
  });
}

function renderScore(breakdown) {
  elements.scoreBreakdown.innerHTML = "";
  if (!breakdown) {
    const empty = document.createElement("div");
    empty.textContent = tr("no_score");
    elements.scoreBreakdown.appendChild(empty);
    return;
  }
  const summary = document.createElement("div");
  summary.className = "score-row";
  summary.innerHTML = `
    <div><strong>Hand:</strong> ${breakdown.hand}</div>
    <div><strong>Base:</strong> ${breakdown.base_chips} chips × ${breakdown.base_mult.toFixed(2)}</div>
    <div><strong>Rank chips:</strong> ${breakdown.rank_chips}</div>
    <div><strong>Total:</strong> ${breakdown.total_chips} chips × ${breakdown.total_mult.toFixed(
      2
    )} = ${breakdown.total_score}</div>
    <div><strong>Scoring indices:</strong> ${breakdown.scoring_indices.join(", ")}</div>
  `;
  elements.scoreBreakdown.appendChild(summary);

  if (breakdown.played_cards && breakdown.played_cards.length > 0) {
    const list = breakdown.played_cards
      .map((card, idx) => `${idx}: ${formatCard(card)} ${formatMods(card)}`)
      .join("<br/>");
    elements.scoreBreakdown.appendChild(
      renderScoreSection("Played cards", "played", list)
    );
  }

  if (breakdown.scoring_cards && breakdown.scoring_cards.length > 0) {
    const list = breakdown.scoring_cards
      .map(
        (entry) =>
          `${entry.index}: ${formatCard(entry.card)} ${formatMods(entry.card)} ⇒ ${entry.chips}`
      )
      .join("<br/>");
    elements.scoreBreakdown.appendChild(
      renderScoreSection("Scoring cards", "scoring", list)
    );
  }

  if (breakdown.steps && breakdown.steps.length > 0) {
    const list = breakdown.steps
      .map((step, idx) => {
        const deltaChips = step.after_chips - step.before_chips;
        const deltaMult = step.after_mult - step.before_mult;
        const deltaText = `Δchips ${formatSigned(deltaChips)}, Δmult ${formatSignedFloat(
          deltaMult
        )}`;
        return `${idx + 1}. ${step.source} | ${step.effect} | ${step.before_chips}×${step.before_mult.toFixed(
          2
        )} → ${step.after_chips}×${step.after_mult.toFixed(2)} (${deltaText})`;
      })
      .join("<br/>");
    elements.scoreBreakdown.appendChild(
      renderScoreSection("Effect steps", "steps", list)
    );
  }
}

function renderScoreSection(title, key, bodyHtml) {
  if (!(key in state.scoreSections)) {
    state.scoreSections[key] = true;
  }
  const open = state.scoreSections[key];
  const section = document.createElement("div");
  section.className = "score-section";
  const header = document.createElement("button");
  header.type = "button";
  header.className = "score-toggle";
  header.textContent = `${open ? "▾" : "▸"} ${title}`;
  header.addEventListener("click", () => {
    state.scoreSections[key] = !state.scoreSections[key];
    if (state.lastSnapshot) {
      renderScore(state.lastSnapshot.last_breakdown);
    }
  });
  const body = document.createElement("div");
  body.className = "score-body";
  body.innerHTML = bodyHtml;
  if (!open) {
    body.classList.add("hidden");
  }
  section.appendChild(header);
  section.appendChild(body);
  return section;
}

function renderLevels(levels) {
  elements.levels.innerHTML = "";
  levels.forEach((level) => {
    const el = document.createElement("div");
    el.className = "badge";
    el.textContent = `${level.hand}: ${level.level}`;
    elements.levels.appendChild(el);
  });
}

function renderTags(tags, dupNext, dupExclude) {
  elements.tags.innerHTML = "";
  if (tags.length === 0 && !dupNext) {
    elements.tags.textContent = "Tags: none";
    return;
  }
  if (tags.length > 0) {
    const tagEl = document.createElement("div");
    tagEl.textContent = `Tags: ${tags.join(", ")}`;
    elements.tags.appendChild(tagEl);
  }
  if (dupNext) {
    const dupEl = document.createElement("div");
    dupEl.textContent = dupExclude
      ? `Duplicate next tag (excluding ${dupExclude})`
      : "Duplicate next tag";
    elements.tags.appendChild(dupEl);
  }
}

function renderLog() {
  elements.log.innerHTML = "";
  state.logLines.forEach((line) => {
    const el = document.createElement("div");
    el.className = "log-line";
    if (line.includes("error:")) {
      el.classList.add("error");
    }
    el.textContent = line;
    elements.log.appendChild(el);
  });
  elements.log.scrollTop = elements.log.scrollHeight;
}

function pushLog(line) {
  const timestamp = new Date().toLocaleTimeString();
  state.logLines.push(`[${timestamp}] ${line}`);
  if (state.logLines.length > 200) {
    state.logLines.shift();
  }
}

function updateQuickBuyButton() {
  const button = actionButtons.toggle_quick_buy;
  if (!button) return;
  button.textContent = state.quickBuy ? tr("quick_buy_on_btn") : tr("quick_buy_off_btn");
}

function scheduleAction(action, payload, label) {
  cancelPendingAction();
  const deadline = Date.now() + QUICK_BUY_DELAY_MS;
  const pending = {
    action,
    payload,
    label,
    deadline,
    timeoutId: null,
    intervalId: null,
  };
  pending.timeoutId = setTimeout(() => {
    clearInterval(pending.intervalId);
    state.pendingAction = null;
    updateToast();
    callAction(action, payload);
  }, QUICK_BUY_DELAY_MS);
  pending.intervalId = setInterval(updateToast, 200);
  state.pendingAction = pending;
  updateToast();
  if (state.lastSnapshot) {
    updateControls(state.lastSnapshot);
  }
}

function cancelPendingAction() {
  if (!state.pendingAction) {
    return;
  }
  clearTimeout(state.pendingAction.timeoutId);
  clearInterval(state.pendingAction.intervalId);
  state.pendingAction = null;
  updateToast();
  if (state.lastSnapshot) {
    updateControls(state.lastSnapshot);
  }
}

function updateToast() {
  elements.toastArea.innerHTML = "";
  if (!state.pendingAction) {
    return;
  }
  const remaining = Math.max(0, state.pendingAction.deadline - Date.now());
  const seconds = (remaining / 1000).toFixed(1);
  const toast = document.createElement("div");
  toast.className = "toast";
  const label = document.createElement("div");
  label.textContent = `${state.pendingAction.label} ${tr("buy_in")} ${seconds}s`;
  const undo = document.createElement("button");
  undo.type = "button";
  undo.textContent = tr("undo_key");
  undo.addEventListener("click", () => cancelPendingAction());
  toast.appendChild(label);
  toast.appendChild(undo);
  elements.toastArea.appendChild(toast);
}

function runPurchaseAction(action, payload, label) {
  if (state.quickBuy) {
    scheduleAction(action, payload, label);
    return;
  }
  callAction(action, payload);
}

function handleBuySelected() {
  if (!state.lastSnapshot || !state.lastSnapshot.state.shop) {
    pushLog(tr("shop_unavailable"));
    return;
  }
  if (state.lastSnapshot.open_pack) {
    pushLog(tr("resolve_pack_first"));
    return;
  }
  if (state.selectedShopCard != null) {
    runPurchaseAction(
      "buy_card",
      { target: String(state.selectedShopCard) },
      `Buy card #${state.selectedShopCard}`
    );
    return;
  }
  if (state.selectedShopPack != null) {
    runPurchaseAction(
      "buy_pack",
      { target: String(state.selectedShopPack) },
      `Buy pack #${state.selectedShopPack}`
    );
    return;
  }
  if (state.selectedVoucher != null) {
    runPurchaseAction(
      "buy_voucher",
      { target: String(state.selectedVoucher) },
      `Buy voucher #${state.selectedVoucher}`
    );
    return;
  }
  pushLog(tr("select_shop_first"));
}

function isBlindCleared(run) {
  return run.target > 0 && run.score >= run.target;
}

function nextActionHint(snapshot) {
  const run = snapshot.state;
  if (snapshot.open_pack) {
    return tr("next_action_pack");
  }
  if (run.phase === "Setup") {
    return tr("next_action_setup");
  }
  if (run.phase === "Shop") {
    return tr("next_action_shop");
  }
  if (isBlindCleared(run)) {
    return tr("next_action_cleared");
  }
  if (run.phase === "Deal") {
    return run.blind === "Boss"
      ? tr("next_action_deal")
      : tr("next_action_deal_skip");
  }
  if (run.phase === "Play") {
    return tr("next_action_play");
  }
  return tr("next_action_generic");
}

function formatSigned(value) {
  return value >= 0 ? `+${value}` : `${value}`;
}

function formatSignedFloat(value) {
  const fixed = value.toFixed(2);
  return value >= 0 ? `+${fixed}` : fixed;
}

function toggleSetSelection(set, idx, element) {
  if (set.has(idx)) {
    set.delete(idx);
    element.classList.remove("selected");
  } else {
    set.add(idx);
    element.classList.add("selected");
  }
  if (state.lastSnapshot) {
    updateSummaries(state.lastSnapshot);
    updateControls(state.lastSnapshot);
  }
}

function toggleHandSelection(idx, element) {
  if (!state.selectedHand.has(idx) && state.selectedHand.size >= 5) {
    pushLog(tr("max_hand_selected"));
    return;
  }
  toggleSetSelection(state.selectedHand, idx, element);
}

function togglePackSelection(idx, picks, element) {
  if (!state.selectedPackOptions.has(idx) && state.selectedPackOptions.size >= picks) {
    pushLog(tr("pack_pick_limit", { count: picks }));
    return;
  }
  toggleSetSelection(state.selectedPackOptions, idx, element);
}

function highlightSelection(container, index) {
  Array.from(container.children).forEach((child, idx) => {
    if (idx === index) {
      child.classList.add("selected");
    } else {
      child.classList.remove("selected");
    }
  });
}

function clearSelection(container) {
  Array.from(container.children).forEach((child) => {
    child.classList.remove("selected");
  });
}

function formatCard(card) {
  if (card.face_down) {
    return "??";
  }
  return `${rankShort(card.rank)}${suitShort(card.suit)}`;
}

function formatCardMeta(card, idx, scoringValue) {
  const parts = [`#${idx}`];
  if (scoringValue != null) parts.push("score:" + scoringValue);
  const mods = formatMods(card);
  if (mods) parts.push(mods);
  const value = formatValue(card);
  if (value) parts.push(value);
  return parts.join(" | ");
}

function formatCardTooltip(card, idx) {
  const lines = [`index: ${idx}`];
  if (card.face_down) {
    lines.push("face down");
    return lines.join(" | ");
  }
  lines.push(`rank: ${card.rank}`);
  lines.push(`suit: ${card.suit}`);
  if (card.enhancement) lines.push(`enhancement: ${card.enhancement}`);
  if (card.edition) lines.push(`edition: ${card.edition}`);
  if (card.seal) lines.push(`seal: ${card.seal}`);
  if (card.bonus_chips) lines.push(`bonus: ${card.bonus_chips}`);
  const value = cardValue(card);
  if (value) lines.push(`value: ${value}`);
  return lines.join(" | ");
}

function formatMods(card) {
  const mods = [];
  if (card.enhancement) mods.push(card.enhancement);
  if (card.edition) mods.push(card.edition);
  if (card.seal) mods.push(card.seal);
  if (card.bonus_chips !== 0) mods.push(`bonus:${card.bonus_chips}`);
  return mods.join(", ");
}

function formatPackOption(option) {
  if (option.kind === "Joker") {
    return `Joker ${option.value.name || option.value.id}`;
  }
  if (option.kind === "Consumable") {
    return `${option.value.kind} ${option.value.name || option.value.id}`;
  }
  if (option.kind === "PlayingCard") {
    return `Card ${formatCard(option.value)}`;
  }
  return tr("unknown");
}

function formatEvent(event) {
  if (!event || typeof event !== "object") {
    return String(event);
  }
  if (event.BlindStarted) {
    const e = event.BlindStarted;
    return "blind started: ante " + e.ante + " " + e.blind + " target " + e.target + " hands " + e.hands + " discards " + e.discards;
  }
  if (event.BlindSkipped) {
    const e = event.BlindSkipped;
    return "blind skipped: ante " + e.ante + " " + e.blind + " tag " + (e.tag ?? "none");
  }
  if (event.HandDealt) {
    return "hand dealt: " + event.HandDealt.count + " cards";
  }
  if (event.HandScored) {
    const e = event.HandScored;
    return "hand scored: " + e.hand + " " + e.chips + "×" + e.mult.toFixed(2) + " = " + e.total;
  }
  if (event.ShopEntered) {
    const e = event.ShopEntered;
    return "shop entered: offers " + e.offers + " reroll " + e.reroll_cost + (e.reentered ? " (reenter)" : "");
  }
  if (event.ShopRerolled) {
    const e = event.ShopRerolled;
    return "shop reroll: offers " + e.offers + " reroll " + e.reroll_cost + " cost " + e.cost + " money " + e.money;
  }
  if (event.ShopBought) {
    const e = event.ShopBought;
    return "shop bought: " + e.offer + " cost " + e.cost + " money " + e.money;
  }
  if (event.PackOpened) {
    const e = event.PackOpened;
    return "pack opened: " + e.kind + " options " + e.options + " picks " + e.picks;
  }
  if (event.PackChosen) {
    return "pack chosen: picks " + event.PackChosen.picks;
  }
  if (event.JokerSold) {
    const e = event.JokerSold;
    return "joker sold: " + e.id + " value " + e.sell_value + " money " + e.money;
  }
  if (event.BlindCleared) {
    const e = event.BlindCleared;
    return "blind cleared: score " + e.score + " reward " + e.reward + " money " + e.money;
  }
  if (event.BlindFailed) {
    return "blind failed: score " + event.BlindFailed.score;
  }
  return JSON.stringify(event);
}

function formatValue(card) {
  const value = cardValue(card);
  return value === 0 ? "" : `value:${value}`;
}

function cardValue(card) {
  if (card.enhancement === "Stone") {
    return 0;
  }
  const base = state.rankChipMap.get(card.rank) || 0;
  return base + (card.bonus_chips || 0);
}

function rankShort(rank) {
  switch (rank) {
    case "Ace":
      return "A";
    case "King":
      return "K";
    case "Queen":
      return "Q";
    case "Jack":
      return "J";
    case "Ten":
      return "T";
    case "Nine":
      return "9";
    case "Eight":
      return "8";
    case "Seven":
      return "7";
    case "Six":
      return "6";
    case "Five":
      return "5";
    case "Four":
      return "4";
    case "Three":
      return "3";
    case "Two":
      return "2";
    case "Joker":
      return "Jk";
    default:
      return "?";
  }
}

function rankValue(rank) {
  switch (rank) {
    case "Ace":
      return 14;
    case "King":
      return 13;
    case "Queen":
      return 12;
    case "Jack":
      return 11;
    case "Ten":
      return 10;
    case "Nine":
      return 9;
    case "Eight":
      return 8;
    case "Seven":
      return 7;
    case "Six":
      return 6;
    case "Five":
      return 5;
    case "Four":
      return 4;
    case "Three":
      return 3;
    case "Two":
      return 2;
    case "Joker":
      return 0;
    default:
      return 0;
  }
}

function suitShort(suit) {
  switch (suit) {
    case "Spades":
      return "S";
    case "Hearts":
      return "H";
    case "Clubs":
      return "C";
    case "Diamonds":
      return "D";
    case "Wild":
      return "W";
    default:
      return "?";
  }
}

function isEditableTarget(target) {
  if (!target) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

document.addEventListener("keydown", (event) => {
  if (isEditableTarget(event.target)) {
    return;
  }
  if (event.repeat) {
    return;
  }
  if (event.shiftKey && event.key.toLowerCase() === "r") {
    event.preventDefault();
    handleAction("reset");
    return;
  }
  if (event.shiftKey && event.key.toLowerCase() === "c") {
    event.preventDefault();
    handleAction("clear_pack");
    return;
  }
  switch (event.key.toLowerCase()) {
    case "s":
      event.preventDefault();
      handleAction("start");
      break;
    case "d":
      event.preventDefault();
      handleAction("deal");
      break;
    case "p":
      event.preventDefault();
      handleAction("play");
      break;
    case "x":
      event.preventDefault();
      handleAction("discard");
      break;
    case "c":
      event.preventDefault();
      handleAction("clear_hand");
      break;
    case "o":
      event.preventDefault();
      handleAction("enter_shop");
      break;
    case "r":
      event.preventDefault();
      handleAction("reroll");
      break;
    case "b":
      event.preventDefault();
      handleAction("buy_selected");
      break;
    case "l":
      event.preventDefault();
      handleAction("leave_shop");
      break;
    case "q":
      event.preventDefault();
      handleAction("toggle_quick_buy");
      break;
    case "k":
      event.preventDefault();
      handleAction("pick_pack");
      break;
    case "y":
      event.preventDefault();
      handleAction("skip_pack");
      break;
    case "u":
      event.preventDefault();
      handleAction("use_consumable");
      break;
    case "j":
      event.preventDefault();
      handleAction("sell_joker");
      break;
    case "n":
      event.preventDefault();
      handleAction("next_blind");
      break;
    case "g":
      event.preventDefault();
      handleAction("skip_blind");
      break;
    case "z":
      event.preventDefault();
      handleAction("undo_pending");
      break;
    default:
      break;
  }
});

const initialLocalSave = readLocalSave();
if (initialLocalSave) {
  applySavedPrefs(initialLocalSave);
  if (Array.isArray(initialLocalSave.actions)) {
    state.actionHistory = initialLocalSave.actions
      .filter((step) => step && typeof step.action === "string")
      .map((step) => ({
        action: step.action,
        indices: Array.isArray(step.indices)
          ? step.indices.filter((value) => Number.isInteger(value) && value >= 0)
          : [],
        target: step.target == null ? null : String(step.target),
      }));
  }
}
updateQuickBuyButton();

fetchState().catch((err) => {
  pushLog(`init error: ${err}`);
  renderLog();
});
