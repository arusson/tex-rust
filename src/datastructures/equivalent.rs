use crate::constants::*;
use crate::datastructures::MemoryWord;
use crate::{
    HalfWord, Integer, QuarterWord, Scaled
};
use std::ops::{Index, IndexMut};

// Part 17: The table of equivalents

pub(crate) struct Eqtb([MemoryWord; (EQTB_SIZE - ACTIVE_BASE + 1) as usize]);
pub(crate) static mut EQTB: Eqtb = Eqtb([MemoryWord::ZERO; (EQTB_SIZE - ACTIVE_BASE + 1) as usize]);

impl Index<usize> for Eqtb {
    type Output = MemoryWord;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - ACTIVE_BASE as usize]
    }
}

impl IndexMut<usize> for Eqtb {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - ACTIVE_BASE as usize]
    }
}

#[macro_export]
macro_rules! eqtb {
    ($p:expr) => [
        unsafe { EQTB[$p] }
    ];
}

#[macro_export]
macro_rules! eqtb_mut {
    ($p:expr) => [
        unsafe { &mut EQTB[$p] }
    ];
}

pub(crate) struct XeqLevel([QuarterWord; (EQTB_SIZE - INT_BASE + 1) as usize]);
pub(crate) static mut XEQ_LEVEL: XeqLevel = XeqLevel([0; (EQTB_SIZE - INT_BASE + 1) as usize]);

impl Index<usize> for XeqLevel {
    type Output = QuarterWord;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - INT_BASE as usize]
    }
}

impl IndexMut<usize> for XeqLevel {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - INT_BASE as usize]
    }
}

impl MemoryWord {
    // Section 221
    fn eq_level_field(&self) -> QuarterWord {
        self.hh_b1()
    }

    fn eq_level_field_mut(&mut self) -> &mut QuarterWord {
        self.hh_b1_mut()
    }

    pub(crate) fn eq_type_field(&self) -> QuarterWord {
        self.hh_b0()
    }

    fn eq_type_field_mut(&mut self) -> &mut QuarterWord {
        self.hh_b0_mut()
    }

    pub(crate) fn equiv_field(&self) -> HalfWord {
        self.hh_rh()
    }

    fn equiv_field_mut(&mut self) -> &mut HalfWord {
        self.hh_rh_mut()
    }

}

// Section 224
macro_rules! section_224_glue_par {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> HalfWord {
            glue_par($offset)
        }
    };
}

macro_rules! section_224_glue_par_mut {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> &'static mut HalfWord {
            glue_par_mut($offset)
        }
    };
}

// Section 230
macro_rules! section_230_equiv_fixed {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> HalfWord {
           equiv($offset)
        }
    };
}

macro_rules! section_230_equiv_fixed_mut {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> &'static mut HalfWord {
            equiv_mut($offset)
         }
    };
}

macro_rules! section_230_equiv_var {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name(p: HalfWord) -> HalfWord {
           equiv($offset + p)
        }
    };
}

macro_rules! section_230_equiv_var_mut {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name(p: HalfWord) -> &'static mut HalfWord {
            equiv_mut($offset + p)
         }
    };
}

// Section 236
macro_rules! section_236_int_par {
    ($fn_name:ident, $offset:expr) => {
        pub fn $fn_name() -> Integer {
            int_par($offset)
        }
    }
}

macro_rules! section_236_int_par_mut {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> &'static mut Integer {
            int_par_mut($offset)
        }
    }
}

// Section 247
macro_rules! section_247_dimen_par {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> Scaled {
            dimen_par($offset)
        }
    };
}

macro_rules! section_247_dimen_par_mut {
    ($fn_name:ident, $offset:expr) => {
        pub(crate) fn $fn_name() -> &'static mut Scaled {
            dimen_par_mut($offset)
        }
    };
}

// Section 221
pub(crate) fn eq_level(p: HalfWord) -> QuarterWord {
    eqtb![p as usize].eq_level_field()
}

pub(crate) fn eq_level_mut(p: HalfWord) -> &'static mut QuarterWord {
    unsafe { EQTB[p as usize].eq_level_field_mut() }
}

pub(crate) fn eq_type(p: HalfWord) -> QuarterWord {
    eqtb![p as usize].eq_type_field()
}

pub(crate) fn eq_type_mut(p: HalfWord) -> &'static mut QuarterWord {
    unsafe { EQTB[p as usize].eq_type_field_mut() }
}

pub(crate) fn equiv(p: HalfWord) -> HalfWord {
    eqtb![p as usize].equiv_field()
}

pub(crate) fn equiv_mut(p: HalfWord) -> &'static mut HalfWord {
    unsafe { EQTB[p as usize].equiv_field_mut() }
}

// Section 224
pub(crate) fn skip(p: HalfWord) -> HalfWord {
    equiv(p + SKIP_BASE)
}

pub(crate) fn mu_skip(p: HalfWord) -> HalfWord {
    equiv(p + MU_SKIP_BASE)
}

pub(crate) fn glue_par(p: HalfWord) -> HalfWord {
    equiv(p + GLUE_BASE)
}

pub(crate) fn glue_par_mut(p: HalfWord) -> &'static mut HalfWord {
    equiv_mut(p + GLUE_BASE)
}

section_224_glue_par!(baseline_skip, BASELINE_SKIP_CODE);
section_224_glue_par!(left_skip, LEFT_SKIP_CODE);
section_224_glue_par!(right_skip, RIGHT_SKIP_CODE);
section_224_glue_par!(split_top_skip, SPLIT_TOP_SKIP_CODE);
section_224_glue_par!(space_skip, SPACE_SKIP_CODE);
section_224_glue_par!(xspace_skip, XSPACE_SKIP_CODE);
section_224_glue_par_mut!(split_top_skip_mut, SPLIT_TOP_SKIP_CODE);

// Section 230
section_230_equiv_fixed!(par_shape_ptr, PAR_SHAPE_LOC);
section_230_equiv_fixed!(output_routine, OUTPUT_ROUTINE_LOC);
section_230_equiv_fixed!(every_par, EVERY_PAR_LOC);
section_230_equiv_fixed!(every_math, EVERY_MATH_LOC);
section_230_equiv_fixed!(every_display, EVERY_DISPLAY_LOC);
section_230_equiv_fixed!(every_hbox, EVERY_HBOX_LOC);
section_230_equiv_fixed!(every_vbox, EVERY_VBOX_LOC);
section_230_equiv_fixed!(every_job, EVERY_JOB_LOC);
section_230_equiv_fixed!(every_cr, EVERY_CR_LOC);
section_230_equiv_fixed_mut!(par_shape_ptr_mut, PAR_SHAPE_LOC);
section_230_equiv_var!(r#box, BOX_BASE);
section_230_equiv_fixed!(cur_font, CUR_FONT_LOC);
section_230_equiv_var!(fam_fnt, MATH_FONT_BASE);
section_230_equiv_var!(cat_code, CAT_CODE_BASE);
section_230_equiv_var!(lc_code, LC_CODE_BASE);
section_230_equiv_var!(sf_code, SF_CODE_BASE);
section_230_equiv_var!(math_code, MATH_CODE_BASE);
section_230_equiv_var_mut!(box_mut, BOX_BASE);
section_230_equiv_fixed_mut!(cur_font_mut, CUR_FONT_LOC);
section_230_equiv_var_mut!(cat_code_mut, CAT_CODE_BASE);
section_230_equiv_var_mut!(lc_code_mut, LC_CODE_BASE);
section_230_equiv_var_mut!(uc_code_mut, UC_CODE_BASE);
section_230_equiv_var_mut!(sf_code_mut, SF_CODE_BASE);
section_230_equiv_var_mut!(math_code_mut, MATH_CODE_BASE);

// Section 236
pub(crate) fn del_code(p: HalfWord) -> Integer {
    eqtb![(DEL_CODE_BASE + p) as usize].int()
}

pub(crate) fn del_code_mut(p: HalfWord) -> &'static mut Integer {
    eqtb_mut![(DEL_CODE_BASE + p) as usize].int_mut()
}

pub(crate) fn count(p: HalfWord) -> Integer {
    eqtb![(COUNT_BASE + p) as usize].int()
}

fn int_par(p: HalfWord) -> Integer {
    eqtb![(INT_BASE + p) as usize].int()
}

fn int_par_mut(p: HalfWord) -> &'static mut Integer {
    eqtb_mut![(INT_BASE + p) as usize].int_mut()
}

section_236_int_par!(pretolerance, PRETOLERANCE_CODE);
section_236_int_par!(tolerance, TOLERANCE_CODE);
section_236_int_par!(line_penalty, LINE_PENALTY_CODE);
section_236_int_par!(hyphen_penalty, HYPHEN_PENALTY_CODE);
section_236_int_par!(ex_hyphen_penalty, EX_HYPHEN_PENALTY_CODE);
section_236_int_par!(club_penalty, CLUB_PENALTY_CODE);
section_236_int_par!(widow_penalty, WIDOW_PENALTY_CODE);
section_236_int_par!(display_widow_penalty, DISPLAY_WIDOW_PENALTY_CODE);
section_236_int_par!(broken_penalty, BROKEN_PENALTY_CODE);
section_236_int_par!(bin_op_penalty, BIN_OP_PENALTY_CODE);
section_236_int_par!(rel_penalty, REL_PENALTY_CODE);
section_236_int_par!(pre_display_penalty, PRE_DISPLAY_PENALTY_CODE);
section_236_int_par!(post_display_penalty, POST_DISPLAY_PENALTY_CODE);
section_236_int_par!(inter_line_penalty, INTER_LINE_PENALTY_CODE);
section_236_int_par!(double_hyphen_demerits, DOUBLE_HYPHEN_DEMERITS_CODE);
section_236_int_par!(final_hyphen_demerits, FINAL_HYPHEN_DEMERITS_CODE);
section_236_int_par!(adj_demerits, ADJ_DEMERITS_CODE);
section_236_int_par!(mag, MAG_CODE);
section_236_int_par!(delimiter_factor, DELIMITER_FACTOR_CODE);
section_236_int_par!(looseness, LOOSENESS_CODE);
section_236_int_par!(time, TIME_CODE);
section_236_int_par!(day, DAY_CODE);
section_236_int_par!(month, MONTH_CODE);
section_236_int_par!(year, YEAR_CODE);
section_236_int_par!(show_box_breadth, SHOW_BOX_BREADTH_CODE);
section_236_int_par!(show_box_depth, SHOW_BOX_DEPTH_CODE);
section_236_int_par!(hbadness, HBADNESS_CODE);
section_236_int_par!(vbadness, VBADNESS_CODE);
section_236_int_par!(pausing, PAUSING_CODE);
section_236_int_par!(tracing_online, TRACING_ONLINE_CODE);
section_236_int_par!(tracing_macros, TRACING_MACROS_CODE);
#[cfg(feature = "stat")]
section_236_int_par!(tracing_stats, TRACING_STATS_CODE);
#[cfg(feature = "stat")]
section_236_int_par!(tracing_paragraphs, TRACING_PARAGRAPHS_CODE);
#[cfg(feature = "stat")]
section_236_int_par!(tracing_pages, TRACING_PAGES_CODE);
section_236_int_par!(tracing_output, TRACING_OUTPUT_CODE);
section_236_int_par!(tracing_lost_chars, TRACING_LOST_CHARS_CODE);
section_236_int_par!(tracing_commands, TRACING_COMMANDS_CODE);
#[cfg(feature = "stat")]
section_236_int_par!(tracing_restores, TRACING_RESTORES_CODE);
section_236_int_par!(uc_hyph, UC_HYPH_CODE);
section_236_int_par!(max_dead_cycles, MAX_DEAD_CYCLES_CODE);
section_236_int_par!(hang_after, HANG_AFTER_CODE);
section_236_int_par!(floating_penalty, FLOATING_PENALTY_CODE);
section_236_int_par!(global_defs, GLOBAL_DEFS_CODE);
section_236_int_par!(cur_fam, CUR_FAM_CODE);
section_236_int_par!(escape_char, ESCAPE_CHAR_CODE);
section_236_int_par!(default_hyphen_char, DEFAULT_HYPHEN_CHAR_CODE);
section_236_int_par!(default_skew_char, DEFAULT_SKEW_CHAR_CODE);
section_236_int_par!(end_line_char, END_LINE_CHAR_CODE);
section_236_int_par!(new_line_char, NEW_LINE_CHAR_CODE);
section_236_int_par!(language, LANGUAGE_CODE);
section_236_int_par!(left_hyphen_min, LEFT_HYPHEN_MIN_CODE);
section_236_int_par!(right_hyphen_min, RIGHT_HYPHEN_MIN_CODE);
section_236_int_par!(holding_inserts, HOLDING_INSERTS_CODE);
section_236_int_par!(error_context_lines, ERROR_CONTEXT_LINES_CODE);

section_236_int_par_mut!(tolerance_mut, TOLERANCE_CODE);
section_236_int_par_mut!(mag_mut, MAG_CODE);
section_236_int_par_mut!(time_mut, TIME_CODE);
section_236_int_par_mut!(day_mut, DAY_CODE);
section_236_int_par_mut!(month_mut, MONTH_CODE);
section_236_int_par_mut!(year_mut, YEAR_CODE);
section_236_int_par_mut!(vbadness_mut, VBADNESS_CODE);
section_236_int_par_mut!(tracing_stats_mut, TRACING_STATS_CODE);
section_236_int_par_mut!(max_dead_cycles_mut, MAX_DEAD_CYCLES_CODE);
section_236_int_par_mut!(hang_after_mut, HANG_AFTER_CODE);
section_236_int_par_mut!(escape_char_mut, ESCAPE_CHAR_CODE);
section_236_int_par_mut!(end_line_char_mut, END_LINE_CHAR_CODE);
section_236_int_par_mut!(new_line_char_mut, NEW_LINE_CHAR_CODE);

// Section 247
pub(crate) fn dimen(p: HalfWord) -> Scaled {
    eqtb![(SCALED_BASE + p) as usize].sc()
}

fn dimen_par(p: HalfWord) -> Scaled {
    eqtb![(DIMEN_BASE + p) as usize].sc()
}

fn dimen_par_mut(p: HalfWord) -> &'static mut Scaled {
    eqtb_mut![(DIMEN_BASE + p) as usize].sc_mut()
}

section_247_dimen_par!(par_indent, PAR_INDENT_CODE);
section_247_dimen_par!(math_surround, MATH_SURROUND_CODE);
section_247_dimen_par!(line_skip_limit, LINE_SKIP_LIMIT_CODE);
section_247_dimen_par!(hsize, HSIZE_CODE);
section_247_dimen_par!(vsize, VSIZE_CODE);
section_247_dimen_par!(max_depth, MAX_DEPTH_CODE);
section_247_dimen_par!(split_max_depth, SPLIT_MAX_DEPTH_CODE);
section_247_dimen_par!(box_max_depth, BOX_MAX_DEPTH_CODE);
section_247_dimen_par!(hfuzz, HFUZZ_CODE);
section_247_dimen_par!(vfuzz, VFUZZ_CODE);
section_247_dimen_par!(delimiter_shortfall, DELIMITER_SHORTFALL_CODE);
section_247_dimen_par!(null_delimiter_space, NULL_DELIMITER_SPACE_CODE);
section_247_dimen_par!(script_space, SCRIPT_SPACE_CODE);
section_247_dimen_par!(pre_display_size, PRE_DISPLAY_SIZE_CODE);
section_247_dimen_par!(display_width, DISPLAY_WIDTH_CODE);
section_247_dimen_par!(display_indent, DISPLAY_INDENT_CODE);
section_247_dimen_par!(overfull_rule, OVERFULL_RULE_CODE);
section_247_dimen_par!(hang_indent, HANG_INDENT_CODE);
section_247_dimen_par!(h_offset, H_OFFSET_CODE);
section_247_dimen_par!(v_offset, V_OFFSET_CODE);
section_247_dimen_par!(emergency_stretch, EMERGENCY_STRETCH_CODE);

section_247_dimen_par_mut!(vfuzz_mut, VFUZZ_CODE);
section_247_dimen_par_mut!(overfull_rule_mut, OVERFULL_RULE_CODE);
