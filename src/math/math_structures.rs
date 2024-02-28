use crate::constants::{
    CHOICE_NODE, NOAD_SIZE, NORMAL, NULL, ORD_NOAD, STYLE_NODE, STYLE_NODE_SIZE
};
use crate::datastructures::{
    MEM, depth_mut, font, font_mut, info, info_mut, link, link_mut,
    subtype_mut, type_mut, width, width_mut
};
use crate::error::TeXResult;
use crate::{
    Global, HalfWord, QuarterWord, Scaled, SmallNumber, mem, mem_mut
};

// Part 34: Data structures for math mode

// Section 681
#[macro_export]
macro_rules! nucleus {
    ($p:expr) => {
        $p + 1
    };
}

#[macro_export]
macro_rules! supscr {
    ($p:expr) => {
        $p + 2
    };
}

#[macro_export]
macro_rules! subscr {
    ($p:expr) => {
        $p + 3
    };
}

// Section 681
pub(crate) fn math_type(p: HalfWord) -> HalfWord {
    link(p)
}

pub(crate) fn math_type_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p)
}

pub(crate) fn fam(p: HalfWord) -> QuarterWord {
    font(p)
}

pub(crate) fn fam_mut(p: HalfWord) -> &'static mut QuarterWord {
    font_mut(p)
}

// Section 683
#[macro_export]
macro_rules! left_delimiter {
    ($p:expr) => {
        $p + 4
    };
}

#[macro_export]
macro_rules! right_delimiter {
    ($p:expr) => {
        $p + 5
    };
}

#[macro_export]
macro_rules! numerator {
    ($p:expr) => {
        supscr!($p)
    };
}

#[macro_export]
macro_rules! denominator {
    ($p:expr) => {
        subscr!($p)
    };
}

// Section 683
pub(crate) fn small_fam(p: HalfWord) -> QuarterWord {
    mem![p as usize].qqqq_b0()
}

pub(crate) fn small_fam_mut(p: HalfWord) -> &'static mut QuarterWord {
    mem_mut![p as usize].qqqq_b0_mut()
}

pub(crate) fn small_char(p: HalfWord) -> QuarterWord {
    mem![p as usize].qqqq_b1()
}

pub(crate) fn small_char_mut(p: HalfWord) -> &'static mut QuarterWord {
    mem_mut![p as usize].qqqq_b1_mut()
}

pub(crate) fn large_fam(p: HalfWord) -> QuarterWord {
    mem![p as usize].qqqq_b2()
}

pub(crate) fn large_fam_mut(p: HalfWord) -> &'static mut QuarterWord {
    mem_mut![p as usize].qqqq_b2_mut()
}

pub(crate) fn large_char(p: HalfWord) -> QuarterWord {
    mem![p as usize].qqqq_b3()
}

pub(crate) fn large_char_mut(p: HalfWord) -> &'static mut QuarterWord {
    mem_mut![p as usize].qqqq_b3_mut()
}

pub(crate) fn thickness(p: HalfWord) -> Scaled {
    width(p)
}

pub(crate) fn thickness_mut(p: HalfWord) -> &'static mut Scaled {
    width_mut(p)
}

// Section 687
#[macro_export]
macro_rules! accent_chr {
    ($p:expr) => {
        $p + 4
    };
}

#[macro_export]
macro_rules! delimiter {
    ($p:expr) => {
        nucleus!($p)
    };
}

#[macro_export]
macro_rules! scripts_allowed {
    ($p:expr) => {
        r#type($p) >= ORD_NOAD && r#type($p) < LEFT_NOAD
    };
}

// Section 689
pub(crate) fn display_mlist(p: HalfWord) -> HalfWord {
    info(p + 1)
}

pub(crate) fn display_mlist_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 1)
}

pub(crate) fn text_mlist(p: HalfWord) -> HalfWord {
    link(p + 1)
}

pub(crate) fn text_mlist_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 1)
}

pub(crate) fn script_mlist(p: HalfWord) -> HalfWord {
    info(p + 2)
}

pub(crate) fn script_mlist_mut(p: HalfWord) -> &'static mut HalfWord {
    info_mut(p + 2)
}

pub(crate) fn script_script_mlist(p: HalfWord) -> HalfWord {
    link(p + 2)
}

pub(crate) fn script_script_mlist_mut(p: HalfWord) -> &'static mut HalfWord {
    link_mut(p + 2)
}

impl Global {
    // Section 686
    pub(crate) fn new_noad(&mut self) -> TeXResult<HalfWord> {
        let p = self.get_node(NOAD_SIZE)?;
        *type_mut(p) = ORD_NOAD;
        *subtype_mut(p) = NORMAL;
        *mem_mut![nucleus!(p) as usize] = self.empty_field;
        *mem_mut![subscr!(p) as usize] = self.empty_field;
        *mem_mut![supscr!(p) as usize] = self.empty_field;
        Ok(p)
    }

    // Section 688
    pub(crate) fn new_style(&mut self, s: SmallNumber) -> TeXResult<HalfWord> {
        let p = self.get_node(STYLE_NODE_SIZE)?;
        *type_mut(p) = STYLE_NODE;
        *subtype_mut(p) = s;
        *width_mut(p) = 0;
        *depth_mut(p) = 0;
        Ok(p)
    }

    // Section 689
    pub(crate) fn new_choice(&mut self) -> TeXResult<HalfWord> {
        let p = self.get_node(STYLE_NODE_SIZE)?;
        *type_mut(p) = CHOICE_NODE;
        *subtype_mut(p) = 0;
        *display_mlist_mut(p) = NULL;
        *text_mlist_mut(p) = NULL;
        *script_mlist_mut(p) = NULL;
        *script_script_mlist_mut(p) = NULL;
        Ok(p)
    }
}
