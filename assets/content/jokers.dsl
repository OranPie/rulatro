# Joker DSL (templates + definitions). One joker per block.
# Variables: hand, hand_id, hand_level, most_played_hand, blind, played_count, scoring_count, held_count, deck_count,
#            hands_left, hands_max, discards_left, discards_max, joker_count, joker_slots, empty_joker_slots, hand_play_count,
#            money, hand_size, ante, blind_score, target, is_boss_blind, boss_disabled,
#            blinds_skipped, hands_played, unused_discards, unique_planets_used,
#            last_destroyed_sell_value, other_joker_sell_value,
#            card.rank, card.rank_id, card.suit, card.suit_id, card.enhancement, card.edition, card.seal, card.lucky_triggers,
#            card.is_face, card.is_odd, card.is_even, card.is_stone, card.is_wild,
#            consumable.kind, consumable.id.
# Functions: contains(hand, Pair|TwoPair|ThreeOfAKind|Straight|Flush|FullHouse|Quads|...),
#            count(played|scoring|held|discarded|deck, target), count_rarity(rarity),
#            count_joker("joker_id"|"Joker Name"), suit_match(suit|id),
#            hand_count(hand), var(name), lowest_rank(scope),
#            min(...), max(...), floor(x), ceil(x), pow(base, exp),
#            roll(sides), rand(min, max).
# Actions: add_chips, add_mult, mul_mult, mul_chips, add_money, add_hand_size,
#          add_hands, add_discards, set_discards, retrigger_scored, retrigger_held, add_stone_card,
#          add_tarot, add_planet, add_spectral, add_free_reroll, set_shop_price, add_sell_bonus,
#          add_joker, destroy_random_joker, destroy_joker_right, destroy_joker_left, destroy_self,
#          upgrade_hand, upgrade_random_hand, duplicate_random_joker, duplicate_random_consumable,
#          add_random_hand_card, disable_boss, prevent_death, copy_joker_right, copy_joker_leftmost,
#          add_tag, duplicate_next_tag, add_pack, add_shop_joker, add_voucher,
#          set_reroll_cost, set_shop_joker_edition, reroll_boss,
#          set_var, add_var.

template indep_add_mult(id, name, rarity, mult) {
  joker $id $name $rarity {
    on independent { add_mult $mult }
  }
}

template indep_add_mult_contains(id, name, rarity, hand, mult) {
  joker $id $name $rarity {
    on independent when contains(hand, $hand) { add_mult $mult }
  }
}

template indep_add_chips_contains(id, name, rarity, hand, chips) {
  joker $id $name $rarity {
    on independent when contains(hand, $hand) { add_chips $chips }
  }
}

template scored_suit_mult(id, name, rarity, suit, mult) {
  joker $id $name $rarity {
    on scored when card.suit == $suit { add_mult $mult }
  }
}

template scored_suit_chips(id, name, rarity, suit, chips) {
  joker $id $name $rarity {
    on scored when card.suit == $suit { add_chips $chips }
  }
}

template scored_rank_mult(id, name, rarity, rank, mult) {
  joker $id $name $rarity {
    on scored when card.rank == $rank { add_mult $mult }
  }
}

template indep_mul_mult_contains(id, name, rarity, hand, mult) {
  joker $id $name $rarity {
    on independent when contains(hand, $hand) { mul_mult $mult }
  }
}

template scored_face_chips(id, name, rarity, chips) {
  joker $id $name $rarity {
    on scored when card.is_face { add_chips $chips }
  }
}

template scored_face_mult(id, name, rarity, mult) {
  joker $id $name $rarity {
    on scored when card.is_face { add_mult $mult }
  }
}

template copy_right_all(id, name, rarity) {
  joker $id $name $rarity {
    on played { copy_joker_right }
    on scored { copy_joker_right }
    on held { copy_joker_right }
    on independent { copy_joker_right }
    on discard { copy_joker_right }
    on discard_batch { copy_joker_right }
    on card_destroyed { copy_joker_right }
    on card_added { copy_joker_right }
    on round_end { copy_joker_right }
    on hand_end { copy_joker_right }
    on blind_start { copy_joker_right }
    on blind_failed { copy_joker_right }
    on shop_enter { copy_joker_right }
    on shop_reroll { copy_joker_right }
    on shop_exit { copy_joker_right }
    on pack_opened { copy_joker_right }
    on pack_skipped { copy_joker_right }
    on use { copy_joker_right }
    on sell { copy_joker_right }
    on any_sell { copy_joker_right }
    on acquire { copy_joker_right }
  }
}

template copy_leftmost_all(id, name, rarity) {
  joker $id $name $rarity {
    on played { copy_joker_leftmost }
    on scored { copy_joker_leftmost }
    on held { copy_joker_leftmost }
    on independent { copy_joker_leftmost }
    on discard { copy_joker_leftmost }
    on discard_batch { copy_joker_leftmost }
    on card_destroyed { copy_joker_leftmost }
    on card_added { copy_joker_leftmost }
    on round_end { copy_joker_leftmost }
    on hand_end { copy_joker_leftmost }
    on blind_start { copy_joker_leftmost }
    on blind_failed { copy_joker_leftmost }
    on shop_enter { copy_joker_leftmost }
    on shop_reroll { copy_joker_leftmost }
    on shop_exit { copy_joker_leftmost }
    on pack_opened { copy_joker_leftmost }
    on pack_skipped { copy_joker_leftmost }
    on use { copy_joker_leftmost }
    on sell { copy_joker_leftmost }
    on any_sell { copy_joker_leftmost }
    on acquire { copy_joker_leftmost }
  }
}

use indep_add_mult(joker, "Joker", Common, 4)

use scored_suit_mult(greedy_joker, "Greedy Joker", Common, Diamonds, 3)
use scored_suit_mult(lusty_joker, "Lusty Joker", Common, Hearts, 3)
use scored_suit_mult(wrathful_joker, "Wrathful Joker", Common, Spades, 3)
use scored_suit_mult(gluttonous_joker, "Gluttonous Joker", Common, Clubs, 3)

use indep_add_mult_contains(jolly_joker, "Jolly Joker", Common, Pair, 8)
use indep_add_mult_contains(zany_joker, "Zany Joker", Common, ThreeOfAKind, 12)
use indep_add_mult_contains(mad_joker, "Mad Joker", Common, TwoPair, 10)
use indep_add_mult_contains(crazy_joker, "Crazy Joker", Common, Straight, 12)
use indep_add_mult_contains(droll_joker, "Droll Joker", Common, Flush, 10)

use indep_add_chips_contains(sly_joker, "Sly Joker", Common, Pair, 50)
use indep_add_chips_contains(wily_joker, "Wily Joker", Common, ThreeOfAKind, 100)
use indep_add_chips_contains(clever_joker, "Clever Joker", Common, TwoPair, 80)
use indep_add_chips_contains(devious_joker, "Devious Joker", Common, Straight, 100)
use indep_add_chips_contains(crafty_joker, "Crafty Joker", Common, Flush, 80)

joker half_joker "Half Joker" Common {
  desc "If hand size played is 3 or fewer, gain +20 Mult"
  on independent when played_count <= 3 { add_mult 20 }
}

joker banner "Banner" Common {
  desc "Gain +30 Chips for each remaining discard"
  on independent { add_chips discards_left * 30 }
}

joker mystic_summit "Mystic Summit" Common {
  desc "When 0 discards remain, gain +15 Mult"
  on independent when discards_left == 0 { add_mult 15 }
}

use scored_face_chips(scary_face, "Scary Face", Common, 30)
use scored_face_mult(smiley_face, "Smiley Face", Common, 5)

joker abstract_joker "Abstract Joker" Common {
  desc "Gain +3 Mult for each Joker card"
  on independent { add_mult joker_count * 3 }
}

joker even_steven "Even Steven" Common {
  desc "Scored even-ranked cards each give +4 Mult"
  on scored when card.is_even { add_mult 4 }
}

joker odd_todd "Odd Todd" Common {
  desc "Scored odd-ranked cards each give +31 Chips"
  on scored when card.is_odd { add_chips 31 }
}

joker scholar "Scholar" Common {
  desc "Scored Aces give +20 Chips and +4 Mult"
  on scored when card.rank == Ace { add_chips 20; add_mult 4 }
}

joker fibonacci "Fibonacci" Uncommon {
  desc "Scored Ace, 2, 3, 5, or 8 each give +8 Mult"
  on scored when card.rank == Ace || card.rank == Two || card.rank == Three || card.rank == Five || card.rank == Eight { add_mult 8 }
}

joker mime "Mime" Uncommon {
  desc "Retrigger all held-in-hand card abilities once"
  on held { retrigger_held 1 }
}

joker dusk "Dusk" Uncommon {
  desc "On your final hand of the round, retrigger all scored cards once"
  on scored when hands_left == 1 { retrigger_scored 1 }
}

joker hack "Hack" Uncommon {
  desc "Retrigger each scored 2, 3, 4, or 5 once"
  on scored when card.rank == Two || card.rank == Three || card.rank == Four || card.rank == Five { retrigger_scored 1 }
}

joker business_card "Business Card" Common {
  desc "Scored face cards have a 1 in 2 chance to give $2"
  on scored when card.is_face && roll(2) { add_money 2 }
}

joker reserved_parking "Reserved Parking" Common {
  desc "Each face card held in hand has a 1 in 2 chance to give $1"
  on held when card.is_face && roll(2) { add_money 1 }
}

joker shoot_the_moon "Shoot the Moon" Common {
  desc "Each held Queen gives +13 Mult"
  on held when card.rank == Queen { add_mult 13 }
}

joker misprint "Misprint" Common {
  desc "Gain between +0 and +23 Mult (random each scoring)"
  on independent { add_mult rand(0, 23) }
}

joker ice_cream "Ice Cream" Common {
  desc "Starts with +{chips} Chips; loses 5 each hand played"
  on independent when var(init) == 0 { set_var chips 100; set_var init 1 }
  on independent { add_chips var(chips) }
  on independent { set_var chips max(0, var(chips) - 5) }
}

joker popcorn "Popcorn" Common {
  desc "Starts with +{mult} Mult; loses 4 each round"
  on independent when var(init) == 0 { set_var mult 20; set_var init 1 }
  on independent { add_mult var(mult) }
  on round_end { set_var mult max(0, var(mult) - 4) }
}

joker blue_joker "Blue Joker" Common {
  desc "Gain +2 Chips for each card remaining in deck"
  on independent { add_chips deck_count * 2 }
}

joker golden_joker "Golden Joker" Common {
  desc "Earn $4 at end of each round"
  on round_end { add_money 4 }
}

joker square_joker "Square Joker" Common {
  desc "Gain +4 Chips when exactly 4 cards are played; currently +{chips} Chips"
  on played when played_count == 4 { add_var chips 4 }
  on independent { add_chips var(chips) }
}

joker faceless_joker "Faceless Joker" Common {
  desc "If 3 or more face cards are discarded at once, earn $5"
  on discard_batch when count(discarded, face) >= 3 { add_money 5 }
}

joker marble_joker "Marble Joker" Uncommon {
  desc "When each Blind is selected, add a Stone card to the deck"
  on blind_start { add_stone_card 1 }
}

joker steel_joker "Steel Joker" Uncommon {
  desc "Gain X0.2 Mult for each Steel card in your full deck"
  on independent { mul_mult (1 + count(deck, steel) * 0.2) }
}

joker stone_joker "Stone Joker" Uncommon {
  desc "Gain +25 Chips for each Stone card in your full deck"
  on independent { add_chips count(deck, stone) * 25 }
}

joker wee_joker "Wee Joker" Rare {
  desc "Gain +8 Chips (stacking) when a 2 is scored; currently +{chips} Chips"
  on scored when card.rank == Two { add_var chips 8 }
  on independent { add_chips var(chips) }
}

joker supernova "Supernova" Common {
  desc "Gain +Mult equal to how many times the played hand type has been played this run"
  on independent { add_mult hand_play_count }
}

joker green_joker "Green Joker" Common {
  desc "Gain +1 Mult per hand played, lose 1 per discard; currently +{mult} Mult"
  on played { add_var mult 1 }
  on discard { set_var mult max(0, var(mult) - 1) }
  on independent { add_mult var(mult) }
}

joker ride_the_bus "Ride the Bus" Common {
  desc "Gain +1 Mult if no face cards scored; reset when a face card is scored; currently +{mult} Mult"
  on played when count(scoring, face) == 0 { add_var mult 1 }
  on scored when card.is_face { set_var mult 0 }
  on independent { add_mult var(mult) }
}

joker joker_stencil "Joker Stencil" Uncommon {
  desc "XMult equal to empty Joker slots plus copies of this joker"
  on independent { mul_mult (empty_joker_slots + count_joker("joker_stencil")) }
}

joker blackboard "Blackboard" Uncommon {
  desc "X3 Mult if all held cards are Spades or Clubs"
  on independent when count(held, black) == held_count { mul_mult 3 }
}

joker ancient_joker "Ancient Joker" Rare {
  desc "Scored cards of a random suit give X1.5 Mult; suit changes each round"
  on independent when var(init) == 0 { set_var suit rand(0, 3); set_var init 1 }
  on scored when suit_match(var(suit)) { mul_mult 1.5 }
  on round_end { set_var suit rand(0, 3) }
}

joker burnt_joker "Burnt Joker" Rare {
  desc "Once per round, the first discard upgrades that hand type"
  on blind_start { set_var used 0 }
  on discard_batch when var(used) == 0 { upgrade_hand hand; set_var used 1 }
}

joker space_joker "Space Joker" Uncommon {
  desc "1 in 4 chance to upgrade the played poker hand"
  on played when roll(4) { upgrade_hand hand }
}

joker glass_joker "Glass Joker" Uncommon {
  desc "Gain X0.75 Mult for each Glass card destroyed; currently X{mult} Mult"
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on destroyed when var(init) == 0 { set_var mult 1; set_var init 1 }
  on destroyed when card.enhancement == Glass { set_var mult (var(mult) + 0.75) }
  on independent { mul_mult var(mult) }
}

joker invisible_joker "Invisible Joker" Rare {
  desc "After 2 rounds, sell this to duplicate a random Joker"
  on round_end { add_var rounds 1 }
  on sell when var(rounds) >= 2 { duplicate_random_joker 1 }
}

joker smeared_joker "Smeared Joker" Uncommon {
  desc "Hearts and Diamonds count as each other; Spades and Clubs count as each other"
  on passive { set_rule smeared_suits 1 }
}

joker eight_ball "8 Ball" Common {
  desc "Each scored 8 has a 1 in 4 chance to create a Tarot card"
  on scored when card.rank == Eight && roll(4) { add_tarot 1 }
}

joker acrobat "Acrobat" Uncommon {
  desc "X3 Mult on your final hand of the round"
  on independent when hands_left == 1 { mul_mult 3 }
}

use scored_suit_chips(arrowhead, "Arrowhead", Uncommon, Spades, 50)

joker baron "Baron" Rare {
  desc "Each King held in hand gives X1.5 Mult"
  on held when card.rank == King { mul_mult 1.5 }
}

joker bloodstone "Bloodstone" Uncommon {
  desc "Each scored Heart has a 1 in 2 chance to give X1.5 Mult"
  on scored when suit_match(Hearts) && roll(2) { mul_mult 1.5 }
}

joker bootstraps "Bootstraps" Uncommon {
  desc "Gain +2 Mult for every $5 you have"
  on independent { add_mult floor(money / 5) * 2 }
}

joker bull "Bull" Uncommon {
  desc "Gain +2 Chips for each dollar you have"
  on independent { add_chips money * 2 }
}

joker burglar "Burglar" Uncommon {
  desc "When Blind is selected, gain +3 Hands and lose all discards"
  on blind_start { add_hands 3; set_discards 0 }
}

joker cartomancer "Cartomancer" Uncommon {
  desc "Create a Tarot card when each Blind is selected"
  on blind_start { add_tarot 1 }
}

joker cloud_9 "Cloud 9" Uncommon {
  desc "Earn $1 for each 9 in your full deck at end of round"
  on round_end { add_money count(deck, 9) }
}

joker drivers_license "Driver's License" Rare {
  desc "X3 Mult when you have at least 16 enhanced cards in your deck"
  on independent when count(deck, enhanced) >= 16 { mul_mult 3 }
}

joker erosion "Erosion" Uncommon {
  desc "Gain +4 Mult for each card below 52 in your deck"
  on independent { add_mult max(0, 52 - deck_count) * 4 }
}

joker constellation "Constellation" Uncommon {
  desc "Gain X0.1 Mult each time a Planet card is used; currently X{mult} Mult"
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on use when consumable.kind == Planet { set_var mult (var(mult) + 0.1) }
  on independent { mul_mult var(mult) }
}

use scored_suit_mult(onyx_agate, "Onyx Agate", Uncommon, Clubs, 7)

joker runner "Runner" Common {
  desc "Gain +15 Chips (stacking) each time a Straight is played; currently +{chips} Chips"
  on played when contains(hand, Straight) { add_var chips 15 }
  on independent { add_chips var(chips) }
}

joker spare_trousers "Spare Trousers" Uncommon {
  desc "Gain +2 Mult (stacking) each time a Two Pair is played; currently +{mult} Mult"
  on played when contains(hand, TwoPair) { add_var mult 2 }
  on independent { add_mult var(mult) }
}

use indep_mul_mult_contains(the_duo, "The Duo", Rare, Pair, 2)
use indep_mul_mult_contains(the_trio, "The Trio", Rare, ThreeOfAKind, 3)
use indep_mul_mult_contains(the_family, "The Family", Rare, Quads, 4)
use indep_mul_mult_contains(the_order, "The Order", Rare, Straight, 3)
use indep_mul_mult_contains(the_tribe, "The Tribe", Rare, Flush, 2)

joker chaos_the_clown "Chaos the Clown" Common {
  desc "1 free reroll each shop"
  on shop_enter { add_free_reroll 1 }
}

joker drunkard "Drunkard" Common {
  desc "Gain +1 Discard each round"
  on blind_start { add_discards 1 }
}

joker flash_card "Flash Card" Uncommon {
  desc "Gain +2 Mult each shop reroll; currently +{mult} Mult"
  on shop_reroll { add_var mult 2 }
  on independent { add_mult var(mult) }
}

joker card_sharp "Card Sharp" Uncommon {
  desc "X3 Mult if the played poker hand has been played this round before"
  on independent when hand_play_count > 1 { mul_mult 3 }
}

joker walkie_talkie "Walkie Talkie" Common {
  desc "Each scored 10 or 4 gives +10 Chips and +4 Mult"
  on scored when card.rank == Ten || card.rank == Four { add_chips 10; add_mult 4 }
}

joker rough_gem "Rough Gem" Uncommon {
  desc "Scored Diamond cards earn $1"
  on scored when suit_match(Diamonds) { add_money 1 }
}

joker golden_ticket "Golden Ticket" Common {
  desc "Scored Gold cards earn $4"
  on scored when card.enhancement == Gold { add_money 4 }
}

joker flower_pot "Flower Pot" Uncommon {
  desc "X3 Mult if played hand contains a Diamond, Club, Heart, and Spade"
  on independent when count(played, Diamonds) > 0 && count(played, Clubs) > 0 && count(played, Hearts) > 0 && count(played, Spades) > 0 { mul_mult 3 }
}

joker seeing_double "Seeing Double" Uncommon {
  desc "X2 Mult if played hand has a Club and any other suit scoring"
  on independent when count(scoring, Clubs) > 0 && (count(scoring, Diamonds) + count(scoring, Hearts) + count(scoring, Spades)) > 0 { mul_mult 2 }
}

joker superposition "Superposition" Common {
  desc "Create a Tarot card if played hand is a Straight containing an Ace"
  on independent when contains(hand, Straight) && count(played, Ace) > 0 { add_tarot 1 }
}

joker vagabond "Vagabond" Rare {
  desc "Create a Tarot card if you have $4 or less when scoring"
  on independent when money <= 4 { add_tarot 1 }
}

joker hallucination "Hallucination" Common {
  desc "1 in 2 chance to create a Tarot card when any Booster Pack is opened"
  on pack_opened when roll(2) { add_tarot 1 }
}

joker ramen "Ramen" Uncommon {
  desc "X2 Mult; loses X0.01 Mult for each discarded card; currently X{mult} Mult"
  on independent when var(init) == 0 { set_var mult 2; set_var init 1 }
  on independent { mul_mult var(mult) }
  on discard { set_var mult max(1, var(mult) - 0.01) }
}

joker sock_and_buskin "Sock and Buskin" Uncommon {
  desc "Retrigger all scored face cards once"
  on scored when card.is_face { retrigger_scored 1 }
}

joker astronomer "Astronomer" Uncommon {
  desc "All Planet cards and Celestial Packs in the shop are free"
  on shop_enter { set_shop_price planet 0; set_shop_price celestial_pack 0 }
  on shop_reroll { set_shop_price planet 0; set_shop_price celestial_pack 0 }
}

joker fortune_teller "Fortune Teller" Common {
  desc "Gain +1 Mult per Tarot card used this run; currently +{mult} Mult"
  on use when consumable.kind == Tarot { add_var mult 1 }
  on independent { add_mult var(mult) }
}

joker loyalty_card "Loyalty Card" Uncommon {
  desc "X4 Mult every 6 hands played"
  on played { add_var count 1 }
  on independent when var(count) >= 6 { mul_mult 4; set_var count 0 }
}

joker raised_fist "Raised Fist" Common {
  desc "Adds double the rank of the lowest ranked card held in hand as Mult"
  on independent { add_mult lowest_rank(held) * 2 }
}

joker photograph "Photograph" Common {
  desc "First face card scored each hand gives X2 Mult"
  on played { set_var used 0 }
  on scored when card.is_face && var(used) == 0 { mul_mult 2; set_var used 1 }
}

joker hanging_chad "Hanging Chad" Common {
  desc "Retrigger the first scored card each hand twice"
  on played { set_var used 0 }
  on scored when var(used) == 0 { retrigger_scored 2; set_var used 1 }
}

joker castle "Castle" Uncommon {
  desc "Gain +3 Chips (stacking) for each discarded card of the current random suit; currently +{chips} Chips"
  on blind_start { set_var suit rand(0, 3) }
  on discard when suit_match(var(suit)) { add_var chips 3 }
  on independent { add_chips var(chips) }
}

joker red_card "Red Card" Common {
  desc "Gain +3 Mult (stacking) when any Booster Pack is skipped; currently +{mult} Mult"
  on pack_skipped { add_var mult 3 }
  on independent { add_mult var(mult) }
}

joker riff_raff "Riff-Raff" Common {
  desc "When each Blind is selected, create 2 Common Jokers"
  on blind_start { add_joker common 2 }
}

joker gros_michel "Gros Michel" Common {
  desc "Gain +15 Mult; 1 in 6 chance to be destroyed at the end of each round"
  on independent { add_mult 15 }
  on round_end when roll(6) { destroy_self 1 }
}

joker cavendish "Cavendish" Common {
  desc "X3 Mult; very rarely destroyed at end of round"
  on independent { mul_mult 3 }
  on round_end when roll(1000) { destroy_self 1 }
}

joker madness "Madness" Uncommon {
  desc "Gain X0.5 Mult and destroy a random Joker when Small or Big Blind is selected; currently X{mult} Mult"
  on blind_start when var(init) == 0 { set_var mult 1; set_var init 1 }
  on blind_start when blind == Small || blind == Big { set_var mult (var(mult) + 0.5); destroy_random_joker 1 }
  on independent { mul_mult var(mult) }
}

joker campfire "Campfire" Rare {
  desc "Gain X0.25 Mult for each card sold; resets after Boss Blind; currently X{mult} Mult"
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on any_sell { set_var mult (var(mult) + 0.25) }
  on round_end when is_boss_blind { set_var mult 1 }
  on independent { mul_mult var(mult) }
}

# Additional jokers with partial/engine-backed effects.
joker four_fingers "Four Fingers" Uncommon {
  desc "All Flushes and Straights can be made with 4 cards"
  on passive { set_rule four_fingers 1 }
}

joker credit_card "Credit Card" Common {
  desc "You may go up to -$20 in debt"
  on passive { set_rule money_floor -20 }
}

joker delayed_gratification "Delayed Gratification" Common {
  desc "If no discards are used by end of round, earn $2 per discard"
  on round_end when discards_left == discards_max { add_money discards_left * 2 }
}

joker pareidolia "Pareidolia" Uncommon {
  desc "All cards are considered face cards"
  on passive { set_rule pareidolia 1 }
}

joker egg "Egg" Common {
  desc "Gains $3 of sell value at end of each round"
  on round_end { add_var sell_bonus 3 }
}

joker splash "Splash" Common {
  desc "Every played card counts in scoring"
  on passive { set_rule splash 1 }
}

joker turtle_bean "Turtle Bean" Uncommon {
  desc "Hand size increases by {beans} each round; loses 1 per round"
  on independent when var(init) == 0 { set_var beans 5; set_var init 1 }
  on blind_start when var(beans) > 0 { add_hand_size var(beans) }
  on round_end when var(beans) > 0 { add_var beans -1 }
}

joker to_the_moon "To the Moon" Uncommon {
  desc "Earn $1 for every $5 you have at end of round"
  on round_end { add_money floor(money / 5) }
}

joker juggler "Juggler" Common {
  desc "Gain +1 hand size when each Blind is selected"
  on blind_start { add_hand_size 1 }
}

joker trading_card "Trading Card" Uncommon {
  desc "If first discard of the round is a single card, earn $3"
  on discard when discards_left == discards_max && count(discarded, all) == 1 { add_money 3 }
}

joker troubadour "Troubadour" Uncommon {
  desc "Gain +2 hand size and -1 Hand each round"
  on blind_start { add_hand_size 2; add_hands -1 }
}

joker merry_andy "Merry Andy" Uncommon {
  desc "Gain -1 hand size and +3 Discards each round"
  on blind_start { add_hand_size -1; add_discards 3 }
}

joker stuntman "Stuntman" Rare {
  desc "Gain +250 Chips; hand size is reduced by 2 each round"
  on independent { add_chips 250 }
  on blind_start { add_hand_size -2 }
}

joker hit_the_road "Hit the Road" Rare {
  desc "Gain X0.5 Mult for each Jack discarded; resets at end of round; currently X{mult} Mult"
  on discard when card.rank == Jack { add_var mult 0.5 }
  on round_end { set_var mult 1 }
  on independent when var(mult) == 0 { set_var mult 1 }
  on independent { mul_mult var(mult) }
}

joker canio "Canio" Legendary {
  desc "Gain X1 Mult for each face card destroyed; currently X{mult} Mult"
  on independent when var(mult) == 0 { set_var mult 1 }
  on card_destroyed when card.is_face { add_var mult 1 }
  on independent { mul_mult var(mult) }
}

joker triboulet "Triboulet" Legendary {
  desc "Scored Kings and Queens each give X2 Mult"
  on scored when card.rank == King || card.rank == Queen { mul_mult 2 }
}

joker yorick "Yorick" Legendary {
  desc "Gain X1 Mult for every 23 cards discarded; currently X{mult} Mult"
  on independent when var(mult) == 0 { set_var mult 1 }
  on discard_batch { add_var count count(discarded, all) }
  on discard_batch when var(count) >= 23 { add_var mult 1; add_var count -23 }
  on independent { mul_mult var(mult) }
}

joker rocket "Rocket" Uncommon {
  desc "Earn ${payout} at end of round; payout increases by $2 when Boss Blind is defeated"
  on independent when var(init) == 0 { set_var payout 1; set_var init 1 }
  on round_end { add_money var(payout) }
  on round_end when is_boss_blind { add_var payout 2 }
}

joker mail_in_rebate "Mail-In Rebate" Common {
  desc "Earn $5 for each discarded card of a random rank; rank changes each round"
  on blind_start { set_var rank rand(2, 14) }
  on discard when card.rank_id == var(rank) { add_money 5 }
}

joker to_do_list "To Do List" Common {
  desc "Earn $4 if the played hand matches a random poker hand; changes each round"
  on blind_start when var(init) == 0 { set_var target rand(0, 12); set_var init 1 }
  on round_end { set_var target rand(0, 12) }
  on played when hand_id == var(target) { add_money 4 }
}

joker shortcut "Shortcut" Uncommon {
  desc "Allows Straights to be made with gaps of 1 rank"
  on passive { set_rule shortcut 1 }
}

joker the_idol "The Idol" Uncommon {
  desc "Scored cards that match a random rank and suit give X2 Mult; changes each round"
  on blind_start { set_var suit rand(0, 3); set_var rank rand(2, 14) }
  on scored when card.rank_id == var(rank) && suit_match(var(suit)) { mul_mult 2 }
}

joker obelisk "Obelisk" Rare {
  desc "Gain X0.2 Mult for each consecutive hand played that is not the most played; resets on most played; currently X{mult} Mult"
  on independent when var(mult) == 0 { set_var mult 1 }
  on played when hand != most_played_hand { add_var mult 0.2 }
  on played when hand == most_played_hand { set_var mult 1 }
  on independent { mul_mult var(mult) }
}

joker hiker "Hiker" Uncommon {
  desc "Each scored card permanently gains +5 Chips"
  on scored_pre { add_card_bonus 5 }
}

joker vampire "Vampire" Uncommon {
  desc "Gain X0.1 Mult for each scored enhanced card; removes enhancements; currently X{mult} Mult"
  on scored_pre when card.has_enhancement { clear_card_enhancement; add_var mult 0.1 }
  on independent { mul_mult max(1, var(mult)) }
}

joker midas_mask "Midas Mask" Uncommon {
  desc "All scored face cards become Gold cards"
  on scored_pre when card.is_face { set_card_enhancement Gold }
}

joker dna "DNA" Rare {
  desc "If first hand of round is a single card, add a permanent copy to your deck"
  on blind_start { set_var used 0 }
  on scored_pre when hands_left == hands_max && played_count == 1 && var(used) == 0 { copy_played_card; set_var used 1 }
}

joker sixth_sense "Sixth Sense" Uncommon {
  desc "If first hand of round is a single 6, destroy it and create a Spectral card"
  on blind_start { set_var used 0 }
  on scored_pre when hands_left == hands_max && played_count == 1 && card.rank_id == 6 && consumable_count < consumable_slots && var(used) == 0 { destroy_card; add_spectral 1; set_var used 1 }
}

joker ceremonial_dagger "Ceremonial Dagger" Uncommon {
  desc "Destroys the Joker to the right, permanently gaining double its sell value as Mult; currently +{mult} Mult"
  on blind_start { destroy_joker_right 1; add_var mult last_destroyed_sell_value * 2 }
  on independent { add_mult var(mult) }
}

joker seance "Seance" Uncommon {
  desc "If played hand is a Straight Flush, create a Spectral card"
  on independent when contains(hand, StraightFlush) { add_spectral 1 }
}

joker hologram "Hologram" Uncommon {
  desc "Gain X0.25 Mult for each playing card added to your deck; currently X{mult} Mult"
  on acquire when var(init) == 0 { set_var mult 1; set_var init 1 }
  on card_added { add_var mult 0.25 }
  on independent { mul_mult var(mult) }
}

joker luchador "Luchador" Uncommon {
  desc "Sell this card to disable the current Boss Blind effect"
  on sell { disable_boss }
}

joker gift_card "Gift Card" Uncommon {
  desc "Add $1 of sell value to every Joker and Consumable at end of round"
  on round_end { add_sell_bonus all 1 }
}

joker lucky_cat "Lucky Cat" Uncommon {
  desc "Gain X0.25 Mult each time a Lucky card triggers; currently X{mult} Mult"
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on scored when card.enhancement == Lucky && card.lucky_triggers > 0 { add_var mult card.lucky_triggers * 0.25 }
  on independent { mul_mult var(mult) }
}

joker baseball_card "Baseball Card" Rare {
  desc "Each Uncommon Joker gives X1.5 Mult"
  on independent { mul_mult pow(1.5, count_rarity(Uncommon)) }
}

joker diet_cola "Diet Cola" Uncommon {
  desc "Sell this to create a Double Tag"
  on sell { add_tag double 1 }
}

joker seltzer "Seltzer" Uncommon {
  desc "Retriggers all cards for the next 10 hands played"
  on acquire { set_var remaining 10 }
  on scored when var(remaining) > 0 { retrigger_scored 1 }
  on hand_end when var(remaining) > 0 { add_var remaining -1 }
}

joker mr_bones "Mr. Bones" Uncommon {
  desc "Prevents death if score is at least 25% of required; self destructs"
  on blind_failed when blind_score >= target * 0.25 { prevent_death 1; destroy_self }
}

joker swashbuckler "Swashbuckler" Common {
  desc "Adds the sell value of all other Jokers as Mult"
  on independent { add_mult other_joker_sell_value }
}

joker certificate "Certificate" Uncommon {
  desc "When each Blind is selected, add a random playing card with a random enhancement to your hand"
  on blind_start { add_random_hand_card 1 }
}

joker throwback "Throwback" Uncommon {
  desc "X0.25 Mult for each Blind skipped this run"
  on independent { mul_mult (1 + blinds_skipped * 0.25) }
}

joker showman "Showman" Uncommon {
  desc "Joker, Tarot, Planet, and Spectral cards may appear multiple times in the shop"
  on passive { set_rule shop_allow_duplicates 1 }
}

use copy_right_all(blueprint, "Blueprint", Rare)

joker oops_all_6s "Oops! All 6s" Uncommon {
  desc "Doubles all probabilities (e.g. 1 in 3 -> 2 in 3)"
  on passive { add_rule roll_bonus 1 }
}

joker matador "Matador" Uncommon {
  desc "Earn $8 if the played hand triggers the Boss Blind ability"
  on played when is_boss_blind && !boss_disabled { add_money 8 }
}

use copy_leftmost_all(brainstorm, "Brainstorm", Rare)

joker satellite "Satellite" Uncommon {
  desc "Earn $1 for each unique Planet card used this run at end of round"
  on round_end { add_money unique_planets_used }
}

joker chicot "Chicot" Legendary {
  desc "Disables the Boss Blind effect when entering the Boss Blind"
  on blind_start when is_boss_blind { disable_boss }
}

joker perkeo "Perkeo" Legendary {
  desc "Creates a negative copy of a random consumable in your possession when leaving the shop"
  on shop_exit { duplicate_random_consumable 1 }
}

joker shortcut_joker "Shortcut" Uncommon {
  mixin shortcut_hand
}

joker card_shark "Card Shark" Uncommon {
  mixin free_reroll_on_enter
}

joker restructure "Restructure" Rare {
  mixin boss_reroll_on_sell
}
