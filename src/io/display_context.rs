use crate::constants::*;
use crate::datastructures::{
    end_line_char, error_context_lines, link
};
use crate::{
    Global, Integer, StrNum
};

impl Global {
    // Section 311
    pub(crate) fn show_context(&mut self) {
        self.base_ptr = self.input_ptr;
        self.input_stack[self.base_ptr] = self.cur_input;
        let mut nn = -1;
        let mut bottom_line = false;
        loop {
            self.cur_input = self.input_stack[self.base_ptr];
            if self.state() != TOKEN_LIST
                && (self.name() > 17 || self.base_ptr == 0)
            {
                bottom_line = true;
            }
            if self.base_ptr == self.input_ptr
                || bottom_line
                || nn < error_context_lines()
            {
                // Section 312
                if self.base_ptr == self.input_ptr
                    || self.state() != TOKEN_LIST
                    || self.token_type() != BACKED_UP
                    || self.loc() != NULL
                {
                    self.tally = 0;
                    self.old_setting = self.selector;
                    let l = if self.state() != TOKEN_LIST {
                        self.sec313_print_location_current_line();
                        self.sec318_pseudoprint_the_line()
                    }
                    else {
                        self.sec314_print_type_token_list();
                        self.sec319_pseudoprint_token_list()
                    };

                    self.selector = self.old_setting;
                    self.sec317_print_two_lines_tricky_pseudoprinted(l);
                    nn += 1;
                }
                // End section 312
            }
            else if nn == error_context_lines() {
                self.print_nl("...");
                nn += 1;
            }
            if bottom_line {
                break; // Goto done
            }
            self.base_ptr -= 1;
        }
        // done:
        self.cur_input = self.input_stack[self.input_ptr];
    }

    // Section 313
    fn sec313_print_location_current_line(&mut self) {
        if self.name() <= 17 {
            if self.terminal_input() {
                if self.base_ptr == 0 {
                    self.print_nl("<*>");
                }
                else {
                    self.print_nl("<insert> ");
                }
            }
            else {
                self.print_nl("<read ");
                if self.name() == 17 {
                    self.print_char(b'*');
                }
                else {
                    self.print_int(self.name() - 1);
                }
                self.print_char(b'>');
            }
        }
        else {
            self.print_nl("l.");
            self.print_int(self.line);
        }
        self.print_char(b' ');
    }

    // Section 314
    fn sec314_print_type_token_list(&mut self) {
        match self.token_type() {
            PARAMETER => self.print_nl("<argument> "),

            U_TEMPLATE
            | V_TEMPLATE => self.print_nl("<template> "),

            BACKED_UP => {
                if self.loc() == NULL {
                    self.print_nl("<recently read> ")
                }
                else {
                    self.print_nl("<to be read again> ")
                }
            },
    
            INSERTED => self.print_nl("<inserted text> "),

            MACRO => {
                self.print_ln();
                self.print_cs(self.name())
            },

            OUTPUT_TEXT => self.print_nl("<output> "),

            EVERY_PAR_TEXT => self.print_nl("<everypar> "),

            EVERY_MATH_TEXT => self.print_nl("<everymath> "),

            EVERY_DISPLAY_TEXT => self.print_nl("<everydisplay> "),

            EVERY_HBOX_TEXT => self.print_nl("<everyhbox> "),

            EVERY_VBOX_TEXT => self.print_nl("<everyvbox> "),

            EVERY_JOB_TEXT => self.print_nl("<everyjob> "),

            EVERY_CR_TEXT => self.print_nl("<everycr> "),

            MARK_TEXT => self.print_nl("<mark> "),

            WRITE_TEXT => self.print_nl("<write> "),

            _ => self.print_nl("?"),
        }
    }

    // Section 316
    fn begin_pseudoprint(&mut self) -> Integer {
        let l = self.tally;
        self.tally = 0;
        self.selector = PSEUDO;
        self.trick_count = 1_000_000;
        l
    }

    pub(crate) fn set_trick_count(&mut self) {
        self.first_count = self.tally;
        self.trick_count = self.tally + 1 + ERROR_LINE - HALF_ERROR_LINE;
        if self.trick_count < ERROR_LINE {
            self.trick_count = ERROR_LINE;
        }
    }

    // Section 317
    fn sec317_print_two_lines_tricky_pseudoprinted(&mut self, l: Integer) {
        if self.trick_count == 1_000_000 {
            self.set_trick_count();
        }
        let m = if self.tally < self.trick_count {
            self.tally - self.first_count
        }
        else {
            self.trick_count - self.first_count
        };

        let (mut p, n) = if l + self.first_count <= HALF_ERROR_LINE {
            (0, l + self.first_count)
        }
        else {
            self.print("...");
            let p = l + self.first_count - HALF_ERROR_LINE + 3;
            (p, HALF_ERROR_LINE)
        };

        for q in (p as usize)..(self.first_count as usize) {
            self.print_char(self.trick_buf[q % (ERROR_LINE as usize)]);
        }
        self.print_ln();
        for _ in 1..=n {
            self.print_char(b' ');
        }
        p = if m + n <= ERROR_LINE {
            self.first_count + m
        }
        else {
            self.first_count + (ERROR_LINE - n - 3)
        };

        for q in self.first_count..p {
            self.print_char(self.trick_buf[(q % ERROR_LINE) as usize]);
        }
        if m + n > ERROR_LINE {
            self.print("...");
        }
    }

    // Section 318
    fn sec318_pseudoprint_the_line(&mut self) -> Integer {
        let l = self.begin_pseudoprint();
        let j = match (self.buffer[self.limit() as usize] as Integer) == end_line_char() {
            true => self.limit(),
            false => self.limit() + 1,
        };
        if j > 0 {
            for i in (self.start() as usize)..(j as usize) {
                if i == self.loc() as usize {
                    self.set_trick_count();
                }
                self.print_strnumber(self.buffer[i] as usize);
            }
        }
        l
    }

    // Section 319
    fn sec319_pseudoprint_token_list(&mut self) -> Integer {
        let l = self.begin_pseudoprint();
        if self.token_type() < MACRO {
            self.show_token_list(self.start(), self.loc(), 100_000);
        }
        else {
            self.show_token_list(link(self.start()), self.loc(), 100_000);
        }
        l
    }

    // Section 518
    pub(crate) fn print_file_name(&mut self, n: StrNum, a: StrNum, e: StrNum) {
        self.slow_print(a);
        self.slow_print(n);
        self.slow_print(e);
    }
}
