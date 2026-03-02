# Astral Dominion — Joker Definitions
# Four factions: Celestial, Shadow, Fortune, Arcane
# All 32 jokers use complex hook chains, variable tracking, cross-joker synergies,
# and varied trigger/condition/action combinations.
#
# ─────────────────────────────────────────────────────
#  FACTION I: CELESTIAL  (stars, planets, cosmic runs)
# ─────────────────────────────────────────────────────

# The Archivist — Memory builds over the run.
# Each discard charges the archive; those chips score every hand.
joker ad_archivist "The Archivist" Common {
  on acquire when var(init) == 0 { set_var chips 0; set_var init 1 }
  on discard { add_var chips 4 }
  on independent { add_chips var(chips) }
}

# Stellar Core — The current hand's level radiates energy.
# +3 mult × hand_level; combos powerfully with planets and The Conductor.
joker ad_stellar_core "Stellar Core" Uncommon {
  on independent { add_mult hand_level * 3 }
}

# Star Eater — Grows hungrier with every planet consumed.
# Use planets to feed it; pair with Omen Reader for passive accumulation.
joker ad_star_eater "Star Eater" Uncommon {
  on acquire when var(init) == 0 { set_var devoured 0; set_var init 1 }
  on use when consumable.kind == Planet { add_var devoured 1 }
  on independent { add_mult var(devoured) * 2 }
}

# Omen Reader — The stars whisper of what comes next.
# Generates a tarot card at end of each round; synergises with Tarot Sage.
joker ad_omen_reader "Omen Reader" Uncommon {
  on round_end { add_tarot 1 }
}

# Void Wanderer — Power rises from the scars of failure.
# Each failed blind grants permanent x mult; risky but potent.
joker ad_void_wanderer "Void Wanderer" Rare {
  on acquire when var(init) == 0 { set_var power 1; set_var init 1 }
  on blind_failed { add_var power 1 }
  on independent { mul_mult var(power) }
}

# Nebula Weaver — Threads planets together into cascading energy.
# +4 chips per unique planet type ever used; becomes enormous late-game.
joker ad_nebula_weaver "Nebula Weaver" Uncommon {
  on independent { add_chips unique_planets_used * 4 }
}

# Pulsar — A cosmic heartbeat: chips on even rounds, mult on odd rounds.
# Phase flips each round; plan around it for burst combos.
joker ad_pulsar "Pulsar" Rare {
  on acquire when var(init) == 0 { set_var phase 1; set_var init 1 }
  on round_end when var(phase) == 1 { set_var phase 0 }
  on round_end when var(phase) == 0 { set_var phase 1 }
  on independent when var(phase) == 1 { add_chips 60 }
  on independent when var(phase) == 0 { add_mult 18 }
}

# Event Horizon — Gravitational inevitability.
# At the start of each boss blind, upgrades a random hand and grants a free reroll.
# Represents the event horizon: once you cross it, laws change.
joker ad_event_horizon "Event Horizon" Legendary {
  on blind_start when is_boss_blind { upgrade_random_hand; add_free_reroll 1 }
  on independent when is_boss_blind { mul_mult 2 }
}

# ─────────────────────────────────────────────────────
#  FACTION II: SHADOW  (destruction, risk, dark trades)
# ─────────────────────────────────────────────────────

# Blood Moon — Darkness amplifies the ritual.
# Boss blinds are feeding grounds: x3 mult; elsewhere silent.
joker ad_blood_moon "Blood Moon" Rare {
  on independent when is_boss_blind { mul_mult 3 }
  on scored when is_boss_blind && card.is_face { add_mult 5 }
}

# Death Spiral — Fed by destruction, endless in hunger.
# Every destroyed card (yours or opponent's) increases mult permanently.
joker ad_death_spiral "Death Spiral" Uncommon {
  on acquire when var(init) == 0 { set_var mult 0; set_var init 1 }
  on card_destroyed { add_var mult 2 }
  on independent { add_mult var(mult) }
}

# Phantom Blade — The unseen hand strikes hardest.
# Face cards held in hand deal 15 chips each; unseen, unfelt.
joker ad_phantom_blade "Phantom Blade" Uncommon {
  on held when card.is_face { add_chips 15 }
}

# Glass Cannon — Immense power, catastrophic fragility.
# x4 mult every hand; but if you fail a boss blind, this joker shatters.
# Pair with Luchador or chicot to prevent loss.
joker ad_glass_cannon "Glass Cannon" Rare {
  on independent { mul_mult 4 }
  on blind_failed when is_boss_blind { destroy_self }
}

# Shadow Veil — Formlessness is the ultimate weapon.
# Wild cards scored deal +8 mult each; build a wild-heavy deck to amplify.
joker ad_shadow_veil "Shadow Veil" Common {
  on scored when card.is_wild { add_mult 8 }
}

# Bone Collector — Gathers the remnants of fallen allies.
# Every joker sold (including this one last) adds 4 mult permanently.
# Synergises with Executioner; sell bad jokers for compounding rewards.
joker ad_bone_collector "Bone Collector" Uncommon {
  on acquire when var(init) == 0 { set_var mult 0; set_var init 1 }
  on any_sell { add_var mult 4 }
  on independent { add_mult var(mult) }
}

# The Executioner — Death is profitable.
# On sell, gain money equal to the sold joker's value × 1.5 (rounded).
# Destroy cheap jokers first to fund stronger ones.
joker ad_executioner "The Executioner" Uncommon {
  on any_sell { add_money floor(last_destroyed_sell_value * 1.5) }
}

# Soul Bargain — Sacrifice discards for unstoppable might.
# On blind start, forfeit all discards for this blind in exchange for x mult per forfeited discard.
# Zero discards left is your strength; requires careful planning.
joker ad_soul_bargain "Soul Bargain" Rare {
  on acquire when var(init) == 0 { set_var chips_gained 0; set_var init 1 }
  on blind_start { add_var chips_gained discards_max * 8; set_discards 0 }
  on independent { add_chips var(chips_gained) }
  on round_end { set_var chips_gained 0 }
}

# ─────────────────────────────────────────────────────
#  FACTION III: FORTUNE  (luck, money, gambling)
# ─────────────────────────────────────────────────────

# Midas Gloves — Everything the golden hand touches becomes profit.
# Gold seal cards scored grant +$3 each; run with talisman spectrals.
joker ad_midas_gloves "Midas Gloves" Uncommon {
  on scored when card.seal == Gold { add_money 3 }
  on independent when money >= 20 { add_mult floor(money / 10) }
}

# The Gambler — Fortune favours the bold (and the lucky).
# Each hand scored adds rand(1,8) mult. High variance, high reward.
joker ad_the_gambler "The Gambler" Common {
  on independent { add_mult rand(1, 8) }
}

# Lucky Constellation — Stars align for the fortunate.
# Each lucky trigger permanently charges the constellation; pairs with Lucky Cat.
joker ad_lucky_constellation "Lucky Constellation" Uncommon {
  on acquire when var(init) == 0 { set_var charge 0; set_var init 1 }
  on scored when card.enhancement == Lucky && card.lucky_triggers > 0 {
    add_var charge card.lucky_triggers
  }
  on independent { add_mult var(charge) }
}

# The Hoarder — Wealth is its own reward.
# +1 mult per every $5 held; the richer you stay, the stronger you become.
joker ad_the_hoarder "The Hoarder" Uncommon {
  on independent { add_mult floor(money / 5) }
}

# Coin Flipper — Probability is just a law waiting to be broken.
# Rolls a d2 each hand: heads = +50 chips; tails = +15 mult. Never boring.
joker ad_coin_flipper "Coin Flipper" Common {
  on acquire when var(init) == 0 { set_var face 1; set_var init 1 }
  on played when var(face) == 1 { set_var face 0 }
  on played when var(face) == 0 { set_var face 1 }
  on independent when var(face) == 1 { add_chips 50 }
  on independent when var(face) == 0 { add_mult 15 }
}

# Merchant Prince — Trade routes compound over centuries.
# Gains sell bonus on all jokers each round (+$1 each), then scores that total as mult.
joker ad_merchant_prince "Merchant Prince" Rare {
  on round_end { add_sell_bonus all 1 }
  on independent { add_mult floor(other_joker_sell_value / 3) }
}

# Treasure Hunter — Packs hide riches for those willing to dig.
# Each pack opened permanently boosts chips; run with lots of packs.
joker ad_treasure_hunter "Treasure Hunter" Common {
  on acquire when var(init) == 0 { set_var chips 0; set_var init 1 }
  on pack_opened { add_var chips 10 }
  on independent { add_chips var(chips) }
}

# Jackpot Jinx — All or nothing; the gambler's final bet.
# +$25 every round end. But on a failed boss blind: all money lost and self-destructs.
# The ultimate high-risk money engine.
joker ad_jackpot_jinx "Jackpot Jinx" Rare {
  on round_end { add_money 25 }
  on blind_failed when is_boss_blind { destroy_self }
}

# ─────────────────────────────────────────────────────
#  FACTION IV: ARCANE  (rituals, consumables, spells)
# ─────────────────────────────────────────────────────

# Wraith Hunter — Every spectral consumed is power contained.
# Using spectrals permanently charges this joker's xmult; pairs with Seance.
joker ad_wraith_hunter "Wraith Hunter" Rare {
  on acquire when var(init) == 0 { set_var power 1; set_var init 1 }
  on use when consumable.kind == Spectral { add_var power 1 }
  on independent { mul_mult var(power) }
}

# Storm Bringer — The tempest builds between rounds, unleashed each hand.
# +3 mult each hand played; resets at blind start; a sprint, not a marathon.
joker ad_storm_bringer "Storm Bringer" Rare {
  on acquire when var(init) == 0 { set_var buildup 0; set_var init 1 }
  on hand_end { add_var buildup 3 }
  on blind_start { set_var buildup 0 }
  on independent { add_mult var(buildup) }
}

# The Conductor — Orchestrates the evolution of your entire hand repertoire.
# Upgrades a random scoring hand type at round end; a slow but inevitable ramp.
joker ad_the_conductor "The Conductor" Rare {
  on round_end { upgrade_random_hand }
  on independent when hand_level >= 3 { add_mult hand_level }
}

# Tarot Sage — Ancient knowledge of the cards grants clarity.
# Each tarot used grants +5 mult permanently; pairs with Omen Reader.
joker ad_tarot_sage "Tarot Sage" Uncommon {
  on acquire when var(init) == 0 { set_var mult 0; set_var init 1 }
  on use when consumable.kind == Tarot { add_var mult 5 }
  on independent { add_mult var(mult) }
}

# Ritual Master — Every invocation leaves traces.
# Tracks total consumables used; +3 chips per ritual performed.
joker ad_ritual_master "Ritual Master" Uncommon {
  on acquire when var(init) == 0 { set_var rites 0; set_var init 1 }
  on use { add_var rites 1 }
  on independent { add_chips var(rites) * 3 }
}

# Spectral Echo — The ritual reverberates through the deck.
# Once per hand, when any spectral is used, all scored cards trigger again.
# Use a spectral before playing your hand for bonus retriggers.
joker ad_spectral_echo "Spectral Echo" Uncommon {
  on acquire when var(init) == 0 { set_var echoed 0; set_var init 1 }
  on use when consumable.kind == Spectral && var(echoed) == 0 {
    set_var echoed 1; retrigger_scored 1
  }
  on hand_end { set_var echoed 0 }
}

# Prism — Editions refract scoring into glorious spectacle.
# Each scored card with any edition multiplies mult by 1.3; stack editions for absurd results.
joker ad_prism "Prism" Uncommon {
  on scored when card.edition == Foil { mul_mult 1.3 }
  on scored when card.edition == Holographic { mul_mult 1.3 }
  on scored when card.edition == Polychrome { mul_mult 1.3 }
}

# Eclipse — A dark ritual that alternates between catastrophe and brilliance.
# Phase A: x3 mult. Phase B: +80 chips. Shifts each round; plan your deck around phases.
# Complement with Pulsar for phase-chained combos.
joker ad_eclipse "Eclipse" Rare {
  on acquire when var(init) == 0 { set_var phase 1; set_var init 1 }
  on round_end when var(phase) == 1 { set_var phase 0 }
  on round_end when var(phase) == 0 { set_var phase 1 }
  on independent when var(phase) == 1 { mul_mult 3 }
  on independent when var(phase) == 0 { add_chips 80 }
}

# ─────────────────────────────────────────────────────
#  CROSS-FACTION SYNERGY JOKERS (Legendary / Rare)
# ─────────────────────────────────────────────────────

# Cosmic Convergence — When all four factions meet, reality bends.
# Counts how many distinct Astral Dominion faction jokers are held.
# For every 4 faction jokers, add a free reroll and upgrade a random hand.
joker ad_cosmic_convergence "Cosmic Convergence" Legendary {
  on acquire when var(init) == 0 { set_var init 1 }
  on shop_enter when count_joker("ad_archivist") + count_joker("ad_blood_moon") + count_joker("ad_the_gambler") + count_joker("ad_wraith_hunter") >= 1 {
    add_free_reroll 1
  }
  on round_end when count_joker("ad_stellar_core") + count_joker("ad_death_spiral") + count_joker("ad_lucky_constellation") + count_joker("ad_tarot_sage") >= 2 {
    upgrade_random_hand
  }
  on independent { mul_mult max(1, joker_count * 0.15 + 1) }
}

# The Devourer — Consumes the weakest link for mutual power.
# On blind start, destroys the leftmost joker and permanently grows from its value.
# A dangerous ritual that rewards careful joker ordering.
joker ad_devourer "The Devourer" Legendary {
  on acquire when var(init) == 0 { set_var devoured 0; set_var init 1 }
  on blind_start when var(devoured) < 5 {
    add_var devoured 1; destroy_joker_left 1
  }
  on independent { mul_mult (1 + var(devoured) * 0.5) }
  on independent { add_chips var(devoured) * last_destroyed_sell_value * 2 }
}
