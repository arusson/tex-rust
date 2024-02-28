use crate::builder::norm_min;
use crate::constants::*;
use crate::datastructures::{
    info, info_mut, language, left_hyphen_min, link, link_mut,
    right_hyphen_min, subtype, subtype_mut, r#type, type_mut
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, QuarterWord, SmallNumber
};

// Part 53: Extensions

// Section 1341
pub(crate) fn what_lang(p: HalfWord) -> HalfWord {
    link(p + 1)
}

pub(crate) fn what_lang_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 1)
}

pub(crate) fn what_lhm(p: HalfWord) -> QuarterWord {
    r#type(p + 1)
}

pub(crate) fn what_lhm_mut(p: HalfWord) -> &'static mut QuarterWord {
    type_mut(p + 1)
}

pub(crate) fn what_rhm(p: HalfWord) -> QuarterWord {
    subtype(p + 1)
}

pub(crate) fn what_rhm_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p + 1)
}

pub(crate) fn write_tokens(p: HalfWord) -> HalfWord {
    link(p + 1)
}

pub(crate) fn write_tokens_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 1)
}

pub(crate) fn write_stream(p: HalfWord) -> HalfWord {
    info(p + 1)
}

pub(crate) fn write_stream_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 1)
}

pub(crate) fn open_name(p: HalfWord) -> HalfWord {
    link(p + 1)
}

pub(crate) fn open_name_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 1)
}

pub(crate) fn open_area(p: HalfWord) -> HalfWord {
    info(p + 2)
}

pub(crate)fn open_area_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 2)
}

pub(crate) fn open_ext(p: HalfWord) -> HalfWord {
    link(p + 2)
}

pub(crate) fn open_ext_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 2)
}

impl Global {
    // Section 1348
    pub(crate) fn do_extension(&mut self) -> TeXResult<()> {
        match self.cur_chr {
            OPEN_NODE => {
                // Section 1351
                self.new_write_whatsit(OPEN_NODE_SIZE)?;
                self.scan_optional_equals()?;
                self.scan_file_name()?;
                *open_name_mut(self.tail()) = self.cur_name as Integer;
                *open_area_mut(self.tail()) = self.cur_area as Integer;
                *open_ext_mut(self.tail()) = self.cur_ext as Integer;
                // End section 1351
            },
            
            WRITE_NODE => {
                // Section 1352
                let k = self.cur_cs;
                self.new_write_whatsit(WRITE_NODE_SIZE)?;
                self.cur_cs = k;
                _ = self.scan_toks(false, false)?;
                *write_tokens_mut(self.tail()) = self.def_ref;
                // End section 1352
            },
            
            CLOSE_NODE => {
                // Section 1353
                self.new_write_whatsit(WRITE_NODE_SIZE)?;
                *write_tokens_mut(self.tail()) = NULL;
                // End section 1353
            },

            SPECIAL_NODE => {
                // Section 1354
                self.new_whatsit(SPECIAL_NODE as QuarterWord, WRITE_NODE_SIZE)?;
                *write_stream_mut(self.tail()) = NULL;
                _ = self.scan_toks(false, true)?;
                *write_tokens_mut(self.tail()) = self.def_ref;
                // End section 1354
            },

            IMMEDIATE_CODE => {
                // Section 1375
                self.get_x_token()?;
                if self.cur_cmd == EXTENSION && self.cur_chr <= CLOSE_NODE {
                    let p = self.tail();
                    self.do_extension()?;
                    self.out_what(self.tail())?;
                    self.flush_node_list(self.tail())?;
                    *self.tail_mut() = p;
                    *link_mut(p) = NULL;
                }
                else {
                    self.back_input()?;
                }
                // End section 1375
            },

            SET_LANGUAGE_CODE => {
                // Section 1377
                if self.mode().abs() != HMODE {
                    return Err(TeXError::ReportIllegalCase);
                }
                self.new_whatsit(LANGUAGE_NODE as QuarterWord, SMALL_NODE_SIZE)?;
                self.scan_int()?;
                *self.clang_mut() = if (1..=254).contains(&self.cur_val) {
                    self.cur_val
                }
                else {
                    0
                };
                *what_lang_mut(self.tail()) = self.clang();
                *what_lhm_mut(self.tail()) = norm_min(left_hyphen_min()) as QuarterWord;
                *what_rhm_mut(self.tail()) = norm_min(right_hyphen_min()) as QuarterWord;
                // End section 1377
            },

            _ => {
                return Err(TeXError::Confusion("ext1"));
            }
        }
        Ok(())
    }

    // Section 1349
    fn new_whatsit(&mut self, s: SmallNumber, w: Integer) -> TeXResult<()> {
        let p = self.get_node(w)?;
        *type_mut(p) = WHATSIT_NODE;
        *subtype_mut(p) = s;
        *link_mut(self.tail()) = p;
        *self.tail_mut() = p;
        Ok(())
    }

    // Section 1350
    fn new_write_whatsit(&mut self, w: Integer) -> TeXResult<()> {
        self.new_whatsit(self.cur_chr as SmallNumber, w)?;
        if w != WRITE_NODE_SIZE {
            self.scan_four_bit_int()?;
        }
        else {
            self.scan_int()?;
            if self.cur_val < 0 {
                self.cur_val = 17;
            }
            else if self.cur_val > 15 {
                self.cur_val = 16;
            }
        }
        *write_stream_mut(self.tail()) = self.cur_val;
        Ok(())
    }

    // Section 1376
    pub(crate) fn fix_language(&mut self) -> TeXResult<()> {
        let l = if (1..=254).contains(&language()) {
            language()
        }
        else {
            0
        };
        if l != self.clang() {
            self.new_whatsit(LANGUAGE_NODE as QuarterWord, SMALL_NODE_SIZE)?;
            *what_lang_mut(self.tail()) = l;
            *self.clang_mut() = l;
            *what_lhm_mut(self.tail()) = norm_min(left_hyphen_min()) as QuarterWord;
            *what_rhm_mut(self.tail()) = norm_min(right_hyphen_min()) as QuarterWord;
        }
        Ok(())
    }
}
