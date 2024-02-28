use crate::constants::*;
use crate::datastructures::{
    Status, eq_type, info, info_mut, link, link_mut,
    tracing_commands, tracing_macros
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord,  Integer, QuarterWord, StrNum,
    fast_get_avail, free_avail
};

// Part 25: Expanding the next token

impl Global {
    // Section 366
    pub(crate) fn expand(&mut self) -> TeXResult<()> {
        let cv_backup = self.cur_val;
        let cvl_backup = self.cur_val_level;
        let radix_backup = self.radix;
        let co_backup = self.cur_order;
        let backup_backup = link(BACKUP_HEAD);
        if self.cur_cmd < CALL {
            self.sec367_expand_a_nonmacro()?;
        }
        else if self.cur_cmd < END_TEMPLATE {
            self.macro_call()?;
        }
        else {
            // Section 375
            self.cur_tok = CS_TOKEN_FLAG + FROZEN_ENDV;
            self.back_input()?;
            // End section 375
        }
        self.cur_val = cv_backup;
        self.cur_val_level = cvl_backup;
        self.radix = radix_backup;
        self.cur_order = co_backup;
        *link_mut(BACKUP_HEAD) = backup_backup;
        Ok(())
    }

    // Section 367
    fn sec367_expand_a_nonmacro(&mut self) -> TeXResult<()> {
        if tracing_commands() > 1 {
            self.show_cur_cmd_chr();
        }
        match self.cur_cmd {
            TOP_BOT_MARK => {
                // Section 386
                if self.cur_mark[self.cur_chr as usize] != NULL {
                    self.begin_token_list(self.cur_mark[self.cur_chr as usize], MARK_TEXT)?;
                }
                // End section 386
            },

            EXPAND_AFTER => {
                // Section 368
                self.get_token()?;
                let t = self.cur_tok;
                self.get_token()?;
                if self.cur_cmd > MAX_COMMAND {
                    self.expand()?;
                }
                else {
                    self.back_input()?;
                }
                self.cur_tok = t;
                self.back_input()?;
                // End section 368
            },

            NO_EXPAND => {
                // Section 369
                let save_scanner_status = self.scanner_status;
                self.scanner_status = Status::Normal;
                self.get_token()?;
                self.scanner_status = save_scanner_status;
                let t = self.cur_tok;
                self.back_input()?;
                if t >= CS_TOKEN_FLAG {
                    let p = self.get_avail()?;
                    *info_mut(p) = CS_TOKEN_FLAG + FROZEN_DONT_EXPAND;
                    *link_mut(p) = self.loc();
                    *self.start_mut() = p;
                    *self.loc_mut() = p;
                }
                // End section 369
            },

            CS_NAME => self.sec372_manufacture_control_sequence_name()?,

            CONVERT => self.conv_toks()?,

            THE => self.ins_the_toks()?,
            
            IF_TEST => self.conditional()?,

            FI_OR_ELSE => {
                // Section 510
                if self.cur_chr > (self.if_limit as HalfWord) {
                    if self.if_limit == (IF_CODE as QuarterWord) {
                        self.insert_relax()?;
                    }
                    else {
                        return Err(TeXError::ExtraFiOrElse);
                    }
                }
                else {
                    while self.cur_chr != FI_CODE {
                        self.pass_text()?;
                    }
                    self.sec496_pop_the_condition_stack();
                }
                // End section 510
            },

            INPUT => {
                // Section 378
                if self.cur_chr > 0 {
                    self.force_eof = true;
                }
                else if self.name_in_progress {
                    self.insert_relax()?;
                }
                else {
                    self.start_input()?;
                }
                // End section 378
            },

            _ => return Err(TeXError::UndefinedControlSequence) 
        }

        Ok(())
    }

    // Section 371
    pub(crate) fn store_new_token(&mut self, p: &mut HalfWord, info: HalfWord) -> TeXResult<()> {
        let q = self.get_avail()?;
        *link_mut(*p) = q;
        *info_mut(q) = info;
        *p = q;
        Ok(())
    }

    pub(crate) fn fast_store_new_token(&mut self, p: &mut HalfWord, info: HalfWord) -> TeXResult<()> {
        let mut q: HalfWord;
        fast_get_avail!(self, q);
        *link_mut(*p) = q;
        *info_mut(q) = info;
        *p = q;
        Ok(())
    }

    // Section 372
    fn sec372_manufacture_control_sequence_name(&mut self) -> TeXResult<()> {
        let r = self.get_avail()?;
        let mut p = r;
        loop {
            self.get_x_token()?;
            if self.cur_cs == 0 {
                self.store_new_token(&mut p, self.cur_tok)?;
            }
            if self.cur_cs != 0 {
                break;
            }
        }
        if self.cur_cmd != END_CS_NAME {
            return Err(TeXError::MissingEncCSName);
        }

        // Section 374
        let mut j = self.first;
        p = link(r);
        while p != NULL {
            if j >= self.max_buf_stack {
                self.max_buf_stack = j + 1;
                if self.max_buf_stack == BUF_SIZE {
                    return Err(TeXError::Overflow("buffer size", BUF_SIZE));
                }
            }
            self.buffer[j as usize] = (info(p) % 256) as u8;
            j += 1;
            p = link(p);
        }
        if j > self.first + 1 {
            self.no_new_control_sequence = false;
            self.cur_cs = self.id_lookup(self.first as usize, (j - self.first) as usize)?;
            self.no_new_control_sequence = true;
        }
        else if j == self.first {
            self.cur_cs = NULL_CS;
        }
        else {
            self.cur_cs = SINGLE_BASE + self.buffer[self.first as usize] as HalfWord;
        }
        // End section 374

        self.flush_list(r);
        if eq_type(self.cur_cs) == UNDEFINED_CS {
            self.eq_define(self.cur_cs, RELAX, 256)?;
        }
        self.cur_tok = self.cur_cs + CS_TOKEN_FLAG;
        self.back_input()
    }

    // Section 379
    fn insert_relax(&mut self) -> TeXResult<()> {
        self.cur_tok = CS_TOKEN_FLAG + self.cur_cs;
        self.back_input()?;
        self.cur_tok = CS_TOKEN_FLAG + FROZEN_RELAX;
        self.back_input()?;
        *self.token_type_mut() = INSERTED;
        Ok(())
    }

    // Section 380
    pub(crate) fn get_x_token(&mut self) -> TeXResult<()> {
        'restart: loop {
            self.get_next()?;
            if self.cur_cmd <= MAX_COMMAND {
                break 'restart; // Goto done
            }
            if self.cur_cmd >= CALL {
                if self.cur_cmd < END_TEMPLATE {
                    self.macro_call()?;
                }
                else {
                    self.cur_cs = FROZEN_ENDV;
                    self.cur_cmd = ENDV;
                    break 'restart; // Goto done
                }
            }
            else {
                self.expand()?;
            }
        }

        // done:
        self.cur_tok = match self.cur_cs {
            0 => (self.cur_cmd as HalfWord)*256 + self.cur_chr,
            _ => CS_TOKEN_FLAG + self.cur_cs
        };
        Ok(())
    }

    // Section 381
    pub(crate) fn x_token(&mut self) -> TeXResult<()> {
        while self.cur_cmd > MAX_COMMAND {
            self.expand()?;
            self.get_next()?;
        }
        self.cur_tok = match self.cur_cs {
            0 => (self.cur_cmd as HalfWord)*256 + self.cur_chr,
            _ => CS_TOKEN_FLAG + self.cur_cs
        };
        Ok(())
    }

    // Section 382
    pub(crate) fn top_mark(&self) -> HalfWord {
        self.cur_mark[TOP_MARK_CODE as usize]
    }

    pub(crate) fn first_mark(&self) -> HalfWord {
        self.cur_mark[FIRST_MARK_CODE as usize]
    }

    pub(crate) fn bot_mark(&self) -> HalfWord {
        self.cur_mark[BOT_MARK_CODE as usize]
    }

    pub(crate) fn split_first_mark(&self) -> HalfWord {
        self.cur_mark[SPLIT_FIRST_MARK_CODE as usize]
    }

    pub(crate) fn split_bot_mark(&self) -> HalfWord {
        self.cur_mark[SPLIT_BOT_MARK_CODE as usize]
    }

    pub(crate) fn top_mark_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_mark[TOP_MARK_CODE as usize]
    }

    pub(crate) fn first_mark_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_mark[FIRST_MARK_CODE as usize]
    }

    pub(crate) fn bot_mark_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_mark[BOT_MARK_CODE as usize]
    }

    pub(crate) fn split_first_mark_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_mark[SPLIT_FIRST_MARK_CODE as usize]
    }

    pub(crate) fn split_bot_mark_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_mark[SPLIT_BOT_MARK_CODE as usize]
    }

    // Section 389
    fn macro_call(&mut self) -> TeXResult<()> {
        let save_scanner_status = self.scanner_status;
        let save_warning_index = self.warning_index;
        self.warning_index = self.cur_cs;
        let ref_count = self.cur_chr;
        let mut r = link(ref_count);
        let mut n = 0;
        if tracing_macros() > 0 {
            // Section 401
            self.begin_diagnostic();
            self.print_ln();
            self.print_cs(self.warning_index);
            self.token_show(ref_count);
            self.end_diagnostic(false);
            // End section 401
        }
        if info(r) != END_MATCH_TOKEN {
            (r, n) = self.sec391_scan_the_parameters_and_make(r, n)?;
        }

        // Section 390
        while self.state() == TOKEN_LIST
            && self.loc() == NULL
            && self.token_type() != V_TEMPLATE
        {
            self.end_token_list()?;
        }
        self.begin_token_list(ref_count, MACRO)?;
        *self.name_mut() = self.warning_index;
        *self.loc_mut() = link(r);
        if n > 0 {
            if self.param_ptr + n > self.max_param_stack {
                self.max_param_stack = self.param_ptr + n;
                if self.max_param_stack > PARAM_SIZE as usize {
                    return Err(TeXError::Overflow("parameter stack size", PARAM_SIZE));
                }
            }
            for m in 0..n {
                self.param_stack[self.param_ptr + m] = self.pstack[m];
            }
            self.param_ptr += n;
        }
        // End section 390

        // exit:
        self.scanner_status = save_scanner_status;
        self.warning_index = save_warning_index;
        Ok(())
    }

    // Section 391
    fn sec391_scan_the_parameters_and_make(&mut self, mut r: HalfWord, mut n: usize) -> TeXResult<(HalfWord, usize)> {
        self.scanner_status = Status::Matching;
        let mut unbalance;
        self.long_state = eq_type(self.cur_cs);
        if self.long_state >= OUTER_CALL {
            self.long_state -= 2;
        }
        'sec391: loop {
            *link_mut(TEMP_HEAD) = NULL;
            let mut match_chr: HalfWord = 0;
            let s: HalfWord;
            let mut p: HalfWord = 0;
            let mut m: HalfWord = 0;
            if info(r) > MATCH_TOKEN + 255 || info(r) < MATCH_TOKEN {
                s = NULL;
            }
            else {
                match_chr = info(r) - MATCH_TOKEN;
                s = link(r);
                r = s;
                p = TEMP_HEAD;
                m = 0;
            }
            let mut rbrace_ptr: HalfWord = 0;

            // Section 392
            // continue:
            'sec392: loop {
                self.get_token()?;
                if self.cur_tok == info(r) {
                    // Section 394
                    r = link(r);
                    if (MATCH_TOKEN..=END_MATCH_TOKEN).contains(&info(r)) {
                        if self.cur_tok < LEFT_BRACE_LIMIT {
                            self.align_state -= 1;
                        }
                        break 'sec392; // Goto found
                    }
                    continue 'sec392; // Goto continue
                    // End section 394
                }

                // Section 397
                if s != r {
                    if s == NULL {
                        return Err(TeXError::DoesNotMatchDefinition);
                    }
                    let mut t = s;
                    'sec397: loop {
                        self.store_new_token(&mut p, info(t))?;
                        m += 1;
                        let mut u = link(t);
                        let mut v = s;
                        'inner: loop {
                            if u == r {
                                if self.cur_tok != info(v) {
                                    break 'inner; // Goto done
                                }
                                r = link(v);
                                continue 'sec392;
                            }
                            if info(u) != info(v) {
                                break 'inner; // Goto done
                            }
                            u = link(u);
                            v = link(v);
                        }
                        // done:
                        t = link(t);
                        if t == r {
                            break 'sec397;
                        }
                    }
                    r = s;
                }
                // End section 397

                if self.cur_tok == self.par_token && self.long_state != LONG_CALL {
                    // Section 396
                    self.runaway();
                    return Err(TeXError::ParagraphEndedBefore);
                    // End section 396
                }
                if self.cur_tok < RIGHT_BRACE_LIMIT {
                    if self.cur_tok < LEFT_BRACE_LIMIT {
                        // Section 399
                        unbalance = 1;
                        'sec399: loop {
                            self.fast_store_new_token(&mut p, self.cur_tok)?;
                            self.get_token()?;
                            if self.cur_tok == self.par_token && self.long_state != LONG_CALL {
                                self.runaway();
                                return Err(TeXError::ParagraphEndedBefore);
                            }
                            if self.cur_tok < RIGHT_BRACE_LIMIT {
                                if self.cur_tok < LEFT_BRACE_LIMIT {
                                    unbalance += 1;
                                }
                                else {
                                    unbalance -= 1;
                                    if unbalance == 0 {
                                        break 'sec399; // Goto done1
                                    }
                                }
                            }
                        }
                        // done1:
                        rbrace_ptr = p;
                        self.store_new_token(&mut p, self.cur_tok)?;
                        // End section 399
                    }
                    else {
                        return Err(TeXError::ArgumentExtraRightBrace);
                    }
                }
                else {
                    // Section 393
                    if self.cur_tok == SPACE_TOKEN && (MATCH_TOKEN..=END_MATCH_TOKEN).contains(&info(r)) {
                        continue 'sec392; // Goto continue
                    }
                    self.store_new_token(&mut p, self.cur_tok)?;
                    // End section 393
                }
                m += 1;
                if (MATCH_TOKEN..=END_MATCH_TOKEN).contains(&info(r)) {
                    break 'sec392;
                }
            }

            // found:
            if s != NULL {
                // Section 400
                if m == 1 && info(p) < RIGHT_BRACE_LIMIT && p != TEMP_HEAD {
                    *link_mut(rbrace_ptr) = NULL;
                    free_avail!(self, p);
                    p = link(TEMP_HEAD);
                    self.pstack[n] = link(p);
                    free_avail!(self, p);
                }
                else {
                    self.pstack[n] = link(TEMP_HEAD);
                }
                n += 1;
                if tracing_macros() > 0 {
                    self.begin_diagnostic();
                    self.print_nl_strnumber(match_chr as StrNum);
                    self.print_int(n as Integer);
                    self.print("<-");
                    self.show_token_list(self.pstack[n - 1], NULL, 1000);
                    self.end_diagnostic(false);
                }
                // End section 400
            }
            // End section 392

            if info(r) == END_MATCH_TOKEN {
                break 'sec391;
            }
        }
        Ok((r, n))
    }
}
