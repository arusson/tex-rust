use crate::constants::*;
use crate::datastructures::{
    character, character_mut, font, font_mut, info, lig_ptr, lig_ptr_mut,
    link, link_mut, post_break_mut, pre_break_mut, r#type, replace_count_mut,
    subtype, subtype_mut, type_mut
};
use crate::error::TeXResult;
use crate::strings::{
    POOL, length, str_start
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, SmallNumber,
    free_avail, lig_char, odd, str_pool
};

// Part 40: Pre-hyphenation
// Part 41: Post-hyphenation

impl Global {
    // Section 895
    pub(crate) fn hyphenate(&mut self) -> TeXResult<()> {
        // Section 923
        for j in 0..=(self.hn as usize) {
            self.hyf[j] = 0;
        }
        // Section 930
        let mut h = self.hc[1];
        self.hn += 1;
        self.hc[self.hn as usize] = self.cur_lang as QuarterWord;
        for j in 2..=(self.hn as usize) {
            h = (h + h + self.hc[j]) % HYPH_SIZE as QuarterWord;
        }
        'block: {
            'sec930: loop {
                // Section 931
                let k = self.hyph_word[h as usize];
                if k == 0 || length(k) < self.hn as usize {
                    break 'sec930; // Goto not_found
                }
                'innerblock: {
                    if length(k) == self.hn as usize {
                        let mut j = 1;
                        let mut u = str_start(k);
                        
                        'sec931: loop {
                            if str_pool![u] < self.hc[j] as u8 {
                                break 'sec930; // Goto not_found
                            }
                            if str_pool![u] > self.hc[j] as u8 {
                                break 'innerblock; // Goto done
                            }
                            j += 1;
                            u += 1;
                            if j > self.hn as usize {
                                break 'sec931;
                            }
                        }
                        // Section 932
                        let mut s = self.hyph_list[h as usize];
                        while s != NULL {
                            self.hyf[info(s) as usize] = 1;
                            s = link(s);
                        }
                        // End section 932
                        self.hn -= 1;
                        break 'block; // Goto found
                    }
                }
                // done:
                // End section 931

                if h > 0 {
                    h -= 1;
                }
                else {
                    h = HYPH_SIZE as QuarterWord;
                }
            }

            // not_found:
            self.hn -= 1;
            // End section 930

            if self.trie_char((self.cur_lang + 1) as usize) != self.cur_lang as QuarterWord {
                return Ok(())
            }
            self.hc[0] = 0;
            self.hc[(self.hn + 1) as usize] = 0;
            self.hc[(self.hn + 2) as usize] = 256;
            for j in 0..=(self.hn as Integer - self.r_hyf + 1) {
                let mut z = self.trie_link((self.cur_lang + 1) as usize) + self.hc[j as usize] as HalfWord;
                let mut l = j;
                while self.hc[l as usize] == self.trie_char(z as usize) {
                    if self.trie_op(z as usize) != MIN_QUARTERWORD {
                        // Section 924
                        let mut v = self.trie_op(z as usize);
                        loop {
                            v += self.op_start[self.cur_lang as usize] as QuarterWord;
                            let i = l - self.hyf_distance[v as usize] as Integer;
                            if self.hyf_num[v as usize] > self.hyf[i as usize] {
                                self.hyf[i as usize] = self.hyf_num[v as usize];
                            }
                            v = self.hyf_next[v as usize];
                            if v == MIN_QUARTERWORD {
                                break;
                            }
                        }
                        // End section 924
                    }
                    l += 1;
                    z = self.trie_link(z as usize) + self.hc[l as usize] as HalfWord;
                }
            }
        }

        // found:
        self.hyf[0..(self.l_hyf as usize)].fill(0);
        for j in 0..(self.r_hyf as usize) {
            self.hyf[(self.hn as usize) - j] = 0;
        }
        // End section 923

        // Section 902
        'found1: {
            for j in self.l_hyf..=((self.hn as Integer) - self.r_hyf) {
                if odd!(self.hyf[j as usize]) {
                    break 'found1;
                }
            }
            return Ok(());
        }
        // End section 902
        
        // Section 903
        let q = link(self.hb);
        *link_mut(self.hb) = NULL;
        let mut r = link(self.ha);
        *link_mut(self.ha) = NULL;
        let bchar = self.hyf_bchar;

        let found2 = 'block: {
            if self.is_char_node(self.ha) {
                if font(self.ha) != self.hf {
                    break 'block None; // Goto found2
                }
                self.init_list = self.ha;
                self.init_lig = false;
                self.hu[0] = character(self.ha);
            }
            else if r#type(self.ha) == LIGATURE_NODE {
                if font(lig_char!(self.ha)) != self.hf {
                    break 'block None; // Goto found2
                }
                self.init_list = lig_ptr(self.ha);
                self.init_lig = true;
                self.init_lft = subtype(self.ha) > 1;
                self.hu[0] = character(lig_char!(self.ha));
                if self.init_list == NULL && self.init_lft {
                    self.hu[0] = 256;
                    self.init_lig = false;
                }
                self.free_node(self.ha, SMALL_NODE_SIZE);
            }
            else {
                if !self.is_char_node(r)
                    && r#type(r) == LIGATURE_NODE
                    && subtype(r) > 1
                {
                    break 'block None; // Goto found2
                }
                self.init_list = NULL;
                break 'block Some((1, self.ha)); // Goto common_ending
            }
            let mut s = self.cur_p;
            while link(s) != self.ha {
                s = link(s);
            }
            Some((0, s))
        };

        let (mut j, mut s) = match found2 {
            Some((j, s)) => (j, s),
            None => {
                // found2:
                self.hu[0] = 256;
                self.init_lig = false;
                self.init_list = NULL;
                (0, self.ha)
            }
        };

        // common_ending:
        self.flush_node_list(r)?;

        // Section 913
        'sec913: loop {
            let mut l = j;
            j = self.reconstitute(j, self.hn, bchar, self.hyf_char as HalfWord)? + 1;
            if self.hyphen_passed == 0 {
                *link_mut(s) = link(HOLD_HEAD);
                while link(s) > NULL {
                    s = link(s);
                }
                if odd!(self.hyf[(j - 1) as usize]) {
                    l = j;
                    self.hyphen_passed = j - 1;
                    *link_mut(HOLD_HEAD) = NULL;
                }
            }
            if self.hyphen_passed > 0 {
                // Section 914
                let mut c = 0;
                'sec914: loop {
                    r = self.get_node(SMALL_NODE_SIZE)?;
                    *link_mut(r) = link(HOLD_HEAD);
                    *type_mut(r) = DISC_NODE;
                    let mut major_tail = r;
                    let mut r_count = 0;
                    while link(major_tail) > NULL {
                        // advance_major_tail:
                        major_tail = link(major_tail);
                        r_count += 1;
                    }
                    let mut i = self.hyphen_passed;
                    self.hyf[i as usize] = 0;
                    
                    // Section 915
                    let mut minor_tail = NULL;
                    *pre_break_mut(r) = NULL;
                    let hyf_node = self.new_character(self.hf, self.hyf_char as u8)?;
                    if hyf_node != NULL {
                        i += 1;
                        c = self.hu[i as usize];
                        self.hu[i as usize] = self.hyf_char as QuarterWord;
                        free_avail!(self, hyf_node);
                    }
                    while l <= i {
                        l = self.reconstitute(l, i, self.font_bchar[self.hf as usize] as HalfWord, NON_CHAR)? + 1;
                        if link(HOLD_HEAD) > NULL {
                            if minor_tail == NULL {
                                *pre_break_mut(r) = link(HOLD_HEAD);
                            }
                            else {
                                *link_mut(minor_tail) = link(HOLD_HEAD);
                            }
                            minor_tail = link(HOLD_HEAD);
                            while link(minor_tail) > NULL {
                                minor_tail = link(minor_tail);
                            }
                        }
                    }
                    if hyf_node != NULL {
                        self.hu[i as usize] = c;
                        l = i;
                        // Thanks Clippy for finding that!
                        // i -= 1;
                    }
                    // End section 915

                    // Section 916
                    minor_tail = NULL;
                    *post_break_mut(r) = NULL;
                    let mut c_loc = 0;
                    if self.bchar_label[self.hf as usize] != NON_ADDRESS as usize {
                        l -= 1;
                        c = self.hu[l as usize];
                        c_loc = l;
                        self.hu[l as usize] = 256;
                    }
                    while l < j {
                        loop {
                            l = self.reconstitute(l, self.hn, bchar, NON_CHAR)? + 1;
                            if c_loc > 0 {
                                self.hu[c_loc as usize] = c;
                                c_loc = 0;
                            }
                            if link(HOLD_HEAD) > NULL {
                                if minor_tail == NULL {
                                    *post_break_mut(r) = link(HOLD_HEAD);
                                }
                                else {
                                    *link_mut(minor_tail) = link(HOLD_HEAD);
                                }
                                minor_tail = link(HOLD_HEAD);
                                while link(minor_tail) > NULL {
                                    minor_tail = link(minor_tail);
                                }
                            }
                            if l >= j {
                                break;
                            }
                        }
                        while l > j {
                            // Section 917
                            j = self.reconstitute(j, self.hn, bchar, NON_CHAR)? + 1;
                            *link_mut(major_tail) = link(HOLD_HEAD);
                            while link(major_tail) > NULL {
                                // advance_major_tail:
                                major_tail = link(major_tail);
                                r_count += 1;
                            }
                            // End section 917
                        }
                    }
                    // End section 916
                    
                    // Section 918
                    if r_count > 127 {
                        *link_mut(s) = link(r);
                        *link_mut(r) = NULL;
                        self.flush_node_list(r)?;
                    }
                    else {
                        *link_mut(s) = r;
                        *replace_count_mut(r) = r_count as QuarterWord;
                    }
                    s = major_tail;
                    // End section 918

                    self.hyphen_passed = j - 1;
                    *link_mut(HOLD_HEAD) = NULL;
                    if !odd!(self.hyf[(j - 1) as usize]) {
                        break 'sec914;
                    }
                }
                // End section 914
            }
            if j > self.hn {
                break 'sec913;
            }
        }
        *link_mut(s) = q;
        // End section 913

        self.flush_list(self.init_list);
        // End section 903
        Ok(())
    }

    // Section 906
    fn reconstitute(&mut self, mut j: SmallNumber, n: SmallNumber, mut bchar: HalfWord, mut hchar: HalfWord) -> TeXResult<SmallNumber> {
        self.hyphen_passed = 0;
        let mut cur_rh: HalfWord;
        let mut t = HOLD_HEAD;
        let mut w = 0;
        *link_mut(HOLD_HEAD) = NULL;

        // Section 908
        macro_rules! append_charnode_to_t {
            ($p:expr) => {
                *link_mut(t) = self.get_avail()?;
                t = link(t);
                *font_mut(t) = self.hf;
                *character_mut(t) = $p;
            };
        }

        macro_rules! set_cur_r {
            () => {
                self.cur_r = if j < n {
                    self.hu[(j + 1) as usize] as HalfWord
                }
                else {
                    bchar
                };
                cur_rh = if odd!(self.hyf[j as usize]) {
                    hchar
                }
                else {
                    NON_CHAR
                };
            };
        }

        self.cur_l = self.hu[j as usize] as HalfWord;
        self.cur_q = t;
        if j == 0 {
            self.ligature_present = self.init_lig;
            let mut p = self.init_list;
            if self.ligature_present {
                self.lft_hit = self.init_lft;
            }
            while p > NULL {
                append_charnode_to_t!(character(p));
                p = link(p);
            }
        }
        else if self.cur_l < NON_CHAR {
            append_charnode_to_t!(self.cur_l as QuarterWord);
        }
        self.lig_stack = NULL;
        set_cur_r!();
        // End section 908

        // Section 910
        macro_rules! wrap_lig {
            ($b:expr) => {
                if self.ligature_present {
                    let p = self.new_ligature(self.hf, self.cur_l as QuarterWord, link(self.cur_q))?;
                    if self.lft_hit {
                        *subtype_mut(p) = 2;
                        self.lft_hit = false;
                    }
                    if $b && self.lig_stack == NULL {
                        *subtype_mut(p) += 1;
                        self.rt_hit = false;
                    }
                    *link_mut(self.cur_q) = p;
                    t = p;
                    self.ligature_present = false;
                }
            };
        }

        macro_rules! pop_lig_stack {
            () => {
                if lig_ptr(self.lig_stack) > NULL {
                    *link_mut(t) = lig_ptr(self.lig_stack);
                    t = link(t);
                    j += 1;
                }
                let p = self.lig_stack;
                self.lig_stack = link(p);
                self.free_node(p, SMALL_NODE_SIZE);
                if self.lig_stack == NULL {
                    set_cur_r!();
                }
                else {
                    self.cur_r = character(self.lig_stack) as HalfWord;
                }
            };
        }        
        // End section 910

        // continue:
        'sec906: loop {
            // Section 909
            'sec909block: {
                let (mut k, mut q) = if self.cur_l == NON_CHAR {
                    let k = self.bchar_label[self.hf as usize];
                    if k == NON_ADDRESS as usize {
                        break 'sec909block; // Goto done
                    }
                    (k, self.font_info[k])
                }
                else {
                    let mut q = self.char_info(self.hf, self.cur_l as QuarterWord);
                    if q.char_tag() != LIG_TAG {
                        break 'sec909block; // Goto done
                    }
                    let mut k = self.lig_kern_start(self.hf, q);
                    q = self.font_info[k as usize];
                    if q.skip_byte() > STOP_FLAG {
                        k = self.lig_kern_restart(self.hf, q);
                        q = self.font_info[k as usize];
                    }
                    (k as usize, q)
                };

                let test_char = if cur_rh < NON_CHAR {
                    cur_rh
                }
                else {
                    self.cur_r
                };

                loop {
                    if (q.next_char() as HalfWord) == test_char
                        && q.skip_byte() <= STOP_FLAG
                    {
                        if cur_rh < NON_CHAR {
                            self.hyphen_passed = j;
                            hchar = NON_CHAR;
                            cur_rh = NON_CHAR;
                            continue 'sec906; // Goto continue
                        }
                        if hchar < NON_CHAR && odd!(self.hyf[j as usize]) {
                            self.hyphen_passed = j;
                            hchar = NON_CHAR;
                        }
                        if q.op_byte() < KERN_FLAG {
                            // Section 911
                            if self.cur_l == NON_CHAR {
                                self.lft_hit = true;
                            }
                            if j == n && self.lig_stack == NULL {
                                self.rt_hit = true;
                            }
                            self.check_interrupt()?;

                            match q.op_byte() {
                                1 | 5 => {
                                    self.cur_l = q.rem_byte() as HalfWord;
                                    self.ligature_present = true;
                                },

                                2 | 6 => {
                                    self.cur_r = q.rem_byte() as HalfWord;
                                    if self.lig_stack > NULL {
                                        *character_mut(self.lig_stack) = self.cur_r as QuarterWord;
                                    }
                                    else {
                                        self.lig_stack = self.new_lig_item(self.cur_r as QuarterWord)?;
                                        if j == n {
                                            bchar = NON_CHAR;
                                        }
                                        else {
                                            let p = self.get_avail()?;
                                            *lig_ptr_mut(self.lig_stack) = p;
                                            *character_mut(p) = self.hu[(j + 1) as usize];
                                            *font_mut(p) = self.hf;
                                        }
                                    }
                                },

                                3 => {
                                    self.cur_r = q.rem_byte() as HalfWord;
                                    let p = self.lig_stack;
                                    self.lig_stack = self.new_lig_item(self.cur_r as QuarterWord)?;
                                    *link_mut(self.lig_stack) = p;
                                },

                                7 | 11 => {
                                    wrap_lig!(false);
                                    self.cur_q = t;
                                    self.cur_l = q.rem_byte() as HalfWord;
                                    self.ligature_present = true;
                                },

                                _ => {
                                    self.cur_l = q.rem_byte() as HalfWord;
                                    self.ligature_present = true;
                                    if self.lig_stack > NULL {
                                        pop_lig_stack!();
                                    }
                                    else if j == n {
                                        break 'sec909block; // Goto done
                                    }
                                    else {
                                        append_charnode_to_t!(self.cur_r as QuarterWord);
                                        j += 1;
                                        set_cur_r!();
                                    }
                                }
                            }

                            if q.op_byte() > 4 && q.op_byte() != 7 {
                                break 'sec909block; // Goto done
                            }
                            continue 'sec906; // Goto continue
                            // End section 911
                        }
                        w = self.char_kern(self.hf, q);
                        break 'sec909block; // Goto done   
                    }

                    if q.skip_byte() >= STOP_FLAG {
                        if cur_rh == NON_CHAR {
                            break 'sec909block; // Goto done
                        }
                        cur_rh = NON_CHAR;
                        continue 'sec906; // Goto continue
                    }
                    k += (q.skip_byte() + 1) as usize;
                    q = self.font_info[k];
                }
            }
            // done:
            // End section 909

            // Section 910
            wrap_lig!(self.rt_hit);
            if w != 0 {
                *link_mut(t) = self.new_kern(w)?;
                t = link(t);
                w = 0;
            }
            if self.lig_stack > NULL {
                self.cur_q = t;
                self.cur_l = character(self.lig_stack) as HalfWord;
                self.ligature_present = true;
                pop_lig_stack!();
                // Goto continue
            }
            else {
                break 'sec906;
            }
            // End section 910
        }
        Ok(j)
    }
}
