use crate::constants::*;
use crate::datastructures::{
    MEM, glue_par, info, info_mut, link, link_mut, llink, llink_mut,
    rlink, rlink_mut
};
use crate::error::TeXResult;
use crate::{
    Global, GlueRatio, HalfWord, Integer, QuarterWord, Scaled, SmallNumber,
    mem, mem_mut
};

// Part 10: Data structures for boxes and their friends

// Section 133
pub(crate) fn r#type(p: HalfWord) -> QuarterWord {
    mem![p as usize].qqqq_b0()
}

pub(crate) fn type_mut(p: HalfWord) -> &'static mut QuarterWord {
    mem_mut![p as usize].qqqq_b0_mut()
}

pub(crate) fn subtype(p: HalfWord) -> QuarterWord {
    mem![p as usize].qqqq_b1()
}

pub(crate) fn subtype_mut(p: HalfWord) -> &'static mut QuarterWord {
    mem_mut![p as usize].qqqq_b1_mut()
}

// Section 134
impl Global {
    pub(crate) fn is_char_node(&self, p: HalfWord) -> bool {
        p >= self.hi_mem_min
    }
}

pub(crate) fn font(p: HalfWord) -> QuarterWord {
    r#type(p)
}

pub(crate) fn font_mut(p: HalfWord) -> &'static mut QuarterWord {
    type_mut(p)
}

pub(crate) fn character(p: HalfWord) -> QuarterWord {
    subtype(p)
}

pub(crate) fn character_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p)
}

// Section 135
pub(crate) fn width(p: HalfWord) -> Scaled {
    mem![(p + WIDTH_OFFSET) as usize].sc()
}

pub(crate) fn width_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![(p + WIDTH_OFFSET) as usize].sc_mut()
}

pub(crate) fn depth(p: HalfWord) -> Scaled {
    mem![(p + DEPTH_OFFSET) as usize].sc()
}

pub(crate) fn depth_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![(p + DEPTH_OFFSET) as usize].sc_mut()
}

pub(crate) fn height(p: HalfWord) -> Scaled {
    mem![(p + HEIGHT_OFFSET) as usize].sc()
}

pub(crate) fn height_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![(p + HEIGHT_OFFSET) as usize].sc_mut()
}

pub(crate) fn shift_amount(p: HalfWord) -> Scaled {
    mem![p as usize + 4].sc()
}

pub(crate) fn shift_amount_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![p as usize + 4].sc_mut()
}

pub(crate) fn list_ptr(p: HalfWord) -> HalfWord {
    link(p + LIST_OFFSET)
}

pub(crate) fn list_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + LIST_OFFSET)
}

pub(crate) fn glue_order(p: HalfWord) -> QuarterWord {
    subtype(p + LIST_OFFSET)
}

pub(crate) fn glue_order_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p + LIST_OFFSET)
}

pub(crate) fn glue_sign(p: HalfWord) -> QuarterWord {
    r#type(p + LIST_OFFSET)
}

pub(crate) fn glue_sign_mut(p: HalfWord) -> &'static mut QuarterWord {
    type_mut(p + LIST_OFFSET)
}

pub(crate) fn glue_set(p: HalfWord) -> GlueRatio {
    mem![(p + GLUE_OFFSET) as usize].gr()
}

pub(crate) fn glue_set_mut(p: HalfWord) -> &'static mut GlueRatio {
    mem_mut![(p + GLUE_OFFSET) as usize].gr_mut()
}

// Section 138
#[macro_export]
macro_rules! is_running {
    ($d:expr) => {
        $d == NULL_FLAG
    };
}

impl Global {
    // Section 136
     pub(crate) fn new_null_box(&mut self) -> TeXResult<HalfWord> {
        let p = self.get_node(BOX_NODE_SIZE)?;
        *type_mut(p) = HLIST_NODE;
        *subtype_mut(p) = MIN_QUARTERWORD;
        *width_mut(p) = 0;
        *depth_mut(p) = 0;
        *height_mut(p) = 0;
        *shift_amount_mut(p) = 0;
        *list_ptr_mut(p) =  NULL;
        *glue_sign_mut(p) = NORMAL;
        *glue_order_mut(p) = NORMAL;
        *glue_set_mut(p) = 0.0;
        Ok(p)
    }

    // Section 139
    pub(crate) fn new_rule(&mut self) -> TeXResult<HalfWord> {
        let p = self.get_node(RULE_NODE_SIZE)?;
        *type_mut(p) = RULE_NODE;
        *subtype_mut(p) = 0;
        *width_mut(p) = NULL_FLAG;
        *depth_mut(p) = NULL_FLAG;
        *height_mut(p) = NULL_FLAG;
        Ok(p)
    }
}

// Section 140
pub(crate) fn float_cost(p: HalfWord) -> Integer {
    mem![(p + 1) as usize].int()
}

pub(crate) fn float_cost_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut![(p + 1) as usize].int_mut()
}

pub(crate) fn ins_ptr(p: HalfWord) -> HalfWord {
    info(p + 4)
}

pub(crate) fn ins_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 4)
}

pub(crate) fn split_top_ptr(p: HalfWord) -> HalfWord {
    link(p + 4)
}

pub(crate) fn split_top_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 4)
}

// Section 141
pub(crate) fn mark_ptr(p: HalfWord) -> Integer {
    mem![(p + 1) as usize].int()
}

pub(crate) fn mark_ptr_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut![(p + 1) as usize].int_mut()
}

// Section 142
pub(crate) fn adjust_ptr(p: HalfWord) -> Integer {
    mark_ptr(p)
}

pub(crate) fn adjust_ptr_mut(p: HalfWord) -> &'static mut Integer {
    mark_ptr_mut(p)
}

// Section 143
#[macro_export]
macro_rules! lig_char {
    ($p:expr) => {
        $p + 1
    };
}

// Section 143
pub(crate) fn lig_ptr(p: HalfWord) -> HalfWord {
    link(lig_char!(p))
}

pub(crate) fn lig_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(lig_char!(p))
}

impl Global {
    // Section 144
    pub(crate) fn new_ligature(&mut self, f: QuarterWord, c: QuarterWord, q: HalfWord) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = LIGATURE_NODE;
        *font_mut(lig_char!(p)) = f;
        *character_mut(lig_char!(p)) = c;
        *lig_ptr_mut(p) = q;
        *subtype_mut(p) = 0;
        Ok(p)
    }

    pub(crate) fn new_lig_item(&mut self, c: QuarterWord) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *character_mut(p) = c;
        *lig_ptr_mut(p) = NULL;
        Ok(p)
    }
}

// Section 145
pub(crate) fn replace_count(p: HalfWord) -> QuarterWord {
    subtype(p)
}

pub(crate) fn replace_count_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p)
}

pub(crate) fn pre_break(p: HalfWord) -> HalfWord {
    llink(p)
}

pub(crate) fn pre_break_mut(p: HalfWord) -> &'static mut HalfWord {
    llink_mut(p)
}

pub(crate) fn post_break(p: HalfWord) -> HalfWord {
    rlink(p)
}

pub(crate) fn post_break_mut(p: HalfWord) -> &'static mut HalfWord {
    rlink_mut(p)
}

impl Global {
    pub(crate) fn new_disc(&mut self) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = DISC_NODE;
        *replace_count_mut(p) = 0;
        *pre_break_mut(p) = NULL;
        *post_break_mut(p) = NULL;
        Ok(p)
    }

    // Section 147
    pub(crate) fn new_math(&mut self, w: Scaled, s: SmallNumber) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = MATH_NODE;
        *subtype_mut(p) = s;
        *width_mut(p) = w;
        Ok(p)
    }
}

// Section 148
#[macro_export]
macro_rules! precedes_break {
    ($p:expr) => {
        r#type($p) < MATH_NODE
    };
}

#[macro_export]
macro_rules! non_discardable {
    ($p:expr) => {
        r#type($p) < MATH_NODE
    };
}

// Section 149
pub(crate) fn glue_ptr(p: HalfWord) -> HalfWord {
    llink(p)
}

pub(crate) fn glue_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    llink_mut(p)
}

pub(crate) fn leader_ptr(p: HalfWord) -> HalfWord {
    rlink(p)
}

pub(crate) fn leader_ptr_mut(p: HalfWord) -> &'static mut HalfWord {
    rlink_mut(p)
}

// Section 150
pub(crate) fn glue_ref_count(p: HalfWord) -> HalfWord {
    link(p)
}

pub(crate) fn glue_ref_count_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p)
}

pub(crate) fn stretch( p: HalfWord) -> Scaled {
    mem![(p + 2) as usize].sc()
}

pub(crate) fn stretch_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![(p + 2) as usize].sc_mut()
}

pub(crate) fn shrink(p: HalfWord) -> Scaled {
    mem![(p + 3) as usize].sc()
}

pub(crate) fn shrink_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![(p + 3) as usize].sc_mut()
}

pub(crate) fn stretch_order(p: HalfWord) -> QuarterWord {
    r#type(p)
}

pub(crate) fn stretch_order_mut(p: HalfWord) -> &'static mut QuarterWord {
    type_mut(p)
}

pub(crate) fn shrink_order(p: HalfWord) -> QuarterWord {
    subtype(p)
}

pub(crate) fn shrink_order_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p)
}

impl Global {
     // Section 151
     pub(crate) fn new_spec(&mut self, p: HalfWord) -> TeXResult<HalfWord> {
        let q = self.get_node(GLUE_SPEC_SIZE)?;
        *mem_mut![q as usize] = mem![p as usize];
        *glue_ref_count_mut(q) = NULL;
        *width_mut(q) = width(p);
        *stretch_mut(q) = stretch(p);
        *shrink_mut(q) = shrink(p);
        Ok(q)
    }

    // Section 152
    pub(crate) fn new_param_glue(&mut self, n: SmallNumber) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = GLUE_NODE;
        *subtype_mut(p) = n + 1;
        *leader_ptr_mut(p) = NULL;
        let q = glue_par(n as HalfWord);
        *glue_ptr_mut(p) = q;
        *glue_ref_count_mut(q) += 1;
        Ok(p)
    }

    // Section 153
    pub(crate) fn new_glue(&mut self, q: HalfWord) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = GLUE_NODE;
        *subtype_mut(p) = NORMAL;
        *leader_ptr_mut(p) = NULL;
        *glue_ptr_mut(p) = q;
        *glue_ref_count_mut(q) += 1;
        Ok(p)
    }

    // Section 154
    pub(crate) fn new_skip_param(&mut self, n: SmallNumber) -> TeXResult<HalfWord> {
        self.temp_ptr = self.new_spec(glue_par(n as HalfWord))?;
        let p = self.new_glue(self.temp_ptr)?;
        *glue_ref_count_mut(self.temp_ptr) = NULL;
        *subtype_mut(p) = n + 1;
        Ok(p)
    }

    // Section 156
    pub(crate) fn new_kern(&mut self, w: Scaled) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = KERN_NODE;
        *subtype_mut(p) = NORMAL;
        *width_mut(p) = w;
        Ok(p)
    }
}

// Section 157
pub(crate) fn penalty(p: HalfWord) -> Integer {
    mem![(p + 1) as usize].int()
}

pub(crate) fn penalty_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut![(p + 1) as usize].int_mut()
}

impl Global {
    // Section 158
    pub(crate) fn new_penalty(&mut self, m: Integer) -> TeXResult<HalfWord> {
        let p = self.get_node(SMALL_NODE_SIZE)?;
        *type_mut(p) = PENALTY_NODE;
        *subtype_mut(p) = 0;
        *penalty_mut(p) = m;
        Ok(p)
    }
}

// Section 159
pub(crate) fn glue_stretch(p: HalfWord) -> Scaled {
    mem![(p + GLUE_OFFSET) as usize].sc()
}

pub(crate) fn glue_stretch_mut(p: HalfWord) -> &'static mut Scaled {
    mem_mut![(p + GLUE_OFFSET) as usize].sc_mut()
}

pub(crate) fn glue_shrink(p: HalfWord) -> Scaled {
    shift_amount(p)
}

pub(crate) fn glue_shrink_mut(p: HalfWord) -> &'static mut Scaled {
    shift_amount_mut(p)
}

pub(crate) fn span_count(p: HalfWord) -> QuarterWord {
    subtype(p)
}

pub(crate) fn span_count_mut(p: HalfWord) -> &'static mut QuarterWord {
    subtype_mut(p)
}
