use crate::constants::NEST_SIZE;
use crate::datastructures::{MemoryWord, link_mut};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, Scaled, free_avail
};

// Part 16: The semantic nest

impl Global {
    // Section 213
    pub(crate) fn mode(&self) -> Integer {
        self.cur_list.mode_field
    }

    pub(crate) fn mode_mut(&mut self) -> &mut Integer {
        &mut self.cur_list.mode_field
    }

    pub(crate) fn head(&self) -> HalfWord {
        self.cur_list.head_field
    }

    pub(crate) fn head_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_list.head_field
    }

    pub(crate) fn tail(&self) -> HalfWord {
        self.cur_list.tail_field
    }

    pub(crate) fn tail_mut(&mut self) -> &mut HalfWord {
        &mut self.cur_list.tail_field
    }

    pub(crate) fn prev_graf(&self) -> Integer {
        self.cur_list.pg_field
    }

    pub(crate) fn prev_graf_mut(&mut self) -> &mut Integer {
        &mut self.cur_list.pg_field
    }

    pub(crate) fn aux(&self) -> MemoryWord {
        self.cur_list.aux_field
    }

    pub(crate) fn aux_mut(&mut self) -> &mut MemoryWord {
        &mut self.cur_list.aux_field
    }

    pub(crate) fn prev_depth(&self) -> Scaled {
        self.aux().sc()
    }

    pub(crate) fn prev_depth_mut(&mut self) -> &mut Scaled {
        self.aux_mut().sc_mut()
    }

    pub(crate) fn space_factor(&self) -> HalfWord {
        self.aux().hh_lh()
    }

    pub(crate) fn space_factor_mut(&mut self) -> &mut HalfWord {
        self.aux_mut().hh_lh_mut()
    }

    pub(crate) fn clang(&self) -> HalfWord {
        self.aux().hh_rh()
    }

    pub(crate) fn clang_mut(&mut self) -> &mut HalfWord {
        self.aux_mut().hh_rh_mut()
    }

    pub(crate) fn incomplete_noad(&self) -> Integer {
        self.aux().int()
    }

    pub(crate) fn incomplete_noad_mut(&mut self) -> &mut Integer {
        self.aux_mut().int_mut()
    }

    pub(crate) fn mode_line(&self) -> Integer {
        self.cur_list.ml_field
    }

    pub(crate) fn mode_line_mut(&mut self) -> &mut Integer {
        &mut self.cur_list.ml_field
    }
}

// Section 214
#[macro_export]
macro_rules! tail_append {
    ($s:ident, $p:expr) => {
        {
            *link_mut($s.tail()) = $p;
            *$s.tail_mut() = link($s.tail());
        }
    };
}

impl Global {
    // Section 216
    pub(crate) fn push_nest(&mut self) -> TeXResult<()> {
        if self.nest_ptr > self.max_nest_stack {
            self.max_nest_stack = self.nest_ptr;
            if self.nest_ptr == NEST_SIZE as usize {
                return Err(TeXError::Overflow("semantic nest size", NEST_SIZE))
            }
        }
        self.nest[self.nest_ptr] = self.cur_list;
        self.nest_ptr += 1;
        *self.head_mut() = self.get_avail()?;
        *self.tail_mut() = self.head();
        *self.prev_graf_mut() = 0;
        *self.mode_line_mut() = self.line;
        Ok(())
    }

    // Section 217
    pub(crate) fn pop_nest(&mut self) {
        free_avail!(self, self.head());
        self.nest_ptr -= 1;
        self.cur_list = self.nest[self.nest_ptr];
    }
}

// Section 212
#[derive(Default, Clone, Copy)]
pub(crate) struct ListStateRecord {
    pub(crate) mode_field: Integer, // -MMODE..MMODE
    pub(crate) head_field: HalfWord,
    pub(crate) tail_field: HalfWord,
    pub(crate) pg_field: Integer,
    pub(crate) ml_field: Integer,
    pub(crate) aux_field: MemoryWord
}
