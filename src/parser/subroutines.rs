use crate::arithmetic::{mult_and_add, xn_over_d};
use crate::constants::*;
use crate::datastructures::{
    EQTB, mem, count, cur_font, depth_mut, dimen, equiv, glue_ptr,
    glue_ref_count_mut, height_mut, info, info_mut, link, link_mut, mag,
    math_code, mu_skip, par_shape_ptr, penalty, r#box, r#type, shrink,
    shrink_mut, shrink_order_mut, skip, stretch, stretch_mut,
    stretch_order_mut, subtype, width, width_mut
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, QuarterWord, Scaled, SmallNumber,
    add_glue_ref, back_list, eqtb, free_avail, ho, nx_plus_y
};

use std::cmp::Ordering::{Equal, Greater, Less};

// Part 26: Basic scanning subroutines

// Section 404
#[macro_export]
macro_rules! sec404_get_next_nonblank_nonrelax_noncall_token {
    ($s:ident) => {
        loop {
            $s.get_x_token()?;
            if $s.cur_cmd != SPACER && $s.cur_cmd != RELAX {
                break;
            }
        }
    };
}

// Section 406
#[macro_export]
macro_rules! sec406_get_next_nonblank_noncall_token {
    ($s:ident) => {
        loop {
            $s.get_x_token()?;
            if $s.cur_cmd != SPACER {
                break;
            }
        }
    };
}

// Section 413
macro_rules! scanned_result {
    ($s:ident,$p:expr,$q:expr) => {
        {
            $s.cur_val = $p;
            $s.cur_val_level = $q;
        }
    };
}

// Section 443
#[macro_export]
macro_rules! sec443_scan_an_optional_space {
    ($s:ident) => {
        $s.get_x_token()?;
        if $s.cur_cmd != SPACER {
            $s.back_input()?;
        }
    };
}

impl Global {
    // Section 403
    pub(crate) fn scan_left_brace(&mut self) -> TeXResult<()> {
        sec404_get_next_nonblank_nonrelax_noncall_token!(self);
        match self.cur_cmd {
            LEFT_BRACE => Ok(()),
            _ => Err(TeXError::MissingLeftBrace)
        }
    }
    
    // Section 405
    pub(crate) fn scan_optional_equals(&mut self) -> TeXResult<()> {
        sec406_get_next_nonblank_noncall_token!(self);
        if self.cur_tok != OTHER_TOKEN + (b'=' as HalfWord) {
            self.back_input()?;
        }
        Ok(())
    }

    // Section 407
    pub(crate) fn scan_keyword(&mut self, s: &[u8]) -> TeXResult<bool> {
        let mut p = BACKUP_HEAD;
        *link_mut(p) = NULL;
        let mut k = 0;
        while k < s.len() {
            self.get_x_token()?;
            if self.cur_cs == 0 && (self.cur_chr == (s[k] as HalfWord) || self.cur_chr == (s[k] - b'a' + b'A') as HalfWord) {
                self.store_new_token(&mut p, self.cur_tok)?;
                k += 1;
            }
            else if self.cur_cmd != SPACER || p != BACKUP_HEAD {
                self.back_input()?;
                if p != BACKUP_HEAD {
                    back_list!(self, link(BACKUP_HEAD));
                }
                return Ok(false);
            }
        }
        self.flush_list(link(BACKUP_HEAD));
        Ok(true)
    }

    // Section 413
    pub(crate) fn scan_something_internal(&mut self, level: SmallNumber, negative: bool) -> TeXResult<()> {
        let mut m = self.cur_chr;
        match self.cur_cmd {
            DEF_CODE => {
                // Section 414
                self.scan_char_num()?;
                match m.cmp(&MATH_CODE_BASE) {
                    Equal => scanned_result!(self, ho!(math_code(self.cur_val)), INT_VAL),
                    Less => scanned_result!(self, equiv(m + self.cur_val), INT_VAL),
                    Greater => scanned_result!(self, eqtb![(m + self.cur_val) as usize].int(), INT_VAL)
                }
                // End section 414
            },

            TOKS_REGISTER
            | ASSIGN_TOKS
            | DEF_FAMILY
            | SET_FONT
            | DEF_FONT => {
                // Section 415
                if (level as Integer) != TOK_VAL {
                    return Err(TeXError::MissingNumber);
                }
                if self.cur_cmd <= ASSIGN_TOKS {
                    if self.cur_cmd < ASSIGN_TOKS {
                        self.scan_eight_bit_int()?;
                        m = TOKS_BASE + self.cur_val;
                    }
                    scanned_result!(self, equiv(m), TOK_VAL);
                }
                else {
                    self.back_input()?;
                    self.scan_font_ident()?;
                    scanned_result!(self, FONT_ID_BASE + self.cur_val, IDENT_VAL);
                }
                // End section 415
            },

            ASSIGN_INT => scanned_result!(self, eqtb![m as usize].int(), INT_VAL),

            ASSIGN_DIMEN => scanned_result!(self, eqtb![m as usize].sc(), DIMEN_VAL),

            ASSIGN_GLUE => scanned_result!(self, equiv(m), GLUE_VAL),

            ASSIGN_MU_GLUE => scanned_result!(self, equiv(m), MU_VAL),

            SET_AUX => {
                // Section 418
                if self.mode().abs() != m {
                    return Err(TeXError::ImproperMode(m ));
                }
                if m == VMODE {
                    scanned_result!(self, self.prev_depth(), DIMEN_VAL);
                }
                else {
                    scanned_result!(self, self.space_factor(), INT_VAL);
                }
                // End section 418
            },

            SET_PREV_GRAF => {
                // Section 422
                if self.mode() == 0 {
                    scanned_result!(self, 0, INT_VAL);
                }
                else {
                    self.nest[self.nest_ptr] = self.cur_list;
                    let mut p = self.nest_ptr;
                    while self.nest[p].mode_field.abs() != VMODE {
                        p -= 1;
                    }
                    scanned_result!(self, self.nest[p].pg_field, INT_VAL);
                }
                // End section 422
            },

            SET_PAGE_INT => {
                // Section 419
                self.cur_val = match m {
                    0 => self.dead_cycles,
                    _ => self.insert_penalties
                };
                self.cur_val_level = INT_VAL;
                // End section 419
            },

            SET_PAGE_DIMEN => {
                // Section 421
                self.cur_val = if (self.page_contents as HalfWord) == EMPTY && !self.output_active {
                    match m {
                        0 => MAX_DIMEN,
                        _ => 0
                    }
                }
                else {
                    self.page_so_far[m as usize]
                };

                self.cur_val_level = DIMEN_VAL;
                // End section 421
            },

            SET_SHAPE => {
                // Section 423
                self.cur_val = match par_shape_ptr() {
                    NULL => 0,
                    _ => info(par_shape_ptr()),
                };
                self.cur_val_level = INT_VAL;
                // End section 423
            },

            SET_BOX_DIMEN => {
                // Section 420
                self.scan_eight_bit_int()?;
                self.cur_val = match r#box(self.cur_val) {
                    NULL => 0,
                    _ => mem((r#box(self.cur_val) + m) as usize).sc(),
                };
                self.cur_val_level = DIMEN_VAL;
                // End section 420
            },

            CHAR_GIVEN
            | MATH_GIVEN => scanned_result!(self, self.cur_chr, INT_VAL),

            ASSIGN_FONT_DIMEN => {
                // Section 425
                self.find_font_dimen(false)?;
                *self.font_info[self.fmem_ptr].sc_mut() = 0;
                scanned_result!(self, self.font_info[self.cur_val as usize].sc(), DIMEN_VAL);
                // End section 425
            },

            ASSIGN_FONT_INT =>  {
                // Section 426
                self.scan_font_ident()?;
                if m == 0 {
                    scanned_result!(self, self.hyphen_char[self.cur_val as usize], INT_VAL);
                }
                else {
                    scanned_result!(self, self.skew_char[self.cur_val as usize], INT_VAL);
                }
                // End section 426
            },

            REGISTER => {
                // Section 427
                self.scan_eight_bit_int()?;
                match m {
                    INT_VAL => self.cur_val = count(self.cur_val),
                    DIMEN_VAL => self.cur_val = dimen(self.cur_val),
                    GLUE_VAL => self.cur_val = skip(self.cur_val),
                    MU_VAL => self.cur_val = mu_skip(self.cur_val),
                    _ => () // There are no other cases
                }
                self.cur_val_level = m;
                // End section 427
            },

            LAST_ITEM => self.sec424_fetch_item_in_current_node(),

            _ => return Err(TeXError::CantUseAfterThe),
        }

        while self.cur_val_level > level as Integer {
            // Section 429
            if self.cur_val_level == GLUE_VAL {
                self.cur_val = width(self.cur_val);
            }
            else if self.cur_val_level == MU_VAL {
                return Err(TeXError::IncompatibleGlueUnits);
            }
            self.cur_val_level -= 1;
            // End section 429
        }

        // Section 430
        if negative {
            if self.cur_val_level >= GLUE_VAL {
                self.cur_val = self.new_spec(self.cur_val)?;
                // Section 431 
                *width_mut(self.cur_val) = -width(self.cur_val);
                *stretch_mut(self.cur_val) = -stretch(self.cur_val);
                *shrink_mut(self.cur_val) = -shrink(self.cur_val);
                // End section 431
            }
            else {
                self.cur_val = -self.cur_val;
            }
        }
        else if self.cur_val_level >= GLUE_VAL && self.cur_val_level <= MU_VAL {
            add_glue_ref!(self.cur_val);
        }
        // End section 430

        Ok(())
    }

    // Section 424
    fn sec424_fetch_item_in_current_node(&mut self) {
        if self.cur_chr > GLUE_VAL {
            self.cur_val = match self.cur_chr {
                INPUT_LINE_NO_CODE => self.line,
                _ => self.last_badness,
            };
            self.cur_val_level = INT_VAL;
        }
        else {
            self.cur_val = match self.cur_chr {
                GLUE_VAL => ZERO_GLUE,
                _ => 0,
            };
            self.cur_val_level = self.cur_chr;

            if !self.is_char_node(self.tail()) && self.mode() != 0 {
                match self.cur_chr {
                    INT_VAL => {
                        if r#type(self.tail()) == PENALTY_NODE {
                            self.cur_val = penalty(self.tail());
                        }
                    },

                    DIMEN_VAL => {
                        if r#type(self.tail()) == KERN_NODE {
                            self.cur_val = width(self.tail());
                        }
                    },

                    GLUE_VAL => {
                        if r#type(self.tail()) == GLUE_NODE {
                            self.cur_val = glue_ptr(self.tail());
                            if subtype(self.tail()) == MU_GLUE {
                                self.cur_val_level = MU_VAL;
                            }
                        }
                    },

                    _ => () // There are no other cases                    
                }
            }
            else if self.mode() == VMODE && self.tail() == self.head() {
                match self.cur_chr {
                    INT_VAL => self.cur_val = self.last_penalty,

                    DIMEN_VAL => self.cur_val = self.last_kern,
                    
                    GLUE_VAL => {
                        if self.last_glue != MAX_HALFWORD {
                            self.cur_val = self.last_glue;
                        }
                    },

                    _ => () // There are no other cases
                }
            }
        }
    }

    // Section 433
    pub(crate) fn scan_eight_bit_int(&mut self) -> TeXResult<()> {
        self.scan_int()?;
        if (0..=255).contains(&self.cur_val) {
            Ok(())
        }
        else {
            Err(TeXError::BadRegisterCode)
        }
    }

    // Section 434
    pub(crate) fn scan_char_num(&mut self) -> TeXResult<()> {
        self.scan_int()?;
        if (0..=255).contains(&self.cur_val) {
            Ok(())
        }
        else {
            Err(TeXError::BadCharacterCode)
        }
    }

    // Section 435
    pub(crate) fn scan_four_bit_int(&mut self) -> TeXResult<()> {
        self.scan_int()?;
        if (0..=15).contains(&self.cur_val) {
            Ok(())
        }
        else {
            Err(TeXError::BadNumber)
        }
    }

    // Section 436
    pub(crate) fn scan_fifteen_bit_int(&mut self) -> TeXResult<()> {
        self.scan_int()?;
        if (0..=0x7fff).contains(&self.cur_val) {
            Ok(())
        }
        else {
            Err(TeXError::BadMathChar)
        }
    }

    // Section 437
    pub(crate) fn scan_twenty_seven_bit_int(&mut self) -> TeXResult<()> {
        self.scan_int()?;
        if (0..=0x7ff_ffff).contains(&self.cur_val) {
            Ok(())
        }
        else {
            Err(TeXError::BadDelimiterCode)
        }
    }

    // Section 440
    pub(crate) fn scan_int(&mut self) -> TeXResult<()> {
        self.radix = 0;

        // Section 441
        let mut negative = false;
        loop {
            sec406_get_next_nonblank_noncall_token!(self);
            if self.cur_tok == OTHER_TOKEN + b'-' as HalfWord {
                negative = !negative;
                self.cur_tok = OTHER_TOKEN + b'+' as HalfWord;
            }
            if self.cur_tok != OTHER_TOKEN + b'+' as HalfWord {
                break;
            }
        }
        // End section 441

        if self.cur_tok == ALPHA_TOKEN {
            self.sec442_scan_alphabetic_character_code()?;
        }
        else if (MIN_INTERNAL..=MAX_INTERNAL).contains(&self.cur_cmd) {
            self.scan_something_internal(INT_VAL as QuarterWord, false)?;
        }
        else {
            // Section 444
            self.radix = 10;
            let mut m = 214_748_364;
            if self.cur_tok == OCTAL_TOKEN {
                self.radix = 8;
                m = 0x1000_0000;
                self.get_x_token()?;
            }
            else if self.cur_tok == HEX_TOKEN {
                self.radix = 16;
                m = 0x800_0000;
                self.get_x_token()?;
            }
            let mut vacuous = true;
            self.cur_val = 0;
            self.sec445_accumulate_constant_until(&mut vacuous, m)?;
            
            if vacuous {
                return Err(TeXError::MissingNumber);
            }
            if self.cur_cmd != SPACER {
                self.back_input()?;
            }
            // End section 444
        }
        if negative {
            self.cur_val = -self.cur_val;
        }
        Ok(())
    }

    // Section 442
    fn sec442_scan_alphabetic_character_code(&mut self) -> TeXResult<()> {
        self.get_token()?;
        self.cur_val = if self.cur_tok < CS_TOKEN_FLAG {
            if self.cur_cmd <= RIGHT_BRACE {
                if self.cur_cmd == RIGHT_BRACE {
                    self.align_state += 1;
                }
                else {
                    self.align_state -= 1;
                }
            }
            self.cur_chr
        }
        else if self.cur_tok < CS_TOKEN_FLAG + SINGLE_BASE {
            self.cur_tok - CS_TOKEN_FLAG - ACTIVE_BASE
        }
        else {
            self.cur_tok - CS_TOKEN_FLAG - SINGLE_BASE
        };

        if self.cur_val > 255 {
            Err(TeXError::ImproperAlphabeticConstant)
        }
        else {
            sec443_scan_an_optional_space!(self);
            Ok(())
        }
    }

    // Section 445
    fn sec445_accumulate_constant_until(&mut self, vacuous: &mut bool, m: Integer) -> TeXResult<()> {
        loop {
            let d = if self.cur_tok < ZERO_TOKEN + self.radix && self.cur_tok >= ZERO_TOKEN && self.cur_tok <= ZERO_TOKEN + 9 {
                self.cur_tok - ZERO_TOKEN
            }
            else if self.radix == 16 {
                if self.cur_tok <= A_TOKEN + 5 && self.cur_tok >= A_TOKEN {
                    self.cur_tok - A_TOKEN + 10
                }
                else if self.cur_tok <= OTHER_A_TOKEN + 5 && self.cur_tok >= OTHER_A_TOKEN {
                    self.cur_tok - OTHER_A_TOKEN + 10
                }
                else {
                    break; // Goto done
                }
            }
            else {
                break; // Goto done
            };

            *vacuous = false;
            if self.cur_val >= m
                && (self.cur_val > m || d > 7 || self.radix != 10)
            {
                return Err(TeXError::NumberTooBig);
            }
            self.cur_val = self.cur_val * self.radix + d;
            self.get_x_token()?;
        }
        // done:
        Ok(())
    }

    // Section 448
    pub(crate) fn scan_dimen(&mut self, mu: bool, inf: bool, shortcut: bool) -> TeXResult<()> {
        let mut f = 0;
        self.cur_order = NORMAL;
        let mut negative = false;

        'block: {
            if !shortcut {
                // Section 441
                negative = false;
                loop {
                    sec406_get_next_nonblank_noncall_token!(self);
                    if self.cur_tok == OTHER_TOKEN + b'-' as HalfWord {
                        negative = !negative;
                        self.cur_tok = OTHER_TOKEN + b'+' as HalfWord;
                    }
                    if self.cur_tok != OTHER_TOKEN + b'+' as HalfWord {
                        break;
                    }
                }
                // End section 441
            
                if self.cur_cmd >= MIN_INTERNAL && self.cur_cmd <= MAX_INTERNAL {
                    // Section 449
                    if mu {
                        self.scan_something_internal(MU_VAL as QuarterWord, false)?;

                        // Section 451
                        if self.cur_val_level >= GLUE_VAL {
                            let v = width(self.cur_val);
                            self.delete_glue_ref(self.cur_val);
                            self.cur_val = v;
                        }
                        // End section 451

                        if self.cur_val_level == MU_VAL {
                            break 'block; // Goto attach_sign
                        }

                        if self.cur_val_level != INT_VAL {
                            return Err(TeXError::IncompatibleGlueUnits);
                        }
                    }
                    else {
                        self.scan_something_internal(DIMEN_VAL as QuarterWord, false)?;
                        if self.cur_val_level == DIMEN_VAL {
                            break 'block; // Goto attach_sign
                        }
                    }
                    // End section 449                    
                }
                else {
                    self.back_input()?;
                    if self.cur_tok == CONTINENTAL_POINT_TOKEN {
                        self.cur_tok = POINT_TOKEN;
                    }
                    if self.cur_tok != POINT_TOKEN {
                        self.scan_int()?;
                    }
                    else {
                        self.radix = 10;
                        self.cur_val = 0;
                    }
                    if self.cur_tok == CONTINENTAL_POINT_TOKEN {
                        self.cur_tok = POINT_TOKEN;
                    }
                    if self.radix == 10 && self.cur_tok == POINT_TOKEN {
                        f = self.sec452_scan_decimal_fraction()?;
                    }
                }
            }

            if self.cur_val < 0 {
                negative = !negative;
                self.cur_val = -self.cur_val;
            }
            if self.sec453_scan_units(f, inf, mu)? {
                break 'block; // Goto attach_sign
            }
            sec443_scan_an_optional_space!(self);
        }

        // attach_sign:
        if self.cur_val.abs() >= 0x4000_0000 {
            return Err(TeXError::DimensionTooLarge);
        }
        if negative {
            self.cur_val = -self.cur_val
        }
        Ok(())
    }

    // Section 452
    fn sec452_scan_decimal_fraction(&mut self) -> TeXResult<Integer> {
        let mut k = 0;
        let mut p = NULL;
        self.get_token()?;
        loop {
            self.get_x_token()?;
            if self.cur_tok > ZERO_TOKEN + 9 || self.cur_tok < ZERO_TOKEN {
                break; // Goto done1
            }
            if k < 17 {
                let q = self.get_avail()?;
                *link_mut(q) = p;
                *info_mut(q) = self.cur_tok - ZERO_TOKEN;
                p = q;
                k += 1;
            }
        }

        // done1:
        for kk in (1..=k).rev() {
            self.dig[kk - 1] = info(p) as u8;
            let q = p;
            p = link(p);
            free_avail!(self, q);
        }
        let f = self.round_decimals(k);
        if self.cur_cmd != SPACER {
            self.back_input()?;
        }
        Ok(f)
    }

    // Section 453
    // returns `true` if goto attach_sign
    fn sec453_scan_units(&mut self, mut f: Integer, inf: bool, mu: bool) -> TeXResult<bool> {
        'block: {
            if inf {
                // Section 454
                if self.scan_keyword(b"fil")? {
                    self.cur_order = FIL;
                    while self.scan_keyword(b"l")? {
                        if self.cur_order == FILLL {
                            return Err(TeXError::IllegalUnitOfMeasureFilll);
                        }
                        self.cur_order += 1;
                    }
                    break 'block; // Goto attach_fraction
                }
                // End section 454
            }

            if self.sec455_scan_for_units_that_are_internal_dimensions(f, mu)? {
                return Ok(true);
            }

            if mu {
                // Section 456
                match self.scan_keyword(b"mu")? {
                    true => break 'block, // Goto attach_fraction
                    false => return Err(TeXError::IllegalUnitOfMeasureMu),
                }
                // End section 456
            }

            if self.scan_keyword(b"true")? {
                // Section 457
                self.prepare_mag()?;
                if mag() != 1000 {
                    let rem: Scaled;
                    (self.cur_val, rem) = xn_over_d(self.cur_val, 1000, mag())?;
                    f = (1000 * f + 65536 * rem) / mag();
                    self.cur_val += f / 65536;
                    f %= 65536;
                }
                // End section 457
            }

            if self.scan_keyword(b"pt")? {
                break 'block; // Goto attach_fraction
            }

            if self.sec458_scan_for_all_other_units(&mut f)? {
                return Ok(false); // Goto done
            }
        }

        // attach_fraction:
        match self.cur_val.cmp(&16384) {
            Less => {
                self.cur_val = self.cur_val * UNITY + f;
                Ok(false)
            },
            _ => Err(TeXError::Arith)
        }
    }

    // Section 455
    // returns `true` if goto attach_sign
    fn sec455_scan_for_units_that_are_internal_dimensions(&mut self, f: Integer, mu: bool) -> TeXResult<bool> {
        let save_cur_val = self.cur_val;
        sec406_get_next_nonblank_noncall_token!(self);
        let v = 'block: {
            if self.cur_cmd < MIN_INTERNAL || self.cur_cmd > MAX_INTERNAL {
                self.back_input()?;
            }
            else {
                if mu {
                    self.scan_something_internal(MU_VAL as QuarterWord, false)?;
                    // Section 451
                    if self.cur_val_level >= GLUE_VAL {
                        let v = width(self.cur_val);
                        self.delete_glue_ref(self.cur_val);
                        self.cur_val = v;
                    }
                    // End section 451
                    if self.cur_val_level != MU_VAL {
                        return Err(TeXError::IncompatibleGlueUnits);
                    }
                }
                else {
                    self.scan_something_internal(DIMEN_VAL as QuarterWord, false)?;
                }
                break 'block self.cur_val;
            }

            if mu {
                return Ok(false);
            }

            let v = if self.scan_keyword(b"em")? {
                // Section 558
                self.quad(cur_font() as QuarterWord)
                // End section 558
            }
            else if self.scan_keyword(b"ex")? {
                // Section 559
                self.x_height(cur_font() as QuarterWord)
                // End section 559
            }
            else {
                return Ok(false);
            };
            sec443_scan_an_optional_space!(self);
            v
        };

        // found:
        let (quo, _) = xn_over_d(v, f, 65536)?;
        self.cur_val = nx_plus_y!(save_cur_val, v, quo);
        Ok(true)
    }

    // Section 458
    // returns `true` if goto done
    fn sec458_scan_for_all_other_units(&mut self, f: &mut Integer) -> TeXResult<bool> {
        let (num, denom) = if self.scan_keyword(b"in")? {
            (7227, 100)
        }
        else if self.scan_keyword(b"pc")? {
            (12, 1)
        }
        else if self.scan_keyword(b"cm")? {
            (7227, 254)
        }
        else if self.scan_keyword(b"mm")? {
            (7227, 2540)
        }
        else if self.scan_keyword(b"bp")? {
            (7227, 7200)
        }
        else if self.scan_keyword(b"dd")? {
            (1238, 1157)
        }
        else if self.scan_keyword(b"cc")? {
            (14856, 1157)
        }
        else if self.scan_keyword(b"sp")? {
            return Ok(true); // Goto done in section 453
        }
        else {
            return Err(TeXError::IllegalUnitOfMeasurePt);
        };

        let rem: Scaled;
        (self.cur_val, rem) = xn_over_d(self.cur_val, num, denom)?;
        *f = (num * (*f) + 65536 * rem) / denom;
        self.cur_val += *f / 65536;
        *f %= 65536;
        Ok(false)
    }

    // Section 461
    pub(crate) fn scan_glue(&mut self, level: SmallNumber) -> TeXResult<()> {
        let mu = level == MU_VAL as QuarterWord;

        // Section 441
        let mut negative = false;
        loop {
            sec406_get_next_nonblank_noncall_token!(self);
            if self.cur_tok == OTHER_TOKEN + b'-' as HalfWord {
                negative = !negative;
                self.cur_tok = OTHER_TOKEN + b'+' as HalfWord;
            }
            if self.cur_tok != OTHER_TOKEN + b'+' as HalfWord {
                break;
            }
        }
        // End section 441

        if (MIN_INTERNAL..=MAX_INTERNAL).contains(&self.cur_cmd) {
            self.scan_something_internal(level, negative)?;
            if self.cur_val_level >= GLUE_VAL {
                if self.cur_val_level != (level as Integer) {
                    return Err(TeXError::IncompatibleGlueUnits);
                }
                return Ok(());
            }
            if self.cur_val_level == INT_VAL {
                self.scan_dimen(mu, false, true)?;
            }
            else if level == (MU_VAL as QuarterWord) {
                return Err(TeXError::IncompatibleGlueUnits);
            }
        }
        else {
            self.back_input()?;
            self.scan_dimen(mu, false, false)?;
            if negative {
                self.cur_val = -self.cur_val;
            }
        }

        // Section 462
        let q = self.new_spec(ZERO_GLUE)?;
        *width_mut(q) = self.cur_val;
        if self.scan_keyword(b"plus")? {
            self.scan_dimen(mu, true, false)?;
            *stretch_mut(q) = self.cur_val;
            *stretch_order_mut(q) = self.cur_order;
        }
        if self.scan_keyword(b"minus")? {
            self.scan_dimen(mu, true, false)?;
            *shrink_mut(q) = self.cur_val;
            *shrink_order_mut(q) = self.cur_order;
        }
        self.cur_val = q;
        // End section 462

        Ok(())
    }

    // Section 463
    pub(crate) fn scan_rule_spec(&mut self) -> TeXResult<HalfWord> {
        let q = self.new_rule()?;
        if self.cur_cmd == VRULE {
            *width_mut(q) = DEFAULT_RULE;
        }
        else {
            *height_mut(q) = DEFAULT_RULE;
            *depth_mut(q) = 0;
        }

        // reswitch:
        loop {
            if self.scan_keyword(b"width")? {
                self.scan_dimen(false, false, false)?;
                *width_mut(q) = self.cur_val;
                continue;
            }
            if self.scan_keyword(b"height")? {
                self.scan_dimen(false, false, false)?;
                *height_mut(q) = self.cur_val;
                continue;
            }
            if self.scan_keyword(b"depth")? {
                self.scan_dimen(false, false, false)?;
                *depth_mut(q) = self.cur_val;
                continue;
            }
            break;
        }
        Ok(q)
    }
}

// Part 30: Font metric data

impl Global {
    // Section 577
    pub(crate) fn scan_font_ident(&mut self) -> TeXResult<()> {
        sec406_get_next_nonblank_noncall_token!(self);
        self.cur_val = match self.cur_cmd {
            DEF_FONT => cur_font(),

            SET_FONT => self.cur_chr,

            DEF_FAMILY => {
                let m = self.cur_chr;
                self.scan_four_bit_int()?;
                equiv(m + self.cur_val)
            },

            _ => return Err(TeXError::MissingFontIdentifier)
        };
        Ok(())
    }

    // Section 578
    pub(crate) fn find_font_dimen(&mut self, writing: bool) -> TeXResult<()> {
        self.scan_int()?;
        let n = self.cur_val;
        self.scan_font_ident()?;
        let f = self.cur_val;
        if n <= 0 {
            self.cur_val = self.fmem_ptr as Integer;
        }
        else {
            if writing && (SPACE_CODE..=SPACE_SHRINK_CODE).contains(&n) && self.font_glue[f as usize] != NULL {
                self.delete_glue_ref(self.font_glue[f as usize]);
                self.font_glue[f as usize] = NULL;
            }
            if n > (self.font_params[f as usize] as Integer) {
                if f < (self.font_ptr as Integer) {
                    self.cur_val = self.fmem_ptr as Integer;
                }
                else {
                    // Section 580
                    loop {
                        if self.fmem_ptr == FONT_MEM_SIZE as usize {
                            return Err(TeXError::Overflow("font memory", FONT_MEM_SIZE));
                        }
                        *self.font_info[self.fmem_ptr].sc_mut() = 0;
                        self.fmem_ptr += 1;
                        self.font_params[f as usize] += 1;
                        if n == (self.font_params[f as usize] as Integer) {
                            break;
                        }
                    }
                    self.cur_val = (self.fmem_ptr - 1) as Integer;
                    // End section 580
                }
            }
            else {
                self.cur_val = n + self.param_base[f as usize];
            }
        }

        // Section 579
        if self.cur_val == (self.fmem_ptr as Integer) {
            Err(TeXError::FontHasOnly(f as QuarterWord))
        }
        else {
            Ok(())
        }
    }

    // Section 645
    pub(crate) fn scan_spec(&mut self, c: Integer, three_codes: bool) -> TeXResult<()> {
        let s = match three_codes {
            true => self.saved(0),
            false => 0,
        };

        let spec_code = 'block: {
            let spec_code = if self.scan_keyword(b"to")? {
                EXACTLY
            }
            else if self.scan_keyword(b"spread")? {
                ADDITIONAL
            }
            else {
                self.cur_val = 0;
                break 'block ADDITIONAL;
            };
            self.scan_dimen(false, false, false)?;
            spec_code
        };

        // found:
        if three_codes {
            *self.saved_mut(0) = s;
            self.save_ptr += 1;
        }
        *self.saved_mut(0) = spec_code as Integer;
        *self.saved_mut(1) = self.cur_val;
        self.save_ptr += 2;
        self.new_save_level(c)?;
        self.scan_left_brace()
    }
}
