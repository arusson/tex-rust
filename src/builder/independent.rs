use crate::arithmetic::{
    mult_and_add, x_over_n, xn_over_d
};
use crate::constants::*;
use crate::datastructures::{
    EQTB, MEM, r#box, equiv, equiv_mut, font_id_text_mut, geq_word_define,
    global_defs, glue_ref_count_mut, info, info_mut, link, link_mut,
    shrink, shrink_mut, shrink_order, shrink_order_mut, stretch,
    stretch_mut, stretch_order, stretch_order_mut, text, token_ref_count_mut,
    width, width_mut
};
use crate::error::{TeXError, TeXResult};
use crate::io::AlphaFileInSelector;
use crate::strings::{
    str_room, make_string, str_eq_str, flush_string, str_ptr, length
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, SmallNumber, StrNum,
    add_glue_ref, add_token_ref, back_list, eqtb, eqtb_mut,
    free_avail, hi, mem_mut, mult_integers, nx_plus_y, odd,
    sec404_get_next_nonblank_nonrelax_noncall_token, update_terminal
};
use std::io::Write;

// Part 49: Mode-independent processing

impl Global {
    // Section 1211
    pub(crate) fn prefixed_command(&mut self) -> TeXResult<()> {
        let mut a = 0;
        while self.cur_cmd == PREFIX {
            if !odd!(a / self.cur_chr) {
                a += self.cur_chr;
            }
            sec404_get_next_nonblank_nonrelax_noncall_token!(self);
            if self.cur_cmd <= MAX_NON_PREFIXED_COMMAND {
                return Err(TeXError::CantUsePrefix);
            }
        }

        // Section 1213
        if self.cur_cmd != DEF && a % 4 != 0 {
            return Err(TeXError::CantUseLongOuter);
        }
        // End section 1213

        // Section 1214
        macro_rules! global {
            () => {
                a >= 4
            };
        }
        
        macro_rules! define {
            ($($args:expr),*) => {
                if global!() {
                    self.geq_define($($args),*)?;
                }
                else {
                    self.eq_define($($args),*)?;
                }
            };
        }
        
        macro_rules! word_define {
            ($($args:expr),*) => {
                if global!() {
                    geq_word_define($($args),*);
                }
                else {
                    self.eq_word_define($($args),*)?;
                }
            };
        }

        if global_defs() != 0 {
            if global_defs() < 0 {
                if global!() {
                    a -= 4;
                }
            }
            else if !global!() {
                a += 4;
            }
        }
        // End section 1214

        match self.cur_cmd {
            // Section 1217
            SET_FONT => define!(CUR_FONT_LOC, DATA, self.cur_chr),

            // Section 1218
            DEF => {
                if odd!(self.cur_chr) && !global!() && global_defs() >= 0 {
                    a += 4;
                }
                let e = self.cur_chr >= 2;
                self.get_r_token()?;
                let p = self.cur_cs;
                _ = self.scan_toks(true, e)?;
                define!(p, CALL + (a % 4) as QuarterWord, self.def_ref);
            }
            // End section 1218

            // Section 1221
            LET => {
                let n = self.cur_chr;
                self.get_r_token()?;
                let p = self.cur_cs;
                if n == NORMAL as HalfWord {
                    loop {
                        self.get_token()?;
                        if self.cur_cmd != SPACER {
                            break;
                        }
                    }
                    if self.cur_tok == OTHER_TOKEN + b'=' as HalfWord {
                        self.get_token()?;
                        if self.cur_cmd == SPACER {
                            self.get_token()?;
                        }
                    }
                }
                else {
                    self.get_token()?;
                    let q = self.cur_tok;
                    self.get_token()?;
                    self.back_input()?;
                    self.cur_tok = q;
                    self.back_input()?;
                }
                if self.cur_cmd >= CALL {
                    add_token_ref!(self.cur_chr);
                }
                define!(p, self.cur_cmd, self.cur_chr);
            }
            // End section 1221

            // Section 1224
            SHORTHAND_DEF => {
                let n = self.cur_chr;
                self.get_r_token()?;
                let p = self.cur_cs;
                define!(p, RELAX, 256);
                self.scan_optional_equals()?;
                match n {
                    CHAR_DEF_CODE => {
                        self.scan_char_num()?;
                        define!(p, CHAR_GIVEN, self.cur_val);
                    },

                    MATH_CHAR_DEF_CODE => {
                        self.scan_fifteen_bit_int()?;
                        define!(p, MATH_GIVEN, self.cur_val);
                    }

                    _ => {
                        self.scan_eight_bit_int()?;
                        match n {
                            COUNT_DEF_CODE => define!(p, ASSIGN_INT, COUNT_BASE + self.cur_val),
                            DIMEN_DEF_CODE => define!(p, ASSIGN_DIMEN, SCALED_BASE + self.cur_val),
                            SKIP_DEF_CODE => define!(p, ASSIGN_GLUE, SKIP_BASE + self.cur_val),
                            MU_SKIP_DEF_CODE => define!(p, ASSIGN_MU_GLUE, MU_SKIP_BASE + self.cur_val),
                            _ /* TOKS_DEF_CODE */ => define!(p, ASSIGN_TOKS, TOKS_BASE + self.cur_val),
                        }
                    }
                }
            },
            // End section 1224

            // Section 1225
            READ_TO_CS => {
                self.scan_int()?;
                let n = self.cur_val;
                if !self.scan_keyword(b"to")? {
                    return Err(TeXError::MissingTo2);
                }
                self.get_r_token()?;
                let p = self.cur_cs;
                self.read_toks(n, p)?;
                define!(p, CALL, self.cur_val);
            }
            // End section 1225

            // Section 1226
            TOKS_REGISTER
            | ASSIGN_TOKS => 'toks: {
                let mut q = self.cur_cs;
                let p = if self.cur_cmd == TOKS_REGISTER {
                    self.scan_eight_bit_int()?;
                    TOKS_BASE + self.cur_val
                }
                else {
                    self.cur_chr
                };

                self.scan_optional_equals()?;
                sec404_get_next_nonblank_nonrelax_noncall_token!(self);
                if self.cur_cmd != LEFT_BRACE {
                    // Section 1227
                    if self.cur_cmd == TOKS_REGISTER {
                        self.scan_eight_bit_int()?;
                        self.cur_cmd = ASSIGN_TOKS;
                        self.cur_chr = TOKS_BASE + self.cur_val;
                    }
                    if self.cur_cmd == ASSIGN_TOKS {
                        q = equiv(self.cur_chr);
                        if q == NULL {
                            define!(p, UNDEFINED_CS, NULL);
                        }
                        else {
                            add_token_ref!(q);
                            define!(p, CALL, q);
                        }
                        break 'toks; // Goto done
                    }
                    // End section 1227
                }
                self.back_input()?;
                self.cur_cs = q;
                q = self.scan_toks(false, false)?;
                if link(self.def_ref) == NULL {
                    define!(p, UNDEFINED_CS, NULL);
                    free_avail!(self, self.def_ref);
                }
                else {
                    if p == OUTPUT_ROUTINE_LOC {
                        *link_mut(q) = self.get_avail()?;
                        q = link(q);
                        *info_mut(q) = RIGHT_BRACE_TOKEN + b'}' as HalfWord;
                        q = self.get_avail()?;
                        *info_mut(q) = LEFT_BRACE_TOKEN + b'{' as HalfWord;
                        *link_mut(q) = link(self.def_ref);
                        *link_mut(self.def_ref) = q;
                    }
                    define!(p, CALL, self.def_ref);
                }
            },
            // End section 1226

            // Section 1228
            ASSIGN_INT => {
                let p = self.cur_chr;
                self.scan_optional_equals()?;
                self.scan_int()?;
                word_define!(p, self.cur_val);
            },

            ASSIGN_DIMEN => {
                let p = self.cur_chr;
                self.scan_optional_equals()?;
                self.scan_dimen(false, false, false)?;
                word_define!(p, self.cur_val);
            },

            ASSIGN_GLUE
            | ASSIGN_MU_GLUE => {
                let p = self.cur_chr;
                let n = self.cur_cmd;
                self.scan_optional_equals()?;
                if n == ASSIGN_MU_GLUE {
                    self.scan_glue(MU_VAL as QuarterWord)?;
                }
                else {
                    self.scan_glue(GLUE_VAL as QuarterWord)?;
                }
                self.trap_zero_glue();
                define!(p, GLUE_REF, self.cur_val);
            },
            // End section 1228

            // Section 1232
            DEF_CODE => {
                // Section 1233
                let n = match self.cur_chr {
                    CAT_CODE_BASE => MAX_CHAR_CODE as Integer,
                    MATH_CODE_BASE => 32768,
                    SF_CODE_BASE => 0x7fff,
                    DEL_CODE_BASE => 0xff_ffff,
                    _ => 255
                };
                // End section 1233

                let mut p = self.cur_chr;
                self.scan_char_num()?;
                p += self.cur_val;
                self.scan_optional_equals()?;
                self.scan_int()?;
                if self.cur_val < 0 && p < DEL_CODE_BASE || self.cur_val > n {
                    return Err(TeXError::InvalidCode(n, p));
                }
                if p < MATH_CODE_BASE {
                    define!(p, DATA, self.cur_val);
                }
                else if p < DEL_CODE_BASE {
                    define!(p, DATA, hi!(self.cur_val));
                }
                else {
                    word_define!(p, self.cur_val);
                }
            },
            // End section 1232

            // Section 1234
            DEF_FAMILY => {
                let mut p = self.cur_chr;
                self.scan_four_bit_int()?;
                p += self.cur_val;
                self.scan_optional_equals()?;
                self.scan_font_ident()?;
                define!(p, DATA, self.cur_val);
            }
            // End section 1234

            // Section 1235
            REGISTER
            | ADVANCE
            | MULTIPLY
            | DIVIDE => self.do_register_command(a as SmallNumber)?,
            // End section 1235

            // Section 1241
            SET_BOX => {
                self.scan_eight_bit_int()?;
                let n = if global!() {
                    256 + self.cur_val
                }
                else {
                    self.cur_val
                };

                self.scan_optional_equals()?;
                if self.set_box_allowed {
                    self.scan_box(BOX_FLAG + n)?;
                }
                else {
                    return Err(TeXError::ImproperSetbox);
                }
            },
            // End section 1241

            // Section 1242
            SET_AUX => self.alter_aux()?,

            SET_PREV_GRAF => self.alter_prev_graf()?,

            SET_PAGE_DIMEN => self.alter_page_so_far()?,

            SET_PAGE_INT => self.alter_integer()?,

            SET_BOX_DIMEN => self.alter_box_dimen()?,
            // End section 1242

            // Section 1248
            SET_SHAPE => {
                self.scan_optional_equals()?;
                self.scan_int()?;
                let n = self.cur_val;
                let p = if n <= 0 {
                    NULL
                }
                else {
                    let p = self.get_node(2*n + 1)?;
                    *info_mut(p) = n;
                    for j in 1..=n {
                        self.scan_dimen(false, false, false)?;
                        *mem_mut![(p + 2*j - 1) as usize].sc_mut() = self.cur_val;
                        self.scan_dimen(false, false, false)?;
                        *mem_mut![(p + 2*j) as usize].sc_mut() = self.cur_val;
                    }
                    p
                };
                define!(PAR_SHAPE_LOC, SHAPE_REF, p);
            },
            // End section 1248

            // Section 1252
            HYPH_DATA => {
                if self.cur_chr == 1 {
                    if self.initex_mode {
                        self.new_patterns()?;
                    }
                    else {
                        return Err(TeXError::PatternsOnlyIniTeX);
                    }
                }
                else {
                    self.new_hyph_exceptions()?;
                }
            },
            // End section 1252

            // Section 1253
            ASSIGN_FONT_DIMEN => {
                self.find_font_dimen(true)?;
                let k = self.cur_val;
                self.scan_optional_equals()?;
                self.scan_dimen(false, false, false)?;
                *self.font_info[k as usize].sc_mut() = self.cur_val;
            },

            ASSIGN_FONT_INT => {
                let n = self.cur_chr;
                self.scan_font_ident()?;
                let f = self.cur_val;
                self.scan_optional_equals()?;
                self.scan_int()?;
                if n == 0 {
                    self.hyphen_char[f as usize] = self.cur_val;
                }
                else {
                    self.skew_char[f as usize] = self.cur_val;
                }
            },
            // End section 1253

            // Section 1256
            DEF_FONT => self.new_font(a as SmallNumber)?,
            // End section 1256

            // Section 1264
            SET_INTERACTION => self.new_interaction(),
            // End section 1264
            // End section 1217

            _ => return Err(TeXError::Confusion("prefix")),
        }

        // done:
        // Section 1269
        if self.after_token != 0 {
            self.cur_tok = self.after_token;
            self.back_input()?;
            self.after_token = 0;
        }
        // End section 1269

        Ok(())
    }

    // Section 1215
    fn get_r_token(&mut self) -> TeXResult<()> {
        loop {
            self.get_token()?;
            if self.cur_tok != SPACE_TOKEN {
                break;
            }
        }
        if self.cur_cs == 0 || self.cur_cs > FROZEN_CONTROL_SEQUENCE {
            Err(TeXError::MissingControlSequence)
        }
        else {
            Ok(())
        }
    }

    // Section 1229
    fn trap_zero_glue(&mut self) {
        if width(self.cur_val) == 0
            && stretch(self.cur_val) == 0
            && shrink(self.cur_val) == 0
        {
            add_glue_ref!(ZERO_GLUE);
            self.delete_glue_ref(self.cur_val);
            self.cur_val = ZERO_GLUE;
        }
    }

    // Section 1236
    fn do_register_command(&mut self, a: SmallNumber) -> TeXResult<()> {
        macro_rules! global {
            () => {
                a >= 4
            };
        }
    
        macro_rules! define {
            ($($args:expr),*) => {
                if global!() {
                    self.geq_define($($args),*)?;
                }
                else {
                    self.eq_define($($args),*)?;
                }
            };
        }

        macro_rules! word_define {
            ($($args:expr),*) => {
                if global!() {
                    geq_word_define($($args),*);
                }
                else {
                    self.eq_word_define($($args),*)?;
                }
            };
        }
        
        let q = self.cur_cmd;
        
        // Section 1237
        let (l, p) = 'sec1237: {
            if q != REGISTER {
                self.get_x_token()?;
                if (ASSIGN_INT..=ASSIGN_MU_GLUE).contains(&self.cur_cmd) {
                    // Goto found
                    break 'sec1237 (self.cur_chr, (self.cur_cmd as Integer) - (ASSIGN_INT as Integer));
                }
                if self.cur_cmd != REGISTER {
                    return Err(TeXError::CantUseAfterCmd(q as QuarterWord));
                }
            }
            let p = self.cur_chr;
            self.scan_eight_bit_int()?;
            match p {
                INT_VAL => (self.cur_val + COUNT_BASE, p),
                DIMEN_VAL => (self.cur_val + SCALED_BASE, p),
                GLUE_VAL => (self.cur_val + SKIP_BASE, p),
                _ /* MU_VAL */ => (self.cur_val + MU_SKIP_BASE, p),
            }
        };
        // found:
        // End section 1237

        if q == REGISTER {
            self.scan_optional_equals()?;
        }
        else {
            // optional 'by'
            _ = self.scan_keyword(b"by")?;
        }
        if q < MULTIPLY {
            // Section 1238
            if p < GLUE_VAL {
                if p == INT_VAL {
                    self.scan_int()?;
                }
                else {
                    self.scan_dimen(false, false, false)?;
                }
                if q == ADVANCE {
                    self.cur_val += eqtb![l as usize].int()
                }
            }
            else {
                self.scan_glue(p as QuarterWord)?;
                if q == ADVANCE {
                    // Section 1239
                    let q = self.new_spec(self.cur_val)?;
                    let r = equiv(l);
                    self.delete_glue_ref(self.cur_val);
                    *width_mut(q) += width(r);
                    if stretch(q) == 0 {
                        *stretch_order_mut(q) = NORMAL;
                    }
                    if stretch_order(q) == stretch_order(r) {
                        *stretch_mut(q) += stretch(r);
                    }
                    else if stretch_order(q) < stretch_order(r) && stretch(r) != 0 {
                        *stretch_mut(q) = stretch(r);
                        *stretch_order_mut(q) = stretch_order(r);
                    }
                    if shrink(q) == 0 {
                        *shrink_order_mut(q) = NORMAL;
                    }
                    if shrink_order(q) == shrink_order(r) {
                        *shrink_mut(q) += shrink(r);
                    }
                    else if shrink_order(q) < shrink_order(r) && shrink(r) != 0 {
                        *shrink_mut(q) = shrink(r);
                        *shrink_order_mut(q) = shrink_order(r);
                    }
                    self.cur_val = q;
                    // End section 1239
                }
            }
            // End section 1238
        }
        else {
            // Section 1240
            self.scan_int()?;
            self.cur_val = if p < GLUE_VAL {
                if q == MULTIPLY {
                    if p == INT_VAL {
                        mult_integers!(eqtb![l as usize].int(), self.cur_val)
                    }
                    else {
                        nx_plus_y!(eqtb![l as usize].int(), self.cur_val, 0)
                    }
                }
                else {
                    x_over_n(eqtb![l as usize].int(), self.cur_val)?.0
                }
            }
            else {
                let s = equiv(l);
                let r = self.new_spec(s)?;
                (
                    *width_mut(r),
                    *stretch_mut(r),
                    *shrink_mut(r)
                ) = if q == MULTIPLY {
                    (
                        nx_plus_y!(width(s), self.cur_val, 0),
                        nx_plus_y!(stretch(s), self.cur_val, 0),
                        nx_plus_y!(shrink(s), self.cur_val, 0),
                    )
                }
                else {
                    (
                        x_over_n(width(s), self.cur_val)?.0,
                        x_over_n(stretch(s), self.cur_val)?.0,
                        x_over_n(shrink(s), self.cur_val)?.0,
                    )
                };
                r
            };
            // End section 1240
        }
        if p < GLUE_VAL {
            word_define!(l, self.cur_val);
        }
        else {
            self.trap_zero_glue();
            define!(l, GLUE_REF, self.cur_val);
        }
        Ok(())
    }

    // Section 1243
    fn alter_aux(&mut self) -> TeXResult<()> {
        if self.cur_chr != self.mode().abs() {
            return Err(TeXError::ReportIllegalCase);
        }
        let c = self.cur_chr;
        self.scan_optional_equals()?;
        if c == VMODE {
            self.scan_dimen(false, false, false)?;
            *self.prev_depth_mut() = self.cur_val;
        }
        else {
            self.scan_int()?;
            if self.cur_val <= 0 || self.cur_val > 32767 {
                return Err(TeXError::BadSpaceFactor);
            }
            *self.space_factor_mut() = self.cur_val;
        }
        Ok(())
    }

    // Section 1244
    fn alter_prev_graf(&mut self) -> TeXResult<()> {
        self.nest[self.nest_ptr] = self.cur_list;
        let mut p = self.nest_ptr;
        while self.nest[p].mode_field.abs() != VMODE {
            p -= 1;
        }
        self.scan_optional_equals()?;
        self.scan_int()?;
        if self.cur_val < 0 {
            return Err(TeXError::BadPrevGraf);
        }
        self.nest[p].pg_field = self.cur_val;
        self.cur_list = self.nest[self.nest_ptr];
        Ok(())
    }

    // Section 1245
    fn alter_page_so_far(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        self.scan_optional_equals()?;
        self.scan_dimen(false, false, false)?;
        self.page_so_far[c as usize] = self.cur_val;
        Ok(())
    }

    // Section 1246
    fn alter_integer(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        self.scan_optional_equals()?;
        self.scan_int()?;
        if c == 0 {
            self.dead_cycles = self.cur_val;
        }
        else {
            self.insert_penalties = self.cur_val;
        }
        Ok(())
    }

    // Section 1247
    fn alter_box_dimen(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        self.scan_eight_bit_int()?;
        let b = self.cur_val;
        self.scan_optional_equals()?;
        self.scan_dimen(false, false, false)?;
        if r#box(b) != NULL {
            *mem_mut![(r#box(b) + c) as usize].sc_mut() = self.cur_val;
        }
        Ok(())
    }

    // Section 1257
    fn new_font(&mut self, a: SmallNumber) -> TeXResult<()> {
        macro_rules! global {
            () => {
                a >= 4
            };
        }
        
        macro_rules! define {
            ($($args:expr),*) => {
                if global!() {
                    self.geq_define($($args),*)?;
                }
                else {
                    self.eq_define($($args),*)?;
                }
            };
        }

        if self.job_name == 0 {
            self.open_log_file()?;
        }
        self.get_r_token()?;
        let u = self.cur_cs;
        let t = if u >= HASH_BASE {
            text(u)
        }
        else if u >= SINGLE_BASE {
            if u == NULL_CS {
                FONT_STRING as HalfWord // "FONT"
            }
            else {
                u - SINGLE_BASE
            }
        }
        else {
            let old_setting = self.selector;
            self.selector = NEW_STRING;
            self.print("FONT");
            self.print_strnumber((u - ACTIVE_BASE) as StrNum);
            self.selector = old_setting;
            str_room(1)?;
            make_string()? as HalfWord
        };

        define!(u, SET_FONT, NULL_FONT);
        self.scan_optional_equals()?;
        self.scan_file_name()?;
        
        // Section 1258
        self.name_in_progress = true;
        let s = if self.scan_keyword(b"at")? {
            // Section 1259
            self.scan_dimen(false, false, false)?;
            let s = self.cur_val;
            if s <= 0 || s >= 0x800_0000 {
                return Err(TeXError::ImproperAt(s));
            }
            // End section 1259
            s
        }
        else if self.scan_keyword(b"scaled")? {
            self.scan_int()?;
            let s = -self.cur_val;
            if self.cur_val <= 0 || self.cur_val >= 32768 {
                return Err(TeXError::IllegalMag(self.cur_val));
            }
            s
        }
        else {
            -1000
        };

        self.name_in_progress = false;
        // End section 1258

        let f = 'block: {
            // Section 1260
            let flushable_string = str_ptr() - 1;
            for f in (FONT_BASE as usize + 1)..=(self.font_ptr as usize) {
                if str_eq_str(self.cur_name, self.font_name[f]) && str_eq_str(self.font_area[f], self.cur_area) {
                    if self.cur_name == flushable_string {
                        flush_string();
                        self.cur_name = self.font_name[f];
                    }
                    if s > 0 {
                        if s == self.font_size[f] {
                            break 'block f; // Goto common_ending
                        }
                    }
                    else if self.font_size[f] == xn_over_d(self.font_dsize[f], -s, 1000)?.0 {
                        break 'block f; // Goto common_ending
                    }
                }
            }
            // End section 1260

            self.read_font_info(u, self.cur_name, self.cur_area, s)?
        };
    
        // common_ending:
        *equiv_mut(u) = f as HalfWord;
        *eqtb_mut![FONT_ID_BASE as usize + f] = eqtb![u as usize];
        *font_id_text_mut(f as QuarterWord) = t;
        Ok(())
    }

    // Section 1265
    fn new_interaction(&mut self) {
        self.print_ln();
        self.interaction = self.cur_chr;
        self.sec75_initialize_print_selector();
        if self.log_opened {
            self.selector += 2;
        }
    }

    // Section 1270
    pub(crate) fn do_assignments(&mut self) -> TeXResult<()> {
        loop {
            sec404_get_next_nonblank_nonrelax_noncall_token!(self);
            if self.cur_cmd <= MAX_NON_PREFIXED_COMMAND {
                break;
            }
            self.set_box_allowed = false;
            self.prefixed_command()?;
            self.set_box_allowed = true;
        }
        Ok(())
    }

    // Section 1275
    pub(crate) fn open_or_close_in(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        self.scan_four_bit_int()?;
        let n = self.cur_val;
        if self.read_open[n as usize] != CLOSED {
            self.read_file[n as usize].close();
            self.read_open[n as usize] = CLOSED;
        }
        if c != 0 {
            self.scan_optional_equals()?;
            self.scan_file_name()?;
            if self.cur_ext == EMPTY_STRING {
                self.cur_ext = EXT_TEX;
            }
            self.pack_cur_name();
            if self.a_open_in(AlphaFileInSelector::ReadFile(n as usize)) {
                self.read_open[n as usize] = JUST_OPEN;
            }
        }
        Ok(())
    }

    // Section 1279
    pub(crate) fn issue_message(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        *link_mut(GARBAGE) = self.scan_toks(false, true)?;
        let old_setting = self.selector;
        self.selector = NEW_STRING;
        self.token_show(self.def_ref);
        self.selector = old_setting;
        self.flush_list(self.def_ref);
        str_room(1)?;
        let s = make_string()? as StrNum;
        if c == 0 {
            // Section 1280
            if self.term_offset + length(s) as Integer > MAX_PRINT_LINE - 2 {
                self.print_ln();
            }
            else if self.term_offset > 0 || self.file_offset > 0 {
                self.print_char(b' ');
            }
            self.slow_print(s);
            update_terminal!();
            // End section 1280
        }
        else {
            return Err(TeXError::ErrMessage(s));
        }
        flush_string();
        Ok(())
    }

    // Section 1288
    pub(crate) fn shift_case(&mut self) -> TeXResult<()> {
        let b = self.cur_chr;
        let _ = self.scan_toks(false, false)?;
        let mut p = link(self.def_ref);
        while p != NULL {
            // Section 1289
            let t = info(p);
            if t < CS_TOKEN_FLAG + SINGLE_BASE {
                let c = t % 256;
                if equiv(b + c) != 0 {
                    *info_mut(p) = t - c + equiv(b + c);
                }
            }
            // End section 1289
            p = link(p);
        }
        back_list!(self, link(self.def_ref));
        free_avail!(self, self.def_ref);
        Ok(())
    }
}
