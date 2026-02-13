const state = {
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
};

const elements = {
  status: document.getElementById("status"),
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
  sortKey: document.getElementById("sort-key"),
  sortDir: document.getElementById("sort-dir"),
};

const buttons = document.querySelectorAll("[data-action]");
const actionButtons = {};
buttons.forEach((button) => {
  actionButtons[button.dataset.action] = button;
  button.addEventListener("click", () => handleAction(button.dataset.action));
});

elements.sortKey.addEventListener("change", () => {
  state.sortKey = elements.sortKey.value;
  if (state.lastSnapshot) {
    render(state.lastSnapshot);
  }
});

elements.sortDir.addEventListener("click", () => {
  state.sortDir = state.sortDir === "asc" ? "desc" : "asc";
  elements.sortDir.textContent = state.sortDir === "asc" ? "Asc" : "Desc";
  if (state.lastSnapshot) {
    render(state.lastSnapshot);
  }
});

async function fetchState() {
  const res = await fetch("/api/state");
  const data = await res.json();
  render(data);
}

async function callAction(action, payload = {}) {
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
  }
  render(data);
}

function handleAction(action) {
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
        pushLog("select a consumable");
        break;
      }
      callAction("use_consumable", {
        target: String(state.selectedConsumable),
        indices: Array.from(state.selectedHand).sort(),
      });
      break;
    case "sell_joker":
      if (state.selectedJoker == null) {
        pushLog("select a joker");
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
    case "next_blind":
      callAction("next_blind");
      break;
    case "reset":
      callAction("reset");
      break;
    case "clear_log":
      state.logLines = [];
      renderLog();
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
  const phase = snapshot.state.phase;
  const hasHandSelection = state.selectedHand.size > 0;
  const hasPackSelection = state.selectedPackOptions.size > 0;
  const hasConsumable = state.selectedConsumable != null;
  const hasJoker = state.selectedJoker != null;
  setActionEnabled("play", phase === "Play" && hasHandSelection);
  setActionEnabled(
    "discard",
    phase === "Play" && hasHandSelection && snapshot.state.discards_left > 0
  );
  setActionEnabled("deal", phase === "Deal");
  setActionEnabled("start", phase === "Setup" || phase === "Shop");
  setActionEnabled("enter_shop", phase !== "Shop");
  setActionEnabled("leave_shop", phase === "Shop");
  setActionEnabled("reroll", phase === "Shop");
  setActionEnabled("use_consumable", hasConsumable);
  setActionEnabled("sell_joker", hasJoker);
  setActionEnabled("pick_pack", snapshot.open_pack && hasPackSelection);
  setActionEnabled("skip_pack", snapshot.open_pack != null);
  setActionEnabled("clear_hand", state.selectedHand.size > 0);
  setActionEnabled("clear_pack", state.selectedPackOptions.size > 0);
}

function render(data) {
  state.lastSnapshot = data;
  state.rankChipMap = new Map(
    (data.state.rank_chips || []).map((entry) => [entry.rank, entry.chips])
  );
  if (data.events && data.events.length > 0) {
    data.events.forEach((event) => pushLog(JSON.stringify(event)));
  }
  renderStatus(data.state);
  renderHand(data.state.hand);
  renderShop(data.state.shop);
  renderInventory(data.state);
  renderPack(data.open_pack);
  renderScore(data.last_breakdown);
  renderLevels(data.state.hand_levels);
  renderTags(data.state.tags, data.state.duplicate_next_tag, data.state.duplicate_tag_exclude);
  updateSummaries(data);
  updateControls(data);
  renderLog();
}

function renderStatus(run) {
  const fields = elements.status.querySelectorAll("[data-field]");
  fields.forEach((el) => {
    const key = el.dataset.field;
    el.textContent = run[key] ?? "-";
  });
}

function updateSummaries(snapshot) {
  const run = snapshot.state;
  const handCount = run.hand.length;
  elements.handSummary.textContent = `Selected: ${state.selectedHand.size} | Hand: ${handCount} | Hands: ${run.hands_left}/${run.hands_max} | Discards: ${run.discards_left}/${run.discards_max}`;

  if (!run.shop) {
    elements.shopSummary.textContent = "Shop closed.";
  } else {
    let selected = "none";
    if (state.selectedShopCard != null) {
      selected = `card #${state.selectedShopCard}`;
    } else if (state.selectedShopPack != null) {
      selected = `pack #${state.selectedShopPack}`;
    } else if (state.selectedVoucher != null) {
      selected = `voucher #${state.selectedVoucher}`;
    }
    elements.shopSummary.textContent = `Cards: ${run.shop.cards.length} | Packs: ${run.shop.packs.length} | Vouchers: ${run.shop.vouchers} | Reroll: ${run.shop.reroll_cost} | Selected: ${selected}`;
  }

  elements.invSummary.textContent = `Jokers: ${run.jokers.length} | Consumables: ${run.consumables.length} | Selected: ${
    state.selectedJoker != null ? `joker #${state.selectedJoker}` : state.selectedConsumable != null ? `consumable #${state.selectedConsumable}` : "none"
  }`;

  if (!snapshot.open_pack) {
    elements.packSummary.textContent = "No open pack.";
  } else {
    const picks = snapshot.open_pack.offer.picks;
    elements.packSummary.textContent = `Options: ${snapshot.open_pack.options.length} | Picks: ${picks} | Selected: ${state.selectedPackOptions.size}`;
  }

  elements.logSummary.textContent = `Entries: ${state.logLines.length}`;
}

function renderHand(hand) {
  elements.hand.innerHTML = "";
  state.selectedHand.forEach((idx) => {
    if (idx >= hand.length) {
      state.selectedHand.delete(idx);
    }
  });
  const entries = hand.map((card, idx) => ({ card, idx }));
  const sorted = sortHandEntries(entries);
  sorted.forEach(({ card, idx }) => {
    const el = document.createElement("div");
    el.className = "card";
    if (state.selectedHand.has(idx)) {
      el.classList.add("selected");
    }
    el.title = formatCardTooltip(card, idx);
    el.innerHTML = `
      <div class="title">${formatCard(card)}</div>
      <div class="meta">${formatCardMeta(card, idx)}</div>
    `;
    el.addEventListener("click", () => toggleSelection(state.selectedHand, idx, el));
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
  if (state.selectedVoucher != null && state.selectedVoucher >= shop.vouchers) {
    state.selectedVoucher = null;
  }

  shop.cards.forEach((card, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    if (state.selectedShopCard === idx) {
      el.classList.add("selected");
    }
    el.title = `price ${card.price}${card.rarity ? ` | rarity ${card.rarity}` : ""}${
      card.edition ? ` | edition ${card.edition}` : ""
    }`;
    el.innerHTML = `[${idx}] ${card.kind} ${card.item_id} (${card.price})`;
    el.addEventListener("click", () => {
      state.selectedShopCard = idx;
      state.selectedShopPack = null;
      state.selectedVoucher = null;
      highlightSelection(elements.shopCards, idx);
      clearSelection(elements.shopPacks);
      clearSelection(elements.shopVouchers);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    });
    el.addEventListener("dblclick", () => {
      callAction("buy_card", { target: String(idx) });
    });
    elements.shopCards.appendChild(el);
  });

  shop.packs.forEach((pack, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    if (state.selectedShopPack === idx) {
      el.classList.add("selected");
    }
    el.title = `options ${pack.options} | picks ${pack.picks} | price ${pack.price}`;
    el.innerHTML = `[${idx}] ${pack.kind} ${pack.size} (pick ${pack.picks}) ${pack.price}`;
    el.addEventListener("click", () => {
      state.selectedShopPack = idx;
      state.selectedShopCard = null;
      state.selectedVoucher = null;
      highlightSelection(elements.shopPacks, idx);
      clearSelection(elements.shopCards);
      clearSelection(elements.shopVouchers);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    });
    el.addEventListener("dblclick", () => {
      callAction("buy_pack", { target: String(idx) });
    });
    elements.shopPacks.appendChild(el);
  });

  for (let idx = 0; idx < shop.vouchers; idx += 1) {
    const el = document.createElement("div");
    el.className = "list-item";
    if (state.selectedVoucher === idx) {
      el.classList.add("selected");
    }
    el.textContent = `[${idx}] Voucher`;
    el.addEventListener("click", () => {
      state.selectedVoucher = idx;
      state.selectedShopCard = null;
      state.selectedShopPack = null;
      highlightSelection(elements.shopVouchers, idx);
      clearSelection(elements.shopCards);
      clearSelection(elements.shopPacks);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    });
    el.addEventListener("dblclick", () => {
      callAction("buy_voucher", { target: String(idx) });
    });
    elements.shopVouchers.appendChild(el);
  }
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
    if (state.selectedJoker === idx) {
      el.classList.add("selected");
    }
    el.title = `rarity ${joker.rarity}${joker.edition ? ` | edition ${joker.edition}` : ""}`;
    el.innerHTML = `[${idx}] ${joker.id} (${joker.rarity})`;
    el.addEventListener("click", () => {
      state.selectedJoker = idx;
      highlightSelection(elements.invJokers, idx);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
      }
    });
    elements.invJokers.appendChild(el);
  });

  run.consumables.forEach((consumable, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    if (state.selectedConsumable === idx) {
      el.classList.add("selected");
    }
    el.title = consumable.edition ? `edition ${consumable.edition}` : "";
    el.innerHTML = `[${idx}] ${consumable.kind} ${consumable.id}`;
    el.addEventListener("click", () => {
      state.selectedConsumable = idx;
      highlightSelection(elements.invConsumables, idx);
      if (state.lastSnapshot) {
        updateSummaries(state.lastSnapshot);
        updateControls(state.lastSnapshot);
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
    empty.textContent = "No open pack.";
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
    el.textContent = `[${idx}] ${formatPackOption(option)}`;
    el.addEventListener("click", () => toggleSelection(state.selectedPackOptions, idx, el));
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
    empty.textContent = "No scoring yet.";
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
    const played = document.createElement("div");
    played.className = "score-row";
    const list = breakdown.played_cards
      .map((card, idx) => `${idx}: ${formatCard(card)} ${formatMods(card)}`)
      .join("<br/>");
    played.innerHTML = `<div><strong>Played cards:</strong></div><div>${list}</div>`;
    elements.scoreBreakdown.appendChild(played);
  }

  if (breakdown.scoring_cards && breakdown.scoring_cards.length > 0) {
    const scoring = document.createElement("div");
    scoring.className = "score-row";
    const list = breakdown.scoring_cards
      .map(
        (entry) =>
          `${entry.index}: ${formatCard(entry.card)} ${formatMods(entry.card)} ⇒ ${entry.chips}`
      )
      .join("<br/>");
    scoring.innerHTML = `<div><strong>Scoring cards:</strong></div><div>${list}</div>`;
    elements.scoreBreakdown.appendChild(scoring);
  }

  if (breakdown.steps && breakdown.steps.length > 0) {
    const steps = document.createElement("div");
    steps.className = "score-row";
    const list = breakdown.steps
      .map(
        (step, idx) =>
          `${idx + 1}. ${step.source} | ${step.effect} | ${step.before_chips}×${step.before_mult.toFixed(
            2
          )} → ${step.after_chips}×${step.after_mult.toFixed(2)}`
      )
      .join("<br/>");
    steps.innerHTML = `<div><strong>Effect steps:</strong></div><div>${list}</div>`;
    elements.scoreBreakdown.appendChild(steps);
  }
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

function toggleSelection(set, idx, element) {
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

function formatCardMeta(card, idx) {
  const parts = [`#${idx}`];
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
    return `Joker ${option.value}`;
  }
  if (option.kind === "Consumable") {
    return `${option.value.kind} ${option.value.id}`;
  }
  if (option.kind === "PlayingCard") {
    return `Card ${formatCard(option.value)}`;
  }
  return "Unknown";
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

fetchState().catch((err) => {
  pushLog(`init error: ${err}`);
  renderLog();
});
