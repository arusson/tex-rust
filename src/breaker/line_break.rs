use crate::arithmetic::badness;
use crate::constants::*;
use crate::datastructures::{
    MEM, adj_demerits, broken_penalty, character, club_penalty,
    double_hyphen_demerits, emergency_stretch, ex_hyphen_penalty,
    final_hyphen_demerits, font, glue_ptr, glue_ptr_mut, glue_ref_count_mut,
    hang_after, hang_indent, hsize, hyphen_penalty, info, inter_line_penalty,
    lc_code, leader_ptr, left_skip, lig_ptr, line_penalty, link, link_mut,
    llink, llink_mut, looseness, par_shape_ptr, penalty, penalty_mut,
    post_break, post_break_mut, pre_break, pre_break_mut, pretolerance, r#type,
    replace_count, replace_count_mut, right_skip, rlink, rlink_mut,
    shift_amount_mut, shrink, shrink_order, stretch, stretch_order, subtype,
    subtype_mut, tolerance, type_mut, uc_hyph, width, width_mut
};
use crate::error::{TeXResult, TeXError};
use crate::extensions::{
    what_lang, what_lhm, what_rhm
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, Scaled, SmallNumber,
    add_glue_ref, hpack, lig_char, mem, mem_mut, non_discardable,
    odd, precedes_break, tail_append
};

#[cfg(feature = "stat")]
use crate::datastructures::{info_mut, tracing_paragraphs};

use std::ops::{Index, IndexMut};

// Part 38: Breaking paragraphs into lines
// Part 39: Breaking paragraphs into lines, continued

// Section 823
#[derive(Default)]
pub(crate) struct Array1to6([Scaled; 6]);

impl Index<usize> for Array1to6 {
    type Output = Scaled;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - 1]
    }
}

impl IndexMut<usize> for Array1to6 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - 1]
    }
}

// Section 823
pub(crate) struct HyfArray([QuarterWord; TRIE_OP_SIZE as usize]);

impl Default for HyfArray {
    fn default() -> Self {
        Self([0; TRIE_OP_SIZE as usize])
    }
}

impl Index<usize> for HyfArray {
    type Output = QuarterWord;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index - 1]
    }
}

impl IndexMut<usize> for HyfArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index - 1]
    }
}

#[macro_export]
macro_rules! do_all_six {
    ($f:ident) => {
        $f!(1);
        $f!(2);
        $f!(3);
        $f!(4);
        $f!(5);
        $f!(6);
    }
}

// Section 1362
macro_rules! adv_past {
    ($self:ident, $p:expr) => {
        if subtype($p) == LANGUAGE_NODE as QuarterWord {
            $self.cur_lang = what_lang($p) as u8;
            $self.l_hyf = what_lhm($p) as Integer;
            $self.r_hyf = what_rhm($p) as Integer;
        }
    };
}

impl Global {
    // Section 815
    pub(crate) fn line_break(&mut self, final_widow_penalty: Integer) -> TeXResult<()> {
        self.pack_begin_line = self.mode_line();

        // Section 816
        *link_mut(TEMP_HEAD) = link(self.head());
        if self.is_char_node(self.tail()) || r#type(self.tail()) != GLUE_NODE {
            tail_append!(self, self.new_penalty(INF_PENALTY)?);
        }
        else {
            *type_mut(self.tail()) = PENALTY_NODE;
            self.delete_glue_ref(glue_ptr(self.tail()));
            self.flush_node_list(leader_ptr(self.tail()))?;
            *penalty_mut(self.tail()) = INF_PENALTY;
        }
        *link_mut(self.tail()) = self.new_param_glue(PAR_FILL_SKIP_CODE as SmallNumber)?;
        self.init_cur_lang = (self.prev_graf() % 65536) as u8;
        self.init_l_hyf = self.prev_graf() / 0x40_0000;
        self.init_r_hyf = (self.prev_graf() / 65536) % 64;
        self.pop_nest();

        // Section 827
        self.check_shrinkage(left_skip())?;
        self.check_shrinkage(right_skip())?;
        let mut q = left_skip();
        let mut r = right_skip();
        self.background[1] = width(q) + width(r);
        self.background[2] = 0;
        self.background[3] = 0;
        self.background[4] = 0;
        self.background[5] = 0;
        self.background[(2 + stretch_order(q)) as usize] = stretch(q);
        self.background[(2 + stretch_order(r)) as usize] += stretch(r);
        self.background[6] = shrink(q) + shrink(r);
        // End section 827

        // Section 834
        self.minimum_demerits = AWFUL_BAD;
        self.minimal_demerits[TIGHT_FIT] = AWFUL_BAD;
        self.minimal_demerits[DECENT_FIT] = AWFUL_BAD;
        self.minimal_demerits[LOOSE_FIT] = AWFUL_BAD;
        self.minimal_demerits[VERY_LOOSE_FIT] = AWFUL_BAD;
        // End section 834

        // Section 848
        if par_shape_ptr() == NULL {
            if hang_indent() == 0 {
                self.last_special_line = 0;
                self.second_width = hsize();
                self.second_indent = 0;
            }
            else {
                // Section 849
                self.last_special_line = hang_after().abs();
                if hang_after() < 0 {
                    self.first_width = hsize() - hang_indent().abs();
                    self.first_indent = if hang_indent() >= 0 {
                        hang_indent()
                    }
                    else {
                        0
                    };
                    self.second_width = hsize();
                    self.second_indent = 0;
                }
                else {
                    self.first_width = hsize();
                    self.first_indent = 0;
                    self.second_width = hsize() - hang_indent().abs();
                    self.second_indent = if hang_indent() >= 0 {
                        hang_indent()
                    }
                    else {
                        0
                    };
                }
                // End section 849
            }
        }
        else {
            self.last_special_line = info(par_shape_ptr()) - 1;
            self.second_width = mem![(par_shape_ptr() + 2*(self.last_special_line + 1)) as usize].sc();
            self.second_indent = mem![(par_shape_ptr() + 2*self.last_special_line + 1) as usize].sc();
        }
        self.easy_line = if looseness() == 0 {
            self.last_special_line
        }
        else {
            MAX_HALFWORD
        };
        // End section 848
        // End section 816
        
        // Section 863
        self.threshold = pretolerance();
        if self.threshold >= 0 {
            #[cfg(feature = "stat")]
            if tracing_paragraphs() > 0 {
                self.begin_diagnostic();
                self.print_nl("@firstpass");
            }
            self.second_pass = false;
            self.final_pass = false;
        }
        else {
            self.threshold = tolerance();
            self.second_pass = true;
            self.final_pass = emergency_stretch() <= 0;
            #[cfg(feature = "stat")]
            if tracing_paragraphs() > 0 {
                self.begin_diagnostic();
            }
        }

        'sec863: loop {
            if self.threshold > INF_BAD {
                self.threshold = INF_BAD;
            }
            if self.second_pass {
                // Section 891
                if self.initex_mode && self.trie_not_ready {
                    self.init_trie()?;
                }
                self.cur_lang = self.init_cur_lang;
                self.l_hyf = self.init_l_hyf;
                self.r_hyf = self.init_r_hyf;
                // End section 891
            }

            // Section 864
            macro_rules! store_background {
                ($p:expr) => {
                    self.active_width[$p] = self.background[$p];
                };
            }

            q = self.get_node(ACTIVE_NODE_SIZE)?;
            *type_mut(q) = UNHYPHENATED;
            *fitness_mut(q) = DECENT_FIT as QuarterWord;
            *link_mut(q) = LAST_ACTIVE;
            *break_node_mut(q) = NULL;
            *line_number_mut(q) = self.prev_graf() + 1;
            *total_demerits_mut(q) = 0;
            *link_mut(ACTIVE) = q;
            do_all_six!(store_background);
            self.passive = NULL;
            self.printed_node = TEMP_HEAD;
            self.pass_number = 0;
            self.font_in_short_display = NULL_FONT as QuarterWord;
            // End section 864

            self.cur_p = link(TEMP_HEAD);
            let mut auto_breaking = true;
            let mut prev_p = self.cur_p;
            while self.cur_p != NULL && link(ACTIVE) != LAST_ACTIVE {
                // Section 866
                if self.is_char_node(self.cur_p) {
                   // Section 867
                    prev_p = self.cur_p;
                    loop {
                        let f = font(self.cur_p);
                        self.active_width[1] += self.char_width(f, self.char_info(f, character(self.cur_p)));
                        self.cur_p = link(self.cur_p);
                        if !self.is_char_node(self.cur_p) {
                            break;
                        }
                    }
                   // End section 867
                }
                (auto_breaking, prev_p) = self.sec866_call_try_break_if(auto_breaking, prev_p)?;
                // End section 866
            }

            if self.cur_p == NULL {
                // Section 873
                self.try_break(EJECT_PENALTY, HYPHENATED)?;
                if link(ACTIVE) != LAST_ACTIVE {
                    // Section 874
                    r = link(ACTIVE);
                    self.fewest_demerits = AWFUL_BAD;
                    loop {
                        if r#type(r) != DELTA_NODE && total_demerits(r) < self.fewest_demerits {
                            self.fewest_demerits = total_demerits(r);
                            self.best_bet = r;
                        }
                        r = link(r);
                        if r == LAST_ACTIVE {
                            break;
                        }
                    }
                    self.best_line = line_number(self.best_bet);
                    // End section 874

                    if looseness() == 0 {
                        break 'sec863; // Goto done
                    }

                    // Section 875
                    r = link(ACTIVE);
                    self.actual_looseness = 0;
                    loop {
                        if r#type(r) != DELTA_NODE {
                            self.line_diff = line_number(r) - self.best_line;
                            if (self.line_diff < self.actual_looseness && looseness() <= self.line_diff)
                                || (self.line_diff > self.actual_looseness && looseness() >= self.line_diff)
                            {
                                self.best_bet = r;
                                self.actual_looseness = self.line_diff;
                                self.fewest_demerits = total_demerits(r);
                            }
                            else if self.line_diff == self.actual_looseness && total_demerits(r) < self.fewest_demerits {
                                self.best_bet = r;
                                self.fewest_demerits = total_demerits(r);
                            }
                        }
                        r = link(r);
                        if r == LAST_ACTIVE {
                            break;
                        }
                    }
                    self.best_line = line_number(self.best_bet);
                    // End section 875

                    if self.actual_looseness == looseness()
                        || self.final_pass
                    {
                        break 'sec863; // Goto done
                    }
                }
                // End section 873
            }
            
            // Section 865
            q = link(ACTIVE);
            while q != LAST_ACTIVE {
                self.cur_p = link(q);
                if r#type(q) == DELTA_NODE {
                    self.free_node(q, DELTA_NODE_SIZE);
                }
                else {
                    self.free_node(q, ACTIVE_NODE_SIZE);
                }
                q = self.cur_p;
            }
            q = self.passive;
            while q != NULL {
                self.cur_p = link(q);
                self.free_node(q, PASSIVE_NODE_SIZE);
                q = self.cur_p;
            }
            // End section 865

            if !self.second_pass {
                #[cfg(feature = "stat")]
                if tracing_paragraphs() > 0 {
                    self.print_nl("@secondpass");
                }
                self.threshold = tolerance();
                self.second_pass = true;
                self.final_pass = emergency_stretch() <= 0;
            }
            else {
                #[cfg(feature = "stat")]
                if tracing_paragraphs() > 0 {
                    self.print_nl("@emergencypass");
                }
                self.background[2] += emergency_stretch();
                self.final_pass = true;
            }
        }

        // done:
        #[cfg(feature = "stat")]
        if tracing_paragraphs() > 0 {
            self.end_diagnostic(true);
            self.normalize_selector()?;
        }
        // End section 863
        
        // Section 876
        self.post_line_break(final_widow_penalty)?;
        // End section 876

        // Section 865
        q = link(ACTIVE);
        while q != LAST_ACTIVE {
            self.cur_p = link(q);
            if r#type(q) == DELTA_NODE {
                self.free_node(q, DELTA_NODE_SIZE);
            }
            else {
                self.free_node(q, ACTIVE_NODE_SIZE);
            }
            q = self.cur_p;
        }
        q = self.passive;
        while q != NULL {
            self.cur_p = link(q);
            self.free_node(q, PASSIVE_NODE_SIZE);
            q = self.cur_p;
        }
        // End section 865
        
        self.pack_begin_line = 0;
        Ok(())
    }
}

// Section 819
fn fitness(p: HalfWord) -> QuarterWord {
    subtype(p)
}

fn fitness_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p)
}

fn break_node(p: HalfWord) -> HalfWord {
    rlink(p)
}

fn break_node_mut(p: HalfWord) -> &'static mut HalfWord {
    rlink_mut(p)
}

fn line_number(p: HalfWord) -> HalfWord {
    llink(p)
}

pub(crate) fn line_number_mut(p: HalfWord) -> &'static mut HalfWord {
    llink_mut(p)
}

fn total_demerits(p: HalfWord) -> Integer {
    mem![(p + 2) as usize].int()
}

fn total_demerits_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut![(p + 2) as usize].int_mut()
}

// Section 821
fn cur_break(p: HalfWord) -> HalfWord {
    rlink(p)
}

fn cur_break_mut(p: HalfWord) -> &'static mut HalfWord {
    rlink_mut(p)
}

fn prev_break(p: HalfWord) -> HalfWord {
    llink(p)
}

fn prev_break_mut(p: HalfWord) -> &'static mut HalfWord {
    llink_mut(p)
}

#[cfg(feature = "stat")]
fn serial(p: HalfWord) -> HalfWord {
    info(p)
}

#[cfg(feature = "stat")]
fn serial_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p)
}

impl Global {
    // Section 825
    fn check_shrinkage(&mut self, p: HalfWord) -> TeXResult<()> {
        if shrink_order(p) != NORMAL && shrink(p) != 0 {
            // Section 826
            #[cfg(feature = "stat")]
            if tracing_paragraphs() > 0 {
                self.end_diagnostic(true);
            }
            Err(TeXError::InfiniteGlueShrinkageInParagraph)
        }
        else {
            Ok(())
        }
    }

    // Section 829
    fn try_break(&mut self, mut pi: Integer, break_type: SmallNumber) -> TeXResult<()> {
        macro_rules! copy_to_cur_active {
            ($p:expr) => {
                self.cur_active_width[$p] = self.active_width[$p];
            };
        }
        
        // Section 831
        if pi.abs() >= INF_PENALTY {
            if pi > 0 {
                #[cfg(feature = "stat")]
                self.sec858_update_the_value_of_printed_node();
                return Ok(());
            }
            pi = EJECT_PENALTY;
        }
        // End section 831

        let mut no_break_yet = true;
        let mut prev_r = ACTIVE;
        let mut old_l = 0;
        do_all_six!(copy_to_cur_active);

        let mut r: HalfWord;

        // Section 832
        macro_rules! update_width {
            ($p:expr) => {
                self.cur_active_width[$p] += mem![(r + $p) as usize].sc();
            };
        }
        // End section 832

        // Section 843
        macro_rules! convert_to_break_width {
            ($p:expr) => {
                *mem_mut![(prev_r + $p) as usize].sc_mut() += -self.cur_active_width[$p] + self.break_width[$p];
            };
        }

        macro_rules! store_break_width {
            ($p:expr) => {
                self.active_width[$p] = self.break_width[$p];
            };
        }
        // End section 843

        let mut prev_prev_r = 0;
        let mut line_width: Scaled = 0;
        // continue:
        'sec829: loop {
            r = link(prev_r);

            // Section 832
            if r#type(r) == DELTA_NODE {
                do_all_six!(update_width);
                prev_prev_r = prev_r;
                prev_r = r;
                continue 'sec829; // Goto continue
            }
            // End section 832
            
            // Section 835
            let l = line_number(r);
            if l > old_l {
                if self.minimum_demerits < AWFUL_BAD
                    && (old_l != self.easy_line || r == LAST_ACTIVE)
                {
                    // Section 836
                    if no_break_yet {
                        // Section 837
                        no_break_yet = false;
                        self.sec837_compute_the_values_of_break_width(break_type)?;
                        // End section 837
                    }

                    // Section 843
                    if r#type(prev_r) == DELTA_NODE {
                        do_all_six!(convert_to_break_width);
                    }
                    else if prev_r == ACTIVE {
                        do_all_six!(store_break_width);
                    }
                    else {
                        let q = self.get_node(DELTA_NODE_SIZE)?;

                        macro_rules! new_delta_to_break_width {
                            ($p:expr) => {
                                *mem_mut![(q + $p) as usize].sc_mut() = self.break_width[$p] - self.cur_active_width[$p];
                            };
                        }

                        *link_mut(q) = r;
                        *type_mut(q) = DELTA_NODE;
                        *subtype_mut(q) = 0;
                        do_all_six!(new_delta_to_break_width);
                        *link_mut(prev_r) = q;
                        prev_prev_r = prev_r;
                        prev_r = q;
                    }
                    // End section 843

                    self.minimum_demerits = if adj_demerits().abs() >= AWFUL_BAD - self.minimum_demerits {
                        AWFUL_BAD - 1
                    }
                    else {
                        self.minimum_demerits + adj_demerits().abs()
                    };

                    for fit_class in VERY_LOOSE_FIT..=TIGHT_FIT {
                        if self.minimal_demerits[fit_class] <= self.minimum_demerits {
                            // Section 845
                            let mut q = self.get_node(PASSIVE_NODE_SIZE)?;
                            *link_mut(q) = self.passive;
                            self.passive = q;
                            *cur_break_mut(q) = self.cur_p;
                            #[cfg(feature = "stat")]
                            {
                                self.pass_number += 1;
                                *serial_mut(q) = self.pass_number;
                            }
                            *prev_break_mut(q) = self.best_place[fit_class];
                            q = self.get_node(ACTIVE_NODE_SIZE)?;
                            *break_node_mut(q) = self.passive;
                            *line_number_mut(q) = self.best_pl_line[fit_class] + 1;
                            *fitness_mut(q) = fit_class as QuarterWord;
                            *type_mut(q) = break_type;
                            *total_demerits_mut(q) = self.minimal_demerits[fit_class];
                            *link_mut(q) = r;
                            *link_mut(prev_r) = q;
                            prev_r = q;
                            #[cfg(feature = "stat")]
                            if tracing_paragraphs() > 0 {
                                // Section 846
                                self.print_nl("@@");
                                self.print_int(serial(self.passive));
                                self.print(": line ");
                                self.print_int(line_number(q) - 1);
                                self.print_char(b'.');
                                self.print_int(fit_class as Integer);
                                if break_type == HYPHENATED {
                                    self.print_char(b'-');
                                }
                                self.print(" t=");
                                self.print_int(total_demerits(q));
                                self.print(" -> @@");
                                if prev_break(self.passive) == NULL {
                                    self.print_char(b'0');
                                }
                                else {
                                    self.print_int(serial(prev_break(self.passive)));
                                }
                                // End section 846
                            }
                            // End section 845
                        }
                        self.minimal_demerits[fit_class] = AWFUL_BAD;
                    }
                    self.minimum_demerits = AWFUL_BAD;

                    // Section 844
                    if r != LAST_ACTIVE {
                        let q = self.get_node(DELTA_NODE_SIZE)?;
                        
                        macro_rules! new_delta_from_break_width {
                            ($p:expr) => {
                                *mem_mut![(q + $p) as usize].sc_mut() = self.cur_active_width[$p] - self.break_width[$p];
                            };
                        }

                        *link_mut(q) = r;
                        *type_mut(q) = DELTA_NODE;
                        *subtype_mut(q) = 0;
                        do_all_six!(new_delta_from_break_width);
                        *link_mut(prev_r) = q;
                        prev_prev_r = prev_r;
                        prev_r = q;
                    }
                    // End section 844
                    // End section 836
                }

                if r == LAST_ACTIVE {
                    #[cfg(feature = "stat")]
                    self.sec858_update_the_value_of_printed_node();
                    return Ok(());
                }

                // Section 850
                line_width = if l > self.easy_line {
                    old_l = MAX_HALFWORD - 1;
                    self.second_width
                }
                else {
                    old_l = l;
                    if l > self.last_special_line {
                        self.second_width
                    }
                    else if par_shape_ptr() == NULL {
                        self.first_width
                    }
                    else {
                        mem![(par_shape_ptr() + 2*l) as usize].sc()
                    }
                };
                // End section 850
            }
            // End section 835

            // Section 851
            let mut artificial_demerits = false;
            let shortfall = line_width - self.cur_active_width[1];
            let (b, fit_class) = if shortfall > 0 {
                self.sec852_set_the_value_b(shortfall)
            }
            else {
                // Section 853
                let b = if -shortfall > self.cur_active_width[6] {
                    INF_BAD + 1
                }
                else {
                    badness(-shortfall, self.cur_active_width[6])
                };
                let fit_class = if b > 12 {
                    TIGHT_FIT
                }
                else {
                    DECENT_FIT
                };
                (b, fit_class)
                // End section 853
            };
            
            'block: {
                let node_r_stays_active = if b > INF_BAD || pi == EJECT_PENALTY {
                    // Section 854
                    if self.final_pass
                        && self.minimum_demerits == AWFUL_BAD
                        && link(r) == LAST_ACTIVE
                        && prev_r == ACTIVE
                    {
                        artificial_demerits = true;
                    }
                    else if b > self.threshold {
                        break 'block; // Goto deactivate
                    }
                    false
                    // End section 854
                }
                else {
                    prev_r = r;
                    if b > self.threshold {
                        continue 'sec829; // Goto continue
                    }
                    true
                };

                // Section 855
                let mut d = if artificial_demerits {
                    0
                }
                else {
                    // Section 859
                    let mut d = line_penalty() + b;
                    d = if d.abs() >= 10_000 {
                        100_000_000
                    }
                    else {
                        d*d
                    };

                    if pi != 0 {
                        if pi > 0 {
                            d += pi*pi;
                        }
                        else if pi > EJECT_PENALTY {
                            d -= pi*pi;
                        }
                    }
                    if break_type == HYPHENATED && r#type(r) == HYPHENATED {
                        if self.cur_p != NULL {
                            d += double_hyphen_demerits();
                        }
                        else {
                            d += final_hyphen_demerits();
                        }
                    }
                    if Integer::abs((fit_class as Integer) - (fitness(r) as Integer)) > 1 {
                        d += adj_demerits();
                    }
                    d
                    // End section 859
                };

                #[cfg(feature = "stat")]
                if tracing_paragraphs() > 0 {
                    // Section 856
                    if self.printed_node != self.cur_p {
                        // Section 857
                        self.print_nl("");
                        if self.cur_p == NULL {
                            self.short_display(link(self.printed_node));
                        }
                        else {
                            let save_link = link(self.cur_p);
                            *link_mut(self.cur_p) = NULL;
                            self.print_nl("");
                            self.short_display(link(self.printed_node));
                            *link_mut(self.cur_p) = save_link;
                        }
                        self.printed_node = self.cur_p;
                        // End section 857
                    }
                    self.print_nl("@");
                    if self.cur_p == NULL {
                        self.print_esc("par");
                    }
                    else {
                        match r#type(self.cur_p) {
                            GLUE_NODE => (),
                            PENALTY_NODE => self.print_esc("penalty"),
                            DISC_NODE => self.print_esc("discretionary"),
                            KERN_NODE => self.print_esc("kern"),
                            _ => self.print_esc("math"),
                        }
                    }
                    self.print(" via @@");
                    if break_node(r) == NULL {
                        self.print_char(b'0');
                    }
                    else {
                        self.print_int(serial(break_node(r)));
                    }
                    self.print(" b=");
                    if b > INF_BAD {
                        self.print_char(b'*');
                    }
                    else {
                        self.print_int(b);
                    }
                    self.print(" p=");
                    self.print_int(pi);
                    self.print(" d=");
                    if artificial_demerits {
                        self.print_char(b'*');
                    }
                    else {
                        self.print_int(d);
                    }
                    // End section 856
                }

                d += total_demerits(r);
                if d <= self.minimal_demerits[fit_class] {
                    self.minimal_demerits[fit_class] = d;
                    self.best_place[fit_class] = break_node(r);
                    self.best_pl_line[fit_class] = l;
                    if d < self.minimum_demerits {
                        self.minimum_demerits = d;
                    }
                }
                // End section 855

                if node_r_stays_active {
                    continue 'sec829; // Goto continue
                }
            }

            // deactivate:
            // Section 860
            macro_rules! combine_two_deltas {
                ($p:expr) => {
                    *mem_mut![(prev_r + $p) as usize].sc_mut() += mem![(r + $p) as usize].sc()
                };
            }

            macro_rules! downdate_width {
                ($p:expr) => {
                    self.cur_active_width[$p] -= mem![(prev_r + $p) as usize].sc();
                };
            }

            // Section 861
            macro_rules! update_active {
                ($p:expr) => {
                    self.active_width[$p] += mem![(r + $p) as usize].sc();
                };
            }
            // End section 861

            *link_mut(prev_r) = link(r);
            self.free_node(r, ACTIVE_NODE_SIZE);
            if prev_r == ACTIVE {
                // Section 861
                r = link(ACTIVE);
                if r#type(r) == DELTA_NODE {
                    do_all_six!(update_active);
                    do_all_six!(copy_to_cur_active);
                    *link_mut(ACTIVE) = link(r);
                    self.free_node(r, DELTA_NODE_SIZE);
                }
                // End section 861
            }
            else if r#type(prev_r) == DELTA_NODE {
                r = link(prev_r);
                if r == LAST_ACTIVE {
                    do_all_six!(downdate_width);
                    *link_mut(prev_prev_r) = LAST_ACTIVE;
                    self.free_node(prev_r, DELTA_NODE_SIZE);
                    prev_r = prev_prev_r;
                }
                else if r#type(r) == DELTA_NODE {
                    do_all_six!(update_width);
                    do_all_six!(combine_two_deltas);
                    *link_mut(prev_r) = link(r);
                    self.free_node(r, DELTA_NODE_SIZE);
                }
            }
            // End section 860
            // End section 851
        }
    }

    // Section 837
    fn sec837_compute_the_values_of_break_width(&mut self, break_type: QuarterWord) -> TeXResult<()> {
        macro_rules! set_break_width_to_background {
            ($p:expr) => {
                self.break_width[$p] = self.background[$p];
            };
        }

        do_all_six!(set_break_width_to_background);
        let mut s = self.cur_p;
        if break_type > UNHYPHENATED && self.cur_p != NULL {
            // Section 840
            let mut t = replace_count(self.cur_p);
            let mut v = self.cur_p;
            s = post_break(self.cur_p);
            while t > 0 {
                t -= 1;
                v = link(v);
                // Section 841
                if self.is_char_node(v) {
                    let f = font(v);
                    self.break_width[1] -= self.char_width(f, self.char_info(f, character(v)));
                }
                else {
                    match r#type(v) {
                        LIGATURE_NODE => {
                            let f = font(lig_char!(v));
                            self.break_width[1] -= self.char_width(f, self.char_info(f, character(lig_char!(v))));
                        },

                        HLIST_NODE
                        | VLIST_NODE
                        | RULE_NODE
                        | KERN_NODE => self.break_width[1] -= width(v),

                        _ => return Err(TeXError::Confusion("disc1"))
                    }
                }
                // End section 841
            }

            while s != NULL {
                // Section 842
                if self.is_char_node(s) {
                    let f = font(s);
                    self.break_width[1] += self.char_width(f, self.char_info(f, character(s)));
                }
                else {
                    match r#type(s) {
                        LIGATURE_NODE => {
                            let f = font(lig_char!(s));
                            self.break_width[1] += self.char_width(f, self.char_info(f, character(lig_char!(s))));
                        },

                        HLIST_NODE
                        | VLIST_NODE
                        | RULE_NODE
                        | KERN_NODE => self.break_width[1] += width(s),

                        _ => return Err(TeXError::Confusion("disc2"))
                    }
                }
                // End section 842
                s = link(s);
            }

            self.break_width[1] += self.disc_width;
            if post_break(self.cur_p) == NULL {
                s = link(v);
            }
            // End section 840
        }

        while s != NULL {
            if self.is_char_node(s) {
                break; // Goto done
            }
            match r#type(s) {
                GLUE_NODE => {
                    // Section 838
                    let v = glue_ptr(s);
                    self.break_width[1] -= width(v);
                    self.break_width[2 + stretch_order(v) as usize] -= stretch(v);
                    self.break_width[6] -= shrink(v);
                    // End section 838
                },

                PENALTY_NODE => (), // Do nothing

                MATH_NODE => self.break_width[1] -= width(s),

                KERN_NODE => {
                    if subtype(s) != EXPLICIT {
                        break; // Goto done
                    }
                    self.break_width[1] -= width(s);
                },

                _ => break, // Goto done
            }
            s = link(s);
        }
        // done:
        Ok(())
    }

    // Section 858
    #[cfg(feature = "stat")]
    fn sec858_update_the_value_of_printed_node(&mut self) {
        if self.cur_p == self.printed_node
            && self.cur_p != NULL
            && r#type(self.cur_p) == DISC_NODE
        {
            let mut t = replace_count(self.cur_p);
            while t > 0 {
                t -= 1;
                self.printed_node = link(self.printed_node);
            }
        }
    }
    // End section 858
    
    // Section 852
    fn sec852_set_the_value_b(&self, shortfall: Scaled) -> (HalfWord, usize) {
        if self.cur_active_width[3] != 0
            || self.cur_active_width[4] != 0
            || self.cur_active_width[5] != 0
        {
            (0, DECENT_FIT)
        }
        else if shortfall > 7_230_584 && self.cur_active_width[2] < 1_663_497 {
            (INF_BAD, VERY_LOOSE_FIT)
        }
        else {
            let b = badness(shortfall, self.cur_active_width[2]);
            if b > 12 {
                if b > 99 {
                    (b, VERY_LOOSE_FIT)
                }
                else {
                    (b, LOOSE_FIT)
                }
            }
            else {
                (b, DECENT_FIT)
            }
        }
    }

    // Section 866
    fn sec866_call_try_break_if(&mut self, mut auto_breaking: bool, mut prev_p: HalfWord) -> TeXResult<(bool, HalfWord)> {
        macro_rules! act_width {
            () => {
                self.active_width[1]  
            };
        }

        macro_rules! kern_break {
            () => {
                if !self.is_char_node(link(self.cur_p)) && auto_breaking {
                    if r#type(link(self.cur_p)) == GLUE_NODE {
                        self.try_break(0, UNHYPHENATED)?;
                    }
                }
                act_width!() += width(self.cur_p);
            };
        }

        match r#type(self.cur_p) {
            HLIST_NODE
            | VLIST_NODE
            | RULE_NODE => act_width!() += width(self.cur_p),

            WHATSIT_NODE => adv_past!(self, self.cur_p), // Section 1362

            GLUE_NODE => {
                // Section 868
                if auto_breaking && (
                    self.is_char_node(prev_p)
                        || precedes_break!(prev_p)
                        || (r#type(prev_p) == KERN_NODE && subtype(prev_p) != EXPLICIT)
                    )
                {
                    self.try_break(0, UNHYPHENATED)?;
                }
                self.check_shrinkage(glue_ptr(self.cur_p))?;
                let q = glue_ptr(self.cur_p);
                act_width!() += width(q);
                self.active_width[2 + stretch_order(q) as usize] += stretch(q);
                self.active_width[6] += shrink(q);
                // End section 868
                if self.second_pass && auto_breaking {
                    self.sec894_try_to_hyphenate_the_following_word()?;
                }
            },

            KERN_NODE => {
                if subtype(self.cur_p) == EXPLICIT {
                    kern_break!();
                }
                else {
                    act_width!() += width(self.cur_p);
                }
            },

            LIGATURE_NODE => {
                let f = font(lig_char!(self.cur_p));
                let tmp = self.char_info(f, character(lig_char!(self.cur_p)));
                act_width!() += self.char_width(f, tmp);
            },

            DISC_NODE => {
                // Section 869
                let mut s = pre_break(self.cur_p);
                self.disc_width = 0;
                if s == NULL {
                    self.try_break(ex_hyphen_penalty(), HYPHENATED)?;
                }
                else {
                    loop {
                        // Section 870
                        if self.is_char_node(s) {
                            let f = font(s);
                            self.disc_width += self.char_width(f, self.char_info(f, character(s)));
                        }
                        else {
                            match r#type(s) {
                                LIGATURE_NODE => {
                                    let f = font(lig_char!(s));
                                    self.disc_width += self.char_width(f, self.char_info(f, character(lig_char!(s))));
                                },

                                HLIST_NODE
                                | VLIST_NODE
                                | RULE_NODE
                                | KERN_NODE => self.disc_width += width(s),
                                
                                _ => return Err(TeXError::Confusion("disc3"))
                            }
                        }
                        // End section 870
                        s = link(s);
                        if s == NULL {
                            break;
                        }
                    }
                    act_width!() += self.disc_width;
                    self.try_break(hyphen_penalty(), HYPHENATED)?;
                    act_width!() -= self.disc_width;
                }
                let mut r = replace_count(self.cur_p);
                s = link(self.cur_p);
                while r > 0 {
                    // Section 871
                    if self.is_char_node(s) {
                        let f = font(s);
                        act_width!() += self.char_width(f, self.char_info(f, character(s)));
                    }
                    else {
                        match r#type(s) {
                            LIGATURE_NODE => {
                                let f = font(lig_char!(s));
                                act_width!() += self.char_width(f, self.char_info(f, character(lig_char!(s))));
                            },

                            HLIST_NODE
                            | VLIST_NODE
                            | RULE_NODE
                            | KERN_NODE => act_width!() += width(s),
                            
                            _ => return Err(TeXError::Confusion("disc4"))
                        }
                    }
                    // End section 871
                    r -= 1;
                    s = link(s);
                }
                prev_p = self.cur_p;
                self.cur_p = s;
                return Ok((auto_breaking, prev_p)); // Goto done5
                // End section 869
            },

            MATH_NODE => {
                auto_breaking = subtype(self.cur_p) == AFTER;
                kern_break!();
            },

            PENALTY_NODE => self.try_break(penalty(self.cur_p), UNHYPHENATED)?,
            
            MARK_NODE
            | INS_NODE
            | ADJUST_NODE => (), // Do nothing
            
            _ => return Err(TeXError::Confusion("paragraph")),
        }

        prev_p = self.cur_p;
        self.cur_p = link(self.cur_p);
        // done5:
        Ok((auto_breaking, prev_p))
    }
}

// Section 877
fn next_break(p: HalfWord) -> HalfWord {
    prev_break(p)
}

fn next_break_mut(p: HalfWord) -> &'static mut HalfWord {
    prev_break_mut(p)
}

impl Global {
    // Section 877
    fn post_line_break(&mut self, final_widow_penalty: Integer) -> TeXResult<()> {
        // Section 878
        let mut q = break_node(self.best_bet);
        self.cur_p = NULL;
        loop {
            let r = q;
            q = prev_break(q);
            *next_break_mut(r) = self.cur_p;
            self.cur_p = r;
            if q == NULL {
                break;
            }
        }
        // End section 878
        
        let mut cur_line = self.prev_graf() + 1;

        'sec877: loop {
            let post_disc_break = self.sec880_justify_the_line_ending(cur_line, final_widow_penalty)?;
            cur_line += 1;
            self.cur_p = next_break(self.cur_p);
            if self.cur_p != NULL && !post_disc_break {
                // Section 879
                let mut r = TEMP_HEAD;
                'sec879: loop {
                    q = link(r);
                    if q == cur_break(self.cur_p)
                        || self.is_char_node(q)
                        || non_discardable!(q)
                        || (r#type(q) == KERN_NODE && subtype(q) != EXPLICIT)
                    {
                        break 'sec879; // Goto done1
                    }
                    r = q;
                }
                // done1:
                if r != TEMP_HEAD {
                    *link_mut(r) = NULL;
                    self.flush_node_list(link(TEMP_HEAD))?;
                    *link_mut(TEMP_HEAD) = q;
                }
                // End section 879
            }
            if self.cur_p == NULL {
                break 'sec877;
            }
        }
        if cur_line != self.best_line || link(TEMP_HEAD) != NULL {
            return Err(TeXError::Confusion("line breaking"));
        }
        *self.prev_graf_mut() = self.best_line - 1;
        Ok(())
    }

    // Section 880
    fn sec880_justify_the_line_ending(&mut self, cur_line: HalfWord, final_widow_penalty: Integer) -> TeXResult<bool> {
        // Section 881
        let mut q = cur_break(self.cur_p);
        let mut disc_break = false;
        let mut post_disc_break = false;
        'sec881: {
            if q != NULL {
                if r#type(q) == GLUE_NODE {
                    self.delete_glue_ref(glue_ptr(q));
                    *glue_ptr_mut(q) = right_skip();
                    *subtype_mut(q) = (RIGHT_SKIP_CODE + 1) as QuarterWord;
                    add_glue_ref!(right_skip());
                    break 'sec881; // Goto done
                }
                if r#type(q) == DISC_NODE {
                    // Section 882
                    let mut t = replace_count(q);

                    // Section 883
                    let mut r = if t == 0 {
                        link(q)
                    }
                    else {
                        let mut r = q;
                        while t > 1 {
                            r = link(r);
                            t -= 1;
                        }
                        let s = link(r);
                        r = link(s);
                        *link_mut(s) = NULL;
                        self.flush_node_list(link(q))?;
                        *replace_count_mut(q) = 0;
                        r
                    };
                    // End section 883

                    if post_break(q) != NULL {
                        // Section 884
                        let mut s = post_break(q);
                        while link(s) != NULL {
                            s = link(s);
                        }
                        *link_mut(s) = r;
                        r = post_break(q);
                        *post_break_mut(q) = NULL;
                        post_disc_break = true;
                        // End section 884
                    }
                    if pre_break(q) != NULL {
                        // Section 885
                        let mut s = pre_break(q);
                        *link_mut(q) = s;
                        while link(s) != NULL {
                            s = link(s);
                        }
                        *pre_break_mut(q) = NULL;
                        q = s;
                        // End section 885
                    }
                    *link_mut(q) = r;
                    disc_break = true;
                    // End section 882
                }
                else if r#type(q) == MATH_NODE || r#type(q) == KERN_NODE {
                    *width_mut(q) = 0
                }
            }
            else {
                q = TEMP_HEAD;
                while link(q) != NULL {
                    q = link(q);
                }
            }

            // Section 886
            let r = self.new_param_glue(RIGHT_SKIP_CODE as SmallNumber)?;
            *link_mut(r) = link(q);
            *link_mut(q) = r;
            q = r;
            // End section 886
        }
        // done:
        // End section 881

        // Section 887
        let mut r = link(q);
        *link_mut(q) = NULL;
        q = link(TEMP_HEAD);
        *link_mut(TEMP_HEAD) = r;
        if left_skip() != ZERO_GLUE {
            r = self.new_param_glue(LEFT_SKIP_CODE as SmallNumber)?;
            *link_mut(r) = q;
            q = r;
        }
        // End section 887
        
        // Section 889
        let (cur_width, cur_indent) = if cur_line > self.last_special_line {
            (self.second_width, self.second_indent)
        }
        else if par_shape_ptr() == NULL {
            (self.first_width, self.first_indent)
        }
        else {
            (
                mem![(par_shape_ptr() + 2*cur_line) as usize].sc(),
                mem![(par_shape_ptr() + 2*cur_line - 1) as usize].sc()
            )
        };

        self.adjust_tail = ADJUST_HEAD;
        self.just_box = hpack!(self, q, cur_width, EXACTLY)?;
        *shift_amount_mut(self.just_box) = cur_indent;
        // End section 889

        // Section 888
        self.append_to_vlist(self.just_box)?;
        if ADJUST_HEAD != self.adjust_tail {
            *link_mut(self.tail()) = link(ADJUST_HEAD);
            *self.tail_mut() = self.adjust_tail;
        }
        self.adjust_tail = NULL;
        // End section 888

        // Section 890
        if cur_line + 1 != self.best_line {
            let mut pen = inter_line_penalty();
            if cur_line == self.prev_graf() + 1 {
                pen += club_penalty();
            }
            if cur_line + 2 == self.best_line {
                pen += final_widow_penalty;
            }
            if disc_break {
                pen += broken_penalty();
            }
            if pen != 0 {
                r = self.new_penalty(pen)?;
                *link_mut(self.tail()) = r;
                *self.tail_mut() = r;
            }
        }
        // End section 890
        Ok(post_disc_break)
    }

    // Section 894
    fn sec894_try_to_hyphenate_the_following_word(&mut self) -> TeXResult<()> {
        let mut prev_s = self.cur_p;
        let mut s = link(prev_s);
        if s != NULL {
            // Section 896
            let mut c: QuarterWord;
            loop {
                if self.is_char_node(s) {
                    c = character(s);
                    self.hf = font(s);
                }
                else if r#type(s) == LIGATURE_NODE {
                    if lig_ptr(s) == NULL {
                        // continue:
                        prev_s = s;
                        s = link(prev_s);
                        continue;
                    }
                    let q = lig_ptr(s);
                    c = character(q);
                    self.hf = font(q);
                }
                else if r#type(s) == KERN_NODE && subtype(s) == NORMAL {
                    // continue:
                    prev_s = s;
                    s = link(prev_s);
                    continue;
                }
                else if r#type(s) == WHATSIT_NODE {
                    // Section 1363
                    adv_past!(self, s);
                    // End section 1363
                    // continue:
                    prev_s = s;
                    s = link(prev_s);
                    continue;
                }
                else {
                    return Ok(()); // Goto done1
                }
                if lc_code(c as HalfWord) != 0 {
                    if lc_code(c as HalfWord) == (c as HalfWord)
                        || uc_hyph() > 0
                    {
                        break; // Goto done2
                    }
                    return Ok(()); // Goto done1
                }
                // continue:
                prev_s = s;
                s = link(prev_s);
            }

            // done2:
            self.hyf_char = self.hyphen_char[self.hf as usize];
            if self.hyf_char < 0 || self.hyf_char > 255 {
                return Ok(()); // Goto done1
            }
            self.ha = prev_s;
            // End section 896

            if self.l_hyf + self.r_hyf > 63 {
                return Ok(()); // Goto done1
            }

            // Section 897
            self.hn = 0;
            'sec897: loop {
                if self.is_char_node(s) {
                    if font(s) != self.hf {
                        break 'sec897; // Goto done3
                    }
                    self.hyf_bchar = character(s) as HalfWord;
                    c = self.hyf_bchar as QuarterWord;
                    if lc_code(c as HalfWord) == 0 || self.hn == 63 {
                        break 'sec897; // Goto done3
                    }
                    self.hb = s;
                    self.hn += 1;
                    self.hu[self.hn as usize] = c;
                    self.hc[self.hn as usize] = lc_code(c as HalfWord) as QuarterWord;
                    self.hyf_bchar = NON_CHAR;
                }
                else if r#type(s) == LIGATURE_NODE {
                    // Section 898
                    if font(lig_char!(s)) != self.hf {
                        break 'sec897; // Goto done3
                    }
                    let mut j = self.hn;
                    let mut q = lig_ptr(s);
                    if q > NULL {
                        self.hyf_bchar = character(q) as HalfWord;
                    }
                    while q > NULL {
                        c = character(q);
                        if lc_code(c as HalfWord) == 0 || j == 63 {
                            break 'sec897; // Goto done3
                        }
                        j += 1;
                        self.hu[j as usize] = c;
                        self.hc[j as usize] = lc_code(c as HalfWord) as QuarterWord;
                        q = link(q);
                    }
                    self.hb = s;
                    self.hn = j;
                    self.hyf_bchar = if odd!(subtype(s)) {
                        self.font_bchar[self.hf as usize] as HalfWord
                    }
                    else {
                        NON_CHAR
                    };
                    // End section 898
                }
                else if r#type(s) == KERN_NODE && subtype(s) == NORMAL {
                    self.hb = s;
                    self.hyf_bchar = self.font_bchar[self.hf as usize] as HalfWord;
                }
                else {
                    break 'sec897; // Goto done3
                }
                s = link(s);
            }
            // done3:
            // End section 897

            // Section 899
            if (self.hn as Integer) < self.l_hyf + self.r_hyf {
                return Ok(()); // Goto done1
            }
            loop {
                if !self.is_char_node(s) {
                    match r#type(s) {
                        LIGATURE_NODE => (), // Do nothing

                        KERN_NODE => {
                            if subtype(s) != NORMAL {
                                break; // Goto done4
                            }
                        },

                        WHATSIT_NODE
                        | GLUE_NODE
                        | PENALTY_NODE
                        | INS_NODE
                        | ADJUST_NODE
                        | MARK_NODE => break, // Goto done4

                        _ => return Ok(()) // Goto done1
                    }
                }
                s = link(s);
            }
            // done4:
            // End section 899
            self.hyphenate()?;
        }
        // done1:
        Ok(())
    }
}
