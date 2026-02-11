# Boss definitions (DSL).

boss the_hook "The Hook" {
  on passive { set_rule discard_held_after_hand 2 }
}

boss the_ox "The Ox" {
  on played when hand == most_played_hand { set_money 0 }
}

boss the_house "The House" {
  on passive { set_rule draw_face_down_first_hand 1 }
}

boss the_wall "The Wall" {
  on blind_start { mul_target 2 }
}

boss the_wheel "The Wheel" {
  on passive { set_rule draw_face_down_roll 7 }
}

boss the_arm "The Arm" {
  on passive { set_rule hand_level_delta -1 }
}

boss the_club "The Club" {
  on passive { set_rule debuff_suit_clubs 1 }
}

boss the_fish "The Fish" {
  on passive { set_rule draw_face_down_after_hand 1 }
}

boss the_psychic "The Psychic" {
  on passive { set_rule required_play_count 5 }
}

boss the_goad "The Goad" {
  on passive { set_rule debuff_suit_spades 1 }
}

boss the_water "The Water" {
  on blind_start { set_discards 0 }
}

boss the_window "The Window" {
  on passive { set_rule debuff_suit_diamonds 1 }
}

boss the_manacle "The Manacle" {
  on blind_start { add_hand_size -1 }
}

boss the_eye "The Eye" {
  on passive { set_rule no_repeat_hand 1 }
}

boss the_mouth "The Mouth" {
  on passive { set_rule single_hand_type 1 }
}

boss the_plant "The Plant" {
  on passive { set_rule debuff_face 1 }
}

boss the_serpent "The Serpent" {
  on passive { set_rule draw_after_play 3; set_rule draw_after_discard 3 }
}

boss the_pillar "The Pillar" {
  on passive { set_rule debuff_played_ante 1 }
}

boss the_needle "The Needle" {
  on blind_start { set_hands 1 }
}

boss the_head "The Head" {
  on passive { set_rule debuff_suit_hearts 1 }
}

boss the_tooth "The Tooth" {
  on played { add_money -played_count }
}

boss the_flint "The Flint" {
  on passive { set_rule base_chips_mult 0.5; set_rule base_mult_mult 0.5 }
}

boss the_mark "The Mark" {
  on passive { set_rule draw_face_down_face 1 }
}
