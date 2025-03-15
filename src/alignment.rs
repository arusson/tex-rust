
use crate::constants::*; 
use crate::datastructures::{
    mem, mem_mut, Status, depth, depth_mut, display_indent, every_cr, global_defs,
    glue_order, glue_order_mut, glue_ptr, glue_ptr_mut, glue_ref_count_mut,
    glue_set, glue_set_mut, glue_shrink, glue_shrink_mut, glue_sign,
    glue_sign_mut, glue_stretch, glue_stretch_mut, height, height_mut,
    info, info_mut, link, link_mut, list_ptr, llink, llink_mut, overfull_rule,
    overfull_rule_mut, post_display_penalty, pre_display_penalty, rlink,
    rlink_mut, shift_amount_mut, shrink, shrink_order, span_count,
    span_count_mut, stretch, stretch_order, subtype_mut, r#type, type_mut,
    width, width_mut
};
use crate::error::{TeXResult, TeXError};
use crate::{
    Global, HalfWord, Integer, QuarterWord, Real, Scaled, SmallNumber,
    add_glue_ref, free_avail, hpack, is_running,
    sec406_get_next_nonblank_noncall_token, tail_append, vpack
};

use std::cmp::Ordering::{Equal, Greater, Less};

// Part 37: Alignment

// Section 769
fn u_part(p: HalfWord) -> Integer {
    mem((p + HEIGHT_OFFSET) as usize).int()
}

fn u_part_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut((p + HEIGHT_OFFSET) as usize).int_mut()
}

pub(crate) fn v_part(p: HalfWord) -> Integer {
    mem((p + DEPTH_OFFSET) as usize).int()
}

fn v_part_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut((p + DEPTH_OFFSET) as usize).int_mut()
}

pub(crate) fn extra_info(p: HalfWord) -> HalfWord {
    info(p + LIST_OFFSET)
}

pub(crate) fn extra_info_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + LIST_OFFSET)
}

// Section 770
fn preamble() -> HalfWord {
    link(ALIGN_HEAD)
}

fn preamble_mut() -> &'static mut HalfWord {
    link_mut(ALIGN_HEAD)
}

impl Global {
    // Section 772
    fn push_alignment(&mut self) -> TeXResult<()> {
        let p = self.get_node(ALIGN_STACK_NODE_SIZE)?;
        *link_mut(p) = self.align_ptr;
        *info_mut(p) = self.cur_align;
        *llink_mut(p) = preamble();
        *rlink_mut(p) = self.cur_span;
        *mem_mut((p + 2) as usize).int_mut() = self.cur_loop;
        *mem_mut((p + 3) as usize).int_mut() = self.align_state;
        *info_mut(p + 4) = self.cur_head;
        *link_mut(p + 4) = self.cur_tail;
        self.align_ptr = p;
        self.cur_head = self.get_avail()?;
        Ok(())
    }

    fn pop_alignment(&mut self) {
        free_avail!(self, self.cur_head);
        let p = self.align_ptr;
        self.cur_tail = link(p + 4);
        self.cur_head = info(p + 4);
        self.align_state = mem((p + 3) as usize).int();
        self.cur_loop = mem((p + 2) as usize).int();
        self.cur_span = rlink(p);
        *preamble_mut() = llink(p);
        self.cur_align = info(p);
        self.align_ptr = link(p);
        self.free_node(p, ALIGN_STACK_NODE_SIZE);
    }

    // Section 774
    pub(crate) fn init_align(&mut self) -> TeXResult<()> {
        let save_cs_ptr = self.cur_cs;
        self.push_alignment()?;
        self.align_state = -1_000_000;

        // Section 776
        if self.mode() == MMODE
            && (self.tail() != self.head() || self.incomplete_noad() != NULL)
        {
            return Err(TeXError::ImproperHalignDisplay);
        }
        // End section 776

        self.push_nest()?;

        // Section 775
        if self.mode() == MMODE {
            *self.mode_mut() = -VMODE;
            *self.prev_depth_mut() = self.nest[self.nest_ptr - 2].aux_field.sc();
        }
        else if self.mode() > 0 {
            *self.mode_mut() = -self.mode();
        }
        // End section 775

        self.scan_spec(ALIGN_GROUP, false)?;

        // Section 777
        *preamble_mut() = NULL;
        self.cur_align = ALIGN_HEAD;
        self.cur_loop = NULL;
        self.scanner_status = Status::Aligning;
        self.warning_index = save_cs_ptr;
        self.align_state = -1_000_000;
        'sec777: loop {
            // Section 778
            *link_mut(self.cur_align) = self.new_param_glue(TAB_SKIP_CODE as SmallNumber)?;
            self.cur_align = link(self.cur_align);
            // End section 778

            if self.cur_cmd == CAR_RET {
                break 'sec777; // Goto done
            }

            // Section 779
            // Section 783
            let mut p = HOLD_HEAD;
            *link_mut(p) = NULL;
            'sec783: loop {
                self.get_preamble_token()?;
                if self.cur_cmd == MAC_PARAM {
                    break 'sec783; // Goto done1
                }
                if self.cur_cmd <= CAR_RET
                    && self.cur_cmd >= TAB_MARK
                    && self.align_state == -1_000_000
                {
                    if p == HOLD_HEAD
                        && self.cur_loop == NULL
                        && self.cur_cmd == TAB_MARK
                    {
                        self.cur_loop = self.cur_align;
                    }
                    else {
                        return Err(TeXError::MissingCroisillonAlign);
                    }
                }
                else if self.cur_cmd != SPACER || p != HOLD_HEAD {
                    *link_mut(p) = self.get_avail()?;
                    p = link(p);
                    *info_mut(p) = self.cur_tok;
                }
            }
            // done1:
            // End section 783
            
            *link_mut(self.cur_align) = self.new_null_box()?;
            self.cur_align = link(self.cur_align);
            *info_mut(self.cur_align) = END_SPAN;
            *width_mut(self.cur_align) = NULL_FLAG;
            *u_part_mut(self.cur_align) = link(HOLD_HEAD);

            // Section 784
            p = HOLD_HEAD;
            *link_mut(p) = NULL;
            'sec784: loop {
                self.get_preamble_token()?;
                if self.cur_cmd <= CAR_RET
                    && self.cur_cmd >= TAB_MARK
                    && self.align_state == -1_000_000
                {
                    break 'sec784; // Goto done2
                }
                if self.cur_cmd == MAC_PARAM {
                    return Err(TeXError::OnlyOneCroisillonAllowed);
                }
                *link_mut(p) = self.get_avail()?;
                p = link(p);
                *info_mut(p) = self.cur_tok;
            }
            // done2:
            *link_mut(p) = self.get_avail()?;
            p = link(p);
            *info_mut(p) = END_TEMPLATE_TOKEN;
            // End section 784

            *v_part_mut(self.cur_align) = link(HOLD_HEAD);
            // End section 779
        }

        // done:
        self.scanner_status = Status::Normal;
        // End section 777

        self.new_save_level(ALIGN_GROUP)?;
        if every_cr() != NULL {
            self.begin_token_list(every_cr(), EVERY_CR_TEXT)?;
        }
        self.align_peek()?;
        Ok(())
    }

    // Section 782
    fn get_preamble_token(&mut self) -> TeXResult<()> {
        // restart:
        loop {
            self.get_token()?;
            while self.cur_chr == SPAN_CODE && self.cur_cmd == TAB_MARK {
                self.get_token()?;
                if self.cur_cmd > MAX_COMMAND {
                    self.expand()?;
                    self.get_token()?;
                }
            }
            if self.cur_cmd == ENDV {
                return Err(TeXError::Fatal("(interwoven alignment preambles are not allowed)"));
            }
            if self.cur_cmd == ASSIGN_GLUE
                && self.cur_chr == GLUE_BASE + TAB_SKIP_CODE
            {
                self.scan_optional_equals()?;
                self.scan_glue(GLUE_VAL as QuarterWord)?;
                if global_defs() > 0 {
                    self.geq_define(GLUE_BASE + TAB_SKIP_CODE, GLUE_REF, self.cur_val)?;
                }
                else {
                    self.eq_define(GLUE_BASE + TAB_SKIP_CODE, GLUE_REF, self.cur_val)?;
                }
                // Goto restart
            }
            else {
                break;
            }
        }
        Ok(())
    }

    // Section 785
    pub(crate) fn align_peek(&mut self) -> TeXResult<()> {
        // restart:
        loop {
            self.align_state = 1_000_000;
            sec406_get_next_nonblank_noncall_token!(self);
            if self.cur_cmd == NO_ALIGN {
                self.scan_left_brace()?;
                self.new_save_level(NO_ALIGN_GROUP)?;
                if self.mode() == -VMODE {
                    self.normal_paragraph()?;
                }
            }
            else if self.cur_cmd == RIGHT_BRACE {
                self.fin_align()?;
            }
            else if self.cur_cmd == CAR_RET
                && self.cur_chr == CR_CR_CODE
            {
                continue; // Goto restart
            }
            else {
                self.init_row()?;
                self.init_col()?;
            }
            break;
        }
        Ok(())
    }

    // Section 786
    fn init_row(&mut self) -> TeXResult<()> {
        self.push_nest()?;
        *self.mode_mut() = (-HMODE - VMODE) - self.mode();
        if self.mode() == -HMODE {
            *self.space_factor_mut() = 0;
        }
        else {
            *self.prev_depth_mut() = 0;
        }
        tail_append!(self, self.new_glue(glue_ptr(preamble()))?);
        *subtype_mut(self.tail()) = (TAB_SKIP_CODE + 1) as QuarterWord;
        self.cur_align = link(preamble());
        self.cur_tail = self.cur_head;
        self.init_span(self.cur_align)?;
        Ok(())
    }

    // Section 787
    fn init_span(&mut self, p: HalfWord) -> TeXResult<()> {
        self.push_nest()?;
        if self.mode() == -HMODE {
            *self.space_factor_mut() = 1000;
        }
        else {
            *self.prev_depth_mut() = IGNORE_DEPTH;
            self.normal_paragraph()?;
        }
        self.cur_span = p;
        Ok(())
    }

    // Section 788
    fn init_col(&mut self) -> TeXResult<()> {
        *extra_info_mut(self.cur_align) = self.cur_cmd as HalfWord;
        if self.cur_cmd == OMIT {
            self.align_state = 0;
        }
        else {
            self.back_input()?;
            self.begin_token_list(u_part(self.cur_align), U_TEMPLATE)?;
        }
        Ok(())
    }

    // Section 791
    pub(crate) fn fin_col(&mut self) -> TeXResult<bool> {
        if self.cur_align == NULL {
            return Err(TeXError::Confusion("endv"));
        }

        let mut q = link(self.cur_align);
        if q == NULL {
            return Err(TeXError::Confusion("endv"));
        }

        if self.align_state < 500_000 {
            return Err(TeXError::Fatal("(interwoven alignment preambles are not allowed)"));
        }

        let mut p = link(q);

        // Section 792
        if p == NULL && extra_info(self.cur_align) < CR_CODE {
            if self.cur_loop != NULL {
                // Section 793
                *link_mut(q) = self.new_null_box()?;
                p = link(q);
                *info_mut(p) = END_SPAN;
                *width_mut(p) = NULL_FLAG;
                self.cur_loop = link(self.cur_loop);

                // Setion 794
                q = HOLD_HEAD;
                let mut r = u_part(self.cur_loop);
                while r != NULL {
                    *link_mut(q) = self.get_avail()?;
                    q = link(q);
                    *info_mut(q) = info(r);
                    r = link(r);
                }
                *link_mut(q) = NULL;
                *u_part_mut(p) = link(HOLD_HEAD);
                q = HOLD_HEAD;
                r = v_part(self.cur_loop);
                while r != NULL {
                    *link_mut(q) = self.get_avail()?;
                    q = link(q);
                    *info_mut(q) = info(r);
                    r = link(r);
                }
                *link_mut(q) = NULL;
                *v_part_mut(p) = link(HOLD_HEAD);
                // End section 794

                self.cur_loop = link(self.cur_loop);
                *link_mut(p) = self.new_glue(glue_ptr(self.cur_loop))?;
                *subtype_mut(link(p)) = (TAB_SKIP_CODE + 1) as QuarterWord;
                // End section 793
            }
            else {
                return Err(TeXError::ExtraAlignmentTab);
            }
        }
        // End section 792

        if extra_info(self.cur_align) != SPAN_CODE {
            self.unsave()?;
            self.new_save_level(ALIGN_GROUP)?;

            // Section 796
            let (u, w) = if self.mode() == -HMODE {
                self.adjust_tail = self.cur_tail;
                let u = hpack!(self, link(self.head()), NATURAL)?;
                let w = width(u);
                self.cur_tail = self.adjust_tail;
                self.adjust_tail = NULL;
                (u, w)
            }
            else {
                let u = self.vpackage(link(self.head()), 0, ADDITIONAL, 0)?;
                let w = height(u);
                (u, w)
            };

            let mut n = MIN_QUARTERWORD as HalfWord;
            if self.cur_span != self.cur_align {
                // Section 798
                q = self.cur_span;
                loop {
                    n += 1;
                    q = link(link(q));
                    if q == self.cur_align {
                        break;
                    }
                }
                if n > (MAX_QUARTERWORD as HalfWord) {
                    return Err(TeXError::Confusion("65536 spans"));
                }
                q = self.cur_span;
                while link(info(q)) < n {
                    q = info(q);
                }
                if link(info(q)) > n {
                    let s = self.get_node(SPAN_NODE_SIZE)?;
                    *info_mut(s) = info(q);
                    *link_mut(s) = n;
                    *info_mut(q) = s;
                    *width_mut(s) = w;
                }
                else if width(info(q)) < w {
                    *width_mut(info(q)) = w;
                }
                // End section 798
            }
            else if w > width(self.cur_align) {
                *width_mut(self.cur_align) = w;
            }
            *type_mut(u) = UNSET_NODE;
            *span_count_mut(u) = n as QuarterWord;

            // Section 659
            let mut o = if self.total_stretch[FILLL as usize] != 0 {
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

            *glue_order_mut(u) = o;
            *glue_stretch_mut(u) = self.total_stretch[o as usize];

            // Section 665
            o = if self.total_shrink[FILLL as usize] != 0 {
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

            *glue_sign_mut(u) = o;
            *glue_shrink_mut(u) = self.total_shrink[o as usize];
            self.pop_nest();
            *link_mut(self.tail()) = u;
            *self.tail_mut() = u;
            // End section 796

            // Section 795
            tail_append!(self, self.new_glue(glue_ptr(link(self.cur_align)))?);
            *subtype_mut(self.tail()) = (TAB_SKIP_CODE + 1) as QuarterWord;
            // End section 795

            if extra_info(self.cur_align) >= CR_CODE {
                return Ok(true);
            }
            self.init_span(p)?;
        }
        self.align_state = 1_000_000;
        sec406_get_next_nonblank_noncall_token!(self);
        self.cur_align = p;
        self.init_col()?;
        Ok(false)
    }

    // Section 799
    pub(crate) fn fin_row(&mut self) -> TeXResult<()> {
        let p = if self.mode() == -HMODE {
            let p = hpack!(self, link(self.head()), NATURAL)?;
            self.pop_nest();
            self.append_to_vlist(p)?;
            if self.cur_head != self.cur_tail {
                *link_mut(self.tail()) = link(self.cur_head);
                *self.tail_mut() = self.cur_tail;
            }
            p
        }
        else {
            let p = vpack!(self, link(self.head()), NATURAL)?;
            self.pop_nest();
            *link_mut(self.tail()) = p;
            *self.tail_mut() = p;
            *self.space_factor_mut() = 1000;
            p
        };

        *type_mut(p) = UNSET_NODE;
        *glue_stretch_mut(p) = 0;
        if every_cr() != NULL {
            self.begin_token_list(every_cr(), EVERY_CR_TEXT)?;
        }
        self.align_peek()
    }

    // Section 800
    fn fin_align(&mut self) -> TeXResult<()> {
        if self.cur_group != ALIGN_GROUP {
            return Err(TeXError::Confusion("align1"));
        }
        self.unsave()?;
        if self.cur_group != ALIGN_GROUP {
            return Err(TeXError::Confusion("align0"));
        }
        self.unsave()?;

        let o = if self.nest[self.nest_ptr - 1].mode_field == MMODE {
            display_indent()
        }
        else {
            0
        };

        // Section 801
        let mut q = link(preamble());
        'sec801: loop {
            self.flush_list(u_part(q));
            self.flush_list(v_part(q));
            let p = link(link(q));
            if width(q) == NULL_FLAG {
                // Section 802
                *width_mut(q) = 0;
                let r = link(q);
                let s = glue_ptr(r);
                if s != ZERO_GLUE {
                    add_glue_ref!(ZERO_GLUE);
                    self.delete_glue_ref(s);
                    *glue_ptr_mut(r) = ZERO_GLUE;
                }
                // End section 802
            }
            if info(q) != END_SPAN {
                // Section 803
                let t = width(q) + width(glue_ptr(link(q)));
                let mut r = info(q);
                let mut s = END_SPAN;
                *info_mut(s) = p;
                let mut n = (MIN_QUARTERWORD + 1) as HalfWord;
                'sec803: loop {
                    *width_mut(r) -= t;
                    let u = info(r);
                    while link(r) > n {
                        s = info(s);
                        n = link(info(s)) + 1;
                    }
                    if link(r) < n {
                        *info_mut(r) = info(s);
                        *info_mut(s) = r;
                        *link_mut(r) -= 1;
                        s = r;
                    }
                    else {
                        if width(r) > width(info(s)) {
                            *width_mut(info(s)) = width(r);
                        }
                        self.free_node(r, SPAN_NODE_SIZE);
                    }
                    r = u;
                    if r == END_SPAN {
                        break 'sec803;
                    }
                }
                // End section 803
            }
            *type_mut(q) = UNSET_NODE;
            *span_count_mut(q) = MIN_QUARTERWORD;
            *height_mut(q) = 0;
            *depth_mut(q) = 0;
            *glue_order_mut(q) = NORMAL;
            *glue_sign_mut(q) = NORMAL;
            *glue_stretch_mut(q) = 0;
            *glue_shrink_mut(q) = 0;
            q = p;
            if q == NULL {
                break 'sec801;
            }
        }
        // End section 801
        
        // Section 804
        self.save_ptr -= 2;
        self.pack_begin_line = -self.mode_line();

        let mut p = if self.mode() == -VMODE {
            let rule_save = overfull_rule();
            *overfull_rule_mut() = 0;
            let p = hpack!(self, preamble(), self.saved(1), self.saved(0) as QuarterWord)?;
            *overfull_rule_mut() = rule_save;
            p
        }
        else {
            q = link(preamble());
            loop {
                *height_mut(q) = width(q);
                *width_mut(q) = 0;
                q = link(link(q));
                if q == NULL {
                    break;
                }
            }
            
            let p = vpack!(self, preamble(), self.saved(1), self.saved(0) as QuarterWord)?;
            
            q = link(preamble());
            loop {
                *width_mut(q) = height(q);
                *height_mut(q) = 0;
                q = link(link(q));
                if q == NULL {
                    break;
                }
            }
            p
        };

        self.pack_begin_line = 0;
        // End setion 804

        // Section 805
        q = link(self.head());
        let mut s = self.head();
        while q != NULL {
            if !self.is_char_node(q) {
                if r#type(q) == UNSET_NODE {
                    // Section 807
                    if self.mode() == -VMODE {
                        *type_mut(q) = HLIST_NODE;
                        *width_mut(q) = width(p);
                    }
                    else {
                        *type_mut(q) = VLIST_NODE;
                        *height_mut(q) = height(p);
                    }
                    *glue_order_mut(q) = glue_order(p);
                    *glue_sign_mut(q) = glue_sign(p);
                    *glue_set_mut(q) = glue_set(p);
                    *shift_amount_mut(q) = o;
                    let mut r = link(list_ptr(q));
                    s = link(list_ptr(p));
                    'sec807: loop {
                        // Section 808
                        let mut n = span_count(r) as HalfWord;
                        let mut t = width(s);
                        let w = t;
                        let mut u = HOLD_HEAD;
                        while n > (MIN_QUARTERWORD as HalfWord) {
                            n -= 1;
                            // Section 809
                            s = link(s);
                            let v = glue_ptr(s);
                            *link_mut(u) = self.new_glue(v)?;
                            u = link(u);
                            *subtype_mut(u) = (TAB_SKIP_CODE + 1) as QuarterWord;
                            t += width(v);
                            match glue_sign(p) {
                                STRETCHING => {
                                    if stretch_order(v) == glue_order(p) {
                                        t += (glue_set(p)*(stretch(v) as Real)).round() as Scaled;
                                    }
                                },

                                SHRINKING => {
                                    if shrink_order(v) == glue_order(p) {
                                        t -= (glue_set(p)*(shrink(v) as Real)).round() as Scaled;
                                    }
                                },

                                _ => ()
                            }
                            s = link(s);
                            *link_mut(u) = self.new_null_box()?;
                            u = link(u);
                            t += width(s);
                            if self.mode() == -VMODE {
                                *width_mut(u) = width(s);
                            }
                            else {
                                *type_mut(u) = VLIST_NODE;
                                *height_mut(u) = width(s);
                            }
                            // End section 809
                        }

                        if self.mode() == -VMODE {
                            // Section 810
                            *height_mut(r) = height(q);
                            *depth_mut(r) = depth(q);

                            match t.cmp(&width(r)) {
                                Equal => {
                                    *glue_sign_mut(r) = NORMAL;
                                    *glue_order_mut(r) = NORMAL;
                                    *glue_set_mut(r) = 0.0;
                                },

                                Greater => {
                                    *glue_sign_mut(r) = STRETCHING;
                                    if glue_stretch(r) == 0 {
                                        *glue_set_mut(r) = 0.0;
                                    }
                                    else {
                                        *glue_set_mut(r) = ((t - width(r)) as Real) / (glue_stretch(r) as Real);
                                    }
                                },

                                Less => {
                                    *glue_order_mut(r) = glue_sign(r);
                                    *glue_sign_mut(r) = SHRINKING;
                                    if glue_shrink(r) == 0 {
                                        *glue_set_mut(r) = 0.0;
                                    }
                                    else if glue_order(r) == NORMAL && (width(r) - t) > glue_shrink(r) {
                                        *glue_set_mut(r) = 1.0;
                                    }
                                    else {
                                        *glue_set_mut(r) = ((width(r) - t) as Real) / (glue_shrink(r) as Real);
                                    }
                                },
                            }

                            *width_mut(r) = w;
                            *type_mut(r) = HLIST_NODE;
                            // End section 810
                        }
                        else {
                            // Section 811
                            *width_mut(r) = width(q);
                            
                            match t.cmp(&height(r)) {
                                Equal => {
                                    *glue_sign_mut(r) = NORMAL;
                                    *glue_order_mut(r) = NORMAL;
                                    *glue_set_mut(r) = 0.0;
                                },

                                Greater => {
                                    *glue_sign_mut(r) = STRETCHING;
                                    if glue_stretch(r) == 0 {
                                        *glue_set_mut(r) = 0.0;
                                    }
                                    else {
                                        *glue_set_mut(r) = ((t - height(r)) as Real) / (glue_stretch(r) as Real);
                                    }
                                },

                                Less => {
                                    *glue_order_mut(r) = glue_sign(r);
                                    *glue_sign_mut(r) = SHRINKING;
                                    if glue_shrink(r) == 0 {
                                        *glue_set_mut(r) = 0.0;
                                    }
                                    else if glue_order(r) == NORMAL && (height(r) - t) > glue_shrink(r) {
                                        *glue_set_mut(r) = 1.0;
                                    }
                                    else {
                                        *glue_set_mut(r) = ((height(r) - t) as Real) / (glue_shrink(r) as Real);
                                    }
                                },
                            }

                            *height_mut(r) = w;
                            *type_mut(r) = VLIST_NODE;
                            // End section 811
                        }
                        *shift_amount_mut(r) = 0;
                        if u != HOLD_HEAD {
                            *link_mut(u) = link(r);
                            *link_mut(r) = link(HOLD_HEAD);
                            r = u;
                        }
                        // End section 808

                        r = link(link(r));
                        s = link(link(s));
                        if r == NULL {
                            break 'sec807;
                        }
                    }
                    // End section 807
                }
                else if r#type(q) == RULE_NODE {
                    // Section 806
                    if is_running!(width(q)) {
                        *width_mut(q) = width(p);
                    }
                    if is_running!(height(q)) {
                        *height_mut(q) = height(p);
                    }
                    if is_running!(depth(q)) {
                        *depth_mut(q) = depth(p);
                    }
                    if o != 0 {
                        let r = link(q);
                        *link_mut(q) = NULL;
                        q = hpack!(self, q, NATURAL)?;
                        *shift_amount_mut(q) = o;
                        *link_mut(q) = r;
                        *link_mut(s) = q;
                    }
                    // End section 806
                }
            }
            s = q;
            q = link(q);
        }
        // End section 805

        self.flush_node_list(p)?;
        self.pop_alignment();

        // Section 812
        let aux_save = self.aux();
        p = link(self.head());
        q = self.tail();
        self.pop_nest();
        if self.mode() == MMODE {
            // Section 1206
            self.do_assignments()?;
            if self.cur_cmd != MATH_SHIFT {
                return Err(TeXError::MissingDollarDollar);
            }

            // Section 1197
            self.get_x_token()?;
            if self.cur_cmd != MATH_SHIFT {
                return Err(TeXError::DisplayMathEndsWithDollars);
            }
            // End section 1197

            self.pop_nest();
            tail_append!(self, self.new_penalty(pre_display_penalty())?);
            tail_append!(self, self.new_param_glue(ABOVE_DISPLAY_SKIP_CODE as SmallNumber)?);
            *link_mut(self.tail()) = p;
            if p != NULL {
                *self.tail_mut() = q;
            }
            tail_append!(self, self.new_penalty(post_display_penalty())?);
            tail_append!(self, self.new_param_glue(BELOW_DISPLAY_SKIP_CODE as SmallNumber)?);
            *self.prev_depth_mut() = aux_save.sc();
            self.resume_after_display()?;
            // End section 1206
        }
        else {
            *self.aux_mut() = aux_save;
            *link_mut(self.tail()) = p;
            if p != NULL {
                *self.tail_mut() = q;
            }
            if self.mode() == VMODE {
                self.build_page()?;
            }
        }
        // End section 812
        
        Ok(())
    }
}
