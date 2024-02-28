use crate::constants::{
    BANNER, BUF_SIZE, EXT_LOG, LOG_ONLY
};
use crate::datastructures::{
    day, end_line_char, month, time, year
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer
};

use std::fs::File;
use std::io::{Write, BufRead, BufReader};

#[allow(clippy::enum_variant_names)]
pub(crate) enum AlphaFileInSelector {
    CurFile,
    ReadFile(usize),
    TermIn
}

pub(crate) enum AlphaFileOutSelector {
    LogFile,
    WriteFile(usize)
}

pub(crate) struct AlphaFileIn {
    file: Option<BufReader<File>>
}

pub(crate) struct AlphaFileOut {
    file: Option<File>
}

impl AlphaFileIn {
    pub(crate) const INIT: Self = Self { file: None };

    pub(crate) fn close(&mut self) {
        self.file = None;
    }
}

impl AlphaFileOut {
    pub(crate) const INIT: Self = Self { file: None };

    pub(crate) fn new() -> Self {
        Self {
            file: None
        }
    }

    pub(crate) fn write_cr(&mut self) {
        if self.file.as_mut().unwrap().write_all(b"\n").is_err() {
            panic!();
        }
    }

    #[cfg(feature = "stat")]
    pub(crate) fn write_ln(&mut self, s: &str) {
        self.write_str(s);
        self.write_cr();
    }

    pub(crate) fn write_str(&mut self, s: &str) {
        if self.file.as_mut().unwrap().write_all(s.as_bytes()).is_err() {
            panic!();
       }
    }

    pub(crate) fn write(&mut self, s: &[u8]) {
        if self.file.as_mut().unwrap().write_all(s).is_err() {
            panic!();
       }
    }

    pub(crate) fn close(&mut self) {
        self.file = None;
    }
}

impl Global {
    // Section 27
    pub(crate) fn a_open_in(&mut self, selection: AlphaFileInSelector) -> bool {
        match File::open(&self.name_of_file) {
            Ok(file) => {
                let alpha_file = match selection {
                    AlphaFileInSelector::CurFile => self.cur_file_mut(),
                    AlphaFileInSelector::ReadFile(n) => &mut self.read_file[n],
                    AlphaFileInSelector::TermIn => return false,
                };
                alpha_file.file = Some(BufReader::new(file));
                true
            },
            Err(_) => false
        }
    }

    pub(crate) fn a_open_out(&mut self, selection: AlphaFileOutSelector) -> TeXResult<()> {
        match File::create(&self.name_of_file) {
            Ok(file) => {
                let alpha_file = match selection {
                    AlphaFileOutSelector::LogFile => &mut self.log_file,
                    AlphaFileOutSelector::WriteFile(j) => &mut self.write_file[j],
                };
                alpha_file.file = Some(file);
                Ok(())
            },
            Err(_) => Err(TeXError::CantWriteFile),
        }
    }
}

impl Global {
    // Section 31
    pub(crate) fn input_ln(&mut self, selection: AlphaFileInSelector) -> TeXResult<bool> {
        self.buffer_string.clear();

        match selection {
            AlphaFileInSelector::CurFile => {
                let index = self.index() as usize;
                if let Some(f) = &mut self.input_file[index].file {
                    match f.read_line(&mut self.buffer_string) {
                        Ok(0) => return Ok(false), // EOF
                        Ok(_) => (),
                        Err(_) => return Err(TeXError::IO("reading input file")),
                    }
                }
            },

            AlphaFileInSelector::ReadFile(n) => {
                if let Some(f) = &mut self.read_file[n].file {
                    match f.read_line(&mut self.buffer_string) {
                        Ok(0) => return Ok(false), // EOF
                        Ok(_) => (),
                        Err(_) => return Err(TeXError::IO("reading read file")),
                    }
                }
            },

            AlphaFileInSelector::TermIn => {
                match std::io::stdin().read_line(&mut self.buffer_string) {
                    Ok(0) => return Ok(false), // EOF
                    Ok(_) => (),
                    Err(_) => return Err(TeXError::IO("reading from terminal")),
                }
            }
        };
        
        // Copy line into `buffer`.
        // For now, we suppose the input is only ASCII.
        let line = self.buffer_string.trim_end().as_bytes();
        self.last = self.first + line.len() as Integer;

        if self.last >= self.max_buf_stack {
            self.max_buf_stack = self.last;
            if self.max_buf_stack >= BUF_SIZE {
                return Err(TeXError::Overflow("buffer size", BUF_SIZE));
            }
        }

        self.buffer[(self.first as usize)..(self.last as usize)].copy_from_slice(line);

        Ok(true)
    }

    // Section 36
    pub(crate) fn loc(&self) -> HalfWord {
        self.cur_input.loc_field
    }

    pub(crate) fn loc_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_input.loc_field
    }

    // Section 534
    pub(crate) fn open_log_file(&mut self) -> TeXResult<()> {
        let old_setting = self.selector;
        if self.job_name == 0 {
            return Err(TeXError::IO("no job file name"));
        }
        self.pack_job_name(EXT_LOG);
        self.a_open_out(AlphaFileOutSelector::LogFile)?;
        self.log_name = self.make_name_string()?;
        self.selector = LOG_ONLY;
        self.log_opened = true;
        self.sec536_print_banner_line();
        self.input_stack[self.input_ptr] = self.cur_input;
        self.print_nl("**");
        let mut l = self.input_stack[0].limit_field;
        if self.buffer[l as usize] == (end_line_char() as u8) {
            l -= 1;
        }
        for k in 0..=l {
            self.print_strnumber(self.buffer[k as usize] as usize);
        }
        self.print_ln();
        self.selector = old_setting + 2;
        Ok(())
    }

    // Section 535
    // No prompt to ask another log file name.

    // Section 536
    fn sec536_print_banner_line(&mut self) {
        self.log_file.write_str(BANNER);
        self.print_strnumber(self.format_ident);
        self.print("  ");
        self.print_int(day());
        self.print_char(b' ');
        self.print(
                match month() {
                    1 => "JAN ",
                    2 => "FEB ",
                    3 => "MAR ",
                    4 => "APR ",
                    5 => "MAY ",
                    6 => "JUN ",
                    7 => "JUL ",
                    8 => "AUG ",
                    9 => "SEP ",
                    10 => "OCT ",
                    11 => "NOV ",
                    _ => "DEC "
            }
        );
        self.print_int(year());
        self.print_char(b' ');
        self.print_two(time() / 60);
        self.print_char(b':');
        self.print_two(time() % 60);
    }
}
