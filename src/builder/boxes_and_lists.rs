use crate::constants::*;
use crate::datastructures::{
    mem, mem_mut, adjust_ptr_mut, box_max_depth, box_mut, character, cur_font,
    depth, depth_mut, every_hbox, every_par, every_vbox, float_cost_mut,
    floating_penalty, font, glue_ref_count_mut, hang_after, hang_indent,
    height, height_mut, info_mut, ins_ptr_mut, leader_ptr_mut,
    left_hyphen_min, link, link_mut, list_ptr, looseness, mark_ptr_mut,
    par_indent, par_shape_ptr, post_break_mut, pre_break_mut, r#box, r#type,
    replace_count, replace_count_mut, right_hyphen_min, shift_amount_mut,
    split_max_depth, split_top_ptr_mut, split_top_skip, subtype_mut, type_mut,
    widow_penalty, width_mut
};
use crate::error::{TeXError, TeXResult};
use crate::math::{math_type, math_type_mut};
use crate::{
    Global, HalfWord, Integer, QuarterWord, Real, Scaled, SmallNumber,
    add_glue_ref, contrib_tail, hpack, lig_char, nucleus,
    sec404_get_next_nonblank_nonrelax_noncall_token, subscr, supscr,
    tail_append, vpack
};

// Part 47: Building boxes and lists

impl Global {
    // Section 1060
    pub(crate) fn append_glue(&mut self) -> TeXResult<()> {
        let s = self.cur_chr;
        match s {
            FIL_CODE => self.cur_val = FIL_GLUE,
            FILL_CODE => self.cur_val = FILL_GLUE,
            SS_CODE => self.cur_val = SS_GLUE,
            FIL_NEG_CODE => self.cur_val = FIL_NEG_GLUE,
            SKIP_CODE => self.scan_glue(GLUE_VAL as QuarterWord)?,
            MSKIP_CODE => self.scan_glue(MU_VAL as QuarterWord)?,
            _ => (),
        }
        tail_append!(self, self.new_glue(self.cur_val)?);
        if s >= SKIP_CODE {
            *glue_ref_count_mut(self.cur_val) -= 1;
            if s > SKIP_CODE {
                *subtype_mut(self.tail()) = MU_GLUE;
            }
        }
        Ok(())
    }

    // Section 1061
    pub(crate) fn append_kern(&mut self) -> TeXResult<()> {
        let s = self.cur_chr;
        self.scan_dimen(s == (MU_GLUE as HalfWord), false, false)?;
        tail_append!(self, self.new_kern(self.cur_val)?);
        *subtype_mut(self.tail()) = s as QuarterWord;
        Ok(())
    }

    // Section 1064
    pub(crate) fn off_save(&mut self) -> TeXResult<()> {
        if self.cur_group == BOTTOM_LEVEL {
            Err(TeXError::Extra)
        }
        else {
            // Section 1065
            match self.cur_group {
                SEMI_SIMPLE_GROUP => Err(TeXError::MissingEndGroup),
                MATH_SHIFT_GROUP => Err(TeXError::MissingDollar),
                MATH_LEFT_GROUP => Err(TeXError::MissingMathRight),
                _ => Err(TeXError::MissingRightBrace),
            }
            // End section 1065
        }
    }

    // Section 1068
    pub(crate) fn handle_right_brace(&mut self) -> TeXResult<()> {
        match self.cur_group {
            SIMPLE_GROUP => self.unsave()?,

            BOTTOM_LEVEL => return Err(TeXError::TooManyRightBraces),

            SEMI_SIMPLE_GROUP
            | MATH_SHIFT_GROUP
            | MATH_LEFT_GROUP => return Err(TeXError::ExtraRightBraceOrForgotten),
            
            // Section 1085
            HBOX_GROUP => self.package(0)?,

            ADJUSTED_HBOX_GROUP => {
                self.adjust_tail = ADJUST_HEAD;
                self.package(0)?;
            },

            VBOX_GROUP => {
                self.end_graf()?;
                self.package(0)?;
            },

            VTOP_GROUP => {
                self.end_graf()?;
                self.package(VTOP_CODE as SmallNumber)?;
            },

            // Section 1100
            INSERT_GROUP => {
                self.end_graf()?;
                let q = split_top_skip();
                add_glue_ref!(q);
                let d = split_max_depth();
                let f = floating_penalty();
                self.unsave()?;
                self.save_ptr -= 1;
                let p = vpack!(self, link(self.head()), NATURAL)?;
                self.pop_nest();
                if self.saved(0) < 255 {
                    tail_append!(self, self.get_node(INS_NODE_SIZE)?);
                    *type_mut(self.tail()) = INS_NODE;
                    *subtype_mut(self.tail()) = self.saved(0) as QuarterWord;
                    *height_mut(self.tail()) = height(p) + depth(p);
                    *ins_ptr_mut(self.tail()) = list_ptr(p);
                    *split_top_ptr_mut(self.tail()) = q;
                    *depth_mut(self.tail()) = d;
                    *float_cost_mut(self.tail()) = f;
                }
                else {
                    tail_append!(self, self.get_node(SMALL_NODE_SIZE)?);
                    *type_mut(self.tail()) = ADJUST_NODE;
                    *subtype_mut(self.tail()) = 0;
                    *adjust_ptr_mut(self.tail()) = list_ptr(p);
                    self.delete_glue_ref(q);
                }
                self.free_node(p, BOX_NODE_SIZE);
                if self.nest_ptr == 0 {
                    self.build_page()?;
                }
            },

            OUTPUT_GROUP => {
                // Section 1026
                if self.loc() != NULL || (self.token_type() != OUTPUT_TEXT && self.token_type() != BACKED_UP) {
                    return Err(TeXError::UnbalancedOutputRoutine);
                }
                self.end_token_list()?;
                self.end_graf()?;
                self.unsave()?;
                self.output_active = false;
                self.insert_penalties = 0;

                // Section 1028
                if r#box(255) != NULL {
                    return Err(TeXError::OutputRoutineDidntUseAllOfBox255);
                }
                // End section 1028

                if self.tail() != self.head() {
                    *link_mut(self.page_tail) = link(self.head());
                    self.page_tail = self.tail();
                }
                if link(PAGE_HEAD) != NULL {
                    if link(CONTRIB_HEAD) == NULL {
                        contrib_tail!(self) = self.page_tail;
                    }
                    *link_mut(self.page_tail) = link(CONTRIB_HEAD);
                    *link_mut(CONTRIB_HEAD) = link(PAGE_HEAD);
                    *link_mut(PAGE_HEAD) = NULL;
                    self.page_tail = PAGE_HEAD;
                }
                self.pop_nest();
                self.build_page()?;
                // End section 1026
            }
            // End section 1100

            // Section 1118
            DISC_GROUP => self.build_discretionary()?,
            // End section 1118

            // Section 1132
            ALIGN_GROUP => return Err(TeXError::MissingCr),
            // End section 1132

            // Section 1133
            NO_ALIGN_GROUP => {
                self.end_graf()?;
                self.unsave()?;
                self.align_peek()?;
            }
            // End section 1133
            
            // Section 1168
            VCENTER_GROUP => {
                self.end_graf()?;
                self.unsave()?;
                self.save_ptr -= 2;
                let p = vpack!(self, link(self.head()), self.saved(1), self.saved(0) as QuarterWord)?;
                self.pop_nest();
                tail_append!(self, self.new_noad()?);
                *type_mut(self.tail()) = VCENTER_NOAD;
                *math_type_mut(nucleus!(self.tail())) = SUB_BOX;
                *info_mut(nucleus!(self.tail())) = p;
            },
            // End section 1168

            // Section 1173
            MATH_CHOICE_GROUP => self.build_choices()?,
            // End section 1173

            // Section 1186
            MATH_GROUP => {
                self.unsave()?;
                self.save_ptr -= 1;
                *math_type_mut(self.saved(0)) = SUB_MLIST;
                let p = self.fin_mlist(NULL)?;
                *info_mut(self.saved(0)) = p;
                if p != NULL && link(p) == NULL {
                    if r#type(p) == ORD_NOAD {
                        if math_type(subscr!(p)) == EMPTY && math_type(supscr!(p)) == EMPTY {
                            *mem_mut(self.saved(0) as usize) = mem(nucleus!(p) as usize);
                            self.free_node(p, NOAD_SIZE);
                        }
                    }
                    else if r#type(p) == ACCENT_NOAD && self.saved(0) == nucleus!(self.tail()) && r#type(self.tail()) == ORD_NOAD {
                        // Section 1187
                        let mut q = self.head();
                        while link(q) != self.tail() {
                            q = link(q);
                        }
                        *link_mut(q) = p;
                        self.free_node(self.tail(), NOAD_SIZE);
                        *self.tail_mut() = p;
                        // End section 1187
                    }
                }
            },
            // End section 1186
            // End section 1085

            _ => return Err(TeXError::Confusion("rightbrace"))
        }
        Ok(())
    }

    // Section 1070
    pub(crate) fn normal_paragraph(&mut self) -> TeXResult<()> {
        if looseness() != 0 {
            self.eq_word_define(INT_BASE + LOOSENESS_CODE, 0)?;
        }
        if hang_indent() != 0 {
            self.eq_word_define(DIMEN_BASE + HANG_INDENT_CODE, 0)?;
        }
        if hang_after() != 1 {
            self.eq_word_define(INT_BASE + HANG_AFTER_CODE, 1)?;
        }
        if par_shape_ptr() != NULL {
            self.eq_define(PAR_SHAPE_LOC, SHAPE_REF, NULL)?;
        }
        Ok(())
    }

    // Section 1075
    pub(crate) fn box_end(&mut self, box_content: Integer) -> TeXResult<()> {
        if box_content < BOX_FLAG {
            // Section 1076
            if self.cur_box != NULL {
                *shift_amount_mut(self.cur_box) = box_content;
                if self.mode().abs() == VMODE {
                    self.append_to_vlist(self.cur_box)?;
                    if self.adjust_tail != NULL {
                        if ADJUST_HEAD != self.adjust_tail {
                            *link_mut(self.tail()) = link(ADJUST_HEAD);
                            *self.tail_mut() = self.adjust_tail;
                        }
                        self.adjust_tail = NULL;
                    }
                    if self.mode() > 0 {
                        self.build_page()?;
                    }
                }
                else {
                    if self.mode().abs() == HMODE {
                        *self.space_factor_mut() = 1000;
                    }
                    else {
                        let p = self.new_noad()?;
                        *math_type_mut(nucleus!(p)) = SUB_BOX;
                        *info_mut(nucleus!(p)) = self.cur_box;
                        self.cur_box = p;
                    }
                    *link_mut(self.tail()) = self.cur_box;
                    *self.tail_mut() = self.cur_box;
                }
            }
            // End section 1076
        }
        else if box_content < SHIP_OUT_FLAG {
            // Section 1077
            if box_content < BOX_FLAG + 256 {
                self.eq_define(BOX_BASE - BOX_FLAG + box_content, BOX_REF, self.cur_box)?;
            }
            else {
                self.geq_define(BOX_BASE - BOX_FLAG - 256 + box_content, BOX_REF, self.cur_box)?;
            }
            // End section 1077
        }
        else if self.cur_box != NULL {
            if box_content > SHIP_OUT_FLAG {
                // Section 1078
                sec404_get_next_nonblank_nonrelax_noncall_token!(self);
                if self.cur_cmd == HSKIP && self.mode().abs() != VMODE
                    || self.cur_cmd == VSKIP && self.mode().abs() == VMODE
                {
                    self.append_glue()?;
                    *subtype_mut(self.tail()) = (box_content - (LEADER_FLAG - (A_LEADERS as Integer))) as QuarterWord;
                    *leader_ptr_mut(self.tail()) = self.cur_box;
                }
                else {
                    return Err(TeXError::LeadersNotFollowedByProperGlue);
                }
                // End section 1078
            }
            else {
                self.ship_out(self.cur_box)?;
            }
        }
        Ok(())
    }

    // Section 1079
    pub(crate) fn begin_box(&mut self, box_content: Integer) -> TeXResult<()> {
        match self.cur_chr {
            BOX_CODE => {
                self.scan_eight_bit_int()?;
                self.cur_box = r#box(self.cur_val);
                *box_mut(self.cur_val) = NULL;
            },

            COPY_CODE => {
                self.scan_eight_bit_int()?;
                self.cur_box = self.copy_node_list(r#box(self.cur_val))?;
            },

            LAST_BOX_CODE => {
                // Section 1080
                self.cur_box = NULL;
                if self.mode().abs() == MMODE {
                    return Err(TeXError::CantUseIn);
                }
                if self.mode() == VMODE && self.head() == self.tail() {
                    return Err(TeXError::CantUseIn2);
                }
                if !self.is_char_node(self.tail())
                    && (r#type(self.tail()) == HLIST_NODE || r#type(self.tail()) == VLIST_NODE)
                {
                    // Section 1081
                    let mut q = self.head();
                    loop {
                        let mut p = q;
                        if !self.is_char_node(q) && r#type(q) == DISC_NODE {
                            for _ in 1..=replace_count(q) {
                                p = link(p);
                            }
                            if p == self.tail() {
                                break; // Goto done
                            }
                        }
                        q = link(p);
                        if q == self.tail() {
                            self.cur_box = self.tail();
                            *shift_amount_mut(self.cur_box) = 0;
                            *self.tail_mut() = p;
                            *link_mut(p) = NULL;    
                            break;
                        }
                    }
                    // done:
                    // End section 1081
                }
                // End section 1080
            },

            VSPLIT_CODE => {
                // Section 1082
                self.scan_eight_bit_int()?;
                let n = self.cur_val;
                if !self.scan_keyword(b"to")? {
                    return Err(TeXError::MissingTo);
                }
                self.scan_dimen(false, false, false)?;
                self.cur_box = self.vsplit(n as u8, self.cur_val)?;
                // End section 1082
            },

            _ => {
                // Section 1083
                let mut k = self.cur_chr - VTOP_CODE;
                *self.saved_mut(0) = box_content;
                if k == HMODE {
                    if box_content < BOX_FLAG && self.mode().abs() == VMODE {
                        self.scan_spec(ADJUSTED_HBOX_GROUP, true)?;
                    }
                    else {
                        self.scan_spec(HBOX_GROUP, true)?;
                    }
                }
                else {
                    if k == VMODE {
                        self.scan_spec(VBOX_GROUP, true)?;
                    }
                    else {
                        self.scan_spec(VTOP_GROUP, true)?;
                        k = VMODE;
                    }
                    self.normal_paragraph()?;
                }
                self.push_nest()?;
                *self.mode_mut() = -k;
                if k == VMODE {
                    *self.prev_depth_mut() = IGNORE_DEPTH;
                    if every_vbox() != NULL {
                        self.begin_token_list(every_vbox(), EVERY_VBOX_TEXT)?;
                    }
                }
                else {
                    *self.space_factor_mut() = 1000;
                    if every_hbox() != NULL {
                        self.begin_token_list(every_hbox(), EVERY_HBOX_TEXT)?;
                    }
                }
                return Ok(());
                // End section 1083
            }
        }
        self.box_end(box_content)
    }

    // Section 1084
    pub(crate) fn scan_box(&mut self, box_content: Integer) -> TeXResult<()> {
        sec404_get_next_nonblank_nonrelax_noncall_token!(self);
        if self.cur_cmd == MAKE_BOX {
            self.begin_box(box_content)
        }
        else if box_content >= LEADER_FLAG && (self.cur_cmd == HRULE || self.cur_cmd == VRULE) {
            self.cur_box = self.scan_rule_spec()?;
            self.box_end(box_content)
        }
        else {
            Err(TeXError::BoxWasSupposedToBeHere)
        }
    }

    // Section 1086
    pub(crate) fn package(&mut self, c: SmallNumber) -> TeXResult<()> {
        let d = box_max_depth();
        self.unsave()?;
        self.save_ptr -= 3;
        if self.mode() == -HMODE {
            self.cur_box = hpack!(self, link(self.head()), self.saved(2), self.saved(1) as SmallNumber)?;
        }
        else {
            self.cur_box = self.vpackage(link(self.head()), self.saved(2), self.saved(1) as SmallNumber, d)?;
            if c == VTOP_CODE as SmallNumber {
                // Section 1087
                let p = list_ptr(self.cur_box);
                let h = if p != NULL && r#type(p) <= RULE_NODE {
                    height(p)
                }
                else {
                    0
                };

                *depth_mut(self.cur_box) += - h + height(self.cur_box);
                *height_mut(self.cur_box) = h;
                // End section 1087
            }
        }
        self.pop_nest();
        self.box_end(self.saved(0))
    }
}

// Section 1091
pub fn norm_min(h: Integer) -> Integer {
    if h <= 0 {
        1
    }
    else if h >= 63 {
        63
    }
    else {
        h
    }
}

impl Global {
    // Section 1091
    pub(crate) fn new_graf(&mut self, indented: bool) -> TeXResult<()> {
        *self.prev_graf_mut() = 0;
        if self.mode() == VMODE || self.head() != self.tail() {
            tail_append!(self, self.new_param_glue(PAR_SKIP_CODE as QuarterWord)?);
        }
        self.push_nest()?;
        *self.mode_mut() = HMODE;
        *self.space_factor_mut() = 1000;
        self.set_cur_lang();
        *self.clang_mut() = self.cur_lang as HalfWord;
        *self.prev_graf_mut() = (norm_min(left_hyphen_min())*64 + norm_min(right_hyphen_min()))*65536 + self.cur_lang as Integer;
        if indented {
            *self.tail_mut() = self.new_null_box()?;
            *link_mut(self.head()) = self.tail();
            *width_mut(self.tail()) = par_indent();
        }
        if every_par() != NULL {
            self.begin_token_list(every_par(), EVERY_PAR_TEXT)?;
        }
        if self.nest_ptr == 1 {
            self.build_page()?
        }
        Ok(())
    }

    // Section 1093
    pub(crate) fn indent_in_hmode(&mut self) -> TeXResult<()> {
        if self.cur_chr > 0 {
            let mut p = self.new_null_box()?;
            *width_mut(p) = par_indent();
            if self.mode().abs() == HMODE {
                *self.space_factor_mut() = 1000;
            }
            else {
                let q = self.new_noad()?;
                *math_type_mut(nucleus!(q)) = SUB_BOX;
                *info_mut(nucleus!(q)) = p;
                p = q;
            }
            tail_append!(self, p);
        }
        Ok(())
    }

    // Section 1095
    pub(crate) fn head_for_vmode(&mut self) -> TeXResult<()> {
        if self.mode() < 0 {
            if self.cur_cmd != HRULE {
                self.off_save()
            }
            else {
                Err(TeXError::CantUseHrule)
            }
        }
        else {
            self.back_input()?;
            self.cur_tok = self.par_token;
            self.back_input()?;
            *self.token_type_mut() = INSERTED;
            Ok(())
        }
    }

    // Section 1096
    pub(crate) fn end_graf(&mut self) -> TeXResult<()> {
        if self.mode() == HMODE {
            if self.head() == self.tail() {
                self.pop_nest();
            }
            else {
                self.line_break(widow_penalty())?;
            }
            self.normal_paragraph()?;
            // no error_count
        }
        Ok(())
    }

    // Section 1099
    pub(crate) fn begin_insert_or_adjust(&mut self) -> TeXResult<()> {
        if self.cur_cmd == VADJUST {
            self.cur_val = 255;
        }
        else {
            self.scan_eight_bit_int()?;
            if self.cur_val == 255 {
                return Err(TeXError::CantInsert255);
            }
        }
        *self.saved_mut(0) = self.cur_val;
        self.save_ptr += 1;
        self.new_save_level(INSERT_GROUP)?;
        self.scan_left_brace()?;
        self.normal_paragraph()?;
        self.push_nest()?;
        *self.mode_mut() = -VMODE;
        *self.prev_depth_mut() = IGNORE_DEPTH;
        Ok(())
    }

    // Section 1101
    pub(crate) fn make_mark(&mut self) -> TeXResult<()> {
        let _ = self.scan_toks(false, true)?;
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = MARK_NODE;
        *subtype_mut(p) = 0;
        *mark_ptr_mut(p) = self.def_ref;
        *link_mut(self.tail()) = p;
        *self.tail_mut() = p;
        Ok(())
    }

    // Section 1103
    pub(crate) fn append_penalty(&mut self) -> TeXResult<()> {
        self.scan_int()?;
        tail_append!(self, self.new_penalty(self.cur_val)?);
        if self.mode() == VMODE {
            self.build_page()?;
        }
        Ok(())
    }

    // Section 1105
    pub(crate) fn delete_last(&mut self) -> TeXResult<()> {
        if self.mode() == VMODE && self.tail() == self.head() {
            // Section 1106
            if self.cur_chr != (GLUE_NODE as HalfWord) || self.last_glue != MAX_HALFWORD {
                return Err(TeXError::CantTakeThings);
            }
            // End section 1106
        }
        else if !self.is_char_node(self.tail())
            && (r#type(self.tail()) as HalfWord) == self.cur_chr
        {
            let mut q = self.head();
            let mut p: HalfWord;
            loop {
                p = q;
                if !self.is_char_node(q) && r#type(q) == DISC_NODE {
                    for _ in 1..=replace_count(q) {
                        p = link(p);
                    }
                    if p == self.tail() {
                        return Ok(())
                    }
                }
                q = link(p);
                if q == self.tail() {
                    break;
                }
            }
            *link_mut(p) = NULL;
            self.flush_node_list(self.tail())?;
            *self.tail_mut() = p;
        }
        Ok(())
    }

    // Section 1110
    pub(crate) fn unpackage(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        self.scan_eight_bit_int()?;
        let p = r#box(self.cur_val);
        if p == NULL {
            return Ok(())
        }
        if self.mode().abs() == MMODE
            || (self.mode().abs() == VMODE && r#type(p) != VLIST_NODE)
            || (self.mode().abs() == HMODE && r#type(p) != HLIST_NODE)
        {
            return Err(TeXError::IncompatibleListCantBeUnboxed);
        }
        if c == COPY_CODE {
            *link_mut(self.tail()) = self.copy_node_list(list_ptr(p))?;
        }
        else {
            *link_mut(self.tail()) = list_ptr(p);
            *box_mut(self.cur_val) = NULL;
            self.free_node(p, BOX_NODE_SIZE);
        }
        while link(self.tail()) != NULL {
            *self.tail_mut() = link(self.tail());
        }
        Ok(())
    }

    // Section 1113
    pub(crate) fn append_italic_correction(&mut self) -> TeXResult<()> {
        if self.tail() != self.head() {
            let p = if self.is_char_node(self.tail()) {
                self.tail()
            }
            else if r#type(self.tail()) == LIGATURE_NODE {
                lig_char!(self.tail())
            }
            else {
                return Ok(())
            };

            let f = font(p);
            let tmp = self.char_italic(f, self.char_info(f, character(p)));
            tail_append!(self, self.new_kern(tmp)?);
            *subtype_mut(self.tail()) = EXPLICIT;
        }
        Ok(())
    }

    // Section 1117
    pub(crate) fn append_discretionary(&mut self) -> TeXResult<()> {
        tail_append!(self, self.new_disc()?);
        if self.cur_chr == 1 {
            let c = self.hyphen_char[cur_font() as usize];
            if (0..=255).contains(&c) {
                *pre_break_mut(self.tail()) = self.new_character(cur_font() as QuarterWord, c as u8)?;
            }
        }
        else {
            self.save_ptr += 1;
            *self.saved_mut(-1) = 0;
            self.new_save_level(DISC_GROUP)?;
            self.scan_left_brace()?;
            self.push_nest()?;
            *self.mode_mut() = -HMODE;
            *self.space_factor_mut() = 1000;
        }
        Ok(())
    }

    // Section 1119
    pub(crate) fn build_discretionary(&mut self) -> TeXResult<()> {
        self.unsave()?;
        
        // Section 1121
        let mut q = self.head();
        let mut p = link(q);
        let mut n = 0;
        while p != NULL {
            if !self.is_char_node(p)
                && r#type(p) > RULE_NODE
                && r#type(p) != KERN_NODE
                && r#type(p) != LIGATURE_NODE
            {
                return Err(TeXError::ImproperDiscList);
            }
            q = p;
            p = link(q);
            n += 1;
        }
        // End section 1121
        
        p = link(self.head());
        self.pop_nest();
        match self.saved(-1) {
            0 => *pre_break_mut(self.tail()) = p,

            1 => *post_break_mut(self.tail()) = p,

            _ /* 2 */ => {
                // Section 1120
                if n > 0 && self.mode().abs() == MMODE {
                    return Err(TeXError::IllegalMathDisc);
                }
                *link_mut(self.tail()) = p;
                if n <= MAX_QUARTERWORD as Integer {
                    *replace_count_mut(self.tail()) = n as QuarterWord;
                }
                else {
                    return Err(TeXError::DiscListTooLong);
                }
                if n > 0 {
                    *self.tail_mut() = q;
                }
                self.save_ptr -= 1;
                return Ok(())
                // End section 1120
            }
        }
        *self.saved_mut(-1) += 1;
        self.new_save_level(DISC_GROUP)?;
        self.scan_left_brace()?;
        self.push_nest()?;
        *self.mode_mut() = -HMODE;
        *self.space_factor_mut() = 1000;
        Ok(())
    }

    // Section 1123
    pub(crate) fn make_accent(&mut self) -> TeXResult<()> {
        self.scan_char_num()?;
        let mut f = cur_font() as QuarterWord;
        let mut p = self.new_character(f, self.cur_val as u8)?;
        if p != NULL {
            let x = self.x_height(f);
            let s = self.slant(f) as Real / 65536.0;
            let a = self.char_width(f, self.char_info(f, character(p)));
            self.do_assignments()?;

            // Section 1124
            let mut q = NULL;
            f = cur_font() as QuarterWord;
            if self.cur_cmd == LETTER
                || self.cur_cmd == OTHER_CHAR
                || self.cur_cmd == CHAR_GIVEN
            {
                q = self.new_character(f, self.cur_chr as u8)?;
            }
            else if self.cur_cmd == CHAR_NUM {
                self.scan_char_num()?;
                q = self.new_character(f, self.cur_val as u8)?;
            }
            else {
                self.back_input()?;
            }
            // End section 1124

            if q != NULL {
                // Section 1125
                let t = self.slant(f) as Real / 65536.0;
                let i = self.char_info(f, character(q));
                let w = self.char_width(f, i);
                let h = self.char_height(f, i.height_depth());
                if h != x {
                    p = hpack!(self, p, NATURAL)?;
                    *shift_amount_mut(p) = x - h;
                }
                let delta = ((w - a) as Real / 2.0 + (h as Real)*t - (x as Real)*s).round() as Scaled;
                let r = self.new_kern(delta)?;
                *subtype_mut(r) = ACC_KERN;
                *link_mut(self.tail()) = r;
                *link_mut(r) = p;
                *self.tail_mut() = self.new_kern(-a - delta)?;
                *subtype_mut(self.tail()) = ACC_KERN;
                *link_mut(p) = self.tail();
                p = q;
                // End section 1125
            }
            *link_mut(self.tail()) = p;
            *self.tail_mut() = p;
            *self.space_factor_mut() = 1000;
        }
        Ok(())
    }

    // Section 1127
    pub(crate) fn align_error(&mut self) -> TeXResult<()> {
        if self.align_state.abs() > 2 {
            Err(TeXError::MisplacedTabMark)
        }
        else {
            self.back_input()?;
            if self.align_state < 0 {
                Err(TeXError::MissingLeftBrace3)
            }
            else {
                Err(TeXError::MissingRightBrace2)
            }
        }
    }

    // Section 1131
    pub(crate) fn do_endv(&mut self) -> TeXResult<()> {
        self.base_ptr = self.input_ptr;
        self.input_stack[self.base_ptr] = self.cur_input;
        while self.input_stack[self.base_ptr].index_field != V_TEMPLATE
            && self.input_stack[self.base_ptr].loc_field == NULL
            && self.input_stack[self.base_ptr].state_field == TOKEN_LIST
        {
            self.base_ptr -= 1;
        }

        if self.input_stack[self.base_ptr].index_field != V_TEMPLATE
            || self.input_stack[self.base_ptr].loc_field != NULL
            || self.input_stack[self.base_ptr].state_field != TOKEN_LIST
        {
            return Err(TeXError::Fatal("(interwoven alignment preambles are not allowed)"));
        }

        if self.cur_group == ALIGN_GROUP {
            self.end_graf()?;
            if self.fin_col()? {
                self.fin_row()?;
            }
        }
        else {
            self.off_save()?
        }
        Ok(())
    }
}
