use super::*;
use crate::*;
use super::helpers::*;

impl RunState {
    pub(super) fn eval_bool(&mut self, expr: &Expr, ctx: &EvalContext<'_>) -> bool {
        self.eval_expr(expr, ctx).truthy()
    }

    pub(super) fn eval_expr(&mut self, expr: &Expr, ctx: &EvalContext<'_>) -> EvalValue {
        match expr {
            Expr::Bool(value) => EvalValue::Bool(*value),
            Expr::Number(value) => EvalValue::Num(*value),
            Expr::String(value) => EvalValue::Str(normalize(value)),
            Expr::Ident(value) => self.eval_ident(value, ctx),
            Expr::Call { name, args } => self.eval_call(name, args, ctx),
            Expr::Unary { op, expr } => {
                let inner = self.eval_expr(expr, ctx);
                match op {
                    UnaryOp::Not => EvalValue::Bool(!inner.truthy()),
                    UnaryOp::Neg => inner.as_number().map(|v| EvalValue::Num(-v)).unwrap_or(EvalValue::None),
                }
            }
            Expr::Binary { left, op, right } => {
                let left_val = self.eval_expr(left, ctx);
                let right_val = self.eval_expr(right, ctx);
                match op {
                    BinaryOp::Or => EvalValue::Bool(left_val.truthy() || right_val.truthy()),
                    BinaryOp::And => EvalValue::Bool(left_val.truthy() && right_val.truthy()),
                    BinaryOp::Eq => EvalValue::Bool(values_equal(
                        &left_val,
                        &right_val,
                        self.smeared_suits_active(),
                    )),
                    BinaryOp::Ne => EvalValue::Bool(!values_equal(
                        &left_val,
                        &right_val,
                        self.smeared_suits_active(),
                    )),
                    BinaryOp::Lt => EvalValue::Bool(compare_numbers(&left_val, &right_val, |a, b| a < b)),
                    BinaryOp::Le => EvalValue::Bool(compare_numbers(&left_val, &right_val, |a, b| a <= b)),
                    BinaryOp::Gt => EvalValue::Bool(compare_numbers(&left_val, &right_val, |a, b| a > b)),
                    BinaryOp::Ge => EvalValue::Bool(compare_numbers(&left_val, &right_val, |a, b| a >= b)),
                    BinaryOp::Add => combine_numbers(&left_val, &right_val, |a, b| a + b),
                    BinaryOp::Sub => combine_numbers(&left_val, &right_val, |a, b| a - b),
                    BinaryOp::Mul => combine_numbers(&left_val, &right_val, |a, b| a * b),
                    BinaryOp::Div => combine_numbers(&left_val, &right_val, |a, b| if b == 0.0 { a } else { a / b }),
                }
            }
        }
    }

    pub(super) fn eval_ident(&mut self, ident: &str, ctx: &EvalContext<'_>) -> EvalValue {
        let card_debuffed = ctx
            .card
            .map(|card| self.is_card_debuffed(card))
            .unwrap_or(false);
        match ident {
            "hand" => EvalValue::Str(normalize(hand_name(ctx.hand_kind))),
            "hand_id" => EvalValue::Num(hand_id(ctx.hand_kind) as f64),
            "blind" => EvalValue::Str(normalize(blind_name(ctx.blind))),
            "played_count" => EvalValue::Num(ctx.played_count as f64),
            "scoring_count" => EvalValue::Num(ctx.scoring_count as f64),
            "held_count" => EvalValue::Num(ctx.held_cards.len() as f64),
            "deck_count" => EvalValue::Num(self.deck_count(ctx) as f64),
            "joker_count" => EvalValue::Num(ctx.joker_count as f64),
            "hands_left" => EvalValue::Num(ctx.hands_left as f64),
            "discards_left" => EvalValue::Num(ctx.discards_left as f64),
            "hands_max" => EvalValue::Num(self.state.hands_max as f64),
            "discards_max" => EvalValue::Num(self.state.discards_max as f64),
            "money" => EvalValue::Num(self.state.money as f64),
            "blinds_skipped" => EvalValue::Num(self.state.blinds_skipped as f64),
            "hands_played" => {
                let total: u32 = self.state.hand_play_counts.values().copied().sum();
                EvalValue::Num(total as f64)
            }
            "unused_discards" => EvalValue::Num(self.state.unused_discards as f64),
            "unique_planets_used" => EvalValue::Num(self.state.planets_used.len() as f64),
            "hand_size" => EvalValue::Num(self.state.hand_size as f64),
            "ante" => EvalValue::Num(self.state.ante as f64),
            "blind_score" => EvalValue::Num(self.state.blind_score as f64),
            "target" => EvalValue::Num(self.state.target as f64),
            "joker_slots" => EvalValue::Num(self.inventory.joker_capacity() as f64),
            "consumable_slots" => EvalValue::Num(self.inventory.consumable_slots as f64),
            "consumable_count" => EvalValue::Num(self.inventory.consumable_count() as f64),
            "empty_joker_slots" => {
                let empty = self
                    .inventory
                    .joker_capacity()
                    .saturating_sub(ctx.joker_count);
                EvalValue::Num(empty as f64)
            }
            "hand_play_count" => {
                let count = self
                    .state
                    .hand_play_counts
                    .get(&ctx.hand_kind)
                    .copied()
                    .unwrap_or(0);
                EvalValue::Num(count as f64)
            }
            "hand_level" => EvalValue::Num(self.hand_level(ctx.hand_kind) as f64),
            "most_played_hand" => {
                EvalValue::Str(normalize(hand_name(self.most_played_hand())))
            }
            "is_boss_blind" => EvalValue::Bool(self.state.blind == BlindKind::Boss),
            "boss_disabled" => EvalValue::Bool(self.boss_disabled()),
            "is_scoring" => EvalValue::Bool(ctx.is_scoring),
            "is_held" => EvalValue::Bool(ctx.is_held),
            "is_played" => EvalValue::Bool(ctx.is_played),
            "sold_value" => EvalValue::Num(ctx.sold_value.unwrap_or(0) as f64),
            "last_destroyed_sell_value" => EvalValue::Num(self.last_destroyed_sell_value as f64),
            "other_joker_sell_value" => {
                EvalValue::Num(self.other_joker_sell_value(ctx.joker_index) as f64)
            }
            "card.rank" => ctx
                .card
                .map(|card| EvalValue::Str(normalize(rank_name(card.rank))))
                .unwrap_or(EvalValue::None),
            "card.rank_id" => ctx
                .card
                .map(|card| EvalValue::Num(rank_id(card.rank) as f64))
                .unwrap_or(EvalValue::None),
            "card.suit" => ctx
                .card
                .map(|card| EvalValue::Str(normalize(suit_name(card.suit))))
                .unwrap_or(EvalValue::None),
            "card.suit_id" => ctx
                .card
                .map(|card| EvalValue::Num(suit_index(card.suit) as f64))
                .unwrap_or(EvalValue::None),
            "card.enhancement" => ctx
                .card
                .and_then(|card| {
                    if card_debuffed {
                        None
                    } else {
                        card.enhancement.map(enhancement_name)
                    }
                })
                .map(|value| EvalValue::Str(normalize(value)))
                .unwrap_or(EvalValue::None),
            "card.has_enhancement" => EvalValue::Bool(
                !card_debuffed
                    && ctx
                        .card
                        .map(|card| card.enhancement.is_some())
                        .unwrap_or(false),
            ),
            "card.edition" => ctx
                .card
                .and_then(|card| {
                    if card_debuffed {
                        None
                    } else {
                        card.edition.map(edition_name)
                    }
                })
                .map(|value| EvalValue::Str(normalize(value)))
                .unwrap_or(EvalValue::None),
            "card.seal" => ctx
                .card
                .and_then(|card| {
                    if card_debuffed {
                        None
                    } else {
                        card.seal.map(seal_name)
                    }
                })
                .map(|value| EvalValue::Str(normalize(value)))
                .unwrap_or(EvalValue::None),
            "card.lucky_triggers" => EvalValue::Num(ctx.card_lucky_triggers as f64),
            "card.is_face" => {
                if card_debuffed {
                    EvalValue::Bool(false)
                } else if self.pareidolia_active() {
                    EvalValue::Bool(ctx.card.map(|card| !card.is_stone()).unwrap_or(false))
                } else {
                    EvalValue::Bool(ctx.card.map(is_face).unwrap_or(false))
                }
            }
            "card.is_odd" => EvalValue::Bool(
                !card_debuffed && ctx.card.map(is_odd).unwrap_or(false),
            ),
            "card.is_even" => EvalValue::Bool(
                !card_debuffed && ctx.card.map(is_even).unwrap_or(false),
            ),
            "card.is_stone" => EvalValue::Bool(
                !card_debuffed && ctx.card.map(|card| card.is_stone()).unwrap_or(false),
            ),
            "card.is_wild" => EvalValue::Bool(
                !card_debuffed && ctx.card.map(|card| card.is_wild()).unwrap_or(false),
            ),
            "consumable.kind" => ctx
                .consumable_kind
                .map(consumable_kind_name)
                .map(|value| EvalValue::Str(normalize(value)))
                .unwrap_or(EvalValue::None),
            "consumable.id" => ctx
                .consumable_id
                .map(|value| EvalValue::Str(normalize(value)))
                .unwrap_or(EvalValue::None),
            _ => EvalValue::Str(normalize(ident)),
        }
    }

    pub(super) fn eval_call(&mut self, name: &str, args: &[Expr], ctx: &EvalContext<'_>) -> EvalValue {
        match name.to_lowercase().as_str() {
            "contains" => {
                if args.len() != 2 {
                    return EvalValue::Bool(false);
                }
                let target = self.eval_expr(&args[1], ctx);
                let target_kind = target.as_string().and_then(hand_kind_from_str);
                let left = match &args[0] {
                    Expr::Ident(ident) if ident == "hand" => Some(ctx.hand_kind),
                    other => self
                        .eval_expr(other, ctx)
                        .as_string()
                        .and_then(hand_kind_from_str),
                };
                if let (Some(hand), Some(target)) = (left, target_kind) {
                    EvalValue::Bool(hand_contains_kind(hand, target))
                } else {
                    EvalValue::Bool(false)
                }
            }
            "roll" => {
                if args.len() != 1 {
                    return EvalValue::Bool(false);
                }
                let sides = self.eval_expr(&args[0], ctx).as_number().unwrap_or(0.0);
                if sides <= 0.0 {
                    return EvalValue::Bool(false);
                }
                EvalValue::Bool(self.roll(sides.floor() as u64))
            }
            "rand" => {
                if args.len() != 2 {
                    return EvalValue::None;
                }
                let left = self.eval_expr(&args[0], ctx).as_number();
                let right = self.eval_expr(&args[1], ctx).as_number();
                let (Some(mut low), Some(mut high)) = (left, right) else {
                    return EvalValue::None;
                };
                low = low.floor();
                high = high.floor();
                if high < low {
                    std::mem::swap(&mut low, &mut high);
                }
                let span = (high - low) as u64;
                let value = if span == 0 {
                    low
                } else {
                    let roll = self.rng.next_u64() % (span + 1);
                    low + roll as f64
                };
                EvalValue::Num(value)
            }
            "count" => {
                if args.len() != 2 {
                    return EvalValue::Num(0.0);
                }
                let smeared = self.smeared_suits_active();
                let scope = self.eval_expr(&args[0], ctx);
                let target = self.eval_expr(&args[1], ctx);
                let scope_str = scope.as_string().unwrap_or("");
                let target_str = target.as_string().unwrap_or("");
                match normalize(scope_str).as_str() {
                    "deck" | "full_deck" => {
                        EvalValue::Num(self.count_matching_deck(ctx, target_str) as f64)
                    }
                    _ => {
                        let cards = scope_cards(ctx, scope_str);
                        EvalValue::Num(
                            self.count_matching_with_debuff(cards, target_str, smeared) as f64,
                        )
                    }
                }
            }
            "count_joker" => {
                if args.len() != 1 {
                    return EvalValue::Num(0.0);
                }
                let query = self.eval_expr(&args[0], ctx);
                let query_str = query.as_string().unwrap_or("");
                EvalValue::Num(self.count_joker(query_str) as f64)
            }
            "count_rarity" | "count_joker_rarity" => {
                if args.len() != 1 {
                    return EvalValue::Num(0.0);
                }
                let query = self.eval_expr(&args[0], ctx);
                let query_str = normalize(query.as_string().unwrap_or(""));
                let rarity = match query_str.as_str() {
                    "common" => Some(crate::JokerRarity::Common),
                    "uncommon" => Some(crate::JokerRarity::Uncommon),
                    "rare" => Some(crate::JokerRarity::Rare),
                    "legendary" => Some(crate::JokerRarity::Legendary),
                    _ => None,
                };
                if let Some(rarity) = rarity {
                    let count = self
                        .inventory
                        .jokers
                        .iter()
                        .filter(|joker| joker.rarity == rarity)
                        .count();
                    EvalValue::Num(count as f64)
                } else {
                    EvalValue::Num(0.0)
                }
            }
            "suit_match" => {
                if args.len() != 1 {
                    return EvalValue::Bool(false);
                }
                let card = match ctx.card {
                    Some(card) => card,
                    None => return EvalValue::Bool(false),
                };
                if card.is_wild() {
                    return EvalValue::Bool(true);
                }
                let arg = self.eval_expr(&args[0], ctx);
                let smeared = self.smeared_suits_active();
                match arg {
                    EvalValue::Str(value) => {
                        let Some(suit) = suit_from_str(&value) else {
                            return EvalValue::Bool(false);
                        };
                        if smeared {
                            EvalValue::Bool(smeared_suit_group(card.suit) == smeared_suit_group(suit))
                        } else {
                            EvalValue::Bool(card.suit == suit)
                        }
                    }
                    EvalValue::Num(value) => {
                        let idx = value.floor() as i64;
                        let suit = match idx {
                            0 => crate::Suit::Spades,
                            1 => crate::Suit::Hearts,
                            2 => crate::Suit::Clubs,
                            3 => crate::Suit::Diamonds,
                            _ => return EvalValue::Bool(false),
                        };
                        if smeared {
                            EvalValue::Bool(smeared_suit_group(card.suit) == smeared_suit_group(suit))
                        } else {
                            EvalValue::Bool(card.suit == suit)
                        }
                    }
                    _ => EvalValue::Bool(false),
                }
            }
            "hand_count" => {
                if args.len() != 1 {
                    return EvalValue::Num(0.0);
                }
                let query = self.eval_expr(&args[0], ctx);
                let query_str = query.as_string().unwrap_or("");
                let hand = hand_kind_from_str(query_str);
                if let Some(hand) = hand {
                    let count = self.state.hand_play_counts.get(&hand).copied().unwrap_or(0);
                    EvalValue::Num(count as f64)
                } else {
                    EvalValue::Num(0.0)
                }
            }
            "var" => {
                if args.len() != 1 {
                    return EvalValue::Num(0.0);
                }
                let key = self.eval_expr(&args[0], ctx);
                let key_str = key.as_string().unwrap_or("");
                if let Some(vars) = ctx.joker_vars.as_ref() {
                    let value = vars.get(&normalize(key_str)).copied().unwrap_or(0.0);
                    EvalValue::Num(value)
                } else {
                    EvalValue::Num(0.0)
                }
            }
            "lowest_rank" | "min_rank" => {
                if args.len() != 1 {
                    return EvalValue::Num(0.0);
                }
                let query = self.eval_expr(&args[0], ctx);
                let query_str = query.as_string().unwrap_or("");
                let cards = scope_cards(ctx, query_str);
                let mut best: Option<i64> = None;
                for card in cards {
                    if card.is_stone() {
                        continue;
                    }
                    let value = self.tables.rank_chips(card.rank);
                    best = Some(best.map_or(value, |current| current.min(value)));
                }
                EvalValue::Num(best.unwrap_or(0) as f64)
            }
            "max" => {
                let mut values = args
                    .iter()
                    .filter_map(|expr| self.eval_expr(expr, ctx).as_number())
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    EvalValue::None
                } else {
                    let mut best = values[0];
                    for value in values.drain(1..) {
                        if value > best {
                            best = value;
                        }
                    }
                    EvalValue::Num(best)
                }
            }
            "min" => {
                let mut values = args
                    .iter()
                    .filter_map(|expr| self.eval_expr(expr, ctx).as_number())
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    EvalValue::None
                } else {
                    let mut best = values[0];
                    for value in values.drain(1..) {
                        if value < best {
                            best = value;
                        }
                    }
                    EvalValue::Num(best)
                }
            }
            "floor" => {
                if args.len() != 1 {
                    return EvalValue::None;
                }
                let value = self.eval_expr(&args[0], ctx).as_number();
                value.map(|v| EvalValue::Num(v.floor())).unwrap_or(EvalValue::None)
            }
            "ceil" => {
                if args.len() != 1 {
                    return EvalValue::None;
                }
                let value = self.eval_expr(&args[0], ctx).as_number();
                value.map(|v| EvalValue::Num(v.ceil())).unwrap_or(EvalValue::None)
            }
            "pow" => {
                if args.len() != 2 {
                    return EvalValue::None;
                }
                let base = self.eval_expr(&args[0], ctx).as_number();
                let exp = self.eval_expr(&args[1], ctx).as_number();
                if let (Some(base), Some(exp)) = (base, exp) {
                    EvalValue::Num(base.powf(exp))
                } else {
                    EvalValue::None
                }
            }
            _ => EvalValue::None,
        }
    }

    pub(super) fn count_joker(&self, query: &str) -> usize {
        let key = normalize(query);
        if let Some(count) = self.current_joker_counts.get(&key) {
            return *count;
        }
        if let Some(def) = self
            .content
            .jokers
            .iter()
            .find(|joker| normalize(&joker.name) == key)
        {
            let id_key = normalize(&def.id);
            return *self.current_joker_counts.get(&id_key).unwrap_or(&0);
        }
        0
    }

    pub(super) fn deck_count(&self, ctx: &EvalContext<'_>) -> usize {
        self.deck.draw.len()
            + self.deck.discard.len()
            + ctx.held_cards.len()
            + ctx.played_cards.len()
            + ctx.discarded_cards.len()
    }

    pub(super) fn count_matching_deck(&mut self, ctx: &EvalContext<'_>, target: &str) -> usize {
        let smeared = self.smeared_suits_active();
        let draw = self.deck.draw.clone();
        let discard = self.deck.discard.clone();
        let mut total = 0;
        total += self.count_matching_with_debuff(&draw, target, smeared);
        total += self.count_matching_with_debuff(&discard, target, smeared);
        total += self.count_matching_with_debuff(ctx.held_cards, target, smeared);
        total += self.count_matching_with_debuff(ctx.played_cards, target, smeared);
        total += self.count_matching_with_debuff(ctx.discarded_cards, target, smeared);
        total
    }

    fn count_matching_with_debuff(&mut self, cards: &[Card], target: &str, smeared: bool) -> usize {
        let target_norm = normalize(target);
        match target_norm.as_str() {
            "any" | "all" => cards.len(),
            "face" => cards
                .iter()
                .filter(|card| {
                    if card.is_stone() || self.is_card_debuffed(**card) {
                        return false;
                    }
                    if self.pareidolia_active() {
                        true
                    } else {
                        is_face(**card)
                    }
                })
                .count(),
            "odd" => cards
                .iter()
                .filter(|card| {
                    !card.is_stone() && !self.is_card_debuffed(**card) && is_odd(**card)
                })
                .count(),
            "even" => cards
                .iter()
                .filter(|card| {
                    !card.is_stone() && !self.is_card_debuffed(**card) && is_even(**card)
                })
                .count(),
            "wild" => cards
                .iter()
                .filter(|card| !self.is_card_debuffed(**card) && card.is_wild())
                .count(),
            "stone" => cards
                .iter()
                .filter(|card| card.is_stone() && !self.is_card_debuffed(**card))
                .count(),
            "enhanced" => cards
                .iter()
                .filter(|card| !self.is_card_debuffed(**card) && card.enhancement.is_some())
                .count(),
            "black" => cards
                .iter()
                .filter(|card| !card.is_stone() && is_black(**card))
                .count(),
            "red" => cards
                .iter()
                .filter(|card| !card.is_stone() && is_red(**card))
                .count(),
            _ => {
                if let Some(suit) = suit_from_str(&target_norm) {
                    if smeared {
                        let target_group = smeared_suit_group(suit);
                        return cards
                            .iter()
                            .filter(|card| {
                                if card.is_stone() {
                                    return false;
                                }
                                let debuffed = self.is_card_debuffed(**card);
                                let is_wild = !debuffed && card.is_wild();
                                is_wild || smeared_suit_group(card.suit) == target_group
                            })
                            .count();
                    }
                    return cards
                        .iter()
                        .filter(|card| {
                            if card.is_stone() {
                                return false;
                            }
                            let debuffed = self.is_card_debuffed(**card);
                            let is_wild = !debuffed && card.is_wild();
                            is_wild || card.suit == suit
                        })
                        .count();
                }
                if let Some(rank) = rank_from_str(&target_norm) {
                    return cards
                        .iter()
                        .filter(|card| !card.is_stone() && card.rank == rank)
                        .count();
                }
                if let Some(kind) = enhancement_from_str(&target_norm) {
                    return cards
                        .iter()
                        .filter(|card| {
                            !self.is_card_debuffed(**card) && card.enhancement == Some(kind)
                        })
                        .count();
                }
                if let Some(kind) = edition_from_str(&target_norm) {
                    return cards
                        .iter()
                        .filter(|card| {
                            !self.is_card_debuffed(**card) && card.edition == Some(kind)
                        })
                        .count();
                }
                if let Some(kind) = seal_from_str(&target_norm) {
                    return cards
                        .iter()
                        .filter(|card| !self.is_card_debuffed(**card) && card.seal == Some(kind))
                        .count();
                }
                0
            }
        }
    }

    pub(super) fn json_conditions_met(
        &mut self,
        conditions: &[Condition],
        hand_kind: crate::HandKind,
        card: Option<Card>,
    ) -> bool {
        if conditions.is_empty() {
            return true;
        }
        let card_debuffed = card.map(|c| self.is_card_debuffed(c)).unwrap_or(false);
        for condition in conditions {
            let matched = match condition {
                Condition::Always => true,
                Condition::HandKind(kind) => *kind == hand_kind,
                Condition::BlindKind(kind) => *kind == self.state.blind,
                Condition::CardSuit(suit) => card
                    .map(|c| {
                        if c.is_wild() {
                            true
                        } else if self.smeared_suits_active() {
                            smeared_suit_group(c.suit) == smeared_suit_group(*suit)
                        } else {
                            c.suit == *suit
                        }
                    })
                    .unwrap_or(false),
                Condition::CardRank(rank) => card.map(|c| c.rank == *rank).unwrap_or(false),
                Condition::CardIsFace => card
                    .map(|c| {
                        !card_debuffed
                            && matches!(
                                c.rank,
                                crate::Rank::Jack | crate::Rank::Queen | crate::Rank::King
                            )
                    })
                    .unwrap_or(false),
                Condition::CardIsOdd => card
                    .map(|c| {
                        !card_debuffed
                            && matches!(
                                c.rank,
                                crate::Rank::Ace
                                    | crate::Rank::Three
                                    | crate::Rank::Five
                                    | crate::Rank::Seven
                                    | crate::Rank::Nine
                            )
                    })
                    .unwrap_or(false),
                Condition::CardIsEven => card
                    .map(|c| {
                        !card_debuffed
                            && matches!(
                                c.rank,
                                crate::Rank::Two
                                    | crate::Rank::Four
                                    | crate::Rank::Six
                                    | crate::Rank::Eight
                                    | crate::Rank::Ten
                            )
                    })
                    .unwrap_or(false),
                Condition::CardHasEnhancement(kind) => card
                    .map(|c| !card_debuffed && c.enhancement == Some(*kind))
                    .unwrap_or(false),
                Condition::CardHasEdition(kind) => card
                    .map(|c| !card_debuffed && c.edition == Some(*kind))
                    .unwrap_or(false),
                Condition::CardHasSeal(kind) => card
                    .map(|c| !card_debuffed && c.seal == Some(*kind))
                    .unwrap_or(false),
                Condition::CardIsStone => card
                    .map(|c| !card_debuffed && c.is_stone())
                    .unwrap_or(false),
                Condition::CardIsWild => card
                    .map(|c| !card_debuffed && c.is_wild())
                    .unwrap_or(false),
                Condition::IsBossBlind => self.state.blind == BlindKind::Boss,
                Condition::IsScoringCard | Condition::IsHeldCard | Condition::IsPlayedCard => false,
            };
            if !matched {
                return false;
            }
        }
        true
    }

}
