use crate::arithmetic::{badness, x_over_n};
use crate::constants::*;
use crate::datastructures::{
    r#box, box_mut, count, depth, dimen, float_cost, geq_word_define, glue_ptr,
    glue_ref_count_mut, height, height_mut, holding_inserts, info, info_mut,
    ins_ptr, ins_ptr_mut, link, link_mut, list_ptr, mark_ptr, max_dead_cycles,
    max_depth, output_routine, penalty, penalty_mut, shrink, shrink_order,
    skip, split_top_ptr, split_top_skip, split_top_skip_mut, stretch,
    stretch_order, subtype, subtype_mut, token_ref_count_mut, r#type, type_mut,
    vbadness, vbadness_mut, vfuzz, vfuzz_mut, vsize, width, width_mut
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, SmallNumber,
    add_glue_ref, add_token_ref, precedes_break, vpack
};

#[cfg(feature = "stat")]
use crate::datastructures::tracing_pages;

// Part 45: The page builder

// Section 981
fn broken_ptr(p: HalfWord) -> HalfWord {
    link(p + 1)
}

fn broken_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 1)
}

pub(crate) fn broken_ins(p: HalfWord) -> HalfWord {
    info(p + 1)
}

fn broken_ins_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 1)
}

fn last_ins_ptr(p: HalfWord) -> HalfWord {
    link(p + 2)
}

fn last_ins_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 2)
}

fn best_ins_ptr(p: HalfWord) -> HalfWord {
    info(p + 2)
}

fn best_ins_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 2)
}

// Section 982
#[macro_export]
macro_rules! page_goal {
    ($s:ident) => {
        $s.page_so_far[0]
    };
}

#[macro_export]
macro_rules! page_total {
    ($s:ident) => {
        $s.page_so_far[1]
    };
}

#[macro_export]
macro_rules! page_shrink {
    ($s:ident) => {
        $s.page_so_far[6]
    };
}

#[macro_export]
macro_rules! page_depth {
    ($s:ident) => {
        $s.page_so_far[7]
    };
}

impl Global {
    // Section 987
    fn freeze_page_specs(&mut self, s: SmallNumber) {
        self.page_contents = s;
        page_goal!(self) = vsize();
        self.page_max_depth = max_depth();
        page_depth!(self) = 0;
        self.page_so_far[1..=6].fill(0);
        self.least_page_cost = AWFUL_BAD;
        #[cfg(feature = "stat")]
        if tracing_pages() > 0 {
            self.begin_diagnostic();
            self.print_nl("%% goal height=");
            self.print_scaled(page_goal!(self));
            self.print(", max depth=");
            self.print_scaled(self.page_max_depth);
            self.end_diagnostic(false);
        }
    }
}

// Section 993
fn ensure_vbox(n: u8) -> TeXResult<()> {
    let p = r#box(n as HalfWord);
    if p != NULL && r#type(p) == HLIST_NODE {
        Err(TeXError::InsertionCanOnlyBeAddedToVbox(n as HalfWord))
    }
    else {
        Ok(())
    }
}

// Section 995
#[macro_export]
macro_rules! contrib_tail {
    ($s:ident) => {
        $s.nest[0].tail_field
    };
}

enum Goto {
    UpdateHeight,
    Contribute,
    Done1,
    Done
}

impl Global {
    // Section 994
    pub(crate) fn build_page(&mut self) -> TeXResult<()> {
        if link(CONTRIB_HEAD) == NULL || self.output_active {
            return Ok(())
        }

        // continue:
        'sec994: loop {
            let p = link(CONTRIB_HEAD);

            // Section 996
            if self.last_glue != MAX_HALFWORD {
                self.delete_glue_ref(self.last_glue);
            }
            self.last_penalty = 0;
            self.last_kern = 0;
            if r#type(p) == GLUE_NODE {
                self.last_glue = glue_ptr(p);
                add_glue_ref!(self.last_glue);
            }
            else {
                self.last_glue = MAX_HALFWORD;
                if r#type(p) == PENALTY_NODE {
                    self.last_penalty = penalty(p);
                }
                else if r#type(p) == KERN_NODE {
                    self.last_kern = width(p);
                }
            }
            // End section 996
            
            // Section 997
            let mut goto = 'block: {
                // Section 1000
                let pi = match r#type(p) {
                    HLIST_NODE
                    | VLIST_NODE
                    | RULE_NODE => {
                        if self.page_contents < BOX_THERE {
                            // Section 1001
                            if self.page_contents == EMPTY as SmallNumber {
                                self.freeze_page_specs(BOX_THERE);
                            }
                            else {
                                self.page_contents = BOX_THERE;
                            }
                            let q = self.new_skip_param(TOP_SKIP_CODE as SmallNumber)?;
                            *width_mut(self.temp_ptr) = if width(self.temp_ptr) > height(p) {
                                width(self.temp_ptr) - height(p)
                            }
                            else {
                                0
                            };

                            *link_mut(q) = p;
                            *link_mut(CONTRIB_HEAD) = q;
                            continue 'sec994; // Goto continue
                            // End section 1001
                        }
                        
                        // Section 1002
                        page_total!(self) += page_depth!(self) + height(p);
                        page_depth!(self) = depth(p);
                        break 'block Goto::Contribute;
                        // End section 1002
                    },

                    WHATSIT_NODE => break 'block Goto::Contribute, // Section 1364

                    GLUE_NODE => {
                        if self.page_contents < BOX_THERE {
                            break 'block Goto::Done1;
                        }
                        if precedes_break!(self.page_tail) {
                            0
                        }
                        else {
                            break 'block Goto::UpdateHeight;
                        }
                    },

                    KERN_NODE => {
                        if self.page_contents < BOX_THERE {
                            break 'block Goto::Done1;
                        }
                        if link(p) == NULL {
                            return Ok(());
                        }
                        if r#type(link(p)) == GLUE_NODE {
                            0
                        }
                        else {
                            break 'block Goto::UpdateHeight;
                        }
                    },

                    PENALTY_NODE => {
                        if self.page_contents < BOX_THERE {
                            break 'block Goto::Done1;
                        }
                        penalty(p)
                    },

                    MARK_NODE => break 'block Goto::Contribute,

                    INS_NODE => {
                        // Section 1008
                        if self.page_contents == EMPTY as SmallNumber {
                            self.freeze_page_specs(INSERTS_ONLY);
                        }
                        let n = subtype(p);
                        let mut r = PAGE_INS_HEAD;
                        while n >= subtype(link(r)) {
                            r = link(r);
                        }
                        if subtype(r) != n {
                            // Section 1009
                            let mut q = self.get_node(PAGE_INS_NODE_SIZE)?;
                            *link_mut(q) = link(r);
                            *link_mut(r) = q;
                            r = q;
                            *subtype_mut(r) = n;
                            *type_mut(r) = INSERTING;
                            ensure_vbox(n as u8)?;

                            *height_mut(r) = if r#box(n as HalfWord) == NULL {
                                0
                            }
                            else {
                                height(r#box(n as HalfWord)) + depth(r#box(n as HalfWord))
                            };

                            *best_ins_ptr_mut(r) = NULL;
                            q = skip(n as HalfWord);

                            let h = if count(n as HalfWord) == 1000 {
                                height(r)
                            }
                            else {
                                x_over_n(height(r), 1000)?.0*count(n as HalfWord)
                            };

                            page_goal!(self) -= h + width(q);
                            self.page_so_far[2 + stretch_order(q) as usize] += stretch(q);
                            page_shrink!(self) += shrink(q);
                            if shrink_order(q) != NORMAL && shrink(q) != 0 {
                                return Err(TeXError::InfiniteGlueShrinkageInsertedFrom(n as Integer));
                            }
                            // End section 1009
                        }

                        if r#type(r) == SPLIT_UP {
                            self.insert_penalties += float_cost(p);
                        }
                        else {
                            *last_ins_ptr_mut(r) = p;
                            let delta = page_goal!(self) - page_total!(self) - page_depth!(self) + page_shrink!(self);

                            let h = if count(n as HalfWord) == 1000 {
                                height(p)
                            }
                            else {
                                x_over_n(height(p), 1000)?.0*count(n as HalfWord)
                            };

                            if (h <= 0 || h <= delta) && (height(p) + height(r) <= dimen(n as HalfWord)) {
                                page_goal!(self) -= h;
                                *height_mut(r) += height(p);
                            }
                            else {
                                // Section 1010
                                let mut w = if count(n as HalfWord) <= 0 {
                                    MAX_DIMEN
                                }
                                else {
                                    let mut w = page_goal!(self) - page_total!(self) - page_depth!(self);
                                    if count(n as HalfWord) != 1000 {
                                        w = x_over_n(w, count(n as HalfWord))?.0 * 1000;
                                    }
                                    w
                                };

                                if w > dimen(n as HalfWord) - height(r) {
                                    w = dimen(n as HalfWord) - height(r);
                                }
                                let q = self.vert_break(ins_ptr(p), w, depth(p))?;
                                *height_mut(r) += self.best_height_plus_depth;

                                #[cfg(feature = "stat")]
                                if tracing_pages() > 0 {
                                    // Section 1011
                                    self.begin_diagnostic();
                                    self.print_nl("% split");
                                    self.print_int(n as Integer);
                                    self.print(" to ");
                                    self.print_scaled(w);
                                    self.print_char(b',');
                                    self.print_scaled(self.best_height_plus_depth);
                                    self.print(" p=");
                                    if q == NULL {
                                        self.print_int(EJECT_PENALTY);
                                    }
                                    else if r#type(q) == PENALTY_NODE {
                                        self.print_int(penalty(q));
                                    }
                                    else {
                                        self.print_char(b'0');
                                    }
                                    self.end_diagnostic(false);
                                    // End section 1011
                                }

                                if count(n as HalfWord) != 1000 {
                                    self.best_height_plus_depth = x_over_n(self.best_height_plus_depth, 1000)?.0 * count(n as HalfWord);
                                }
                                page_goal!(self) -= self.best_height_plus_depth;
                                *type_mut(r) = SPLIT_UP;
                                *broken_ptr_mut(r) = q;
                                *broken_ins_mut(r) = p;
                                if q == NULL {
                                    self.insert_penalties += EJECT_PENALTY;
                                }
                                else if r#type(q) == PENALTY_NODE {
                                    self.insert_penalties += penalty(q);
                                }
                                // End section 1010
                            }
                        }
                        break 'block Goto::Contribute;
                        // End section 1008
                    },

                    _ => return Err(TeXError::Confusion("page")),
                };
                // End section 1000

                // Section 1005
                if pi < INF_PENALTY {
                    // Section 1007
                    let b = if page_total!(self) < page_goal!(self) {
                        if self.page_so_far[3] != 0 || self.page_so_far[4] != 0 || self.page_so_far[5] != 0 {
                            0
                        }
                        else {
                            badness(page_goal!(self) - page_total!(self), self.page_so_far[2])
                        }
                    }
                    else if page_total!(self) - page_goal!(self) > page_shrink!(self) {
                        AWFUL_BAD
                    }
                    else {
                        badness(page_total!(self) - page_goal!(self), page_shrink!(self))
                    };
                    // End section 1007

                    let mut c = if b < AWFUL_BAD {
                        if pi <= EJECT_PENALTY {
                            pi
                        }
                        else if b < INF_BAD {
                            b + pi + self.insert_penalties
                        }
                        else {
                            DEPLORABLE
                        }
                    }
                    else {
                        b
                    };

                    if self.insert_penalties >= 10_000 {
                        c = AWFUL_BAD;
                    }

                    #[cfg(feature = "stat")]
                    if tracing_pages() > 0 {
                        // Section 1006
                        self.begin_diagnostic();
                        self.print_nl("%");
                        self.print(" t=");
                        self.print_totals();
                        self.print(" g=");
                        self.print_scaled(page_goal!(self));
                        self.print(" b=");
                        if b == AWFUL_BAD {
                            self.print_char(b'*');
                        }
                        else {
                            self.print_int(b);
                        }
                        self.print(" p=");
                        self.print_int(pi);
                        self.print(" c=");
                        if c == AWFUL_BAD {
                            self.print_char(b'*');
                        }
                        else {
                            self.print_int(c);
                        }
                        if c <= self.least_page_cost {
                            self.print_char(b'#');
                        }
                        self.end_diagnostic(false);
                        // End section 1006
                    }

                    if c <= self.least_page_cost {
                        self.best_page_break = p;
                        self.best_size = page_goal!(self);
                        self.least_page_cost = c;
                        let mut r = link(PAGE_INS_HEAD);
                        while r != PAGE_INS_HEAD {
                            *best_ins_ptr_mut(r) = last_ins_ptr(r);
                            r = link(r);
                        }
                    }
                    if c == AWFUL_BAD || pi <= EJECT_PENALTY {
                        self.fire_up(p)?;
                        if self.output_active {
                            return Ok(())
                        }
                        break 'block Goto::Done;
                    }
                }
                // End section 1005

                if r#type(p) < GLUE_NODE || r#type(p) > KERN_NODE {
                    break 'block Goto::Contribute;
                }
                Goto::UpdateHeight
            };
        
            if let Goto::UpdateHeight = goto {
                // update heights:
                // Section 1004
                let q = match r#type(p) {
                    KERN_NODE => p,

                    _ => {
                        let q = glue_ptr(p);
                        self.page_so_far[2 + stretch_order(q) as usize] += stretch(q);
                        page_shrink!(self) += shrink(q);
                        if shrink_order(q) != NORMAL && shrink(q) != 0 {
                            return Err(TeXError::InfiniteGlueShrinkageOnCurrentPage);
                        }
                        q
                    }
                };

                page_total!(self) += page_depth!(self) + width(q);
                page_depth!(self) = 0;
                // End section 1004
            }

            match goto {
                Goto::Contribute
                | Goto::UpdateHeight => {
                    // contribute:
                    // Section 1003
                    if page_depth!(self) > self.page_max_depth {
                        page_total!(self) += page_depth!(self) - self.page_max_depth;
                        page_depth!(self) = self.page_max_depth;
                    }
                    // End section 1003

                    // Section 998
                    *link_mut(self.page_tail) = p;
                    self.page_tail = p;
                    *link_mut(CONTRIB_HEAD) = link(p);
                    *link_mut(p) = NULL;
                    goto = Goto::Done;
                    // End section 998
                },

                _ => (),
            }

            match goto {
                Goto::Done => (),

                _=> {
                    // done1:
                    // Section 999
                    *link_mut(CONTRIB_HEAD) = link(p);
                    *link_mut(p) = NULL;
                    self.flush_node_list(p)?;
                    // End section 999
                },
            }

            // done:
            // End section 997

            if link(CONTRIB_HEAD) == NULL {
                break 'sec994;
            }
        }

        // Section 995
        if self.nest_ptr == 0 {
            *self.tail_mut() = CONTRIB_HEAD;
        }
        else {
            contrib_tail!(self) = CONTRIB_HEAD;
        }
        // End section 995
        Ok(())
    }

    // Section 1012
    fn fire_up(&mut self, c: HalfWord) -> TeXResult<()> {
        // Section 1013
        if r#type(self.best_page_break) == PENALTY_NODE {
            geq_word_define(INT_BASE + OUTPUT_PENALTY_CODE, penalty(self.best_page_break));
            *penalty_mut(self.best_page_break) = INF_PENALTY;
        }
        else {
            geq_word_define(INT_BASE + OUTPUT_PENALTY_CODE, INF_PENALTY);
        }
        // End section 1013

        if self.bot_mark() != NULL {
            if self.top_mark() != NULL {
                self.delete_token_ref(self.top_mark());
            }
            *self.top_mark_mut() = self.bot_mark();
            add_token_ref!(self.top_mark());
            self.delete_token_ref(self.first_mark());
            *self.first_mark_mut() = NULL;
        }

        // Section 1014
        if c == self.best_page_break {
            self.best_page_break = NULL;
        }

        // Section 1015
        if r#box(255) != NULL {
            return Err(TeXError::Box255IsNotVoid);
        }
        // End section 1015

        self.insert_penalties = 0;
        let save_split_top_skip = split_top_skip();
        if holding_inserts() <= 0 {
            // Section 1018
            let mut r = link(PAGE_INS_HEAD);
            while r != PAGE_INS_HEAD {
                if best_ins_ptr(r) != NULL {
                    let n = subtype(r);
                    ensure_vbox(n as u8)?;
                    if r#box(n as HalfWord) == NULL {
                        *box_mut(n as HalfWord) = self.new_null_box()?;
                    }
                    let mut p = r#box(n as HalfWord) + LIST_OFFSET;
                    while link(p) != NULL {
                        p = link(p);
                    }
                    *last_ins_ptr_mut(r) = p;
                }
                r = link(r);
            }
            // End section 1018
        }

        let mut q = HOLD_HEAD;
        *link_mut(q) = NULL;
        let mut prev_p = PAGE_HEAD;
        let mut p = link(prev_p);
        while p != self.best_page_break {
            if r#type(p) == INS_NODE {
                if holding_inserts() <= 0 {
                    // Section 1020
                    let mut r = link(PAGE_INS_HEAD);
                    while subtype(r) != subtype(p) {
                        r = link(r);
                    }
                    let wait = match best_ins_ptr(r) {
                        NULL => true,

                        _ => {
                            let mut wait = false;
                            let mut s = last_ins_ptr(r);
                            *link_mut(s) = ins_ptr(p);
                            if best_ins_ptr(r) == p {
                                // Section 1021
                                if r#type(r) == SPLIT_UP && broken_ins(r) == p && broken_ptr(r) != NULL {
                                    while link(s) != broken_ptr(r) {
                                        s = link(s);
                                    }
                                    *link_mut(s) = NULL;
                                    *split_top_skip_mut() = split_top_ptr(p);
                                    *ins_ptr_mut(p) = self.prune_page_top(broken_ptr(r))?;
                                    if ins_ptr(p) != NULL {
                                        self.temp_ptr = vpack!(self, ins_ptr(p), NATURAL)?;
                                        *height_mut(p) = height(self.temp_ptr) + depth(self.temp_ptr);
                                        self.free_node(self.temp_ptr, BOX_NODE_SIZE);
                                        wait = true;
                                    }
                                }
                                *best_ins_ptr_mut(r) = NULL;
                                let n = subtype(r);
                                self.temp_ptr = list_ptr(r#box(n as HalfWord));
                                self.free_node(r#box(n as HalfWord), BOX_NODE_SIZE);
                                *box_mut(n as HalfWord) = vpack!(self, self.temp_ptr, NATURAL)?;
                                // End section 1021
                            }
                            else {
                                while link(s) != NULL {
                                    s = link(s);
                                }
                                *last_ins_ptr_mut(r) = s;
                            }
                            wait
                        }
                    };

                    // Section 1022
                    *link_mut(prev_p) = link(p);
                    *link_mut(p) = NULL;
                    if wait {
                        *link_mut(q) = p;
                        q = p;
                        self.insert_penalties += 1;
                    }
                    else {
                        self.delete_glue_ref(split_top_ptr(p));
                        self.free_node(p, INS_NODE_SIZE);
                    }
                    p = prev_p;
                    // End section 1022
                    // End section 1020
                }
            }
            else if r#type(p) == MARK_NODE {
                // Section 1016
                if self.first_mark() == NULL {
                    *self.first_mark_mut() = mark_ptr(p);
                    add_token_ref!(self.first_mark());
                }
                if self.bot_mark() != NULL {
                    self.delete_token_ref(self.bot_mark());
                }
                *self.bot_mark_mut() = mark_ptr(p);
                add_token_ref!(self.bot_mark());
                // End section 1016
            }
            prev_p = p;
            p = link(prev_p);
        }
        *split_top_skip_mut() = save_split_top_skip;

        // Section 1017
        if p != NULL {
            if link(CONTRIB_HEAD) == NULL {
                if self.nest_ptr == 0 {
                    *self.tail_mut() = self.page_tail;
                }
                else {
                    contrib_tail!(self) = self.page_tail;
                }
            }
            *link_mut(self.page_tail) = link(CONTRIB_HEAD);
            *link_mut(CONTRIB_HEAD) = p;
            *link_mut(prev_p) = NULL;
        }
        let save_vbadness = vbadness();
        *vbadness_mut() = INF_BAD;
        let save_vfuzz = vfuzz();
        *vfuzz_mut() = MAX_DIMEN;
        *box_mut(255) = self.vpackage(link(PAGE_HEAD), self.best_size, EXACTLY, self.page_max_depth)?;
        *vbadness_mut() = save_vbadness;
        *vfuzz_mut() = save_vfuzz;
        if self.last_glue != MAX_HALFWORD {
            self.delete_glue_ref(self.last_glue);
        }

        // Section 991
        self.page_contents = EMPTY as SmallNumber;
        self.page_tail = PAGE_HEAD;
        *link_mut(PAGE_HEAD) = NULL;
        self.last_glue = MAX_HALFWORD;
        self.last_penalty = 0;
        self.last_kern = 0;
        page_depth!(self) = 0;
        self.page_max_depth = 0;
        // End section 991

        if q != HOLD_HEAD {
            *link_mut(PAGE_HEAD) = link(HOLD_HEAD);
            self.page_tail = q;
        }
        // End section 1017
        
        // Section 1019
        let mut r = link(PAGE_INS_HEAD);
        while r != PAGE_INS_HEAD {
            q = link(r);
            self.free_node(r, PAGE_INS_NODE_SIZE);
            r = q;
        }
        *link_mut(PAGE_INS_HEAD) = PAGE_INS_HEAD;
        // End section 1019
        // End section 1014

        if self.top_mark() != NULL && self.first_mark() == NULL {
            *self.first_mark_mut() = self.top_mark();
            add_token_ref!(self.top_mark());
        }

        if output_routine() != NULL {
            if self.dead_cycles >= max_dead_cycles() {
                return Err(TeXError::OutputLoop);
            }
            else {
                // Section 1025
                self.output_active = true;
                self.dead_cycles += 1;
                self.push_nest()?;
                *self.mode_mut() = -VMODE;
                *self.prev_depth_mut() = IGNORE_DEPTH;
                *self.mode_line_mut() = -self.line;
                self.begin_token_list(output_routine(), OUTPUT_TEXT)?;
                self.new_save_level(OUTPUT_GROUP)?;
                self.normal_paragraph()?;
                self.scan_left_brace()?;
                return Ok(())
                // End section 1025
            }
        }

        // Section 1023
        if link(PAGE_HEAD) != NULL {
            if link(CONTRIB_HEAD) == NULL {
                if self.nest_ptr == 0 {
                    *self.tail_mut() = self.page_tail;
                }
                else {
                    contrib_tail!(self) = self.page_tail;
                }
            }
            else {
                *link_mut(self.page_tail) = link(CONTRIB_HEAD);
            }
            *link_mut(CONTRIB_HEAD) = link(PAGE_HEAD);
            *link_mut(PAGE_HEAD) = NULL;
            self.page_tail = PAGE_HEAD;
        }
        self.ship_out(r#box(255))?;
        *box_mut(255) = NULL;
        // End secction 1023
        Ok(())
    }
}
