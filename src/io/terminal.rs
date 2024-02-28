use crate::error::{TeXError, TeXResult};
use crate::io::AlphaFileInSelector;
use crate::{Global, StrNum};

#[cfg(feature = "debug")]
use crate::Integer;

use std::io::{stdin, Write};

// Section 34
#[macro_export]
macro_rules! update_terminal {
    () => {
        _ = std::io::stdout().flush();
    };
}

#[cfg(feature = "debug")]
pub(crate) fn term_input_int() -> TeXResult<Option<Integer>> {
    let mut input_line = String::new();
    match stdin().read_line(&mut input_line) {
        Ok(_) => (),
        Err(_) => return Err(TeXError::IO("reading from terminal")),
    }

    match input_line.trim().parse::<Integer>() {
        Ok(x) => Ok(Some(x)),
        Err(_) => Ok(None)
    }
}

pub(crate) fn term_input_string() -> TeXResult<String> {
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => Ok(input),
        Err(_) => Err(TeXError::IO("reading from terminal")),
    }
}

impl Global {
    fn term_input(&mut self) -> TeXResult<()> {
        update_terminal!();
        if !self.input_ln(AlphaFileInSelector::TermIn)? {
            return Err(TeXError::Fatal("End of file on the terminal!"));
        }
        self.term_offset = 0;
        self.selector -= 1;
        if self.last != self.first {
            for k in self.first..self.last {
                self.print_strnumber(self.buffer[k as usize] as StrNum);
            }
        }
        self.print_ln();
        self.selector += 1;
        Ok(())
    }

    pub(crate) fn prompt_input(&mut self, s: &str) -> TeXResult<()> {
        self.print(s);
        self.term_input()
    }
}

#[macro_export]
macro_rules! clear_terminal {
    () => {
        print!("\x1B[2J\x1B[1;1H");
    };
}
