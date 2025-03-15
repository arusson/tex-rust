use crate::arithmetic::badness;
use crate::constants::*;
use crate::error::{TeXError, TeXResult};
use crate::datastructures::{
    mem, mem_mut, adjust_ptr, baseline_skip, character, depth, depth_mut, font,
    glue_order_mut, glue_ptr, glue_set_mut, glue_sign_mut, hbadness, height,
    height_mut, hfuzz, leader_ptr, line_skip_limit, link, link_mut, list_ptr,
    list_ptr_mut, overfull_rule, shift_amount, shift_amount_mut, shrink,
    shrink_order, stretch, stretch_order, subtype, subtype_mut, r#type,
    type_mut, vbadness, vfuzz, width, width_mut
};
use crate::{
    Global, HalfWord, QuarterWord, Real, Scaled, SmallNumber, lig_char
};

// Part 33: Packaging

#[macro_export]
macro_rules! hpack {
    ($s:ident, $b:expr, NATURAL) => {
        $s.hpack($b, 0, ADDITIONAL)
    };

    ($s:ident, $($args:expr),*) => {
        $s.hpack($($args),*)
    };
}

impl Global {
    // Section 649
    pub(crate) fn hpack(&mut self, mut p: HalfWord, mut w: Scaled, m: SmallNumber) -> TeXResult<HalfWord> {
        self.last_badness = 0;
        let r = self.get_node(BOX_NODE_SIZE)?;
        *type_mut(r) = HLIST_NODE;
        *subtype_mut(r) = MIN_QUARTERWORD;
        *shift_amount_mut(r) = 0;
        let mut q = r + (LIST_OFFSET);
        *link_mut(q) = p;
        let mut h = 0;

        // Section 650
        let mut d = 0;
        let mut x = 0;
        self.total_stretch.fill(0);
        self.total_shrink.fill(0);
        // End section 650

        while p != NULL {
            // Section 651
            'reswitch: loop {
                while self.is_char_node(p) {
                    // Section 654
                    let f = font(p);
                    let i = self.char_info(f, character(p));
                    let hd = i.height_depth();
                    x += self.char_width(f, i);
                    let mut s = self.char_height(f, hd);
                    if s > h {
                        h = s;
                    }
                    s = self.char_depth(f, hd);
                    if s > d {
                        d = s;
                    }
                    p = link(p);
                    // End section 654
                }

                if p != NULL {
                    match r#type(p) {
                        HLIST_NODE
                        | VLIST_NODE
                        | RULE_NODE
                        | UNSET_NODE => {
                            // Section 653
                            x += width(p);
                            let s = if r#type(p) >= RULE_NODE {
                                0
                            }
                            else {
                                shift_amount(p)
                            };

                            if height(p) - s > h {
                                h = height(p) - s;
                            }
                            if depth(p) + s > d {
                                d = depth(p) + s;
                            }
                            // End section 653
                        },

                        INS_NODE
                        | MARK_NODE
                        | ADJUST_NODE => {
                            if self.adjust_tail != NULL {
                                // Section 655
                                while link(q) != p {
                                    q = link(q);
                                }
                                if r#type(p) == ADJUST_NODE {
                                    *link_mut(self.adjust_tail) = adjust_ptr(p);
                                    while link(self.adjust_tail) != NULL {
                                        self.adjust_tail = link(self.adjust_tail);
                                    }
                                    p = link(p);
                                    self.free_node(link(q), SMALL_NODE_SIZE);
                                }
                                else {
                                    *link_mut(self.adjust_tail) = p;
                                    self.adjust_tail = p;
                                    p = link(p);
                                }
                                *link_mut(q) = p;
                                p = q;
                                // End section 655
                            }
                        },

                        WHATSIT_NODE => (), // Section 1360, Do nothing

                        GLUE_NODE => {
                            // Section 656
                            let mut g = glue_ptr(p);
                            x += width(g);
                            let mut o = stretch_order(g);
                            self.total_stretch[o as usize] += stretch(g);
                            o = shrink_order(g);
                            self.total_shrink[o as usize] += shrink(g);
                            if subtype(p) >= A_LEADERS {
                                g = leader_ptr(p);
                                if height(g) > h {
                                    h = height(g);
                                }
                                if depth(g) > d {
                                    d = depth(g);
                                }
                            }
                            // End section 656
                        },

                        KERN_NODE
                        | MATH_NODE => x += width(p),

                        LIGATURE_NODE => {
                            // Section 652
                            *mem_mut(LIG_TRICK as usize) = mem(lig_char!(p) as usize);
                            *link_mut(LIG_TRICK) = link(p);
                            p = LIG_TRICK;
                            continue 'reswitch;
                            // End section 652
                        },
                        
                        _ => (),
                    }
                    p = link(p);
                }
                break 'reswitch;
                // End section 651
            }
        }

        if self.adjust_tail != NULL {
            *link_mut(self.adjust_tail) = NULL;
        }
        *height_mut(r) = h;
        *depth_mut(r) = d;

        // Section 657
        if m == ADDITIONAL {
            w += x;
        }
        *width_mut(r) = w;
        x = w - x;
        if x == 0 {
            *glue_sign_mut(r) = NORMAL;
            *glue_order_mut(r) = NORMAL;
            *glue_set_mut(r) = 0.0;
            return Ok(r);
        }

        'block: {
            if x > 0 {
                // Section 658
                // Section 659
                let o = if self.total_stretch[FILLL as usize] != 0 {
                    FILLL
                }
                else if self.total_stretch[FILL as usize] != 0 {
                    FILL
                }
                else if self.total_stretch[FIL as usize] != 0 {
                    FIL
                }
                else {
                    NORMAL
                };
                // End section 659
                
                *glue_order_mut(r) = o;
                *glue_sign_mut(r) = STRETCHING;
                if self.total_stretch[o as usize] != 0 {
                    *glue_set_mut(r) = (x as Real) / (self.total_stretch[o as usize] as Real);
                }
                else {
                    *glue_sign_mut(r) = NORMAL;
                    *glue_set_mut(r) = 0.0;
                }

                if o == NORMAL && list_ptr(r) != NULL {
                    // Section 660
                    self.last_badness = badness(x, self.total_stretch[NORMAL as usize]);
                    if self.last_badness > hbadness() {
                        self.print_ln();
                        if self.last_badness > 100 {
                            self.print_nl("Underfull");
                        }
                        else {
                            self.print_nl("Loose");
                        }
                        self.print(" \\hbox (badness ");
                        self.print_int(self.last_badness);
                        break 'block; // Goto common_ending
                    }
                    // End section 660
                }                
                return Ok(r);
                // End section 658
            }
            // Section 664
            // Section 665
            let o = if self.total_shrink[FILLL as usize] != 0 {
                FILLL
            }
            else if self.total_shrink[FILL as usize] != 0 {
                FILL
            }
            else if self.total_shrink[FIL as usize] != 0 {
                FIL
            }
            else {
                NORMAL
            };
            // End section 665

            *glue_order_mut(r) = o;
            *glue_sign_mut(r) = SHRINKING;
            if self.total_shrink[o as usize] != 0 {
                *glue_set_mut(r) = (-x as Real) / (self.total_shrink[o as usize] as Real);
            }
            else {
                *glue_sign_mut(r) = NORMAL;
                *glue_set_mut(r) = 0.0;
            }
            if self.total_shrink[o as usize] < -x && o == NORMAL && list_ptr(r) != NULL {
                self.last_badness = 1_000_000;
                *glue_set_mut(r) = 1.0;

                // Section 666
                if -x -self.total_shrink[NORMAL as usize] > hfuzz() || hbadness() < 100 {
                    if overfull_rule() > 0 && -x - self.total_shrink[NORMAL as usize] > hfuzz() {
                        while link(q) != NULL {
                            q = link(q);
                        }
                        *link_mut(q) = self.new_rule()?;
                        *width_mut(link(q)) = overfull_rule();
                    }
                    self.print_ln();
                    self.print_nl("Overfull \\hbox (");
                    self.print_scaled(-x - self.total_shrink[NORMAL as usize]);
                    self.print("pt too wide");
                    break 'block; // Goto common_ending
                }
                // End section 666
            }
            else if o == NORMAL && list_ptr(r) != NULL {
                // Section 667
                self.last_badness = badness(-x, self.total_shrink[NORMAL as usize]);
                if self.last_badness > hbadness() {
                    self.print_ln();
                    self.print_nl("Tight \\hbox (badness ");
                    self.print_int(self.last_badness);
                    break 'block; // Goto common_ending
                }
                // End section 667
            }
            return Ok(r);
            // End section 664
        }
        // End section 657

        // common_ending:
        // Section 663
        if self.output_active {
            self.print(") has occurred while \\output is active");
        }
        else {
            if self.pack_begin_line != 0 {
                if self.pack_begin_line > 0 {
                    self.print(") in paragraph at lines ");
                }
                else {
                    self.print(") in alignment at lines ");
                }
                self.print_int(self.pack_begin_line.abs());
                self.print("--");
            }
            else {
                self.print(") detected at line ");
            }
            self.print_int(self.line);
        }
        self.print_ln();
        self.font_in_short_display = NULL_FONT as QuarterWord;
        self.short_display(list_ptr(r));
        self.print_ln();
        self.begin_diagnostic();
        self.show_box(r);
        self.end_diagnostic(true);
        // End section 663

        Ok(r)
    }
}

// Section 668
#[macro_export]
macro_rules! vpack {
    ($s:ident, $p:expr, NATURAL) => {
        $s.vpackage($p, 0, ADDITIONAL, MAX_DIMEN)
    };

    ($s:ident, $($args:expr),*) => {
        $s.vpackage($($args),*, MAX_DIMEN)
    };
}

impl Global {
    // Section 668
    pub(crate) fn vpackage(&mut self, mut p: HalfWord, mut h: Scaled, m: QuarterWord, l: Scaled) -> TeXResult<HalfWord> {
        self.last_badness = 0;
        let r = self.get_node(BOX_NODE_SIZE)?;
        *type_mut(r) = VLIST_NODE;
        *subtype_mut(r) = MIN_QUARTERWORD;
        *shift_amount_mut(r) = 0;
        *list_ptr_mut(r) = p;
        let mut w = 0;

        // Section 650
        let mut d = 0;
        let mut x = 0;
        self.total_stretch.fill(0);
        self.total_shrink.fill(0);
        // End section 650

        while p != NULL {
            // Section 669
            if self.is_char_node(p) {
                return Err(TeXError::Confusion("vpack"));
            }

            match r#type(p) {
                HLIST_NODE
                | VLIST_NODE
                | RULE_NODE
                | UNSET_NODE => {
                    // Section 670
                    x += d + height(p);
                    d = depth(p);
                    let s = if r#type(p) >= RULE_NODE {
                        0
                    }
                    else {
                        shift_amount(p)
                    };

                    if width(p) + s > w {
                        w = width(p) + s;
                    }
                    // End section 670
                },

                WHATSIT_NODE => (), // Do nothing, section 1359

                GLUE_NODE => {
                    // Section 671
                    x += d;
                    d = 0;
                    let mut g = glue_ptr(p);
                    x += width(g);
                    let mut o = stretch_order(g);
                    self.total_stretch[o as usize] += stretch(g);
                    o = shrink_order(g);
                    self.total_shrink[o as usize] += shrink(g);
                    if subtype(p) >= A_LEADERS {
                        g = leader_ptr(p);
                        if width(g) > w {
                            w = width(g);
                        }
                    }
                    // End section 671
                },

                KERN_NODE => {
                    x += d + width(p);
                    d = 0;
                },

                _ => (), // Do nothing
            }
            p = link(p);
            // End section 669
        }

        *width_mut(r) = w;
        if d > l {
            x += d - l;
            *depth_mut(r) = l;
        }
        else {
            *depth_mut(r) = d;
        }

        // Section 672
        if m == ADDITIONAL {
            h += x;
        }
        *height_mut(r) = h;
        x = h - x;
        if x == 0 {
            *glue_sign_mut(r) = NORMAL;
            *glue_order_mut(r) = NORMAL;
            *glue_set_mut(r) = 0.0;
            return Ok(r);
        }

        'block: {
            if x > 0 {
                // Section 673
                // Section 659
                let o = if self.total_stretch[FILLL as usize] != 0 {
                    FILLL
                }
                else if self.total_stretch[FILL as usize] != 0 {
                    FILL
                }
                else if self.total_stretch[FIL as usize] != 0 {
                    FIL
                }
                else {
                    NORMAL
                };
                // End section 659

                *glue_order_mut(r) = o;
                *glue_sign_mut(r) = STRETCHING;
                if self.total_stretch[o as usize] != 0 {
                    *glue_set_mut(r) = (x as Real) / (self.total_stretch[o as usize] as Real);
                }
                else {
                    *glue_sign_mut(r) = NORMAL;
                    *glue_set_mut(r) = 0.0;
                }
                if o == NORMAL && list_ptr(r) != NULL {
                    // Section 674
                    self.last_badness = badness(x, self.total_stretch[NORMAL as usize]);
                    if self.last_badness > vbadness() {
                        self.print_ln();
                        if self.last_badness > 100 {
                            self.print_nl("Underfull");
                        }
                        else {
                            self.print_nl("Loose");
                        }
                        self.print(" \\vbox (badness ");
                        self.print_int(self.last_badness);
                        break 'block; // Goto common_ending
                    }
                    // End section 674
                }
                return Ok(r);
                // End section 673
            }
            else {
                // Section 676
                // Section 665
                let o = if self.total_shrink[FILLL as usize] != 0 {
                    FILLL
                }
                else if self.total_shrink[FILL as usize] != 0 {
                    FILL
                }
                else if self.total_shrink[FIL as usize] != 0 {
                    FIL
                }
                else {
                    NORMAL
                };
                // End section 665

                *glue_order_mut(r) = o;
                *glue_sign_mut(r) = SHRINKING;
                if self.total_shrink[o as usize] != 0 {
                    *glue_set_mut(r) = (-x as Real) / (self.total_shrink[o as usize] as Real);
                }
                else {
                    *glue_sign_mut(r) = NORMAL;
                    *glue_set_mut(r) = 0.0;
                }
                if self.total_shrink[o as usize] < -x && o == NORMAL && list_ptr(r) != NULL {
                    self.last_badness = 1_000_000;
                    *glue_set_mut(r) = 1.0;
                    // Section 677
                    if -x - self.total_shrink[NORMAL as usize] > vfuzz() || vbadness() < 100 {
                        self.print_ln();
                        self.print_nl("Overfull \\vbox (");
                        self.print_scaled(-x - self.total_shrink[NORMAL as usize]);
                        self.print("pt too high");
                        break 'block; // Goto common_ending
                    }
                    // End section 677
                }
                else if o == NORMAL && list_ptr(r) != NULL {
                    // Section 678
                    self.last_badness = badness(-x, self.total_shrink[NORMAL as usize]);
                    if self.last_badness > vbadness() {
                        self.print_ln();
                        self.print_nl("Tight \\vbox (badness ");
                        self.print_int(self.last_badness);
                        break 'block; // Goto common_ending
                    }
                    // End section 678
                }
                return Ok(r);
                // End section 676
            }
            // End section 672
        }

        // common_ending:
        // Section 675
        if self.output_active {
            self.print(") has occured while \\output is active");
        }
        else {
            if self.pack_begin_line != 0 {
                self.print(") in alignment at lines ");
                self.print_int(self.pack_begin_line.abs());
                self.print("--");
            }
            else {
                self.print(") detected at line ");
            }
            self.print_int(self.line);
            self.print_ln();
        }
        self.begin_diagnostic();
        self.show_box(r);
        self.end_diagnostic(true);
        // End section 675

        Ok(r)
    }

    // Section 679
    pub(crate) fn append_to_vlist(&mut self, b: HalfWord) -> TeXResult<()> {
        if self.prev_depth() > IGNORE_DEPTH {
            let d = width(baseline_skip()) - self.prev_depth() - height(b);

            let p = if d < line_skip_limit() {
                self.new_param_glue(LINE_SKIP_CODE as SmallNumber)?
            }
            else {
                let p = self.new_skip_param(BASELINE_SKIP_CODE as SmallNumber)?;
                *width_mut(self.temp_ptr) = d;
                p
            };

            *link_mut(self.tail()) = p;
            *self.tail_mut() = p;
        }

        *link_mut(self.tail()) = b;
        *self.tail_mut() = b;
        *self.prev_depth_mut() = depth(b);
        Ok(())
    }
}
