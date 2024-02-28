use crate::constants::*;
use crate::datastructures::{
    adjust_ptr, adjust_ptr_mut, glue_ptr, glue_ptr_mut, glue_ref_count,
    glue_ref_count_mut, ins_ptr, ins_ptr_mut, leader_ptr, leader_ptr_mut,
    lig_ptr, lig_ptr_mut, list_ptr, list_ptr_mut, mark_ptr, post_break,
    post_break_mut, pre_break, pre_break_mut, r#type, split_top_ptr, subtype
};
use crate::error::{TeXError, TeXResult};
use crate::extensions::write_tokens;
use crate::math::{
    display_mlist, math_type, script_mlist, script_script_mlist, text_mlist
};
use crate::{
    Global, GlueRatio, HalfWord, Integer, QuarterWord, Scaled,
    denominator, lig_char, nucleus, numerator, subscr, supscr
};

#[cfg(feature = "debug")]
use crate::datastructures::equiv;

// Part 8: Packed data

// Section 112
#[macro_export]
macro_rules! hi {
    ($a:expr) => {
        $a + MIN_HALFWORD
    };
}

#[macro_export]
macro_rules! ho {
    ($a:expr) => {
        $a - MIN_HALFWORD
    };
}

#[derive(Clone, Copy)]
pub(crate) union MemoryWord {
    pub(crate) int: Integer,
    pub(crate) sc: Scaled,
    pub(crate) gr: GlueRatio,
    pub(crate) hh: [HalfWord; 2],
    pub(crate) qqqq: [QuarterWord; 4],
    pub(crate) word: u64
}

impl Default for MemoryWord {
    fn default() -> Self {
        Self::ZERO
    }
}

impl MemoryWord {
    pub(crate) const ZERO: MemoryWord = MemoryWord { word: 0 };

    pub(crate) fn word(&self) -> u64 {
        unsafe { self.word }
    }

    pub(crate) fn sc(&self) -> Scaled {
        unsafe { self.sc }
    }

    pub(crate) fn sc_mut(&mut self) -> &mut Scaled {
        unsafe { &mut self.sc }
    }

    pub(crate) fn gr(&self) -> GlueRatio {
        unsafe { self.gr }
    }

    pub(crate) fn gr_mut(&mut self) -> &mut GlueRatio {
        unsafe { &mut self.gr }
    }

    pub(crate) fn int(&self) -> Integer {
        unsafe { self.int }
    }

    pub(crate) fn int_mut(&mut self) -> &mut Integer {
        unsafe { &mut self.int }
    }

    pub(crate) fn qqqq_b0(&self) -> QuarterWord {
        unsafe { self.qqqq[0] }
    }

    pub(crate) fn qqqq_b0_mut(&mut self) -> &mut QuarterWord {
        unsafe { &mut self.qqqq[0] }
    }

    pub(crate) fn qqqq_b1(&self) -> QuarterWord {
        unsafe { self.qqqq[1] }
    }

    pub(crate) fn qqqq_b1_mut(&mut self) -> &mut QuarterWord {
        unsafe { &mut self.qqqq[1] }
    }

    pub(crate) fn qqqq_b2(&self) -> QuarterWord {
        unsafe { self.qqqq[2] }
    }

    pub(crate) fn qqqq_b2_mut(&mut self) -> &mut QuarterWord {
        unsafe { &mut self.qqqq[2] }
    }

    pub(crate) fn qqqq_b3(&self) -> QuarterWord {
        unsafe { self.qqqq[3] }
    }

    pub(crate) fn qqqq_b3_mut(&mut self) -> &mut QuarterWord {
        unsafe { &mut self.qqqq[3] }
    }

    pub(crate) fn hh_b0(&self) -> QuarterWord {
        self.qqqq_b0()
    }

    pub(crate) fn hh_b0_mut(&mut self) -> &mut QuarterWord {
        self.qqqq_b0_mut()
    }

    pub(crate) fn hh_b1(&self) -> QuarterWord {
        self.qqqq_b1()
    }

    pub(crate) fn hh_b1_mut(&mut self) -> &mut QuarterWord {
        self.qqqq_b1_mut()
    }

    pub(crate) fn hh_rh(&self) -> HalfWord {
        unsafe { self.hh[1] }
    }
    
    pub(crate) fn hh_rh_mut(&mut self) -> &mut HalfWord {
        unsafe { &mut self.hh[1] }
    }

    pub(crate) fn hh_lh(&self) -> HalfWord {
        unsafe { self.hh[0] }
    }
    
    pub(crate) fn hh_lh_mut(&mut self) -> &mut HalfWord {
        unsafe { &mut self.hh[0] }
    }
}

// Part 9: Dynamic memory allocation

pub(crate) static mut MEM: [MemoryWord; (MEM_MAX - MEM_MIN + 1) as usize] = [MemoryWord::ZERO; (MEM_MAX - MEM_MIN + 1) as usize];

#[macro_export]
macro_rules! mem {
    ($p:expr) => {
        unsafe { MEM[$p] }
    };
}

#[macro_export]
macro_rules! mem_mut {
    ($p:expr) => {
        unsafe { &mut MEM[$p] }
    };
}

// Section 118
pub(crate) fn link(p: HalfWord) -> HalfWord {
    mem![p as usize].hh_rh()
}

pub(crate) fn link_mut(p: HalfWord) -> &'static mut HalfWord {
    mem_mut![p as usize].hh_rh_mut()
}

pub(crate) fn info(p: HalfWord) -> HalfWord {
    mem![p as usize].hh_lh()
}

pub(crate) fn info_mut(p: HalfWord) -> &'static mut HalfWord {
    mem_mut![p as usize].hh_lh_mut()
}

impl Global {
    // Section 120
    pub(crate) fn get_avail(&mut self) -> TeXResult<HalfWord> {
        let mut p = self.avail;
        match p {
            NULL => {
                if self.mem_end < MEM_MAX {
                    self.mem_end += 1;
                    p = self.mem_end;
                }
                else {
                    self.hi_mem_min -= 1;
                    p = self.hi_mem_min;
                    if self.hi_mem_min <= self.lo_mem_max {
                        self.runaway();
                        return Err(TeXError::Overflow("main memory size", MEM_MAX + 1 - MEM_MIN))
                    }
                }
            }
            _ => self.avail = link(self.avail),
        }

        *link_mut(p) = NULL;
        #[cfg(feature = "stat")]
        { self.dyn_used += 1; }
        Ok(p)
    }
}

// Section 121
#[macro_export]
macro_rules! free_avail {
    ($s:ident, $p:expr) => {
        *link_mut($p) = $s.avail;
        $s.avail = $p;
        #[cfg(feature = "stat")]
        { $s.dyn_used -= 1; }
    };
}

// Section 122
#[macro_export]
macro_rules! fast_get_avail {
    ($s:ident, $p:expr) => {
        $p = $s.avail;
        if $p == NULL {
            $p = $s.get_avail()?;
        }
        else {
            $s.avail = link($p);
            *link_mut($p) = NULL;
            #[cfg(feature = "stat")]
            { $s.dyn_used += 1; }
        }
    };
}

impl Global {
    // Section 123
    pub(crate) fn flush_list(&mut self, p: HalfWord) {
        if p != NULL {
            let mut r = p;
            let q = loop {
                let q = r;
                r = link(r);
                #[cfg(feature = "stat")]
                { self.dyn_used -= 1; }
                if r == NULL {
                    break q; // Last node on the list
                }
            };
            *link_mut(q) = self.avail;
            self.avail = p;
        }
    }
}

// Section 124
macro_rules! is_empty {
    ($p:expr) => {
        link($p) == EMPTY_FLAG    
    };
}

pub(crate) fn node_size(p: HalfWord) -> HalfWord {
    info(p)
}

pub(crate) fn node_size_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p)
}

pub(crate) fn llink(p: HalfWord) -> HalfWord {
    info(p + 1)
}

pub(crate) fn llink_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 1)
}

pub(crate) fn rlink(p: HalfWord) -> HalfWord {
    link(p + 1)
}

pub(crate) fn rlink_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 1)
}

impl Global {
    // Section 125
    pub(crate) fn get_node(&mut self, s: Integer) -> TeXResult<HalfWord> {
        // restart:
        loop {
            let mut p = self.rover;

            loop {
                if let Some(node) = self.sec127_try_to_allocate(p, s) {
                    // found:
                    *link_mut(node) = NULL;
                    #[cfg(feature = "stat")]
                    { self.var_used += s; }
                    return Ok(node);
                }
                p = rlink(p);
                if p == self.rover {
                    break;
                }
            }

            if s == 0x4000_0000 {
                return Ok(MAX_HALFWORD)
            }
            if self.lo_mem_max + 2 < self.hi_mem_min
                && self.lo_mem_max + 2 <= MEM_BOT + MAX_HALFWORD
            {
                self.sec126_grow_more();
            }
            else {
                return Err(TeXError::Overflow("main memory size", MEM_MAX + 1 - MEM_MIN));
            }
        }
    }

    // Section 126
    fn sec126_grow_more(&mut self) {
        let mut t = if self.hi_mem_min - self.lo_mem_max >= 1998 {
            self.lo_mem_max + 1000
        }
        else {
            self.lo_mem_max + 1 + (self.hi_mem_min - self.lo_mem_max) / 2
        };
        let p = llink(self.rover);
        let q = self.lo_mem_max;
        *rlink_mut(p) = q;
        *llink_mut(self.rover) = q;

        if t > MEM_BOT + MAX_HALFWORD {
            t = MEM_BOT + MAX_HALFWORD;
        }
        *rlink_mut(q) = self.rover;
        *llink_mut(q) = p;
        *link_mut(q) = EMPTY_FLAG;
        *node_size_mut(q) = t - self.lo_mem_max;
        self.lo_mem_max = t;
        *link_mut(self.lo_mem_max) = NULL;
        *info_mut(self.lo_mem_max) = NULL;
        self.rover = q;
        // goto restart
    }

    // Section 127
    fn sec127_try_to_allocate(&mut self, p: HalfWord, s: Integer) -> Option<HalfWord> {
        let mut q = p + node_size(p);
        while is_empty!(q) {
            let t = rlink(q);
            if q == self.rover {
                self.rover = t;
            }
            *llink_mut(t) = llink(q);
            *rlink_mut(llink(q)) = t;
            q += node_size(q);
        }
        let r = q - s;
        if r > p + 1 {
            // Section 128
            *node_size_mut(p) = r - p;
            self.rover = p;
            return Some(r);
            // End section 128
        }
        if r == p && rlink(p) != p {
            // Section 129
            self.rover = rlink(p);
            let t = llink(p);
            *llink_mut(self.rover) = t;
            *rlink_mut(t) = self.rover;
            return Some(r);
            // End section 129
        }
        *node_size_mut(p) = q - p;
        None
    }

    // Section 130
    pub(crate) fn free_node(&mut self, p: HalfWord, s: HalfWord) {
        *node_size_mut(p) = s;
        *link_mut(p) = EMPTY_FLAG;
        let q = llink(self.rover);
        *llink_mut(p) = q;
        *rlink_mut(p) = self.rover;
        *llink_mut(self.rover) = p;
        *rlink_mut(q) = p;
        #[cfg(feature = "stat")]
        { self.var_used -= s; }
    }

    // Section 131
    pub(crate) fn sort_avail(&mut self) -> TeXResult<()> {
        let _ = self.get_node(0x4000_0000)?;
        let mut p = rlink(self.rover);
        *rlink_mut(self.rover) = MAX_HALFWORD;
        let old_rover = self.rover;
        while p != old_rover {
            // Section 132
            if p < self.rover {
                let q = p;
                p = rlink(q);
                *rlink_mut(q) = self.rover;
                self.rover = q;
            }
            else {
                let mut q = self.rover;
                while rlink(q) < p {
                    q = rlink(q);
                }
                let r = rlink(p);
                *rlink_mut(p) = rlink(q);
                *rlink_mut(q) = p;
                p = r;
            }
            // End secction 132
        }
        p = self.rover;
        while rlink(p) != MAX_HALFWORD {
            *llink_mut(rlink(p)) = p;
            p = rlink(p);
        }
        *rlink_mut(p) = self.rover;
        *llink_mut(self.rover) = p;
        Ok(())
    }
}

// Part 11: Memory layout

#[cfg(feature = "debug")]
impl Global {
    // Section 167
    pub(crate) fn check_mem(&mut self, print_locs: bool) {
        for p in MEM_MIN..=self.lo_mem_max {
            self.free[p as usize] = false;
        }
        for p in self.hi_mem_min..=self.mem_end {
            self.free[p as usize] = false;
        }
        self.sec168_check_single_word_avail_list();
        self.sec169_check_variable_size_avail_list();
        self.sec170_check_flags_of_unavailable_nodes();
        if print_locs {
            self.sec171_print_newly_busy_locations();
        }
        for p in MEM_MIN..=self.lo_mem_max {
            self.was_free[p as usize] = self.free[p as usize];
        }
        for p in self.hi_mem_min..=self.mem_end {
            self.was_free[p as usize] = self.free[p as usize];
        }
        self.was_mem_end = self.mem_end;
        self.was_lo_max = self.lo_mem_max;
        self.was_hi_min = self.hi_mem_min;
    }

    // Section 168
    fn sec168_check_single_word_avail_list(&mut self) {
        let mut p = self.avail;
        let mut q = NULL;
        while p != NULL {
            if p > self.mem_end || p < self.hi_mem_min || self.free[p as usize] {
                self.print_nl("AVAIL list clobbered at ");
                self.print_int(q);
                break; // Goto done1
            }
            self.free[p as usize] = true;
            q = p;
            p = link(q);
        }
        // done1:
    }

    // Section 169
    fn sec169_check_variable_size_avail_list(&mut self) {
        let mut p = self.rover;
        let mut q = NULL;
        'outer: loop {
            if p >= self.lo_mem_max
                || p < MEM_MIN
                || rlink(p) >= self.lo_mem_max
                || rlink(p) < MEM_MIN
                || !is_empty!(p)
                || node_size(p) < 2
                || p + node_size(p) > self.lo_mem_max
                || llink(rlink(p)) != p
            {
                self.print_nl("Double-AVAIL list clobbered at ");
                self.print_int(q);
                break 'outer; // Goto done2
            }
            for q in p..(p + node_size(p)) {
                if self.free[q as usize] {
                    self.print_nl("Doubly free location at ");
                    self.print_int(q);
                    break 'outer; // Goto done2
                }
                self.free[q as usize] = true;
            }
            q = p;
            p = rlink(p);
            if p == self.rover {
                break 'outer;
            }
        }
        // done2:
    }
    
    // Section 170
    fn sec170_check_flags_of_unavailable_nodes(&mut self) {
        let mut p = MEM_MIN;
        while p <= self.lo_mem_max {
            if is_empty!(p) {
                self.print_nl("Bad flag at ");
                self.print_int(p);
            }
            while p <= self.lo_mem_max && !self.free[p as usize] {
                p += 1;
            }
            while p <= self.lo_mem_max && self.free[p as usize] {
                p += 1;
            }
        }
    }

    // Section 171
    fn sec171_print_newly_busy_locations(&mut self) {
        self.print_nl("New busy locs:");
        for p in MEM_MIN..=self.lo_mem_max {
            if !self.free[p as usize] && (p > self.was_lo_max || self.was_free[p as usize]) {
                self.print_char(b' ');
                self.print_int(p);
            }
        }
        for p in self.hi_mem_min..=self.mem_end {
            if !self.free[p as usize]
                && (p < self.was_hi_min || p > self.was_mem_end || self.was_free[p as usize])
            {
                self.print_char(b' ');
                self.print_int(p);
            }
        }
    }

    // Section 172
    #[cfg(feature = "debug")]
    pub(crate) fn search_mem(&mut self, p: HalfWord) {
        for q in MEM_MIN..=self.lo_mem_max {
            if link(q) == p {
                self.print_nl("LINK(");
                self.print_int(q);
                self.print_char(b')');
            }
            if info(q) == p {
                self.print_nl("INFO(");
                self.print_int(q);
                self.print_char(b')');
            }
        }
        for q in self.hi_mem_min..=self.mem_end {
            if link(q) == p {
                self.print_nl("LINK(");
                self.print_int(q);
                self.print_char(b')');
            }
            if info(q) == p {
                self.print_nl("INFO(");
                self.print_int(q);
                self.print_char(b')');
            }
        }

        // Section 255
        for q in ACTIVE_BASE..=(BOX_BASE + 255) {
            if equiv(q) == p {
                self.print_nl("EQUIV(");
                self.print_int(q);
                self.print_char(b')');
            }
        }
        // End section 255
        
        // Section 285
        if self.save_ptr > 0 {
            for q in 0..self.save_ptr {
                if self.save_stack[q].equiv_field() == p {
                    self.print_nl("SAVE(");
                    self.print_int(q as Integer);
                    self.print_char(b')');
                }
            }
        }
        // End section 285
        
        // Section 933
        for q in 0..=HYPH_SIZE {
            if self.hyph_list[q as usize] == p {
                self.print_nl("HYPH(");
                self.print_int(q);
                self.print_char(b')');
            }
        }
        // End section 933
    }
}

// Part 13: Destroying boxes

// Section 200
fn token_ref_count(p: HalfWord) -> HalfWord {
    info(p)
}

pub(crate) fn token_ref_count_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p)
}

impl Global {
    pub(crate) fn delete_token_ref(&mut self, p: HalfWord) {
        if token_ref_count(p) == NULL {
            self.flush_list(p);
        }
        else {
            *token_ref_count_mut(p) -= 1;
        }
    }
}

// Section 201
macro_rules! fast_delete_glue_ref {
    ($s:ident, $p:expr) => {
        if glue_ref_count($p) == NULL {
            $s.free_node($p, GLUE_SPEC_SIZE);
        }
        else {
            *glue_ref_count_mut($p) -= 1;
        }
    };
}

impl Global {
    // Section 201
    pub(crate) fn delete_glue_ref(&mut self, p: HalfWord) {
        fast_delete_glue_ref!(self, p);
    }

    // Section 202
    pub(crate) fn flush_node_list(&mut self, mut p: HalfWord) -> TeXResult<()> {
        while p != NULL {
            let q = link(p);
            if self.is_char_node(p) {
                free_avail!(self, p);
            }
            else {
                let mut done = false;
                match r#type(p) {
                    HLIST_NODE
                    | VLIST_NODE
                    | UNSET_NODE => {
                        self.flush_node_list(list_ptr(p))?;
                        self.free_node(p, BOX_NODE_SIZE);
                        done = true;
                    },

                    RULE_NODE => {
                        self.free_node(p, RULE_NODE_SIZE);
                        done = true;
                    },

                    INS_NODE => {
                        self.flush_node_list(ins_ptr(p))?;
                        self.delete_glue_ref(split_top_ptr(p));
                        self.free_node(p, INS_NODE_SIZE);
                        done = true;
                    },

                    WHATSIT_NODE => {
                        // Section 1358
                        match subtype(p) as Integer {
                            OPEN_NODE => self.free_node(p, OPEN_NODE_SIZE),

                            WRITE_NODE
                            | SPECIAL_NODE => {
                                self.delete_token_ref(write_tokens(p));
                                self.free_node(p, WRITE_NODE_SIZE);
                            },

                            CLOSE_NODE
                            | LANGUAGE_NODE => self.free_node(p, SMALL_NODE_SIZE),

                            _ => return Err(TeXError::Confusion("ext3")),
                        }
                        done = true;
                        // End section 1358
                    },

                    GLUE_NODE => {
                        fast_delete_glue_ref!(self, glue_ptr(p));
                        if leader_ptr(p) != NULL {
                            self.flush_node_list(leader_ptr(p))?;
                        }
                    },

                    KERN_NODE
                    | MATH_NODE
                    | PENALTY_NODE => (),

                    LIGATURE_NODE => self.flush_node_list(lig_ptr(p))?,

                    MARK_NODE => self.delete_token_ref(mark_ptr(p)),

                    DISC_NODE => {
                        self.flush_node_list(pre_break(p))?;
                        self.flush_node_list(post_break(p))?;
                    }

                    ADJUST_NODE => self.flush_node_list(adjust_ptr(p))?,

                    // Section 698
                    STYLE_NODE => {
                        self.free_node(p, STYLE_NODE_SIZE);
                        done = true;
                    },

                    CHOICE_NODE => {
                        self.flush_node_list(display_mlist(p))?;
                        self.flush_node_list(text_mlist(p))?;
                        self.flush_node_list(script_mlist(p))?;
                        self.flush_node_list(script_script_mlist(p))?;
                        self.free_node(p, STYLE_NODE_SIZE);
                        done = true;
                    },

                    ORD_NOAD
                    | OP_NOAD
                    | BIN_NOAD
                    | REL_NOAD
                    | OPEN_NOAD
                    | CLOSE_NOAD
                    | PUNCT_NOAD
                    | INNER_NOAD
                    | RADICAL_NOAD
                    | OVER_NOAD
                    | UNDER_NOAD
                    | VCENTER_NOAD
                    | ACCENT_NOAD => {
                        if math_type(nucleus!(p)) >= SUB_BOX {
                            self.flush_node_list(info(nucleus!(p)))?;
                        }
                        if math_type(supscr!(p)) >= SUB_BOX {
                            self.flush_node_list(info(supscr!(p)))?;
                        }
                        if math_type(subscr!(p)) >= SUB_BOX {
                            self.flush_node_list(info(subscr!(p)))?;
                        }
                        if r#type(p) == RADICAL_NOAD {
                            self.free_node(p, RADICAL_NOAD_SIZE);
                        }
                        else if r#type(p) == ACCENT_NOAD {
                            self.free_node(p, ACCENT_NOAD_SIZE);
                        }
                        else {
                            self.free_node(p,NOAD_SIZE);
                        }
                        done = true;
                    },

                    LEFT_NOAD
                    | RIGHT_NOAD => {
                        self.free_node(p, NOAD_SIZE);
                        done = true;
                    },

                    FRACTION_NOAD => {
                        self.flush_node_list(info(numerator!(p)))?;
                        self.flush_node_list(info(denominator!(p)))?;
                        self.free_node(p, FRACTION_NOAD_SIZE);
                        done = true;
                    }
                    // End section 698

                    _ => return Err(TeXError::Confusion("flushing"))
                }
                if !done {
                    self.free_node(p, SMALL_NODE_SIZE);
                }
            }
            p = q;
        }
        Ok(())
    }
}

// Part 14: Copying boxes

// Section 203
#[macro_export]
macro_rules! add_token_ref {
    ($p:expr) => {
        *token_ref_count_mut($p) += 1;
    };
}

#[macro_export]
macro_rules! add_glue_ref {
    ($p:expr) => {
        *glue_ref_count_mut($p) += 1;
    };
}

impl Global {
    // Section 204
    pub(crate) fn copy_node_list(&mut self, mut p: HalfWord) -> TeXResult<HalfWord> {
        let h = self.get_avail()?;
        let mut q = h;
        while p != NULL {
            // Section 205
            let (r, mut words) = if self.is_char_node(p) {
                (self.get_avail()?, 1)
            }
            else {
                self.sec206_case_statement(p)?
            };

            while words > 0 {
                words -= 1;
                *mem_mut![(r + words) as usize] = mem![(p + words) as usize];
            }
            // End section 205

            *link_mut(q) = r;
            q = r;
            p = link(p);
        }
        *link_mut(q) = NULL;
        q = link(h);
        free_avail!(self, h);
        Ok(q)
    }

    // Section 206
    fn sec206_case_statement(&mut self, p: HalfWord) -> TeXResult<(HalfWord, HalfWord)> {
        let r: HalfWord;
        let mut words: HalfWord = 1;
        match r#type(p) {
            HLIST_NODE
            | VLIST_NODE
            | UNSET_NODE => {
                r = self.get_node(BOX_NODE_SIZE)?;
                *mem_mut![(r + 6) as usize] = mem![(p + 6) as usize];
                *mem_mut![(r + 5) as usize] = mem![(p + 5) as usize];
                *list_ptr_mut(r) = self.copy_node_list(list_ptr(p))?;
                words = 5;
            },

            RULE_NODE => {
                r = self.get_node(RULE_NODE_SIZE)?;
                words = RULE_NODE_SIZE;
            },

            INS_NODE => {
                r = self.get_node(INS_NODE_SIZE)?;
                *mem_mut![(r + 4) as usize] = mem![(p + 4) as usize];
                add_glue_ref!(split_top_ptr(p));
                *ins_ptr_mut(r) = self.copy_node_list(ins_ptr(p))?;
                words = INS_NODE_SIZE - 1;
            },

            WHATSIT_NODE => {
                // Section 1357
                match subtype(p) as Integer {
                    OPEN_NODE => {
                        r = self.get_node(OPEN_NODE_SIZE)?;
                        words = OPEN_NODE_SIZE;
                    },

                    WRITE_NODE
                    | SPECIAL_NODE => {
                        r = self.get_node(WRITE_NODE_SIZE)?;
                        add_token_ref!(write_tokens(p));
                        words = WRITE_NODE_SIZE;
                    },

                    CLOSE_NODE
                    | LANGUAGE_NODE => {
                        r = self.get_node(SMALL_NODE_SIZE)?;
                        words = SMALL_NODE_SIZE;
                    },

                    _ => return Err(TeXError::Confusion("ext2")),
                }
                // End section 1357
            },

            GLUE_NODE => {
                r = self.get_node(SMALL_NODE_SIZE)?;
                add_glue_ref!(glue_ptr(p));
                *glue_ptr_mut(r) = glue_ptr(p);
                *leader_ptr_mut(r) = self.copy_node_list(leader_ptr(p))?;
            },

            KERN_NODE
            | MATH_NODE
            | PENALTY_NODE => {
                r = self.get_node(SMALL_NODE_SIZE)?;
                words = SMALL_NODE_SIZE;
            },

            LIGATURE_NODE => {
                r = self.get_node(SMALL_NODE_SIZE)?;
                *mem_mut![lig_char!(r) as usize] = mem![lig_char!(p) as usize];
                *lig_ptr_mut(r) = self.copy_node_list(lig_ptr(p))?;
            },

            DISC_NODE => {
                r = self.get_node(SMALL_NODE_SIZE)?;
                *pre_break_mut(r) = self.copy_node_list(pre_break(p))?;
                *post_break_mut(r) = self.copy_node_list(post_break(p))?;
            },

            MARK_NODE => {
                r = self.get_node(SMALL_NODE_SIZE)?;
                add_token_ref!(mark_ptr(p));
                words = SMALL_NODE_SIZE;
            },

            ADJUST_NODE => {
                r = self.get_node(SMALL_NODE_SIZE)?;
                *adjust_ptr_mut(r) = self.copy_node_list(adjust_ptr(p))?;
            },
            
            _ => return Err(TeXError::Confusion("copying"))
        }
        Ok((r, words))
    }
}
