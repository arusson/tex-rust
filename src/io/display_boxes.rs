use crate::constants::*;
use crate::datastructures::{
    adjust_ptr, character, depth, float_cost, font, font_id_text, glue_order,
    glue_ptr, glue_set, glue_shrink, glue_sign, glue_stretch, height, ins_ptr,
    leader_ptr, lig_ptr, link, list_ptr, mark_ptr, penalty, post_break,
    pre_break, replace_count, shift_amount, show_box_breadth, show_box_depth,
    shrink, shrink_order, span_count, split_top_ptr, stretch, stretch_order,
    subtype, r#type, width
};
use crate::extensions::{
    open_area, open_ext, open_name, what_lang, what_lhm, what_rhm, write_tokens
};
use crate::math::{
    display_mlist, script_mlist, script_script_mlist, text_mlist
};
use crate::strings::{
    append_char, cur_length, flush_char, pool_ptr
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, Real, Scaled, StrNum,
    is_running, lig_char, odd
};

use std::cmp::Ordering::{Equal, Greater, Less};

// Part 12: Displaying boxes

impl Global {
    // Section 174
    pub(crate) fn short_display(&mut self, mut p: Integer) {
        while p > MEM_MIN {
            if self.is_char_node(p) {
                if p <= self.mem_end {
                    if font(p) != self.font_in_short_display {
                        if font(p) < FONT_BASE as QuarterWord
                            || font(p) > FONT_MAX as QuarterWord
                        {
                            self.print_char(b'*');
                        }
                        else {
                            // Section 267
                            self.print_esc_strnumber(font_id_text(font(p)) as StrNum);
                            // End section 267
                        }
                        self.print_char(b' ');
                        self.font_in_short_display = font(p);
                    }
                    self.print_strnumber(character(p) as StrNum);
                }
            }
            else {
                // Section 175
                match r#type(p) {
                    HLIST_NODE
                    | VLIST_NODE
                    | INS_NODE
                    | WHATSIT_NODE
                    | MARK_NODE
                    | ADJUST_NODE
                    | UNSET_NODE => self.print("[]"),

                    RULE_NODE => self.print_char(b'|'),

                    GLUE_NODE => {
                        if glue_ptr(p) != ZERO_GLUE {
                            self.print_char(b' ');
                        }
                    },

                    MATH_NODE => self.print_char(b'$'),

                    LIGATURE_NODE => self.short_display(lig_ptr(p)),

                    DISC_NODE => {
                        self.short_display(pre_break(p));
                        self.short_display(post_break(p));
                        let mut n = replace_count(p);
                        while n > 0 {
                            if link(p) != NULL {
                                p = link(p);
                            }
                            n -= 1;
                        }
                    },

                    _ => ()
                }
                // End section 175
            }
            p = link(p);
        }
    }

    // Section 176
    pub(crate) fn print_font_and_char(&mut self, p: Integer) {
        if p > self.mem_end {
            self.print_esc("CLOBBERED.")
        }
        else {
            if font(p) < FONT_BASE as QuarterWord || font(p) > FONT_MAX as QuarterWord {
                self.print_char(b'*');
            }
            else {
                // Section 267
                self.print_esc_strnumber(font_id_text(font(p)) as StrNum);
                // End section 267
            }
            self.print_char(b' ');
            self.print_strnumber(character(p) as StrNum);
        }
    }

    pub(crate) fn print_mark(&mut self, p: Integer) {
        self.print_char(b'{');
        if p < self.hi_mem_min || p > self.mem_end {
            self.print_esc("CLOBBERED.");
        }
        else {
            self.show_token_list(link(p), NULL, MAX_PRINT_LINE - 10);
        }
        self.print_char(b'}');
    }

    pub(crate) fn print_rule_dimen(&mut self, d: Scaled) {
        if is_running!(d) {
            self.print_char(b'*');
        }
        else {
            self.print_scaled(d);
        }
    }

    // Section 177
    pub(crate) fn print_glue(&mut self, d: Scaled, mut order: Integer, s: &str) {
        self.print_scaled(d);
        if order < (NORMAL as Integer) || order > FILLL as Integer {
            self.print("foul");
        }
        else if order > (NORMAL as Integer) {
            self.print("fil");
            while order > (FIL as Integer) {
                self.print_char(b'l');
                order -= 1;
            }
        }
        else if !s.is_empty() {
            self.print(s);
        }
    }

    // Section 178
    pub(crate) fn print_spec(&mut self, p: Integer, s: &str) {
        if p < MEM_MIN || p >= self.lo_mem_max {
            self.print_char(b'*');
        }
        else {
            self.print_scaled(width(p));
            if !s.is_empty() {
                self.print(s);
            }
            if stretch(p) != 0 {
                self.print(" plus ");
                self.print_glue(stretch(p), stretch_order(p) as Integer, s);
            }
            if shrink(p) != 0 {
                self.print(" minus ");
                self.print_glue(shrink(p), shrink_order(p) as Integer, s);
            }
        }
    }
}

// Section 180
#[macro_export]
macro_rules! node_list_display {
    ($s:ident, $p:expr) => {
        append_char(b'.');
        $s.show_node_list($p);
        flush_char();
    };
}
impl Global {
    // Section 182
    pub(crate) fn show_node_list(&mut self, mut p: Integer) {
        if cur_length() as Integer > self.depth_threshold {
            if p > NULL {
                self.print(" []");
            }
            return;
        }
        let mut n = 0;
        while p > MEM_MIN {
            self.print_ln();
            self.print_current_string();
            if p > self.mem_end {
                self.print("Bad link, display aborted.");
                return;
            }
            n += 1;
            if n > self.breadth_max {
                self.print("etc.");
                return;
            }
            self.sec183_display_node(p);
            p = link(p);
        }
    }

    // Section 183
    fn sec183_display_node(&mut self, p: Integer) {
        if self.is_char_node(p) {
            self.print_font_and_char(p);
        }
        else {
            match r#type(p) {
                HLIST_NODE
                | VLIST_NODE
                | UNSET_NODE => self.sec184_display_box(p),

                RULE_NODE => {
                    // Section 187
                    self.print_esc("rule(");
                    self.print_rule_dimen(height(p));
                    self.print_char(b'+');
                    self.print_rule_dimen(depth(p));
                    self.print(")x");
                    self.print_rule_dimen(width(p));
                    // End section 187
                },  

                INS_NODE => {
                    // Section 188
                    self.print_esc("insert");
                    self.print_int(subtype(p) as Integer);
                    self.print(", natural size ");
                    self.print_scaled(height(p));
                    self.print("; split(");
                    self.print_spec(split_top_ptr(p), "");
                    self.print_char(b',');
                    self.print_scaled(depth(p));
                    self.print("); float cost ");
                    self.print_int(float_cost(p));
                    node_list_display!(self, ins_ptr(p));
                    // End section 188
                },

                WHATSIT_NODE => {
                    // Section 1356
                    match subtype(p) as Integer {
                        OPEN_NODE => {
                            self.print_write_whatsit("openout", p);
                            self.print_char(b'=');
                            self.print_file_name(open_name(p) as StrNum, open_area(p) as StrNum, open_ext(p) as StrNum);
                        },

                        WRITE_NODE => {
                            self.print_write_whatsit("write", p);
                            self.print_mark(write_tokens(p));
                        },

                        CLOSE_NODE => self.print_write_whatsit("closeout", p),

                        SPECIAL_NODE => {
                            self.print_esc("special");
                            self.print_mark(write_tokens(p));
                        },

                        LANGUAGE_NODE => {
                            self.print_esc("setlanguage");
                            self.print_int(what_lang(p));
                            self.print(" (hyphenmin ");
                            self.print_int(what_lhm(p) as Integer);
                            self.print_char(b',');
                            self.print_int(what_rhm(p) as Integer);
                            self.print_char(b')');
                        }

                        _ => self.print("whatsit?"),
                    }
                    // End section 1356
                },

                GLUE_NODE => self.sec189_display_glue(p),

                KERN_NODE => {
                    // Section 191
                    if subtype(p) != MU_GLUE {
                        self.print_esc("kern");
                        if subtype(p) != NORMAL {
                            self.print_char(b' ');
                        }
                        self.print_scaled(width(p));
                        if subtype(p) == ACC_KERN {
                            self.print(" (for accent)");
                        }
                    }
                    else {
                        self.print_esc("mkern");
                        self.print_scaled(width(p));
                        self.print("mu");
                    }
                    // End section 191
                }

                MATH_NODE => {
                    // Section 192
                    self.print_esc("math");
                    if subtype(p) == BEFORE {
                        self.print("on");
                    }
                    else {
                        self.print("off");
                    }
                    if width(p) != 0 {
                        self.print(", surrounded ");
                        self.print_scaled(width(p));
                    }
                    // End section 192
                }

                LIGATURE_NODE => {
                    // Section 193
                    self.print_font_and_char(lig_char!(p));
                    self.print(" (ligature ");
                    if subtype(p) > 1 {
                        self.print_char(b'|');
                    }
                    self.font_in_short_display = font(lig_char!(p));
                    self.short_display(lig_ptr(p));
                    if odd!(subtype(p)) {
                        self.print_char(b'|');
                    }
                    self.print_char(b')');
                    // End section 193
                }

                PENALTY_NODE => {
                    // Section 194
                    self.print_esc("penalty ");
                    self.print_int(penalty(p));
                    // End section 194
                },

                DISC_NODE => {
                    // Section 195
                    self.print_esc("discretionary");
                    if replace_count(p) > 0 {
                        self.print_esc(" replacing ");
                        self.print_int(replace_count(p) as Integer);
                    }
                    node_list_display!(self, pre_break(p));
                    append_char(b'|');
                    self.show_node_list(post_break(p));
                    flush_char();
                    // End section 195
                }

                MARK_NODE => {
                    // Secion 196
                    self.print_esc("mark");
                    self.print_mark(mark_ptr(p));
                    // End section 196
                },

                ADJUST_NODE => {
                    // Section 197
                    self.print_esc("vadjust");
                    node_list_display!(self, adjust_ptr(p));
                    // End section 197
                },

                // Section 690
                STYLE_NODE => self.print_style(subtype(p) as Integer),

                CHOICE_NODE => {
                    // Section 695
                    self.print_esc("mathchoice");
                    append_char(b'D');
                    self.show_node_list(display_mlist(p));
                    flush_char();
                    append_char(b'T');
                    self.show_node_list(text_mlist(p));
                    flush_char();
                    append_char(b'S');
                    self.show_node_list(script_mlist(p));
                    flush_char();
                    append_char(b's');
                    self.show_node_list(script_script_mlist(p));
                    flush_char();
                    // End section 695
                },

                ORD_NOAD
                | OP_NOAD
                | BIN_NOAD
                | REL_NOAD
                | OPEN_NOAD
                | CLOSE_NOAD
                | PUNCT_NOAD
                | INNER_NOAD
                | RADICAL_NOAD
                | OVER_NOAD
                | UNDER_NOAD
                | VCENTER_NOAD
                | ACCENT_NOAD
                | LEFT_NOAD
                | RIGHT_NOAD => self.sec696_display_normal_noad(p),

                FRACTION_NOAD => self.sec697_display_fraction_noad(p),
                // End section 690

                _ => self.print("Unknown node type!"),
            }
        }
    }

    // Section 184
    fn sec184_display_box(&mut self, p: Integer) {
        match r#type(p) {
            HLIST_NODE => self.print_esc("h"),
            VLIST_NODE => self.print_esc("v"),
            _ /* Unset */ => self.print_esc("unset"),
        }
        self.print("box(");
        self.print_scaled(height(p));
        self.print_char(b'+');
        self.print_scaled(depth(p));
        self.print(")x");
        self.print_scaled(width(p));
        if r#type(p) == UNSET_NODE {
            // Secion 185
            if span_count(p) != MIN_QUARTERWORD {
                self.print(" (");
                self.print_int((span_count(p) + 1) as Integer);
                self.print(" columns)");
            }
            if glue_stretch(p) != 0 {
                self.print(", stretch ");
                self.print_glue(glue_stretch(p), glue_order(p) as Integer, "");
            }
            if glue_shrink(p) != 0 {
                self.print(", shrink ");
                self.print_glue(glue_shrink(p), glue_sign(p) as Integer, "");
            }
            // End section 185
        }
        else {
            // Section 186
            let g = glue_set(p);
            if g != 0.0 && glue_sign(p) != NORMAL {
                self.print(", glue set ");
                if glue_sign(p) == SHRINKING {
                    self.print("- ");
                }
                if g.abs() > 20_000.0 {
                    if g > 0.0 {
                        self.print_char(b'>');
                    }
                    else {
                        self.print("< -");
                    }
                    self.print_glue(20000*UNITY, glue_order(p) as Integer, "");
                }
                else {
                    self.print_glue(((UNITY as Real)*g).round() as Scaled, glue_order(p) as Integer, "");
                }
            }
            // End section 186

            if shift_amount(p) != 0 {
                self.print(", shifted ");
                self.print_scaled(shift_amount(p));
            }
        }
        node_list_display!(self, list_ptr(p));
    }

    // Section 189
    fn sec189_display_glue(&mut self, p: Integer) {
        if subtype(p) >= A_LEADERS {
            // Section 190
            match subtype(p) {
                C_LEADERS => self.print_esc("cleaders "),
                X_LEADERS => self.print_esc("xleaders "),
                _         => self.print_esc("leaders "),
            }
            self.print_spec(glue_ptr(p), "");
            node_list_display!(self, leader_ptr(p));
            // End section 190
        }
        else {
            self.print_esc("glue");
            if subtype(p) != NORMAL {
                self.print_char(b'(');
                
                match subtype(p).cmp(&COND_MATH_GLUE) {
                    Less => self.print_skip_param(subtype(p) as Integer - 1),
                    Equal => self.print_esc("nonscript"),
                    Greater => self.print_esc("mskip"),
                }

                self.print_char(b')');
            }

            if subtype(p) != COND_MATH_GLUE {
                self.print_char(b' ');
                if subtype(p) < COND_MATH_GLUE {
                    self.print_spec(glue_ptr(p), "");
                }
                else {
                    self.print_spec(glue_ptr(p), "mu");
                }
            }
        }
    }

    // Section 198
    pub(crate) fn show_box(&mut self, p: HalfWord) {
        // Section 236
        self.depth_threshold = show_box_depth();
        self.breadth_max = show_box_breadth();
        // End section 236

        if self.breadth_max <= 0 {
            self.breadth_max = 5;
        }
        if pool_ptr() as Integer + self.depth_threshold >= POOL_SIZE {
            self.depth_threshold = POOL_SIZE - (pool_ptr() - 1) as Integer;
        }
        self.show_node_list(p);
        self.print_ln();
    }
}
