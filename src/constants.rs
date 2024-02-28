use crate::{
    HalfWord, Integer, QuarterWord, Scaled, SmallNumber, StrNum
};

// String initialization in Global
pub const EMPTY_STRING: StrNum = 256;
pub(crate) const EXT_TEX: StrNum = 257;
pub(crate) const EXT_LOG: StrNum = 258;
pub(crate) const EXT_DVI: StrNum = 259;
pub(crate) const EXT_FMT: StrNum = 260;
pub(crate) const EXT_TFM: StrNum = 261;

pub(crate) const TEX_AREA: StrNum = 262;
pub(crate) const TEX_FONT_AREA: StrNum = 263;

pub(crate) const TEX_AREA_STRING: &str = "TeXinputs/";
pub(crate) const TEX_FONT_AREA_STRING: &str = "TeXfonts/";
pub(crate) const TEX_FORMAT_AREA: &str = "TeXformats/";
pub(crate) const TEX_FORMAT_DEFAULT: &str = "plain.fmt";

pub(crate) const FONT_STRING: StrNum = 264;
pub(crate) const NOTEXPANDED_STRING: StrNum = 265;
pub(crate) const NULLFONT_STRING: StrNum = 266;
pub(crate) const INACCESSIBLE_STRING: StrNum = 267;
pub(crate) const INITEX_IDENT_STRING: StrNum = 268;
pub(crate) const ENDWRITE_STRING: StrNum = 269;
pub(crate) const ENDTEMPLATE_STRING: StrNum = 270;

// Part 1: Introduction
// Section 2
pub(crate) const BANNER: &str = "This is TeX, Version 3.141592653";

// Section 11
pub(crate) const MEM_MAX: Integer = 30000;
pub(crate) const MEM_MIN: Integer = 0;
pub(crate) const BUF_SIZE: Integer = 200_000; // 500 is not enough
pub(crate) const ERROR_LINE: Integer = 72;
pub(crate) const HALF_ERROR_LINE: Integer = 36;
pub(crate) const MAX_PRINT_LINE: Integer = 79;
pub(crate) const STACK_SIZE: Integer = 200;
pub(crate) const MAX_IN_OPEN: Integer = 6;
pub(crate) const FONT_MAX: Integer = 75;
pub(crate) const FONT_MEM_SIZE: Integer = 20000;
pub(crate) const PARAM_SIZE: Integer = 60;
pub(crate) const NEST_SIZE: Integer = 40;
pub(crate) const MAX_STRINGS: Integer = 3000;
pub(crate) const POOL_SIZE: Integer = 32000;
pub(crate) const SAVE_SIZE: Integer = 600;
pub(crate) const TRIE_SIZE: Integer = 8000;
pub(crate) const TRIE_OP_SIZE: Integer = 500;
pub(crate) const DVI_BUF_SIZE: Integer = 800;

// Section 12
pub(crate) const MEM_BOT: Integer = 0;
pub(crate) const MEM_TOP: Integer = 30000;
pub(crate) const FONT_BASE: Integer = 0;
pub(crate) const HASH_SIZE: Integer = 2100;
pub(crate) const HASH_PRIME: Integer = 1777;
pub(crate) const HYPH_SIZE: Integer = 307;

// Section 16
pub(crate) const EMPTY: HalfWord = 0;

// Part 2: The character set
// Section 22
pub(crate) const NULL_CODE: Integer = 0;
pub(crate) const CARRIAGE_RETURN: Integer = 13; // `15
pub(crate) const INVALID_CODE: Integer = 127; // `177;

// Part 5: On-line and off-line printing
// Section 54
pub(crate) const NO_PRINT: Integer = 16;
pub(crate) const TERM_ONLY: Integer = 17;
pub(crate) const LOG_ONLY: Integer = 18;
pub(crate) const TERM_AND_LOG: Integer = 19;
pub(crate) const PSEUDO: Integer = 20;
pub(crate) const NEW_STRING: Integer = 21;

// Part 6: Reporting errors
// Section 73
pub(crate) const BATCH_MODE: Integer = 0;
pub(crate) const ERROR_STOP_MODE: Integer = 3;

// Section 76
pub const SPOTLESS: usize = 0;
pub(crate) const WARNING_ISSUED: usize = 1;
// pub(crate) const ERROR_MESSAGE_ISSUED: usize = 2;
// pub(crate) const FATAL_ERROR_STOP: usize = 3;

// Part 7: Arithmetic with scaled dimensions
// Section 101
pub(crate) const UNITY: Scaled = 1 << 16;
pub(crate) const TWO: Scaled = 1 << 17;

// Section 108
pub(crate) const INF_BAD: HalfWord = 10_000;

// Part 8: Packed data
// Section 110
pub(crate) const MIN_QUARTERWORD: QuarterWord = 0;
pub(crate) const MAX_QUARTERWORD: QuarterWord = 65535;
pub(crate) const MIN_HALFWORD: HalfWord = -0x3fff_ffff;
pub(crate) const MAX_HALFWORD: HalfWord = 0x3fff_ffff;

// Part 9: Dynamic memory allocation
// Section 115
pub(crate) const NULL: HalfWord = MIN_HALFWORD;
pub(crate) const EMPTY_FLAG: HalfWord = MAX_HALFWORD;

// Part 10: Data structures for boxes and their friends
// Section 135
pub(crate) const HLIST_NODE: QuarterWord = 0;
pub(crate) const BOX_NODE_SIZE: Integer = 7;
pub(crate) const WIDTH_OFFSET: Integer = 1;
pub(crate) const DEPTH_OFFSET: Integer = 2;
pub(crate) const HEIGHT_OFFSET: Integer = 3;
pub(crate) const LIST_OFFSET: Integer = 5;
pub(crate) const NORMAL: QuarterWord = 0;
pub(crate) const STRETCHING: QuarterWord = 1;
pub(crate) const SHRINKING: QuarterWord = 2;
pub(crate) const GLUE_OFFSET: Integer = 6;

// Section 137
pub(crate) const VLIST_NODE: QuarterWord = 1;

// Section 138
pub(crate) const RULE_NODE: QuarterWord = 2;
pub(crate) const RULE_NODE_SIZE: Integer = 4;
pub(crate) const NULL_FLAG: Integer = -(1 << 30); // `10000000000;

// Section 140
pub(crate) const INS_NODE: QuarterWord = 3;
pub(crate) const INS_NODE_SIZE: Integer = 5;

// Section 141
pub(crate) const MARK_NODE: QuarterWord = 4;
pub(crate) const SMALL_NODE_SIZE: Integer = 2;

// Section 142
pub(crate) const ADJUST_NODE: QuarterWord = 5;

// Section 143
pub(crate) const LIGATURE_NODE: QuarterWord = 6;

// Section 145
pub(crate) const DISC_NODE: QuarterWord = 7;

// Section 146
pub(crate) const WHATSIT_NODE: QuarterWord = 8;

// Section 147
pub(crate) const MATH_NODE: QuarterWord = 9;
pub(crate) const BEFORE: QuarterWord = 0;
pub(crate) const AFTER: QuarterWord = 1;

// Section 149
pub(crate) const GLUE_NODE: QuarterWord = 10;
pub(crate) const COND_MATH_GLUE: QuarterWord = 98;
pub(crate) const MU_GLUE: QuarterWord = 99;
pub(crate) const A_LEADERS: QuarterWord = 100;
pub(crate) const C_LEADERS: QuarterWord = 101;
pub(crate) const X_LEADERS: QuarterWord = 102;

// Section 150
pub(crate) const GLUE_SPEC_SIZE: Integer = 4;
pub(crate) const FIL: QuarterWord = 1;
pub(crate) const FILL: QuarterWord = 2;
pub(crate) const FILLL: QuarterWord = 3;

// Section 155
pub(crate) const KERN_NODE: QuarterWord = 11;
pub(crate) const EXPLICIT: QuarterWord = 1;
pub(crate) const ACC_KERN: QuarterWord = 2;

// Section 157
pub(crate) const PENALTY_NODE: QuarterWord = 12;
pub(crate) const INF_PENALTY: Integer = INF_BAD;
pub(crate) const EJECT_PENALTY: Integer = -INF_PENALTY;

// Section 159
pub(crate) const UNSET_NODE: QuarterWord = 13;

// Part 11: Memory Layout
// Section 162
pub(crate) const ZERO_GLUE: Integer = MEM_BOT;
pub(crate) const FIL_GLUE: Integer = ZERO_GLUE + GLUE_SPEC_SIZE;
pub(crate) const FILL_GLUE: Integer = FIL_GLUE + GLUE_SPEC_SIZE;
pub(crate) const SS_GLUE: Integer = FILL_GLUE + GLUE_SPEC_SIZE;
pub(crate) const FIL_NEG_GLUE: Integer = SS_GLUE + GLUE_SPEC_SIZE;
pub(crate) const LO_MEM_STAT_MAX: Integer = FIL_NEG_GLUE + GLUE_SPEC_SIZE - 1;
pub(crate) const PAGE_INS_HEAD: HalfWord = MEM_TOP;
pub(crate) const CONTRIB_HEAD: HalfWord = MEM_TOP - 1;
pub(crate) const PAGE_HEAD: HalfWord = MEM_TOP - 2;
pub(crate) const TEMP_HEAD: HalfWord = MEM_TOP - 3;
pub(crate) const HOLD_HEAD: HalfWord = MEM_TOP - 4;
pub(crate) const ADJUST_HEAD: HalfWord = MEM_TOP - 5;
pub(crate) const ACTIVE: HalfWord = MEM_TOP - 7;
pub(crate) const ALIGN_HEAD: HalfWord = MEM_TOP - 8;
pub(crate) const END_SPAN: HalfWord = MEM_TOP - 9;
pub(crate) const OMIT_TEMPLATE: HalfWord = MEM_TOP - 10;
pub(crate) const NULL_LIST: HalfWord = MEM_TOP - 11;
pub(crate) const LIG_TRICK: Integer = MEM_TOP - 12;
pub(crate) const GARBAGE: HalfWord = MEM_TOP - 12;
pub(crate) const BACKUP_HEAD: HalfWord = MEM_TOP - 13;
pub(crate) const HI_MEM_STAT_MIN: Integer = MEM_TOP - 13;
pub(crate) const HI_MEM_STAT_USAGE: Integer = 14;

// Part 15: The command codes
// Section 207
pub(crate) const ESCAPE: QuarterWord = 0;
pub(crate) const RELAX: QuarterWord = 0;
pub(crate) const LEFT_BRACE: QuarterWord = 1;
pub(crate) const RIGHT_BRACE: QuarterWord = 2;
pub(crate) const MATH_SHIFT: QuarterWord = 3;
pub(crate) const TAB_MARK: QuarterWord = 4;
pub(crate) const CAR_RET: QuarterWord = 5;
pub(crate) const OUT_PARAM: QuarterWord = 5;
pub(crate) const MAC_PARAM: QuarterWord = 6;
pub(crate) const SUP_MARK: QuarterWord = 7;
pub(crate) const SUB_MARK: QuarterWord = 8;
pub(crate) const IGNORE: QuarterWord = 9;
pub(crate) const ENDV: QuarterWord = 9;
pub(crate) const SPACER: QuarterWord = 10;
pub(crate) const LETTER: QuarterWord = 11;
pub(crate) const OTHER_CHAR: QuarterWord = 12;
pub(crate) const ACTIVE_CHAR: QuarterWord = 13;
pub(crate) const PAR_END: QuarterWord = 13;
pub(crate) const MATCH: QuarterWord = 13;
pub(crate) const COMMENT: QuarterWord = 14;
pub(crate) const END_MATCH: QuarterWord = 14;
pub(crate) const STOP: QuarterWord = 14;
pub(crate) const INVALID_CHAR: QuarterWord = 15;
pub(crate) const DELIM_NUM: QuarterWord = 15;
pub(crate) const MAX_CHAR_CODE: QuarterWord = 15;

// Section 208
pub(crate) const CHAR_NUM: QuarterWord = 16;
pub(crate) const MATH_CHAR_NUM: QuarterWord = 17;
pub(crate) const MARK: QuarterWord = 18;
pub(crate) const XRAY: QuarterWord = 19;
pub(crate) const MAKE_BOX: QuarterWord = 20;
pub(crate) const HMOVE: QuarterWord = 21;
pub(crate) const VMOVE: QuarterWord = 22;
pub(crate) const UN_HBOX: QuarterWord = 23;
pub(crate) const UN_VBOX: QuarterWord = 24;
pub(crate) const REMOVE_ITEM: QuarterWord = 25;
pub(crate) const HSKIP: QuarterWord = 26;
pub(crate) const VSKIP: QuarterWord = 27;
pub(crate) const MSKIP: QuarterWord = 28;
pub(crate) const KERN: QuarterWord = 29;
pub(crate) const MKERN: QuarterWord = 30;
pub(crate) const LEADER_SHIP: QuarterWord = 31;
pub(crate) const HALIGN: QuarterWord = 32;
pub(crate) const VALIGN: QuarterWord = 33;
pub(crate) const NO_ALIGN: QuarterWord = 34;
pub(crate) const VRULE: QuarterWord = 35;
pub(crate) const HRULE: QuarterWord = 36;
pub(crate) const INSERT: QuarterWord = 37;
pub(crate) const VADJUST: QuarterWord = 38;
pub(crate) const IGNORE_SPACES: QuarterWord = 39;
pub(crate) const AFTER_ASSIGNMENT: QuarterWord = 40;
pub(crate) const AFTER_GROUP: QuarterWord = 41;
pub(crate) const BREAK_PENALTY: QuarterWord = 42;
pub(crate) const START_PAR: QuarterWord = 43;
pub(crate) const ITAL_CORR: QuarterWord = 44;
pub(crate) const ACCENT: QuarterWord = 45;
pub(crate) const MATH_ACCENT: QuarterWord = 46;
pub(crate) const DISCRETIONARY: QuarterWord = 47;
pub(crate) const EQ_NO: QuarterWord = 48;
pub(crate) const LEFT_RIGHT: QuarterWord = 49;
pub(crate) const MATH_COMP: QuarterWord = 50;
pub(crate) const LIMIT_SWITCH: QuarterWord = 51;
pub(crate) const ABOVE: QuarterWord = 52;
pub(crate) const MATH_STYLE: QuarterWord = 53;
pub(crate) const MATH_CHOICE: QuarterWord = 54;
pub(crate) const NON_SCRIPT: QuarterWord = 55;
pub(crate) const VCENTER: QuarterWord = 56;
pub(crate) const CASE_SHIFT: QuarterWord = 57;
pub(crate) const MESSAGE: QuarterWord = 58;
pub(crate) const EXTENSION: QuarterWord = 59;
pub(crate) const IN_STREAM: QuarterWord = 60;
pub(crate) const BEGIN_GROUP: QuarterWord = 61;
pub(crate) const END_GROUP: QuarterWord = 62;
pub(crate) const OMIT: QuarterWord = 63;
pub(crate) const EX_SPACE: QuarterWord = 64;
pub(crate) const NO_BOUNDARY: QuarterWord = 65;
pub(crate) const RADICAL: QuarterWord = 66;
pub(crate) const END_CS_NAME: QuarterWord = 67;
pub(crate) const MIN_INTERNAL: QuarterWord = 68;
pub(crate) const CHAR_GIVEN: QuarterWord = 68;
pub(crate) const MATH_GIVEN: QuarterWord = 69;
pub(crate) const LAST_ITEM: QuarterWord = 70;
pub(crate) const MAX_NON_PREFIXED_COMMAND: QuarterWord = 70;

// Section 209
pub(crate) const TOKS_REGISTER: QuarterWord = 71;
pub(crate) const ASSIGN_TOKS: QuarterWord = 72;
pub(crate) const ASSIGN_INT: QuarterWord = 73;
pub(crate) const ASSIGN_DIMEN: QuarterWord = 74;
pub(crate) const ASSIGN_GLUE: QuarterWord = 75;
pub(crate) const ASSIGN_MU_GLUE: QuarterWord = 76;
pub(crate) const ASSIGN_FONT_DIMEN: QuarterWord = 77;
pub(crate) const ASSIGN_FONT_INT: QuarterWord = 78;
pub(crate) const SET_AUX: QuarterWord = 79;
pub(crate) const SET_PREV_GRAF: QuarterWord = 80;
pub(crate) const SET_PAGE_DIMEN: QuarterWord = 81;
pub(crate) const SET_PAGE_INT: QuarterWord = 82;
pub(crate) const SET_BOX_DIMEN: QuarterWord = 83;
pub(crate) const SET_SHAPE: QuarterWord = 84;
pub(crate) const DEF_CODE: QuarterWord = 85;
pub(crate) const DEF_FAMILY: QuarterWord = 86;
pub(crate) const SET_FONT: QuarterWord = 87;
pub(crate) const DEF_FONT: QuarterWord = 88;
pub(crate) const REGISTER: QuarterWord = 89;
pub(crate) const MAX_INTERNAL: QuarterWord = 89;
pub(crate) const ADVANCE: QuarterWord = 90;
pub(crate) const MULTIPLY: QuarterWord = 91;
pub(crate) const DIVIDE: QuarterWord = 92;
pub(crate) const PREFIX: QuarterWord = 93;
pub(crate) const LET: QuarterWord = 94;
pub(crate) const SHORTHAND_DEF: QuarterWord = 95;
pub(crate) const READ_TO_CS: QuarterWord = 96;
pub(crate) const DEF: QuarterWord = 97;
pub(crate) const SET_BOX: QuarterWord = 98;
pub(crate) const HYPH_DATA: QuarterWord = 99;
pub(crate) const SET_INTERACTION: QuarterWord = 100;
pub(crate) const MAX_COMMAND: QuarterWord = 100;

// Section 210
pub(crate) const UNDEFINED_CS: QuarterWord = MAX_COMMAND + 1;
pub(crate) const EXPAND_AFTER: QuarterWord = MAX_COMMAND + 2;
pub(crate) const NO_EXPAND: QuarterWord = MAX_COMMAND + 3;
pub(crate) const INPUT: QuarterWord = MAX_COMMAND + 4;
pub(crate) const IF_TEST: QuarterWord = MAX_COMMAND + 5;
pub(crate) const FI_OR_ELSE: QuarterWord = MAX_COMMAND + 6;
pub(crate) const CS_NAME: QuarterWord = MAX_COMMAND + 7;
pub(crate) const CONVERT: QuarterWord = MAX_COMMAND + 8;
pub(crate) const THE: QuarterWord = MAX_COMMAND + 9;
pub(crate) const TOP_BOT_MARK: QuarterWord = MAX_COMMAND + 10;
pub(crate) const CALL: QuarterWord = MAX_COMMAND + 11;
pub(crate) const LONG_CALL: QuarterWord = MAX_COMMAND + 12;
pub(crate) const OUTER_CALL: QuarterWord = MAX_COMMAND + 13;
pub(crate) const LONG_OUTER_CALL: QuarterWord = MAX_COMMAND + 14;
pub(crate) const END_TEMPLATE: QuarterWord = MAX_COMMAND + 15;
pub(crate) const DONT_EXPAND: QuarterWord = MAX_COMMAND + 16;
pub(crate) const GLUE_REF: QuarterWord = MAX_COMMAND + 17;
pub(crate) const SHAPE_REF: QuarterWord = MAX_COMMAND + 18;
pub(crate) const BOX_REF: QuarterWord = MAX_COMMAND + 19;
pub(crate) const DATA: QuarterWord = MAX_COMMAND + 20;

// Part 16: The semantic nest
// Section 211
pub(crate) const VMODE: Integer = 1;
pub(crate) const HMODE: Integer = VMODE + (MAX_COMMAND as Integer) + 1;
pub(crate) const MMODE: Integer = HMODE + (MAX_COMMAND as Integer) + 1;

// Section 212
pub(crate) const IGNORE_DEPTH: Integer = -65_536_000;

// Part 17: The table of equivalents
// Section 221
pub(crate) const LEVEL_ZERO: QuarterWord = MIN_QUARTERWORD;
pub(crate) const LEVEL_ONE: QuarterWord = LEVEL_ZERO + 1;

// Section 222
pub(crate) const ACTIVE_BASE: Integer = 1;
pub(crate) const SINGLE_BASE: Integer = ACTIVE_BASE + 256;
pub(crate) const NULL_CS: Integer = SINGLE_BASE + 256;
pub(crate) const HASH_BASE: Integer = NULL_CS + 1;
pub(crate) const FROZEN_CONTROL_SEQUENCE: Integer = HASH_BASE + HASH_SIZE;
pub(crate) const FROZEN_PROTECTION: Integer = FROZEN_CONTROL_SEQUENCE;
pub(crate) const FROZEN_CR: Integer = FROZEN_CONTROL_SEQUENCE + 1;
pub(crate) const FROZEN_END_GROUP: Integer = FROZEN_CONTROL_SEQUENCE + 2;
pub(crate) const FROZEN_RIGHT: Integer = FROZEN_CONTROL_SEQUENCE + 3;
pub(crate) const FROZEN_FI: Integer = FROZEN_CONTROL_SEQUENCE + 4;
pub(crate) const FROZEN_END_TEMPLATE: Integer = FROZEN_CONTROL_SEQUENCE + 5;
pub(crate) const FROZEN_ENDV: Integer = FROZEN_CONTROL_SEQUENCE + 6;
pub(crate) const FROZEN_RELAX: Integer = FROZEN_CONTROL_SEQUENCE + 7;
pub(crate) const END_WRITE: Integer = FROZEN_CONTROL_SEQUENCE + 8;
pub(crate) const FROZEN_DONT_EXPAND: Integer = FROZEN_CONTROL_SEQUENCE + 9;
pub(crate) const FROZEN_NULL_FONT: Integer = FROZEN_CONTROL_SEQUENCE + 10;
pub(crate) const FONT_ID_BASE: Integer = FROZEN_NULL_FONT - FONT_BASE;
pub(crate) const UNDEFINED_CONTROL_SEQUENCE: Integer = FROZEN_NULL_FONT + 257;
pub(crate) const GLUE_BASE: Integer = UNDEFINED_CONTROL_SEQUENCE + 1;

// Section 224
pub(crate) const LINE_SKIP_CODE: Integer = 0;
pub(crate) const BASELINE_SKIP_CODE: Integer = 1;
pub(crate) const PAR_SKIP_CODE: Integer = 2;
pub(crate) const ABOVE_DISPLAY_SKIP_CODE: Integer = 3;
pub(crate) const BELOW_DISPLAY_SKIP_CODE: Integer = 4;
pub(crate) const ABOVE_DISPLAY_SHORT_SKIP_CODE: Integer = 5;
pub(crate) const BELOW_DISPLAY_SHORT_SKIP_CODE: Integer = 6;
pub(crate) const LEFT_SKIP_CODE: Integer = 7;
pub(crate) const RIGHT_SKIP_CODE: Integer = 8;
pub(crate) const TOP_SKIP_CODE: Integer = 9;
pub(crate) const SPLIT_TOP_SKIP_CODE: Integer = 10;
pub(crate) const TAB_SKIP_CODE: Integer = 11;
pub(crate) const SPACE_SKIP_CODE: Integer = 12;
pub(crate) const XSPACE_SKIP_CODE: Integer = 13;
pub(crate) const PAR_FILL_SKIP_CODE: Integer = 14;
pub(crate) const THIN_MU_SKIP_CODE: Integer = 15;
pub(crate) const MED_MU_SKIP_CODE: Integer = 16;
pub(crate) const THICK_MU_SKIP_CODE: Integer = 17;
pub(crate) const GLUE_PARS: Integer = 18;
pub(crate) const SKIP_BASE: Integer = GLUE_BASE + GLUE_PARS;
pub(crate) const MU_SKIP_BASE: Integer = SKIP_BASE + 256;
pub(crate) const LOCAL_BASE: Integer = MU_SKIP_BASE + 256;

// Section 230
pub(crate) const PAR_SHAPE_LOC: Integer = LOCAL_BASE;
pub(crate) const OUTPUT_ROUTINE_LOC: Integer = LOCAL_BASE + 1;
pub(crate) const EVERY_PAR_LOC: Integer = LOCAL_BASE + 2;
pub(crate) const EVERY_MATH_LOC: Integer = LOCAL_BASE + 3;
pub(crate) const EVERY_DISPLAY_LOC: Integer = LOCAL_BASE + 4;
pub(crate) const EVERY_HBOX_LOC: Integer = LOCAL_BASE + 5;
pub(crate) const EVERY_VBOX_LOC: Integer = LOCAL_BASE + 6;
pub(crate) const EVERY_JOB_LOC: Integer = LOCAL_BASE + 7;
pub(crate) const EVERY_CR_LOC: Integer = LOCAL_BASE + 8;
pub(crate) const ERR_HELP_LOC: Integer = LOCAL_BASE + 9;
pub(crate) const TOKS_BASE: Integer = LOCAL_BASE + 10;
pub(crate) const BOX_BASE: Integer = TOKS_BASE + 256;
pub(crate) const CUR_FONT_LOC: Integer = BOX_BASE + 256;
pub(crate) const MATH_FONT_BASE: Integer = CUR_FONT_LOC + 1;
pub(crate) const CAT_CODE_BASE: Integer = MATH_FONT_BASE + 48;
pub(crate) const LC_CODE_BASE: Integer = CAT_CODE_BASE + 256;
pub(crate) const UC_CODE_BASE: Integer = LC_CODE_BASE + 256;
pub(crate) const SF_CODE_BASE: Integer = UC_CODE_BASE + 256;
pub(crate) const MATH_CODE_BASE: Integer = SF_CODE_BASE + 256;
pub(crate) const INT_BASE: Integer = MATH_CODE_BASE + 256;

// Section 232
pub(crate) const NULL_FONT: Integer = FONT_BASE;
pub(crate) const VAR_CODE: Integer = 0x7000;

// Section 236
pub(crate) const PRETOLERANCE_CODE: Integer = 0;
pub(crate) const TOLERANCE_CODE: Integer = 1;
pub(crate) const LINE_PENALTY_CODE: Integer = 2;
pub(crate) const HYPHEN_PENALTY_CODE: Integer = 3;
pub(crate) const EX_HYPHEN_PENALTY_CODE: Integer = 4;
pub(crate) const CLUB_PENALTY_CODE: Integer = 5;
pub(crate) const WIDOW_PENALTY_CODE: Integer = 6;
pub(crate) const DISPLAY_WIDOW_PENALTY_CODE: Integer = 7;
pub(crate) const BROKEN_PENALTY_CODE: Integer = 8;
pub(crate) const BIN_OP_PENALTY_CODE: Integer = 9;
pub(crate) const REL_PENALTY_CODE: Integer = 10;
pub(crate) const PRE_DISPLAY_PENALTY_CODE: Integer = 11;
pub(crate) const POST_DISPLAY_PENALTY_CODE: Integer = 12;
pub(crate) const INTER_LINE_PENALTY_CODE: Integer = 13;
pub(crate) const DOUBLE_HYPHEN_DEMERITS_CODE: Integer = 14;
pub(crate) const FINAL_HYPHEN_DEMERITS_CODE: Integer = 15;
pub(crate) const ADJ_DEMERITS_CODE: Integer = 16;
pub(crate) const MAG_CODE: Integer = 17;
pub(crate) const DELIMITER_FACTOR_CODE: Integer = 18;
pub(crate) const LOOSENESS_CODE: Integer = 19;
pub(crate) const TIME_CODE: Integer = 20;
pub(crate) const DAY_CODE: Integer = 21;
pub(crate) const MONTH_CODE: Integer = 22;
pub(crate) const YEAR_CODE: Integer = 23;
pub(crate) const SHOW_BOX_BREADTH_CODE: Integer = 24;
pub(crate) const SHOW_BOX_DEPTH_CODE: Integer = 25;
pub(crate) const HBADNESS_CODE: Integer = 26;
pub(crate) const VBADNESS_CODE: Integer = 27;
pub(crate) const PAUSING_CODE: Integer = 28;
pub(crate) const TRACING_ONLINE_CODE: Integer = 29;
pub(crate) const TRACING_MACROS_CODE: Integer = 30;
pub(crate) const TRACING_STATS_CODE: Integer = 31;
pub(crate) const TRACING_PARAGRAPHS_CODE: Integer = 32;
pub(crate) const TRACING_PAGES_CODE: Integer = 33;
pub(crate) const TRACING_OUTPUT_CODE: Integer = 34;
pub(crate) const TRACING_LOST_CHARS_CODE: Integer = 35;
pub(crate) const TRACING_COMMANDS_CODE: Integer = 36;
pub(crate) const TRACING_RESTORES_CODE: Integer = 37;
pub(crate) const UC_HYPH_CODE: Integer = 38;
pub(crate) const OUTPUT_PENALTY_CODE: Integer = 39;
pub(crate) const MAX_DEAD_CYCLES_CODE: Integer = 40;
pub(crate) const HANG_AFTER_CODE: Integer = 41;
pub(crate) const FLOATING_PENALTY_CODE: Integer = 42;
pub(crate) const GLOBAL_DEFS_CODE: Integer = 43;
pub(crate) const CUR_FAM_CODE: Integer = 44;
pub(crate) const ESCAPE_CHAR_CODE: Integer = 45;
pub(crate) const DEFAULT_HYPHEN_CHAR_CODE: Integer = 46;
pub(crate) const DEFAULT_SKEW_CHAR_CODE: Integer = 47;
pub(crate) const END_LINE_CHAR_CODE: Integer = 48;
pub(crate) const NEW_LINE_CHAR_CODE: Integer = 49;
pub(crate) const LANGUAGE_CODE: Integer = 50;
pub(crate) const LEFT_HYPHEN_MIN_CODE: Integer = 51;
pub(crate) const RIGHT_HYPHEN_MIN_CODE: Integer = 52;
pub(crate) const HOLDING_INSERTS_CODE: Integer = 53;
pub(crate) const ERROR_CONTEXT_LINES_CODE: Integer = 54;
pub(crate) const INT_PARS: Integer = 55;
pub(crate) const COUNT_BASE: Integer = INT_BASE + INT_PARS;
pub(crate) const DEL_CODE_BASE: Integer = COUNT_BASE + 256;
pub(crate) const DIMEN_BASE: Integer = DEL_CODE_BASE + 256;

// Section 247
pub(crate) const PAR_INDENT_CODE: Integer = 0;
pub(crate) const MATH_SURROUND_CODE: Integer = 1;
pub(crate) const LINE_SKIP_LIMIT_CODE: Integer = 2;
pub(crate) const HSIZE_CODE: Integer = 3;
pub(crate) const VSIZE_CODE: Integer = 4;
pub(crate) const MAX_DEPTH_CODE: Integer = 5;
pub(crate) const SPLIT_MAX_DEPTH_CODE: Integer = 6;
pub(crate) const BOX_MAX_DEPTH_CODE: Integer = 7;
pub(crate) const HFUZZ_CODE: Integer = 8;
pub(crate) const VFUZZ_CODE: Integer = 9;
pub(crate) const DELIMITER_SHORTFALL_CODE: Integer = 10;
pub(crate) const NULL_DELIMITER_SPACE_CODE: Integer = 11;
pub(crate) const SCRIPT_SPACE_CODE: Integer = 12;
pub(crate) const PRE_DISPLAY_SIZE_CODE: Integer = 13;
pub(crate) const DISPLAY_WIDTH_CODE: Integer = 14;
pub(crate) const DISPLAY_INDENT_CODE: Integer = 15;
pub(crate) const OVERFULL_RULE_CODE: Integer = 16;
pub(crate) const HANG_INDENT_CODE: Integer = 17;
pub(crate) const H_OFFSET_CODE: Integer = 18;
pub(crate) const V_OFFSET_CODE: Integer = 19;
pub(crate) const EMERGENCY_STRETCH_CODE: Integer = 20;
pub(crate) const DIMEN_PARS: Integer = 21;
pub(crate) const SCALED_BASE: Integer = DIMEN_BASE + DIMEN_PARS;
pub(crate) const EQTB_SIZE: Integer = SCALED_BASE + 255;

// Part 19: Saving and restoring equivalents
// Section 268
pub(crate) const RESTOVE_OLD_VALUE: QuarterWord = 0;
pub(crate) const RESTORE_ZERO: QuarterWord = 1;
pub(crate) const INSERT_TOKEN: QuarterWord = 2;
pub(crate) const LEVEL_BOUNDARY: QuarterWord = 3;

// Section 269
pub(crate) const BOTTOM_LEVEL: Integer = 0;
pub(crate) const SIMPLE_GROUP: Integer = 1;
pub(crate) const HBOX_GROUP: Integer = 2;
pub(crate) const ADJUSTED_HBOX_GROUP: Integer = 3;
pub(crate) const VBOX_GROUP: Integer = 4;
pub(crate) const VTOP_GROUP: Integer = 5;
pub(crate) const ALIGN_GROUP: Integer = 6;
pub(crate) const NO_ALIGN_GROUP: Integer = 7;
pub(crate) const OUTPUT_GROUP: Integer = 8;
pub(crate) const MATH_GROUP: Integer = 9;
pub(crate) const DISC_GROUP: Integer = 10;
pub(crate) const INSERT_GROUP: Integer = 11;
pub(crate) const VCENTER_GROUP: Integer = 12;
pub(crate) const MATH_CHOICE_GROUP: Integer = 13;
pub(crate) const SEMI_SIMPLE_GROUP: Integer = 14;
pub(crate) const MATH_SHIFT_GROUP: Integer = 15;
pub(crate) const MATH_LEFT_GROUP: Integer = 16;
// pub(crate) const MAX_GROUP_CODE: Integer = 16;

// Part 20: Token lists
// Section 289
pub(crate) const CS_TOKEN_FLAG: HalfWord = 0xfff;
pub(crate) const LEFT_BRACE_TOKEN: HalfWord = 0x100;
pub(crate) const LEFT_BRACE_LIMIT: HalfWord = 0x200;
pub(crate) const RIGHT_BRACE_TOKEN: HalfWord = 0x200;
pub(crate) const RIGHT_BRACE_LIMIT: HalfWord = 0x300;
// pub(crate) const MATH_SHIFT_TOKEN: HalfWord = 0x300;
pub(crate) const TAB_TOKEN: HalfWord = 0x400;
pub(crate) const OUT_PARAM_TOKEN: HalfWord = 0x500;
pub(crate) const SPACE_TOKEN: HalfWord = 0xa20;
pub(crate) const LETTER_TOKEN: HalfWord = 0xb00;
pub(crate) const OTHER_TOKEN: HalfWord = 0xc00;
pub(crate) const MATCH_TOKEN: HalfWord = 0xd00;
pub(crate) const END_MATCH_TOKEN: HalfWord = 0xe00;

// Part 22: Input stacks and states
// Section 303
pub(crate) const MID_LINE: QuarterWord = 1;
pub(crate) const SKIP_BLANKS: QuarterWord = 2 + MAX_CHAR_CODE;
pub(crate) const NEW_LINE: QuarterWord = 3 + MAX_CHAR_CODE + MAX_CHAR_CODE;

// Section 307
pub(crate) const TOKEN_LIST: QuarterWord = 0;
pub(crate) const PARAMETER: QuarterWord = 0;
pub(crate) const U_TEMPLATE: QuarterWord = 1;
pub(crate) const V_TEMPLATE: QuarterWord = 2;
pub(crate) const BACKED_UP: QuarterWord = 3;
pub(crate) const INSERTED: QuarterWord = 4;
pub(crate) const MACRO: QuarterWord = 5;
pub(crate) const OUTPUT_TEXT: QuarterWord = 6;
pub(crate) const EVERY_PAR_TEXT: QuarterWord = 7;
pub(crate) const EVERY_MATH_TEXT: QuarterWord = 8;
pub(crate) const EVERY_DISPLAY_TEXT: QuarterWord = 9;
pub(crate) const EVERY_HBOX_TEXT: QuarterWord = 10;
pub(crate) const EVERY_VBOX_TEXT: QuarterWord = 11;
pub(crate) const EVERY_JOB_TEXT: QuarterWord = 12;
pub(crate) const EVERY_CR_TEXT: QuarterWord = 13;
pub(crate) const MARK_TEXT: QuarterWord = 14;
pub(crate) const WRITE_TEXT: QuarterWord = 15;

// Part 24: Getting the next token
// Section 358
pub(crate) const NO_EXPAND_FLAG: HalfWord = 257;

// Part 25: Expanding the next token
// Section 382
pub(crate) const TOP_MARK_CODE: HalfWord = 0;
pub(crate) const FIRST_MARK_CODE: HalfWord = 1;
pub(crate) const BOT_MARK_CODE: HalfWord = 2;
pub(crate) const SPLIT_FIRST_MARK_CODE: HalfWord = 3;
pub(crate) const SPLIT_BOT_MARK_CODE: HalfWord = 4;

// Part 26: Basic scanning subroutines
// Section 410
pub(crate) const INT_VAL: Integer = 0;
pub(crate) const DIMEN_VAL: Integer = 1;
pub(crate) const GLUE_VAL: Integer = 2;
pub(crate) const MU_VAL: Integer = 3;
pub(crate) const IDENT_VAL: Integer = 4;
pub(crate) const TOK_VAL: Integer = 5;

// Section 416
pub(crate) const INPUT_LINE_NO_CODE: Integer = GLUE_VAL + 1;
pub(crate) const BADNESS_CODE: Integer = GLUE_VAL + 2;

// Section 421
pub(crate) const MAX_DIMEN: Scaled = (1 << 30) - 1;

// Section 438
pub(crate) const OCTAL_TOKEN: HalfWord = OTHER_TOKEN + b'\'' as HalfWord;
pub(crate) const HEX_TOKEN: HalfWord = OTHER_TOKEN + b'\"' as HalfWord;
pub(crate) const ALPHA_TOKEN: HalfWord = OTHER_TOKEN + b'`' as HalfWord;
pub(crate) const POINT_TOKEN: HalfWord = OTHER_TOKEN + b'.' as HalfWord;
pub(crate) const CONTINENTAL_POINT_TOKEN: HalfWord = OTHER_TOKEN + b',' as HalfWord;

// Section 445
// pub(crate) const INFINITY: Integer = 0x7fff_ffff;
pub(crate) const ZERO_TOKEN: HalfWord = OTHER_TOKEN + b'0' as HalfWord;
pub(crate) const A_TOKEN: HalfWord = LETTER_TOKEN + b'A' as HalfWord;
pub(crate) const OTHER_A_TOKEN: HalfWord = OTHER_TOKEN + b'A' as HalfWord;

// Section 463
pub(crate) const DEFAULT_RULE: Scaled = 26214;

// Part 27: Building token lists
// Section 468
pub(crate) const NUMBER_CODE: HalfWord = 0;
pub(crate) const ROMAN_NUMERAL_CODE: HalfWord = 1;
pub(crate) const STRING_CODE: HalfWord = 2;
pub(crate) const MEANING_CODE: HalfWord = 3;
pub(crate) const FONT_NAME_CODE: HalfWord = 4;
pub(crate) const JOB_NAME_CODE: HalfWord = 5;

// Section 480
pub(crate) const CLOSED: usize = 2;
pub(crate) const JUST_OPEN: usize = 1;

// Part 28: Conditional processing
// Section 487
pub(crate) const IF_CHAR_CODE: HalfWord = 0;
pub(crate) const IF_CAT_CODE: HalfWord = 1;
pub(crate) const IF_INT_CODE: HalfWord = 2;
pub(crate) const IF_DIM_CODE: HalfWord = 3;
pub(crate) const IF_ODD_CODE: HalfWord = 4;
pub(crate) const IF_VMODE_CODE: HalfWord = 5;
pub(crate) const IF_HMODE_CODE: HalfWord = 6;
pub(crate) const IF_MMODE_CODE: HalfWord = 7;
pub(crate) const IF_INNER_CODE: HalfWord = 8;
pub(crate) const IF_VOID_CODE: HalfWord = 9;
pub(crate) const IF_HBOX_CODE: HalfWord = 10;
pub(crate) const IF_VBOX_CODE: HalfWord = 11;
pub(crate) const IFX_CODE: HalfWord = 12;
pub(crate) const IF_EOF_CODE: HalfWord = 13;
pub(crate) const IF_TRUE_CODE: HalfWord = 14;
pub(crate) const IF_FALSE_CODE: HalfWord = 15;
pub(crate) const IF_CASE_CODE: HalfWord = 16;

// Section 489
pub(crate) const IF_NODE_SIZE: Integer = 2;
pub(crate) const IF_CODE: HalfWord = 1;
pub(crate) const FI_CODE: HalfWord = 2;
pub(crate) const ELSE_CODE: HalfWord = 3;
pub(crate) const OR_CODE: HalfWord = 4;

// Part 30: Font metric data
// Section 544
// pub(crate) const NO_TAG: QuarterWord = 0;
pub(crate) const LIG_TAG: QuarterWord = 1;
pub(crate) const LIST_TAG: QuarterWord = 2;
pub(crate) const EXT_TAG: QuarterWord = 3;

// Section 545
pub(crate) const STOP_FLAG: QuarterWord = 128;
pub(crate) const KERN_FLAG: QuarterWord = 128;

// Section 547
pub(crate) const SLANT_CODE: Integer = 1;
pub(crate) const SPACE_CODE: Integer = 2;
// pub(crate) const SPACE_STRETCH_CODE: Integer = 3;
pub(crate) const SPACE_SHRINK_CODE: Integer = 4;
pub(crate) const X_HEIGHT_CODE: Integer = 5;
pub(crate) const QUAD_CODE: Integer = 6;
pub(crate) const EXTRA_SPACE_CODE: Integer = 7;

// Section 549
pub(crate) const NON_CHAR: HalfWord = 256;
pub(crate) const NON_ADDRESS: HalfWord = 0;

// Section 557
pub(crate) const KERN_BASE_OFFSET: Integer = 256*128; // 256*(128 + MIN_QUARTERWORD)

// Part 31: Device-independent file format
// Section 586
// pub(crate) const SET_CHAR_0: usize = 0;
pub(crate) const SET1: u8 = 128;
pub(crate) const SET_RULE: u8 = 132;
pub(crate) const PUT_RULE: u8 = 137;
// pub(crate) const NOP: usize = 138;
pub(crate) const BOP: u8 = 139;
pub(crate) const EOP: u8 = 140;
pub(crate) const PUSH: u8 = 141;
pub(crate) const POP: u8 = 142;
pub(crate) const RIGHT1: u8 = 143;
// pub(crate) const W0: usize = 147;
// pub(crate) const W1: usize = 148;
// pub(crate) const X0: usize = 152;
// pub(crate) const X1: usize = 153;
pub(crate) const DOWN1: u8 = 157;
pub(crate) const Y0: u8 = 161;
pub(crate) const Y1: u8 = 162;
pub(crate) const Z0: u8 = 166;
pub(crate) const Z1: u8 = 167;
pub(crate) const FNT_NUM_0: u8 = 171;
pub(crate) const FNT1: u8 = 235;
pub(crate) const XXX1: u8 = 239;
pub(crate) const XXX4: u8 = 242;
pub(crate) const FNT_DEF1: u8 = 243;
pub(crate) const PRE: u8 = 247;
pub(crate) const POST: u8 = 248;
pub(crate) const POST_POST: u8 = 249;

// Section 587
pub(crate) const ID_BYTE: u8 = 2;

// Part 32: Shipping pages out
// Section 605
pub(crate) const MOVEMENT_NODE_SIZE: Integer = 3;

// Section 608
pub(crate) const Y_HERE: Integer = 1;
pub(crate) const Z_HERE: Integer = 2;
pub(crate) const YZ_OK: Integer = 3;
pub(crate) const Y_OK: Integer = 4;
pub(crate) const Z_OK: Integer = 5;
pub(crate) const D_FIXED: Integer = 6;

// Section 611
pub(crate) const NONE_SEEN: Integer = 0;
pub(crate) const Y_SEEN: Integer = 6;
pub(crate) const Z_SEEN: Integer = 12;

// Part 33: Packaging
// Section 644
pub(crate) const EXACTLY: QuarterWord = 0;
pub(crate) const ADDITIONAL: QuarterWord = 1;

// Part 34: Data structures for math mode
// Section 681
pub(crate) const NOAD_SIZE: Integer = 4;
pub(crate) const MATH_CHAR: HalfWord = 1;
pub(crate) const SUB_BOX: HalfWord = 2;
pub(crate) const SUB_MLIST: HalfWord = 3;
pub(crate) const MATH_TEXT_CHAR: HalfWord = 4;

// Section 682
pub(crate) const ORD_NOAD: QuarterWord = UNSET_NODE + 3;
pub(crate) const OP_NOAD: QuarterWord = ORD_NOAD + 1;
pub(crate) const BIN_NOAD: QuarterWord = ORD_NOAD + 2;
pub(crate) const REL_NOAD: QuarterWord = ORD_NOAD + 3;
pub(crate) const OPEN_NOAD: QuarterWord = ORD_NOAD + 4;
pub(crate) const CLOSE_NOAD: QuarterWord = ORD_NOAD + 5;
pub(crate) const PUNCT_NOAD: QuarterWord = ORD_NOAD + 6;
pub(crate) const INNER_NOAD: QuarterWord = ORD_NOAD + 7;
pub(crate) const LIMITS: QuarterWord = 1;
pub(crate) const NO_LIMITS: QuarterWord = 2;

// Section 683
pub(crate) const RADICAL_NOAD: QuarterWord = INNER_NOAD + 1;
pub(crate) const RADICAL_NOAD_SIZE: Integer = 5;
pub(crate) const FRACTION_NOAD: QuarterWord = RADICAL_NOAD + 1;
pub(crate) const FRACTION_NOAD_SIZE: Integer = 6;
pub(crate) const DEFAULT_CODE: Scaled = 1 << 30;

// Section 687
pub(crate) const UNDER_NOAD: QuarterWord = FRACTION_NOAD + 1;
pub(crate) const OVER_NOAD: QuarterWord = UNDER_NOAD + 1;
pub(crate) const ACCENT_NOAD: QuarterWord = OVER_NOAD + 1;
pub(crate) const ACCENT_NOAD_SIZE: Integer = 5;
pub(crate) const VCENTER_NOAD: QuarterWord = ACCENT_NOAD + 1;
pub(crate) const LEFT_NOAD: QuarterWord = VCENTER_NOAD + 1;
pub(crate) const RIGHT_NOAD: QuarterWord = LEFT_NOAD + 1;

// Section 688
pub(crate) const STYLE_NODE: QuarterWord = UNSET_NODE + 1;
pub(crate) const STYLE_NODE_SIZE: Integer = 3;
pub(crate) const DISPLAY_STYLE: QuarterWord = 0;
pub(crate) const TEXT_STYLE: QuarterWord = 2;
pub(crate) const SCRIPT_STYLE: QuarterWord = 4;
pub(crate) const SCRIPT_SCRIPT_STYLE: QuarterWord = 6;
pub(crate) const CRAMPED: QuarterWord = 1;

// Section 689
pub(crate) const CHOICE_NODE: QuarterWord = UNSET_NODE + 2;

// Part 35: Subroutines for math mode
// Section 699
pub(crate) const TEXT_SIZE: QuarterWord = 0;
pub(crate) const SCRIPT_SIZE: QuarterWord = 16;
pub(crate) const SCRIPT_SCRIPT_SIZE: QuarterWord = 32;

// Section 700
pub(crate) const TOTAL_MATHSY_PARAMS: usize = 22;

// Section 701
pub(crate) const TOTAL_MATHEX_PARAMS: usize = 13;

// Part 36: Typesetting math formulas
// Section 764
pub(crate) const MATH_SPACING: &[u8; 64] = b"0234000122*4000133**3**344*0400400*000000234000111*1111112341011";

// Part 37: Alignment
// Section 770
pub(crate) const ALIGN_STACK_NODE_SIZE: Integer = 5;

// Section 780
pub(crate) const SPAN_CODE: HalfWord = 256;
pub(crate) const CR_CODE: HalfWord = 257;
pub(crate) const CR_CR_CODE: HalfWord = CR_CODE + 1;
pub(crate) const END_TEMPLATE_TOKEN: HalfWord = CS_TOKEN_FLAG + FROZEN_END_TEMPLATE;

// Section 797
pub(crate) const SPAN_NODE_SIZE: Integer = 2;

// Part 38: Breaking paragraphs into lines
// Section 817
pub(crate) const TIGHT_FIT: usize = 3;
pub(crate) const LOOSE_FIT: usize = 1;
pub(crate) const VERY_LOOSE_FIT: usize = 0;
pub(crate) const DECENT_FIT: usize = 2;

// Section 819
pub(crate) const ACTIVE_NODE_SIZE: Integer = 3;
pub(crate) const UNHYPHENATED: QuarterWord = 0;
pub(crate) const HYPHENATED: QuarterWord = 1;
pub(crate) const LAST_ACTIVE: HalfWord = ACTIVE;

// Section 821
pub(crate) const PASSIVE_NODE_SIZE: Integer = 2;

// Section 822
pub(crate) const DELTA_NODE_SIZE: Integer = 7;
pub(crate) const DELTA_NODE: QuarterWord = 2;

// Section 833
pub(crate) const AWFUL_BAD: Integer = 0x3fff_ffff;

// Part 44: Breaking vertical lists into pages
// Section 974
pub(crate) const DEPLORABLE: Integer = 100_000;

// Part 45: The page builder
// Section 980
pub(crate) const INSERTS_ONLY: SmallNumber = 1;
pub(crate) const BOX_THERE: SmallNumber = 2;

// Section 981
pub(crate) const PAGE_INS_NODE_SIZE: Integer = 4;
pub(crate) const INSERTING: SmallNumber = 0;
pub(crate) const SPLIT_UP: QuarterWord = 1;

// Part 47: Building boxes and lists
// Section 1058
pub(crate) const FIL_CODE: HalfWord = 0;
pub(crate) const FILL_CODE: HalfWord = 1;
pub(crate) const SS_CODE: HalfWord = 2;
pub(crate) const FIL_NEG_CODE: HalfWord = 3;
pub(crate) const SKIP_CODE: HalfWord = 4;
pub(crate) const MSKIP_CODE: HalfWord = 5;

// Section 1071
pub(crate) const BOX_FLAG: Integer = 0x4000_0000;
pub(crate) const SHIP_OUT_FLAG: Integer = BOX_FLAG + 512;
pub(crate) const LEADER_FLAG: Integer = BOX_FLAG + 513;
pub(crate) const BOX_CODE: HalfWord = 0;
pub(crate) const COPY_CODE: HalfWord = 1;
pub(crate) const LAST_BOX_CODE: HalfWord = 2;
pub(crate) const VSPLIT_CODE: HalfWord = 3;
pub(crate) const VTOP_CODE: HalfWord = 4;
pub(crate) const VTOP_CODE_PLUS_VMODE: HalfWord = VTOP_CODE + VMODE;

// Part 48: Building math lists
// Section 1178
pub(crate) const ABOVE_CODE: HalfWord = 0;
pub(crate) const OVER_CODE: HalfWord = 1;
pub(crate) const ATOP_CODE: HalfWord = 2;
pub(crate) const DELIMITED_CODE: HalfWord = 3;
pub(crate) const DELIMITED_ABOVE_CODE: HalfWord = DELIMITED_CODE + ABOVE_CODE;
pub(crate) const DELIMITED_OVER_CODE: HalfWord = DELIMITED_CODE + OVER_CODE;
pub(crate) const DELIMITED_ATOP_CODE: HalfWord = DELIMITED_CODE + ATOP_CODE;

// Part 49: Mode-independent processing
// Section 1222
pub(crate) const CHAR_DEF_CODE: HalfWord = 0;
pub(crate) const MATH_CHAR_DEF_CODE: HalfWord = 1;
pub(crate) const COUNT_DEF_CODE: HalfWord = 2;
pub(crate) const DIMEN_DEF_CODE: HalfWord = 3;
pub(crate) const SKIP_DEF_CODE: HalfWord = 4;
pub(crate) const MU_SKIP_DEF_CODE: HalfWord = 5;
pub(crate) const TOKS_DEF_CODE: HalfWord = 6;

// Section 1291
pub(crate) const SHOW_CODE: HalfWord = 0;
pub(crate) const SHOW_BOX_CODE: HalfWord = 1;
pub(crate) const SHOW_THE_CODE: HalfWord = 2;
pub(crate) const SHOW_LISTS: HalfWord = 3;

// Part 53: Extensions
// Section 1341
pub(crate) const WRITE_NODE_SIZE: Integer = 2;
pub(crate) const OPEN_NODE_SIZE: Integer = 3;
pub(crate) const OPEN_NODE: Integer = 0;
pub(crate) const WRITE_NODE: Integer = 1;
pub(crate) const CLOSE_NODE: Integer = 2;
pub(crate) const SPECIAL_NODE: Integer = 3;
pub(crate) const LANGUAGE_NODE: Integer = 4;

// Section 1344
pub(crate) const IMMEDIATE_CODE: HalfWord = 4;
pub(crate) const SET_LANGUAGE_CODE: HalfWord = 5;

// Section 1371
pub(crate) const END_WRITE_TOKEN: HalfWord = CS_TOKEN_FLAG + END_WRITE;
