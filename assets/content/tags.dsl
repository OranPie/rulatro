# Tag definitions (DSL).

tag boss_tag "Boss Tag" {
  on blind_start when is_boss_blind { reroll_boss }
}

tag buffoon_tag "Buffoon Tag" {
  on shop_enter { add_pack buffoon_mega 0 }
}

tag charm_tag "Charm Tag" {
  on shop_enter { add_pack arcana_mega 0 }
}

tag coupon_tag "Coupon Tag" {
  on shop_enter { set_shop_price cards 0; set_shop_price packs 0 }
}

tag d6_tag "D6 Tag" {
  on shop_enter { set_reroll_cost 0 }
}

tag double_tag "Double Tag" {
  on acquire { duplicate_next_tag double_tag }
}

tag economy_tag "Economy Tag" {
  on shop_enter { set_money min(money * 2, 40) }
}

tag ethereal_tag "Ethereal Tag" {
  on shop_enter { add_pack spectral_normal 0 }
}

tag foil_tag "Foil Tag" {
  on shop_enter { set_shop_joker_edition foil 0 }
}

tag garbage_tag "Garbage Tag" {
  on shop_enter { add_money unused_discards }
}

tag handy_tag "Handy Tag" {
  on shop_enter { add_money hands_played }
}

tag holographic_tag "Holographic Tag" {
  on shop_enter { set_shop_joker_edition holographic 0 }
}

tag investment_tag "Investment Tag" {
  on round_end when is_boss_blind && blind_score >= target { add_money 25 }
}

tag juggle_tag "Juggle Tag" {
  on blind_start { add_hand_size 3 }
}

tag meteor_tag "Meteor Tag" {
  on shop_enter { add_pack celestial_mega 0 }
}

tag negative_tag "Negative Tag" {
  on shop_enter { set_shop_joker_edition negative 0 }
}

tag orbital_tag "Orbital Tag" {
  on shop_enter { upgrade_random_hand 3 }
}

tag polychrome_tag "Polychrome Tag" {
  on shop_enter { set_shop_joker_edition polychrome 0 }
}

tag rare_tag "Rare Tag" {
  on shop_enter { add_shop_joker rare 0 }
}

tag speed_tag "Speed Tag" {
  on shop_enter { add_money blinds_skipped * 5 }
}

tag standard_tag "Standard Tag" {
  on shop_enter { add_pack standard_mega 0 }
}

tag top_up_tag "Top-up Tag" {
  on shop_enter { add_joker common 2 }
}

tag uncommon_tag "Uncommon Tag" {
  on shop_enter { add_shop_joker uncommon 0 }
}

tag voucher_tag "Voucher Tag" {
  on shop_enter { add_voucher 1 }
}
