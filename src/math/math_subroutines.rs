use crate::arithmetic::{mult_and_add, x_over_n, xn_over_d};
use crate::constants::*;
use crate::datastructures::{
    MemoryWord, character, character_mut, depth, depth_mut, fam_fnt, font,
    font_mut, height, height_mut, link, link_mut, list_ptr, list_ptr_mut,
    null_delimiter_space, r#type, shift_amount_mut, shrink, shrink_mut,
    shrink_order, shrink_order_mut, stretch, stretch_mut, stretch_order,
    stretch_order_mut, subtype, subtype_mut, type_mut, width, width_mut
};
use crate::error::TeXResult;
use crate::math::math_structures::{
    large_char, large_fam, small_char, small_fam
};
use crate::{
    Global, HalfWord, QuarterWord, Scaled, SmallNumber,
    half, hpack, nx_plus_y, odd, vpack
};

// Part 35: Subroutines for math mode

// Section 700
macro_rules! section_700_mathsy {
    ($fn_name:ident, $p:expr) => {
        pub(crate) fn $fn_name(&self, p: QuarterWord) -> Scaled {
            self.font_info[$p + (self.param_base[fam_fnt(2 + p as HalfWord) as usize] as usize)].sc()
        }
    };
}

// Section 701
macro_rules! section_701_mathex {
    ($fn_name:ident, $p:expr) => {
        pub(crate) fn $fn_name(&self) -> Scaled {
            self.font_info[$p + (self.param_base[fam_fnt(3 + self.cur_size as HalfWord) as usize] as usize)].sc()
        }
    };
}

// Section 702
#[macro_export]
macro_rules! cramped_style {
    ($p:expr) => {
        2*($p / 2) + CRAMPED
    };
}

#[macro_export]
macro_rules! sub_style {
    ($p:expr) => {
        2*($p / 4) + SCRIPT_STYLE + CRAMPED
    };
}

#[macro_export]
macro_rules! sup_style {
    ($p:expr) => {
        2*($p / 4) + SCRIPT_STYLE + ($p % 2)
    };
}

#[macro_export]
macro_rules! num_style {
    ($p:expr) => {
        $p + 2 - 2*($p / 6)
    };
}

#[macro_export]
macro_rules! denom_style {
    ($p:expr) => {
        2*($p / 2) + CRAMPED + 2 - 2*($p / 6)
    };
}

// Section 703
#[macro_export]
macro_rules! sec703_set_up_values {
    ($s:ident) => {
        $s.cur_size = if $s.cur_style < SCRIPT_STYLE {
            TEXT_SIZE
        }
        else {
            16*(($s.cur_style - TEXT_STYLE) / 2)
        };

        $s.cur_mu = x_over_n($s.math_quad($s.cur_size), 18)?.0;
    };
}

impl Global {
    // Section 700
    section_700_mathsy!(math_x_height, 5);
    section_700_mathsy!(math_quad, 6);
    section_700_mathsy!(num1, 8);
    section_700_mathsy!(num2, 9);
    section_700_mathsy!(num3, 10);
    section_700_mathsy!(denom1, 11);
    section_700_mathsy!(denom2, 12);
    section_700_mathsy!(sup1, 13);
    section_700_mathsy!(sup2, 14);
    section_700_mathsy!(sup3, 15);
    section_700_mathsy!(sub1, 16);
    section_700_mathsy!(sub2, 17);
    section_700_mathsy!(sup_drop, 18);
    section_700_mathsy!(sub_drop, 19);
    section_700_mathsy!(delim1, 20);
    section_700_mathsy!(delim2, 21);
    section_700_mathsy!(axis_height, 22);

    // Section 701
    section_701_mathex!(default_rule_thickness, 8);
    section_701_mathex!(big_op_spacing1, 9);
    section_701_mathex!(big_op_spacing2, 10);
    section_701_mathex!(big_op_spacing3, 11);
    section_701_mathex!(big_op_spacing4, 12);
    section_701_mathex!(big_op_spacing5, 13);

    // Section 704
    pub(crate) fn fraction_rule(&mut self, t: Scaled) -> TeXResult<HalfWord> {
        let p = self.new_rule()?;
        *height_mut(p) = t;
        *depth_mut(p) = 0;
        Ok(p)
    }

    // Section 705
    pub(crate) fn overbar(&mut self, b: HalfWord, k: Scaled, t: Scaled) -> TeXResult<HalfWord> {
        let mut p = self.new_kern(k)?;
        *link_mut(p) = b;
        let q = self.fraction_rule(t)?;
        *link_mut(q) = p;
        p = self.new_kern(t)?;
        *link_mut(p) = q;
        vpack!(self, p, NATURAL)
    }

    // Section 706
    pub(crate) fn var_delimiter(&mut self, d: HalfWord, s: SmallNumber, v: Scaled) -> TeXResult<HalfWord> {
        let mut f = NULL_FONT as QuarterWord;
        let mut w = 0;
        let mut large_attempt = false;
        let mut z = small_fam(d);
        let mut x = small_char(d);

        let mut c = 0;
        let mut q = MemoryWord::ZERO;
        'main_loop: loop {
            // Section 707
            if z != 0 || x != MIN_QUARTERWORD {
                z += s + 16;
                'sec707: loop {
                    z -= 16;
                    let g = fam_fnt(z as HalfWord);
                    if g != NULL_FONT {
                        // Section 708
                        let mut y = x;
                        if y >= (self.font_bc[g as usize] as QuarterWord)
                            && y <= (self.font_ec[g as usize] as QuarterWord)
                        {
                            // continue:
                            'sec708: loop {
                                q = self.char_info(g as QuarterWord, y);
                                if q.char_exists() {
                                    if q.char_tag() == EXT_TAG {
                                        f = g as QuarterWord;
                                        c = y;
                                        break 'main_loop; // Goto found
                                    }
                                    let hd = q.height_depth();
                                    let u = self.char_height(g as QuarterWord, hd) + self.char_depth(g as QuarterWord, hd);
                                    if u > w {
                                        f = g as QuarterWord;
                                        c = y;
                                        w = u;
                                        if u >= v {
                                            break 'main_loop; // Goto found
                                        }
                                    }
                                    if q.char_tag() == LIST_TAG {
                                        y = q.rem_byte();
                                        continue 'sec708; // Goto continue
                                    }
                                }
                                break 'sec708;
                            }
                        }
                        // End section 708
                    }
                    if z < 16 {
                        break 'sec707;
                    }
                }
            }
            // End section 707

            if large_attempt {
                break 'main_loop; // Goto done
            }
            large_attempt = true;
            z = large_fam(d);
            x = large_char(d);
        }

        // found:
        let b = if f != (NULL_FONT as QuarterWord) {
            // Section 710
            if q.char_tag() == EXT_TAG {
                // Section 713
                let b = self.new_null_box()?;
                *type_mut(b) = VLIST_NODE;
                let r = self.font_info[(self.exten_base[f as usize] as usize) + (q.rem_byte() as usize)];

                // Section 714
                c = r.ext_rep();
                let u = self.height_plus_depth(f, c);
                w = 0;
                q = self.char_info(f, c);
                *width_mut(b) = self.char_width(f, q) + self.char_italic(f, q);
                c = r.ext_bot();
                if c != MIN_QUARTERWORD {
                    w += self.height_plus_depth(f, c);
                }
                c = r.ext_mid();
                if c != MIN_QUARTERWORD {
                    w += self.height_plus_depth(f, c);
                }
                c = r.ext_top();
                if c != MIN_QUARTERWORD {
                    w += self.height_plus_depth(f, c);
                }
                let mut n = 0;
                if u > 0 {
                    while w < v {
                        w += u;
                        n += 1;
                        if r.ext_mid() != MIN_QUARTERWORD {
                            w += u
                        }
                    }
                }
                // End section 714

                c = r.ext_bot();
                if c != MIN_QUARTERWORD {
                    self.stack_into_box(b, f, c)?;
                }
                c = r.ext_rep();
                for _ in 1..=n {
                    self.stack_into_box(b, f, c)?;
                }
                c = r.ext_mid();
                if c != MIN_QUARTERWORD {
                    self.stack_into_box(b, f, c)?;
                    c = r.ext_rep();
                    for _ in 1..=n {
                        self.stack_into_box(b, f, c)?;
                    }
                }
                c = r.ext_top();
                if c != MIN_QUARTERWORD {
                    self.stack_into_box(b, f, c)?;
                }
                *depth_mut(b) = w - height(b);
                // End section 713
                b
            }
            else {
                self.char_box(f, c)?
            }
            // End section 710
        }
        else {
            let b = self.new_null_box()?;
            *width_mut(b) = null_delimiter_space();
            b
        };

        *shift_amount_mut(b) = half!(height(b) - depth(b)) - self.axis_height(s);
        Ok(b)
    }

    // Section 709
    pub(crate) fn char_box(&mut self, f: QuarterWord, c: QuarterWord) -> TeXResult<HalfWord> {
        let q = self.char_info(f, c);
        let hd = q.height_depth();
        let b = self.new_null_box()?;
        *width_mut(b) = self.char_width(f, q) + self.char_italic(f, q);
        *height_mut(b) = self.char_height(f, hd);
        *depth_mut(b) = self.char_depth(f, hd);
        let p = self.get_avail()?;
        *character_mut(p) = c;
        *font_mut(p) = f;
        *list_ptr_mut(b) = p;
        Ok(b)
    }

    // Section 711
    fn stack_into_box(&mut self, b: HalfWord, f: QuarterWord, c: QuarterWord) -> TeXResult<()> {
        let p = self.char_box(f, c)?;
        *link_mut(p) = list_ptr(b);
        *list_ptr_mut(b) = p;
        *height_mut(b) = height(p);
        Ok(())
    }

    // Section 712
    fn height_plus_depth(&mut self, f: QuarterWord, c: QuarterWord) -> Scaled {
        let q = self.char_info(f, c);
        let hd = q.height_depth();
        self.char_height(f, hd) + self.char_depth(f, hd)
    }

    // Section 715
    pub(crate) fn rebox(&mut self, mut b: HalfWord, w: Scaled) -> TeXResult<HalfWord> {
        if width(b) != w && list_ptr(b) != NULL {
            if r#type(b) == VLIST_NODE {
                b = hpack!(self, b, NATURAL)?;
            }
            let mut p = list_ptr(b);
            if self.is_char_node(p) && link(p) == NULL {
                let f = font(p);
                let v = self.char_width(f, self.char_info(f, character(p)));
                if v != width(b) {
                    *link_mut(p) = self.new_kern(width(b) - v)?;
                }
            }
            self.free_node(b, BOX_NODE_SIZE);
            b = self.new_glue(SS_GLUE)?;
            *link_mut(b) = p;
            while link(p) != NULL {
                p = link(p);
            }
            *link_mut(p) = self.new_glue(SS_GLUE)?;
            hpack!(self, b, w, EXACTLY)
        }
        else {
            *width_mut(b) = w;
            Ok(b)
        }
    }

    // Section 716
    pub(crate) fn math_glue(&mut self, g: HalfWord, m: Scaled) -> TeXResult<HalfWord> {
        let (mut n, mut f) = x_over_n(m, 65536)?;

        macro_rules! mu_mult {
            ($x:expr) => {
                nx_plus_y!(n, $x, xn_over_d($x, f, 65536)?.0)
            };
        }

        if f < 0 {
            n -= 1;
            f += 65536;
        }
        let p = self.get_node(GLUE_SPEC_SIZE)?;
        *width_mut(p) = mu_mult!(width(g));
        *stretch_order_mut(p) = stretch_order(g);

        *stretch_mut(p) = match stretch_order(p) {
            NORMAL => mu_mult!(stretch(g)),
            _ => stretch(g),
        };

        *shrink_order_mut(p) = shrink_order(g);
        
        *shrink_mut(p) = match shrink_order(p) {
            NORMAL => mu_mult!(shrink(g)),
            _ => shrink(g)
        };

        Ok(p)
    }
}

// Section 717
pub(crate) fn math_kern(p: HalfWord, m: Scaled) -> TeXResult<()> {
    if subtype(p) == MU_GLUE {
        let (mut n, mut f) = x_over_n(m, 65536)?;
        if f < 0 {
            n -= 1;
            f += 65536;
        }
        *width_mut(p) = nx_plus_y!(n, width(p), xn_over_d(width(p), f, 65536)?.0);
        *subtype_mut(p) = EXPLICIT;
    }
    Ok(())
}
