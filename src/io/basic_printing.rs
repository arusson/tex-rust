use crate::constants::*;
use crate::datastructures::{
    escape_char, new_line_char, new_line_char_mut, 
};
use crate::strings::{
    str_pool, XCHR, append_char, pool_ptr, str_ptr, str_start
};
use crate::{
    Global, Integer, Scaled, StrNum, odd
};

#[cfg(feature = "debug")]
use crate::{
    datastructures::MemoryWord,
    Real
};

// Part 5: On-line and off-line printing

impl Global {
    // Section 57
    pub(crate) fn print_ln(&mut self) {
        match self.selector {
            TERM_AND_LOG => {
                println!();
                self.term_offset = 0;
                self.log_file.write_cr();
                self.file_offset = 0;
            },

            TERM_ONLY => {
                println!();
                self.term_offset = 0;
            },

            LOG_ONLY => {
                self.log_file.write_cr();
                self.file_offset = 0;
            },

            NO_PRINT
            | PSEUDO
            | NEW_STRING => (),

            _ => self.write_file[self.selector as usize].write_cr(),
        }
    }

    // Section 58
    pub(crate) fn print_char(&mut self, s: u8) {
        // Section 244 (first condition)
        if s as Integer == new_line_char() && self.selector < PSEUDO {
            self.print_ln();
            return;
        }
        match self.selector {
            TERM_AND_LOG => {
                print!("{}", XCHR[s as usize]);
                self.term_offset += 1;
                if self.term_offset == MAX_PRINT_LINE {
                    println!();
                    self.term_offset = 0;
                }
                self.log_file.write(&[XCHR[s as usize] as u8]);
                self.file_offset += 1;
                if self.file_offset == MAX_PRINT_LINE {
                    self.log_file.write_cr();
                    self.file_offset = 0;
                }
            },

            LOG_ONLY => {
                self.log_file.write(&[XCHR[s as usize] as u8]);
                self.file_offset += 1;
                if self.file_offset == MAX_PRINT_LINE {
                    self.log_file.write_cr();
                    self.file_offset = 0;
                }
            },

            TERM_ONLY => {
                print!("{}", XCHR[s as usize]);
                self.term_offset += 1;
                if self.term_offset == MAX_PRINT_LINE {
                    println!();
                    self.term_offset = 0;
                }
            },

            NO_PRINT => (), // Do nothing

            PSEUDO => {
                if self.tally < self.trick_count {
                    self.trick_buf[(self.tally % ERROR_LINE) as usize] = s;
                }
            },

            NEW_STRING => {
                if pool_ptr() < POOL_SIZE as usize {
                    append_char(s);
                }
            }
            
            _ => self.write_file[self.selector as usize].write(&[XCHR[s as usize] as u8]),
        }
        self.tally += 1;
    }

    // We expect only ASCII for now.
    pub(crate) fn print(&mut self, s: &str) {
        for b in s.bytes() {
            self.print_char(b);
        }
    }

    // Section 59
    pub(crate) fn print_strnumber(&mut self, s: StrNum) {
        if s >= str_ptr() {
            self.print("???");
        }
        else if s < 256 {
            if self.selector > PSEUDO {
                self.print_char(s as u8);
                return;
            }
            // Section 244 (first condition below)
            if s as Integer == new_line_char() && self.selector < PSEUDO {
                return self.print_ln();
            }
            let nl = new_line_char();
            *new_line_char_mut() = -1;
            let mut j = str_start(s);
            while j < str_start(s + 1) {
                self.print_char(str_pool(j));
                j += 1;
            }
            *new_line_char_mut() = nl;
            return;
        }

        let mut j = str_start(s);
        while j < str_start(s + 1) {
            self.print_char(str_pool(j));
            j += 1;
        }
    }

    // Section 60
    pub fn slow_print(&mut self, s: StrNum) {
        if s >= str_ptr() || s < 256 {
            self.print_strnumber(s);
        }
        else {
            let mut j = str_start(s);
            while j < str_start(s + 1) {
                self.print_strnumber(str_pool(j) as usize);
                j += 1;
            }
        }
    }

    // Section 62
    pub(crate) fn print_nl_strnumber(&mut self, s: StrNum) {
        if self.term_offset > 0 && odd!(self.selector)
            || self.file_offset > 0 && self.selector >= LOG_ONLY
        {
            self.print_ln();
        }
        self.print_strnumber(s);
    }

    pub(crate) fn print_nl(&mut self, s: &str) {
        if self.term_offset > 0 && odd!(self.selector)
            || self.file_offset > 0 && self.selector >=  LOG_ONLY
        {
            self.print_ln();
        }
        self.print(s);
    }

    // Section 63
    pub(crate) fn print_esc_strnumber(&mut self, s: StrNum) {
        let c = escape_char();
        if (0..256).contains(&c) {
            self.print_strnumber(c as usize);
        }
        self.slow_print(s);
    }

    // Another one that does not use pool but &str
    pub(crate) fn print_esc(&mut self, s: &str) {
        let c = escape_char();
        if (0..256).contains(&c) {
            self.print_strnumber(c as usize);
        }
        self.print(s);
    }

    // Section 64
    fn print_the_digs(&mut self, mut k: usize) {
        while k > 0 {
            k -= 1;
            if self.dig[k] < 10 {
                self.print_char(self.dig[k] + b'0');
            }
            else {
                self.print_char(b'A' - 10 + self.dig[k]);
            }
        }
    }

    // Section 65
    pub(crate) fn print_int(&mut self, mut n: Integer) {
        let mut k = 0;
        if n < 0 {
            self.print_char(b'-');
            if n > -100_000_000 {
                n = -n;
            }
            else {
                let mut m = -1 - n;
                n = m / 10;
                m = (m % 10) + 1;
                k = 1;
                if m < 10 {
                    self.dig[0] = m as u8;
                }
                else {
                    self.dig[0] = 0;
                    n += 1;
                }
            }
        }
        loop {
            self.dig[k] = (n % 10) as u8;
            n /= 10;
            k += 1;
            if n == 0 {
                break;
            }
        }
        self.print_the_digs(k);
    }

    // Section 66
    pub(crate) fn print_two(&mut self, mut n: Integer) {
        n = n.abs() % 100;
        self.print_char(b'0' + (n / 10) as u8);
        self.print_char(b'0' + (n % 10) as u8);
    }

    // Section 67
    pub(crate) fn print_hex(&mut self, mut n: Integer) {
        let mut k = 0;
        self.print_char(b'"');
        loop {
            self.dig[k] = (n % 16) as u8;
            n /= 16;
            k += 1;
            if n == 0 {
                break;
            }
        }
        self.print_the_digs(k);
    }

    // Section 69
    pub(crate) fn print_roman_int(&mut self, mut n: Integer) {
        let s = b"m2d5c2l5x2v5i";
        let mut j = 0;
        let mut v = 1000;
        loop {
            while n >= v {
                self.print_char(s[j]);
                n -= v;
            }
            if n <= 0 {
                break;
            }
            let mut k = j + 2;
            let mut u = v / (s[k - 1] - b'0') as Integer;
            if s[k - 1] == b'2' {
                k += 2;
                u /= (s[k - 1] - b'0') as Integer;
            }
            if n + u >= v {
                self.print_char(s[k]);
                n += u;
            }
            else {
                j += 2;
                v /= (s[j - 1] - b'0') as Integer;
            }
        }
    }

    // Section 70
    pub(crate) fn print_current_string(&mut self) {
        let mut j = str_start(str_ptr());
        while j < pool_ptr() {
            self.print_char(str_pool(j));
            j += 1;
        }
    }

    // Section 75
    pub fn sec75_initialize_print_selector(&mut self) {
        self.selector = match self.interaction {
            BATCH_MODE => NO_PRINT,
            _ => TERM_ONLY,
        };
    }

    // Section 103
    pub(crate) fn print_scaled(&mut self, mut s: Scaled) {
        if s < 0 {
            self.print_char(b'-');
            s = -s;
        }
        self.print_int(s / UNITY);
        self.print_char(b'.');
        s = (s % UNITY)*10 + 5;
        let mut delta = 10;
        loop {
            if delta > UNITY {
                s += 32768 - 50_000;
            }
            self.print_char(b'0' + (s / UNITY) as u8);
            s = 10*(s % UNITY);
            delta *= 10;
            if s <= delta {
                break;
            }
        }
    }

    // Section 114
    #[cfg(feature = "debug")]
    pub(crate) fn print_word(&mut self, w: MemoryWord) {
        self.print_int(w.int());
        self.print_char(b' ');
        self.print_scaled(w.sc());
        self.print_char(b' ');
        self.print_scaled((w.gr()*(UNITY as Real)).round() as Scaled);
        self.print_ln();
        self.print_int(w.hh_lh());
        self.print_char(b'=');
        self.print_int(w.hh_b0() as Integer);
        self.print_char(b':');
        self.print_int(w.hh_b1() as Integer);
        self.print_char(b';');
        self.print_int(w.hh_rh());
        self.print_char(b' ');
        self.print_int(w.qqqq_b0() as Integer);
        self.print_char(b':');
        self.print_int(w.qqqq_b1() as Integer);
        self.print_char(b':');
        self.print_int(w.qqqq_b2() as Integer);
        self.print_char(b':');
        self.print_int(w.qqqq_b3() as Integer);
    }
}
