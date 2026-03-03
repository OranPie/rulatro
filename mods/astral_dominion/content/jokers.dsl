# Astral Dominion — Joker Definitions
# Four factions: Celestial, Shadow, Fortune, Arcane
# 32 jokers with complex hook chains, variable tracking, cross-joker synergies.
#
# ─────────────────────────────────────────────────────
#  FACTION I: CELESTIAL  (stars, planets, cosmic runs)
# ─────────────────────────────────────────────────────

joker ad_archivist "The Archivist" Common {
  desc "Each discard permanently adds +4 Chips; currently +{chips} Chips"
  on acquire when var(init) == 0 { set_var chips 0; set_var init 1 }
  on discard { add_var chips 4 }
  on independent { add_chips var(chips) }
}

joker ad_stellar_core "Stellar Core" Uncommon {
  desc "Gain +Mult equal to 3 times the current hand's level"
  on independent { add_mult hand_level * 3 }
}

joker ad_star_eater "Star Eater" Uncommon {
  desc "Each Planet card used permanently gives +2 Mult; {devoured} devoured"
  on acquire when var(init) == 0 { set_var devoured 0; set_var init 1 }
  on use when consumable.kind == Planet { add_var devoured 1 }
  on independent { add_mult var(devoured) * 2 }
}

joker ad_omen_reader "Omen Reader" Uncommon {
  desc "At the end of each round, add 1 Tarot card to your consumables"
  on round_end { add_tarot 1 }
}

joker ad_void_wanderer "Void Wanderer" Rare {
  desc "Each failed Blind permanently gives x1 Mult; currently x{power} Mult"
  on acquire when var(init) == 0 { set_var power 1; set_var init 1 }
  on blind_failed { add_var power 1 }
  on independent { mul_mult var(power) }
}

joker ad_nebula_weaver "Nebula Weaver" Uncommon {
  desc "Gain +4 Chips for each unique Planet type used this run"
  on independent { add_chips unique_planets_used * 4 }
}

joker ad_pulsar "Pulsar" Rare {
  desc "Alternates each round — Phase 1: +60 Chips; Phase 0: +18 Mult; currently Phase {phase}"
  on acquire when var(init) == 0 { set_var phase 1; set_var init 1 }
  on round_end when var(phase) == 1 { set_var phase 0 }
  on round_end when var(phase) == 0 { set_var phase 1 }
  on independent when var(phase) == 1 { add_chips 60 }
  on independent when var(phase) == 0 { add_mult 18 }
}

joker ad_event_horizon "Event Horizon" Legendary {
  desc "On Boss Blind start: upgrade random hand + free reroll; x2 Mult during Boss Blinds"
  on blind_start when is_boss_blind { upgrade_random_hand; add_free_reroll 1 }
  on independent when is_boss_blind { mul_mult 2 }
}

# ─────────────────────────────────────────────────────
#  FACTION II: SHADOW  (destruction, risk, dark trades)
# ─────────────────────────────────────────────────────

joker ad_blood_moon "Blood Moon" Rare {
  desc "During Boss Blinds: x3 Mult; each scored Face card also gives +5 Mult"
  on independent when is_boss_blind { mul_mult 3 }
  on scored when is_boss_blind && card.is_face { add_mult 5 }
}

joker ad_death_spiral "Death Spiral" Uncommon {
  desc "Each destroyed card permanently gives +2 Mult; currently +{mult} Mult"
  on acquire when var(init) == 0 { set_var mult 0; set_var init 1 }
  on card_destroyed { add_var mult 2 }
  on independent { add_mult var(mult) }
}

joker ad_phantom_blade "Phantom Blade" Uncommon {
  desc "Each Face card held in hand gives +15 Chips"
  on held when card.is_face { add_chips 15 }
}

joker ad_glass_cannon "Glass Cannon" Rare {
  desc "x4 Mult every hand; destroys itself on a failed Boss Blind"
  on independent { mul_mult 4 }
  on blind_failed when is_boss_blind { destroy_self }
}

joker ad_shadow_veil "Shadow Veil" Common {
  desc "Each scored Wild card gives +8 Mult"
  on scored when card.is_wild { add_mult 8 }
}

joker ad_bone_collector "Bone Collector" Uncommon {
  desc "Each Joker sold anywhere permanently gives +4 Mult; currently +{mult} Mult"
  on acquire when var(init) == 0 { set_var mult 0; set_var init 1 }
  on any_sell { add_var mult 4 }
  on independent { add_mult var(mult) }
}

joker ad_executioner "The Executioner" Uncommon {
  desc "When any Joker is sold, gain extra money equal to floor(sell value x 1.5)"
  on any_sell { add_money floor(last_destroyed_sell_value * 1.5) }
}

joker ad_soul_bargain "Soul Bargain" Rare {
  desc "On Blind start: sacrifice all discards for +8 Chips each (resets each blind); currently +{chips_gained} Chips"
  on acquire when var(init) == 0 { set_var chips_gained 0; set_var init 1 }
  on blind_start { add_var chips_gained discards_max * 8; set_discards 0 }
  on independent { add_chips var(chips_gained) }
  on round_end { set_var chips_gained 0 }
}

# ─────────────────────────────────────────────────────
#  FACTION III: FORTUNE  (luck, money, gambling)
# ─────────────────────────────────────────────────────

joker ad_midas_gloves "Midas Gloves" Uncommon {
  desc "Scored Gold seal cards give +$3; also +1 Mult per $10 held (above $20)"
  on scored when card.seal == Gold { add_money 3 }
  on independent when money >= 20 { add_mult floor(money / 10) }
}

joker ad_the_gambler "The Gambler" Common {
  desc "Each hand, gain a random +1 to +8 Mult (roll of fate)"
  on independent { add_mult rand(1, 8) }
}

joker ad_lucky_constellation "Lucky Constellation" Uncommon {
  desc "Lucky card triggers permanently charge this Joker; currently +{charge} Mult"
  on acquire when var(init) == 0 { set_var charge 0; set_var init 1 }
  on scored when card.enhancement == Lucky && card.lucky_triggers > 0 {
    add_var charge card.lucky_triggers
  }
  on independent { add_mult var(charge) }
}

joker ad_the_hoarder "The Hoarder" Uncommon {
  desc "Gain +1 Mult for every $5 you currently hold"
  on independent { add_mult floor(money / 5) }
}

joker ad_coin_flipper "Coin Flipper" Common {
  desc "Alternates each hand — Heads: +50 Chips; Tails: +15 Mult; currently {face} (1=Heads)"
  on acquire when var(init) == 0 { set_var face 1; set_var init 1 }
  on played when var(face) == 1 { set_var face 0 }
  on played when var(face) == 0 { set_var face 1 }
  on independent when var(face) == 1 { add_chips 50 }
  on independent when var(face) == 0 { add_mult 15 }
}

joker ad_merchant_prince "Merchant Prince" Rare {
  desc "All Jokers gain +$1 sell value each round; gain +Mult equal to 1/3 of total sell value"
  on round_end { add_sell_bonus all 1 }
  on independent { add_mult floor(other_joker_sell_value / 3) }
}

joker ad_treasure_hunter "Treasure Hunter" Common {
  desc "Each pack opened permanently gives +10 Chips; currently +{chips} Chips"
  on acquire when var(init) == 0 { set_var chips 0; set_var init 1 }
  on pack_opened { add_var chips 10 }
  on independent { add_chips var(chips) }
}

joker ad_jackpot_jinx "Jackpot Jinx" Rare {
  desc "Gain +$25 every round end; destroys itself and loses all money on failed Boss Blind"
  on round_end { add_money 25 }
  on blind_failed when is_boss_blind { destroy_self }
}

# ─────────────────────────────────────────────────────
#  FACTION IV: ARCANE  (rituals, consumables, spells)
# ─────────────────────────────────────────────────────

joker ad_wraith_hunter "Wraith Hunter" Rare {
  desc "Each Spectral card used permanently gives x1 Mult; currently x{power} Mult"
  on acquire when var(init) == 0 { set_var power 1; set_var init 1 }
  on use when consumable.kind == Spectral { add_var power 1 }
  on independent { mul_mult var(power) }
}

joker ad_storm_bringer "Storm Bringer" Rare {
  desc "Gain +3 Mult per hand played this blind (resets each blind start); currently +{buildup} Mult"
  on acquire when var(init) == 0 { set_var buildup 0; set_var init 1 }
  on hand_end { add_var buildup 3 }
  on blind_start { set_var buildup 0 }
  on independent { add_mult var(buildup) }
}

joker ad_the_conductor "The Conductor" Rare {
  desc "Upgrades a random hand type each round; when hand level ≥ 3, gain +Mult equal to hand level"
  on round_end { upgrade_random_hand }
  on independent when hand_level >= 3 { add_mult hand_level }
}

joker ad_tarot_sage "Tarot Sage" Uncommon {
  desc "Each Tarot card used permanently gives +5 Mult; currently +{mult} Mult"
  on acquire when var(init) == 0 { set_var mult 0; set_var init 1 }
  on use when consumable.kind == Tarot { add_var mult 5 }
  on independent { add_mult var(mult) }
}

joker ad_ritual_master "Ritual Master" Uncommon {
  desc "Each consumable used permanently gives +3 Chips; {rites} rites performed"
  on acquire when var(init) == 0 { set_var rites 0; set_var init 1 }
  on use { add_var rites 1 }
  on independent { add_chips var(rites) * 3 }
}

joker ad_spectral_echo "Spectral Echo" Uncommon {
  desc "Once per hand: using a Spectral card causes all scored cards to retrigger once"
  on acquire when var(init) == 0 { set_var echoed 0; set_var init 1 }
  on use when consumable.kind == Spectral && var(echoed) == 0 {
    set_var echoed 1; retrigger_scored 1
  }
  on hand_end { set_var echoed 0 }
}

joker ad_prism "Prism" Uncommon {
  desc "Each scored Foil, Holographic, or Polychrome card multiplies Mult by x1.3"
  on scored when card.edition == Foil { mul_mult 1.3 }
  on scored when card.edition == Holographic { mul_mult 1.3 }
  on scored when card.edition == Polychrome { mul_mult 1.3 }
}

joker ad_eclipse "Eclipse" Rare {
  desc "Alternates each round — Phase 1: x3 Mult; Phase 0: +80 Chips; currently Phase {phase}"
  on acquire when var(init) == 0 { set_var phase 1; set_var init 1 }
  on round_end when var(phase) == 1 { set_var phase 0 }
  on round_end when var(phase) == 0 { set_var phase 1 }
  on independent when var(phase) == 1 { mul_mult 3 }
  on independent when var(phase) == 0 { add_chips 80 }
}

# ─────────────────────────────────────────────────────
#  CROSS-FACTION SYNERGY JOKERS (Legendary)
# ─────────────────────────────────────────────────────

joker ad_cosmic_convergence "Cosmic Convergence" Legendary {
  desc "xMult scales with joker count; with AD jokers: free shop rerolls and hand upgrades; x(1 + joker_count x 0.15)"
  on acquire when var(init) == 0 { set_var init 1 }
  on shop_enter when count_joker("ad_archivist") + count_joker("ad_blood_moon") + count_joker("ad_the_gambler") + count_joker("ad_wraith_hunter") >= 1 {
    add_free_reroll 1
  }
  on round_end when count_joker("ad_stellar_core") + count_joker("ad_death_spiral") + count_joker("ad_lucky_constellation") + count_joker("ad_tarot_sage") >= 2 {
    upgrade_random_hand
  }
  on independent { mul_mult max(1, joker_count * 0.15 + 1) }
}

joker ad_devourer "The Devourer" Legendary {
  desc "On Blind start (up to 5×), destroys the left-most Joker; permanently gains x0.5 Mult and Chips from its sell value; {devoured} devoured"
  on acquire when var(init) == 0 { set_var devoured 0; set_var init 1 }
  on blind_start when var(devoured) < 5 {
    add_var devoured 1; destroy_joker_left 1
  }
  on independent { mul_mult (1 + var(devoured) * 0.5) }
  on independent { add_chips var(devoured) * last_destroyed_sell_value * 2 }
}
