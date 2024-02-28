use crate::constants::{
    MAX_QUARTERWORD, NULL, NULL_FONT, POOL_SIZE, TERM_ONLY
};
use crate::datastructures::{EQTB, MEM, info, link};
use crate::error::TeXResult;
use crate::io::term_input_int;
use crate::strings::pool_ptr;
use crate::{
    Global, Integer, QuarterWord, StrNum, update_terminal, mem, eqtb
};

use std::io::Write;

// Part 52: Debugging

macro_rules! read_integer {
    () => {
        match term_input_int()? {
            Some(m) => m,
            None => {
                println!("\n! The input must be an integer.");
                continue;
            }
        }
    };
}

macro_rules! read_integer_prompt {
    ($s:expr) => {
        {
            print!("debug ({}=) # ", $s);
            update_terminal!();
            read_integer!()
        }
    };
}

#[cfg(feature = "debug")]
impl Global {
    pub(crate) fn debug_help(&mut self) -> TeXResult<()> {
        let old_settings = self.selector;
        self.selector = TERM_ONLY;
        println!("-1: exit");
        println!(" 1: `mem[n]`");
        println!(" 2: `info(n)`");
        println!(" 3: `link(n)`");
        println!(" 4: `eqtb[n]`");
        println!(" 5: `font_info[n]`");
        println!(" 6: `save_stack[n]`");
        println!(" 7: `box(n)`");
        println!(" 8: `box(n)` (full)");
        println!(" 9: token list `n`");
        println!("10: internal string `n`");
        println!("11: check_mem(n > 0)");
        println!("12: search_mem(n)");
        println!("13: `print_cmd_chr(n, l)`");
        println!("14: `buffer[0..=n]`");
        println!("15: `short_display(n)`");
        println!("16: `change panicking`");

        loop {
            println!();
            print!("debug # ");
            update_terminal!();

            let m = read_integer!();

            match m {
                1 => {
                    let n = read_integer_prompt!("n");
                    self.print_word(mem![n as usize]);
                },

                2 => {
                    let n = read_integer_prompt!("n");
                    self.print_int(info(n));
                },

                3 => {
                    let n = read_integer_prompt!("n");
                    self.print_int(link(n));
                },

                4 => {
                    let n = read_integer_prompt!("n");
                    self.print_word(eqtb![n as usize]);
                },

                5 => {
                    let n = read_integer_prompt!("n");
                    self.print_word(self.font_info[n as usize]);
                },

                6 => {
                    let n = read_integer_prompt!("n");
                    self.print_word(self.save_stack[n as usize]);
                },

                7 => {
                    let n = read_integer_prompt!("n");
                    self.show_box(n);
                },

                8 => {
                    let n = read_integer_prompt!("n");
                    self.breadth_max = 10_000;
                    self.depth_threshold = POOL_SIZE - (pool_ptr() as Integer) - 10;
                    self.show_node_list(n);
                },

                9 => {
                    let n = read_integer_prompt!("n");
                    self.show_token_list(n, NULL, 1000);
                },

                10 => {
                    let n = read_integer_prompt!("n");
                    if n >= 0 {
                        self.slow_print(n as usize);
                    }
                },

                11 => {
                    let n = read_integer_prompt!("n");
                    self.check_mem(n > 0);
                },

                12 => {
                    let n = read_integer_prompt!("n");
                    self.search_mem(n);
                },

                13 => {
                    let n = read_integer_prompt!("n");
                    let l = read_integer_prompt!("l");
                    if n >= 0 && n <= (MAX_QUARTERWORD as Integer) {
                        self.print_cmd_chr(n as QuarterWord, l);
                    }
                },

                14 => {
                    let n = read_integer_prompt!("n");
                    for k in 0..=n {
                        self.print_strnumber(self.buffer[k as usize] as StrNum);
                    }
                },

                15 => {
                    let n = read_integer_prompt!("n");
                    self.font_in_short_display = NULL_FONT as QuarterWord;
                    self.short_display(n);
                }

                16 => self.panicking = !self.panicking,

                -1 => {
                    self.selector = old_settings;
                    return Ok(());
                },

                _ => println!("?"),
            }
        }
    }
}
