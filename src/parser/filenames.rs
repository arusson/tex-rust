use crate::constants::*;
use crate::datastructures::end_line_char;
use crate::error::{TeXError, TeXResult};
use crate::io::AlphaFileInSelector;
use crate::strings::{
    POOL, append_char, cur_length, flush_string, init_str_ptr, length,
    make_string, pool_ptr, pool_ptr_mut, str_eq_str, str_pool_slice,
    str_ptr, str_ptr_mut, str_room, str_start, str_start_mut
};
use crate::{
    Global, HalfWord, Integer, StrNum, end_line_char_inactive,
    sec406_get_next_nonblank_noncall_token, str_pool_mut, update_terminal
};

use std::io::Write;

// Part 29: File names

impl Global {
    // Section 515
    pub(crate) fn begin_name(&mut self) {
        self.area_delimiter = 0;
        self.ext_delimiter = 0;
    }

    // Section 516
    pub(crate) fn more_name(&mut self, c: u8) -> TeXResult<bool> {
        if c == b' ' {
            Ok(false)
        }
        else {
            str_room(1)?;
            append_char(c);
            if c == b'/' {
                self.area_delimiter = cur_length();
                self.ext_delimiter = 0;
            }
            else if c == b'.' && self.ext_delimiter == 0 {
                self.ext_delimiter = cur_length();
            }
            Ok(true)
        }
    }

    // Section 517
    pub(crate) fn end_name(&mut self) -> TeXResult<()> {
        if str_ptr() + 3 > MAX_STRINGS as usize {
            return Err(TeXError::Overflow("number of strings", MAX_STRINGS - init_str_ptr() as Integer));
        }

        if self.area_delimiter == 0 {
            self.cur_area = EMPTY_STRING;
        }
        else {
            self.cur_area = str_ptr();
            *str_start_mut(str_ptr() + 1) = str_start(str_ptr()) + self.area_delimiter;
            *str_ptr_mut() += 1;
        }

        if self.ext_delimiter == 0 {
            self.cur_ext = EMPTY_STRING;
            self.cur_name = make_string()?;
        }
        else {
            self.cur_name = str_ptr();
            *str_start_mut(str_ptr() + 1) = str_start(str_ptr()) + self.ext_delimiter - self.area_delimiter - 1;
            *str_ptr_mut() += 1;
            self.cur_ext = make_string()?;
        }
        Ok(())
    }

    // Section 519
    pub(crate) fn pack_file_name(&mut self, n: StrNum, a: StrNum, e: StrNum) {
        self.name_of_file.clear();
        self.name_of_file.push_str(
            unsafe {
                std::str::from_utf8_unchecked(str_pool_slice(a))
            }
        );
        self.name_of_file.push_str(
            unsafe {
                std::str::from_utf8_unchecked(str_pool_slice(n))
            }
        );
        self.name_of_file.push_str(
            unsafe {
                std::str::from_utf8_unchecked(str_pool_slice(e))
            }
        );
    }

    // Section 525
    pub(crate) fn make_name_string(&mut self) -> TeXResult<StrNum> {
        let l = self.name_of_file.as_bytes().len();
        if pool_ptr() + l > POOL_SIZE as usize
            || str_ptr() == MAX_STRINGS as usize
            || cur_length() > 0
        {
            Ok(b'?' as StrNum)
        }
        else {
            str_pool_mut![pool_ptr(), pool_ptr() + l].copy_from_slice(self.name_of_file.as_bytes());
            *pool_ptr_mut() += l;
            make_string()
        }
    }

    // Section 526
    pub(crate) fn scan_file_name(&mut self) -> TeXResult<()> {
        self.name_in_progress = true;
        self.begin_name();
        sec406_get_next_nonblank_noncall_token!(self);
        loop {
            if self.cur_cmd > OTHER_CHAR || self.cur_chr > 255 {
                self.back_input()?;
                break; // Goto done
            }

            if !self.more_name(self.cur_chr as u8)? {
                break; // Goto done
            }
            self.get_x_token()?;
        }

        // done:
        self.end_name()?;
        self.name_in_progress = false;
        Ok(())
    }

    // Section 529
    pub(crate) fn pack_cur_name(&mut self) {
        self.pack_file_name(self.cur_name, self.cur_area, self.cur_ext);
    }

    pub(crate) fn pack_job_name(&mut self, s: StrNum) {
        self.cur_area = EMPTY_STRING;
        self.cur_ext = s;
        self.cur_name = self.job_name;
        self.pack_cur_name();
    }

    // Section 530, 531
    // prompt_file_name: no user interaction.

    // Section 537
    pub fn start_input(&mut self) -> TeXResult<()> {
        self.scan_file_name()?;
        if str_eq_str(self.cur_ext, EMPTY_STRING) {
            self.cur_ext = EXT_TEX;
        }
        self.pack_cur_name();
        self.begin_file_reading()?;
        'block: {
            if self.a_open_in(AlphaFileInSelector::CurFile) {
                break 'block; // Goto done
            }
            if str_eq_str(self.cur_area, EMPTY_STRING) {
                self.pack_file_name(self.cur_name, TEX_AREA, self.cur_ext);
                if self.a_open_in(AlphaFileInSelector::CurFile) {
                    break 'block; // Goto done
                }
            }
            self.end_file_reading();
            // No prompt for file
            return Err(TeXError::CantFindFile);
        }

        // done:
        *self.name_mut() = self.make_name_string()? as HalfWord;
        if self.job_name == 0 {
            self.job_name = self.cur_name;
            self.open_log_file()?
        }
        if self.term_offset + length(self.name() as StrNum) as Integer > MAX_PRINT_LINE - 2 {
            self.print_ln();
        }
        else if self.term_offset > 0 || self.file_offset > 0 {
            self.print_char(b' ');
        }
        self.print_char(b'(');
        self.open_parens += 1;
        self.slow_print(self.name() as usize);
        update_terminal!();
        *self.state_mut() = NEW_LINE;
        
        if (self.name() as StrNum) == str_ptr() - 1 {
            flush_string();
            *self.name_mut() = self.cur_name as HalfWord;
        }

        // Section 538
        self.line = 1;
        _ = self.input_ln(AlphaFileInSelector::CurFile)?; // Do nothing
        self.firm_up_the_line()?;
        if end_line_char_inactive!() {
            *self.limit_mut() -= 1;
        }
        else {
            self.buffer[self.limit() as usize] = end_line_char() as u8;
        }
        self.first = self.limit() + 1;
        *self.loc_mut() = self.start();
        // End section 538

        Ok(())
    }
}
