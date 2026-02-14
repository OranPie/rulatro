# Tag definitions (DSL).

tag boss_tag "Boss Tag" {
  i18n zh_CN "Boss 标签"
  on blind_start when is_boss_blind { reroll_boss }
}

tag buffoon_tag "Buffoon Tag" {
  i18n zh_CN "小丑包标签"
  on shop_enter { add_pack buffoon_mega 0 }
}

tag charm_tag "Charm Tag" {
  i18n zh_CN "魅力标签"
  on shop_enter { add_pack arcana_mega 0 }
}

tag coupon_tag "Coupon Tag" {
  i18n zh_CN "优惠券标签"
  on shop_enter { set_shop_price cards 0; set_shop_price packs 0 }
}

tag d6_tag "D6 Tag" {
  i18n zh_CN "D6 标签"
  on shop_enter { set_reroll_cost 0 }
}

tag double_tag "Double Tag" {
  i18n zh_CN "双倍标签"
  on acquire { duplicate_next_tag double_tag }
}

tag economy_tag "Economy Tag" {
  i18n zh_CN "经济标签"
  on shop_enter { set_money min(money * 2, 40) }
}

tag ethereal_tag "Ethereal Tag" {
  i18n zh_CN "虚空标签"
  on shop_enter { add_pack spectral_normal 0 }
}

tag foil_tag "Foil Tag" {
  i18n zh_CN "闪箔标签"
  on shop_enter { set_shop_joker_edition foil 0 }
}

tag garbage_tag "Garbage Tag" {
  i18n zh_CN "垃圾标签"
  on shop_enter { add_money unused_discards }
}

tag handy_tag "Handy Tag" {
  i18n zh_CN "手数标签"
  on shop_enter { add_money hands_played }
}

tag holographic_tag "Holographic Tag" {
  i18n zh_CN "全息标签"
  on shop_enter { set_shop_joker_edition holographic 0 }
}

tag investment_tag "Investment Tag" {
  i18n zh_CN "投资标签"
  on round_end when is_boss_blind && blind_score >= target { add_money 25 }
}

tag juggle_tag "Juggle Tag" {
  i18n zh_CN "杂耍标签"
  on blind_start { add_hand_size 3 }
}

tag meteor_tag "Meteor Tag" {
  i18n zh_CN "流星标签"
  on shop_enter { add_pack celestial_mega 0 }
}

tag negative_tag "Negative Tag" {
  i18n zh_CN "负片标签"
  on shop_enter { set_shop_joker_edition negative 0 }
}

tag orbital_tag "Orbital Tag" {
  i18n zh_CN "轨道标签"
  on shop_enter { upgrade_random_hand 3 }
}

tag polychrome_tag "Polychrome Tag" {
  i18n zh_CN "多彩标签"
  on shop_enter { set_shop_joker_edition polychrome 0 }
}

tag rare_tag "Rare Tag" {
  i18n zh_CN "稀有标签"
  on shop_enter { add_shop_joker rare 0 }
}

tag speed_tag "Speed Tag" {
  i18n zh_CN "速度标签"
  on shop_enter { add_money blinds_skipped * 5 }
}

tag standard_tag "Standard Tag" {
  i18n zh_CN "标准标签"
  on shop_enter { add_pack standard_mega 0 }
}

tag top_up_tag "Top-up Tag" {
  i18n zh_CN "补充标签"
  on shop_enter { add_joker common 2 }
}

tag uncommon_tag "Uncommon Tag" {
  i18n zh_CN "非凡标签"
  on shop_enter { add_shop_joker uncommon 0 }
}

tag voucher_tag "Voucher Tag" {
  i18n zh_CN "代金券标签"
  on shop_enter { add_voucher 1 }
}
