use crate::constants::*;
use crate::datastructures::{
    MemoryWord, info_mut, language, lc_code, link_mut
};
use crate::error::{TeXError, TeXResult};
use crate::strings::{
    POOL,  append_char, length, make_string, str_room, str_start
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, SmallNumber, str_pool
};

use std::ops::{IndexMut, Index};
use std::cmp::Ordering::{Equal, Greater, Less};

// Part 42: Hyphenatation
// Part 43: Initalizing the hyphenation tables

impl Global {
    // Section 921
    pub(crate) fn trie_link(&self, p: usize) -> HalfWord {
        self.trie[p].hh_rh()
    }

    fn trie_link_mut(&mut self, p: usize) -> &mut HalfWord {
        self.trie[p].hh_rh_mut()
    }

    pub(crate) fn trie_char(&self, p: usize) -> QuarterWord {
        self.trie[p].qqqq_b1()
    }

    fn trie_char_mut(&mut self, p: usize) -> &mut QuarterWord {
        self.trie[p].qqqq_b1_mut()
    }

    pub(crate) fn trie_op(&self, p: usize) -> QuarterWord {
        self.trie[p].qqqq_b0()
    }

    fn trie_op_mut(&mut self, p: usize) -> &mut QuarterWord {
        self.trie[p].qqqq_b0_mut()
    }

    // Section 934
    pub(crate) fn set_cur_lang(&mut self) {
        self.cur_lang = if language() <= 0 || language() > 255 {
            0
        }
        else {
            language() as u8
        };
    }

    pub(crate) fn new_hyph_exceptions(&mut self) -> TeXResult<()> {
        self.scan_left_brace()?;
        self.set_cur_lang();
        // Section 935
        let mut n = 0;
        let mut p = NULL;
        loop {
            self.get_x_token()?;
            'reswitch: loop {
                match self.cur_cmd {
                    LETTER
                    | OTHER_CHAR
                    | CHAR_GIVEN => {
                        // Section 937
                        if self.cur_chr == b'-' as HalfWord {
                            // Section 938
                            if n < 63 {
                                let q = self.get_avail()?;
                                *link_mut(q) = p;
                                *info_mut(q) = n;
                                p = q;
                            }
                            // End section 938
                        }
                        else {
                            if lc_code(self.cur_chr) == 0 {
                                return Err(TeXError::NotALetter);
                            }
                            if n < 63 {
                                n += 1;
                                self.hc[n as usize] = lc_code(self.cur_chr) as QuarterWord;
                            }
                        }
                        // End section 937
                        break 'reswitch;
                    },

                    CHAR_NUM => {
                        self.scan_char_num()?;
                        self.cur_chr = self.cur_val;
                        self.cur_cmd = CHAR_GIVEN;
                        // Goto reswitch
                    },

                    SPACER
                    | RIGHT_BRACE => {
                        if n > 1 {
                            // Section 939
                            n += 1;
                            self.hc[n as usize] = self.cur_lang as QuarterWord;
                            str_room(n as usize)?;
                            let mut h = 0;
                            for j in 1..=(n as usize) {
                                h = (h + h + self.hc[j]) % HYPH_SIZE as QuarterWord;
                                append_char(self.hc[j] as u8);
                            }
                            let mut s = make_string()?;
                            // Section 940
                            if self.hyph_count == HYPH_SIZE {
                                return Err(TeXError::Overflow("exception dictionary", HYPH_SIZE));
                            }
                            self.hyph_count += 1;
                            while self.hyph_word[h as usize] != 0 {
                                // Section 941
                                let k = self.hyph_word[h as usize];
                                let found = match length(k).cmp(&length(s)) {
                                    Less => true,
                                    Greater => false,
                                    Equal => {
                                        let (mut u, mut v) = (str_start(k), str_start(s));
                                        loop {
                                            if str_pool![u] < str_pool![v] {
                                                break true; // Goto found
                                            }
                                            if str_pool![u] > str_pool![v] {
                                                break false; // Goto not_found
                                            }
                                            u += 1;
                                            v += 1;
                                            if u == str_start(k + 1) {
                                                break true;
                                            }
                                        }
                                    }
                                };
                                // found:
                                if found {
                                    std::mem::swap(&mut self.hyph_list[h as usize], &mut p);
                                    std::mem::swap(&mut self.hyph_word[h as usize], &mut s);
                                }
                                // not_found:
                                // End section 941

                                if h > 0 {
                                    h -= 1;
                                }
                                else {
                                    h = HYPH_SIZE as QuarterWord;
                                }
                            }
                            self.hyph_word[h as usize] = s;
                            self.hyph_list[h as usize] = p;
                            // End section 940
                            // End section 939
                        }
                        if self.cur_cmd == RIGHT_BRACE {
                            return Ok(());
                        }
                        (n, p) = (0, NULL);
                        break 'reswitch;
                    },

                    _ => return Err(TeXError::ImproperHyphenation)
                }
            }
        }
        // End section 935
    }
}

// Section 943
pub(crate) struct TrieOpHash([usize; (2*TRIE_OP_SIZE + 1) as usize]);

impl TrieOpHash {
    pub(crate) fn new() -> Self {
        TrieOpHash([0; (2*TRIE_OP_SIZE + 1) as usize])
    }
}

impl Index<Integer> for TrieOpHash {
    type Output = usize;
    fn index(&self, index: Integer) -> &Self::Output {
        &self.0[(index + TRIE_OP_SIZE) as usize]
    }
}

impl IndexMut<Integer> for TrieOpHash {
    fn index_mut(&mut self, index: Integer) -> &mut Self::Output {
        &mut self.0[(index + TRIE_OP_SIZE) as usize]
    }
}

impl Global {
    // Section 944
    fn new_trie_op(&mut self, d: SmallNumber, n: SmallNumber, v: QuarterWord) -> TeXResult<QuarterWord> {
        let mut h = (
            ((n as Integer) + 313*(d as Integer) + 361*(v as Integer) + 1009*(self.cur_lang as Integer)).abs()
            % (2*TRIE_OP_SIZE)
        ) - TRIE_OP_SIZE;

        loop {
            let l = self.trie_op_hash[h];
            if l == 0 {
                if self.trie_op_ptr == TRIE_OP_SIZE as usize {
                    return Err(TeXError::Overflow("pattern memory ops", TRIE_OP_SIZE));
                }
                let mut u = self.trie_used[self.cur_lang as usize];
                if u == MAX_QUARTERWORD {
                    return Err(TeXError::Overflow("pattern memory ops per language", (MAX_QUARTERWORD - MIN_QUARTERWORD) as Integer));
                }
                self.trie_op_ptr += 1;
                u += 1;
                self.trie_used[self.cur_lang as usize] = u;
                self.hyf_distance[self.trie_op_ptr] = d;
                self.hyf_num[self.trie_op_ptr] = n;
                self.hyf_next[self.trie_op_ptr]= v;
                self.trie_op_lang[self.trie_op_ptr] = self.cur_lang;
                self.trie_op_hash[h]= self.trie_op_ptr;
                self.trie_op_val[self.trie_op_ptr] = u;
                return Ok(u);
            }
            if self.hyf_distance[l] == d
                && self.hyf_num[l] == n
                && self.hyf_next[l] == v
                && self.trie_op_lang[l] == self.cur_lang
            {
                return Ok(self.trie_op_val[l]);
            }
            if h > -TRIE_OP_SIZE {
                h -= 1;
            }
            else {
                h = TRIE_OP_SIZE;
            }
        }
    }

    // Section 948
    fn trie_node(&mut self, p: usize) -> usize {
        let mut h = (
            (self.trie_c[p] as Integer)
            + 1009*(self.trie_o[p] as Integer)
            + 2718*(self.trie_l[p] as Integer)
            + 3142*(self.trie_r[p] as Integer)
        ).abs() % TRIE_SIZE;

        loop {
            let q = self.trie_hash[h as usize];
            if q == 0 {
                self.trie_hash[h as usize] = p;
                return p;
            }
            if self.trie_c[q] == self.trie_c[p]
                && self.trie_o[q] == self.trie_o[p]
                && self.trie_l[q] == self.trie_l[p]
                && self.trie_r[q] == self.trie_r[p]
            {
                return q;
            }
            h = if h > 0 {
                h - 1
            }
            else {
                TRIE_SIZE
            };
        }
    }

    // Section 949
    fn compress_trie(&mut self, p: usize) -> usize {
        match p {
            0 => 0,
            _ => {
                self.trie_l[p] = self.compress_trie(self.trie_l[p]);
                self.trie_r[p] = self.compress_trie(self.trie_r[p]);
                self.trie_node(p)
            }
        }
    }
}

// Section 950
macro_rules! trie_ref {
    ($s:ident, $p:expr) => [
        $s.trie_hash[$p]
    ];
}

macro_rules! trie_back {
    ($s:ident, $p:expr) => {
        $s.trie[$p].hh_lh()
    };
}

macro_rules! trie_back_mut {
    ($s:ident, $p:expr) => {
        $s.trie[$p].hh_lh_mut()
    };
}

pub(crate) struct TrieTaken([bool; TRIE_SIZE as usize]);

impl Default for TrieTaken {
    fn default() -> Self {
        Self([false; TRIE_SIZE as usize])
    }
}

impl Index<usize> for TrieTaken {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - 1]
    }
}

impl IndexMut<usize> for TrieTaken {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - 1]
    }
}

impl Global {
    // Section 953
    fn first_fit(&mut self, p: usize) -> TeXResult<()> {
        let c = self.trie_c[p] as usize;
        let mut z = self.trie_min[c];
        let h = 'sec953: loop {
            let h = z - c;
            // Section 954
            if self.trie_max < h + 256 {
                if TRIE_SIZE as usize <= h + 256 {
                    return Err(TeXError::Overflow("pattern memory", TRIE_SIZE));
                }
                'inner: loop {
                    self.trie_max += 1;
                    self.trie_taken[self.trie_max] = false;
                    *self.trie_link_mut(self.trie_max) = (self.trie_max + 1) as HalfWord;
                    *trie_back_mut!(self, self.trie_max) = (self.trie_max - 1) as HalfWord;
                    if self.trie_max == h + 256 {
                        break 'inner;
                    }
                }
            }
            // End section 954

            'block: {
                if self.trie_taken[h] {
                    break 'block;
                }
                else {
                    // Section 955
                    let mut q = self.trie_r[p];
                    while q > 0 {
                        if self.trie_link(h + self.trie_c[q] as usize) == 0 {
                            break 'block;
                        }
                        q = self.trie_r[q];
                    }
                    break 'sec953 h;
                    // End section 955
                }
            }
            // not_found:
            z = self.trie_link(z) as usize;
        };

        // found:
        // Section 956
        self.trie_taken[h] = true;
        trie_ref![self, p] = h;
        let mut q = p;
        loop {
            z = h + self.trie_c[q] as usize;
            let mut l = trie_back!(self, z) as usize;
            let r = self.trie_link(z);
            *trie_back_mut!(self, r as usize) = l as HalfWord;
            *self.trie_link_mut(l) = r;
            *self.trie_link_mut(z) = 0;
            if l < 256 {
                let ll = if z < 256 {
                    z
                }
                else {
                    256
                };

                loop {
                    self.trie_min[l] = r as usize;
                    l += 1;
                    if l == ll {
                        break;
                    }
                }
            }
            q = self.trie_r[q];
            if q == 0 {
                break;
            }
        }
        // End section 956

        Ok(())
    }

    // Section 957
    fn trie_pack(&mut self, mut p: usize) -> TeXResult<()> {
        loop {
            let q = self.trie_l[p];
            if q > 0 && trie_ref![self, q] == 0 {
                self.first_fit(q)?;
                self.trie_pack(q)?;
            }
            p = self.trie_r[p];
            if p == 0 {
                break;
            }
        }
        Ok(())
    }

    // Section 959
    fn trie_fix(&mut self, mut p: usize) {
        let z = trie_ref![self, p];
        loop {
            let q = self.trie_l[p];
            let c = self.trie_c[p];
            *self.trie_link_mut(z + c as usize) = trie_ref![self, q] as HalfWord;
            *self.trie_char_mut(z + c as usize) = c as QuarterWord;
            *self.trie_op_mut(z + c as usize) = self.trie_o[p];
            if q > 0 {
                self.trie_fix(q);
            }
            p = self.trie_r[p];
            if p == 0 {
                break;
            }
        }
    }

    // Section 960
    pub(crate) fn new_patterns(&mut self) -> TeXResult<()> {
        if self.trie_not_ready {
            self.set_cur_lang();
            self.scan_left_brace()?;
            self.sec961_enter_all_of_the_patterns()
        }
        else {
            Err(TeXError::TooLateForPatterns)
        }
    }

    // Section 961
    fn sec961_enter_all_of_the_patterns(&mut self) -> TeXResult<()> {
        let mut k = 0;
        self.hyf[0] = 0;
        let mut digit_sensed = false;
        loop {
            self.get_x_token()?;
            match self.cur_cmd {
                LETTER
                | OTHER_CHAR => {
                    // Section 962
                    if digit_sensed
                        || self.cur_chr < b'0' as HalfWord
                        || self.cur_chr > b'9' as HalfWord
                    {
                        if self.cur_chr == b'.' as HalfWord {
                            self.cur_chr = 0;
                        }
                        else {
                            self.cur_chr = lc_code(self.cur_chr);
                            if self.cur_chr == 0 {
                                return Err(TeXError::Nonletter);
                            }
                        }
                        if k < 63 {
                            k += 1;
                            self.hc[k] = self.cur_chr as QuarterWord;
                            self.hyf[k] = 0;
                            digit_sensed = false;
                        }
                    }
                    else if k < 63 {
                        self.hyf[k] = (self.cur_chr - b'0' as HalfWord) as QuarterWord;
                        digit_sensed = true;
                    }
                    // End section 962
                },

                SPACER
                | RIGHT_BRACE => {
                    if k > 0 {
                        // Section 963
                        // Section 965
                        if self.hc[1] == 0 {
                            self.hyf[0] = 0;
                        }
                        if self.hc[k] == 0 {
                            self.hyf[k] = 0;
                        }
                        let mut l = k;
                        let mut v = MIN_QUARTERWORD;
                        loop {
                            if self.hyf[l] != 0 {
                                v = self.new_trie_op((k - l) as QuarterWord, self.hyf[l], v)?;
                            }
                            if l > 0 {
                                l -= 1;
                            }
                            else {
                                break; // Goto done1
                            }
                        }
                        // done1:
                        // End section 965

                        let mut q = 0;
                        self.hc[0] = self.cur_lang as QuarterWord;
                        while l <= k {
                            let c = self.hc[l];
                            l += 1;
                            let mut p = self.trie_l[q];
                            let mut first_child = true;
                            while p > 0 && c > self.trie_c[p] as QuarterWord {
                                q = p;
                                p = self.trie_r[q];
                                first_child = false;
                            }
                            if p == 0 || c < self.trie_c[p] as QuarterWord {
                                // Section 964
                                if self.trie_ptr == TRIE_SIZE as usize {
                                    return Err(TeXError::Overflow("pattern memory", TRIE_SIZE));
                                }
                                self.trie_ptr += 1;
                                self.trie_r[self.trie_ptr] = p;
                                p = self.trie_ptr;
                                self.trie_l[p] = 0;
                                if first_child {
                                    self.trie_l[q] = p;
                                }
                                else {
                                    self.trie_r[q] = p;
                                }
                                self.trie_c[p] = c as u8;
                                self.trie_o[p] = MIN_QUARTERWORD;
                                // End section 964
                            }
                            q = p;
                        }
                        if self.trie_o[q] != MIN_QUARTERWORD {
                            return Err(TeXError::DuplicatePattern);
                        }
                        self.trie_o[q] = v;
                        // End section 963
                    }
                    if self.cur_cmd == RIGHT_BRACE {
                        break; // Goto done
                    }
                    k = 0;
                    self.hyf[0] = 0;
                    digit_sensed = false;
                },

                _ => return Err(TeXError::BadPatterns)
            }
        }
        // done:
        Ok(())
    }

    // Secion 966
    pub(crate) fn init_trie(&mut self) -> TeXResult<()> {
        // Section 947
        macro_rules! trie_root {
            () => {
                self.trie_l[0]
            };
        }
        // End section 947

        // Section 952
        // Section 945
        self.op_start[0] = MIN_QUARTERWORD as usize;
        for j in 1..=255 {
            self.op_start[j] = self.op_start[j - 1] + self.trie_used[j - 1] as usize;
        }
        for j in 1..=self.trie_op_ptr {
            self.trie_op_hash[j as Integer] = self.op_start[self.trie_op_lang[j] as usize] + self.trie_op_val[j] as usize;
        }
        for j in 1..=self.trie_op_ptr {
            while self.trie_op_hash[j as Integer] > j {
                let k = self.trie_op_hash[j as Integer];
                let mut t = self.hyf_distance[k];
                self.hyf_distance[k] = self.hyf_distance[j];
                self.hyf_distance[j] = t;

                t = self.hyf_num[k];
                self.hyf_num[k] = self.hyf_num[j];
                self.hyf_num[j] = t;

                t = self.hyf_next[k];
                self.hyf_next[k] = self.hyf_next[j];
                self.hyf_next[j] = t;

                self.trie_op_hash[j as Integer] = self.trie_op_hash[k as Integer];
                self.trie_op_hash[k as Integer] = k;
            }
        }
        // End section 945

        self.trie_hash[0..=(TRIE_SIZE as usize)].fill(0);
        trie_root!() = self.compress_trie(trie_root!());
        for p in 0..=self.trie_ptr {
            trie_ref![self, p] = 0;
        }
        for p in 0..=255 {
            self.trie_min[p] = p + 1;
        }
        *self.trie_link_mut(0) = 1;
        self.trie_max = 0;
        // End section 952

        if trie_root!() != 0 {
            self.first_fit(trie_root!())?;
            self.trie_pack(trie_root!())?;
        }

        // Section 958
        if trie_root!() == 0 {
            for r in 0..=256 {
                self.trie[r] = MemoryWord::ZERO;
            }
            self.trie_max = 256;
        }
        else {
            self.trie_fix(trie_root!());
            let mut r = 0;
            loop {
                let s = self.trie_link(r);
                self.trie[r] = MemoryWord::ZERO;
                r = s as usize;
                if r > self.trie_max {
                    break;
                }
            }
        }
        *self.trie_char_mut(0) = b'?' as QuarterWord;
        // End section 958

        self.trie_not_ready = false;
        Ok(())
    }
}
