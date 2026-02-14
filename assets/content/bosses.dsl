# Boss definitions (DSL).

boss the_hook "The Hook" {
  i18n zh_CN "钩子"
  on passive { set_rule discard_held_after_hand 2 }
}

boss the_ox "The Ox" {
  i18n zh_CN "公牛"
  on played when hand == most_played_hand { set_money 0 }
}

boss the_house "The House" {
  i18n zh_CN "房屋"
  on passive { set_rule draw_face_down_first_hand 1 }
}

boss the_wall "The Wall" {
  i18n zh_CN "高墙"
  on blind_start { mul_target 2 }
}

boss the_wheel "The Wheel" {
  i18n zh_CN "车轮"
  on passive { set_rule draw_face_down_roll 7 }
}

boss the_arm "The Arm" {
  i18n zh_CN "手臂"
  on passive { set_rule hand_level_delta -1 }
}

boss the_club "The Club" {
  i18n zh_CN "梅花"
  on passive { set_rule debuff_suit_clubs 1 }
}

boss the_fish "The Fish" {
  i18n zh_CN "鱼"
  on passive { set_rule draw_face_down_after_hand 1 }
}

boss the_psychic "The Psychic" {
  i18n zh_CN "灵媒"
  on passive { set_rule required_play_count 5 }
}

boss the_goad "The Goad" {
  i18n zh_CN "尖刺"
  on passive { set_rule debuff_suit_spades 1 }
}

boss the_water "The Water" {
  i18n zh_CN "流水"
  on blind_start { set_discards 0 }
}

boss the_window "The Window" {
  i18n zh_CN "窗户"
  on passive { set_rule debuff_suit_diamonds 1 }
}

boss the_manacle "The Manacle" {
  i18n zh_CN "镣铐"
  on blind_start { add_hand_size -1 }
}

boss the_eye "The Eye" {
  i18n zh_CN "眼睛"
  on passive { set_rule no_repeat_hand 1 }
}

boss the_mouth "The Mouth" {
  i18n zh_CN "嘴巴"
  on passive { set_rule single_hand_type 1 }
}

boss the_plant "The Plant" {
  i18n zh_CN "植物"
  on passive { set_rule debuff_face 1 }
}

boss the_serpent "The Serpent" {
  i18n zh_CN "巨蛇"
  on passive { set_rule draw_after_play 3; set_rule draw_after_discard 3 }
}

boss the_pillar "The Pillar" {
  i18n zh_CN "支柱"
  on passive { set_rule debuff_played_ante 1 }
}

boss the_needle "The Needle" {
  i18n zh_CN "针"
  on blind_start { set_hands 1 }
}

boss the_head "The Head" {
  i18n zh_CN "头颅"
  on passive { set_rule debuff_suit_hearts 1 }
}

boss the_tooth "The Tooth" {
  i18n zh_CN "牙齿"
  on played { add_money -played_count }
}

boss the_flint "The Flint" {
  i18n zh_CN "燧石"
  on passive { set_rule base_chips_mult 0.5; set_rule base_mult_mult 0.5 }
}

boss the_mark "The Mark" {
  i18n zh_CN "记号"
  on passive { set_rule draw_face_down_face 1 }
}
