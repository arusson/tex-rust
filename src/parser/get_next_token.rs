use crate::alignment::{
    extra_info, extra_info_mut, v_part
};
use crate::constants::*;
use crate::datastructures::{
    Status, cat_code, end_line_char, eq_type, equiv, info, link, pausing
};
use crate::error::{TeXError, TeXResult};
use crate::io::AlphaFileInSelector;
use crate::{
    Global, HalfWord, QuarterWord, StrNum, update_terminal
};

use std::io::Write;

// Part 24: Getting the next token

// Section 352
macro_rules! hex_to_cur_chr {
    ($c:expr, $cc:expr, $cur_chr:expr) => {
        if $c <= b'9' {
            $cur_chr = ($c - b'0') as HalfWord;
        }
        else {
            $cur_chr = ($c - b'a' + 10) as HalfWord;
        }
        if $cc <= b'9' {
            $cur_chr = 16*$cur_chr + ($cc - b'0') as HalfWord;
        }
        else {
            $cur_chr = 16*$cur_chr + ($cc - b'a' + 10) as HalfWord;
        }
    };
}

// Section 360
#[macro_export]
macro_rules! end_line_char_inactive {
    () => {
        end_line_char() < 0 || end_line_char() > 255    
    };
}

// For use in section 343.
enum Goto {
    Restart,
    Switch,
    Reswitch,
    Nothing,
    Exit
}

impl Global {
    // Section 336
    fn check_outer_validity(&mut self) -> TeXResult<()> {
        if self.scanner_status != Status::Normal {
            if self.scanner_status != Status::Skipping {
                Err(TeXError::FileEndedOrForbiddenCSFound)
            }
            else {
                Err(TeXError::IncompleteIf)
            }
        }
        else {
            Ok(())
        }
    }

    // Section 341
    pub(crate) fn get_next(&mut self) -> TeXResult<()> {
        'restart: loop {
            self.cur_cs = 0;
            if self.state() != TOKEN_LIST {
                // Section 343
                'switch: loop {
                    if self.loc() <= self.limit() {
                        self.cur_chr = self.buffer[self.loc() as usize] as HalfWord;
                        *self.loc_mut() += 1;
                        'reswitch: loop {
                            self.cur_cmd = cat_code(self.cur_chr) as QuarterWord;
                            match self.sec344_change_state_if_necessary()? {
                                Goto::Switch => continue 'switch,
                                Goto::Reswitch => continue 'reswitch,
                                _ => break 'switch,
                            }
                        }
                    }
                    else {
                        *self.state_mut() = NEW_LINE;
                        match self.sec360_move_to_next_line_of_file()? {
                            Goto::Restart => continue 'restart,
                            Goto::Exit => return Ok(()),
                            _ => {
                                self.check_interrupt()?;
                                continue 'switch; // Goto switch
                            }
                        }
                    }
                }
                // End section 343
            }
            else if let Goto::Restart = self.sec357_input_from_token_list()? {
                continue 'restart;
            }
                
            // Section 342
            if self.cur_cmd <= CAR_RET
                && self.cur_cmd >= TAB_MARK
                && self.align_state == 0
            {
                // Section 789
                if self.scanner_status == Status::Aligning || self.cur_align == NULL {
                    return Err(TeXError::Fatal("(interwoven alignment preambles are not allowed)"));
                }
                self.cur_cmd = extra_info(self.cur_align) as QuarterWord;
                *extra_info_mut(self.cur_align) = self.cur_chr;
                if self.cur_cmd == OMIT {
                    self.begin_token_list(OMIT_TEMPLATE, V_TEMPLATE)?;
                }
                else {
                    self.begin_token_list(v_part(self.cur_align), V_TEMPLATE)?;
                }
                self.align_state = 1_000_000;
                // Goto restart
                // End section 789
            }
            else {
                break 'restart;
            }
            // End section 342
        }
        Ok(())
    }

    // Section 344
    fn sec344_change_state_if_necessary(&mut self) -> TeXResult<Goto> {
        match (self.state(), self.cur_cmd) {
            // Section 345 (first match)
            (_, IGNORE)
            | (SKIP_BLANKS, SPACER)
            | (NEW_LINE, SPACER) => return Ok(Goto::Switch),

            (_, ESCAPE) => self.sec354_scan_a_control_sequence()?,

            (_, ACTIVE_CHAR) => {
                // Section 353
                self.cur_cs = self.cur_chr + ACTIVE_BASE;
                self.cur_cmd = eq_type(self.cur_cs);
                self.cur_chr = equiv(self.cur_cs);
                *self.state_mut() = MID_LINE;
                if self.cur_cmd >= OUTER_CALL {
                    self.check_outer_validity()?;
                }
                // End section 353
            },

            (_, SUP_MARK) => return Ok(self.sec352_if_this_sup_mark_starts()),

            (_, INVALID_CHAR) => return Err(TeXError::InvalidCharacter), // Section 346

            // Section 347
            (MID_LINE, SPACER) => {
                // Section 349
                *self.state_mut() = SKIP_BLANKS;
                self.cur_chr = b' ' as HalfWord;
                // End section 349
            },

            (MID_LINE, CAR_RET) => {
                // Section 348
                *self.loc_mut() = self.limit() + 1;
                self.cur_cmd = SPACER;
                self.cur_chr = b' ' as HalfWord;
                // End section 348
            },

            (SKIP_BLANKS, CAR_RET)
            | (_, COMMENT) => {
                // Section 350
                *self.loc_mut() = self.limit() + 1;
                return Ok(Goto::Switch);
                // End section 350
            },

            (NEW_LINE, CAR_RET) => {
                // Section 351
                *self.loc_mut() = self.limit() + 1;
                self.cur_cs = self.par_loc;
                self.cur_cmd = eq_type(self.cur_cs);
                self.cur_chr = equiv(self.cur_cs);
                if self.cur_cmd >= OUTER_CALL {
                    self.check_outer_validity()?;
                }
                // End section 351
            },

            (MID_LINE, LEFT_BRACE) => self.align_state += 1,

            (SKIP_BLANKS, LEFT_BRACE)
            | (NEW_LINE, LEFT_BRACE) => {
                *self.state_mut() = MID_LINE;
                self.align_state += 1;
            },

            (MID_LINE, RIGHT_BRACE) => self.align_state -= 1, 

            (SKIP_BLANKS, RIGHT_BRACE)
            | (NEW_LINE, RIGHT_BRACE) => {
                *self.state_mut() = MID_LINE;
                self.align_state -= 1;
            },

            (SKIP_BLANKS, MATH_SHIFT)
            | (SKIP_BLANKS, TAB_MARK)
            | (SKIP_BLANKS, MAC_PARAM)
            | (SKIP_BLANKS, SUB_MARK)
            | (SKIP_BLANKS, LETTER)
            | (SKIP_BLANKS, OTHER_CHAR)
            | (NEW_LINE, MATH_SHIFT)
            | (NEW_LINE, TAB_MARK)
            | (NEW_LINE, MAC_PARAM)
            | (NEW_LINE, SUB_MARK)
            | (NEW_LINE, LETTER)
            | (NEW_LINE, OTHER_CHAR) => *self.state_mut() = MID_LINE,
            // End section 347

            _ => (), // Do nothing
        }
        Ok(Goto::Nothing)
    }

    // Section 352
    fn sec352_if_this_sup_mark_starts(&mut self) -> Goto {
        if self.cur_chr == self.buffer[self.loc() as usize] as HalfWord
            && self.loc() < self.limit()
        {
            let c = self.buffer[(self.loc() + 1) as usize];
            if c < 128 {
                *self.loc_mut() += 2;
                if c.is_ascii_hexdigit() && self.loc() <= self.limit() {
                    let cc = self.buffer[self.loc() as usize];
                    if cc.is_ascii_hexdigit() {
                        *self.loc_mut() += 1;
                        hex_to_cur_chr!(c, cc, self.cur_chr);
                        return Goto::Reswitch;
                    }
                }
                self.cur_chr = if c < 64 {
                    (c + 64) as HalfWord
                }
                else {
                    (c - 64) as HalfWord
                };
                return Goto::Reswitch;
            }
        }
        *self.state_mut() = MID_LINE;
        Goto::Nothing
    }

    // Section 354
    fn sec354_scan_a_control_sequence(&mut self) -> TeXResult<()> {
        if self.loc() > self.limit() {
            self.cur_cs = NULL_CS;
        }
        else {
            'start_cs: loop {
                let mut k = self.loc();
                self.cur_chr = self.buffer[k as usize] as HalfWord;
                let mut cat = cat_code(self.cur_chr);
                k += 1;
                if (cat as QuarterWord) == LETTER || (cat as QuarterWord) == SPACER {
                    *self.state_mut() = SKIP_BLANKS;
                }
                else {
                    *self.state_mut() = MID_LINE;
                }
                if (cat as QuarterWord) == LETTER && k <= self.limit() {
                    // Section 356
                    'sec356: loop {
                        self.cur_chr = self.buffer[k as usize] as HalfWord;
                        cat = cat_code(self.cur_chr);
                        k += 1;
                        if (cat as QuarterWord) != LETTER || k > self.limit() {
                            break 'sec356;
                        }
                    }

                    // Section 355
                    if self.buffer[k as usize] == (self.cur_chr as u8)
                        && (cat as QuarterWord) == SUP_MARK
                        && k < self.limit()
                    {
                        let c = self.buffer[(k + 1) as usize];
                        let mut cc = 0u8;
                        if c < 128 {
                            let mut d = 2;
                            if c.is_ascii_hexdigit() && k + 2 <= self.limit() {
                                cc = self.buffer[(k + 2) as usize];
                                if cc.is_ascii_hexdigit() {
                                    d += 1;
                                }
                            }
                            if d > 2 {
                                hex_to_cur_chr!(c, cc, self.cur_chr);
                                self.buffer[(k - 1) as usize] = self.cur_chr as u8;
                            }
                            else if c < 64 {
                                self.buffer[(k - 1) as usize] = c + 64;
                            }
                            else {
                                self.buffer[(k - 1) as usize] = c - 64;
                            }
                            *self.limit_mut() -= d;
                            self.first -= d;
                            while k <= self.limit() {
                                self.buffer[k as usize] = self.buffer[(k + d) as usize];
                                k += 1;
                            }
                            continue 'start_cs; // Goto start_cs
                        }
                    }
                    // End section 355

                    if (cat as QuarterWord) != LETTER {
                        k -= 1;
                    }
                    if k > self.loc() + 1 {
                        self.cur_cs = self.id_lookup(self.loc() as usize, (k - self.loc()) as usize)?;
                        *self.loc_mut() = k;
                        break 'start_cs; // Goto found
                    }
                    // End section 356
                }
                else {
                    // Section 355
                    if self.buffer[k as usize] == (self.cur_chr as u8)
                        && (cat as QuarterWord) == SUP_MARK
                        && k < self.limit() 
                    {
                        let c = self.buffer[(k + 1) as usize];
                        let mut cc = 0u8;
                        if c < 128 {
                            let mut d = 2;
                            if c.is_ascii_hexdigit() && k + 2 <= self.limit() {
                                cc = self.buffer[(k + 2) as usize];
                                if cc.is_ascii_hexdigit() {
                                    d += 1;
                                }
                            }
                            if d > 2 {
                                hex_to_cur_chr!(c, cc, self.cur_chr);
                                self.buffer[(k - 1) as usize] = self.cur_chr as u8;
                            }
                            else if c < 64 {
                                self.buffer[(k - 1) as usize] = c + 64;
                            }
                            else {
                                self.buffer[(k - 1) as usize] = c - 64;
                            }
                            *self.limit_mut() -= d;
                            self.first -= d;
                            while k <= self.limit() {
                                self.buffer[k as usize] = self.buffer[(k + d) as usize];
                                k += 1;
                            }
                            continue 'start_cs; // Goto start_cs
                        }
                    }
                    // End section 355
                }
                self.cur_cs = SINGLE_BASE + self.buffer[self.loc() as usize] as HalfWord;
                *self.loc_mut() += 1;
                break 'start_cs;
            }
        }

        // found:
        self.cur_cmd = eq_type(self.cur_cs);
        self.cur_chr = equiv(self.cur_cs);
        if self.cur_cmd >= OUTER_CALL {
            self.check_outer_validity()?;
        }
        Ok(())
    }

    // Section 357
    fn sec357_input_from_token_list(&mut self) -> TeXResult<Goto> {
        if self.loc() != NULL {
            let t = info(self.loc());
            *self.loc_mut() = link(self.loc());
            if t >= CS_TOKEN_FLAG {
                self.cur_cs = t - CS_TOKEN_FLAG;
                self.cur_cmd = eq_type(self.cur_cs);
                self.cur_chr = equiv(self.cur_cs);
                if self.cur_cmd >= OUTER_CALL {
                    if self.cur_cmd == DONT_EXPAND {
                        // Section 358
                        self.cur_cs = info(self.loc()) - CS_TOKEN_FLAG;
                        *self.loc_mut() = NULL;
                        self.cur_cmd = eq_type(self.cur_cs);
                        self.cur_chr = equiv(self.cur_cs);
                        if self.cur_cmd > MAX_COMMAND {
                            self.cur_cmd = RELAX;
                            self.cur_chr = NO_EXPAND_FLAG;
                        }
                        // End section 358
                    }
                    else {
                        self.check_outer_validity()?;
                    }
                }
            }
            else {
                self.cur_cmd = (t / 256) as QuarterWord;
                self.cur_chr = t % 256;
                match self.cur_cmd {
                    LEFT_BRACE => self.align_state += 1,
                    RIGHT_BRACE => self.align_state -= 1,
                    OUT_PARAM => {
                        // Section 359
                        self.begin_token_list(self.param_stack[(self.param_start() + self.cur_chr - 1) as usize], PARAMETER)?;
                        return Ok(Goto::Restart);
                        // End section 359
                    },
                    _ => (),
                }
            }
            Ok(Goto::Nothing)
        }
        else {
            self.end_token_list()?;
            Ok(Goto::Restart)
        }
    }

    // Section 360
    fn sec360_move_to_next_line_of_file(&mut self) -> TeXResult<Goto> {
        if self.name() > 17 {
            // Section 362
            self.line += 1;
            self.first = self.start();
            if !self.force_eof {
                if self.input_ln(AlphaFileInSelector::CurFile)? {
                    self.firm_up_the_line()?;
                }
                else {
                    self.force_eof = true;
                }
            }
            if self.force_eof {
                self.print_char(b')');
                self.open_parens -= 1;
                update_terminal!();
                self.force_eof = false;
                self.end_file_reading();
                self.check_outer_validity()?;
                return Ok(Goto::Restart);
            }
            if end_line_char_inactive!() {
                *self.limit_mut() -= 1;
            }
            else {
                self.buffer[self.limit() as usize] = end_line_char() as u8;
            }
            self.first = self.limit() + 1;
            *self.loc_mut() = self.start();
            Ok(Goto::Nothing)
            // End section 362
        }
        else {
            if !self.terminal_input() {
                self.cur_cmd = 0;
                self.cur_chr = 0;
                return Ok(Goto::Exit);
            }
            if self.input_ptr > 0 {
                self.end_file_reading();
                return Ok(Goto::Restart);
            }
            if self.selector < LOG_ONLY {
                self.open_log_file()?;
            }
            if self.interaction > BATCH_MODE {
                if end_line_char_inactive!() {
                    *self.limit_mut() += 1;
                }
                if self.limit() == self.start() {
                    self.print_nl("(Please type a command or say '\\end')");
                }
                self.print_ln();
                self.first = self.start();
                self.prompt_input("*")?;
                *self.limit_mut() = self.last;
                if end_line_char_inactive!() {
                    *self.limit_mut() -= 1;
                }
                else {
                    self.buffer[self.limit() as usize] = end_line_char() as u8;
                }
                self.first = self.limit() + 1;
                *self.loc_mut() = self.start();
                Ok(Goto::Nothing)
            }
            else {
                Err(TeXError::Fatal("*** (job aborted, no legal \\end found)"))
            }
        }
    }

    // Section 363
    pub(crate) fn firm_up_the_line(&mut self) -> TeXResult<()> {
        *self.limit_mut() = self.last;
        if pausing() > 0 && self.interaction > BATCH_MODE {
            self.print_ln();
            if self.start() < self.limit() {
                for k in self.start()..self.limit() {
                    self.print_strnumber(self.buffer[k as usize] as StrNum);
                }
            }
            self.first = self.limit();
            self.prompt_input("=>")?;
            if self.last > self.first {
                for k in self.first..self.last {
                    self.buffer[(k + self.start() - self.first) as usize] = self.buffer[k as usize];
                }
                *self.limit_mut() = self.start() + self.last - self.first;
            }
        }
        Ok(())
    }

    // Section 365
    pub(crate) fn get_token(&mut self) -> TeXResult<()> {
        self.no_new_control_sequence = false;
        self.get_next()?;
        self.no_new_control_sequence = true;
        self.cur_tok = match self.cur_cs {
            0 => (self.cur_cmd as HalfWord)*256 + self.cur_chr,
            _ => CS_TOKEN_FLAG + self.cur_cs,
        };

        Ok(())
    }
}
