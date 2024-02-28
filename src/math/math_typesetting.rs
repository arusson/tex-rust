use crate::arithmetic::x_over_n;
use crate::constants::*;
use crate::datastructures::{
    MEM, bin_op_penalty, character, character_mut, delimiter_factor,
    delimiter_shortfall, depth, depth_mut, fam_fnt, glue_par, glue_ptr,
    glue_ptr_mut, glue_ref_count_mut, height, height_mut, info, info_mut, link, link_mut, list_ptr, list_ptr_mut, rel_penalty, script_space, shift_amount,
    shift_amount_mut, subtype, subtype_mut, r#type, type_mut, width, width_mut
};
use crate::error::{TeXError, TeXResult};
use crate::math::{
    display_mlist, display_mlist_mut, fam, fam_mut, math_type, math_type_mut,
    script_mlist, script_mlist_mut, script_script_mlist,
    script_script_mlist_mut, text_mlist, text_mlist_mut, thickness,
    thickness_mut,
    math_subroutines::math_kern,
};
use crate::{
    Global, HalfWord, Integer, QuarterWord, Scaled, SmallNumber, accent_chr,
    cramped_style, delimiter, denom_style, denominator, half, hpack,
    left_delimiter, mem, mem_mut, nucleus, num_style, numerator, odd,
    right_delimiter, sec703_set_up_values, sub_style, subscr, sup_style,
    supscr, vpack
};

// Part 36: Typesetting math formulas

impl Global {
    // Section 720
    fn clean_box(&mut self, p: HalfWord, s: SmallNumber) -> TeXResult<HalfWord> {
        let mut q = 'block: {
            match math_type(p) {
                MATH_CHAR => {
                    self.cur_mlist = self.new_noad()?;
                    *mem_mut![nucleus!(self.cur_mlist) as usize] = mem![p as usize];
                },

                SUB_BOX => break 'block info(p), // Goto found

                SUB_MLIST => self.cur_mlist = info(p),

                _ => break 'block self.new_null_box()?, // Goto found
            }

            let save_style = self.cur_style;
            self.cur_style = s;
            self.mlist_penalties = false;
            self.mlist_to_hlist()?;
            let q = link(TEMP_HEAD);
            self.cur_style = save_style;
            sec703_set_up_values!(self);
            q
        };

        // found:
        let x = if self.is_char_node(q) || q == NULL {
            hpack!(self, q, NATURAL)?
        }
        else if link(q) == NULL && r#type(q) <= VLIST_NODE && shift_amount(q) == 0 {
            q
        }
        else {
            hpack!(self, q, NATURAL)?
        };
        
        // Section 721
        q = list_ptr(x);
        if self.is_char_node(q) {
            let r = link(q);
            if r != NULL && link(r) == NULL && !self.is_char_node(r) && r#type(r) == KERN_NODE {
                self.free_node(r, SMALL_NODE_SIZE);
                *link_mut(q) = NULL;
            }
        }
        // End section 721
        Ok(x)
    }

    // Section 722
    fn fetch(&mut self, a: HalfWord) -> TeXResult<()> {
        self.cur_c = character(a);
        self.cur_f = fam_fnt((fam(a) + self.cur_size) as HalfWord) as QuarterWord;
        if self.cur_f == (NULL_FONT as QuarterWord) {
            return Err(TeXError::UndefinedCharacter(a));
        }
        self.cur_i = if self.cur_c >= (self.font_bc[self.cur_f as usize] as QuarterWord)
            && self.cur_c <= (self.font_ec[self.cur_f as usize] as QuarterWord)
        {
            self.char_info(self.cur_f, self.cur_c)
        }
        else {
            self.null_character
        };

        if !self.cur_i.char_exists() {
            self.char_warning(self.cur_f, self.cur_c as u8);
            *math_type_mut(a) = EMPTY;
            self.cur_i = self.null_character;
        }
        Ok(())
    }
}

// Section 725
fn new_hlist(p: HalfWord) -> Integer {
    mem![nucleus!(p) as usize].int()
}

fn new_hlist_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut![nucleus!(p) as usize].int_mut()
}

enum Sec727Goto {
    CheckDimen,
    DoneWithNoad,
    DoneWithNode
}

impl Global {
    // Section 726
    pub(crate) fn mlist_to_hlist(&mut self) -> TeXResult<()> {
        let mlist = self.cur_mlist;
        let penalties = self.mlist_penalties;
        let style = self.cur_style;
        let mut q = mlist;
        let mut r = NULL;
        let mut r_type = OP_NOAD;
        let mut max_h = 0;
        let mut max_d = 0;
        sec703_set_up_values!(self);

        while q != NULL {
            // Section 727
            // Section 728
            let goto = 'reswitch: loop {
                let mut delta = 0;
                match r#type(q) {
                    BIN_NOAD => {
                        match r_type {
                            BIN_NOAD
                            | OP_NOAD
                            | REL_NOAD
                            | OPEN_NOAD
                            | PUNCT_NOAD
                            | LEFT_NOAD => {
                                *type_mut(q) = ORD_NOAD;
                                continue 'reswitch;
                            },
                            _ => () // Do nothing
                        }
                    },

                    REL_NOAD
                    | CLOSE_NOAD
                    | PUNCT_NOAD
                    | RIGHT_NOAD => {
                        // Section 729
                        if r_type == BIN_NOAD {
                            *type_mut(r) = ORD_NOAD;
                        }
                        // End section 729

                        if r#type(q) == RIGHT_NOAD {
                            break 'reswitch Sec727Goto::DoneWithNoad;
                        }
                    },

                    // Section 733
                    LEFT_NOAD => break 'reswitch Sec727Goto::DoneWithNoad,

                    FRACTION_NOAD => {
                        self.make_fraction(q)?;
                        break 'reswitch Sec727Goto::CheckDimen;
                    },

                    OP_NOAD => {
                        delta = self.make_op(q)?;
                        if subtype(q) == LIMITS {
                            break 'reswitch Sec727Goto::CheckDimen;
                        }
                    },

                    ORD_NOAD => self.make_ord(q)?,
                    
                    OPEN_NOAD
                    | INNER_NOAD => (), // Do nothing
                    
                    RADICAL_NOAD => self.make_radical(q)?,
                    
                    OVER_NOAD => self.make_over(q)?,
                    
                    UNDER_NOAD => self.make_under(q)?,
                    
                    ACCENT_NOAD => self.make_math_accent(q)?,
                    
                    VCENTER_NOAD => self.make_vcenter(q)?,
                    // End section 733
                    
                    // Section 730
                    STYLE_NODE => {
                        self.cur_style = subtype(q);
                        sec703_set_up_values!(self);
                        break 'reswitch Sec727Goto::DoneWithNode;
                    },

                    CHOICE_NODE => {
                        // Section 731
                        let mut p: HalfWord;
                        match self.cur_style / 2 {
                            0 => {
                                p = display_mlist(q);
                                *display_mlist_mut(q) = NULL;
                            },

                            1 => {
                                p = text_mlist(q);
                                *text_mlist_mut(q) = NULL;
                            },

                            2 => {
                                p = script_mlist(q);
                                *script_mlist_mut(q) = NULL;
                            },

                            _ /* 3 */ => {
                                p = script_script_mlist(q);
                                *script_script_mlist_mut(q) = NULL;
                            }
                        }

                        self.flush_node_list(display_mlist(q))?;
                        self.flush_node_list(text_mlist(q))?;
                        self.flush_node_list(script_mlist(q))?;
                        self.flush_node_list(script_script_mlist(q))?;
                        *type_mut(q) = STYLE_NODE;
                        *subtype_mut(q) = self.cur_style;
                        *width_mut(q) = 0;
                        *depth_mut(q) = 0;
                        if p != NULL {
                            let z = link(q);
                            *link_mut(q) = p;
                            while link(p) != NULL {
                                p = link(p);
                            }
                            *link_mut(p) = z;
                        }
                        break 'reswitch Sec727Goto::DoneWithNode;
                        // End section 731
                    },

                    INS_NODE
                    | MARK_NODE
                    | ADJUST_NODE
                    | WHATSIT_NODE
                    | PENALTY_NODE
                    | DISC_NODE => break 'reswitch Sec727Goto::DoneWithNode,

                    RULE_NODE => {
                        if height(q) > max_h {
                            max_h = height(q);
                        }
                        if depth(q) > max_d {
                            max_d = depth(q);
                        }
                        break 'reswitch Sec727Goto::DoneWithNode;
                    },

                    GLUE_NODE => {
                        // Section 732
                        if subtype(q) == MU_GLUE {
                            let x = glue_ptr(q);
                            let y = self.math_glue(x, self.cur_mu)?;
                            self.delete_glue_ref(x);
                            *glue_ptr_mut(q) = y;
                            *subtype_mut(q) = NORMAL;
                        }
                        else if self.cur_size != TEXT_SIZE && subtype(q) == COND_MATH_GLUE {
                            let p = link(q);
                            if p != NULL && (r#type(p) == GLUE_NODE || r#type(p) == KERN_NODE) {
                                *link_mut(q) = link(p);
                                *link_mut(p) = NULL;
                                self.flush_node_list(p)?;
                            }
                        }
                        // End section 732
                        break 'reswitch Sec727Goto::DoneWithNode;
                    },

                    KERN_NODE => {
                        math_kern(q, self.cur_mu)?;
                        break 'reswitch Sec727Goto::DoneWithNode;
                    },
                    // End section 730

                    _ => return Err(TeXError::Confusion("mlist1"))
                }

                // Section 754
                let p = match math_type(nucleus!(q)) {
                    MATH_CHAR
                    | MATH_TEXT_CHAR => {
                        // Section 755
                        self.fetch(nucleus!(q))?;
                        if self.cur_i.char_exists() {
                            delta = self.char_italic(self.cur_f, self.cur_i);
                            let p = self.new_character(self.cur_f, self.cur_c as u8)?;
                            if math_type(nucleus!(q)) == MATH_TEXT_CHAR && self.space(self.cur_f) != 0 {
                                delta = 0
                            }
                            if math_type(subscr!(q)) == EMPTY && delta != 0 {
                                *link_mut(p) = self.new_kern(delta)?;
                                delta = 0
                            }
                            p
                        }
                        else {
                            NULL
                        }
                        // End section 755
                    },

                    EMPTY => NULL,

                    SUB_BOX => info(nucleus!(q)),

                    SUB_MLIST => {
                        self.cur_mlist = info(nucleus!(q));
                        let save_style = self.cur_style;
                        self.mlist_penalties = false;
                        self.mlist_to_hlist()?;
                        self.cur_style = save_style;
                        sec703_set_up_values!(self);
                        hpack!(self, link(TEMP_HEAD), NATURAL)?
                    },

                    _ => return Err(TeXError::Confusion("mlist2"))
                };

                *new_hlist_mut(q) = p;
                if math_type(subscr!(q)) == EMPTY && math_type(supscr!(q)) == EMPTY {
                    break 'reswitch Sec727Goto::CheckDimen;
                }
                self.make_scripts(q, delta)?;
                // End section 754
                break 'reswitch Sec727Goto::CheckDimen;
            };
            // End section 728

            if let Sec727Goto::CheckDimen = goto {
                // check_dimensions:
                let z = hpack!(self, new_hlist(q), NATURAL)?;
                if height(z) > max_h {
                    max_h = height(z);
                }
                if depth(z) > max_d {
                    max_d = depth(z);
                }
                self.free_node(z, BOX_NODE_SIZE);
            }

            match goto {
                Sec727Goto::CheckDimen
                | Sec727Goto::DoneWithNoad => {
                    // done_with_noad:
                    r = q;
                    r_type = r#type(r);
                },
                _ => (),
            }

            // done_with_node:
            q = link(q);
            // End section 727
        }
        
        // Section 729
        if r_type == BIN_NOAD {
            *type_mut(r) = ORD_NOAD;
        }
        // End section 729

        self.sec760_make_a_second_pass(mlist, style, max_d, max_h, penalties)
    }

    // Section 734
    fn make_over(&mut self, q: HalfWord) -> TeXResult<()> {
        let tmp = self.clean_box(nucleus!(q), cramped_style!(self.cur_style))?;
        *info_mut(nucleus!(q)) = self.overbar(tmp, 3*self.default_rule_thickness(), self.default_rule_thickness())?;
        *math_type_mut(nucleus!(q)) = SUB_BOX;
        Ok(())
    }

    // Section 735
    fn make_under(&mut self, q: HalfWord) -> TeXResult<()> {
        let x = self.clean_box(nucleus!(q), self.cur_style)?;
        let p = self.new_kern(3*self.default_rule_thickness())?;
        *link_mut(x) = p;
        *link_mut(p) = self.fraction_rule(self.default_rule_thickness())?;
        let y = vpack!(self, x, NATURAL)?;
        let delta = height(y) + depth(y) + self.default_rule_thickness();
        *height_mut(y) = height(x);
        *depth_mut(y) = delta - height(y);
        *info_mut(nucleus!(q)) = y;
        *math_type_mut(nucleus!(q)) = SUB_BOX;
        Ok(())
    }

    // Section 736
    fn make_vcenter(&mut self, q: HalfWord) -> TeXResult<()> {
        let v = info(nucleus!(q));
        if r#type(v) != VLIST_NODE {
            return Err(TeXError::Confusion("vcenter"));
        }
        let delta = height(v) + depth(v);
        *height_mut(v) = self.axis_height(self.cur_size) + half!(delta);
        *depth_mut(v) = delta - height(v);
        Ok(())
    }

    // Section 737
    fn make_radical(&mut self, q: HalfWord) -> TeXResult<()> {
        let x = self.clean_box(nucleus!(q), cramped_style!(self.cur_style))?;
        let mut clr = if self.cur_style < TEXT_STYLE {
            self.default_rule_thickness() + self.math_x_height(self.cur_size).abs() / 4
        }
        else {
            let clr = self.default_rule_thickness();
            clr + clr.abs() / 4
        };

        let y = self.var_delimiter(left_delimiter!(q), self.cur_size, height(x) + depth(x) + clr + self.default_rule_thickness())?;
        let delta = depth(y) - (height(x) + depth(x) + clr);
        if delta > 0 {
            clr += half!(delta);
        }
        *shift_amount_mut(y) = -(height(x) + clr);
        *link_mut(y) = self.overbar(x, clr, height(y))?;
        *info_mut(nucleus!(q)) = hpack!(self, y, NATURAL)?;
        *math_type_mut(nucleus!(q)) = SUB_BOX;
        Ok(())
    }

    // Section 738
    fn make_math_accent(&mut self, q: HalfWord) -> TeXResult<()> {
        self.fetch(accent_chr!(q))?;
        if self.cur_i.char_exists() {
            let mut i = self.cur_i;
            let mut c = self.cur_c;
            let f = self.cur_f;
            // Section 741
            let mut s = 0;
            if math_type(nucleus!(q)) == MATH_CHAR {
                self.fetch(nucleus!(q))?;
                if self.cur_i.char_tag() == LIG_TAG {
                    let mut a = self.lig_kern_start(self.cur_f, self.cur_i);
                    self.cur_i = self.font_info[a as usize];
                    if self.cur_i.skip_byte() > STOP_FLAG {
                        a = self.lig_kern_restart(self.cur_f, self.cur_i);
                        self.cur_i = self.font_info[a as usize];
                    }
                    loop {
                        if self.cur_i.next_char() == (self.skew_char[self.cur_f as usize] as QuarterWord) {
                            if self.cur_i.op_byte() >= KERN_FLAG && self.cur_i.skip_byte() <= STOP_FLAG {
                                s = self.char_kern(self.cur_f, self.cur_i);
                            }
                            break; // Goto done1
                        }
                        if self.cur_i.skip_byte() >= STOP_FLAG {
                            break; // Goto done1
                        }
                        a += (self.cur_i.skip_byte() + 1) as Integer;
                        self.cur_i = self.font_info[a as usize];
                    }
                }
            }
            // done1:
            // End section 741

            let mut x = self.clean_box(nucleus!(q), cramped_style!(self.cur_style))?;
            let w = width(x);
            let mut h = height(x);

            // Section 740
            loop {
                if i.char_tag() != LIST_TAG {
                    break; // Goto done
                }
                let y = i.rem_byte();
                i = self.char_info(f, y);
                if !i.char_exists() || self.char_width(f, i) > w {
                    break; // Goto done
                }
                c = y;
            }
            // done:
            // End section 740

            let mut delta = if h < self.x_height(f) {
                h
            }
            else {
                self.x_height(f)
            };

            if (math_type(supscr!(q)) != EMPTY || math_type(subscr!(q)) != EMPTY) && math_type(nucleus!(q)) == MATH_CHAR {
                // Section 742
                self.flush_node_list(x)?;
                x = self.new_noad()?;
                *mem_mut![nucleus!(x) as usize] = mem![nucleus!(q) as usize];
                *mem_mut![supscr!(x) as usize] = mem![supscr!(q) as usize];
                *mem_mut![subscr!(x) as usize] = mem![subscr!(q) as usize];
                *mem_mut![supscr!(q) as usize] = self.empty_field;
                *mem_mut![subscr!(q) as usize] = self.empty_field;
                *math_type_mut(nucleus!(q)) = SUB_MLIST;
                *info_mut(nucleus!(q)) = x;
                x = self.clean_box(nucleus!(q), self.cur_style)?;
                delta += height(x) - h;
                h = height(x);
                // End section 742
            }
            let mut y = self.char_box(f, c)?;
            *shift_amount_mut(y) = s + half!(w - width(y));
            *width_mut(y) = 0;
            let mut p = self.new_kern(-delta)?;
            *link_mut(p) = x;
            *link_mut(y) = p;
            y = vpack!(self, y, NATURAL)?;
            *width_mut(y) = width(x);
            if height(y) < h {
                // Section 739
                p = self.new_kern(h - height(y))?;
                *link_mut(p) = list_ptr(y);
                *list_ptr_mut(y) = p;
                *height_mut(y) = h;
                // End section 739
            }
            *info_mut(nucleus!(q)) = y;
            *math_type_mut(nucleus!(q)) = SUB_BOX;
        }

        Ok(())
    }

    // Section 743
    fn make_fraction(&mut self, q: HalfWord) -> TeXResult<()> {
        if thickness(q) == DEFAULT_CODE {
            *thickness_mut(q) = self.default_rule_thickness();
        }

        // Section 744
        let mut x = self.clean_box(numerator!(q), num_style!(self.cur_style))?;
        let mut z = self.clean_box(denominator!(q), denom_style!(self.cur_style))?;
        if width(x) < width(z) {
            x = self.rebox(x, width(z))?;
        }
        else {
            z = self.rebox(z, width(x))?;
        }
        let (mut shift_up, mut shift_down) = if self.cur_style < TEXT_STYLE {
            (self.num1(self.cur_size), self.denom1(self.cur_size))
        }
        else {
            match thickness(q) {
                0 => (self.num3(self.cur_size), self.denom2(self.cur_size)),
                _ => (self.num2(self.cur_size), self.denom2(self.cur_size))
            }
        };
        // End section 744

        let mut delta = if thickness(q) == 0 {
            // Section 745
            let clr = if self.cur_style < TEXT_STYLE {
                7*self.default_rule_thickness()
            }
            else {
                3*self.default_rule_thickness()
            };

            let delta = half!(clr - ((shift_up - depth(x)) - (height(z) - shift_down)));
            if delta > 0 {
                shift_up += delta;
                shift_down += delta;
            }
            delta
            // End section 745
        }
        else {
            // Section 746
            let clr = if self.cur_style < TEXT_STYLE {
                3*thickness(q)
            }
            else {
                thickness(q)
            };

            let delta = half!(thickness(q));
            let delta1 = clr - ((shift_up - depth(x)) - (self.axis_height(self.cur_size) + delta));
            let delta2 = clr - ((self.axis_height(self.cur_size) - delta) - (height(z) - shift_down));
            if delta1 > 0 {
                shift_up += delta1;
            }
            if delta2 > 0 {
                shift_down += delta2;
            }
            delta
            // End section 746
        };

        // Secetion 747
        let v = self.new_null_box()?;
        *type_mut(v) = VLIST_NODE;
        *height_mut(v) = shift_up + height(x);
        *depth_mut(v) = depth(z) + shift_down;
        *width_mut(v) = width(x);
        let p = if thickness(q) == 0 {
            let p = self.new_kern((shift_up - depth(x)) - (height(z) - shift_down))?;
            *link_mut(p) = z;
            p
        }
        else {
            let y = self.fraction_rule(thickness(q))?;
            let mut p = self.new_kern((self.axis_height(self.cur_size) - delta) - (height(z) - shift_down))?;
            *link_mut(y) = p;
            *link_mut(p) = z;
            p = self.new_kern((shift_up - depth(x)) - (self.axis_height(self.cur_size) + delta))?;
            *link_mut(p) = y;
            p
        };

        *link_mut(x) = p;
        *list_ptr_mut(v) = x;
        // End section 747

        // Section 748
        delta = if self.cur_style < TEXT_STYLE {
            self.delim1(self.cur_size)
        }
        else {
            self.delim2(self.cur_size)
        };

        x = self.var_delimiter(left_delimiter!(q), self.cur_size, delta)?;
        *link_mut(x) = v;
        z = self.var_delimiter(right_delimiter!(q), self.cur_size, delta)?;
        *link_mut(v) = z;
        *new_hlist_mut(q) = hpack!(self, x, NATURAL)?;
        // End section 748
        Ok(())
    }

    // Section 749
    fn make_op(&mut self, q: HalfWord) -> TeXResult<Scaled> {
        if subtype(q) == NORMAL && self.cur_style < TEXT_STYLE {
            *subtype_mut(q) = LIMITS;
        }

        let delta = if math_type(nucleus!(q)) == MATH_CHAR {
            self.fetch(nucleus!(q))?;
            if self.cur_style < TEXT_STYLE && self.cur_i.char_tag() == LIST_TAG {
                let c = self.cur_i.rem_byte();
                let i = self.char_info(self.cur_f, c);
                if i.char_exists() {
                    self.cur_c = c;
                    self.cur_i = i;
                    *character_mut(nucleus!(q)) = c;
                }
            }
            let delta = self.char_italic(self.cur_f, self.cur_i);
            let x = self.clean_box(nucleus!(q), self.cur_style)?;
            if math_type(subscr!(q)) != EMPTY && subtype(q) != LIMITS {
                *width_mut(x) -= delta;
            }
            *shift_amount_mut(x) = half!(height(x) - depth(x)) - self.axis_height(self.cur_size);
            *math_type_mut(nucleus!(q)) = SUB_BOX;
            *info_mut(nucleus!(q)) = x;
            delta
        }
        else {
            0
        };

        if subtype(q) == LIMITS {
            // Section 750
            let mut x = self.clean_box(supscr!(q), sup_style!(self.cur_style))?;
            let mut y = self.clean_box(nucleus!(q), self.cur_style)?;
            let mut z = self.clean_box(subscr!(q), sub_style!(self.cur_style))?;
            let v = self.new_null_box()?;
            *type_mut(v) = VLIST_NODE;
            *width_mut(v) = width(y);
            if width(x) > width(v) {
                *width_mut(v) = width(x);
            }
            if width(z) > width(v) {
                *width_mut(v) = width(z);
            }
            x = self.rebox(x, width(v))?;
            y = self.rebox(y, width(v))?;
            z = self.rebox(z, width(v))?;
            *shift_amount_mut(x) = half!(delta);
            *shift_amount_mut(z) = -shift_amount(x);
            *height_mut(v) = height(y);
            *depth_mut(v) = depth(y);

            // Section 751
            if math_type(supscr!(q)) == EMPTY {
                self.free_node(x, BOX_NODE_SIZE);
                *list_ptr_mut(v) = y;
            }
            else {
                let mut shift_up = self.big_op_spacing3() - depth(x);
                if shift_up < self.big_op_spacing1() {
                    shift_up = self.big_op_spacing1();
                }
                let mut p = self.new_kern(shift_up)?;
                *link_mut(p) = y;
                *link_mut(x) = p;
                p = self.new_kern(self.big_op_spacing5())?;
                *link_mut(p) = x;
                *list_ptr_mut(v) = p;
                *height_mut(v) += self.big_op_spacing5() + height(x) + depth(x) + shift_up;
            }
            if math_type(subscr!(q)) == EMPTY {
                self.free_node(z, BOX_NODE_SIZE);
            }
            else {
                let mut shift_down = self.big_op_spacing4() - height(z);
                if shift_down < self.big_op_spacing2() {
                    shift_down = self.big_op_spacing2();
                }
                let mut p = self.new_kern(shift_down)?;
                *link_mut(y) = p;
                *link_mut(p) = z;
                p = self.new_kern(self.big_op_spacing5())?;
                *link_mut(z) = p;
                *depth_mut(v) += self.big_op_spacing5() + height(z) + depth(z) + shift_down;
            }
            // End section 751

            *new_hlist_mut(q) = v;
            // End section 750
        }
        Ok(delta)
    }

    // Section 752
    fn make_ord(&mut self, q: HalfWord) -> TeXResult<()> {
        'restart: loop {
            if math_type(subscr!(q)) == EMPTY
                && math_type(supscr!(q)) == EMPTY
                && math_type(nucleus!(q)) == MATH_CHAR
            {
                let p = link(q);
                if p != NULL
                    && r#type(p) >= ORD_NOAD
                    && r#type(p) <= PUNCT_NOAD
                    && math_type(nucleus!(p)) == MATH_CHAR
                    && fam(nucleus!(p)) == fam(nucleus!(q))
                {
                    *math_type_mut(nucleus!(q)) = MATH_TEXT_CHAR;
                    self.fetch(nucleus!(q))?;
                    if self.cur_i.char_tag() == LIG_TAG {
                        let mut a = self.lig_kern_start(self.cur_f, self.cur_i);
                        self.cur_c = character(nucleus!(p));
                        self.cur_i = self.font_info[a as usize];
                        if self.cur_i.skip_byte() > STOP_FLAG {
                            a = self.lig_kern_restart(self.cur_f, self.cur_i);
                            self.cur_i = self.font_info[a as usize];
                        }
                        loop {
                            match self.sec753_if_instruction(p, q)? {
                                Goto::Restart => continue 'restart,
                                Goto::Return => return Ok(()),
                                Goto::Nothing => ()
                            }
                            if self.cur_i.skip_byte() >= STOP_FLAG {
                                return Ok(());
                            }
                            a += (self.cur_i.skip_byte() + 1) as Integer;
                            self.cur_i = self.font_info[a as usize];
                        }
                    }
                }
            }
            break 'restart;
        }
        Ok(())
    }
}

enum Goto {
    Restart,
    Return,
    Nothing
}

impl Global {
    // Section 753
    fn sec753_if_instruction(&mut self, p: HalfWord, q: HalfWord) -> TeXResult<Goto> {
        if self.cur_i.next_char() == self.cur_c && self.cur_i.skip_byte() <= STOP_FLAG {
            if self.cur_i.op_byte() >= KERN_FLAG {
                let p = self.new_kern(self.char_kern(self.cur_f, self.cur_i))?;
                *link_mut(p) = link(q);
                *link_mut(q) = p;
                Ok(Goto::Return)
            }
            else {
                self.check_interrupt()?;
                match self.cur_i.op_byte() {
                    1 | 5 => *character_mut(nucleus!(q)) = self.cur_i.rem_byte(),

                    2 | 6 => *character_mut(nucleus!(p)) = self.cur_i.rem_byte(),
                    
                    3 | 7 | 11 => {
                        let r = self.new_noad()?;
                        *character_mut(nucleus!(r)) = self.cur_i.rem_byte();
                        *fam_mut(nucleus!(r)) = fam(nucleus!(q));
                        *link_mut(q) = r;
                        *link_mut(r) = p;
                        *math_type_mut(nucleus!(r)) = if self.cur_i.op_byte() < 11 {
                            MATH_CHAR
                        }
                        else {
                            MATH_TEXT_CHAR
                        };
                    },
                    
                    _ => {
                        *link_mut(q) = link(p);
                        *character_mut(nucleus!(q)) = self.cur_i.rem_byte();
                        *mem_mut![subscr!(q) as usize] = mem![subscr!(p) as usize];
                        *mem_mut![supscr!(q) as usize] = mem![supscr!(p) as usize];
                        self.free_node(p, NOAD_SIZE);
                    }
                }
                if self.cur_i.op_byte() > 3 {
                    Ok(Goto::Return)
                }
                else {
                    *math_type_mut(nucleus!(q)) = MATH_CHAR;
                    Ok(Goto::Restart)
                }
            }
        }
        else {
            Ok(Goto::Nothing)
        }
    }

    // Section 756
    fn make_scripts(&mut self, q: HalfWord, delta: Scaled) -> TeXResult<()> {
        let mut p = new_hlist(q);
        let (mut shift_up, mut shift_down) = if self.is_char_node(p) {
            (0, 0)
        }
        else {
            let z = hpack!(self, p, NATURAL)?;
            let t = if self.cur_style < SCRIPT_STYLE {
                SCRIPT_SIZE
            }
            else {
                SCRIPT_SCRIPT_SIZE
            };

            let shift_up = height(z) - self.sup_drop(t);
            let shift_down = depth(z) + self.sub_drop(t);
            self.free_node(z, BOX_NODE_SIZE);
            (shift_up, shift_down)
        };

        let x = if math_type(supscr!(q)) == EMPTY {
            // Section 757
            let x = self.clean_box(subscr!(q), sub_style!(self.cur_style))?;
            *width_mut(x) += script_space();
            if shift_down < self.sub1(self.cur_size) {
                shift_down = self.sub1(self.cur_size);
            }
            let clr = height(x) - (self.math_x_height(self.cur_size)*4).abs() / 5;
            if shift_down < clr {
                shift_down = clr;
            }
            *shift_amount_mut(x) = shift_down;
            x
            // End section 757
        }
        else {
            // Section 758
            let mut x = self.clean_box(supscr!(q), sup_style!(self.cur_style))?;
            *width_mut(x) += script_space();
            let mut clr = if odd!(self.cur_style) {
                self.sup3(self.cur_size)
            }
            else if self.cur_style < TEXT_STYLE {
                self.sup1(self.cur_size)
            }
            else {
                self.sup2(self.cur_size)
            };

            if shift_up < clr {
                shift_up = clr;
            }
            clr = depth(x) + self.math_x_height(self.cur_size).abs() / 4;
            if shift_up < clr {
                shift_up = clr;
            }
            // End section 758

            if math_type(subscr!(q)) == EMPTY {
                *shift_amount_mut(x) = -shift_up;
            }
            else {
                // Section 759
                let y = self.clean_box(subscr!(q), sub_style!(self.cur_style))?;
                *width_mut(y) += script_space();
                if shift_down < self.sub2(self.cur_size) {
                    shift_down = self.sub2(self.cur_size);
                }
                clr = 4*self.default_rule_thickness() - ((shift_up - depth(x)) - (height(y) - shift_down));
                if clr > 0 {
                    shift_down += clr;
                    clr = (self.math_x_height(self.cur_size)*4).abs() / 5 - (shift_up - depth(x));
                    if clr > 0 {
                        shift_up += clr;
                        shift_down -= clr;
                    }
                }
                *shift_amount_mut(x) = delta;
                p = self.new_kern((shift_up - depth(x)) - (height(y) - shift_down))?;
                *link_mut(x) = p;
                *link_mut(p) = y;
                x = vpack!(self, x, NATURAL)?;
                *shift_amount_mut(x) = shift_down;
                // End section 759
            }
            x
        };

        if new_hlist(q) == NULL {
            *new_hlist_mut(q) = x;
        }
        else {
            p = new_hlist(q);
            while link(p) != NULL {
                p = link(p);
            }
            *link_mut(p) = x;
        }

        Ok(())
    }

    // Section 760
    fn sec760_make_a_second_pass(&mut self, mlist: HalfWord, style: QuarterWord, max_d: Scaled, max_h: Scaled, penalties: bool) -> TeXResult<()> {
        let mut p = TEMP_HEAD;
        *link_mut(p) = NULL;
        let mut q = mlist;
        let mut r_type = 0;
        self.cur_style = style;
        sec703_set_up_values!(self);
        while q != NULL {
            // Section 761
            let mut t = ORD_NOAD;
            let mut s = NOAD_SIZE;
            let mut pen = INF_PENALTY;
            match r#type(q) {
                OP_NOAD
                | OPEN_NOAD
                | CLOSE_NOAD
                | PUNCT_NOAD
                | INNER_NOAD => t = r#type(q),

                BIN_NOAD => {
                    t = BIN_NOAD;
                    pen = bin_op_penalty();
                },

                REL_NOAD => {
                    t = REL_NOAD;
                    pen = rel_penalty();
                },

                ORD_NOAD
                | VCENTER_NOAD
                | OVER_NOAD
                | UNDER_NOAD => (), // Do nothing
                
                RADICAL_NOAD => s = RADICAL_NOAD_SIZE,
                
                ACCENT_NOAD => s = ACCENT_NOAD_SIZE,
                
                FRACTION_NOAD => s = FRACTION_NOAD_SIZE,
                
                LEFT_NOAD
                | RIGHT_NOAD => t = self.make_left_right(q, style, max_d, max_h)?,
                
                STYLE_NODE => {
                    // Section 763
                    self.cur_style = subtype(q);
                    s = STYLE_NODE_SIZE;
                    sec703_set_up_values!(self);
                    // End section 763

                    // delete_q:
                    let r = q;
                    q = link(q);
                    self.free_node(r, s);
                    continue;
                },
                
                WHATSIT_NODE
                | PENALTY_NODE
                | RULE_NODE
                | DISC_NODE
                | ADJUST_NODE
                | INS_NODE
                | MARK_NODE
                | GLUE_NODE
                | KERN_NODE => {
                    *link_mut(p) = q;
                    p = q;
                    q = link(q);
                    *link_mut(p) = NULL;
                    continue; // Goto done (continue)
                }
                
                _ => return Err(TeXError::Confusion("mlist3"))
            }
            // End section 761

            // Section 766
            if r_type > 0 {
                let x = match MATH_SPACING[(r_type*8 + t - 9*ORD_NOAD) as usize] {
                    b'0' => 0,

                    b'1' => {
                        if self.cur_style < SCRIPT_STYLE {
                            THIN_MU_SKIP_CODE
                        }
                        else {
                            0
                        }
                    },

                    b'2' => THIN_MU_SKIP_CODE,

                    b'3' => {
                        if self.cur_style < SCRIPT_STYLE {
                            MED_MU_SKIP_CODE
                        }
                        else {
                            0
                        }
                    },

                    b'4' => if self.cur_style < SCRIPT_STYLE {
                        THICK_MU_SKIP_CODE
                    }
                    else {
                        0
                    },
                    
                    _ => return Err(TeXError::Confusion("mlist4"))
                };

                if x != 0 {
                    let y = self.math_glue(glue_par(x), self.cur_mu)?;
                    let z = self.new_glue(y)?;
                    *glue_ref_count_mut(y) = NULL;
                    *link_mut(p) = z;
                    p = z;
                    *subtype_mut(z) = (x + 1) as QuarterWord;
                }
            }
            // End section 766

            // Section 767
            if new_hlist(q) != NULL {
                *link_mut(p) = new_hlist(q);
                loop {
                    p = link(p);
                    if link(p) == NULL {
                        break;
                    }
                }
            }

            if penalties && link(q) != NULL && pen < INF_PENALTY {
                r_type = r#type(link(q));
                if r_type != PENALTY_NODE && r_type != REL_NOAD {
                    let z = self.new_penalty(pen)?;
                    *link_mut(p) = z;
                    p = z;
                }
            }
            // End section 767

            r_type = t;
            // delete_q:
            let r = q;
            q = link(q);
            self.free_node(r, s);
            // done
        }
        Ok(())
    }

    // Section 762
    fn make_left_right(&mut self, q: HalfWord, style: QuarterWord, max_d: Scaled, max_h: Scaled) -> TeXResult<QuarterWord> {
        self.cur_size = if style < SCRIPT_STYLE {
            TEXT_SIZE
        }
        else {
            16*((style - TEXT_STYLE) / 2)
        };

        let mut delta2 = max_d + self.axis_height(self.cur_size);
        let mut delta1 = max_h + max_d - delta2;
        if delta2 > delta1 {
            delta1 = delta2;
        }
        let mut delta = (delta1 / 500)*delimiter_factor();
        delta2 = delta1 + delta1 - delimiter_shortfall();
        if delta < delta2 {
            delta = delta2;
        }
        *new_hlist_mut(q) = self.var_delimiter(delimiter!(q), self.cur_size, delta)?;
        Ok(r#type(q) - (LEFT_NOAD - OPEN_NOAD))
    }
}
