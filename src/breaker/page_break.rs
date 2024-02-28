use crate::arithmetic::badness;
use crate::constants::*;
use crate::datastructures::{
    box_mut, depth, glue_ptr, height, link, link_mut, list_ptr, list_ptr_mut,
    mark_ptr, penalty, r#box, r#type, shrink, shrink_order, split_max_depth,
    stretch, stretch_order, token_ref_count_mut, width, width_mut
};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Scaled, SmallNumber,
    add_token_ref, do_all_six, precedes_break, vpack
};

// Part 44: Breaking vertical lists into pages

impl Global {
    // Section 968
    pub(crate) fn prune_page_top(&mut self, mut p: HalfWord) -> TeXResult<HalfWord> {
        let mut prev_p = TEMP_HEAD;
        *link_mut(TEMP_HEAD) = p;
        while p != NULL {
            match r#type(p) {
                HLIST_NODE
                | VLIST_NODE
                | RULE_NODE => {
                    // Section 969
                    let q = self.new_skip_param(SPLIT_TOP_SKIP_CODE as SmallNumber)?;
                    *link_mut(prev_p) = q;
                    *link_mut(q) = p;
                    *width_mut(self.temp_ptr) = if width(self.temp_ptr) > height(p) {
                        width(self.temp_ptr) - height(p)
                    }
                    else {
                        0
                    };
                    p = NULL;
                    // End section 969
                },

                WHATSIT_NODE
                | MARK_NODE
                | INS_NODE => {
                    prev_p = p;
                    p = link(prev_p);
                },

                GLUE_NODE
                | KERN_NODE
                | PENALTY_NODE => {
                    let q = p;
                    p = link(q);
                    *link_mut(q) = NULL;
                    *link_mut(prev_p) = p;
                    self.flush_node_list(q)?;
                },

                _ => return Err(TeXError::Confusion("pruning"))
            }
        }
        Ok(link(TEMP_HEAD))
    }

    // Section 970
    pub(crate) fn vert_break(&mut self, mut p: HalfWord, h: Scaled, d: Scaled) -> TeXResult<HalfWord> {
        // Section 970
        macro_rules! active_height {
            ($p:expr) => [
                self.active_width[$p]
            ];
        }

        macro_rules! cur_height {
            () => {
                active_height![1]
            };
        }

        macro_rules! set_height_zero {
            ($p:expr) => {
                active_height![$p] = 0;
            };
        }

        let mut prev_p = p;
        let mut least_cost = AWFUL_BAD;
        do_all_six!(set_height_zero);
        let mut prev_dp = 0;
        let mut best_place = p;
        loop {
            // Section 972
            let not_found = 'sec972: {
                let pi = if p == NULL {
                    EJECT_PENALTY
                }
                else {
                    // Section 973
                    match r#type(p) {
                        HLIST_NODE
                        | VLIST_NODE
                        | RULE_NODE => {
                            cur_height!() += prev_dp + height(p);
                            prev_dp = depth(p);
                            break 'sec972 true; // Goto not_found
                        },

                        WHATSIT_NODE => {
                            // Section 1365
                            break 'sec972 true; // Goto not found
                            // End section 1365
                        },

                        GLUE_NODE => {
                            if precedes_break!(prev_p) {
                                0
                            }
                            else {
                                break 'sec972 false; // Goto update_heights
                            }
                        },

                        KERN_NODE => {
                            let t = if link(p) == NULL {
                                PENALTY_NODE
                            }
                            else {
                                r#type(link(p))
                            };

                            if t == GLUE_NODE {
                                0
                            }
                            else {
                                break 'sec972 false; // Goto update_heights
                            }
                        },

                        PENALTY_NODE => penalty(p),

                        MARK_NODE
                        | INS_NODE => {
                            break 'sec972 true; // Goto not_found
                        },

                        _ => return Err(TeXError::Confusion("vertbreak")),
                    }
                    // End section 973
                };

                // Section 974
                if pi < INF_PENALTY {
                    // Section 975
                    let mut b = if cur_height!() < h {
                        if active_height![3] != 0
                            || active_height![4] != 0
                            || active_height![5] != 0
                        {
                            0
                        }
                        else {
                            badness(h - cur_height!(), active_height![2])
                        }
                    }
                    else if cur_height!() - h > active_height![6] {
                        AWFUL_BAD
                    }
                    else {
                        badness(cur_height!() - h, active_height![6])
                    };
                    // End section 975
                    
                    if b < AWFUL_BAD {
                        if pi <= EJECT_PENALTY {
                            b = pi;
                        }
                        else if b < INF_BAD {
                            b += pi;
                        }
                        else {
                            b = DEPLORABLE;
                        }
                    }

                    if b <= least_cost {
                        best_place = p;
                        least_cost = b;
                        self.best_height_plus_depth = cur_height!() + prev_dp;
                    }

                    if b == AWFUL_BAD || pi <= EJECT_PENALTY {
                        return Ok(best_place); // Goto done
                    }
                }
                // End section 974

                if r#type(p) < GLUE_NODE || r#type(p) > KERN_NODE {
                    true // Goto not_found
                }
                else {
                    false
                }
            };

            // update_heights:
            if !not_found {
                // Section 976
                let q = match r#type(p) {
                    KERN_NODE => p,

                    _ => {
                        let q = glue_ptr(p);
                        active_height![2 + stretch_order(q) as usize] += stretch(q);
                        active_height![6] += shrink(q);
                        if shrink_order(q) != NORMAL && shrink(q) != 0 {
                            return Err(TeXError::InfiniteGlueShrinkageInBoxBeingSplit);
                        }
                        q
                    }
                };

                cur_height!() += prev_dp + width(q);
                prev_dp = 0;
                // End section 976
            }

            // not_found:
            if prev_dp > d {
                cur_height!() += prev_dp - d;
                prev_dp = d;
            }
            // End section 972

            prev_p = p;
            p = link(prev_p);
        }
    }

    // Section 977
    pub(crate) fn vsplit(&mut self, n: u8, h: Scaled) -> TeXResult<HalfWord> {
        let v = r#box(n as HalfWord);
        if self.split_first_mark() != NULL {
            self.delete_token_ref(self.split_first_mark());
            *self.split_first_mark_mut() = NULL;
            self.delete_token_ref(self.split_bot_mark());
            *self.split_bot_mark_mut() = NULL;
        }

        // Section 978
        if v == NULL {
            return Ok(NULL);
        }
        if r#type(v) != VLIST_NODE {
            return Err(TeXError::VsplitNeedsAVbox);
        }
        // End section 978

        let mut q = self.vert_break(list_ptr(v), h, split_max_depth())?;
        
        // Section 979
        let mut p = list_ptr(v);
        if p == q {
            *list_ptr_mut(v) = NULL;
        }
        else {
            loop {
                if r#type(p) == MARK_NODE {
                    if self.split_first_mark() == NULL {
                        *self.split_first_mark_mut() = mark_ptr(p);
                        *self.split_bot_mark_mut() = self.split_first_mark();
                        *token_ref_count_mut(self.split_first_mark()) += 2;
                    }
                    else {
                        self.delete_token_ref(self.split_bot_mark());
                        *self.split_bot_mark_mut() = mark_ptr(p);
                        add_token_ref!(self.split_bot_mark());
                    }
                }
                if link(p) == q {
                    *link_mut(p) = NULL;
                    break; // Goto done
                }
                p = link(p);
            }
        }
        // done:
        // End section 979
        
        q = self.prune_page_top(q)?;
        p = list_ptr(v);
        self.free_node(v, BOX_NODE_SIZE);
        *box_mut(n as HalfWord) = if q == NULL {
            NULL
        }
        else {
            vpack!(self, q, NATURAL)?
        };
        self.vpackage(p, h, EXACTLY, split_max_depth())
    }
}
