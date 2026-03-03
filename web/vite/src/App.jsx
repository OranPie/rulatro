import { useCallback, useEffect, useMemo, useState } from "react";

const MAX_HAND_SELECTION = 5;
const MAX_LOG_ENTRIES = 260;

const COPY = {
  en_US: {
    subtitle: "Friendly WebUI (React + Vite)",
    loading: "Loading game state...",
    nextStep: "Next best step",
    quickGuide: "Quick guide",
    quickGuideBody:
      "Select cards and press Play. In shop, click an offer to focus it, then Buy Selected.",
    controls: "Controls",
    flowActions: "Flow",
    economyActions: "Shop and economy",
    inventoryActions: "Inventory and system",
    hand: "Hand",
    scoring: "Scoring",
    shop: "Shop",
    inventory: "Inventory",
    pack: "Pack",
    inspector: "Inspector",
    events: "Events",
    noScoring: "No scoring details yet.",
    noShop: "Shop is not open.",
    noPack: "No pack is currently open.",
    noEvents: "No events yet.",
    noBossEffects: "No active boss effects.",
    noActiveVouchers: "No active vouchers.",
    selectCardsToPlay: "Pick at least one hand card first.",
    selectOfferFirst: "Select a shop offer first.",
    resolvePackFirst: "Resolve the open pack before buying something else.",
    selectPackOptionFirst: "Pick at least one pack option first.",
    selectConsumableFirst: "Select a consumable first.",
    selectJokerFirst: "Select a joker first.",
    maxHandSelected: "You can select up to five hand cards.",
    packPickLimit: "This pack allows up to {count} picks.",
    selected: "Selected",
    suggestions: "Suggestions",
    openInspectorHint: "Click any row/card to inspect details.",
    runningAction: "Running",
    quickBuyOn: "Quick buy: On",
    quickBuyOff: "Quick buy: Off",
    shiftClickHint: "Shift+click shop offers to focus without buying.",
    sort: "Sort",
    clear: "Clear",
    scoreTotal: "Total",
    baseChips: "Base chips",
    baseMult: "Base mult",
    rankChips: "Rank chips",
    showSteps: "Show steps",
    hideSteps: "Hide steps",
    reroll: "Reroll",
    cards: "Cards",
    packs: "Packs",
    vouchers: "Vouchers",
    jokers: "Jokers",
    consumables: "Consumables",
    bossEffects: "Boss effects",
    activeVouchers: "Active vouchers",
    refresh: "Refresh",
    start: "Start",
    deal: "Deal",
    play: "Play",
    discard: "Discard",
    skipBlind: "Skip blind",
    nextBlind: "Next blind",
    enterShop: "Enter shop",
    leaveShop: "Leave shop",
    buySelected: "Buy selected",
    pickPack: "Pick pack",
    skipPack: "Skip pack",
    useConsumable: "Use consumable",
    sellJoker: "Sell joker",
    reset: "Reset",
    unknown: "Unknown",
    handCard: "Hand card",
    shopOffer: "Shop offer",
    inventoryItem: "Inventory item",
    packOption: "Pack option",
    stateSummary: "Run summary",
    shortcuts: "Shortcuts",
    shortcutsBody: "S start, D deal, P play, X discard, O shop, B buy, R reroll, N next",
    logEntries: "entries",
  },
  zh_CN: {
    subtitle: "友好 WebUI (React + Vite)",
    loading: "正在加载对局状态...",
    nextStep: "下一步建议",
    quickGuide: "快速说明",
    quickGuideBody: "先选手牌再点出牌。商店先点商品聚焦，再点购买。",
    controls: "控制",
    flowActions: "流程",
    economyActions: "商店与经济",
    inventoryActions: "背包与系统",
    hand: "手牌",
    scoring: "计分",
    shop: "商店",
    inventory: "背包",
    pack: "卡包",
    inspector: "详情",
    events: "事件",
    noScoring: "暂无计分详情。",
    noShop: "商店未开启。",
    noPack: "当前没有打开卡包。",
    noEvents: "暂无事件。",
    noBossEffects: "没有激活的 Boss 效果。",
    noActiveVouchers: "没有激活优惠券。",
    selectCardsToPlay: "请先选择至少一张手牌。",
    selectOfferFirst: "请先选择商店商品。",
    resolvePackFirst: "请先处理当前卡包，再购买其他商品。",
    selectPackOptionFirst: "请先选择至少一个卡包选项。",
    selectConsumableFirst: "请先选择消耗牌。",
    selectJokerFirst: "请先选择小丑。",
    maxHandSelected: "最多选择五张手牌。",
    packPickLimit: "该卡包最多可选 {count} 个。",
    selected: "已选",
    suggestions: "建议操作",
    openInspectorHint: "点击任意行/卡牌可查看详情。",
    runningAction: "执行中",
    quickBuyOn: "快速购买：开",
    quickBuyOff: "快速购买：关",
    shiftClickHint: "商店中 Shift+点击只聚焦不购买。",
    sort: "排序",
    clear: "清空",
    scoreTotal: "总分",
    baseChips: "基础筹码",
    baseMult: "基础倍率",
    rankChips: "牌面筹码",
    showSteps: "显示步骤",
    hideSteps: "隐藏步骤",
    reroll: "刷新",
    cards: "卡牌",
    packs: "卡包",
    vouchers: "优惠券",
    jokers: "小丑",
    consumables: "消耗牌",
    bossEffects: "Boss 效果",
    activeVouchers: "激活优惠券",
    refresh: "刷新",
    start: "开始",
    deal: "发牌",
    play: "出牌",
    discard: "弃牌",
    skipBlind: "跳过盲注",
    nextBlind: "下一盲注",
    enterShop: "进入商店",
    leaveShop: "离开商店",
    buySelected: "购买已选",
    pickPack: "确认卡包",
    skipPack: "跳过卡包",
    useConsumable: "使用消耗牌",
    sellJoker: "出售小丑",
    reset: "重置",
    unknown: "未知",
    handCard: "手牌",
    shopOffer: "商店商品",
    inventoryItem: "背包条目",
    packOption: "卡包选项",
    stateSummary: "对局摘要",
    shortcuts: "快捷键",
    shortcutsBody: "S 开始, D 发牌, P 出牌, X 弃牌, O 商店, B 购买, R 刷新, N 下一盲注",
    logEntries: "条",
  },
};

function copyForLocale(locale) {
  return COPY[locale] || COPY.en_US;
}

function isEditableTarget(target) {
  if (!target) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
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
    default:
      return 0;
  }
}

function formatCard(card) {
  if (card.face_down) {
    return "??";
  }
  return `${rankShort(card.rank)}${suitShort(card.suit)}`;
}

function formatCardTags(card) {
  const tags = [];
  if (card.enhancement) tags.push(card.enhancement);
  if (card.edition) tags.push(card.edition);
  if (card.seal) tags.push(card.seal);
  if (card.bonus_chips) tags.push(`+${card.bonus_chips}`);
  return tags.join(" | ");
}

function packOptionLabel(option, fallback) {
  if (option.kind === "PlayingCard") {
    return `Card ${formatCard(option.value)}`;
  }
  if (option.kind === "Consumable") {
    return `${option.value.kind} ${option.value.name || option.value.id || fallback}`;
  }
  if (option.kind === "Joker") {
    return `Joker ${option.value.name || option.value.id || fallback}`;
  }
  return fallback;
}

function formatEvent(event) {
  if (!event || typeof event !== "object") {
    return String(event);
  }
  if (event.BlindStarted) {
    const e = event.BlindStarted;
    return `blind started A${e.ante} ${e.blind} target ${e.target}`;
  }
  if (event.BlindSkipped) {
    const e = event.BlindSkipped;
    return `blind skipped A${e.ante} ${e.blind} tag ${e.tag || "none"}`;
  }
  if (event.HandDealt) {
    return `hand dealt ${event.HandDealt.count}`;
  }
  if (event.HandScored) {
    const e = event.HandScored;
    return `scored ${e.hand}: ${e.chips} x${e.mult.toFixed(2)} = ${e.total}`;
  }
  if (event.ShopEntered) {
    const e = event.ShopEntered;
    return `shop entered offers ${e.offers} reroll ${e.reroll_cost}`;
  }
  if (event.ShopRerolled) {
    const e = event.ShopRerolled;
    return `shop reroll cost ${e.cost} money ${e.money}`;
  }
  if (event.ShopBought) {
    const e = event.ShopBought;
    return `shop bought ${e.offer} cost ${e.cost}`;
  }
  if (event.PackOpened) {
    const e = event.PackOpened;
    return `pack opened ${e.kind} options ${e.options} picks ${e.picks}`;
  }
  if (event.PackChosen) {
    return `pack chosen picks ${event.PackChosen.picks}`;
  }
  if (event.JokerSold) {
    const e = event.JokerSold;
    return `joker sold ${e.id} +${e.sell_value}`;
  }
  if (event.BlindCleared) {
    const e = event.BlindCleared;
    return `blind cleared score ${e.score} reward ${e.reward}`;
  }
  if (event.BlindFailed) {
    return `blind failed score ${event.BlindFailed.score}`;
  }
  return JSON.stringify(event);
}

function nextHint(snapshot, openPack, t) {
  if (!snapshot) {
    return t.loading;
  }
  if (openPack) {
    return "Pick pack options and confirm, or skip.";
  }
  if (snapshot.phase === "Deal") {
    return "Deal cards to start this hand.";
  }
  if (snapshot.phase === "Play") {
    return "Play selected cards or discard to cycle your hand.";
  }
  if (snapshot.phase === "Shop") {
    return "Buy, reroll, or leave shop when ready.";
  }
  if (snapshot.phase === "Setup") {
    return "Start this blind.";
  }
  return "Keep moving forward through the blind flow.";
}

function actionLabel(action) {
  switch (action) {
    case "start":
      return "Start";
    case "deal":
      return "Deal";
    case "play":
      return "Play";
    case "discard":
      return "Discard";
    case "enter_shop":
      return "Enter Shop";
    case "leave_shop":
      return "Leave Shop";
    case "buy_card":
      return "Buy Card";
    case "buy_pack":
      return "Buy Pack";
    case "buy_voucher":
      return "Buy Voucher";
    case "buy_selected":
      return "Buy Selected";
    case "pick_pack":
      return "Pick Pack";
    case "skip_pack":
      return "Skip Pack";
    case "skip_blind":
      return "Skip Blind";
    case "next_blind":
      return "Next Blind";
    case "reroll":
      return "Reroll";
    case "use_consumable":
      return "Use Consumable";
    case "sell_joker":
      return "Sell Joker";
    case "reset":
      return "Reset";
    case "refresh":
      return "Refresh";
    default:
      return action;
  }
}

function replaceTemplate(template, params) {
  return Object.entries(params).reduce(
    (text, [key, value]) => text.replace(`{${key}}`, String(value)),
    template
  );
}

export default function App() {
  const [snapshot, setSnapshot] = useState(null);
  const [openPack, setOpenPack] = useState(null);
  const [breakdown, setBreakdown] = useState(null);
  const [pendingAction, setPendingAction] = useState(null);
  const [locale, setLocale] = useState("en_US");
  const [error, setError] = useState("");
  const [logs, setLogs] = useState([]);
  const [selectedHand, setSelectedHand] = useState([]);
  const [selectedPack, setSelectedPack] = useState([]);
  const [selectedShop, setSelectedShop] = useState(null);
  const [selectedJoker, setSelectedJoker] = useState(null);
  const [selectedConsumable, setSelectedConsumable] = useState(null);
  const [quickBuy, setQuickBuy] = useState(true);
  const [sortKey, setSortKey] = useState("none");
  const [sortDir, setSortDir] = useState("desc");
  const [showTrace, setShowTrace] = useState(false);
  const [inspector, setInspector] = useState(null);

  const t = useMemo(() => copyForLocale(locale), [locale]);

  const pushLog = useCallback((text, kind = "info") => {
    setLogs((current) => {
      const next = [...current, { id: `${Date.now()}-${Math.random()}`, text, kind }];
      if (next.length > MAX_LOG_ENTRIES) {
        return next.slice(next.length - MAX_LOG_ENTRIES);
      }
      return next;
    });
  }, []);

  const applyApiResponse = useCallback(
    (response, actionName = null) => {
      setLocale(response.locale || "en_US");
      setSnapshot(response.state);
      setOpenPack(response.open_pack || null);
      setBreakdown(response.last_breakdown || null);

      if (Array.isArray(response.events)) {
        response.events.forEach((event) => pushLog(formatEvent(event)));
      }

      if (!response.ok) {
        const message = response.error || "Action failed.";
        setError(message);
        pushLog(message, "error");
        return;
      }

      setError("");
      if (actionName) {
        pushLog(`${actionLabel(actionName)} complete.`);
      }
    },
    [pushLog]
  );

  const refreshState = useCallback(async () => {
    const response = await fetch("/api/state");
    if (!response.ok) {
      throw new Error(`state request failed (${response.status})`);
    }
    const payload = await response.json();
    applyApiResponse(payload, "refresh");
  }, [applyApiResponse]);

  const sendAction = useCallback(
    async (action, { indices = [], target } = {}) => {
      setPendingAction(action);
      try {
        const requestBody = { action, indices };
        if (target !== undefined && target !== null) {
          requestBody.target = String(target);
        }
        const response = await fetch("/api/action", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(requestBody),
        });
        if (!response.ok) {
          throw new Error(`action request failed (${response.status})`);
        }
        const payload = await response.json();
        applyApiResponse(payload, action);
      } catch (cause) {
        const message = cause instanceof Error ? cause.message : String(cause);
        setError(message);
        pushLog(message, "error");
      } finally {
        setPendingAction(null);
      }
    },
    [applyApiResponse, pushLog]
  );

  useEffect(() => {
    refreshState().catch((cause) => {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
      pushLog(message, "error");
    });
  }, [refreshState, pushLog]);

  useEffect(() => {
    if (!snapshot) {
      return;
    }

    setSelectedHand((current) => current.filter((idx) => idx < snapshot.hand.length));
    setSelectedPack((current) => current.filter((idx) => idx < (openPack?.options.length || 0)));
    setSelectedJoker((current) =>
      current == null || current < snapshot.jokers.length ? current : null
    );
    setSelectedConsumable((current) =>
      current == null || current < snapshot.consumables.length ? current : null
    );

    const cardCount = snapshot.shop?.cards?.length || 0;
    const packCount = snapshot.shop?.packs?.length || 0;
    const voucherCount = snapshot.shop?.voucher_offers?.length || 0;
    setSelectedShop((current) => {
      if (!current) return null;
      if (current.kind === "card" && current.index < cardCount) return current;
      if (current.kind === "pack" && current.index < packCount) return current;
      if (current.kind === "voucher" && current.index < voucherCount) return current;
      return null;
    });
  }, [openPack, snapshot]);

  const rankChipMap = useMemo(() => {
    const map = new Map();
    if (!snapshot) return map;
    snapshot.rank_chips.forEach((entry) => map.set(entry.rank, entry.chips));
    return map;
  }, [snapshot]);

  const scoringIndices = useMemo(() => {
    const set = new Set();
    if (!breakdown) return set;
    breakdown.scoring_indices.forEach((index) => set.add(index));
    return set;
  }, [breakdown]);

  const sortedHand = useMemo(() => {
    if (!snapshot) return [];
    const items = snapshot.hand.map((card, index) => ({ card, index }));
    if (sortKey !== "none") {
      items.sort((left, right) => {
        if (sortKey === "rank") {
          return rankValue(left.card.rank) - rankValue(right.card.rank);
        }
        const leftValue = (rankChipMap.get(left.card.rank) || 0) + (left.card.bonus_chips || 0);
        const rightValue = (rankChipMap.get(right.card.rank) || 0) + (right.card.bonus_chips || 0);
        return leftValue - rightValue;
      });
    }
    if (sortDir === "desc") {
      items.reverse();
    }
    return items;
  }, [rankChipMap, snapshot, sortDir, sortKey]);

  const requireHandSelection = useCallback(
    (message, callback) => {
      if (selectedHand.length === 0) {
        pushLog(message, "error");
        return;
      }
      callback();
    },
    [pushLog, selectedHand.length]
  );

  const playSelected = useCallback(() => {
    requireHandSelection(t.selectCardsToPlay, () => {
      sendAction("play", { indices: selectedHand });
      setSelectedHand([]);
    });
  }, [requireHandSelection, selectedHand, sendAction, t.selectCardsToPlay]);

  const discardSelected = useCallback(() => {
    requireHandSelection(t.selectCardsToPlay, () => {
      sendAction("discard", { indices: selectedHand });
      setSelectedHand([]);
    });
  }, [requireHandSelection, selectedHand, sendAction, t.selectCardsToPlay]);

  const buySelected = useCallback(() => {
    if (!selectedShop) {
      pushLog(t.selectOfferFirst, "error");
      return;
    }
    if (openPack) {
      pushLog(t.resolvePackFirst, "error");
      return;
    }
    if (selectedShop.kind === "card") {
      sendAction("buy_card", { target: selectedShop.index });
      return;
    }
    if (selectedShop.kind === "pack") {
      sendAction("buy_pack", { target: selectedShop.index });
      return;
    }
    sendAction("buy_voucher", { target: selectedShop.index });
  }, [openPack, pushLog, selectedShop, sendAction, t.resolvePackFirst, t.selectOfferFirst]);

  const pickPack = useCallback(() => {
    if (!openPack) {
      pushLog(t.noPack, "error");
      return;
    }
    if (selectedPack.length === 0) {
      pushLog(t.selectPackOptionFirst, "error");
      return;
    }
    sendAction("pick_pack", { indices: selectedPack });
    setSelectedPack([]);
  }, [openPack, pushLog, selectedPack, sendAction, t.noPack, t.selectPackOptionFirst]);

  const useConsumable = useCallback(() => {
    if (selectedConsumable == null) {
      pushLog(t.selectConsumableFirst, "error");
      return;
    }
    sendAction("use_consumable", { target: selectedConsumable, indices: selectedHand });
  }, [pushLog, selectedConsumable, selectedHand, sendAction, t.selectConsumableFirst]);

  const sellJoker = useCallback(() => {
    if (selectedJoker == null) {
      pushLog(t.selectJokerFirst, "error");
      return;
    }
    sendAction("sell_joker", { target: selectedJoker });
  }, [pushLog, selectedJoker, sendAction, t.selectJokerFirst]);

  useEffect(() => {
    const handleKeyDown = (event) => {
      if (event.repeat || isEditableTarget(event.target)) {
        return;
      }
      if (event.shiftKey && event.key.toLowerCase() === "r") {
        event.preventDefault();
        sendAction("reset");
        return;
      }
      switch (event.key.toLowerCase()) {
        case "s":
          event.preventDefault();
          sendAction("start");
          break;
        case "d":
          event.preventDefault();
          sendAction("deal");
          break;
        case "p":
          event.preventDefault();
          playSelected();
          break;
        case "x":
          event.preventDefault();
          discardSelected();
          break;
        case "o":
          event.preventDefault();
          sendAction("enter_shop");
          break;
        case "r":
          event.preventDefault();
          sendAction("reroll");
          break;
        case "b":
          event.preventDefault();
          buySelected();
          break;
        case "l":
          event.preventDefault();
          sendAction("leave_shop");
          break;
        case "k":
          event.preventDefault();
          pickPack();
          break;
        case "y":
          event.preventDefault();
          sendAction("skip_pack");
          break;
        case "u":
          event.preventDefault();
          useConsumable();
          break;
        case "j":
          event.preventDefault();
          sellJoker();
          break;
        case "g":
          event.preventDefault();
          sendAction("skip_blind");
          break;
        case "n":
          event.preventDefault();
          sendAction("next_blind");
          break;
        default:
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [buySelected, discardSelected, pickPack, playSelected, sellJoker, sendAction, useConsumable]);

  const inspectorLines = useMemo(() => {
    if (!snapshot) {
      return [t.loading];
    }
    if (inspector?.kind === "hand") {
      const card = snapshot.hand[inspector.index];
      if (!card) return [t.unknown];
      return [
        `${t.handCard} #${inspector.index}: ${formatCard(card)}`,
        `Rank: ${card.rank} | Suit: ${card.suit}`,
        card.enhancement ? `Enhancement: ${card.enhancement}` : null,
        card.edition ? `Edition: ${card.edition}` : null,
        card.seal ? `Seal: ${card.seal}` : null,
        card.bonus_chips ? `Bonus chips: ${card.bonus_chips}` : null,
      ].filter(Boolean);
    }
    if (inspector?.kind === "shop") {
      return [`${t.shopOffer}:`, inspector.label];
    }
    if (inspector?.kind === "inventory") {
      return [`${t.inventoryItem}:`, inspector.label];
    }
    if (inspector?.kind === "pack") {
      return [`${t.packOption}:`, inspector.label];
    }

    return [
      `${t.stateSummary}`,
      `Seed: ${snapshot.seed}`,
      `Signature: ${snapshot.content_signature || "-"}`,
      `Phase: ${snapshot.phase}`,
      `${t.openInspectorHint}`,
    ];
  }, [inspector, snapshot, t]);

  const suggestionActions = useMemo(() => {
    if (!snapshot) {
      return [];
    }

    if (openPack) {
      return [
        { id: "pick_pack", label: t.pickPack, onClick: pickPack },
        { id: "skip_pack", label: t.skipPack, onClick: () => sendAction("skip_pack") },
      ];
    }

    if (snapshot.phase === "Deal") {
      return [{ id: "deal", label: t.deal, onClick: () => sendAction("deal") }];
    }

    if (snapshot.phase === "Play") {
      return [
        { id: "play", label: t.play, onClick: playSelected },
        { id: "discard", label: t.discard, onClick: discardSelected },
      ];
    }

    if (snapshot.phase === "Shop") {
      return [
        { id: "buy_selected", label: t.buySelected, onClick: buySelected },
        { id: "reroll", label: t.reroll, onClick: () => sendAction("reroll") },
        { id: "leave_shop", label: t.leaveShop, onClick: () => sendAction("leave_shop") },
      ];
    }

    return [
      { id: "start", label: t.start, onClick: () => sendAction("start") },
      { id: "next_blind", label: t.nextBlind, onClick: () => sendAction("next_blind") },
    ];
  }, [buySelected, discardSelected, openPack, pickPack, playSelected, sendAction, snapshot, t]);

  if (!snapshot) {
    return (
      <div className="friendly-ui loading">
        <h1>Rulatro</h1>
        <p>{t.loading}</p>
        {error ? <p className="error-line">{error}</p> : null}
      </div>
    );
  }

  const shop = snapshot.shop;

  return (
    <div className="friendly-ui">
      <header className="hero">
        <div>
          <h1>Rulatro</h1>
          <p className="hero-subtitle">{t.subtitle}</p>
        </div>
        <div className="hero-status-grid">
          <div className="chip">Ante {snapshot.ante}</div>
          <div className="chip">Blind {snapshot.blind}</div>
          <div className="chip">Phase {snapshot.phase}</div>
          <div className="chip">Money ${snapshot.money}</div>
          <div className="chip">Score {snapshot.score}/{snapshot.target}</div>
          <div className="chip">Hands {snapshot.hands_left}/{snapshot.hands_max}</div>
          <div className="chip">Discards {snapshot.discards_left}/{snapshot.discards_max}</div>
          <div className="chip">Deck {snapshot.deck_draw}/{snapshot.deck_discard}</div>
          <div className="chip">Locale {locale}</div>
        </div>
      </header>

      <main className="layout-grid">
        <section className="panel span-12 guidance-panel">
          <div className="guidance-top-row">
            <div>
              <h2>{t.nextStep}</h2>
              <p className="hint">{nextHint(snapshot, openPack, t)}</p>
            </div>
            <div className="suggestion-buttons">
              {suggestionActions.map((item) => (
                <button key={item.id} onClick={item.onClick} disabled={Boolean(pendingAction)}>
                  {item.label}
                </button>
              ))}
            </div>
          </div>
          <div className="guidance-grid">
            <div className="guidance-card">
              <h3>{t.quickGuide}</h3>
              <p>{t.quickGuideBody}</p>
            </div>
            <div className="guidance-card">
              <h3>{t.shortcuts}</h3>
              <p>{t.shortcutsBody}</p>
            </div>
            <div className="guidance-card">
              <h3>{t.suggestions}</h3>
              <p>{t.shiftClickHint}</p>
            </div>
          </div>
          {pendingAction ? (
            <div className="pending-line">{t.runningAction}: {actionLabel(pendingAction)}...</div>
          ) : null}
          {error ? <div className="error-line">{error}</div> : null}
        </section>

        <section className="panel span-12">
          <div className="panel-head">
            <h2>{t.controls}</h2>
            <div className="control-inline-actions">
              <button onClick={refreshState} disabled={Boolean(pendingAction)}>
                {t.refresh}
              </button>
              <button
                className={quickBuy ? "is-active" : ""}
                onClick={() => setQuickBuy((value) => !value)}
              >
                {quickBuy ? t.quickBuyOn : t.quickBuyOff}
              </button>
            </div>
          </div>

          <div className="control-groups">
            <div className="control-group">
              <h3>{t.flowActions}</h3>
              <div className="control-row">
                <button onClick={() => sendAction("start")} disabled={Boolean(pendingAction)}>{t.start}</button>
                <button onClick={() => sendAction("deal")} disabled={Boolean(pendingAction)}>{t.deal}</button>
                <button onClick={playSelected} disabled={Boolean(pendingAction)}>{t.play}</button>
                <button onClick={discardSelected} disabled={Boolean(pendingAction)}>{t.discard}</button>
                <button onClick={() => sendAction("skip_blind")} disabled={Boolean(pendingAction)}>{t.skipBlind}</button>
                <button onClick={() => sendAction("next_blind")} disabled={Boolean(pendingAction)}>{t.nextBlind}</button>
              </div>
            </div>
            <div className="control-group">
              <h3>{t.economyActions}</h3>
              <div className="control-row">
                <button onClick={() => sendAction("enter_shop")} disabled={Boolean(pendingAction)}>{t.enterShop}</button>
                <button onClick={() => sendAction("reroll")} disabled={Boolean(pendingAction)}>{t.reroll}</button>
                <button onClick={buySelected} disabled={Boolean(pendingAction)}>{t.buySelected}</button>
                <button onClick={() => sendAction("leave_shop")} disabled={Boolean(pendingAction)}>{t.leaveShop}</button>
                <button onClick={pickPack} disabled={Boolean(pendingAction)}>{t.pickPack}</button>
                <button onClick={() => sendAction("skip_pack")} disabled={Boolean(pendingAction)}>{t.skipPack}</button>
              </div>
            </div>
            <div className="control-group">
              <h3>{t.inventoryActions}</h3>
              <div className="control-row">
                <button onClick={useConsumable} disabled={Boolean(pendingAction)}>{t.useConsumable}</button>
                <button onClick={sellJoker} disabled={Boolean(pendingAction)}>{t.sellJoker}</button>
                <button onClick={() => sendAction("reset")} disabled={Boolean(pendingAction)}>{t.reset}</button>
              </div>
            </div>
          </div>
        </section>

        <section className="panel span-7">
          <div className="panel-head">
            <h2>{t.hand}</h2>
            <div className="control-inline-actions">
              <label>
                {t.sort}
                <select value={sortKey} onChange={(event) => setSortKey(event.target.value)}>
                  <option value="none">None</option>
                  <option value="rank">Rank</option>
                  <option value="value">Value</option>
                </select>
              </label>
              <button onClick={() => setSortDir((value) => (value === "desc" ? "asc" : "desc"))}>
                {sortDir.toUpperCase()}
              </button>
              <button onClick={() => setSelectedHand([])}>{t.clear}</button>
            </div>
          </div>
          <div className="panel-subline">{t.selected}: {selectedHand.length}/{MAX_HAND_SELECTION}</div>
          <div className="card-grid">
            {sortedHand.map(({ card, index }) => {
              const selected = selectedHand.includes(index);
              const scoring = scoringIndices.has(index);
              const className = [
                "card-item",
                selected ? "selected" : "",
                scoring ? "scoring" : "",
              ]
                .filter(Boolean)
                .join(" ");
              return (
                <button
                  key={`${card.id}-${index}`}
                  className={className}
                  onClick={() => {
                    setInspector({ kind: "hand", index });
                    setSelectedHand((current) => {
                      if (current.includes(index)) {
                        return current.filter((item) => item !== index);
                      }
                      if (current.length >= MAX_HAND_SELECTION) {
                        pushLog(t.maxHandSelected, "error");
                        return current;
                      }
                      return [...current, index];
                    });
                  }}
                >
                  <div className="card-title">{formatCard(card)}</div>
                  <div className="card-meta">#{index}</div>
                  <div className="card-meta">{formatCardTags(card) || "-"}</div>
                </button>
              );
            })}
          </div>
        </section>

        <section className="panel span-5">
          <div className="panel-head">
            <h2>{t.scoring}</h2>
            {breakdown?.steps?.length ? (
              <button onClick={() => setShowTrace((value) => !value)}>
                {showTrace ? t.hideSteps : t.showSteps}
              </button>
            ) : null}
          </div>
          {!breakdown ? (
            <div className="empty-state">{t.noScoring}</div>
          ) : (
            <div className="score-grid">
              <div className="score-row">Hand: {breakdown.hand}</div>
              <div className="score-row">{t.baseChips}: {breakdown.base_chips}</div>
              <div className="score-row">{t.baseMult}: {breakdown.base_mult.toFixed(2)}</div>
              <div className="score-row">{t.rankChips}: {breakdown.rank_chips}</div>
              <div className="score-row">{t.scoreTotal}: {breakdown.total_chips} x {breakdown.total_mult.toFixed(2)} = {breakdown.total_score}</div>
              {showTrace ? (
                <div className="score-trace-list">
                  {breakdown.steps.map((step, idx) => (
                    <div className="trace-row" key={`${step.source}-${idx}`}>
                      <strong>#{idx + 1}</strong> {step.source} | {step.effect} | {step.before_chips}x{step.before_mult.toFixed(2)} -> {step.after_chips}x{step.after_mult.toFixed(2)}
                    </div>
                  ))}
                </div>
              ) : null}
            </div>
          )}
        </section>

        <section className="panel span-8">
          <div className="panel-head">
            <h2>{t.shop}</h2>
            <div className="panel-subline">{t.reroll}: ${shop?.reroll_cost ?? "-"}</div>
          </div>
          {!shop ? (
            <div className="empty-state">{t.noShop}</div>
          ) : (
            <div className="three-col-grid">
              <div>
                <h3>{t.cards}</h3>
                <div className="list-grid">
                  {shop.cards.map((item, index) => {
                    const selected = selectedShop?.kind === "card" && selectedShop.index === index;
                    return (
                      <button
                        key={`shop-card-${index}`}
                        className={`list-row ${selected ? "selected" : ""}`}
                        onClick={(event) => {
                          const label = `${item.name} | $${item.price} | ${item.rarity || "-"}`;
                          setInspector({ kind: "shop", label });
                          if (openPack) {
                            pushLog(t.resolvePackFirst, "error");
                            return;
                          }
                          if (quickBuy && !event.shiftKey) {
                            sendAction("buy_card", { target: index });
                            return;
                          }
                          setSelectedShop({ kind: "card", index });
                        }}
                      >
                        <div>{item.name}</div>
                        <div className="row-meta">${item.price} | {item.rarity || "-"} | {item.edition || "-"}</div>
                      </button>
                    );
                  })}
                </div>
              </div>
              <div>
                <h3>{t.packs}</h3>
                <div className="list-grid">
                  {shop.packs.map((item, index) => {
                    const selected = selectedShop?.kind === "pack" && selectedShop.index === index;
                    return (
                      <button
                        key={`shop-pack-${index}`}
                        className={`list-row ${selected ? "selected" : ""}`}
                        onClick={(event) => {
                          const label = `${item.kind}/${item.size} options ${item.options} picks ${item.picks} $${item.price}`;
                          setInspector({ kind: "shop", label });
                          if (openPack) {
                            pushLog(t.resolvePackFirst, "error");
                            return;
                          }
                          if (quickBuy && !event.shiftKey) {
                            sendAction("buy_pack", { target: index });
                            return;
                          }
                          setSelectedShop({ kind: "pack", index });
                        }}
                      >
                        <div>{item.kind}/{item.size}</div>
                        <div className="row-meta">options {item.options} | picks {item.picks} | ${item.price}</div>
                      </button>
                    );
                  })}
                </div>
              </div>
              <div>
                <h3>{t.vouchers}</h3>
                <div className="list-grid">
                  {shop.voucher_offers.map((item, index) => {
                    const selected = selectedShop?.kind === "voucher" && selectedShop.index === index;
                    return (
                      <button
                        key={`shop-voucher-${index}`}
                        className={`list-row ${selected ? "selected" : ""}`}
                        onClick={(event) => {
                          const label = `${item.name} | $${shop.voucher_price} | ${item.effect || "-"}`;
                          setInspector({ kind: "shop", label });
                          if (openPack) {
                            pushLog(t.resolvePackFirst, "error");
                            return;
                          }
                          if (quickBuy && !event.shiftKey) {
                            sendAction("buy_voucher", { target: index });
                            return;
                          }
                          setSelectedShop({ kind: "voucher", index });
                        }}
                      >
                        <div>{item.name}</div>
                        <div className="row-meta">${shop.voucher_price} | {item.effect || "-"}</div>
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>
          )}
        </section>

        <section className="panel span-4">
          <div className="panel-head">
            <h2>{t.inventory}</h2>
            <div className="panel-subline">J {snapshot.jokers.length} | C {snapshot.consumables.length}</div>
          </div>
          <div className="two-col-grid">
            <div>
              <h3>{t.jokers}</h3>
              <div className="list-grid">
                {snapshot.jokers.map((item, index) => {
                  const selected = selectedJoker === index;
                  const label = `${item.name} (${item.id}) ${item.rarity} ${item.edition || "-"}`;
                  return (
                    <button
                      key={`inv-joker-${index}`}
                      className={`list-row ${selected ? "selected" : ""}`}
                      onClick={() => {
                        setSelectedJoker(index);
                        setInspector({ kind: "inventory", label });
                      }}
                    >
                      <div>{item.name}</div>
                      <div className="row-meta">{item.rarity} | {item.edition || "-"}</div>
                    </button>
                  );
                })}
              </div>
            </div>
            <div>
              <h3>{t.consumables}</h3>
              <div className="list-grid">
                {snapshot.consumables.map((item, index) => {
                  const selected = selectedConsumable === index;
                  const label = `${item.name} (${item.kind}) ${item.edition || "-"}`;
                  return (
                    <button
                      key={`inv-consumable-${index}`}
                      className={`list-row ${selected ? "selected" : ""}`}
                      onClick={() => {
                        setSelectedConsumable(index);
                        setInspector({ kind: "inventory", label });
                      }}
                    >
                      <div>{item.name}</div>
                      <div className="row-meta">{item.kind} | {item.edition || "-"}</div>
                    </button>
                  );
                })}
              </div>
            </div>
          </div>
        </section>

        <section className="panel span-4">
          <div className="panel-head">
            <h2>{t.pack}</h2>
            <div className="panel-subline">{openPack ? `${t.selected}: ${selectedPack.length}/${openPack.offer.picks}` : t.noPack}</div>
          </div>
          {!openPack ? (
            <div className="empty-state">{t.noPack}</div>
          ) : (
            <div className="list-grid">
              {openPack.options.map((option, index) => {
                const selected = selectedPack.includes(index);
                return (
                  <button
                    key={`pack-option-${index}`}
                    className={`list-row ${selected ? "selected" : ""}`}
                    onClick={() => {
                      const label = packOptionLabel(option, t.unknown);
                      setInspector({ kind: "pack", label });
                      setSelectedPack((current) => {
                        if (current.includes(index)) {
                          return current.filter((item) => item !== index);
                        }
                        if (current.length >= openPack.offer.picks) {
                          pushLog(replaceTemplate(t.packPickLimit, { count: openPack.offer.picks }), "error");
                          return current;
                        }
                        return [...current, index];
                      });
                    }}
                  >
                    {packOptionLabel(option, t.unknown)}
                  </button>
                );
              })}
            </div>
          )}
        </section>

        <section className="panel span-4">
          <div className="panel-head">
            <h2>{t.inspector}</h2>
          </div>
          <div className="inspector-box">
            {inspectorLines.map((line, idx) => (
              <div key={`inspector-line-${idx}`}>{line}</div>
            ))}
          </div>

          <div className="sub-box">
            <h3>{t.bossEffects}</h3>
            {snapshot.boss_effects.length === 0 ? (
              <div className="empty-state">{t.noBossEffects}</div>
            ) : (
              snapshot.boss_effects.map((line, idx) => <div key={`boss-effect-${idx}`}>{line}</div>)
            )}
          </div>

          <div className="sub-box">
            <h3>{t.activeVouchers}</h3>
            {snapshot.active_vouchers.length === 0 ? (
              <div className="empty-state">{t.noActiveVouchers}</div>
            ) : (
              snapshot.active_vouchers.map((line, idx) => <div key={`active-voucher-${idx}`}>{line}</div>)
            )}
          </div>
        </section>

        <section className="panel span-8">
          <div className="panel-head">
            <h2>{t.events}</h2>
            <div className="panel-subline">{logs.length} {t.logEntries}</div>
          </div>
          <div className="event-log">
            {logs.length === 0 ? <div className="empty-state">{t.noEvents}</div> : null}
            {logs.map((entry) => (
              <div key={entry.id} className={`event-row ${entry.kind === "error" ? "error" : ""}`}>
                {entry.text}
              </div>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}
