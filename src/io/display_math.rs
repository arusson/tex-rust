use crate::constants::*;
use crate::datastructures::{
    character, info, subtype, r#type
};
use crate::math::{
    fam, large_char, large_fam, math_type, small_char, small_fam, thickness
};
use crate::strings::{
    append_char, cur_length, flush_char
};
use crate::{
    ASCIICode, Global, HalfWord, Integer, QuarterWord,
    accent_chr, delimiter, denominator, left_delimiter,
    nucleus, numerator, right_delimiter, subscr, supscr
};

impl Global {
    // Section 691
    pub(crate) fn print_fam_and_char(&mut self, p: HalfWord) {
        self.print_esc("fam");
        self.print_int(fam(p) as Integer);
        self.print_char(b' ');
        self.print_strnumber(character(p) as usize);
    }

    pub(crate) fn print_delimiter(&mut self, p: HalfWord) {
        let mut a = (small_fam(p) as Integer)*256 + small_char(p) as Integer;
        a = a*4096 + (large_fam(p) as Integer)*256 + large_char(p) as Integer;
        if a < 0 {
            self.print_int(a);
        }
        else {
            self.print_hex(a);
        }
    }

    // Section 692
    pub(crate) fn print_subsidiary_data(&mut self, p: HalfWord, c: ASCIICode) {
        if (cur_length() as Integer) >= self.depth_threshold {
            if math_type(p) != EMPTY {
                self.print(" []");
            }
        }
        else {
            append_char(c);
            self.temp_ptr = p;
            match math_type(p) {
                MATH_CHAR => {
                    self.print_ln();
                    self.print_current_string();
                    self.print_fam_and_char(p);
                },

                SUB_BOX => self.show_node_list(info(self.temp_ptr)), // Section 693

                SUB_MLIST => {
                    if info(p) == NULL {
                        self.print_ln();
                        self.print_current_string();
                        self.print("{}");
                    }
                    else {
                        self.show_node_list(info(self.temp_ptr));
                    }
                },

                _ => (), // Do nothing
            }
            flush_char();
        }
    }

    // Section 694
    pub(crate) fn print_style(&mut self, c: Integer) {
        match c / 2 {
            0 => self.print("displaystyle"),
            1 => self.print("textstyle"),
            2 => self.print("scriptstyle"),
            3 => self.print("scriptscriptstyle"),
            _ => self.print("Unknown style!"),
        }
    }

    // Section 696
    pub(crate) fn sec696_display_normal_noad(&mut self, p: Integer) {
        match r#type(p) {
            ORD_NOAD => self.print_esc("mathord"),
            OP_NOAD => self.print_esc("mathop"),
            BIN_NOAD => self.print_esc("mathbin"),
            REL_NOAD => self.print_esc("mathrel"),
            OPEN_NOAD => self.print_esc("mathopen"),
            CLOSE_NOAD => self.print_esc("mathclose"),
            PUNCT_NOAD => self.print_esc("mathpunct"),
            INNER_NOAD => self.print_esc("mathinner"),
            OVER_NOAD => self.print_esc("overline"),
            UNDER_NOAD => self.print_esc("underline"),
            VCENTER_NOAD => self.print_esc("vcenter"),
            RADICAL_NOAD => {
                self.print_esc("radical");
                self.print_delimiter(left_delimiter!(p));
            },
            ACCENT_NOAD => {
                self.print_esc("accent");
                self.print_fam_and_char(accent_chr!(p));
            },
            LEFT_NOAD => {
                self.print_esc("left");
                self.print_delimiter(delimiter!(p));
            },
            RIGHT_NOAD => {
                self.print_esc("right");
                self.print_delimiter(delimiter!(p));
            },
            _ => (), // No other cases.
        }

        if subtype(p) != NORMAL {
            if subtype(p) == LIMITS {
                self.print_esc("limits");
            }
            else {
                self.print_esc("nomilits");
            }
        }
        if r#type(p) < LEFT_NOAD {
            self.print_subsidiary_data(nucleus!(p), b'.');
        }
        self.print_subsidiary_data(supscr!(p), b'^');
        self.print_subsidiary_data(subscr!(p), b'_');
    }

    // Section 697
    pub(crate) fn sec697_display_fraction_noad(&mut self, p: Integer) {
        self.print_esc("fraction, thickness ");
        if thickness(p) == DEFAULT_CODE {
            self.print("= default");
        }
        else {
            self.print_scaled(thickness(p));
        }
        if small_fam(left_delimiter!(p)) != 0
            || small_char(left_delimiter!(p)) != MIN_QUARTERWORD
            || large_fam(left_delimiter!(p)) != 0
            || large_char(left_delimiter!(p)) != MIN_QUARTERWORD
        {
            self.print(", left-delimiter ");
            self.print_delimiter(left_delimiter!(p));
        }
        if small_fam(right_delimiter!(p)) != 0
            || small_char(right_delimiter!(p)) != MIN_QUARTERWORD
            || large_fam(right_delimiter!(p)) != 0
            || large_char(right_delimiter!(p)) != MIN_QUARTERWORD
        {
            self.print(", right-delimiter ");
            self.print_delimiter(right_delimiter!(p));
        }
        self.print_subsidiary_data(numerator!(p), b'\\');
        self.print_subsidiary_data(denominator!(p), b'/');
    }

    // Section 699
    pub(crate) fn print_size(&mut self, s: Integer) {
        match s as QuarterWord {
            TEXT_SIZE => self.print_esc("textfont"),
            SCRIPT_SIZE => self.print_esc("scriptfont"),
            _ => self.print_esc("scriptscriptfont"),
        }
    }
}
