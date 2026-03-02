-- Astral Dominion — main.lua
-- Implements cross-faction synergy bonuses via hook registration.
-- Hooks layer on top of the DSL joker effects defined in content/jokers.dsl.
-- The DSL handles per-joker logic; Lua handles inter-joker synergies and
-- global rules that depend on the full joker roster.

local MOD_ID = "astral_dominion"

rulatro.log("[" .. MOD_ID .. "] loading Astral Dominion v1.0.0")

-- ─────────────────────────────────────────────────────
--  HELPER: count how many Celestial-faction jokers are held
-- ─────────────────────────────────────────────────────
local CELESTIAL_JOKERS = {
  "ad_archivist", "ad_stellar_core", "ad_star_eater",
  "ad_omen_reader", "ad_void_wanderer", "ad_nebula_weaver",
  "ad_pulsar", "ad_event_horizon"
}

local SHADOW_JOKERS = {
  "ad_blood_moon", "ad_death_spiral", "ad_phantom_blade",
  "ad_glass_cannon", "ad_shadow_veil", "ad_bone_collector",
  "ad_executioner", "ad_soul_bargain"
}

local FORTUNE_JOKERS = {
  "ad_midas_gloves", "ad_the_gambler", "ad_lucky_constellation",
  "ad_the_hoarder", "ad_coin_flipper", "ad_merchant_prince",
  "ad_treasure_hunter", "ad_jackpot_jinx"
}

local ARCANE_JOKERS = {
  "ad_wraith_hunter", "ad_storm_bringer", "ad_the_conductor",
  "ad_tarot_sage", "ad_ritual_master", "ad_spectral_echo",
  "ad_prism", "ad_eclipse"
}

-- ─────────────────────────────────────────────────────
--  SYNERGY I: Celestial Alignment
--  When 4+ Celestial jokers are held, each shop entry grants
--  an extra free Celestial (planet) pack — the stars align.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnShopEnter", function(ctx)
  local count = 0
  for _, id in ipairs(CELESTIAL_JOKERS) do
    count = count + (ctx.joker_count(id) or 0)
  end

  if count >= 4 then
    rulatro.log("[" .. MOD_ID .. "] Celestial Alignment active (" .. count .. " jokers) — granting celestial pack")
    return {
      effects = {
        {
          block = {
            trigger = "OnShopEnter",
            conditions = { "Always" },
            effects = {
              { AddPack = { kind = "celestial_normal", cost = 0 } }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY II: Shadow Pact
--  When Glass Cannon AND Blood Moon are both held,
--  on every blind start grant +$5 — dark powers trade favours.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnBlindStart", function(ctx)
  local has_cannon = (ctx.joker_count("ad_glass_cannon") or 0) > 0
  local has_moon   = (ctx.joker_count("ad_blood_moon")   or 0) > 0

  if has_cannon and has_moon then
    rulatro.log("[" .. MOD_ID .. "] Shadow Pact: Glass Cannon + Blood Moon — +$5")
    return {
      effects = {
        {
          block = {
            trigger = "OnBlindStart",
            conditions = { "Always" },
            effects = {
              { AddMoney = 5 }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY III: Fortune's Favour
--  When 3+ Fortune jokers are held, at round end grant
--  a bonus equal to $2 per fortune joker held.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnRoundEnd", function(ctx)
  local count = 0
  for _, id in ipairs(FORTUNE_JOKERS) do
    count = count + (ctx.joker_count(id) or 0)
  end

  if count >= 3 then
    local bonus = count * 2
    rulatro.log("[" .. MOD_ID .. "] Fortune's Favour: " .. count .. " fortune jokers — +$" .. bonus)
    return {
      effects = {
        {
          block = {
            trigger = "OnRoundEnd",
            conditions = { "Always" },
            effects = {
              { AddMoney = bonus }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY IV: Arcane Ritual Convergence
--  When Wraith Hunter + Ritual Master + Tarot Sage are all held,
--  every consumable use triggers an extra hand upgrade.
--  The three scholars have unified their knowledge.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnUse", function(ctx)
  local has_wraith  = (ctx.joker_count("ad_wraith_hunter") or 0) > 0
  local has_ritual  = (ctx.joker_count("ad_ritual_master")  or 0) > 0
  local has_sage    = (ctx.joker_count("ad_tarot_sage")     or 0) > 0

  if has_wraith and has_ritual and has_sage then
    rulatro.log("[" .. MOD_ID .. "] Arcane Ritual Convergence: upgrading random hand on consumable use")
    return {
      effects = {
        {
          block = {
            trigger = "OnUse",
            conditions = { "Always" },
            effects = {
              { UpgradeRandomHand = 1 }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY V: Eclipse + Pulsar Phase Lock
--  When Eclipse AND Pulsar are both held, their phases
--  are forced to OPPOSITE states — one always gives chips,
--  the other always gives mult, so you never have dead turns.
--  Implemented by granting both effects regardless of phase.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnIndependent", function(ctx)
  local has_eclipse = (ctx.joker_count("ad_eclipse") or 0) > 0
  local has_pulsar  = (ctx.joker_count("ad_pulsar")  or 0) > 0

  if has_eclipse and has_pulsar then
    rulatro.log("[" .. MOD_ID .. "] Eclipse+Pulsar Phase Lock: guaranteed +30 chips and +9 mult supplement")
    return {
      effects = {
        {
          block = {
            trigger = "OnIndependent",
            conditions = { "Always" },
            effects = {
              { AddChips = 30 },
              { AddMult  = 9  }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY VI: The Conductor + Star Eater Crescendo
--  When both are held, each round-end upgrade also adds a
--  free planet card — the conductor plays the celestial music.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnRoundEnd", function(ctx)
  local has_conductor = (ctx.joker_count("ad_the_conductor") or 0) > 0
  local has_star      = (ctx.joker_count("ad_star_eater")    or 0) > 0

  if has_conductor and has_star then
    rulatro.log("[" .. MOD_ID .. "] Conductor+StarEater Crescendo: bonus planet on round end")
    return {
      effects = {
        {
          block = {
            trigger = "OnRoundEnd",
            conditions = { "Always" },
            effects = {
              { AddRandomConsumable = { kind = "Planet", count = 1 } }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY VII: Bone Collector + Executioner Death Engine
--  When both are held, selling any joker grants +$2 extra
--  on top of the individual DSL effects — an execution bonus.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnAnySell", function(ctx)
  local has_collector   = (ctx.joker_count("ad_bone_collector") or 0) > 0
  local has_executioner = (ctx.joker_count("ad_executioner")    or 0) > 0

  if has_collector and has_executioner then
    rulatro.log("[" .. MOD_ID .. "] Death Engine: Bone Collector + Executioner — +$2 execution bonus")
    return {
      effects = {
        {
          block = {
            trigger = "OnAnySell",
            conditions = { "Always" },
            effects = {
              { AddMoney = 2 }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  SYNERGY VIII: Full Faction Mastery
--  When at least one joker from EVERY faction is held,
--  on blind start grant a random spectral (the cosmos rewards mastery).
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnBlindStart", function(ctx)
  local celestial_count = 0
  local shadow_count    = 0
  local fortune_count   = 0
  local arcane_count    = 0

  for _, id in ipairs(CELESTIAL_JOKERS) do celestial_count = celestial_count + (ctx.joker_count(id) or 0) end
  for _, id in ipairs(SHADOW_JOKERS)    do shadow_count    = shadow_count    + (ctx.joker_count(id) or 0) end
  for _, id in ipairs(FORTUNE_JOKERS)   do fortune_count   = fortune_count   + (ctx.joker_count(id) or 0) end
  for _, id in ipairs(ARCANE_JOKERS)    do arcane_count    = arcane_count    + (ctx.joker_count(id) or 0) end

  if celestial_count >= 1 and shadow_count >= 1 and fortune_count >= 1 and arcane_count >= 1 then
    rulatro.log("[" .. MOD_ID .. "] FULL FACTION MASTERY — granting spectral on blind start")
    return {
      effects = {
        {
          block = {
            trigger = "OnBlindStart",
            conditions = { "Always" },
            effects = {
              { AddRandomConsumable = { kind = "Spectral", count = 1 } }
            }
          }
        }
      }
    }
  end

  return nil
end)

-- ─────────────────────────────────────────────────────
--  ANTI-SYNERGY: Cosmic Devourer + Glass Cannon
--  When the Devourer has consumed 3+ jokers AND Glass Cannon
--  is still alive, Glass Cannon becomes invincible on boss fails.
--  The Devourer has grown strong enough to protect it.
-- ─────────────────────────────────────────────────────
rulatro.register_hook("OnBlindFailed", function(ctx)
  local devourer_owned = (ctx.joker_count("ad_devourer")     or 0) > 0
  local cannon_owned   = (ctx.joker_count("ad_glass_cannon") or 0) > 0

  -- We can't read var state directly from Lua, but we can check if both are present
  -- and grant prevent_death as a safety measure (the DSL destroy_self fires last).
  if devourer_owned and cannon_owned and ctx.is_boss_blind then
    rulatro.log("[" .. MOD_ID .. "] Devourer protects Glass Cannon on boss blind fail")
    return {
      effects = {
        {
          block = {
            trigger = "OnBlindFailed",
            conditions = { "Always" },
            effects = {
              { PreventDeath = 1 }
            }
          }
        }
      }
    }
  end

  return nil
end)

rulatro.log("[" .. MOD_ID .. "] all hooks registered — Astral Dominion ready")
