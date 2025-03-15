use crate::constants::*;
use crate::datastructures::{
    MemoryWord, eq_level_mut, eq_type_mut, equiv_mut
};
use crate::error::{TeXError, TeXResult};
use crate::strings::{
    POOL, append_char, cur_length, length, make_string, pool_ptr, pool_ptr_set,
    str_ptr, str_room, str_start
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, StrNum,
    str_pool, str_pool_mut
};

use std::ops::{Index, IndexMut};

// Part 18: The hash table

// Section 256
pub(crate) struct Hash([MemoryWord; (UNDEFINED_CONTROL_SEQUENCE - HASH_BASE) as usize]);
pub(crate) static mut HASH: Hash = Hash([MemoryWord::ZERO; (UNDEFINED_CONTROL_SEQUENCE - HASH_BASE) as usize]);

impl Index<usize> for Hash {
    type Output = MemoryWord;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - HASH_BASE as usize]
    }
}

impl IndexMut<usize> for Hash {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - HASH_BASE as usize]
    }
}

pub(crate) fn hash(p: usize) -> MemoryWord {
    unsafe { HASH[p] }
}

pub(crate) fn hash_mut(p: usize) -> &'static mut MemoryWord {
    unsafe { &mut HASH[p] }
}

// Section 256
fn next(p: HalfWord) -> HalfWord {
    hash(p as usize).hh_lh()
}

pub(crate) fn next_mut(p: HalfWord) -> &'static mut HalfWord {
    hash_mut(p as usize).hh_lh_mut()
}

pub(crate) fn text(p: HalfWord) -> HalfWord {
    hash(p as usize).hh_rh()
}

pub(crate) fn text_mut(p: HalfWord) -> &'static mut HalfWord {
    hash_mut(p as usize).hh_rh_mut()
}

impl Global {
    fn hash_is_full(&self) -> bool {
        self.hash_used == HASH_BASE
    }
}

pub(crate) fn font_id_text(p: QuarterWord) -> HalfWord {
    text(FONT_ID_BASE + p as HalfWord)
}

pub(crate) fn font_id_text_mut(p: QuarterWord) -> &'static mut HalfWord {
    text_mut(FONT_ID_BASE + p as HalfWord)
}

impl Global {
    // Section 259
    pub(crate) fn id_lookup(&mut self, j: usize, l: usize) -> TeXResult<HalfWord> {
        // Section 261
        let mut h = self.buffer[j] as HalfWord;
        for k in (j + 1)..(j + l) {
            h = (h + h + self.buffer[k] as HalfWord) % HASH_PRIME;
        }
        // End section 261
        let mut p = h + HASH_BASE;
        loop {
            if text(p) > 0
               && length(text(p) as StrNum) == l 
               && self.str_eq_buf(text(p) as StrNum, j)
            {
                break; // Goto found
            }
            if next(p) == 0 {
                if self.no_new_control_sequence {
                    p = UNDEFINED_CONTROL_SEQUENCE;
                }
                else {
                    // Section 260
                    if text(p) > 0 {
                        loop {
                            if self.hash_is_full() {
                                return Err(TeXError::Overflow("hash size", HASH_SIZE));
                            }
                            self.hash_used -= 1;
                            if text(self.hash_used) == 0 {
                                break;
                            }
                        }
                        *next_mut(p) = self.hash_used;
                        p = self.hash_used;
                    }
                    str_room(l)?;
                    let d = cur_length();
                    while pool_ptr() > str_start(str_ptr()) {
                        pool_ptr_set(pool_ptr() - 1);
                        *str_pool_mut![pool_ptr() + l] = str_pool![pool_ptr()];
                    }
                    for k in j..(j + l) {
                        append_char(self.buffer[k]);
                    }
                    *text_mut(p) = make_string()? as HalfWord;
                    pool_ptr_set(pool_ptr() + d);
                    #[cfg(feature = "stat")]
                    { self.cs_count += 1; }
                    // End section 260
                }
                break; // Goto found
            }
            p = next(p);
        }
        // found:
        Ok(p)
    }

    // Section 264
    // We take a slice as input instead of a pool number
    pub(crate) fn primitive(&mut self, s: &[u8], c: QuarterWord, o: HalfWord) -> TeXResult<()>{
        self.cur_val = if s.len() == 1 {
            s[0] as Integer + SINGLE_BASE
        }
        else {
            self.buffer[..s.len()].copy_from_slice(s);
            // `id_lookup` inserts in the control sequence `str_pool`.
            // Since TEX.POOL file is not used, it is the first
            // and only insertion in the pool. 
            self.id_lookup(0, s.len())?
        };
        *eq_level_mut(self.cur_val) = LEVEL_ONE;
        *eq_type_mut(self.cur_val) = c;
        *equiv_mut(self.cur_val) = o;

        Ok(())
    }
}
