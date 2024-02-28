use crate::constants::*;
use crate::datastructures::{
    Status, end_line_char, info, link, link_mut, token_ref_count_mut
};
use crate::error::{TeXError, TeXResult};
use crate::io::AlphaFileInSelector;
use crate::strings::{POOL, pool_ptr, pool_ptr_mut, str_room};
use crate::{
    Global, HalfWord, Integer, QuarterWord, end_line_char_inactive, str_pool
};

// Part 27: Building token lists

impl Global {
    // Section 464
    fn str_toks(&mut self, b: usize) -> TeXResult<HalfWord> {
        str_room(1)?;
        let mut p = TEMP_HEAD;
        *link_mut(p) = NULL;
        let mut k = b;
        while k < pool_ptr() {
            let t = match str_pool![k] {
                b' ' => SPACE_TOKEN,
                _ => OTHER_TOKEN + str_pool![k] as HalfWord,
            };
            self.fast_store_new_token(&mut p, t)?;
            k += 1;
        }
        *pool_ptr_mut() = b;
        Ok(p)
    }

    // Section 465
    pub(crate) fn the_toks(&mut self) -> TeXResult<HalfWord> {
        self.get_x_token()?;
        self.scan_something_internal(TOK_VAL as QuarterWord, false)?;
        if self.cur_val_level >= IDENT_VAL {
            // Section 466
            let mut p = TEMP_HEAD;
            *link_mut(p) = NULL;
            if self.cur_val_level == IDENT_VAL {
                self.store_new_token(&mut p, CS_TOKEN_FLAG + self.cur_val)?;
            }
            else if self.cur_val != NULL {
                let mut r = link(self.cur_val);
                while r != NULL {
                    self.fast_store_new_token(&mut p, info(r))?;
                    r = link(r);
                }
            }
            Ok(p)
            // End section 466
        }
        else {
            let old_setting = self.selector;
            self.selector = NEW_STRING;
            let b = pool_ptr();
            match self.cur_val_level {
                INT_VAL => self.print_int(self.cur_val),

                DIMEN_VAL => {
                    self.print_scaled(self.cur_val);
                    self.print("pt");
                },

                GLUE_VAL => {
                    self.print_spec(self.cur_val, "pt");
                    self.delete_glue_ref(self.cur_val);
                },

                MU_VAL => {
                    self.print_spec(self.cur_val, "mu");
                    self.delete_glue_ref(self.cur_val);
                },

                _ => () // There are no other cases
            }
            self.selector = old_setting;
            self.str_toks(b)
        }
    }

    // Section 467
    pub(crate) fn ins_the_toks(&mut self) -> TeXResult<()> {
        *link_mut(GARBAGE) = self.the_toks()?;
        // ins_list(#) = begin_token_list(#, INSERTED)
        self.begin_token_list(link(TEMP_HEAD), INSERTED)
    }

    // Section 470
    pub(crate) fn conv_toks(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        
        // Section 471
        match c {
            NUMBER_CODE
            | ROMAN_NUMERAL_CODE => self.scan_int()?,

            STRING_CODE
            | MEANING_CODE => {
                let save_scanner_status = self.scanner_status;
                self.scanner_status = Status::Normal;
                self.get_token()?;
                self.scanner_status = save_scanner_status;
            },

            FONT_NAME_CODE => self.scan_font_ident()?,

            JOB_NAME_CODE => {
                if self.job_name == 0 {
                    self.open_log_file()?;
                }
            },

            _ => () // There are no other cases
        }
        // End section 471
        
        let old_setting = self.selector;
        self.selector = NEW_STRING;
        let b = pool_ptr();

        // Section 472
        match c {
            NUMBER_CODE => self.print_int(self.cur_val),

            ROMAN_NUMERAL_CODE => self.print_roman_int(self.cur_val),

            STRING_CODE => {
                if self.cur_cs != 0 {
                    self.sprint_cs(self.cur_cs);
                }
                else {
                    self.print_char(self.cur_chr as u8);
                }
            },

            MEANING_CODE => self.print_meaning(),

            FONT_NAME_CODE => {
                self.print_strnumber(self.font_name[self.cur_val as usize]);
                if self.font_size[self.cur_val as usize] != self.font_dsize[self.cur_val as usize] {
                    self.print(" at ");
                    self.print_scaled(self.font_size[self.cur_val as usize]);
                    self.print("pt");
                }
            },

            JOB_NAME_CODE => self.print_strnumber(self.job_name),
            
            _ => () // There are no other cases
        }
        // End section 472

        self.selector = old_setting;
        *link_mut(GARBAGE) = self.str_toks(b)?;
        self.begin_token_list(link(TEMP_HEAD), INSERTED)
    }

    // Section 473
    pub(crate) fn scan_toks(&mut self, macro_def: bool, xpand: bool) -> TeXResult<HalfWord> {
        self.scanner_status = match macro_def {
            true => Status::Defining,
            false => Status::Absorbing,
        };
        self.warning_index = self.cur_cs;
        self.def_ref = self.get_avail()?;
        *token_ref_count_mut(self.def_ref) = NULL;
        let mut p = self.def_ref;
        let mut hash_brace = 0;
        let mut t = ZERO_TOKEN;
        if macro_def {
            self.sec474_scan_and_build_parameter_part(&mut p, &mut hash_brace, &mut t)?;
        }
        else {
            self.scan_left_brace()?;
        }
        self.sec477_scan_and_build_body(&mut p, macro_def, xpand, t)?;

        // found:
        self.scanner_status = Status::Normal;
        if hash_brace != 0 {
            self.store_new_token(&mut p, hash_brace)?;
        }
        Ok(p)
    }

    // Section 474
    fn sec474_scan_and_build_parameter_part(&mut self, p: &mut HalfWord, hash_brace: &mut HalfWord, t: &mut HalfWord) -> TeXResult<()> {
        loop {
            self.get_token()?;
            if self.cur_tok < RIGHT_BRACE_LIMIT {
                break; // Goto done1
            }
            if self.cur_cmd == MAC_PARAM {
                // Section 476
                let s = MATCH_TOKEN + self.cur_chr;
                self.get_token()?;
                if self.cur_tok < LEFT_BRACE_LIMIT {
                    *hash_brace = self.cur_tok;
                    self.store_new_token(p, self.cur_tok)?;
                    self.store_new_token(p, END_MATCH_TOKEN)?;
                    return Ok(()); // Goto done
                }
                if *t == ZERO_TOKEN + 9 {
                    return Err(TeXError::AlreadyNineParameters);
                }
                else {
                    *t += 1;
                    if self.cur_tok != *t {
                        return Err(TeXError::ParametersNumberedConsecutively);
                    }
                    self.cur_tok = s;
                }
                // End section 476
            }
            self.store_new_token(p, self.cur_tok)?;
        }

        // done1:
        self.store_new_token(p, END_MATCH_TOKEN)?;
        if self.cur_cmd == RIGHT_BRACE {
            Err(TeXError::MissingLeftBrace2)
        }
        else {
            Ok(())
        }
    }

    // Section 477
    fn sec477_scan_and_build_body(&mut self, p: &mut HalfWord, macro_def: bool, xpand: bool, t: HalfWord) -> TeXResult<()> {
        let mut unbalance = 1;
        loop {
            if xpand {
                // Section 478
                'sec478: loop {
                    self.get_next()?;
                    if self.cur_cmd <= MAX_COMMAND {
                        break 'sec478; // Goto done2
                    }
                    if self.cur_cmd != THE {
                        self.expand()?;
                    }
                    else {
                        let q = self.the_toks()?;
                        if link(TEMP_HEAD) != NULL {
                            *link_mut(*p) = link(TEMP_HEAD);
                            *p = q;
                        }
                    }
                }
                // done2:
                self.x_token()?;
                // End section 478
            }
            else {
                self.get_token()?;
            }
            if self.cur_tok < RIGHT_BRACE_LIMIT {
                if self.cur_cmd < RIGHT_BRACE {
                    unbalance += 1;
                }
                else {
                    unbalance -= 1;
                    if unbalance == 0 {
                        return Ok(()); // Goto found
                    }
                }
            }
            else if self.cur_cmd == MAC_PARAM && macro_def {
                // Section 479
                if xpand {
                    self.get_x_token()?;
                }
                else {
                    self.get_token()?;
                }
                if self.cur_cmd != MAC_PARAM {
                    if self.cur_tok <= ZERO_TOKEN || self.cur_tok > t {
                        return Err(TeXError::IllegalParameterNumber);
                    }
                    else {
                        self.cur_tok = OUT_PARAM_TOKEN - (b'0' as HalfWord) + self.cur_chr;
                    }
                }
                // End section 479
            }
            self.store_new_token(p, self.cur_tok)?;
        }
    }

    // Section 482
    pub(crate) fn read_toks(&mut self, mut n: Integer, r: HalfWord) -> TeXResult<()> {
        self.scanner_status = Status::Defining;
        self.warning_index = r;
        self.def_ref = self.get_avail()?;
        *token_ref_count_mut(self.def_ref) = NULL;
        let mut p = self.def_ref;
        self.store_new_token(&mut p, END_MATCH_TOKEN)?;
        let m = match n {
            0..=15 => n,
            _ => 16,
        };
        let s = self.align_state;
        self.align_state = 1_000_000;
        'sec482: loop {
            // Section 483
            self.begin_file_reading()?;
            *self.name_mut() = m + 1;
            match self.read_open[m as usize] {
                CLOSED => {
                    // Section 484
                    if self.interaction != BATCH_MODE {
                        if n < 0 {
                            self.prompt_input("")?;
                        }
                        else {
                            self.print_ln();
                            self.sprint_cs(r);
                            self.prompt_input("=")?;
                            n = -1;
                        }
                    }
                    else {
                        return Err(TeXError::Fatal("*** (cannot \\read from terminal in batch mode)"));
                    }
                    // End section 484
                },

                JUST_OPEN => {
                    // Section 485
                    if self.input_ln(AlphaFileInSelector::ReadFile(m as usize))? {
                        self.read_open[m as usize] = NORMAL as usize;
                    }
                    else {
                        self.read_file[m as usize].close();
                        self.read_open[m as usize] = CLOSED;
                    }
                    // End section 485
                },

                _ => {
                    // Section 486
                    if !self.input_ln(AlphaFileInSelector::ReadFile(m as usize))? {
                        self.read_file[m as usize].close();
                        self.read_open[m as usize] = CLOSED;
                        if self.align_state != 1_000_000 {
                            self.runaway();
                            return Err(TeXError::FileEndedWithin);
                        }
                    }
                    // End section 486
                },
            }

            *self.limit_mut() = self.last;
            if end_line_char_inactive!() {
                *self.limit_mut() -=1;
            }
            else {
                self.buffer[self.limit() as usize] = end_line_char() as u8;
            }
            self.first = self.limit() + 1;
            *self.loc_mut() = self.start();
            *self.state_mut() = NEW_LINE;
    
            'sec483: loop {
                self.get_token()?;
                if self.cur_tok == 0 {
                    break 'sec483; // Goto done
                }
                if self.align_state < 1_000_000 {
                    'inner: loop {
                        self.get_token()?;
                        if self.cur_tok == 0 {
                            break 'inner;
                        }
                    }
                    self.align_state = 1_000_000;
                    break 'sec483; // Goto done
                }
                self.store_new_token(&mut p, self.cur_tok)?;
            }

            // done:
            self.end_file_reading();
            // End section 483

            if self.align_state == 1_000_000 {
                break 'sec482;
            }
        }
        self.cur_val = self.def_ref;
        self.scanner_status = Status::Normal;
        self.align_state = s;
        Ok(())
    }
}
