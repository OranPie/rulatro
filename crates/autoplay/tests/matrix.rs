use rulatro_autoplay::{
    target_reached, weighted_score, AutoAction, EvalMetrics, ObjectiveWeights, TargetConfig,
};

macro_rules! target_case {
    ($name:ident, $ante:expr, $money:expr, $score:expr, $t_ante:expr, $t_money:expr, $t_score:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let metrics = EvalMetrics {
                ante: $ante,
                money: $money,
                blind_score: $score,
                blind_target: 1000,
                blind_failed: false,
                blind_cleared: false,
            };
            let target = TargetConfig {
                target_score: $t_score,
                target_ante: $t_ante,
                target_money: $t_money,
                stop_on_blind_failed: true,
            };
            assert_eq!(target_reached(metrics, target), $expected);
        }
    };
}

target_case!(target_case_0, 1, 0, 0, Some(1), Some(0), Some(0), true);
target_case!(target_case_1, 1, 0, 0, Some(2), Some(0), Some(0), false);
target_case!(target_case_2, 1, 0, 200, Some(1), Some(0), Some(200), true);
target_case!(target_case_3, 1, 0, 200, Some(2), Some(0), Some(200), false);
target_case!(target_case_4, 1, 0, 500, Some(1), Some(0), Some(500), true);
target_case!(target_case_5, 1, 0, 500, Some(2), Some(0), Some(500), false);
target_case!(
    target_case_6,
    1,
    0,
    1000,
    Some(1),
    Some(0),
    Some(1000),
    true
);
target_case!(
    target_case_7,
    1,
    0,
    1000,
    Some(2),
    Some(0),
    Some(1000),
    false
);
target_case!(target_case_8, 1, 10, 0, Some(1), Some(10), Some(0), true);
target_case!(target_case_9, 1, 10, 0, Some(2), Some(10), Some(0), false);
target_case!(
    target_case_10,
    1,
    10,
    200,
    Some(1),
    Some(10),
    Some(200),
    true
);
target_case!(
    target_case_11,
    1,
    10,
    200,
    Some(2),
    Some(10),
    Some(200),
    false
);
target_case!(
    target_case_12,
    1,
    10,
    500,
    Some(1),
    Some(10),
    Some(500),
    true
);
target_case!(
    target_case_13,
    1,
    10,
    500,
    Some(2),
    Some(10),
    Some(500),
    false
);
target_case!(
    target_case_14,
    1,
    10,
    1000,
    Some(1),
    Some(10),
    Some(1000),
    true
);
target_case!(
    target_case_15,
    1,
    10,
    1000,
    Some(2),
    Some(10),
    Some(1000),
    false
);
target_case!(target_case_16, 1, 30, 0, Some(1), Some(30), Some(0), true);
target_case!(target_case_17, 1, 30, 0, Some(2), Some(30), Some(0), false);
target_case!(
    target_case_18,
    1,
    30,
    200,
    Some(1),
    Some(30),
    Some(200),
    true
);
target_case!(
    target_case_19,
    1,
    30,
    200,
    Some(2),
    Some(30),
    Some(200),
    false
);
target_case!(
    target_case_20,
    1,
    30,
    500,
    Some(1),
    Some(30),
    Some(500),
    true
);
target_case!(
    target_case_21,
    1,
    30,
    500,
    Some(2),
    Some(30),
    Some(500),
    false
);
target_case!(
    target_case_22,
    1,
    30,
    1000,
    Some(1),
    Some(30),
    Some(1000),
    true
);
target_case!(
    target_case_23,
    1,
    30,
    1000,
    Some(2),
    Some(30),
    Some(1000),
    false
);
target_case!(target_case_24, 1, 60, 0, Some(1), Some(60), Some(0), true);
target_case!(target_case_25, 1, 60, 0, Some(2), Some(60), Some(0), false);
target_case!(
    target_case_26,
    1,
    60,
    200,
    Some(1),
    Some(60),
    Some(200),
    true
);
target_case!(
    target_case_27,
    1,
    60,
    200,
    Some(2),
    Some(60),
    Some(200),
    false
);
target_case!(
    target_case_28,
    1,
    60,
    500,
    Some(1),
    Some(60),
    Some(500),
    true
);
target_case!(
    target_case_29,
    1,
    60,
    500,
    Some(2),
    Some(60),
    Some(500),
    false
);
target_case!(
    target_case_30,
    1,
    60,
    1000,
    Some(1),
    Some(60),
    Some(1000),
    true
);
target_case!(
    target_case_31,
    1,
    60,
    1000,
    Some(2),
    Some(60),
    Some(1000),
    false
);
target_case!(target_case_32, 1, 100, 0, Some(1), Some(100), Some(0), true);
target_case!(
    target_case_33,
    1,
    100,
    0,
    Some(2),
    Some(100),
    Some(0),
    false
);
target_case!(
    target_case_34,
    1,
    100,
    200,
    Some(1),
    Some(100),
    Some(200),
    true
);
target_case!(
    target_case_35,
    1,
    100,
    200,
    Some(2),
    Some(100),
    Some(200),
    false
);
target_case!(
    target_case_36,
    1,
    100,
    500,
    Some(1),
    Some(100),
    Some(500),
    true
);
target_case!(
    target_case_37,
    1,
    100,
    500,
    Some(2),
    Some(100),
    Some(500),
    false
);
target_case!(
    target_case_38,
    1,
    100,
    1000,
    Some(1),
    Some(100),
    Some(1000),
    true
);
target_case!(
    target_case_39,
    1,
    100,
    1000,
    Some(2),
    Some(100),
    Some(1000),
    false
);
target_case!(target_case_40, 2, 0, 0, Some(2), Some(0), Some(0), true);
target_case!(target_case_41, 2, 0, 0, Some(3), Some(0), Some(0), false);
target_case!(target_case_42, 2, 0, 200, Some(2), Some(0), Some(200), true);
target_case!(
    target_case_43,
    2,
    0,
    200,
    Some(3),
    Some(0),
    Some(200),
    false
);
target_case!(target_case_44, 2, 0, 500, Some(2), Some(0), Some(500), true);
target_case!(
    target_case_45,
    2,
    0,
    500,
    Some(3),
    Some(0),
    Some(500),
    false
);
target_case!(
    target_case_46,
    2,
    0,
    1000,
    Some(2),
    Some(0),
    Some(1000),
    true
);
target_case!(
    target_case_47,
    2,
    0,
    1000,
    Some(3),
    Some(0),
    Some(1000),
    false
);
target_case!(target_case_48, 2, 10, 0, Some(2), Some(10), Some(0), true);
target_case!(target_case_49, 2, 10, 0, Some(3), Some(10), Some(0), false);
target_case!(
    target_case_50,
    2,
    10,
    200,
    Some(2),
    Some(10),
    Some(200),
    true
);
target_case!(
    target_case_51,
    2,
    10,
    200,
    Some(3),
    Some(10),
    Some(200),
    false
);
target_case!(
    target_case_52,
    2,
    10,
    500,
    Some(2),
    Some(10),
    Some(500),
    true
);
target_case!(
    target_case_53,
    2,
    10,
    500,
    Some(3),
    Some(10),
    Some(500),
    false
);
target_case!(
    target_case_54,
    2,
    10,
    1000,
    Some(2),
    Some(10),
    Some(1000),
    true
);
target_case!(
    target_case_55,
    2,
    10,
    1000,
    Some(3),
    Some(10),
    Some(1000),
    false
);
target_case!(target_case_56, 2, 30, 0, Some(2), Some(30), Some(0), true);
target_case!(target_case_57, 2, 30, 0, Some(3), Some(30), Some(0), false);
target_case!(
    target_case_58,
    2,
    30,
    200,
    Some(2),
    Some(30),
    Some(200),
    true
);
target_case!(
    target_case_59,
    2,
    30,
    200,
    Some(3),
    Some(30),
    Some(200),
    false
);
target_case!(
    target_case_60,
    2,
    30,
    500,
    Some(2),
    Some(30),
    Some(500),
    true
);
target_case!(
    target_case_61,
    2,
    30,
    500,
    Some(3),
    Some(30),
    Some(500),
    false
);
target_case!(
    target_case_62,
    2,
    30,
    1000,
    Some(2),
    Some(30),
    Some(1000),
    true
);
target_case!(
    target_case_63,
    2,
    30,
    1000,
    Some(3),
    Some(30),
    Some(1000),
    false
);
target_case!(target_case_64, 2, 60, 0, Some(2), Some(60), Some(0), true);
target_case!(target_case_65, 2, 60, 0, Some(3), Some(60), Some(0), false);
target_case!(
    target_case_66,
    2,
    60,
    200,
    Some(2),
    Some(60),
    Some(200),
    true
);
target_case!(
    target_case_67,
    2,
    60,
    200,
    Some(3),
    Some(60),
    Some(200),
    false
);
target_case!(
    target_case_68,
    2,
    60,
    500,
    Some(2),
    Some(60),
    Some(500),
    true
);
target_case!(
    target_case_69,
    2,
    60,
    500,
    Some(3),
    Some(60),
    Some(500),
    false
);
target_case!(
    target_case_70,
    2,
    60,
    1000,
    Some(2),
    Some(60),
    Some(1000),
    true
);
target_case!(
    target_case_71,
    2,
    60,
    1000,
    Some(3),
    Some(60),
    Some(1000),
    false
);
target_case!(target_case_72, 2, 100, 0, Some(2), Some(100), Some(0), true);
target_case!(
    target_case_73,
    2,
    100,
    0,
    Some(3),
    Some(100),
    Some(0),
    false
);
target_case!(
    target_case_74,
    2,
    100,
    200,
    Some(2),
    Some(100),
    Some(200),
    true
);
target_case!(
    target_case_75,
    2,
    100,
    200,
    Some(3),
    Some(100),
    Some(200),
    false
);
target_case!(
    target_case_76,
    2,
    100,
    500,
    Some(2),
    Some(100),
    Some(500),
    true
);
target_case!(
    target_case_77,
    2,
    100,
    500,
    Some(3),
    Some(100),
    Some(500),
    false
);
target_case!(
    target_case_78,
    2,
    100,
    1000,
    Some(2),
    Some(100),
    Some(1000),
    true
);
target_case!(
    target_case_79,
    2,
    100,
    1000,
    Some(3),
    Some(100),
    Some(1000),
    false
);
target_case!(target_case_80, 3, 0, 0, Some(3), Some(0), Some(0), true);
target_case!(target_case_81, 3, 0, 0, Some(4), Some(0), Some(0), false);
target_case!(target_case_82, 3, 0, 200, Some(3), Some(0), Some(200), true);
target_case!(
    target_case_83,
    3,
    0,
    200,
    Some(4),
    Some(0),
    Some(200),
    false
);
target_case!(target_case_84, 3, 0, 500, Some(3), Some(0), Some(500), true);
target_case!(
    target_case_85,
    3,
    0,
    500,
    Some(4),
    Some(0),
    Some(500),
    false
);
target_case!(
    target_case_86,
    3,
    0,
    1000,
    Some(3),
    Some(0),
    Some(1000),
    true
);
target_case!(
    target_case_87,
    3,
    0,
    1000,
    Some(4),
    Some(0),
    Some(1000),
    false
);
target_case!(target_case_88, 3, 10, 0, Some(3), Some(10), Some(0), true);
target_case!(target_case_89, 3, 10, 0, Some(4), Some(10), Some(0), false);
target_case!(
    target_case_90,
    3,
    10,
    200,
    Some(3),
    Some(10),
    Some(200),
    true
);
target_case!(
    target_case_91,
    3,
    10,
    200,
    Some(4),
    Some(10),
    Some(200),
    false
);
target_case!(
    target_case_92,
    3,
    10,
    500,
    Some(3),
    Some(10),
    Some(500),
    true
);
target_case!(
    target_case_93,
    3,
    10,
    500,
    Some(4),
    Some(10),
    Some(500),
    false
);
target_case!(
    target_case_94,
    3,
    10,
    1000,
    Some(3),
    Some(10),
    Some(1000),
    true
);
target_case!(
    target_case_95,
    3,
    10,
    1000,
    Some(4),
    Some(10),
    Some(1000),
    false
);
target_case!(target_case_96, 3, 30, 0, Some(3), Some(30), Some(0), true);
target_case!(target_case_97, 3, 30, 0, Some(4), Some(30), Some(0), false);
target_case!(
    target_case_98,
    3,
    30,
    200,
    Some(3),
    Some(30),
    Some(200),
    true
);
target_case!(
    target_case_99,
    3,
    30,
    200,
    Some(4),
    Some(30),
    Some(200),
    false
);
target_case!(
    target_case_100,
    3,
    30,
    500,
    Some(3),
    Some(30),
    Some(500),
    true
);
target_case!(
    target_case_101,
    3,
    30,
    500,
    Some(4),
    Some(30),
    Some(500),
    false
);
target_case!(
    target_case_102,
    3,
    30,
    1000,
    Some(3),
    Some(30),
    Some(1000),
    true
);
target_case!(
    target_case_103,
    3,
    30,
    1000,
    Some(4),
    Some(30),
    Some(1000),
    false
);
target_case!(target_case_104, 3, 60, 0, Some(3), Some(60), Some(0), true);
target_case!(target_case_105, 3, 60, 0, Some(4), Some(60), Some(0), false);
target_case!(
    target_case_106,
    3,
    60,
    200,
    Some(3),
    Some(60),
    Some(200),
    true
);
target_case!(
    target_case_107,
    3,
    60,
    200,
    Some(4),
    Some(60),
    Some(200),
    false
);
target_case!(
    target_case_108,
    3,
    60,
    500,
    Some(3),
    Some(60),
    Some(500),
    true
);
target_case!(
    target_case_109,
    3,
    60,
    500,
    Some(4),
    Some(60),
    Some(500),
    false
);
target_case!(
    target_case_110,
    3,
    60,
    1000,
    Some(3),
    Some(60),
    Some(1000),
    true
);
target_case!(
    target_case_111,
    3,
    60,
    1000,
    Some(4),
    Some(60),
    Some(1000),
    false
);
target_case!(
    target_case_112,
    3,
    100,
    0,
    Some(3),
    Some(100),
    Some(0),
    true
);
target_case!(
    target_case_113,
    3,
    100,
    0,
    Some(4),
    Some(100),
    Some(0),
    false
);
target_case!(
    target_case_114,
    3,
    100,
    200,
    Some(3),
    Some(100),
    Some(200),
    true
);
target_case!(
    target_case_115,
    3,
    100,
    200,
    Some(4),
    Some(100),
    Some(200),
    false
);
target_case!(
    target_case_116,
    3,
    100,
    500,
    Some(3),
    Some(100),
    Some(500),
    true
);
target_case!(
    target_case_117,
    3,
    100,
    500,
    Some(4),
    Some(100),
    Some(500),
    false
);
target_case!(
    target_case_118,
    3,
    100,
    1000,
    Some(3),
    Some(100),
    Some(1000),
    true
);
target_case!(
    target_case_119,
    3,
    100,
    1000,
    Some(4),
    Some(100),
    Some(1000),
    false
);
target_case!(target_case_120, 4, 0, 0, Some(4), Some(0), Some(0), true);
target_case!(target_case_121, 4, 0, 0, Some(5), Some(0), Some(0), false);
target_case!(
    target_case_122,
    4,
    0,
    200,
    Some(4),
    Some(0),
    Some(200),
    true
);
target_case!(
    target_case_123,
    4,
    0,
    200,
    Some(5),
    Some(0),
    Some(200),
    false
);
target_case!(
    target_case_124,
    4,
    0,
    500,
    Some(4),
    Some(0),
    Some(500),
    true
);
target_case!(
    target_case_125,
    4,
    0,
    500,
    Some(5),
    Some(0),
    Some(500),
    false
);
target_case!(
    target_case_126,
    4,
    0,
    1000,
    Some(4),
    Some(0),
    Some(1000),
    true
);
target_case!(
    target_case_127,
    4,
    0,
    1000,
    Some(5),
    Some(0),
    Some(1000),
    false
);
target_case!(target_case_128, 4, 10, 0, Some(4), Some(10), Some(0), true);
target_case!(target_case_129, 4, 10, 0, Some(5), Some(10), Some(0), false);
target_case!(
    target_case_130,
    4,
    10,
    200,
    Some(4),
    Some(10),
    Some(200),
    true
);
target_case!(
    target_case_131,
    4,
    10,
    200,
    Some(5),
    Some(10),
    Some(200),
    false
);
target_case!(
    target_case_132,
    4,
    10,
    500,
    Some(4),
    Some(10),
    Some(500),
    true
);
target_case!(
    target_case_133,
    4,
    10,
    500,
    Some(5),
    Some(10),
    Some(500),
    false
);
target_case!(
    target_case_134,
    4,
    10,
    1000,
    Some(4),
    Some(10),
    Some(1000),
    true
);
target_case!(
    target_case_135,
    4,
    10,
    1000,
    Some(5),
    Some(10),
    Some(1000),
    false
);
target_case!(target_case_136, 4, 30, 0, Some(4), Some(30), Some(0), true);
target_case!(target_case_137, 4, 30, 0, Some(5), Some(30), Some(0), false);
target_case!(
    target_case_138,
    4,
    30,
    200,
    Some(4),
    Some(30),
    Some(200),
    true
);
target_case!(
    target_case_139,
    4,
    30,
    200,
    Some(5),
    Some(30),
    Some(200),
    false
);
target_case!(
    target_case_140,
    4,
    30,
    500,
    Some(4),
    Some(30),
    Some(500),
    true
);
target_case!(
    target_case_141,
    4,
    30,
    500,
    Some(5),
    Some(30),
    Some(500),
    false
);
target_case!(
    target_case_142,
    4,
    30,
    1000,
    Some(4),
    Some(30),
    Some(1000),
    true
);
target_case!(
    target_case_143,
    4,
    30,
    1000,
    Some(5),
    Some(30),
    Some(1000),
    false
);
target_case!(target_case_144, 4, 60, 0, Some(4), Some(60), Some(0), true);
target_case!(target_case_145, 4, 60, 0, Some(5), Some(60), Some(0), false);
target_case!(
    target_case_146,
    4,
    60,
    200,
    Some(4),
    Some(60),
    Some(200),
    true
);
target_case!(
    target_case_147,
    4,
    60,
    200,
    Some(5),
    Some(60),
    Some(200),
    false
);
target_case!(
    target_case_148,
    4,
    60,
    500,
    Some(4),
    Some(60),
    Some(500),
    true
);
target_case!(
    target_case_149,
    4,
    60,
    500,
    Some(5),
    Some(60),
    Some(500),
    false
);
target_case!(
    target_case_150,
    4,
    60,
    1000,
    Some(4),
    Some(60),
    Some(1000),
    true
);
target_case!(
    target_case_151,
    4,
    60,
    1000,
    Some(5),
    Some(60),
    Some(1000),
    false
);
target_case!(
    target_case_152,
    4,
    100,
    0,
    Some(4),
    Some(100),
    Some(0),
    true
);
target_case!(
    target_case_153,
    4,
    100,
    0,
    Some(5),
    Some(100),
    Some(0),
    false
);
target_case!(
    target_case_154,
    4,
    100,
    200,
    Some(4),
    Some(100),
    Some(200),
    true
);
target_case!(
    target_case_155,
    4,
    100,
    200,
    Some(5),
    Some(100),
    Some(200),
    false
);
target_case!(
    target_case_156,
    4,
    100,
    500,
    Some(4),
    Some(100),
    Some(500),
    true
);
target_case!(
    target_case_157,
    4,
    100,
    500,
    Some(5),
    Some(100),
    Some(500),
    false
);
target_case!(
    target_case_158,
    4,
    100,
    1000,
    Some(4),
    Some(100),
    Some(1000),
    true
);
target_case!(
    target_case_159,
    4,
    100,
    1000,
    Some(5),
    Some(100),
    Some(1000),
    false
);
target_case!(target_case_160, 5, 0, 0, Some(5), Some(0), Some(0), true);
target_case!(target_case_161, 5, 0, 0, Some(6), Some(0), Some(0), false);
target_case!(
    target_case_162,
    5,
    0,
    200,
    Some(5),
    Some(0),
    Some(200),
    true
);
target_case!(
    target_case_163,
    5,
    0,
    200,
    Some(6),
    Some(0),
    Some(200),
    false
);
target_case!(
    target_case_164,
    5,
    0,
    500,
    Some(5),
    Some(0),
    Some(500),
    true
);
target_case!(
    target_case_165,
    5,
    0,
    500,
    Some(6),
    Some(0),
    Some(500),
    false
);
target_case!(
    target_case_166,
    5,
    0,
    1000,
    Some(5),
    Some(0),
    Some(1000),
    true
);
target_case!(
    target_case_167,
    5,
    0,
    1000,
    Some(6),
    Some(0),
    Some(1000),
    false
);
target_case!(target_case_168, 5, 10, 0, Some(5), Some(10), Some(0), true);
target_case!(target_case_169, 5, 10, 0, Some(6), Some(10), Some(0), false);
target_case!(
    target_case_170,
    5,
    10,
    200,
    Some(5),
    Some(10),
    Some(200),
    true
);
target_case!(
    target_case_171,
    5,
    10,
    200,
    Some(6),
    Some(10),
    Some(200),
    false
);
target_case!(
    target_case_172,
    5,
    10,
    500,
    Some(5),
    Some(10),
    Some(500),
    true
);
target_case!(
    target_case_173,
    5,
    10,
    500,
    Some(6),
    Some(10),
    Some(500),
    false
);
target_case!(
    target_case_174,
    5,
    10,
    1000,
    Some(5),
    Some(10),
    Some(1000),
    true
);
target_case!(
    target_case_175,
    5,
    10,
    1000,
    Some(6),
    Some(10),
    Some(1000),
    false
);
target_case!(target_case_176, 5, 30, 0, Some(5), Some(30), Some(0), true);
target_case!(target_case_177, 5, 30, 0, Some(6), Some(30), Some(0), false);
target_case!(
    target_case_178,
    5,
    30,
    200,
    Some(5),
    Some(30),
    Some(200),
    true
);
target_case!(
    target_case_179,
    5,
    30,
    200,
    Some(6),
    Some(30),
    Some(200),
    false
);
target_case!(
    target_case_180,
    5,
    30,
    500,
    Some(5),
    Some(30),
    Some(500),
    true
);
target_case!(
    target_case_181,
    5,
    30,
    500,
    Some(6),
    Some(30),
    Some(500),
    false
);
target_case!(
    target_case_182,
    5,
    30,
    1000,
    Some(5),
    Some(30),
    Some(1000),
    true
);
target_case!(
    target_case_183,
    5,
    30,
    1000,
    Some(6),
    Some(30),
    Some(1000),
    false
);
target_case!(target_case_184, 5, 60, 0, Some(5), Some(60), Some(0), true);
target_case!(target_case_185, 5, 60, 0, Some(6), Some(60), Some(0), false);
target_case!(
    target_case_186,
    5,
    60,
    200,
    Some(5),
    Some(60),
    Some(200),
    true
);
target_case!(
    target_case_187,
    5,
    60,
    200,
    Some(6),
    Some(60),
    Some(200),
    false
);
target_case!(
    target_case_188,
    5,
    60,
    500,
    Some(5),
    Some(60),
    Some(500),
    true
);
target_case!(
    target_case_189,
    5,
    60,
    500,
    Some(6),
    Some(60),
    Some(500),
    false
);
target_case!(
    target_case_190,
    5,
    60,
    1000,
    Some(5),
    Some(60),
    Some(1000),
    true
);
target_case!(
    target_case_191,
    5,
    60,
    1000,
    Some(6),
    Some(60),
    Some(1000),
    false
);
target_case!(
    target_case_192,
    5,
    100,
    0,
    Some(5),
    Some(100),
    Some(0),
    true
);
target_case!(
    target_case_193,
    5,
    100,
    0,
    Some(6),
    Some(100),
    Some(0),
    false
);
target_case!(
    target_case_194,
    5,
    100,
    200,
    Some(5),
    Some(100),
    Some(200),
    true
);
target_case!(
    target_case_195,
    5,
    100,
    200,
    Some(6),
    Some(100),
    Some(200),
    false
);
target_case!(
    target_case_196,
    5,
    100,
    500,
    Some(5),
    Some(100),
    Some(500),
    true
);
target_case!(
    target_case_197,
    5,
    100,
    500,
    Some(6),
    Some(100),
    Some(500),
    false
);
target_case!(
    target_case_198,
    5,
    100,
    1000,
    Some(5),
    Some(100),
    Some(1000),
    true
);
target_case!(
    target_case_199,
    5,
    100,
    1000,
    Some(6),
    Some(100),
    Some(1000),
    false
);
target_case!(target_case_200, 6, 0, 0, Some(6), Some(0), Some(0), true);
target_case!(target_case_201, 6, 0, 0, Some(7), Some(0), Some(0), false);
target_case!(
    target_case_202,
    6,
    0,
    200,
    Some(6),
    Some(0),
    Some(200),
    true
);
target_case!(
    target_case_203,
    6,
    0,
    200,
    Some(7),
    Some(0),
    Some(200),
    false
);
target_case!(
    target_case_204,
    6,
    0,
    500,
    Some(6),
    Some(0),
    Some(500),
    true
);
target_case!(
    target_case_205,
    6,
    0,
    500,
    Some(7),
    Some(0),
    Some(500),
    false
);
target_case!(
    target_case_206,
    6,
    0,
    1000,
    Some(6),
    Some(0),
    Some(1000),
    true
);
target_case!(
    target_case_207,
    6,
    0,
    1000,
    Some(7),
    Some(0),
    Some(1000),
    false
);
target_case!(target_case_208, 6, 10, 0, Some(6), Some(10), Some(0), true);
target_case!(target_case_209, 6, 10, 0, Some(7), Some(10), Some(0), false);
target_case!(
    target_case_210,
    6,
    10,
    200,
    Some(6),
    Some(10),
    Some(200),
    true
);
target_case!(
    target_case_211,
    6,
    10,
    200,
    Some(7),
    Some(10),
    Some(200),
    false
);
target_case!(
    target_case_212,
    6,
    10,
    500,
    Some(6),
    Some(10),
    Some(500),
    true
);
target_case!(
    target_case_213,
    6,
    10,
    500,
    Some(7),
    Some(10),
    Some(500),
    false
);
target_case!(
    target_case_214,
    6,
    10,
    1000,
    Some(6),
    Some(10),
    Some(1000),
    true
);
target_case!(
    target_case_215,
    6,
    10,
    1000,
    Some(7),
    Some(10),
    Some(1000),
    false
);
target_case!(target_case_216, 6, 30, 0, Some(6), Some(30), Some(0), true);
target_case!(target_case_217, 6, 30, 0, Some(7), Some(30), Some(0), false);
target_case!(
    target_case_218,
    6,
    30,
    200,
    Some(6),
    Some(30),
    Some(200),
    true
);
target_case!(
    target_case_219,
    6,
    30,
    200,
    Some(7),
    Some(30),
    Some(200),
    false
);
target_case!(
    target_case_220,
    6,
    30,
    500,
    Some(6),
    Some(30),
    Some(500),
    true
);
target_case!(
    target_case_221,
    6,
    30,
    500,
    Some(7),
    Some(30),
    Some(500),
    false
);
target_case!(
    target_case_222,
    6,
    30,
    1000,
    Some(6),
    Some(30),
    Some(1000),
    true
);
target_case!(
    target_case_223,
    6,
    30,
    1000,
    Some(7),
    Some(30),
    Some(1000),
    false
);
target_case!(target_case_224, 6, 60, 0, Some(6), Some(60), Some(0), true);
target_case!(target_case_225, 6, 60, 0, Some(7), Some(60), Some(0), false);
target_case!(
    target_case_226,
    6,
    60,
    200,
    Some(6),
    Some(60),
    Some(200),
    true
);
target_case!(
    target_case_227,
    6,
    60,
    200,
    Some(7),
    Some(60),
    Some(200),
    false
);
target_case!(
    target_case_228,
    6,
    60,
    500,
    Some(6),
    Some(60),
    Some(500),
    true
);
target_case!(
    target_case_229,
    6,
    60,
    500,
    Some(7),
    Some(60),
    Some(500),
    false
);
target_case!(
    target_case_230,
    6,
    60,
    1000,
    Some(6),
    Some(60),
    Some(1000),
    true
);
target_case!(
    target_case_231,
    6,
    60,
    1000,
    Some(7),
    Some(60),
    Some(1000),
    false
);
target_case!(
    target_case_232,
    6,
    100,
    0,
    Some(6),
    Some(100),
    Some(0),
    true
);
target_case!(
    target_case_233,
    6,
    100,
    0,
    Some(7),
    Some(100),
    Some(0),
    false
);
target_case!(
    target_case_234,
    6,
    100,
    200,
    Some(6),
    Some(100),
    Some(200),
    true
);
target_case!(
    target_case_235,
    6,
    100,
    200,
    Some(7),
    Some(100),
    Some(200),
    false
);
target_case!(
    target_case_236,
    6,
    100,
    500,
    Some(6),
    Some(100),
    Some(500),
    true
);
target_case!(
    target_case_237,
    6,
    100,
    500,
    Some(7),
    Some(100),
    Some(500),
    false
);
target_case!(
    target_case_238,
    6,
    100,
    1000,
    Some(6),
    Some(100),
    Some(1000),
    true
);
target_case!(
    target_case_239,
    6,
    100,
    1000,
    Some(7),
    Some(100),
    Some(1000),
    false
);

macro_rules! key_case {
    ($name:ident, $action:expr, $prefix:expr) => {
        #[test]
        fn $name() {
            let key = $action.stable_key();
            assert!(key.starts_with($prefix));
        }
    };
}

key_case!(deal_key, AutoAction::Deal, "deal");
key_case!(skip_blind_key, AutoAction::SkipBlind, "skip_blind");
key_case!(enter_shop_key, AutoAction::EnterShop, "enter_shop");
key_case!(leave_shop_key, AutoAction::LeaveShop, "leave_shop");
key_case!(reroll_shop_key, AutoAction::RerollShop, "reroll_shop");
key_case!(skip_pack_key, AutoAction::SkipPack, "skip_pack");
key_case!(next_blind_key, AutoAction::NextBlind, "next_blind");

key_case!(play_key_0, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_1, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_2, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_3, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_4, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_5, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_6, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_7, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_8, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_9, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_10, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_11, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_12, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_13, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_14, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_15, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_16, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_17, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_18, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_19, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_20, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_21, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_22, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_23, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_24, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_25, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_26, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_27, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_28, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_29, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_30, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_31, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_32, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_33, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_34, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_35, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_36, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_37, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_38, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_39, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_40, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_41, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_42, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_43, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_44, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_45, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_46, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_47, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_48, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_49, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_50, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_51, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_52, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_53, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_54, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_55, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_56, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_57, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_58, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_59, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_60, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_61, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_62, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_63, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_64, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_65, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_66, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_67, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_68, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_69, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_70, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_71, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_72, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_73, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_74, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(play_key_75, AutoAction::Play { indices: vec![0] }, "play:");
key_case!(play_key_76, AutoAction::Play { indices: vec![1] }, "play:");
key_case!(play_key_77, AutoAction::Play { indices: vec![2] }, "play:");
key_case!(play_key_78, AutoAction::Play { indices: vec![3] }, "play:");
key_case!(play_key_79, AutoAction::Play { indices: vec![4] }, "play:");
key_case!(
    discard_key_0,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_1,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_2,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_3,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_4,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_5,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_6,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_7,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_8,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_9,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_10,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_11,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_12,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_13,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_14,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_15,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_16,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_17,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_18,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_19,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_20,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_21,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_22,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_23,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_24,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_25,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_26,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_27,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_28,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_29,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_30,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_31,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_32,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_33,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_34,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_35,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_36,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_37,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_38,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_39,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_40,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_41,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_42,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_43,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_44,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_45,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_46,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_47,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_48,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_49,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_50,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_51,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_52,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_53,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_54,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_55,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_56,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_57,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_58,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_59,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_60,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_61,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_62,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_63,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_64,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_65,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_66,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_67,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_68,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_69,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_70,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_71,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_72,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_73,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_74,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);
key_case!(
    discard_key_75,
    AutoAction::Discard { indices: vec![0] },
    "discard:"
);
key_case!(
    discard_key_76,
    AutoAction::Discard { indices: vec![1] },
    "discard:"
);
key_case!(
    discard_key_77,
    AutoAction::Discard { indices: vec![2] },
    "discard:"
);
key_case!(
    discard_key_78,
    AutoAction::Discard { indices: vec![3] },
    "discard:"
);
key_case!(
    discard_key_79,
    AutoAction::Discard { indices: vec![4] },
    "discard:"
);

macro_rules! score_case {
    ($name:ident, $ante:expr, $money:expr, $score:expr, $steps:expr) => {
        #[test]
        fn $name() {
            let metrics = EvalMetrics {
                ante: $ante,
                money: $money,
                blind_score: $score,
                blind_target: 1000,
                blind_failed: false,
                blind_cleared: true,
            };
            let weights = ObjectiveWeights::default();
            let value = weighted_score(metrics, weights, $steps);
            assert!(value.is_finite());
        }
    };
}

score_case!(score_case_0, 1, 0, 0, 0);
score_case!(score_case_1, 1, 0, 100, 1);
score_case!(score_case_2, 1, 0, 300, 2);
score_case!(score_case_3, 1, 0, 600, 3);
score_case!(score_case_4, 1, 0, 1000, 4);
score_case!(score_case_5, 1, 20, 0, 5);
score_case!(score_case_6, 1, 20, 100, 6);
score_case!(score_case_7, 1, 20, 300, 7);
score_case!(score_case_8, 1, 20, 600, 8);
score_case!(score_case_9, 1, 20, 1000, 9);
score_case!(score_case_10, 1, 50, 0, 10);
score_case!(score_case_11, 1, 50, 100, 11);
score_case!(score_case_12, 1, 50, 300, 12);
score_case!(score_case_13, 1, 50, 600, 13);
score_case!(score_case_14, 1, 50, 1000, 14);
score_case!(score_case_15, 1, 100, 0, 15);
score_case!(score_case_16, 1, 100, 100, 16);
score_case!(score_case_17, 1, 100, 300, 17);
score_case!(score_case_18, 1, 100, 600, 18);
score_case!(score_case_19, 1, 100, 1000, 19);
score_case!(score_case_20, 2, 0, 0, 20);
score_case!(score_case_21, 2, 0, 100, 21);
score_case!(score_case_22, 2, 0, 300, 22);
score_case!(score_case_23, 2, 0, 600, 23);
score_case!(score_case_24, 2, 0, 1000, 24);
score_case!(score_case_25, 2, 20, 0, 25);
score_case!(score_case_26, 2, 20, 100, 26);
score_case!(score_case_27, 2, 20, 300, 27);
score_case!(score_case_28, 2, 20, 600, 28);
score_case!(score_case_29, 2, 20, 1000, 29);
score_case!(score_case_30, 2, 50, 0, 30);
score_case!(score_case_31, 2, 50, 100, 31);
score_case!(score_case_32, 2, 50, 300, 32);
score_case!(score_case_33, 2, 50, 600, 33);
score_case!(score_case_34, 2, 50, 1000, 34);
score_case!(score_case_35, 2, 100, 0, 35);
score_case!(score_case_36, 2, 100, 100, 36);
score_case!(score_case_37, 2, 100, 300, 37);
score_case!(score_case_38, 2, 100, 600, 38);
score_case!(score_case_39, 2, 100, 1000, 39);
score_case!(score_case_40, 3, 0, 0, 0);
score_case!(score_case_41, 3, 0, 100, 1);
score_case!(score_case_42, 3, 0, 300, 2);
score_case!(score_case_43, 3, 0, 600, 3);
score_case!(score_case_44, 3, 0, 1000, 4);
score_case!(score_case_45, 3, 20, 0, 5);
score_case!(score_case_46, 3, 20, 100, 6);
score_case!(score_case_47, 3, 20, 300, 7);
score_case!(score_case_48, 3, 20, 600, 8);
score_case!(score_case_49, 3, 20, 1000, 9);
score_case!(score_case_50, 3, 50, 0, 10);
score_case!(score_case_51, 3, 50, 100, 11);
score_case!(score_case_52, 3, 50, 300, 12);
score_case!(score_case_53, 3, 50, 600, 13);
score_case!(score_case_54, 3, 50, 1000, 14);
score_case!(score_case_55, 3, 100, 0, 15);
score_case!(score_case_56, 3, 100, 100, 16);
score_case!(score_case_57, 3, 100, 300, 17);
score_case!(score_case_58, 3, 100, 600, 18);
score_case!(score_case_59, 3, 100, 1000, 19);
score_case!(score_case_60, 4, 0, 0, 20);
score_case!(score_case_61, 4, 0, 100, 21);
score_case!(score_case_62, 4, 0, 300, 22);
score_case!(score_case_63, 4, 0, 600, 23);
score_case!(score_case_64, 4, 0, 1000, 24);
score_case!(score_case_65, 4, 20, 0, 25);
score_case!(score_case_66, 4, 20, 100, 26);
score_case!(score_case_67, 4, 20, 300, 27);
score_case!(score_case_68, 4, 20, 600, 28);
score_case!(score_case_69, 4, 20, 1000, 29);
score_case!(score_case_70, 4, 50, 0, 30);
score_case!(score_case_71, 4, 50, 100, 31);
score_case!(score_case_72, 4, 50, 300, 32);
score_case!(score_case_73, 4, 50, 600, 33);
score_case!(score_case_74, 4, 50, 1000, 34);
score_case!(score_case_75, 4, 100, 0, 35);
score_case!(score_case_76, 4, 100, 100, 36);
score_case!(score_case_77, 4, 100, 300, 37);
score_case!(score_case_78, 4, 100, 600, 38);
score_case!(score_case_79, 4, 100, 1000, 39);
score_case!(score_case_80, 5, 0, 0, 0);
score_case!(score_case_81, 5, 0, 100, 1);
score_case!(score_case_82, 5, 0, 300, 2);
score_case!(score_case_83, 5, 0, 600, 3);
score_case!(score_case_84, 5, 0, 1000, 4);
score_case!(score_case_85, 5, 20, 0, 5);
score_case!(score_case_86, 5, 20, 100, 6);
score_case!(score_case_87, 5, 20, 300, 7);
score_case!(score_case_88, 5, 20, 600, 8);
score_case!(score_case_89, 5, 20, 1000, 9);
score_case!(score_case_90, 5, 50, 0, 10);
score_case!(score_case_91, 5, 50, 100, 11);
score_case!(score_case_92, 5, 50, 300, 12);
score_case!(score_case_93, 5, 50, 600, 13);
score_case!(score_case_94, 5, 50, 1000, 14);
score_case!(score_case_95, 5, 100, 0, 15);
score_case!(score_case_96, 5, 100, 100, 16);
score_case!(score_case_97, 5, 100, 300, 17);
score_case!(score_case_98, 5, 100, 600, 18);
score_case!(score_case_99, 5, 100, 1000, 19);
score_case!(score_case_100, 6, 0, 0, 20);
score_case!(score_case_101, 6, 0, 100, 21);
score_case!(score_case_102, 6, 0, 300, 22);
score_case!(score_case_103, 6, 0, 600, 23);
score_case!(score_case_104, 6, 0, 1000, 24);
score_case!(score_case_105, 6, 20, 0, 25);
score_case!(score_case_106, 6, 20, 100, 26);
score_case!(score_case_107, 6, 20, 300, 27);
score_case!(score_case_108, 6, 20, 600, 28);
score_case!(score_case_109, 6, 20, 1000, 29);
score_case!(score_case_110, 6, 50, 0, 30);
score_case!(score_case_111, 6, 50, 100, 31);
score_case!(score_case_112, 6, 50, 300, 32);
score_case!(score_case_113, 6, 50, 600, 33);
score_case!(score_case_114, 6, 50, 1000, 34);
score_case!(score_case_115, 6, 100, 0, 35);
score_case!(score_case_116, 6, 100, 100, 36);
score_case!(score_case_117, 6, 100, 300, 37);
score_case!(score_case_118, 6, 100, 600, 38);
score_case!(score_case_119, 6, 100, 1000, 39);
score_case!(score_case_120, 7, 0, 0, 0);
score_case!(score_case_121, 7, 0, 100, 1);
score_case!(score_case_122, 7, 0, 300, 2);
score_case!(score_case_123, 7, 0, 600, 3);
score_case!(score_case_124, 7, 0, 1000, 4);
score_case!(score_case_125, 7, 20, 0, 5);
score_case!(score_case_126, 7, 20, 100, 6);
score_case!(score_case_127, 7, 20, 300, 7);
score_case!(score_case_128, 7, 20, 600, 8);
score_case!(score_case_129, 7, 20, 1000, 9);
score_case!(score_case_130, 7, 50, 0, 10);
score_case!(score_case_131, 7, 50, 100, 11);
score_case!(score_case_132, 7, 50, 300, 12);
score_case!(score_case_133, 7, 50, 600, 13);
score_case!(score_case_134, 7, 50, 1000, 14);
score_case!(score_case_135, 7, 100, 0, 15);
score_case!(score_case_136, 7, 100, 100, 16);
score_case!(score_case_137, 7, 100, 300, 17);
score_case!(score_case_138, 7, 100, 600, 18);
score_case!(score_case_139, 7, 100, 1000, 19);
