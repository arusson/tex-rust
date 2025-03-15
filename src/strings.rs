use crate::constants::{
    POOL_SIZE, MAX_STRINGS, TEX_AREA_STRING, TEX_FONT_AREA_STRING
};
use crate::error::{TeXError, TeXResult};
use crate::{
    ASCIICode, Global, Integer, StrNum
};

use std::{
    cmp::Ordering,
    ops::{Index, IndexMut, Range}
};

// Part 2: The character set
// Part 4: String Handling

// Section 20, 21, 23, 24
pub(crate) const XCHR: [char; 256] = [
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', '!', '"', '#', '$', '%', '&', '\'',
    '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', ':', ';', '<', '=', '>', '?',
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
    'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
    'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z', '{', '|', '}', '~', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
];

// Section 39
pub(crate) struct StrPool {
    pool: [ASCIICode; POOL_SIZE as usize],
    str_start: [StrNum; (MAX_STRINGS + 1) as usize],
    pool_ptr: usize,
    str_ptr: StrNum,
    init_pool_ptr: usize,
    init_str_ptr: StrNum,
}

pub(crate) static mut POOL: StrPool = StrPool {
    pool: [0; POOL_SIZE as usize],
    str_start: [0; (MAX_STRINGS + 1) as usize],
    pool_ptr: 0,
    str_ptr: 0,
    init_pool_ptr: 0,
    init_str_ptr: 0,
};

impl Index<usize> for StrPool {
    type Output = ASCIICode;
    fn index(&self, index: usize) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<usize> for StrPool {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pool[index]
    }
}

impl Index<Range<usize>> for StrPool {
    type Output = [ASCIICode];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<Range<usize>> for StrPool {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.pool[index]
    }
}

#[macro_export]
macro_rules! str_pool {
    ($p:expr) => [
        unsafe { POOL[$p] }
    ];
    
    ($start:expr, $end:expr) => [
        unsafe { &POOL[$start..$end] }
    ]
}

#[macro_export]
macro_rules! str_pool_mut {
    ($p:expr) => [
        unsafe { &mut POOL[$p] }
    ];
    
    ($start:expr, $end:expr) => [
        unsafe { &mut POOL[$start..$end] }
    ]
}

pub(crate) fn str_start(p: usize) -> StrNum {
    unsafe { POOL.str_start[p] }
}

pub(crate) fn str_start_mut(p: usize) -> &'static mut StrNum {
    unsafe { &mut POOL.str_start[p] }
}

pub fn pool_ptr() -> usize {
    unsafe { POOL.pool_ptr }
}

pub(crate) fn pool_ptr_set(value: usize) {
    unsafe { POOL.pool_ptr = value };
}

pub fn str_ptr() -> StrNum {
    unsafe { POOL.str_ptr }
}

pub(crate) fn str_ptr_set(value: StrNum) {
    unsafe { POOL.str_ptr = value };
}


#[cfg(feature = "stat")]
pub(crate) fn init_pool_ptr() -> usize {
    unsafe { POOL.init_pool_ptr }
}

pub fn init_pool_ptr_set(value: usize) {
    unsafe { POOL.init_pool_ptr = value };
}

pub(crate) fn init_str_ptr() -> StrNum {
    unsafe { POOL.init_str_ptr }
}

pub fn init_str_ptr_set(value: StrNum) {
    unsafe { POOL.init_str_ptr = value };
}

// Section 40
pub(crate) fn length(p: StrNum) -> usize {
    unsafe { POOL.str_start[p + 1] - POOL.str_start[p] }
}

// Section 41
pub(crate) fn cur_length() -> usize {
    unsafe { POOL.pool_ptr - POOL.str_start[POOL.str_ptr] }
}

// Section 42
pub(crate) fn append_char(c: ASCIICode) {
    unsafe {
        POOL.pool[POOL.pool_ptr] = c;
        POOL.pool_ptr += 1;
    }
}

pub(crate) fn flush_char() {
    unsafe { POOL.pool_ptr -= 1 };
}

pub(crate) fn str_room(p: usize) -> TeXResult<()> {
    unsafe {
        match (POOL.pool_ptr + p).cmp(&(POOL_SIZE as usize)) {
            Ordering::Less => Ok(()),
            _ => Err(TeXError::Overflow("pool size", POOL_SIZE - POOL.init_pool_ptr as Integer))
        }
    }
}

// Section 43
pub(crate) fn make_string() -> TeXResult<StrNum> {
    unsafe {
        match POOL.str_ptr as Integer {
            MAX_STRINGS => Err(TeXError::Overflow("number of strings", MAX_STRINGS - POOL.init_str_ptr as Integer)),
            _ => {
                POOL.str_ptr += 1;
                POOL.str_start[POOL.str_ptr] = POOL.pool_ptr;
                Ok(POOL.str_ptr - 1)
            }
        }
    }
}

// Add a string in the pool
pub(crate) fn put_string(s: &[u8]) -> TeXResult<StrNum> {
    unsafe {
        str_room(s.len())?;
        POOL.pool[POOL.pool_ptr..(POOL.pool_ptr + s.len())].copy_from_slice(s);
        POOL.pool_ptr += s.len();
        make_string()
    }
}

// Get a String from a slice of str_pool.
// Valid utf8 bytes are expected.
#[cfg(feature = "debug")]
pub(crate) fn get_string(n: StrNum) -> String {
    unsafe {
        String::from_utf8_unchecked(
            POOL.pool[POOL.str_start[n]..POOL.str_start[n + 1]].into()
        )
    }
}

#[cfg(feature = "debug")]
pub fn dump_pool() {
    unsafe {
        for s in 0..POOL.str_ptr {
            let chaine = get_string(s);
            println!("{chaine}");
        }
    }
}

// Section 44
pub(crate) fn flush_string() {
    unsafe {
        POOL.str_ptr -= 1;
        POOL.pool_ptr = POOL.str_start[POOL.str_ptr];
    }
}

pub(crate) fn str_pool_slice(s: StrNum) -> &'static [ASCIICode] {
    unsafe {
        &POOL.pool[POOL.str_start[s]..POOL.str_start[s + 1]]
    }
}

// Section 46
// Same as above, we compare with Rust slices
pub(crate) fn str_eq_str(s: StrNum, t: StrNum) -> bool {
    if length(s) == length(t) {
        str_pool_slice(s) == str_pool_slice(t)
    }
    else {
        false
    }
}

pub fn get_strings_started() -> TeXResult<()> {

    macro_rules! app_lc_hex {
        ($c:expr) => {
            let l = $c;
            if l < 10 {
                append_char(l + b'0');
            }
            else {
                append_char(l - 10 + b'a');
            }
        };
    }

    pool_ptr_set(0);
    str_ptr_set(0);
    *str_start_mut(0) = 0;

    // Section 48
    for k in 0..=255 {
        // Section 49 (condition)
        if !(32..=126).contains(&k) {
            append_char(b'^');
            append_char(b'^');
            if k < 64 {
                append_char(k + 64);
            }
            else if k < 128 {
                append_char(k - 64);
            }
            else {
                app_lc_hex!(k / 16);
                app_lc_hex!(k % 16);
            }
        }
        else {
            append_char(k);
        }
        _ = make_string()?
    }

    // Section 51
    // We don't use TEX.POOL, but we add some strings below.

    // Empty string "", its number should be 256.
    _ = make_string()?;

    // File extensions, we put it once.
    // ".tex": 257

    _ = put_string(b".tex")?;

    // ".log": 258
    _ = put_string(b".log")?;

    // ".dvi": 259
    _ = put_string(b".dvi")?;

    // ".fmt": 260
    _ = put_string(b".fmt")?;

    // ".tfm": 261
    _ = put_string(b".tfm")?;

    // TEX_AREA: 262
    _ = put_string(TEX_AREA_STRING.as_bytes())?;

    // TEX_FONT_AREA: 263
    _ = put_string(TEX_FONT_AREA_STRING.as_bytes())?;

    // FONT_STRING: 264
    _ = put_string(b"FONT")?;

    // NOTEXPANDED_STRING: 265
    _ = put_string(b"notexpanded:")?;

    // NULLFONT_STRING: 266
    _ = put_string(b"nullfont")?;

    // INACCESSIBLE_STRING: 267
    _ = put_string(b"inaccessible")?;

    // INITEX_IDENT_STRING: 268
    _ = put_string(b" (INITEX)")?;

    // ENDWRITE_STRING: 269
    _ = put_string(b"endwrite")?;

    // ENDTEMPLATE_STRING: 270
    _ = put_string(b"endtemplate")?;
    
    Ok(())
}

impl Global {
    // Section 45
    // We use slices comparison directly
    pub(crate) fn str_eq_buf(&self, s: StrNum, k: usize) -> bool {
        let a = str_pool_slice(s);
        let b = &self.buffer[k..(k + a.len())];
        a == b
    }
}
