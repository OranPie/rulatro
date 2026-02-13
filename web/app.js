const state = {
  selectedHand: new Set(),
  selectedShopCard: null,
  selectedShopPack: null,
  selectedVoucher: null,
  selectedJoker: null,
  selectedConsumable: null,
  selectedPackOptions: new Set(),
  logLines: [],
};

const elements = {
  status: document.getElementById("status"),
  hand: document.getElementById("hand"),
  shopCards: document.getElementById("shop-cards"),
  shopPacks: document.getElementById("shop-packs"),
  shopVouchers: document.getElementById("shop-vouchers"),
  invJokers: document.getElementById("inv-jokers"),
  invConsumables: document.getElementById("inv-consumables"),
  packOptions: document.getElementById("pack-options"),
  scoreBreakdown: document.getElementById("score-breakdown"),
  levels: document.getElementById("levels"),
  tags: document.getElementById("tags"),
  log: document.getElementById("log"),
};

const buttons = document.querySelectorAll("[data-action]");
buttons.forEach((button) => {
  button.addEventListener("click", () => handleAction(button.dataset.action));
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
    default:
      break;
  }
}

function render(data) {
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
  renderLog();
}

function renderStatus(run) {
  const fields = elements.status.querySelectorAll("[data-field]");
  fields.forEach((el) => {
    const key = el.dataset.field;
    el.textContent = run[key] ?? "-";
  });
}

function renderHand(hand) {
  elements.hand.innerHTML = "";
  state.selectedHand.forEach((idx) => {
    if (idx >= hand.length) {
      state.selectedHand.delete(idx);
    }
  });
  hand.forEach((card, idx) => {
    const el = document.createElement("div");
    el.className = "card";
    if (state.selectedHand.has(idx)) {
      el.classList.add("selected");
    }
    el.innerHTML = `
      <div class="title">${formatCard(card)}</div>
      <div class="meta">#${idx} ${formatMods(card)}</div>
    `;
    el.addEventListener("click", () => toggleSelection(state.selectedHand, idx, el));
    elements.hand.appendChild(el);
  });
}

function renderShop(shop) {
  elements.shopCards.innerHTML = "";
  elements.shopPacks.innerHTML = "";
  elements.shopVouchers.innerHTML = "";
  state.selectedShopCard = null;
  state.selectedShopPack = null;
  state.selectedVoucher = null;

  if (!shop) {
    return;
  }

  shop.cards.forEach((card, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.innerHTML = `[${idx}] ${card.kind} ${card.item_id} (${card.price})`;
    el.addEventListener("click", () => {
      state.selectedShopCard = idx;
      state.selectedShopPack = null;
      state.selectedVoucher = null;
      highlightSelection(elements.shopCards, idx);
    });
    el.addEventListener("dblclick", () => {
      callAction("buy_card", { target: String(idx) });
    });
    elements.shopCards.appendChild(el);
  });

  shop.packs.forEach((pack, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.innerHTML = `[${idx}] ${pack.kind} ${pack.size} (pick ${pack.picks}) ${pack.price}`;
    el.addEventListener("click", () => {
      state.selectedShopPack = idx;
      state.selectedShopCard = null;
      state.selectedVoucher = null;
      highlightSelection(elements.shopPacks, idx);
    });
    el.addEventListener("dblclick", () => {
      callAction("buy_pack", { target: String(idx) });
    });
    elements.shopPacks.appendChild(el);
  });

  for (let idx = 0; idx < shop.vouchers; idx += 1) {
    const el = document.createElement("div");
    el.className = "list-item";
    el.textContent = `[${idx}] Voucher`;
    el.addEventListener("click", () => {
      state.selectedVoucher = idx;
      state.selectedShopCard = null;
      state.selectedShopPack = null;
      highlightSelection(elements.shopVouchers, idx);
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
  state.selectedJoker = null;
  state.selectedConsumable = null;

  run.jokers.forEach((joker, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.innerHTML = `[${idx}] ${joker.id} (${joker.rarity})`;
    el.addEventListener("click", () => {
      state.selectedJoker = idx;
      highlightSelection(elements.invJokers, idx);
    });
    elements.invJokers.appendChild(el);
  });

  run.consumables.forEach((consumable, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.innerHTML = `[${idx}] ${consumable.kind} ${consumable.id}`;
    el.addEventListener("click", () => {
      state.selectedConsumable = idx;
      highlightSelection(elements.invConsumables, idx);
    });
    elements.invConsumables.appendChild(el);
  });
}

function renderPack(openPack) {
  elements.packOptions.innerHTML = "";
  state.selectedPackOptions.clear();
  if (!openPack) {
    const empty = document.createElement("div");
    empty.textContent = "No open pack.";
    elements.packOptions.appendChild(empty);
    return;
  }
  openPack.options.forEach((option, idx) => {
    const el = document.createElement("div");
    el.className = "list-item";
    el.textContent = `[${idx}] ${formatPackOption(option)}`;
    el.addEventListener("click", () => toggleSelection(state.selectedPackOptions, idx, el));
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
  elements.log.textContent = state.logLines.join("\n");
  elements.log.scrollTop = elements.log.scrollHeight;
}

function pushLog(line) {
  state.logLines.push(line);
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

function formatCard(card) {
  if (card.face_down) {
    return "??";
  }
  return `${rankShort(card.rank)}${suitShort(card.suit)}`;
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
