use crate::constants::*;
use crate::datastructures::{
    mem, mem_mut, Status, equiv, info, link, link_mut, r#box, r#type, subtype,
    subtype_mut, tracing_commands, type_mut
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, QuarterWord, SmallNumber,
    odd, sec406_get_next_nonblank_noncall_token
};

// Part 28: Conditional processing

// Used in function `conditional`
enum Goto {
    CommonEnding,
    Return,
    Some(bool)
}

// Section 489
pub(crate) fn if_line_field(p: HalfWord) -> Integer {
    mem((p + 1) as usize).int()
}

fn if_line_field_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut((p + 1) as usize).int_mut()
}

impl Global {
    // Section 494
    pub(crate) fn pass_text(&mut self) -> TeXResult<()> {
        let save_scanner_status = self.scanner_status;
        self.scanner_status = Status::Skipping;
        let mut l = 0;
        self.skip_line = self.line;
        loop {
            self.get_next()?;
            if self.cur_cmd == FI_OR_ELSE {
                if l == 0 {
                    break; // Goto done
                }
                if self.cur_chr == FI_CODE {
                    l -= 1;
                }
            }
            else if self.cur_cmd == IF_TEST {
                l += 1;
            }
        }
        // done:
        self.scanner_status = save_scanner_status;
        Ok(())
    }

    // Section 496
    pub(crate) fn sec496_pop_the_condition_stack(&mut self) {
        let p = self.cond_ptr;
        self.if_line = if_line_field(p);
        self.cur_if = subtype(p);
        self.if_limit = r#type(p);
        self.cond_ptr = link(p);
        self.free_node(p, IF_NODE_SIZE);
    }

    // Section 497
    fn change_if_limit(&mut self, l: SmallNumber, p: HalfWord) -> TeXResult<()> {
        if p == self.cond_ptr {
            self.if_limit = l;
        }
        else {
            let mut q = self.cond_ptr;
            loop {
                if q == NULL {
                    return Err(TeXError::Confusion("if"));
                }
                if link(q) == p {
                    *type_mut(q) = l;
                    break;
                }
                q = link(q);
            }
        }
        Ok(())
    }

    // Section 498
    pub(crate) fn conditional(&mut self) -> TeXResult<()> {
        // Section 495
        let p = self.get_node(IF_NODE_SIZE)?;
        *link_mut(p) = self.cond_ptr;
        *type_mut(p) = self.if_limit;
        *subtype_mut(p) = self.cur_if;
        *if_line_field_mut(p) = self.if_line;
        self.cond_ptr = p;
        self.cur_if = self.cur_chr as QuarterWord;
        self.if_limit = IF_CODE as QuarterWord;
        self.if_line = self.line;
        // End section 495

        let save_cond_ptr = self.cond_ptr;
        let this_if = self.cur_chr;
        
        'block: {
            // Section 501
            let b = match self.sec501_either_process_ifcase(this_if, save_cond_ptr)? {
                Goto::Return => return Ok(()),
                Goto::CommonEnding => break 'block, // Goto common_ending
                Goto::Some(b) => b
            };
            // End section 501
    
            if tracing_commands() > 1 {
                // Section 502
                self.begin_diagnostic();
                match b {
                    true => self.print("{true}"),
                    false => self.print("{false}"),
                }
                self.end_diagnostic(false);
                // End section 502
            }

            if b {
                self.change_if_limit(ELSE_CODE as QuarterWord, save_cond_ptr)?;
                return Ok(());
            }

            // Section 500
            loop {
                self.pass_text()?;
                if self.cond_ptr == save_cond_ptr {
                    if self.cur_chr != OR_CODE {
                        break 'block; // Goto common_ending
                    }
                    return Err(TeXError::ExtraOr);
                }
                if self.cur_chr == FI_CODE {
                    self.sec496_pop_the_condition_stack();
                }
            }
            // End section 500
        }

        // common_ending:
        if self.cur_chr == FI_CODE {
            self.sec496_pop_the_condition_stack();
        }
        else {
            self.if_limit = FI_CODE as QuarterWord;
        }
        Ok(())
    }

    // Section 501
    fn sec501_either_process_ifcase(&mut self, this_if: HalfWord, save_cond_ptr: HalfWord) -> TeXResult<Goto> {
        // Section 506
        macro_rules! get_x_token_or_active_char {
            () => {
                self.get_x_token()?;
                if self.cur_cmd  == RELAX && self.cur_chr == NO_EXPAND_FLAG {
                    self.cur_cmd = ACTIVE_CHAR;
                    self.cur_chr = self.cur_tok - CS_TOKEN_FLAG - ACTIVE_BASE;
                }
            };
        }
        // End section 506

        let b = match this_if {
            IF_CHAR_CODE
            | IF_CAT_CODE => {
                // Section 506
                get_x_token_or_active_char!();
                let (m, n) = match self.cur_cmd > ACTIVE_CHAR || self.cur_chr > 255 {
                    true => (RELAX, 256),
                    false => (self.cur_cmd, self.cur_chr),
                };

                get_x_token_or_active_char!();
                if self.cur_cmd > ACTIVE_CHAR || self.cur_chr > 255 {
                    self.cur_cmd = RELAX;
                    self.cur_chr = 256;
                }

                match this_if {
                    IF_CHAR_CODE => n == self.cur_chr,
                    _ => m == self.cur_cmd
                }
                // End section 506
            },

            IF_INT_CODE
            | IF_DIM_CODE => {
                // Section 503
                match this_if {
                    IF_INT_CODE => self.scan_int()?,
                    _ => self.scan_dimen(false, false, false)?,
                }
                let n = self.cur_val;
                sec406_get_next_nonblank_noncall_token!(self);
                let r = if self.cur_tok >= OTHER_TOKEN + (b'<' as HalfWord) && self.cur_tok <= OTHER_TOKEN + (b'>' as HalfWord) {
                    (self.cur_tok - OTHER_TOKEN) as u8
                }
                else {
                    return Err(TeXError::MissingEqual(this_if));
                };

                match this_if {
                    IF_INT_CODE => self.scan_int()?,
                    _ => self.scan_dimen(false, false, false)?,
                }
                match r {
                    b'<' => n < self.cur_val,
                    b'=' => n == self.cur_val,
                    _ /* b'>' */ => n > self.cur_val,
                }
                // End section 503
            },

            IF_ODD_CODE => {
                // Section 504
                self.scan_int()?;
                odd!(self.cur_val)
                // End section 504
            },

            IF_VMODE_CODE => self.mode().abs() == VMODE,

            IF_HMODE_CODE => self.mode().abs() == HMODE,

            IF_MMODE_CODE => self.mode().abs() == MMODE,

            IF_INNER_CODE => self.mode() < 0,

            IF_VOID_CODE
            | IF_HBOX_CODE
            | IF_VBOX_CODE => {
                // Section 505
                self.scan_eight_bit_int()?;
                let p = r#box(self.cur_val);
                if this_if == IF_VOID_CODE {
                    p == NULL
                }
                else if p == NULL {
                    false
                }
                else if this_if == IF_HBOX_CODE {
                    r#type(p) == HLIST_NODE
                }
                else {
                    r#type(p) == VLIST_NODE
                }
                // End section 505
            },

            IFX_CODE => self.sec507_test_if_two_tokens_match()?,

            IF_EOF_CODE => {
                self.scan_four_bit_int()?;
                self.read_open[self.cur_val as usize] == CLOSED
            },

            IF_TRUE_CODE => true,

            IF_FALSE_CODE => false,

            IF_CASE_CODE => return self.sec509_select_the_appropriate_case(save_cond_ptr),

            _ => false, // There are no other cases
        };

        Ok(Goto::Some(b))
    }

    // Section 507
    fn sec507_test_if_two_tokens_match(&mut self) -> TeXResult<bool> {
        let save_scanner_status = self.scanner_status;
        self.scanner_status = Status::Normal;
        self.get_next()?;
        let n = self.cur_cs;
        let mut p = self.cur_cmd as HalfWord;
        let mut q = self.cur_chr;
        self.get_next()?;
        let b = if (self.cur_cmd as HalfWord) != p {
            false
        }
        else if self.cur_cmd < CALL {
            self.cur_chr == q
        }
        else {
            // Section 508
            p = link(self.cur_chr);
            q = link(equiv(n));
            if p == q {
                true
            }
            else {
                while p != NULL && q != NULL {
                    if info(p) != info(q) {
                        p = NULL;
                    }
                    else {
                        p = link(p);
                        q = link(q);
                    }
                }
                p == NULL && q == NULL
            }
            // End section 508
        };

        self.scanner_status = save_scanner_status;
        Ok(b)
    }

    // Section 509
    fn sec509_select_the_appropriate_case(&mut self, save_cond_ptr: HalfWord) -> TeXResult<Goto> {
        self.scan_int()?;
        let mut n = self.cur_val;
        if tracing_commands() > 1 {
            self.begin_diagnostic();
            self.print("{case ");
            self.print_int(n);
            self.print_char(b'}');
            self.end_diagnostic(false);
        }
        while n != 0 {
            self.pass_text()?;
            if self.cond_ptr == save_cond_ptr {
                if self.cur_chr == OR_CODE {
                    n -= 1;
                }
                else {
                    return Ok(Goto::CommonEnding);
                }
            }
            else if self.cur_chr == FI_CODE {
                self.sec496_pop_the_condition_stack();
            }
        }
        self.change_if_limit(OR_CODE as QuarterWord, save_cond_ptr)?;
        Ok(Goto::Return)
    }
}
