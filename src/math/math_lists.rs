use crate::builder::norm_min;
use crate::constants::*;
use crate::datastructures::{
    mem, mem_mut, character, character_mut, cur_fam, cur_font, del_code, display_indent,
    display_widow_penalty, display_width, eq_type, equiv, every_display,
    every_math, fam_fnt, font, glue_order, glue_ptr, glue_sign, hang_after,
    hang_indent, hsize, info, info_mut, left_hyphen_min, link, link_mut,
    list_ptr, math_code, math_surround, par_shape_ptr, post_display_penalty,
    pre_display_penalty, pre_display_size, right_hyphen_min, shift_amount,
    shift_amount_mut, shrink, shrink_order, stretch, stretch_order, subtype,
    subtype_mut, r#type, type_mut, width
};
use crate::error::{TeXError, TeXResult};
use crate::math::{
    display_mlist_mut, fam_mut, large_char_mut, large_fam_mut, math_type,
    math_type_mut, script_mlist_mut, script_script_mlist_mut, small_char_mut,
    small_fam_mut, text_mlist_mut, thickness_mut
};

use crate::{
    Global, HalfWord, Integer, QuarterWord, Scaled, SmallNumber, accent_chr,
    delimiter, denominator, half, ho, hpack, left_delimiter, lig_char, nucleus,
    numerator, odd, right_delimiter, scripts_allowed,
    sec404_get_next_nonblank_nonrelax_noncall_token,
    sec443_scan_an_optional_space, subscr, supscr, tail_append
};

// Part 48: Building math lists

// Section 1151
macro_rules! fam_in_range {
    () => {
        (0..16).contains(&cur_fam())
    };
}

impl Global {
    // Section 1136
    fn push_math(&mut self, c: Integer) -> TeXResult<()> {
        self.push_nest()?;
        *self.mode_mut() = -MMODE;
        *self.incomplete_noad_mut() = NULL;
        self.new_save_level(c)
    }

    // Section 1138
    pub(crate) fn init_math(&mut self) -> TeXResult<()> {
        self.get_token()?;
        if self.cur_cmd == MATH_SHIFT && self.mode() > 0 {
            // Section 1145
            let w = if self.head() == self.tail() {
                self.pop_nest();
                -MAX_DIMEN
            }
            else {
                self.line_break(display_widow_penalty())?;
                self.sec1146_calculate_the_natural_width()
            };

            // Section 1149
            let (l, s) = if par_shape_ptr() == NULL {
                if hang_indent() != 0
                    && (hang_after() >= 0
                    && self.prev_graf() + 2 > hang_after()
                    || self.prev_graf() + 1 < hang_after())
                {
                    let l = hsize() - hang_indent().abs();
                    let s = if hang_indent() > 0 {
                        hang_indent()
                    }
                    else {
                        0
                    };
                    (l, s)
                }
                else {
                    (hsize(), 0)
                }
            }
            else {
                let n = info(par_shape_ptr());
                let p = if self.prev_graf() + 2 >= n {
                    par_shape_ptr() + 2*n
                }
                else {
                    par_shape_ptr() + 2*(self.prev_graf() + 2)
                };

                let s = mem((p - 1) as usize).sc();
                let l = mem(p as usize).sc();
                (l, s)
            };
            // End section 1149

            self.push_math(MATH_SHIFT_GROUP)?;
            *self.mode_mut() = MMODE;
            self.eq_word_define(INT_BASE + CUR_FAM_CODE, -1)?;
            self.eq_word_define(DIMEN_BASE + PRE_DISPLAY_SIZE_CODE, w)?;
            self.eq_word_define(DIMEN_BASE + DISPLAY_WIDTH_CODE, l)?;
            self.eq_word_define(DIMEN_BASE + DISPLAY_INDENT_CODE, s)?;
            if every_display() != NULL {
                self.begin_token_list(every_display(), EVERY_DISPLAY_TEXT)?;
            }
            if self.nest_ptr == 1 {
                self.build_page()?;
            }
            // End section 1145
        }
        else {
            self.back_input()?;
            // Section 1139
            self.push_math(MATH_SHIFT_GROUP)?;
            self.eq_word_define(INT_BASE + CUR_FAM_CODE, -1)?;
            if every_math() != NULL {
                self.begin_token_list(every_math(), EVERY_MATH_TEXT)?;
            }
            // End section 1139
        }
        Ok(())
    }

    // Section 1142
    pub(crate) fn start_eq_no(&mut self) -> TeXResult<()> {
        *self.saved_mut(0) = self.cur_chr;
        self.save_ptr += 1;
        // Section 1139
        self.push_math(MATH_SHIFT_GROUP)?;
        self.eq_word_define(INT_BASE + CUR_FAM_CODE, -1)?;
        if every_math() != NULL {
            self.begin_token_list(every_math(), EVERY_MATH_TEXT)?;
        }
        // End section 1139
        Ok(())
    }

    // Section 1146
    fn sec1146_calculate_the_natural_width(&mut self) -> Scaled {
        let mut v = shift_amount(self.just_box) + 2*self.quad(cur_font() as QuarterWord);
        let mut w = -MAX_DIMEN;
        let mut p = list_ptr(self.just_box);
        while p != NULL {
            // Section 1147
            let (found, d) = 'reswitch: loop {
                if self.is_char_node(p) {
                    let f = font(p);
                    break (true, self.char_width(f, self.char_info(f, character(p))));
                }

                match r#type(p) {
                    HLIST_NODE
                    | VLIST_NODE
                    | RULE_NODE => break (true, width(p)),

                    LIGATURE_NODE => {
                        // Section 652
                        *mem_mut(LIG_TRICK as usize) = mem(lig_char!(p) as usize);
                        *link_mut(LIG_TRICK) = link(p);
                        p = LIG_TRICK;
                        continue 'reswitch; // Goto reswitch
                        // End section 652
                    },

                    KERN_NODE
                    | MATH_NODE => break (false, width(p)),

                    GLUE_NODE => {
                        // Section 1148
                        let q = glue_ptr(p);
                        let d = width(q);
                        if glue_sign(self.just_box) == STRETCHING {
                            if glue_order(self.just_box) == stretch_order(q)
                                && stretch(q) != 0
                            {
                                v = MAX_DIMEN;
                            }
                        }
                        else if glue_sign(self.just_box) == SHRINKING
                            && glue_order(self.just_box) == shrink_order(q)
                            && shrink(q) != 0
                        {
                            v = MAX_DIMEN;
                        }
                        break (subtype(p) >= A_LEADERS, d);
                        // End section 1148
                    },

                    WHATSIT_NODE => break (false, 0), // Section 1361
                    
                    _ => break (false, 0),
                }
            };
            // End section 1147

            if !found {
                if v < MAX_DIMEN {
                    v += d;
                }
            }
            else {
                // found:
                if v < MAX_DIMEN {
                    v += d;
                    w = v;
                }
                else {
                    w = MAX_DIMEN;
                    break; // Goto done
                }
            }

            // not_found:
            p = link(p);
        }
        // done:
        w
    }

    // Section 1151
    pub(crate) fn scan_math(&mut self, p: HalfWord) -> TeXResult<()> {
        let c = 'restart: loop {
            sec404_get_next_nonblank_nonrelax_noncall_token!(self);
            'reswitch: loop {
                match self.cur_cmd {
                    LETTER
                    | OTHER_CHAR
                    | CHAR_GIVEN => {
                        let c = ho!(math_code(self.cur_chr));
                        if c == 32768 {
                            // Section 1152
                            self.cur_cs = self.cur_chr + ACTIVE_BASE;
                            self.cur_cmd = eq_type(self.cur_cs);
                            self.cur_chr = equiv(self.cur_cs);
                            self.x_token()?;
                            self.back_input()?;
                            // End section 1152
                            continue 'restart;
                        }
                        break 'restart c;
                    },

                    CHAR_NUM => {
                        self.scan_char_num()?;
                        self.cur_chr = self.cur_val;
                        self.cur_cmd = CHAR_GIVEN;
                        continue 'reswitch;
                    },

                    MATH_CHAR_NUM => {
                        self.scan_fifteen_bit_int()?;
                        break 'restart self.cur_val;
                    },

                    MATH_GIVEN => break 'restart self.cur_chr,

                    DELIM_NUM => {
                        self.scan_twenty_seven_bit_int()?;
                        break 'restart self.cur_val / 4096;
                    },

                    _ => {
                        // Section 1153
                        self.back_input()?;
                        self.scan_left_brace()?;
                        *self.saved_mut(0) = p;
                        self.save_ptr += 1;
                        self.push_math(MATH_GROUP)?;
                        return Ok(());
                        // End section 1153
                    }
                }
            }
        };

        *math_type_mut(p) = MATH_CHAR;
        *character_mut(p) = (c % 256) as QuarterWord;
        *fam_mut(p) = if c >= VAR_CODE && fam_in_range!() {
            cur_fam() as QuarterWord
        }
        else {
            ((c / 256) % 16) as QuarterWord
        };
        Ok(())
    }

    // Section 1155
    pub(crate) fn set_math_char(&mut self, c: Integer) -> TeXResult<()> {
        if c >= 32768 {
            // Section 1152
            self.cur_cs = self.cur_chr + ACTIVE_BASE;
            self.cur_cmd = eq_type(self.cur_cs);
            self.cur_chr = equiv(self.cur_cs);
            self.x_token()?;
            self.back_input()?;
            // End section 1152
        }
        else {
            let p = self.new_noad()?;
            *math_type_mut(nucleus!(p)) = MATH_CHAR;
            *character_mut(nucleus!(p)) = (c % 256) as QuarterWord;
            *fam_mut(nucleus!(p)) = ((c / 256) % 16) as QuarterWord;
            if c >= VAR_CODE {
                if fam_in_range!() {
                    *fam_mut(nucleus!(p)) = cur_fam() as QuarterWord;
                }
                *type_mut(p) = ORD_NOAD;
            }
            else {
                *type_mut(p) = ORD_NOAD + (c / 4096) as QuarterWord;
            }
            *link_mut(self.tail()) = p;
            *self.tail_mut() = p;
        }
        Ok(())
    }

    // Section 1159
    pub(crate) fn math_limit_switch(&mut self) -> TeXResult<()> {
        if self.head() != self.tail() && r#type(self.tail()) == OP_NOAD {
            *subtype_mut(self.tail()) = self.cur_chr as QuarterWord;
            Ok(())
        }
        else {
            Err(TeXError::LimitControlsMustFollowMathOp)
        }
    }

    // Section 1160
    fn scan_delimiter(&mut self, p: HalfWord, r: bool) -> TeXResult<()> {
        if r {
            self.scan_twenty_seven_bit_int()?;
        }
        else {
            sec404_get_next_nonblank_nonrelax_noncall_token!(self);
            match self.cur_cmd {
                LETTER | OTHER_CHAR => self.cur_val = del_code(self.cur_chr),

                DELIM_NUM => self.scan_twenty_seven_bit_int()?,

                _ => self.cur_val = -1,
            }
        }
        if self.cur_val < 0 {
            return Err(TeXError::MissingDelimiterLeftParen)
        }
        
        *small_fam_mut(p) = ((self.cur_val / 1_048_576) % 16) as QuarterWord;
        *small_char_mut(p) = ((self.cur_val / 4096) % 256) as QuarterWord;
        *large_fam_mut(p) = ((self.cur_val / 256) % 16) as QuarterWord;
        *large_char_mut(p) = (self.cur_val % 256) as QuarterWord;
        Ok(())
    }

    // Section 1163
    pub(crate) fn math_radical(&mut self) -> TeXResult<()> {
        tail_append!(self, self.get_node(RADICAL_NOAD_SIZE)?);
        *type_mut(self.tail()) = RADICAL_NOAD;
        *subtype_mut(self.tail()) = NORMAL;
        *mem_mut(nucleus!(self.tail()) as usize) = self.empty_field;
        *mem_mut(subscr!(self.tail()) as usize) = self.empty_field;
        *mem_mut(supscr!(self.tail()) as usize) = self.empty_field;
        self.scan_delimiter(left_delimiter!(self.tail()), true)?;
        self.scan_math(nucleus!(self.tail()))
    }

    // Section 1165
    pub(crate) fn math_ac(&mut self) -> TeXResult<()> {
        if self.cur_cmd == ACCENT {
            return Err(TeXError::UseMathAccentInMathMode);
        }
        tail_append!(self, self.get_node(ACCENT_NOAD_SIZE)?);
        *type_mut(self.tail()) = ACCENT_NOAD;
        *subtype_mut(self.tail()) = NORMAL;
        *mem_mut(nucleus!(self.tail()) as usize) = self.empty_field;
        *mem_mut(subscr!(self.tail()) as usize) = self.empty_field;
        *mem_mut(supscr!(self.tail()) as usize) = self.empty_field;
        *math_type_mut(accent_chr!(self.tail())) = MATH_CHAR;
        self.scan_fifteen_bit_int()?;
        *character_mut(accent_chr!(self.tail())) = (self.cur_val % 256) as QuarterWord;
        *fam_mut(accent_chr!(self.tail())) = if self.cur_val >= VAR_CODE && fam_in_range!() {
            cur_fam() as QuarterWord
        }
        else {
            ((self.cur_val / 256) % 16) as QuarterWord
        };

        self.scan_math(nucleus!(self.tail()))
    }

    // Section 1172
    pub(crate) fn append_choices(&mut self) -> TeXResult<()> {
        tail_append!(self, self.new_choice()?);
        self.save_ptr += 1;
        *self.saved_mut(-1) = 0;
        self.push_math(MATH_CHOICE_GROUP)?;
        self.scan_left_brace()
    }

    // Section 1174
    pub(crate) fn build_choices(&mut self) -> TeXResult<()> {
        self.unsave()?;
        let p = self.fin_mlist(NULL)?;
        match self.saved(-1) {
            0 => *display_mlist_mut(self.tail()) = p,
            1 => *text_mlist_mut(self.tail()) = p,
            2 => *script_mlist_mut(self.tail()) = p,
            _ /* 3 */ => {
                *script_script_mlist_mut(self.tail()) = p;
                self.save_ptr -= 1;
                return Ok(());
            }
        }
        *self.saved_mut(-1) += 1;
        self.push_math(MATH_CHOICE_GROUP)?;
        self.scan_left_brace()
    }

    // Section 1176
    pub(crate) fn sub_sup(&mut self) -> TeXResult<()> {
        let mut t = EMPTY;
        let mut p = NULL;
        if self.tail() != self.head() && scripts_allowed!(self.tail()) {
            p = supscr!(self.tail()) + (self.cur_cmd as HalfWord) - (SUP_MARK as HalfWord);
            t = math_type(p);
        }

        if p == NULL || t != EMPTY {
            // Section 1177
            tail_append!(self, self.new_noad()?);
            p = supscr!(self.tail()) + (self.cur_cmd as HalfWord) - (SUP_MARK as HalfWord);
            if t != EMPTY {
                if self.cur_cmd == SUP_MARK {
                    return Err(TeXError::DoubleSuperscript);
                }
                else {
                    return Err(TeXError::DoubleSubscript);
                }
            }
            // End section 1177
        }
        self.scan_math(p)
    }

    // Section 1181
    pub(crate) fn math_fraction(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        if self.incomplete_noad() != NULL {
            Err(TeXError::AmbiguousFraction)
        }
        else {
            *self.incomplete_noad_mut() = self.get_node(FRACTION_NOAD_SIZE)?;
            *type_mut(self.incomplete_noad()) = FRACTION_NOAD;
            *subtype_mut(self.incomplete_noad()) = NORMAL;
            *math_type_mut(numerator!(self.incomplete_noad())) = SUB_MLIST;
            *info_mut(numerator!(self.incomplete_noad())) = link(self.head());
            *mem_mut(denominator!(self.incomplete_noad()) as usize) = self.empty_field;
            *mem_mut(left_delimiter!(self.incomplete_noad()) as usize) = self.null_delimiter;
            *mem_mut(right_delimiter!(self.incomplete_noad()) as usize) = self.null_delimiter;
            *link_mut(self.head()) = NULL;
            *self.tail_mut() = self.head();

            // Section 1182
            if c >= DELIMITED_CODE {
                self.scan_delimiter(left_delimiter!(self.incomplete_noad()), false)?;
                self.scan_delimiter(right_delimiter!(self.incomplete_noad()), false)?;
            }

            *thickness_mut(self.incomplete_noad()) = match c % DELIMITED_CODE {
                ABOVE_CODE => {
                    self.scan_dimen(false, false, false)?;
                    self.cur_val
                },

                OVER_CODE => DEFAULT_CODE,

                _ /* ATOP_CODE */ => 0,
            };
            // End section 1182
            Ok(())
        }
    }

    // Section 1184
    pub(crate) fn fin_mlist(&mut self, p: HalfWord) -> TeXResult<HalfWord> {
        let q = if self.incomplete_noad() != NULL {
            // Section 1185
            *math_type_mut(denominator!(self.incomplete_noad())) = SUB_MLIST;
            *info_mut(denominator!(self.incomplete_noad())) = link(self.head());
            if p == NULL {
                self.incomplete_noad()
            }
            else {
                let q = info(numerator!(self.incomplete_noad()));
                if r#type(q) != LEFT_NOAD {
                    return Err(TeXError::Confusion("right"));
                }
                *info_mut(numerator!(self.incomplete_noad())) = link(q);
                *link_mut(q) = self.incomplete_noad();
                *link_mut(self.incomplete_noad()) = p;
                q
            }
            // End section 1985
        }
        else {
            *link_mut(self.tail()) = p;
            link(self.head())
        };

        self.pop_nest();
        Ok(q)
    }

    // Section 1191
    pub(crate) fn math_left_right(&mut self) -> TeXResult<()> {
        let t = self.cur_chr;
        if t == RIGHT_NOAD as HalfWord && self.cur_group != MATH_LEFT_GROUP {
            // Section 1192
            if self.cur_group == MATH_SHIFT_GROUP {
                Err(TeXError::ExtraMathRight)
            }
            else {
                self.off_save()
            }
            // End section 1192
        }
        else {
            let p = self.new_noad()?;
            *type_mut(p) = t as QuarterWord;
            self.scan_delimiter(delimiter!(p), false)?;
            if t == LEFT_NOAD as HalfWord {
                self.push_math(MATH_LEFT_GROUP)?;
                *link_mut(self.head()) = p;
                *self.tail_mut() = p;
            }
            else {
                let p = self.fin_mlist(p)?;
                self.unsave()?;
                tail_append!(self, self.new_noad()?);
                *type_mut(self.tail()) = INNER_NOAD;
                *math_type_mut(nucleus!(self.tail())) = SUB_MLIST;
                *info_mut(nucleus!(self.tail())) = p;
            }
            Ok(())
        }
    }

    // Section 1194
    pub(crate) fn after_math(&mut self) -> TeXResult<()> {
        // Section 1195
        if self.font_params[fam_fnt(2 + TEXT_SIZE as HalfWord) as usize] < TOTAL_MATHSY_PARAMS
            || self.font_params[fam_fnt(2 + SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHSY_PARAMS
            || self.font_params[fam_fnt(2 + SCRIPT_SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHSY_PARAMS
        {
            return Err(TeXError::InsufficientSymbolFonts);
        }

        if self.font_params[fam_fnt(3 + TEXT_SIZE as HalfWord) as usize] < TOTAL_MATHEX_PARAMS
            || self.font_params[fam_fnt(3 + SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHEX_PARAMS
            || self.font_params[fam_fnt(3 + SCRIPT_SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHEX_PARAMS
        {
            return Err(TeXError::InsufficientExtensionFonts);
        }
        // End section 1195

        let mut m = self.mode();
        let mut l = false;
        let mut p = self.fin_mlist(NULL)?;
        let a = if self.mode() == -m {
            // Section 1197
            self.get_x_token()?;
            if self.cur_cmd != MATH_SHIFT {
                return Err(TeXError::DisplayMathEndsWithDollars);
            }
            // End section 1197

            self.cur_mlist = p;
            self.cur_style = TEXT_STYLE;
            self.mlist_penalties = false;
            self.mlist_to_hlist()?;
            let a = hpack!(self, link(TEMP_HEAD), NATURAL)?;
            self.unsave()?;
            self.save_ptr -= 1;
            if self.saved(0) == 1 {
                l = true;
            }
            
            // Section 1195
            if self.font_params[fam_fnt(2 + TEXT_SIZE as HalfWord) as usize] < TOTAL_MATHSY_PARAMS
                || self.font_params[fam_fnt(2 + SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHSY_PARAMS
                || self.font_params[fam_fnt(2 + SCRIPT_SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHSY_PARAMS
            {
                return Err(TeXError::InsufficientSymbolFonts);
            }
            if self.font_params[fam_fnt(3 + TEXT_SIZE as HalfWord) as usize] < TOTAL_MATHEX_PARAMS
                || self.font_params[fam_fnt(3 + SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHEX_PARAMS
                || self.font_params[fam_fnt(3 + SCRIPT_SCRIPT_SIZE as HalfWord) as usize] < TOTAL_MATHEX_PARAMS
            {
                return Err(TeXError::InsufficientExtensionFonts);
            }
            // End section 1195

            m = self.mode();
            p = self.fin_mlist(NULL)?;
            a
        }
        else {
            NULL
        };

        if m < 0 {
            // Section 1196
            tail_append!(self, self.new_math(math_surround(), BEFORE)?);
            self.cur_mlist = p;
            self.cur_style = TEXT_STYLE;
            self.mlist_penalties = self.mode() > 0;
            self.mlist_to_hlist()?;
            *link_mut(self.tail()) = link(TEMP_HEAD);
            while link(self.tail()) != NULL {
                *self.tail_mut() = link(self.tail());
            }
            tail_append!(self, self.new_math(math_surround(), AFTER)?);
            *self.space_factor_mut() = 1000;
            self.unsave()
            // End section 1196
        }
        else {
            if a == NULL {
                // Section 1197
                self.get_x_token()?;
                if self.cur_cmd != MATH_SHIFT {
                    return Err(TeXError::DisplayMathEndsWithDollars);
                }
                // End section 1197
            }
            self.sec1199_finish_displayed_math(p, a, l)
        }
    }

    // Section 1199
    fn sec1199_finish_displayed_math(&mut self, mut p: HalfWord, a: HalfWord, l: bool) -> TeXResult<()> {
        self.cur_mlist = p;
        self.cur_style = DISPLAY_STYLE;
        self.mlist_penalties = false;
        self.mlist_to_hlist()?;
        p = link(TEMP_HEAD);
        self.adjust_tail = ADJUST_HEAD;
        let mut b = hpack!(self, p, NATURAL)?;
        p = list_ptr(b);
        let t = self.adjust_tail;
        self.adjust_tail = NULL;
        let mut w = width(b);
        let z = display_width();
        let s = display_indent();
        let (mut e, q) = if a == NULL {
            (0, 0)
        }
        else {
            let e = width(a);
            (e, e + self.math_quad(TEXT_SIZE))
        };

        if w + q > z {
            // Section 1201
            if e != 0 && ((w - self.total_shrink[NORMAL as usize] + q <= z)
                || self.total_shrink[FIL as usize] != 0
                || self.total_shrink[FILL as usize] != 0
                || self.total_shrink[FILLL as usize] != 0)
            {
                self.free_node(b, BOX_NODE_SIZE);
                b = hpack!(self, p, z - q, EXACTLY)?;
            }
            else {
                e = 0;
                if w > z {
                    self.free_node(b, BOX_NODE_SIZE);
                    b = hpack!(self, p, z, EXACTLY)?;
                }
            }
            w = width(b);
            // End section 1201
        }
        
        // Section 1202
        let mut d = half!(z - w);
        if e > 0 && d < 2*e {
            d = half!(z - w - e);
            if p != NULL && !self.is_char_node(p) && r#type(p) == GLUE_NODE {
                d = 0;
            }
        }
        // End section 1202

        // Section 1203
        tail_append!(self, self.new_penalty(pre_display_penalty())?);
        let (g1, mut g2) = if d + s <= pre_display_size() || l {
            (ABOVE_DISPLAY_SKIP_CODE, BELOW_DISPLAY_SKIP_CODE)
        }
        else {
            (ABOVE_DISPLAY_SHORT_SKIP_CODE, BELOW_DISPLAY_SHORT_SKIP_CODE)
        };

        if l && e == 0 {
            *shift_amount_mut(a) = s;
            self.append_to_vlist(a)?;
            tail_append!(self, self.new_penalty(INF_PENALTY)?);
        }
        else {
            tail_append!(self, self.new_param_glue(g1 as SmallNumber)?);
        }
        // End section 1203

        // Section 1204
        if e != 0 {
            let r = self.new_kern(z - w - e - d)?;
            if l {
                *link_mut(a) = r;
                *link_mut(r) = b;
                b = a;
                d = 0;
            }
            else {
                *link_mut(b) = r;
                *link_mut(r) = a;
            }
            b = hpack!(self, b, NATURAL)?;
        }
        *shift_amount_mut(b) = s + d;
        self.append_to_vlist(b)?;
        // End section 1204

        // Section 1205
        if a != NULL && e == 0 && !l {
            tail_append!(self, self.new_penalty(INF_PENALTY)?);
            *shift_amount_mut(a) = s + z - width(a);
            self.append_to_vlist(a)?;
            g2 = 0;
        }
        if t != ADJUST_HEAD {
            *link_mut(self.tail()) = link(ADJUST_HEAD);
            *self.tail_mut() = t;
        }
        tail_append!(self, self.new_penalty(post_display_penalty())?);
        if g2 > 0 {
            tail_append!(self, self.new_param_glue(g2 as SmallNumber)?);
        }
        // End section 1205

        self.resume_after_display()
    }

    // Section 1200
    pub(crate) fn resume_after_display(&mut self) -> TeXResult<()> {
        if self.cur_group != MATH_SHIFT_GROUP {
            return Err(TeXError::Confusion("display"));
        }
        self.unsave()?;
        *self.prev_graf_mut() += 3;
        self.push_nest()?;
        *self.mode_mut() = HMODE;
        *self.space_factor_mut() = 1000;
        self.set_cur_lang();
        *self.clang_mut() = self.cur_lang as HalfWord;
        *self.prev_graf_mut() = (norm_min(left_hyphen_min())*64 + norm_min(right_hyphen_min()))*65536 + self.cur_lang as Integer;
        sec443_scan_an_optional_space!(self);
        if self.nest_ptr == 1 {
            self.build_page()?;
        }
        Ok(())
    }
}
