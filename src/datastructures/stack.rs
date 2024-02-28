use crate::constants::*;
use crate::datastructures::{
    EQTB, XEQ_LEVEL, MemoryWord, eq_level, eq_level_mut, eq_type_mut,
    equiv_mut, info, info_mut, link, token_ref_count_mut, tracing_macros
};
use crate::error::{TeXError, TeXResult};
use crate::io::AlphaFileIn;
use crate::{
    Global, HalfWord, Integer, QuarterWord, add_token_ref, eqtb, eqtb_mut
};

use std::ops::{Index, IndexMut};

#[cfg(feature = "stat")]
use crate::datastructures::tracing_restores;

// Part 19: Saving and restoring equivalents

pub(crate) struct InputFile(pub(crate) [AlphaFileIn; MAX_IN_OPEN as usize]);

impl Index<usize> for InputFile {
    type Output = AlphaFileIn;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - 1]
    }
}

impl IndexMut<usize> for InputFile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - 1]
    }
}

pub(crate) struct LineStack(pub(crate) [Integer; MAX_IN_OPEN as usize]);

impl Index<usize> for LineStack {
    type Output = Integer;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - 1]
    }
}

impl IndexMut<usize> for LineStack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - 1]
    }
}

impl Global {
    // Section 268
    fn save_type(&self, p: usize) -> QuarterWord {
        self.save_stack[p].hh_b0()
    }

    fn save_type_mut(&mut self, p: usize) -> &mut QuarterWord {
        self.save_stack[p].hh_b0_mut()
    }

    fn save_level(&self, p: usize) -> QuarterWord {
        self.save_stack[p].hh_b1()
    }

    fn save_level_mut(&mut self, p: usize) -> &mut QuarterWord {
        self.save_stack[p].hh_b1_mut()
    }

    fn save_index(&self, p: usize) -> HalfWord {
        self.save_stack[p].hh_rh()
    }

    fn save_index_mut(&mut self, p: usize) -> &mut HalfWord {
        self.save_stack[p].hh_rh_mut()
    }

    // Section 273
    fn check_full_save_stack(&mut self) -> TeXResult<()> {
        if self.save_ptr > self.max_save_stack as usize {
            self.max_save_stack = self.save_ptr as Integer;
            if self.max_save_stack > SAVE_SIZE - 6 {
                return Err(TeXError::Overflow("save size", SAVE_SIZE));
            }
        }
        Ok(())
    }

    // Section 274
    pub(crate) fn saved(&self, p: Integer) -> Integer {
        self.save_stack[(self.save_ptr as Integer + p) as usize].int()
    }

    pub(crate) fn saved_mut(&mut self, p: Integer) -> &mut Integer {
        self.save_stack[(self.save_ptr as Integer + p) as usize].int_mut()
    }

    pub(crate) fn new_save_level(&mut self, c: Integer) -> TeXResult<()> {
        self.check_full_save_stack()?;
        *self.save_type_mut(self.save_ptr) = LEVEL_BOUNDARY;
        *self.save_level_mut(self.save_ptr) = self.cur_group as QuarterWord;
        *self.save_index_mut(self.save_ptr) = self.cur_boundary;
        if self.cur_level == MAX_QUARTERWORD {
            return Err(TeXError::Overflow("grouping levels", (MAX_QUARTERWORD - MIN_QUARTERWORD) as Integer));
        }
        self.cur_boundary = self.save_ptr as Integer;
        self.cur_level += 1;
        self.save_ptr += 1;
        self.cur_group = c;
        Ok(())
    }

    // Section 275
    fn eq_destroy(&mut self, w: MemoryWord) -> TeXResult<()>{
        match w.eq_type_field() {
            CALL
            | LONG_CALL
            | OUTER_CALL
            | LONG_OUTER_CALL => self.delete_token_ref(w.equiv_field()),

            GLUE_REF => self.delete_glue_ref(w.equiv_field()),

            SHAPE_REF => {
                let q = w.equiv_field();
                if q != NULL {
                    self.free_node(q, 2*info(q) + 1);
                }
            },

            BOX_REF => self.flush_node_list(w.equiv_field())?,

            _ => ()
        }
        Ok(())
    }

    // Section 276
    fn eq_save(&mut self, p: HalfWord, l: QuarterWord) -> TeXResult<()> {
        self.check_full_save_stack()?;
        if l == LEVEL_ZERO {
            *self.save_type_mut(self.save_ptr) = RESTORE_ZERO;
        }
        else {
            self.save_stack[self.save_ptr] = eqtb![p as usize];
            self.save_ptr += 1;
            *self.save_type_mut(self.save_ptr) = RESTOVE_OLD_VALUE;
        }
        *self.save_level_mut(self.save_ptr) = l;
        *self.save_index_mut(self.save_ptr) = p;
        self.save_ptr += 1;
        Ok(())
    }

    // Section 277
    pub(crate) fn eq_define(&mut self, p: HalfWord, t: QuarterWord, e: HalfWord) -> TeXResult<()> {
        if eq_level(p) == self.cur_level {
            self.eq_destroy(eqtb![p as usize])?;
        }
        else if self.cur_level > LEVEL_ONE {
            self.eq_save(p, eq_level(p))?;
        }
        *eq_level_mut(p) = self.cur_level;
        *eq_type_mut(p) = t;
        *equiv_mut(p) = e;
        Ok(())
    }

    // Section 278
    pub(crate) fn eq_word_define(&mut self, p: HalfWord, w: Integer) -> TeXResult<()> {
        if unsafe { XEQ_LEVEL[p as usize] } != self.cur_level {
            self.eq_save(p, unsafe { XEQ_LEVEL[p as usize] })?;
            unsafe { XEQ_LEVEL[p as usize] = self.cur_level; }
        }
        *eqtb_mut![p as usize].int_mut() = w;
        Ok(())
    }

    // Section 279
    pub(crate) fn geq_define(&mut self, p: HalfWord, t: QuarterWord, e: HalfWord) -> TeXResult<()> {
        self.eq_destroy(eqtb![p as usize])?;
        *eq_level_mut(p) = LEVEL_ONE;
        *eq_type_mut(p) = t;
        *equiv_mut(p) = e;
        Ok(())
    }
}

pub(crate) fn geq_word_define(p: HalfWord, w: Integer) {
    *eqtb_mut![p as usize].int_mut() = w;
    unsafe { XEQ_LEVEL[p as usize] = LEVEL_ONE; }
}

impl Global {
    // Section 280
    pub(crate) fn save_for_after(&mut self, t: HalfWord) -> TeXResult<()> {
        if self.cur_level > LEVEL_ONE {
            self.check_full_save_stack()?;
            *self.save_type_mut(self.save_ptr) = INSERT_TOKEN;
            *self.save_level_mut(self.save_ptr) = LEVEL_ZERO;
            *self.save_index_mut(self.save_ptr) = t;
            self.save_ptr += 1;
        }
        Ok(())
    }

    // Section 281
    pub(crate) fn unsave(&mut self) -> TeXResult<()> {
        let mut l: QuarterWord = 0;
        if self.cur_level > LEVEL_ONE {
            self.cur_level -= 1;
            // Section 282
            loop {
                self.save_ptr -= 1;
                if self.save_type(self.save_ptr) == LEVEL_BOUNDARY {
                    break; // Goto done
                }
                let p = self.save_index(self.save_ptr);
                if self.save_type(self.save_ptr) == INSERT_TOKEN {
                    // Section 326
                    let t = self.cur_tok;
                    self.cur_tok = p;
                    self.back_input()?;
                    self.cur_tok = t;
                    // End section 326
                }
                else {
                    if self.save_type(self.save_ptr) == RESTOVE_OLD_VALUE {
                        l = self.save_level(self.save_ptr);
                        self.save_ptr -= 1;
                    }
                    else {
                        self.save_stack[self.save_ptr] = eqtb![UNDEFINED_CONTROL_SEQUENCE as usize];
                    }
                    // Section 283
                    if p < INT_BASE {
                        if eq_level(p) == LEVEL_ONE {
                            self.eq_destroy(self.save_stack[self.save_ptr])?;
                            #[cfg(feature = "stat")]
                            if tracing_restores() > 0 {
                                self.restore_trace(p, "retaining");
                            }
                        }
                        else {
                            self.eq_destroy(eqtb![p as usize])?;
                            *eqtb_mut![p as usize] = self.save_stack[self.save_ptr];
                            #[cfg(feature = "stat")]
                            if tracing_restores() > 0 {
                                self.restore_trace(p, "restoring");
                            }
                        }
                    }
                    else if unsafe { XEQ_LEVEL[p as usize] } != LEVEL_ONE {
                        *eqtb_mut![p as usize] = self.save_stack[self.save_ptr];
                        unsafe { XEQ_LEVEL[p as usize] = l; }
                        #[cfg(feature = "stat")]
                        if tracing_restores() > 0 {
                            self.restore_trace(p, "restoring");
                        }
                    }
                    else {
                        #[cfg(feature = "stat")]
                        if tracing_restores() > 0 {
                            self.restore_trace(p, "retaining");
                        }
                    }
                    // End section 283
                }
            }
            // done:
            self.cur_group = self.save_level(self.save_ptr) as Integer;
            self.cur_boundary = self.save_index(self.save_ptr);
            // End section 282
            Ok(())
        }
        else {
            Err(TeXError::Confusion("curlevel"))
        }
    }

    // Section 284
    #[cfg(feature = "stat")]
    fn restore_trace(&mut self, p: HalfWord, s: &str) {
        self.begin_diagnostic();
        self.print_char(b'{');
        self.print(s);
        self.print_char(b' ');
        self.show_eqtb(p);
        self.print_char(b'}');
        self.end_diagnostic(false);
    }
}

// Section 300
#[derive(Default, Clone, Copy)]
pub(crate) struct InStateRecord {
    pub(crate) state_field: QuarterWord,
    pub(crate) index_field: QuarterWord,
    start_field: HalfWord,
    pub(crate) loc_field: HalfWord,
    pub(crate) limit_field: HalfWord,
    name_field: HalfWord,
}

impl Global {
    // Section 302
    pub(crate) fn state(&self) -> QuarterWord {
        self.cur_input.state_field
    }

    pub(crate) fn state_mut(&mut self) -> &mut QuarterWord {
        &mut self.cur_input.state_field
    }

    pub(crate) fn index(&self) -> QuarterWord {
        self.cur_input.index_field
    }

    pub(crate) fn index_mut(&mut self) -> &mut QuarterWord {
        &mut self.cur_input.index_field
    }

    pub(crate) fn start(&self) -> HalfWord {
        self.cur_input.start_field
    }

     pub(crate) fn start_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_input.start_field
    }

    pub(crate) fn limit(&self) -> HalfWord {
        self.cur_input.limit_field
    }

    pub fn limit_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_input.limit_field
    }

    pub(crate) fn name(&self) -> HalfWord {
        self.cur_input.name_field
    }

    pub(crate) fn name_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_input.name_field
    }

    // Section 304
    pub(crate) fn terminal_input(&self) -> bool {
        self.name() == 0
    }

    pub(crate) fn cur_file_mut(&mut self) -> &mut AlphaFileIn {
        let index = self.index() as usize;
        &mut self.input_file[index]
    }
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Status {
    Normal,
    Skipping,
    Defining,
    Matching,
    Aligning,
    Absorbing
}

impl Global {
    // Section 306
    pub(crate) fn runaway(&mut self) {
        if self.scanner_status != Status::Skipping  && self.scanner_status != Status::Normal {
            self.print_nl("Runaway ");
            let p = match self.scanner_status {
                Status::Defining => {
                    self.print("definition");
                    self.def_ref
                },

                Status::Matching => {
                    self.print("argument");
                    TEMP_HEAD
                },

                Status::Aligning => {
                    self.print("preamble");
                    HOLD_HEAD
                },

                _ /* Absorbing */ => {
                    self.print("text");
                    self.def_ref
                },
            };
            self.print_char(b'?');
            self.print_ln();
            self.show_token_list(link(p), NULL, ERROR_LINE - 10);
        }
    }
}

// Section 323
#[macro_export]
macro_rules! back_list {
    ($s:ident,$p:expr) => {
        $s.begin_token_list($p, BACKED_UP)?;
    };
}

#[macro_export]
macro_rules! ins_list {
    ($s:ident,$p:expr) => {
        $s.begin_token_list($p, INSERTED)?;
    };
}

impl Global {
    // Section 307
    pub(crate) fn token_type(&self) -> QuarterWord {
        self.index()
    }

    pub(crate) fn token_type_mut(&mut self) -> &mut QuarterWord {
        self.index_mut()
    }

    pub(crate) fn param_start(&self) -> HalfWord {
        self.limit()
    }

    fn param_start_mut(&mut self) -> &mut HalfWord {
        self.limit_mut()
    }

    // Section 321
    fn push_input(&mut self) -> TeXResult<()> {
        if self.input_ptr > self.max_in_stack {
            self.max_in_stack = self.input_ptr;
            if self.input_ptr == STACK_SIZE as usize {
                return Err(TeXError::Overflow("input stack size", STACK_SIZE));
            }
        }
        self.input_stack[self.input_ptr] = self.cur_input;
        self.input_ptr += 1;
        Ok(())
    }

    // Section 322
    fn pop_input(&mut self) {
        self.input_ptr -= 1;
        self.cur_input = self.input_stack[self.input_ptr];
    }

    // Section 323
    pub(crate) fn begin_token_list(&mut self, p: HalfWord, t: QuarterWord) -> TeXResult<()> {
        self.push_input()?;
        *self.state_mut() = TOKEN_LIST;
        *self.start_mut() = p;
        *self.token_type_mut() = t;
        if t >= MACRO {
            add_token_ref!(p);
            if t == MACRO {
                *self.param_start_mut() = self.param_ptr as HalfWord;
            }
            else {
                *self.loc_mut() = link(p);
                if tracing_macros() > 1 {
                    self.begin_diagnostic();
                    self.print_nl("");
                    match t {
                        MARK_TEXT => self.print_esc("mark"),
                        WRITE_TEXT => self.print_esc("write"),
                        _ => self.print_cmd_chr(ASSIGN_TOKS as QuarterWord, t as HalfWord - OUTPUT_TEXT as HalfWord + OUTPUT_ROUTINE_LOC),
                    }
                    self.print("->");
                    self.token_show(p);
                    self.end_diagnostic(false);
                }
            }
        }
        else {
            *self.loc_mut() = p;
        }
        Ok(())
    }

    // Section 324
    pub(crate) fn end_token_list(&mut self) -> TeXResult<()> {
        if self.token_type() >= BACKED_UP {
            if self.token_type() <= INSERTED {
                self.flush_list(self.start());
            }
            else {
                self.delete_token_ref(self.start());
                if self.token_type() == MACRO {
                    while self.param_ptr > self.param_start() as usize {
                        self.param_ptr -= 1;
                        self.flush_list(self.param_stack[self.param_ptr]);
                    }
                }
            }
        }
        else if self.token_type() == U_TEMPLATE {
            if self.align_state > 500_000 {
                self.align_state = 0;
            }
            else {
                return Err(TeXError::Fatal("(interwoven alignment preambles are not allowed)"));
            }
        }
        self.pop_input();
        Ok(())
    }

    // Section 325
    pub(crate) fn back_input(&mut self) -> TeXResult<()> {
        while self.state() == TOKEN_LIST
            && self.loc() == NULL
            && self.token_type() != V_TEMPLATE
        {
            self.end_token_list()?;
        }

        let p = self.get_avail()?;
        *info_mut(p) = self.cur_tok;
        if self.cur_tok < RIGHT_BRACE_LIMIT {
            if self.cur_tok < LEFT_BRACE_LIMIT {
                self.align_state -= 1;
            }
            else {
                self.align_state += 1;
            }
        }
        self.push_input()?;
        *self.state_mut() = TOKEN_LIST;
        *self.start_mut() = p;
        *self.token_type_mut() = BACKED_UP;
        *self.loc_mut() = p;
        Ok(())
    }

    // Section 327
    // No `back_error` or `ins_error`.
    // When an error occurs, it's endgame.

    // Section 328
    pub(crate) fn begin_file_reading(&mut self) -> TeXResult<()> {
        if self.in_open == MAX_IN_OPEN as usize {
            return Err(TeXError::Overflow("text input levels", MAX_IN_OPEN));
        }
        if self.first == BUF_SIZE {
            return Err(TeXError::Overflow("buffer size", BUF_SIZE));
        }
        self.in_open += 1;
        self.push_input()?;
        *self.index_mut() = self.in_open as QuarterWord;
        self.line_stack[self.in_open] = self.line;
        *self.start_mut() = self.first;
        *self.state_mut() = MID_LINE;
        *self.name_mut() = 0;
        Ok(())
    }

    // Section 329
    pub(crate) fn end_file_reading(&mut self) {
        self.first = self.start();
        self.line = self.line_stack[self.index() as usize];
        if self.name() > 17 {
            self.cur_file_mut().close();
        }
        self.pop_input();
        self.in_open -= 1;
    }
}
