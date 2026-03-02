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
  on independent when played_count <= 3 { add_mult 20 }
}

joker banner "Banner" Common {
  on independent { add_chips discards_left * 30 }
}

joker mystic_summit "Mystic Summit" Common {
  on independent when discards_left == 0 { add_mult 15 }
}

use scored_face_chips(scary_face, "Scary Face", Common, 30)
use scored_face_mult(smiley_face, "Smiley Face", Common, 5)

joker abstract_joker "Abstract Joker" Common {
  on independent { add_mult joker_count * 3 }
}

joker even_steven "Even Steven" Common {
  on scored when card.is_even { add_mult 4 }
}

joker odd_todd "Odd Todd" Common {
  on scored when card.is_odd { add_chips 31 }
}

joker scholar "Scholar" Common {
  on scored when card.rank == Ace { add_chips 20; add_mult 4 }
}

joker fibonacci "Fibonacci" Uncommon {
  on scored when card.rank == Ace || card.rank == Two || card.rank == Three || card.rank == Five || card.rank == Eight { add_mult 8 }
}

joker mime "Mime" Uncommon {
  on held { retrigger_held 1 }
}

joker dusk "Dusk" Uncommon {
  on scored when hands_left == 1 { retrigger_scored 1 }
}

joker hack "Hack" Uncommon {
  on scored when card.rank == Two || card.rank == Three || card.rank == Four || card.rank == Five { retrigger_scored 1 }
}

joker business_card "Business Card" Common {
  on scored when card.is_face && roll(2) { add_money 2 }
}

joker reserved_parking "Reserved Parking" Common {
  on held when card.is_face && roll(2) { add_money 1 }
}

joker shoot_the_moon "Shoot the Moon" Common {
  on held when card.rank == Queen { add_mult 13 }
}

joker misprint "Misprint" Common {
  on independent { add_mult rand(0, 23) }
}

joker ice_cream "Ice Cream" Common {
  on independent when var(init) == 0 { set_var chips 100; set_var init 1 }
  on independent { add_chips var(chips) }
  on independent { set_var chips max(0, var(chips) - 5) }
}

joker popcorn "Popcorn" Common {
  on independent when var(init) == 0 { set_var mult 20; set_var init 1 }
  on independent { add_mult var(mult) }
  on round_end { set_var mult max(0, var(mult) - 4) }
}

joker blue_joker "Blue Joker" Common {
  on independent { add_chips deck_count * 2 }
}

joker golden_joker "Golden Joker" Common {
  on round_end { add_money 4 }
}

joker square_joker "Square Joker" Common {
  on played when played_count == 4 { add_var chips 4 }
  on independent { add_chips var(chips) }
}

joker faceless_joker "Faceless Joker" Common {
  on discard_batch when count(discarded, face) >= 3 { add_money 5 }
}

joker marble_joker "Marble Joker" Uncommon {
  on blind_start { add_stone_card 1 }
}

joker steel_joker "Steel Joker" Uncommon {
  on independent { mul_mult (1 + count(deck, steel) * 0.2) }
}

joker stone_joker "Stone Joker" Uncommon {
  on independent { add_chips count(deck, stone) * 25 }
}

joker wee_joker "Wee Joker" Rare {
  on scored when card.rank == Two { add_var chips 8 }
  on independent { add_chips var(chips) }
}

joker supernova "Supernova" Common {
  on independent { add_mult hand_play_count }
}

joker green_joker "Green Joker" Common {
  on played { add_var mult 1 }
  on discard { set_var mult max(0, var(mult) - 1) }
  on independent { add_mult var(mult) }
}

joker ride_the_bus "Ride the Bus" Common {
  on played when count(scoring, face) == 0 { add_var mult 1 }
  on scored when card.is_face { set_var mult 0 }
  on independent { add_mult var(mult) }
}

joker joker_stencil "Joker Stencil" Uncommon {
  on independent { mul_mult (empty_joker_slots + count_joker("joker_stencil")) }
}

joker blackboard "Blackboard" Uncommon {
  on independent when count(held, black) == held_count { mul_mult 3 }
}

joker ancient_joker "Ancient Joker" Rare {
  on independent when var(init) == 0 { set_var suit rand(0, 3); set_var init 1 }
  on scored when suit_match(var(suit)) { mul_mult 1.5 }
  on round_end { set_var suit rand(0, 3) }
}

joker burnt_joker "Burnt Joker" Rare {
  on blind_start { set_var used 0 }
  on discard_batch when var(used) == 0 { upgrade_hand hand; set_var used 1 }
}

joker space_joker "Space Joker" Uncommon {
  on played when roll(4) { upgrade_hand hand }
}

joker glass_joker "Glass Joker" Uncommon {
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on destroyed when var(init) == 0 { set_var mult 1; set_var init 1 }
  on destroyed when card.enhancement == Glass { set_var mult (var(mult) + 0.75) }
  on independent { mul_mult var(mult) }
}

joker invisible_joker "Invisible Joker" Rare {
  on round_end { add_var rounds 1 }
  on sell when var(rounds) >= 2 { duplicate_random_joker 1 }
}

joker smeared_joker "Smeared Joker" Uncommon {
  on passive { set_rule smeared_suits 1 }
}

joker eight_ball "8 Ball" Common {
  on scored when card.rank == Eight && roll(4) { add_tarot 1 }
}

joker acrobat "Acrobat" Uncommon {
  on independent when hands_left == 1 { mul_mult 3 }
}

use scored_suit_chips(arrowhead, "Arrowhead", Uncommon, Spades, 50)

joker baron "Baron" Rare {
  on held when card.rank == King { mul_mult 1.5 }
}

joker bloodstone "Bloodstone" Uncommon {
  on scored when suit_match(Hearts) && roll(2) { mul_mult 1.5 }
}

joker bootstraps "Bootstraps" Uncommon {
  on independent { add_mult floor(money / 5) * 2 }
}

joker bull "Bull" Uncommon {
  on independent { add_chips money * 2 }
}

joker burglar "Burglar" Uncommon {
  on blind_start { add_hands 3; set_discards 0 }
}

joker cartomancer "Cartomancer" Uncommon {
  on blind_start { add_tarot 1 }
}

joker cloud_9 "Cloud 9" Uncommon {
  on round_end { add_money count(deck, 9) }
}

joker drivers_license "Driver's License" Rare {
  on independent when count(deck, enhanced) >= 16 { mul_mult 3 }
}

joker erosion "Erosion" Uncommon {
  on independent { add_mult max(0, 52 - deck_count) * 4 }
}

joker constellation "Constellation" Uncommon {
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on use when consumable.kind == Planet { set_var mult (var(mult) + 0.1) }
  on independent { mul_mult var(mult) }
}

use scored_suit_mult(onyx_agate, "Onyx Agate", Uncommon, Clubs, 7)

joker runner "Runner" Common {
  on played when contains(hand, Straight) { add_var chips 15 }
  on independent { add_chips var(chips) }
}

joker spare_trousers "Spare Trousers" Uncommon {
  on played when contains(hand, TwoPair) { add_var mult 2 }
  on independent { add_mult var(mult) }
}

use indep_mul_mult_contains(the_duo, "The Duo", Rare, Pair, 2)
use indep_mul_mult_contains(the_trio, "The Trio", Rare, ThreeOfAKind, 3)
use indep_mul_mult_contains(the_family, "The Family", Rare, Quads, 4)
use indep_mul_mult_contains(the_order, "The Order", Rare, Straight, 3)
use indep_mul_mult_contains(the_tribe, "The Tribe", Rare, Flush, 2)

joker chaos_the_clown "Chaos the Clown" Common {
  on shop_enter { add_free_reroll 1 }
}

joker drunkard "Drunkard" Common {
  on blind_start { add_discards 1 }
}

joker flash_card "Flash Card" Uncommon {
  on shop_reroll { add_var mult 2 }
  on independent { add_mult var(mult) }
}

joker card_sharp "Card Sharp" Uncommon {
  on independent when hand_play_count > 1 { mul_mult 3 }
}

joker walkie_talkie "Walkie Talkie" Common {
  on scored when card.rank == Ten || card.rank == Four { add_chips 10; add_mult 4 }
}

joker rough_gem "Rough Gem" Uncommon {
  on scored when suit_match(Diamonds) { add_money 1 }
}

joker golden_ticket "Golden Ticket" Common {
  on scored when card.enhancement == Gold { add_money 4 }
}

joker flower_pot "Flower Pot" Uncommon {
  on independent when count(played, Diamonds) > 0 && count(played, Clubs) > 0 && count(played, Hearts) > 0 && count(played, Spades) > 0 { mul_mult 3 }
}

joker seeing_double "Seeing Double" Uncommon {
  on independent when count(scoring, Clubs) > 0 && (count(scoring, Diamonds) + count(scoring, Hearts) + count(scoring, Spades)) > 0 { mul_mult 2 }
}

joker superposition "Superposition" Common {
  on independent when contains(hand, Straight) && count(played, Ace) > 0 { add_tarot 1 }
}

joker vagabond "Vagabond" Rare {
  on independent when money <= 4 { add_tarot 1 }
}

joker hallucination "Hallucination" Common {
  on pack_opened when roll(2) { add_tarot 1 }
}

joker ramen "Ramen" Uncommon {
  on independent when var(init) == 0 { set_var mult 2; set_var init 1 }
  on independent { mul_mult var(mult) }
  on discard { set_var mult max(1, var(mult) - 0.01) }
}

joker sock_and_buskin "Sock and Buskin" Uncommon {
  on scored when card.is_face { retrigger_scored 1 }
}

joker astronomer "Astronomer" Uncommon {
  on shop_enter { set_shop_price planet 0; set_shop_price celestial_pack 0 }
  on shop_reroll { set_shop_price planet 0; set_shop_price celestial_pack 0 }
}

joker fortune_teller "Fortune Teller" Common {
  on use when consumable.kind == Tarot { add_var mult 1 }
  on independent { add_mult var(mult) }
}

joker loyalty_card "Loyalty Card" Uncommon {
  on played { add_var count 1 }
  on independent when var(count) >= 6 { mul_mult 4; set_var count 0 }
}

joker raised_fist "Raised Fist" Common {
  on independent { add_mult lowest_rank(held) * 2 }
}

joker photograph "Photograph" Common {
  on played { set_var used 0 }
  on scored when card.is_face && var(used) == 0 { mul_mult 2; set_var used 1 }
}

joker hanging_chad "Hanging Chad" Common {
  on played { set_var used 0 }
  on scored when var(used) == 0 { retrigger_scored 2; set_var used 1 }
}

joker castle "Castle" Uncommon {
  on blind_start { set_var suit rand(0, 3) }
  on discard when suit_match(var(suit)) { add_var chips 3 }
  on independent { add_chips var(chips) }
}

joker red_card "Red Card" Common {
  on pack_skipped { add_var mult 3 }
  on independent { add_mult var(mult) }
}

joker riff_raff "Riff-Raff" Common {
  on blind_start { add_joker common 2 }
}

joker gros_michel "Gros Michel" Common {
  on independent { add_mult 15 }
  on round_end when roll(6) { destroy_self 1 }
}

joker cavendish "Cavendish" Common {
  on independent { mul_mult 3 }
  on round_end when roll(1000) { destroy_self 1 }
}

joker madness "Madness" Uncommon {
  on blind_start when var(init) == 0 { set_var mult 1; set_var init 1 }
  on blind_start when blind == Small || blind == Big { set_var mult (var(mult) + 0.5); destroy_random_joker 1 }
  on independent { mul_mult var(mult) }
}

joker campfire "Campfire" Rare {
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on any_sell { set_var mult (var(mult) + 0.25) }
  on round_end when is_boss_blind { set_var mult 1 }
  on independent { mul_mult var(mult) }
}

# Additional jokers with partial/engine-backed effects.
joker four_fingers "Four Fingers" Uncommon {
  on passive { set_rule four_fingers 1 }
}

joker credit_card "Credit Card" Common {
  on passive { set_rule money_floor -20 }
}

joker delayed_gratification "Delayed Gratification" Common {
  on round_end when discards_left == discards_max { add_money discards_left * 2 }
}

joker pareidolia "Pareidolia" Uncommon {
  on passive { set_rule pareidolia 1 }
}

joker egg "Egg" Common {
  on round_end { add_var sell_bonus 3 }
}

joker splash "Splash" Common {
  on passive { set_rule splash 1 }
}

joker turtle_bean "Turtle Bean" Uncommon {
  on independent when var(init) == 0 { set_var beans 5; set_var init 1 }
  on blind_start when var(beans) > 0 { add_hand_size var(beans) }
  on round_end when var(beans) > 0 { add_var beans -1 }
}

joker to_the_moon "To the Moon" Uncommon {
  on round_end { add_money floor(money / 5) }
}

joker juggler "Juggler" Common {
  on blind_start { add_hand_size 1 }
}

joker trading_card "Trading Card" Uncommon {
  on discard when discards_left == discards_max && count(discarded, all) == 1 { add_money 3 }
}

joker troubadour "Troubadour" Uncommon {
  on blind_start { add_hand_size 2; add_hands -1 }
}

joker merry_andy "Merry Andy" Uncommon {
  on blind_start { add_hand_size -1; add_discards 3 }
}

joker stuntman "Stuntman" Rare {
  on independent { add_chips 250 }
  on blind_start { add_hand_size -2 }
}

joker hit_the_road "Hit the Road" Rare {
  on discard when card.rank == Jack { add_var mult 0.5 }
  on round_end { set_var mult 1 }
  on independent when var(mult) == 0 { set_var mult 1 }
  on independent { mul_mult var(mult) }
}

joker canio "Canio" Legendary {
  on independent when var(mult) == 0 { set_var mult 1 }
  on card_destroyed when card.is_face { add_var mult 1 }
  on independent { mul_mult var(mult) }
}

joker triboulet "Triboulet" Legendary {
  on scored when card.rank == King || card.rank == Queen { mul_mult 2 }
}

joker yorick "Yorick" Legendary {
  on independent when var(mult) == 0 { set_var mult 1 }
  on discard_batch { add_var count count(discarded, all) }
  on discard_batch when var(count) >= 23 { add_var mult 1; add_var count -23 }
  on independent { mul_mult var(mult) }
}

joker rocket "Rocket" Uncommon {
  on independent when var(init) == 0 { set_var payout 1; set_var init 1 }
  on round_end { add_money var(payout) }
  on round_end when is_boss_blind { add_var payout 2 }
}

joker mail_in_rebate "Mail-In Rebate" Common {
  on blind_start { set_var rank rand(2, 14) }
  on discard when card.rank_id == var(rank) { add_money 5 }
}

joker to_do_list "To Do List" Common {
  on blind_start when var(init) == 0 { set_var target rand(0, 12); set_var init 1 }
  on round_end { set_var target rand(0, 12) }
  on played when hand_id == var(target) { add_money 4 }
}

joker shortcut "Shortcut" Uncommon {
  on passive { set_rule shortcut 1 }
}

joker the_idol "The Idol" Uncommon {
  on blind_start { set_var suit rand(0, 3); set_var rank rand(2, 14) }
  on scored when card.rank_id == var(rank) && suit_match(var(suit)) { mul_mult 2 }
}

joker obelisk "Obelisk" Rare {
  on independent when var(mult) == 0 { set_var mult 1 }
  on played when hand != most_played_hand { add_var mult 0.2 }
  on played when hand == most_played_hand { set_var mult 1 }
  on independent { mul_mult var(mult) }
}

joker hiker "Hiker" Uncommon {
  on scored_pre { add_card_bonus 5 }
}

joker vampire "Vampire" Uncommon {
  on scored_pre when card.has_enhancement { clear_card_enhancement; add_var mult 0.1 }
  on independent { mul_mult max(1, var(mult)) }
}

joker midas_mask "Midas Mask" Uncommon {
  on scored_pre when card.is_face { set_card_enhancement Gold }
}

joker dna "DNA" Rare {
  on blind_start { set_var used 0 }
  on scored_pre when hands_left == hands_max && played_count == 1 && var(used) == 0 { copy_played_card; set_var used 1 }
}

joker sixth_sense "Sixth Sense" Uncommon {
  on blind_start { set_var used 0 }
  on scored_pre when hands_left == hands_max && played_count == 1 && card.rank_id == 6 && consumable_count < consumable_slots && var(used) == 0 { destroy_card; add_spectral 1; set_var used 1 }
}

joker ceremonial_dagger "Ceremonial Dagger" Uncommon {
  on blind_start { destroy_joker_right 1; add_var mult last_destroyed_sell_value * 2 }
  on independent { add_mult var(mult) }
}

joker seance "Seance" Uncommon {
  on independent when contains(hand, StraightFlush) { add_spectral 1 }
}

joker hologram "Hologram" Uncommon {
  on acquire when var(init) == 0 { set_var mult 1; set_var init 1 }
  on card_added { add_var mult 0.25 }
  on independent { mul_mult var(mult) }
}

joker luchador "Luchador" Uncommon {
  on sell { disable_boss }
}

joker gift_card "Gift Card" Uncommon {
  on round_end { add_sell_bonus all 1 }
}

joker lucky_cat "Lucky Cat" Uncommon {
  on independent when var(init) == 0 { set_var mult 1; set_var init 1 }
  on scored when card.enhancement == Lucky && card.lucky_triggers > 0 { add_var mult card.lucky_triggers * 0.25 }
  on independent { mul_mult var(mult) }
}

joker baseball_card "Baseball Card" Rare {
  on independent { mul_mult pow(1.5, count_rarity(Uncommon)) }
}

joker diet_cola "Diet Cola" Uncommon {
  on sell { add_tag double 1 }
}

joker seltzer "Seltzer" Uncommon {
  on acquire { set_var remaining 10 }
  on scored when var(remaining) > 0 { retrigger_scored 1 }
  on hand_end when var(remaining) > 0 { add_var remaining -1 }
}

joker mr_bones "Mr. Bones" Uncommon {
  on blind_failed when blind_score >= target * 0.25 { prevent_death 1; destroy_self }
}

joker swashbuckler "Swashbuckler" Common {
  on independent { add_mult other_joker_sell_value }
}

joker certificate "Certificate" Uncommon {
  on blind_start { add_random_hand_card 1 }
}

joker throwback "Throwback" Uncommon {
  on independent { mul_mult (1 + blinds_skipped * 0.25) }
}

joker showman "Showman" Uncommon {
  on passive { set_rule shop_allow_duplicates 1 }
}

use copy_right_all(blueprint, "Blueprint", Rare)

joker oops_all_6s "Oops! All 6s" Uncommon {
  on passive { add_rule roll_bonus 1 }
}

joker matador "Matador" Uncommon {
  on played when is_boss_blind && !boss_disabled { add_money 8 }
}

use copy_leftmost_all(brainstorm, "Brainstorm", Rare)

joker satellite "Satellite" Uncommon {
  on round_end { add_money unique_planets_used }
}

joker chicot "Chicot" Legendary {
  on blind_start when is_boss_blind { disable_boss }
}

joker perkeo "Perkeo" Legendary {
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
