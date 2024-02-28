use crate::arithmetic::xn_over_d;
use crate::constants::*;
use crate::datastructures::{
    character, character_mut, cur_font, every_job, every_vbox, font_mut,
    glue_ref_count_mut, hsize, language, lig_ptr, lig_ptr_mut, link, link_mut,
    math_code, sf_code, shrink, shrink_mut, space_skip, stretch, stretch_mut,
    subtype_mut, tracing_commands, type_mut, width_mut, xspace_skip
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, QuarterWord, fast_get_avail, free_avail,
    ho, nucleus, sec406_get_next_nonblank_noncall_token, tail_append
};

// Part 46: The chief executive

#[derive(PartialEq)]
enum Goto {
    MainLoop,
    AppendNormalSpace,
    MainLoopWrapup,
    MainLoopMove(usize),
    MainLoopLookAhead,
    MainLigLoop(usize),
    MainLoopMoveLig,
    ReSwitch,
    BigSwitch
}

impl Global {
    // Section 1030
    pub fn main_control(&mut self) -> TeXResult<()> {
        if every_job() != NULL {
            self.begin_token_list(every_job(), EVERY_JOB_TEXT)?;
        }

        'big_switch: loop {
            self.get_x_token()?;
            'reswitch: loop {
                
                // Section 1031
                if self.interrupt && self.ok_to_interrupt {
                    self.back_input()?;
                    self.check_interrupt()?;
                    continue 'big_switch;
                }
                #[cfg(feature = "debug")]
                if self.panicking {
                    self.check_mem(false);
                }
                if tracing_commands() > 0 {
                    self.show_cur_cmd_chr();
                }
                // End section 1031

                // Default goto value, if not changed in some cases.
                let mut goto = Goto::BigSwitch;
                match (self.mode().abs(), self.cur_cmd) {
                    (HMODE, LETTER)
                    | (HMODE, OTHER_CHAR)
                    | (HMODE, CHAR_GIVEN) => goto = Goto::MainLoop,

                    (HMODE, CHAR_NUM) => {
                        self.scan_char_num()?;
                        self.cur_chr = self.cur_val;
                        goto = Goto::MainLoop;
                    },

                    (HMODE, NO_BOUNDARY) => {
                        self.get_x_token()?;
                        if self.cur_cmd == LETTER
                            || self.cur_cmd == OTHER_CHAR
                            || self.cur_cmd == CHAR_GIVEN
                            || self.cur_cmd == CHAR_NUM
                        {
                            self.cancel_boundary = true;
                        }
                        continue 'reswitch; // Goto reswitch
                    },

                    (HMODE, SPACER) => {
                        if self.space_factor() == 1000 {
                            goto = Goto::AppendNormalSpace;
                        }
                        else {
                            self.app_space()?;
                        }
                    },

                    (HMODE, EX_SPACE)
                    | (MMODE,EX_SPACE) => goto = Goto::AppendNormalSpace,

                    // Section 1045
                    (_, RELAX)
                    | (VMODE, SPACER)
                    | (MMODE, SPACER)
                    | (MMODE, NO_BOUNDARY) => (), // Do nothing

                    (_, IGNORE_SPACES) => {
                        sec406_get_next_nonblank_noncall_token!(self);
                        continue 'reswitch; // Goto reswitch
                    },

                    (VMODE, STOP) => {
                        if self.its_all_over()? {
                            return Ok(()); // This is the only way out
                        }
                    },

                    // Section 1048 (forbidden cases)
                    (VMODE, VMOVE)
                    | (HMODE, HMOVE)
                    | (MMODE, HMOVE)
                    | (_, LAST_ITEM)
                    | (VMODE, VADJUST) // Section 1098
                    | (VMODE, ITAL_CORR) // Section 1111
                    | (VMODE, EQ_NO) // Section 1144
                    | (HMODE, EQ_NO) // End section 1048
                    | (_, MAC_PARAM) => return Err(TeXError::ReportIllegalCase),

                    // Section 1046 (math only cases in non-math mode, or vice-versa)
                    (VMODE, SUP_MARK)
                    | (HMODE, SUP_MARK)
                    | (VMODE, SUB_MARK)
                    | (HMODE, SUB_MARK)
                    | (VMODE, MATH_CHAR_NUM)
                    | (HMODE, MATH_CHAR_NUM)
                    | (VMODE, MATH_GIVEN)
                    | (HMODE, MATH_GIVEN)
                    | (VMODE, MATH_COMP)
                    | (HMODE, MATH_COMP)
                    | (VMODE, DELIM_NUM)
                    | (HMODE, DELIM_NUM)
                    | (VMODE, LEFT_RIGHT)
                    | (HMODE, LEFT_RIGHT)
                    | (VMODE, ABOVE)
                    | (HMODE, ABOVE)
                    | (VMODE, RADICAL)
                    | (HMODE, RADICAL)
                    | (VMODE, MATH_STYLE)
                    | (HMODE, MATH_STYLE)
                    | (VMODE, MATH_CHOICE)
                    | (HMODE, MATH_CHOICE)
                    | (VMODE, VCENTER)
                    | (HMODE, VCENTER)
                    | (VMODE, NON_SCRIPT)
                    | (HMODE, NON_SCRIPT)
                    | (VMODE, MKERN)
                    | (HMODE, MKERN)
                    | (VMODE, LIMIT_SWITCH)
                    | (HMODE, LIMIT_SWITCH)
                    | (VMODE, MSKIP)
                    | (HMODE, MSKIP)
                    | (VMODE, MATH_ACCENT)
                    | (HMODE, MATH_ACCENT)
                    | (MMODE, ENDV)
                    | (MMODE, PAR_END)
                    | (MMODE, STOP)
                    | (MMODE, VSKIP)
                    | (MMODE, UN_VBOX)
                    | (MMODE, VALIGN)
                    | (MMODE, HRULE) => return Err(TeXError::MissingDollar),
                    // End section 1046

                    // Section 1056
                    (VMODE, HRULE)
                    | (HMODE, VRULE)
                    | (MMODE, VRULE) => {
                        tail_append!(self, self.scan_rule_spec()?);
                        match self.mode().abs() {
                            VMODE => *self.prev_depth_mut() = IGNORE_DEPTH,
                            HMODE => *self.space_factor_mut() = 1000,
                            _ => (),
                        }
                    },

                    // Section 1057
                    (VMODE, VSKIP)
                    | (HMODE, HSKIP)
                    | (MMODE, HSKIP)
                    | (MMODE, MSKIP) => self.append_glue()?,

                    (_, KERN)
                    | (MMODE, MKERN) => self.append_kern()?,
                    // End section 1057

                    // Section 1063
                    (VMODE, LEFT_BRACE)
                    | (HMODE, LEFT_BRACE) => self.new_save_level(SIMPLE_GROUP)?,

                    (_, BEGIN_GROUP) => self.new_save_level(SEMI_SIMPLE_GROUP)?,

                    (_, END_GROUP) => {
                        if self.cur_group == SEMI_SIMPLE_GROUP {
                            self.unsave()?;
                        }
                        else {
                            self.off_save()?;
                        }
                    },
                    // End section 1063

                    // Section 1067
                    (_, RIGHT_BRACE) => self.handle_right_brace()?,
                    // End section 1067

                    // Secton 1073
                    (VMODE, HMOVE)
                    | (HMODE, VMOVE)
                    | (MMODE, VMOVE) => {
                        let t = self.cur_chr;
                        self.scan_dimen(false, false, false)?;
                        if t == 0 {
                            self.scan_box(self.cur_val)?;
                        }
                        else {
                            self.scan_box(-self.cur_val)?;
                        }
                    },

                    (_, LEADER_SHIP) => self.scan_box(LEADER_FLAG - A_LEADERS as Integer + self.cur_chr)?,

                    (_, MAKE_BOX) => self.begin_box(0)?,
                    // End section 1073

                    // Section 1090
                    (VMODE, START_PAR) => self.new_graf(self.cur_chr > 0)?,

                    (VMODE, LETTER)
                    | (VMODE, OTHER_CHAR)
                    | (VMODE, CHAR_NUM)
                    | (VMODE, CHAR_GIVEN)
                    | (VMODE, MATH_SHIFT)
                    | (VMODE, UN_HBOX)
                    | (VMODE, VRULE)
                    | (VMODE, ACCENT)
                    | (VMODE, DISCRETIONARY)
                    | (VMODE, HSKIP)
                    | (VMODE, VALIGN)
                    | (VMODE, EX_SPACE)
                    | (VMODE, NO_BOUNDARY) => {
                        self.back_input()?;
                        self.new_graf(true)?;
                    },
                    // End section 1090

                    // Section 1092
                    (HMODE, START_PAR)
                    | (MMODE, START_PAR) => self.indent_in_hmode()?,
                    // End section 1092

                    // Section 1094
                    (VMODE, PAR_END) => {
                        self.normal_paragraph()?;
                        if self.mode() > 0 {
                            self.build_page()?;
                        }
                    },

                    (HMODE, PAR_END) => {
                        if self.align_state < 0 {
                            self.off_save()?;
                        }
                        self.end_graf()?;
                        if self.mode() == VMODE {
                            self.build_page()?;
                        }
                    },

                    (HMODE, STOP)
                    | (HMODE, VSKIP)
                    | (HMODE, HRULE)
                    | (HMODE, UN_VBOX)
                    | (HMODE, HALIGN) => self.head_for_vmode()?,
                    // End section 1094

                    // Section 1097
                    (_, INSERT)
                    | (HMODE, VADJUST)
                    | (MMODE, VADJUST) => self.begin_insert_or_adjust()?,

                    (_ , MARK) => self.make_mark()?,
                    // End section 1097

                    // Section 1102
                    (_, BREAK_PENALTY) => self.append_penalty()?,
                    // End section 1102

                    // Section 1104
                    (_, REMOVE_ITEM) => self.delete_last()?,
                    // End section 1104

                    // Section 1109
                    (VMODE, UN_VBOX)
                    | (HMODE, UN_HBOX)
                    | (MMODE, UN_HBOX) => self.unpackage()?,
                    // End section 1109

                    // Section 1112
                    (HMODE, ITAL_CORR) => self.append_italic_correction()?,

                    (MMODE, ITAL_CORR) => tail_append!(self, self.new_kern(0)?),
                    // End section 1112

                    // Section 1116
                    (HMODE, DISCRETIONARY)
                    | (MMODE, DISCRETIONARY) => self.append_discretionary()?,
                    // End section 1116

                    // Section 1122
                    (HMODE, ACCENT) => self.make_accent()?,
                    // End section 1122

                    // Section 1126
                    (_, CAR_RET)
                    | (_, TAB_MARK) => self.align_error()?,

                    (_, NO_ALIGN) => return Err(TeXError::MisplacedNoalign),

                    (_, OMIT) => return Err(TeXError::MisplacedOmit),
                    // End section 1126

                    // Section 1130
                    (VMODE, HALIGN)
                    | (HMODE, VALIGN) => self.init_align()?,

                    (MMODE, HALIGN) => {
                        self.privileged()?;
                        if self.cur_group == MATH_SHIFT_GROUP {
                            self.init_align()?;
                        }
                        else {
                            self.off_save()?;
                        }
                    },

                    (VMODE, ENDV)
                    | (HMODE, ENDV) => self.do_endv()?,
                    // End section 1130

                    // Section 1134
                    (_, END_CS_NAME) => return Err(TeXError::ExtraEndcsname),
                    // End section 1134

                    // Section 1137
                    (HMODE, MATH_SHIFT) => self.init_math()?,
                    // End section 1137

                    // Section 1140
                    (MMODE, EQ_NO) => {
                        self.privileged()?;
                        if self.cur_group == MATH_SHIFT_GROUP {
                            self.start_eq_no()?;
                        }
                        else {
                            self.off_save()?;
                        }
                    },
                    // End section 1140

                    // Section 1150
                    (MMODE, LEFT_BRACE) => {
                        tail_append!(self, self.new_noad()?);
                        self.back_input()?;
                        self.scan_math(nucleus!(self.tail()))?;
                    },
                    // End section 1150

                    // Section 1154
                    (MMODE, LETTER)
                    | (MMODE, OTHER_CHAR)
                    | (MMODE, CHAR_GIVEN) => self.set_math_char(ho!(math_code(self.cur_chr)))?,

                    (MMODE, CHAR_NUM) => {
                        self.scan_char_num()?;
                        self.cur_chr = self.cur_val;
                        self.set_math_char(ho!(math_code(self.cur_chr)))?;
                    },

                    (MMODE, MATH_CHAR_NUM) => {
                        self.scan_fifteen_bit_int()?;
                        self.set_math_char(self.cur_val)?;
                    },

                    (MMODE, MATH_GIVEN) => self.set_math_char(self.cur_chr)?,

                    (MMODE, DELIM_NUM) => {
                        self.scan_twenty_seven_bit_int()?;
                        self.set_math_char(self.cur_val / 4096)?;
                    }
                    // End section 1154

                    // Section 1158
                    (MMODE, MATH_COMP) => {
                        tail_append!(self, self.new_noad()?);
                        *type_mut(self.tail()) = self.cur_chr as QuarterWord;
                        self.scan_math(nucleus!(self.tail()))?;
                    },

                    (MMODE, LIMIT_SWITCH) => self.math_limit_switch()?,
                    // End section 1158

                    // Section 1162
                    (MMODE, RADICAL) => self.math_radical()?,
                    // End section 1162

                    // Section 1164
                    (MMODE, ACCENT)
                    | (MMODE, MATH_ACCENT) => self.math_ac()?,
                    // End section 1164

                    // Section 1167
                    (MMODE, VCENTER) => {
                        self.scan_spec(VCENTER_GROUP, false)?;
                        self.normal_paragraph()?;
                        self.push_nest()?;
                        *self.mode_mut() = -VMODE;
                        *self.prev_depth_mut() = IGNORE_DEPTH;
                        if every_vbox() != NULL {
                            self.begin_token_list(every_vbox(), EVERY_VBOX_TEXT)?;
                        }
                    },
                    // End section 1167

                    // Section 1171
                    (MMODE, MATH_STYLE) => tail_append!(self, self.new_style(self.cur_chr as QuarterWord)?),

                    (MMODE, NON_SCRIPT) => {
                        tail_append!(self, self.new_glue(ZERO_GLUE)?);
                        *subtype_mut(self.tail()) = COND_MATH_GLUE;
                    },

                    (MMODE, MATH_CHOICE) => self.append_choices()?,
                    // End section 1171

                    // Section 1175
                    (MMODE, SUB_MARK)
                    | (MMODE, SUP_MARK) => self.sub_sup()?,
                    // End section 1175

                    // Section 1180
                    (MMODE, ABOVE) => self.math_fraction()?,
                    // End section 1180

                    // Section 1190
                    (MMODE, LEFT_RIGHT) => self.math_left_right()?,
                    // End section 1190

                    // Section 1193
                    (MMODE, MATH_SHIFT) => {
                        if self.cur_group == MATH_SHIFT_GROUP {
                            self.after_math()?;
                        }
                        else {
                            self.off_save()?;
                        }
                    },
                    // End section 1193
                    // End section 1056

                    // Section 1210
                    (_, TOKS_REGISTER)
                    | (_, ASSIGN_TOKS)
                    | (_, ASSIGN_INT)
                    | (_, ASSIGN_DIMEN)
                    | (_, ASSIGN_GLUE)
                    | (_, ASSIGN_MU_GLUE)
                    | (_, ASSIGN_FONT_DIMEN)
                    | (_, ASSIGN_FONT_INT)
                    | (_, SET_AUX)
                    | (_, SET_PREV_GRAF)
                    | (_, SET_PAGE_DIMEN)
                    | (_, SET_PAGE_INT)
                    | (_, SET_BOX_DIMEN)
                    | (_, SET_SHAPE)
                    | (_, DEF_CODE)
                    | (_, DEF_FAMILY)
                    | (_, SET_FONT)
                    | (_, DEF_FONT)
                    | (_, REGISTER)
                    | (_, ADVANCE)
                    | (_, MULTIPLY)
                    | (_, DIVIDE)
                    | (_, PREFIX)
                    | (_, LET)
                    | (_, SHORTHAND_DEF)
                    | (_, READ_TO_CS)
                    | (_, DEF)
                    | (_, SET_BOX)
                    | (_, HYPH_DATA)
                    | (_, SET_INTERACTION) => self.prefixed_command()?,

                    // Section 1268
                    (_, AFTER_ASSIGNMENT) => {
                        self.get_token()?;
                        self.after_token = self.cur_tok;
                    },
                    // End section 1268

                    // Section 1271
                    (_, AFTER_GROUP) => {
                        self.get_token()?;
                        self.save_for_after(self.cur_tok)?;
                    },
                    // End section 1271

                    // Section 1274
                    (_, IN_STREAM) => self.open_or_close_in()?,
                    // End section 1274

                    // Section 1276
                    (_, MESSAGE) => self.issue_message()?,
                    // End section 1276

                    // Section 1285
                    (_, CASE_SHIFT) => self.shift_case()?,
                    // End section 1285

                    // Section 1290
                    (_, XRAY) => self.show_whatever()?,
                    // End section 1290
                    // End section 1210

                    // Section 1347
                    (_, EXTENSION) => self.do_extension()?,
                    // End section 1347
                    // End section 1045

                    _ => (),
                }
                if goto == Goto::BigSwitch {
                    continue 'big_switch
                }

                // main_loop:
                if goto == Goto::MainLoop {
                    // Section 1034
                    self.adjust_space_factor();
                    self.main_f = cur_font() as QuarterWord;
                    self.bchar = self.font_bchar[self.main_f as usize] as HalfWord;
                    self.false_bchar = self.font_false_bchar[self.main_f as usize] as HalfWord;
                    if self.mode() > 0 && language() != self.clang() {
                        self.fix_language()?;
                    }
                    fast_get_avail!(self, self.lig_stack);
                    *font_mut(self.lig_stack) = self.main_f;
                    self.cur_l = self.cur_chr;
                    *character_mut(self.lig_stack) = self.cur_l as QuarterWord;
                    self.cur_q = self.tail();
                    self.main_k = if self.cancel_boundary {
                        self.cancel_boundary = false;
                        NON_ADDRESS
                    }
                    else {
                        self.bchar_label[self.main_f as usize] as Integer
                    };

                    let mut goto = if self.main_k == NON_ADDRESS {
                        Goto::MainLoopMove(2)
                    }
                    else {
                        self.cur_r = self.cur_l;
                        self.cur_l = NON_CHAR;
                        Goto::MainLigLoop(1)
                    };

                    loop {
                        match goto {
                            Goto::MainLoopWrapup => {
                                // Section 1035 (next line)
                                self.wrapup(self.rt_hit)?;
                                goto = Goto::MainLoopMove(0);
                            },
                            Goto::MainLoopMove(_) => goto = self.sec1036_if_the_cursor_is_immediately(goto),
                            Goto::MainLoopLookAhead => goto = self.sec1038_look_ahead_for_another_character()?,
                            Goto::MainLigLoop(_) => goto = self.sec1039_if_there_is_a_ligature_kern(goto)?,
                            Goto::MainLoopMoveLig => goto = self.sec1037_move_the_cursor_past(),
                            Goto::ReSwitch => continue 'reswitch,
                            Goto::BigSwitch => continue 'big_switch,
                            _ => (), // main_loop and append_normal_space do not happen here
                        }
                    }
                    // End section 1034
                }

                // append_normal_space:
                // Section 1041
                self.temp_ptr = if space_skip() == ZERO_GLUE {
                    // Section 1042
                    self.main_p = self.font_glue[cur_font() as usize];
                    if self.main_p == NULL {
                        self.main_p = self.new_spec(ZERO_GLUE)?;
                        self.main_k = self.param_base[cur_font() as usize] + SPACE_CODE;
                        *width_mut(self.main_p) = self.font_info[self.main_k as usize].sc();
                        *stretch_mut(self.main_p) = self.font_info[(self.main_k + 1) as usize].sc();
                        *shrink_mut(self.main_p) = self.font_info[(self.main_k + 2) as usize].sc();
                        self.font_glue[cur_font() as usize] = self.main_p;
                    }
                    // End section 1042
                    self.new_glue(self.main_p)?
                }
                else {
                    self.new_param_glue(SPACE_SKIP_CODE as QuarterWord)?
                };

                *link_mut(self.tail()) = self.temp_ptr;
                *self.tail_mut() = self.temp_ptr;
                continue 'big_switch;
                // End section 1041
            }
        }
    }

    // Section 1034
    fn adjust_space_factor(&mut self) {
        self.main_s = sf_code(self.cur_chr);
         if self.main_s == 1000 {
            *self.space_factor_mut() = 1000;
        }
        else if self.main_s < 1000 {
            if self.main_s > 0 {
                *self.space_factor_mut() = self.main_s;
            }
        }
        else if self.space_factor() < 1000 {
            *self.space_factor_mut() = 1000;
        }
        else {
            *self.space_factor_mut() = self.main_s;
        }
    }

    // Section 1035
    fn pack_lig(&mut self, b: bool) -> TeXResult<()> {
        self.main_p = self.new_ligature(self.main_f, self.cur_l as QuarterWord, link(self.cur_q))?;
        if self.lft_hit {
            *subtype_mut(self.main_p) = 2;
            self.lft_hit = false;
        }
        if b && self.lig_stack == NULL {
            *subtype_mut(self.main_p) += 1;
            self.rt_hit = false;
        }
        *link_mut(self.cur_q) = self.main_p;
        *self.tail_mut() = self.main_p;
        self.ligature_present = false;
        Ok(())
    }

    fn wrapup(&mut self, b: bool) -> TeXResult<()> {
        if self.cur_l < NON_CHAR {
            if link(self.cur_q) > NULL
                && (character(self.tail()) as Integer) == self.hyphen_char[self.main_f as usize]
            {
                self.ins_disc = true;
            }
            if self.ligature_present {
                self.pack_lig(b)?;
            }
            if self.ins_disc {
                self.ins_disc = false;
                if self.mode() > 0 {
                    tail_append!(self, self.new_disc()?);
                }
            }
        }
        Ok(())
    }

    // Section 1036
    fn sec1036_if_the_cursor_is_immediately(&mut self, goto: Goto) -> Goto {
        if goto == Goto::MainLoopMove(0) {
            if self.lig_stack == NULL {
                return Goto::ReSwitch;
            }
            self.cur_q = self.tail();
            self.cur_l = character(self.lig_stack) as HalfWord;
        }

        // main_loop_move + 1:
        if goto != Goto::MainLoopMove(2) && !self.is_char_node(self.lig_stack) {
            return Goto::MainLoopMoveLig;
        }

        // main_loop_move + 2:
        if self.cur_chr < self.font_bc[self.main_f as usize] as HalfWord
            || self.cur_chr > self.font_ec[self.main_f as usize] as HalfWord
        {
            self.char_warning(self.main_f, self.cur_chr as u8);
            free_avail!(self, self.lig_stack);
            return Goto::BigSwitch;
        }
        self.main_i = self.char_info(self.main_f, self.cur_l as QuarterWord);
        if !self.main_i.char_exists() {
            self.char_warning(self.main_f, self.cur_chr as u8);
            free_avail!(self, self.lig_stack);
            return Goto::BigSwitch;
        }
        *link_mut(self.tail()) = self.lig_stack;
        *self.tail_mut() = self.lig_stack;
        Goto::MainLoopLookAhead // main_loop_lookahead is next
    }

    // Section 1037
    fn sec1037_move_the_cursor_past(&mut self) -> Goto {
        self.main_p = lig_ptr(self.lig_stack);
        if self.main_p > NULL {
            tail_append!(self, self.main_p);
        }
        self.temp_ptr = self.lig_stack;
        self.lig_stack = link(self.temp_ptr);
        self.free_node(self.temp_ptr, SMALL_NODE_SIZE);
        self.main_i = self.char_info(self.main_f, self.cur_l as QuarterWord);
        self.ligature_present = true;
        if self.lig_stack == NULL {
            if self.main_p > NULL {
                return Goto::MainLoopLookAhead;
            }
            self.cur_r = self.bchar;
        }
        else {
            self.cur_r = character(self.lig_stack) as HalfWord;
        }
        Goto::MainLigLoop(0)
    }

    // Section 1038
    fn sec1038_look_ahead_for_another_character(&mut self) -> TeXResult<Goto> {
        self.get_next()?;
        'block: {
            if self.cur_cmd == LETTER
                || self.cur_cmd == OTHER_CHAR
                || self.cur_cmd == CHAR_GIVEN
            {
                break 'block; // Goto main_loop_lookahead + 1
            }
            
            self.x_token()?;

            if self.cur_cmd == LETTER
                || self.cur_cmd == OTHER_CHAR
                || self.cur_cmd == CHAR_GIVEN
            {
                break 'block; // Goto main_loop_lookahead + 1
            }
            
            if self.cur_cmd == CHAR_NUM {
                self.scan_char_num()?;
                self.cur_chr = self.cur_val;
                break 'block; // Goto main_loop_lookahead + 1
            }

            if self.cur_cmd == NO_BOUNDARY {
                self.bchar = NON_CHAR;
            }
            self.cur_r = self.bchar;
            self.lig_stack = NULL;
            return Ok(Goto::MainLigLoop(0));
        }

        // main_loop_lookahead + 1:
        self.adjust_space_factor();
        fast_get_avail!(self, self.lig_stack);
        *font_mut(self.lig_stack) = self.main_f;
        self.cur_r = self.cur_chr;
        *character_mut(self.lig_stack) = self.cur_r as QuarterWord;
        if self.cur_r == self.false_bchar {
            self.cur_r = NON_CHAR;
        }
        Ok(Goto::MainLigLoop(0))
    }

    // Section 1039
    fn sec1039_if_there_is_a_ligature_kern(&mut self, mut goto: Goto) -> TeXResult<Goto> {
        if goto == Goto::MainLigLoop(0) {
            if self.main_i.char_tag() != LIG_TAG || self.cur_r == NON_CHAR {
                return Ok(Goto::MainLoopWrapup);
            }
            self.main_k = self.lig_kern_start(self.main_f, self.main_i);
            self.main_j = self.font_info[self.main_k as usize];
            if self.main_j.skip_byte() <= STOP_FLAG {
                goto = Goto::MainLigLoop(2);
            }
            else {
                self.main_k = self.lig_kern_restart(self.main_f, self.main_j);
            }
        }

        // main_lig_loop + 1:
        if goto != Goto::MainLigLoop(2) {
            self.main_j = self.font_info[self.main_k as usize];
        }

        // main_lig_loop + 2:
        if self.main_j.next_char() == self.cur_r as QuarterWord && self.main_j.skip_byte() <= STOP_FLAG {
            // Section 1040
            if self.main_j.op_byte() >= KERN_FLAG {
                self.wrapup(self.rt_hit)?;
                tail_append!(self, self.new_kern(self.char_kern(self.main_f, self.main_j))?);
                return Ok(Goto::MainLoopMove(0));
            }
            if self.cur_l == NON_CHAR {
                self.lft_hit = true;
            }
            else if self.lig_stack == NULL {
                self.rt_hit = true;
            }
            self.check_interrupt()?;

            match self.main_j.op_byte() {
                1 | 5 => {
                    self.cur_l = self.main_j.rem_byte() as HalfWord;
                    self.main_i = self.char_info(self.main_f, self.cur_l as QuarterWord);
                    self.ligature_present = true;
                },

                2 | 6 => {
                    self.cur_r = self.main_j.rem_byte() as HalfWord;
                    if self.lig_stack == NULL {
                        self.lig_stack = self.new_lig_item(self.cur_r as QuarterWord)?;
                        self.bchar = NON_CHAR;
                    }
                    else if self.is_char_node(self.lig_stack) {
                        self.main_p = self.lig_stack;
                        self.lig_stack = self.new_lig_item(self.cur_r as QuarterWord)?;
                        *lig_ptr_mut(self.lig_stack) = self.main_p;
                    }
                    else {
                        *character_mut(self.lig_stack) = self.cur_r as QuarterWord;
                    }
                },

                3 => {
                    self.cur_r = self.main_j.rem_byte() as HalfWord;
                    self.main_p = self.lig_stack;
                    self.lig_stack = self.new_lig_item(self.cur_r as QuarterWord)?;
                    *link_mut(self.lig_stack) = self.main_p;
                },

                7 | 11 => {
                    self.wrapup(false)?;
                    self.cur_q = self.tail();
                    self.cur_l = self.main_j.rem_byte() as HalfWord;
                    self.main_i = self.char_info(self.main_f, self.cur_l as QuarterWord);
                    self.ligature_present = true;
                },
                
                _ => {
                    self.cur_l = self.main_j.rem_byte() as HalfWord;
                    self.ligature_present = true;
                    if self.lig_stack == NULL {
                        return Ok(Goto::MainLoopWrapup);
                    }
                    return Ok(Goto::MainLoopMove(1));
                }
            }

            if self.main_j.op_byte() > 4 && self.main_j.op_byte() != 7 {
                return Ok(Goto::MainLoopWrapup);
            }
            if self.cur_l < NON_CHAR {
                return Ok(Goto::MainLigLoop(0));
            }
            self.main_k = self.bchar_label[self.main_f as usize] as Integer;
            return Ok(Goto::MainLigLoop(1));
            // End section 1040
        }

        if self.main_j.skip_byte() == 0 {
            self.main_k += 1;
        }
        else {
            if self.main_j.skip_byte() >= STOP_FLAG {
                return Ok(Goto::MainLoopWrapup);
            }
            self.main_k += (self.main_j.skip_byte() + 1) as Integer;
        }
        Ok(Goto::MainLigLoop(1))
    }

    // Section 1043
    fn app_space(&mut self) -> TeXResult<()> {
        let q = if self.space_factor() >= 2000 && xspace_skip() != ZERO_GLUE {
            self.new_param_glue(XSPACE_SKIP_CODE as QuarterWord)?
        }
        else {
            if space_skip() != ZERO_GLUE {
                self.main_p = space_skip();
            }
            else {
                // Section 1042
                self.main_p = self.font_glue[cur_font() as usize];
                if self.main_p == NULL {
                    self.main_p = self.new_spec(ZERO_GLUE)?;
                    self.main_k = self.param_base[cur_font() as usize] + SPACE_CODE;
                    *width_mut(self.main_p) = self.font_info[self.main_k as usize].sc();
                    *stretch_mut(self.main_p) = self.font_info[(self.main_k + 1) as usize].sc();
                    *shrink_mut(self.main_p) = self.font_info[(self.main_k + 2) as usize].sc();
                    self.font_glue[cur_font() as usize] = self.main_p;
                }
                // End section 1042
            }
            self.main_p = self.new_spec(self.main_p)?;

            // Section 1044
            if self.space_factor() >= 2000 {
                *width_mut(self.main_p) += self.extra_space(cur_font() as QuarterWord);
            }
            *stretch_mut(self.main_p) = xn_over_d(stretch(self.main_p), self.space_factor(), 1000)?.0;
            *shrink_mut(self.main_p) = xn_over_d(shrink(self.main_p), 1000, self.space_factor())?.0;
            // End section 1044

            let q = self.new_glue(self.main_p)?;
            *glue_ref_count_mut(self.main_p) = NULL;
            q
        };

        *link_mut(self.tail()) = q;
        *self.tail_mut() = q;
        Ok(())
    }

    // Section 1051
    // Does not output a boolean, use TeXResult instead.
    fn privileged(&self) -> TeXResult<()> {
        if self.mode() > 0 {
            Ok(())
        }
        else {
            Err(TeXError::ReportIllegalCase)
        }
    }

    // Section 1054
    fn its_all_over(&mut self) -> TeXResult<bool> {
        self.privileged()?;
        if PAGE_HEAD == self.page_tail && self.head() == self.tail() && self.dead_cycles == 0 {
            Ok(true)
        }
        else {
            self.back_input()?;
            tail_append!(self, self.new_null_box()?);
            *width_mut(self.tail()) = hsize();
            tail_append!(self, self.new_glue(FILL_GLUE)?);
            tail_append!(self, self.new_penalty(-0x4000_0000)?);
            self.build_page()?;
            Ok(false)
        }
    }
}
