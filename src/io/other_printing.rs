use crate::arithmetic::x_over_n;
use crate::builder::broken_ins;
use crate::constants::*;
use crate::datastructures::{
    r#box, cat_code, count, height, link, subtype, text, tracing_online, r#type
};
use crate::error::TeXResult;
use crate::extensions::write_stream;
use crate::strings::str_ptr;
use crate::{
    Global, HalfWord, Integer, Scaled, StrNum,
    page_goal, page_shrink, page_total
};

#[cfg(feature = "stat")]
use crate::{
    eqtb, hash, ho,
    datastructures::{
        EQTB, HASH, eq_type, equiv, info, par_shape_ptr},
};

use std::cmp::Ordering::{Equal, Greater, Less};

impl Global {
    // Section 211
    pub(crate) fn print_mode(&mut self, m: Integer) {
        match m.cmp(&0) {
            Greater => {
                match m / (MAX_COMMAND as Integer + 1) {
                    0 => self.print("vertical"),
                    1 => self.print("horizontal"),
                    // Last one can only be 2
                    _ => self.print("display math"),
                }
            },
            Equal => self.print("no"),
            Less => {
                match (-m) / (MAX_COMMAND as Integer + 1) {
                    0 => self.print("internal vertical"),
                    1 => self.print("restricted horizontal"),
                    // Last one can only be 2
                    _ => self.print("math"),
                }
            }
        }
        self.print(" mode");
    }

    // Section 218
    pub(crate) fn show_activities(&mut self) -> TeXResult<()> {
        self.nest[self.nest_ptr] = self.cur_list;
        self.print_nl("");
        self.print_ln();
        for p in (0..=self.nest_ptr).rev() {
            let m = self.nest[p].mode_field;
            let a = self.nest[p].aux_field;
            self.print_nl("### ");
            self.print_mode(m);
            self.print(" entered at line ");
            self.print_int(self.nest[p].ml_field.abs());
            if m == HMODE && self.nest[p].pg_field != 0x83_0000 {
                self.print(" (language");
                self.print_int(self.nest[p].pg_field % 65536);
                self.print(":hyphemin");
                self.print_int(self.nest[p].pg_field / 0x40_0000);
                self.print_char(b',');
                self.print_int((self.nest[p].pg_field / 65536) % 64);
                self.print_char(b')');
            }
            if self.nest[p].ml_field < 0 {
                self.print(" (\\output routine)");
            }
            if p == 0 {
                // Section 986
                if PAGE_HEAD != self.page_tail {
                    self.print_nl("### current page:");
                    if self.output_active {
                        self.print(" (held over for next output");
                    }
                    self.show_box(link(PAGE_HEAD));
                    if (self.page_contents as HalfWord) > EMPTY {
                        self.print_nl("total height ");
                        self.print_totals();
                        self.print_nl(" goal height ");
                        self.print_scaled(page_goal!(self));
                        let mut r = link(PAGE_INS_HEAD);
                        while r != PAGE_INS_HEAD {
                            self.print_ln();
                            self.print_esc("insert");
                            let mut t = subtype(r) as Scaled;
                            self.print_int(t);
                            self.print(" adds ");
                            t = if count(t) == 1000 {
                                height(r)
                            }
                            else {
                                x_over_n(height(r), 1000)?.0*count(t)
                            };
                            self.print_scaled(t);
                            if r#type(r) == SPLIT_UP {
                                let mut q = PAGE_HEAD;
                                t = 0;
                                loop {
                                    q = link(q);
                                    if r#type(q) == INS_NODE && subtype(q) == subtype(r) {
                                        t += 1;
                                    }
                                    if q == broken_ins(r) {
                                        break;
                                    }
                                }
                                self.print(", #");
                                self.print_int(t);
                                self.print(" might split");
                            }
                            r = link(r);
                        }
                    }
                }
                // End section 986

                if link(CONTRIB_HEAD) != NULL {
                    self.print_nl("### recent contributions:");
                }
            }
            self.show_box(link(self.nest[p].head_field));

            // Section 219
            match m.abs() / (MAX_COMMAND + 1) as Integer {
                0 => {
                    self.print_nl("prev_depth ");
                    if a.sc() <= IGNORE_DEPTH {
                        self.print("ignored");
                    }
                    else {
                        self.print_scaled(a.sc());
                    }
                    if self.nest[p].pg_field != 0 {
                        self.print(", prevgraph ");
                        self.print_int(self.nest[p].pg_field);
                        self.print(" line");
                        if self.nest[p].pg_field != 1 {
                            self.print_char(b's');
                        }
                    }
                },

                1 => {
                    self.print_nl("spacefactor ");
                    self.print_int(a.hh_lh());
                    if m > 0 && a.hh_lh() > 0 {
                        self.print(", current language ");
                        self.print_int(a.hh_rh());
                    }
                }

                _ /* 2 */ => {
                    if a.int() != NULL {
                        self.print("this will be denominator of :");
                        self.show_box(a.int());
                    }
                }
            }
            // End secton 219
        }
        Ok(())
    }

    // Section 225
    pub(crate) fn print_skip_param(&mut self, n: Integer) {
        match n {
            LINE_SKIP_CODE => self.print_esc("lineskip"),
            BASELINE_SKIP_CODE => self.print_esc("baselineskip"),
            PAR_SKIP_CODE => self.print_esc("parskip"),
            ABOVE_DISPLAY_SKIP_CODE => self.print_esc("abovedisplayskip"),
            BELOW_DISPLAY_SKIP_CODE => self.print_esc("belowdisplayskip"),
            ABOVE_DISPLAY_SHORT_SKIP_CODE => self.print_esc("abovedisplayshortskip"),
            BELOW_DISPLAY_SHORT_SKIP_CODE => self.print_esc("belowdisplayshortskip"),
            LEFT_SKIP_CODE => self.print_esc("leftskip"),
            RIGHT_SKIP_CODE => self.print_esc("rightskip"),
            TOP_SKIP_CODE => self.print_esc("topskip"),
            SPLIT_TOP_SKIP_CODE => self.print_esc("splittopskip"),
            TAB_SKIP_CODE => self.print_esc("tabskip"),
            SPACE_SKIP_CODE => self.print_esc("spaceskip"),
            XSPACE_SKIP_CODE => self.print_esc("xspaceskip"),
            PAR_FILL_SKIP_CODE => self.print_esc("parfillskip"),
            THIN_MU_SKIP_CODE => self.print_esc("thinmuskip"),
            MED_MU_SKIP_CODE => self.print_esc("medmuskip"),
            THICK_MU_SKIP_CODE => self.print_esc("thickmuskip"),
            _ => self.print("[unknown glue parameter!]"),
        }
    }

    // Section 237
    pub(crate) fn print_param(&mut self, n: Integer) {
        match n {
            PRETOLERANCE_CODE => self.print_esc("pretolerance"),
            TOLERANCE_CODE => self.print_esc("tolerance"),
            LINE_PENALTY_CODE => self.print_esc("linepenalty"),
            HYPHEN_PENALTY_CODE => self.print_esc("hyphenpenalty"),
            EX_HYPHEN_PENALTY_CODE => self.print_esc("exhyphenpenalty"),
            CLUB_PENALTY_CODE => self.print_esc("clubpenalty"),
            WIDOW_PENALTY_CODE => self.print_esc("widowpenalty"),
            DISPLAY_WIDOW_PENALTY_CODE => self.print_esc("displaywidowpenalty"),
            BROKEN_PENALTY_CODE => self.print_esc("brokenpenalty"),
            BIN_OP_PENALTY_CODE => self.print_esc("binoppenalty"),
            REL_PENALTY_CODE => self.print_esc("relpenalty"),
            PRE_DISPLAY_PENALTY_CODE => self.print_esc("predisplaypenalty"),
            POST_DISPLAY_PENALTY_CODE => self.print_esc("postdisplaypenalty"),
            INTER_LINE_PENALTY_CODE => self.print_esc("interlinepenalty"),
            DOUBLE_HYPHEN_DEMERITS_CODE => self.print_esc("doublehyphendemerits"),
            FINAL_HYPHEN_DEMERITS_CODE => self.print_esc("finalhyphendemerits"),
            ADJ_DEMERITS_CODE => self.print_esc("adjdemerits"),
            MAG_CODE => self.print_esc("mag"),
            DELIMITER_FACTOR_CODE => self.print_esc("delimiterfactor"),
            LOOSENESS_CODE => self.print_esc("looseness"),
            TIME_CODE => self.print_esc("time"),
            DAY_CODE => self.print_esc("day"),
            MONTH_CODE => self.print_esc("month"),
            YEAR_CODE => self.print_esc("year"),
            SHOW_BOX_BREADTH_CODE => self.print_esc("showboxbreadth"),
            SHOW_BOX_DEPTH_CODE => self.print_esc("showboxdepth"),
            HBADNESS_CODE => self.print_esc("hbadness"),
            VBADNESS_CODE => self.print_esc("vbadness"),
            PAUSING_CODE => self.print_esc("pausing"),
            TRACING_ONLINE_CODE => self.print_esc("tracingonline"),
            TRACING_MACROS_CODE => self.print_esc("tracingmacros"),
            TRACING_STATS_CODE => self.print_esc("tracingstats"),
            TRACING_PARAGRAPHS_CODE => self.print_esc("tracingparagraphs"),
            TRACING_PAGES_CODE => self.print_esc("tracingpages"),
            TRACING_OUTPUT_CODE => self.print_esc("tracingoutput"),
            TRACING_LOST_CHARS_CODE => self.print_esc("tracinglostchars"),
            TRACING_COMMANDS_CODE => self.print_esc("tracingcommands"),
            TRACING_RESTORES_CODE => self.print_esc("tracingrestores"),
            UC_HYPH_CODE => self.print_esc("uchyph"),
            OUTPUT_PENALTY_CODE => self.print_esc("outputpenalty"),
            MAX_DEAD_CYCLES_CODE => self.print_esc("maxdeadcycles"),
            HANG_AFTER_CODE => self.print_esc("hangafter"),
            FLOATING_PENALTY_CODE => self.print_esc("floatingpenalty"),
            GLOBAL_DEFS_CODE => self.print_esc("globaldefs"),
            CUR_FAM_CODE => self.print_esc("fam"),
            ESCAPE_CHAR_CODE => self.print_esc("escapechar"),
            DEFAULT_HYPHEN_CHAR_CODE => self.print_esc("defaulthyphenchar"),
            DEFAULT_SKEW_CHAR_CODE => self.print_esc("defaultskewchar"),
            END_LINE_CHAR_CODE => self.print_esc("endlinechar"),
            NEW_LINE_CHAR_CODE => self.print_esc("newlinechar"),
            LANGUAGE_CODE => self.print_esc("language"),
            LEFT_HYPHEN_MIN_CODE => self.print_esc("lefthyphenmin"),
            RIGHT_HYPHEN_MIN_CODE => self.print_esc("righthyphenmin"),
            HOLDING_INSERTS_CODE => self.print_esc("holdinginserts"),
            ERROR_CONTEXT_LINES_CODE => self.print_esc("errorcontextlines"),
            _ => self.print("[unknown integer parameter!]"),
        }
    }

    // Sectin 247
    pub(crate) fn print_length_param(&mut self, n: Integer) {
        match n {
            PAR_INDENT_CODE => self.print_esc("parindent"),
            MATH_SURROUND_CODE => self.print_esc("mathsurround"),
            LINE_SKIP_LIMIT_CODE => self.print_esc("lineskiplimit"),
            HSIZE_CODE => self.print_esc("hsize"),
            VSIZE_CODE => self.print_esc("vsize"),
            MAX_DEPTH_CODE => self.print_esc("maxdepth"),
            SPLIT_MAX_DEPTH_CODE => self.print_esc("splitmaxdepth"),
            BOX_MAX_DEPTH_CODE => self.print_esc("boxmaxdepth"),
            HFUZZ_CODE => self.print_esc("hfuzz"),
            VFUZZ_CODE => self.print_esc("vfuzz"),
            DELIMITER_SHORTFALL_CODE => self.print_esc("delimitershortfall"),
            NULL_DELIMITER_SPACE_CODE => self.print_esc("nulldelimiterspace"),
            SCRIPT_SPACE_CODE => self.print_esc("scriptspace"),
            PRE_DISPLAY_SIZE_CODE => self.print_esc("predisplaysize"),
            DISPLAY_WIDTH_CODE => self.print_esc("displaywidth"),
            DISPLAY_INDENT_CODE => self.print_esc("displayindent"),
            OVERFULL_RULE_CODE => self.print_esc("overfullrule"),
            HANG_INDENT_CODE => self.print_esc("hangindent"),
            H_OFFSET_CODE => self.print_esc("hoffset"),
            V_OFFSET_CODE => self.print_esc("voffset"),
            EMERGENCY_STRETCH_CODE => self.print_esc("emergencystretch"),
            _ => self.print("[unknown dimen parameter!]"),
        }
    }

    // Section 245
    pub(crate) fn begin_diagnostic(&mut self) {
        self.old_setting = self.selector;
        if tracing_online() <= 0 && self.selector == TERM_AND_LOG {
            self.selector -= 1;
            if self.history == SPOTLESS {
                self.history = WARNING_ISSUED;
            }
        }
    }

    pub(crate) fn end_diagnostic(&mut self, blank_line: bool) {
        self.print_nl("");
        if blank_line {
            self.print_ln();
        }
        self.selector = self.old_setting;
    }

    // Section 252
    #[cfg(feature = "stat")]
    pub(crate) fn show_eqtb(&mut self, n: HalfWord) {
        if n < ACTIVE_BASE {
            self.print_char(b'?');
        }
        else if n < GLUE_BASE {
            // Section 223
            self.sprint_cs(n);
            self.print_char(b'=');
            self.print_cmd_chr(eq_type(n), equiv(n));
            if eq_type(n) >= CALL {
                self.print_char(b':');
                self.show_token_list(link(equiv(n)), NULL, 32);
            }
            // End section 223
        }
        else if n < LOCAL_BASE {
            // Section 229
            if n < SKIP_BASE {
                self.print_skip_param(n - GLUE_BASE);
                self.print_char(b'=');
                if n < GLUE_BASE + THIN_MU_SKIP_CODE {
                    self.print_spec(equiv(n), "pt");
                }
                else {
                    self.print_spec(equiv(n), "mu");
                }
            }
            else if n < MU_SKIP_BASE {
                self.print_esc("skip");
                self.print_int(n - SKIP_BASE);
                self.print_char(b'=');
                self.print_spec(equiv(n), "pt");
            }
            else {
                self.print_esc("muskip");
                self.print_int(n - MU_SKIP_BASE);
                self.print_char(b'=');
                self.print_spec(equiv(n), "mu");
            }
            // End section 229
        }
        else if n < INT_BASE {
            // Section 233
            if n == PAR_SHAPE_LOC {
                self.print_esc("parshape");
                self.print_char(b'=');
                if par_shape_ptr() == NULL {
                    self.print_char(b'0');
                }
                else {
                    self.print_int(info(par_shape_ptr()));
                }
            }
            else if n < TOKS_BASE {
                self.print_cmd_chr(ASSIGN_TOKS, n);
                self.print_char(b'=');
                if equiv(n) != NULL {
                    self.show_token_list(link(equiv(n)), NULL, 32);
                }
            }
            else if n < BOX_BASE {
                self.print_esc("toks");
                self.print_int(n - TOKS_BASE);
                self.print_char(b'=');
                if equiv(n) != NULL {
                    self.show_token_list(link(equiv(n)), NULL, 32);
                }
            }
            else if n < CUR_FONT_LOC {
                self.print_esc("BOX");
                self.print_int(n - BOX_BASE);
                self.print_char(b'=');
                if equiv(n) == NULL {
                    self.print("void");
                }
                else {
                    self.depth_threshold = 0;
                    self.breadth_max = 1;
                    self.show_node_list(equiv(n));
                }
            }
            else if n < CAT_CODE_BASE {
                self.sec234_show_the_font_identifier(n);
            }
            else {
                self.sec235_show_the_halfword_code(n);
            }
            // End section 233
        }
        else if n < DIMEN_BASE {
            // Section 242
            if n < COUNT_BASE {
                self.print_param(n - INT_BASE);
            }
            else if n < DEL_CODE_BASE {
                self.print_esc("count");
                self.print_int(n - COUNT_BASE);
            }
            else {
                self.print_esc("delcode");
                self.print_int(n - DEL_CODE_BASE);
            }
            self.print_char(b'=');
            self.print_int(eqtb![n as usize].int());
            // End section 242
        }
        else if n <= EQTB_SIZE {
            // Section 251
            if n < SCALED_BASE {
                self.print_length_param(n - DIMEN_BASE);
            }
            else {
                self.print_esc("dimen");
                self.print_int(n - SCALED_BASE);
            }
            self.print_char(b'=');
            self.print_scaled(eqtb![n as usize].sc());
            self.print("pt");
            // End section 251
        }
        else {
            self.print_char(b'?');
        }
    }

    // Section 234
    #[cfg(feature = "stat")]
    fn sec234_show_the_font_identifier(&mut self, n: HalfWord) {
        if n == CUR_FONT_LOC {
            self.print("current font");
        }
        else if n < MATH_FONT_BASE + 16 {
            self.print_esc("textfont");
            self.print_int(n - MATH_FONT_BASE);
        }
        else if n < MATH_FONT_BASE + 32 {
            self.print_esc("scriptfont");
            self.print_int(n - MATH_FONT_BASE - 16);
        }
        else {
            self.print_esc("scriptscriptfont");
            self.print_int(n - MATH_FONT_BASE - 32);
        }
        self.print_char(b'=');
        self.print_esc_strnumber(
            hash![(FONT_ID_BASE + equiv(n)) as usize].hh_rh() as StrNum
        );
    }

    // Section 235
    #[cfg(feature = "stat")]
    fn sec235_show_the_halfword_code(&mut self, n: HalfWord) {
        if n < MATH_CODE_BASE {
            if n < LC_CODE_BASE{
                self.print_esc("catcode");
                self.print_int(n - CAT_CODE_BASE);
            }
            else if n < UC_CODE_BASE {
                self.print_esc("lccode");
                self.print_int(n - LC_CODE_BASE);
            }
            else if n < SF_CODE_BASE {
                self.print_esc("uccode");
                self.print_int(n - UC_CODE_BASE);
            }
            else {
                self.print_esc("sfcode");
                self.print_int(n - SF_CODE_BASE);
            }
            self.print_char(b'=');
            self.print_int(equiv(n))
        }
        else {
            self.print_esc("mathcode");
            self.print_int(n - MATH_CODE_BASE);
            self.print_char(b'=');
            self.print_int(ho!(equiv(n)));
        }
    }
}

impl Global {
    // Section 262
    pub(crate) fn print_cs(&mut self, p: Integer) {
        if p < HASH_BASE {
            if p >= SINGLE_BASE {
                if p == NULL_CS {
                    self.print_esc("csname");
                    self.print_esc("endcsname");
                    self.print_char(b' ');
                }
                else {
                    self.print_esc_strnumber((p - SINGLE_BASE) as StrNum);
                    if cat_code(p - SINGLE_BASE) == LETTER as HalfWord{
                        self.print_char(b' ');
                    }
                }
            }
            else if p < ACTIVE_BASE {
                self.print_esc("IMPOSSIBLE.");
            }
            else {
                self.print_strnumber((p - ACTIVE_BASE) as StrNum);
            }
        }
        else if p >= UNDEFINED_CONTROL_SEQUENCE {
            self.print_esc("IMPOSSIBLE.");
        }
        else if text(p) < 0 || text(p) >= (str_ptr() as Integer) {
            self.print_esc("NONEXISTENT.");
        }
        else {
            self.print_esc_strnumber(text(p) as StrNum);
            self.print_char(b' ');
        }
    }

    // Section 263
    pub(crate) fn sprint_cs(&mut self, p: HalfWord) {
        if p < HASH_BASE {
            if p < SINGLE_BASE {
                self.print_strnumber((p - ACTIVE_BASE) as StrNum);
            }
            else if p < NULL_CS {
                self.print_esc_strnumber((p - SINGLE_BASE) as StrNum);
            }
            else {
                self.print_esc("csname");
                self.print_esc("endcsname");
            }
        }
        else {
            self.print_esc_strnumber(text(p) as StrNum);
        }
    }

    // Section 985
    pub(crate) fn print_totals(&mut self) {
        macro_rules! print_plus {
            ($p:expr, $c:expr) => {
                if self.page_so_far[$p] != 0 {
                    self.print(" plus ");
                    self.print_scaled(self.page_so_far[$p]);
                    self.print($c);
                }
            };
        }

        self.print_scaled(page_total!(self));
        print_plus!(2, "");
        print_plus!(3, "fil");
        print_plus!(4, "fill");
        print_plus!(5, "filll");
        if page_shrink!(self) != 0 {
            self.print(" minus ");
            self.print_scaled(page_shrink!(self));
        }
    }

    // Section 1293
    pub(crate) fn show_whatever(&mut self) -> TeXResult<()> {
        'block: {
            match self.cur_chr {
                SHOW_LISTS => {
                    self.begin_diagnostic();
                    self.show_activities()?;
                },
                
                SHOW_BOX_CODE => {
                    // Section 1296
                    self.scan_eight_bit_int()?;
                    self.begin_diagnostic();
                    self.print_nl("> \\box");
                    self.print_int(self.cur_val);
                    self.print_char(b'=');
                    if r#box(self.cur_val) == NULL {
                        self.print("void");
                    }
                    else {
                        self.show_box(r#box(self.cur_val));
                    }
                    // End section 1296
                },

                SHOW_CODE => {
                    // Section 1294
                    self.get_token()?;
                    self.print_nl("> ");
                    if self.cur_cs != 0 {
                        self.sprint_cs(self.cur_cs);
                        self.print_char(b'=');
                    }
                    self.print_meaning();
                    break 'block; // Goto common_ending
                    // End section 1294
                },

                _ => {
                    // Section 1297
                    _ = self.the_toks()?;
                    self.print_nl("> ");
                    self.token_show(TEMP_HEAD);
                    self.flush_list(link(TEMP_HEAD));
                    break 'block; // Goto common_ending
                    // End section 1297
                }
            }

            // Section 1298
            self.end_diagnostic(true);
            self.print_nl("! OK");
            if self.selector == TERM_AND_LOG && tracing_online() <= 0 {
                self.selector = TERM_ONLY;
                self.print(" (see the transcript file)");
                self.selector = TERM_AND_LOG;
            }
            // End section 1298
        }

        // common_ending:
        // No support for interaction.
        Ok(())
    }

    // Section 1355
    pub(crate) fn print_write_whatsit(&mut self, s: &str, p: HalfWord) {
        self.print_esc(s);
        match write_stream(p).cmp(&16) {
            Less => self.print_int(write_stream(p)),
            Equal => self.print_char(b'*'),
            Greater => self.print_char(b'-'),
        }
    }
}
