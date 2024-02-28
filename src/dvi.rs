use crate::constants::*;
use crate::datastructures::{
    MEM, character, count, day, depth, font, glue_order, glue_ptr,
    glue_set, glue_sign, h_offset, height, info, info_mut, leader_ptr,
    link, link_mut, list_ptr, mag, month, r#type, shift_amount, shrink,
    shrink_order, stretch, stretch_order, subtype, time, tracing_output,
    v_offset, width, width_mut, year
};
use crate::error::{TeXError, TeXResult};
use crate::extensions::{
    open_area, open_ext, open_name, write_stream, write_tokens
};
use crate::io::{AlphaFileOutSelector, ByteFileOutSelector};
use crate::strings::{
    POOL, cur_length, length, pool_ptr, pool_ptr_mut, str_pool_slice,
    str_ptr, str_room, str_start
};

use crate::{
    Global, HalfWord, Integer, QuarterWord, Real, Scaled, StrNum,
    ins_list, is_running, lig_char, mem, mem_mut, str_pool, update_terminal
};

use std::io::Write;

#[cfg(feature = "stat")]
use crate::datastructures::tracing_stats;

// Part 32: Shipping pages out

impl Global {
    // Section 597
    fn write_dvi(&mut self, a: usize, b: usize) {
        self.dvi_file.write(&self.dvi_buf[a..=b]);
    }

    // Section 598
    fn dvi_swap(&mut self) {
        if self.dvi_limit == DVI_BUF_SIZE as usize {
            self.write_dvi(0, self.half_buf - 1);
            self.dvi_limit = self.half_buf;
            self.dvi_offset += DVI_BUF_SIZE;
            self.dvi_ptr = 0;
        }
        else {
            self.write_dvi(self.half_buf, (DVI_BUF_SIZE - 1) as usize);
            self.dvi_limit = DVI_BUF_SIZE as usize;
        }
        self.dvi_gone += self.half_buf as Integer;
    }
}

// Section 598
macro_rules! dvi_out {
    ($s:ident, $x:expr) => {
        $s.dvi_buf[$s.dvi_ptr] = $x;
        $s.dvi_ptr += 1;
        if $s.dvi_ptr == $s.dvi_limit {
            $s.dvi_swap();
        }
    };
}

impl Global {
    // Section 600
    fn dvi_four(&mut self, mut x: Integer) {
        if x >= 0 {
            dvi_out!(self, (x / 0x100_0000) as u8);
        }
        else {
            x += 0x4000_0000;
            x += 0x4000_0000;
            dvi_out!(self, ((x / 0x100_0000) + 128) as u8);
        }
        x %= 0x100_0000;
        dvi_out!(self, (x / 65536) as u8);
        x %= 65536;
        dvi_out!(self, (x / 256) as u8);
        dvi_out!(self, (x % 256) as u8);
    }

    // Section 601
    fn dvi_pop(&mut self, l: Integer) {
        if l == self.dvi_offset + (self.dvi_ptr as Integer)
            && self.dvi_ptr > 0
        {
            self.dvi_ptr -= 1;
        }
        else {
            dvi_out!(self, POP);
        }
    }

    // Section 602
    fn dvi_font_def(&mut self, f: QuarterWord) {
        dvi_out!(self, FNT_DEF1);
        dvi_out!(self, (f as Integer - FONT_BASE - 1) as u8);
        dvi_out!(self, self.font_check[f as usize].qqqq_b0() as u8);
        dvi_out!(self, self.font_check[f as usize].qqqq_b1() as u8);
        dvi_out!(self, self.font_check[f as usize].qqqq_b2() as u8);
        dvi_out!(self, self.font_check[f as usize].qqqq_b3() as u8);
        self.dvi_four(self.font_size[f as usize]);
        self.dvi_four(self.font_dsize[f as usize]);
        
        let len_area = length(self.font_area[f as usize]);
        let len_name = length(self.font_name[f as usize]);
        dvi_out!(self, len_area as u8);
        dvi_out!(self, len_name as u8);

        // Section 603
        let s = self.font_area[f as usize];
        for b in str_pool_slice(s) {
            dvi_out!(self, *b);
        }
        let s = self.font_name[f as usize];
        for b in str_pool_slice(s) {
            dvi_out!(self, *b);
        }
        // End section 603
    }
}

// Section 605
fn location(p: HalfWord) -> Integer {
    mem![(p + 2) as usize].int()
}

fn location_mut(p: HalfWord) -> &'static mut Integer {
    mem_mut![(p + 2) as usize].int_mut()
}

impl Global {
    // Section 607
    fn movement(&mut self, mut w: Scaled, o: u8) -> TeXResult<()> {
        let mut q = self.get_node(MOVEMENT_NODE_SIZE)?;
        *width_mut(q) = w;
        *location_mut(q) = self.dvi_offset + (self.dvi_ptr as Integer);
        if o == DOWN1 {
            *link_mut(q) = self.down_ptr;
            self.down_ptr = q;
        }
        else {
            *link_mut(q) = self.right_ptr;
            self.right_ptr = q;
        }

        if let Some(p) = self.sec611_look_at_the_other(w, q) {
            // found:
            // Section 609
            *info_mut(q) = info(p);
            if info(q) == Y_HERE {
                dvi_out!(self, o + (Y0 - DOWN1));
                while link(q) != p {
                    q = link(q);
                    match info(q) {
                        YZ_OK => *info_mut(q) = Z_OK,
                        Y_OK => *info_mut(q) = D_FIXED,
                        _ => (), // Do nothing
                    }
                }
            }
            else {
                dvi_out!(self, o + (Z0 - DOWN1));
                while link(q) != p {
                    q = link(q);
                    match info(q){
                        YZ_OK => *info_mut(q) = Y_OK,
                        Z_OK => *info_mut(q) = D_FIXED,
                        _ => (), // Do nothing
                    }
                }
            }
            // End section 609
        }
        else {
            // not_found:
            // Section 610
            *info_mut(q) = YZ_OK;
            if w.abs() >= 0x80_0000 {
                dvi_out!(self, o + 3);
                self.dvi_four(w);
                return Ok(())
            }
            let goto = 'block: {
                if w.abs() >= 32768 {
                    dvi_out!(self, o + 2);
                    if w < 0 {
                        w += 0x100_0000;
                    }
                    dvi_out!(self, (w / 65536) as u8);
                    w %= 65536;
                    break 'block 2; // Goto 2
                }
                if w.abs() >= 128 {
                    dvi_out!(self, o + 1);
                    if w < 0 {
                        w += 65536;
                    }
                    break 'block 2; // Goto 2
                }
                dvi_out!(self, o);
                if w < 0 {
                    w += 256;
                }
                1
            };

            if goto == 2 {
                dvi_out!(self, (w / 256) as u8);
            }
            dvi_out!(self, (w % 256) as u8);
            // End section 610
        }
        Ok(())
    }

    // Section 611
    fn sec611_look_at_the_other(&mut self, w: Scaled, q: HalfWord) -> Option<HalfWord> {
        let mut p = link(q);
        let mut mstate = NONE_SEEN;
        while p != NULL {
            if width(p) == w {
                // Section 612
                match self.sec612_consider_a_node(p, mstate) {
                    OptionSec612::Continue => (), // Do nothing, stay in the loop
                    OptionSec612::Found => return Some(p),
                    OptionSec612::NotFound => break, // Goto not_found
                }
                // End section 612
            }
            else {
                match (mstate, info(p)) {
                    (NONE_SEEN, Y_HERE) => mstate = Y_SEEN,
                    (NONE_SEEN, Z_HERE) => mstate = Z_SEEN,
                    (Y_SEEN, Z_HERE)
                    | (Z_SEEN, Y_HERE) => break, // Goto not_found
                    _ => () // Do nothing
                }
            }
            p = link(p);
        }
        // not_found:
        None
    }
}

enum OptionSec612 {
    Found,
    NotFound,
    Continue
}

impl Global {
    // Section 612
    fn sec612_consider_a_node(&mut self, p: HalfWord, mstate: Integer) -> OptionSec612 {
        match (mstate, info(p)) {
            (NONE_SEEN, YZ_OK)
            | (NONE_SEEN, Y_OK)
            | (Z_SEEN, YZ_OK)
            | (Z_SEEN, Y_OK) => {
                if location(p) < self.dvi_gone {
                    OptionSec612::NotFound
                }
                else {
                    // Section 613
                    let mut k = location(p) - self.dvi_offset;
                    if k < 0 {
                        k += DVI_BUF_SIZE;
                    }
                    self.dvi_buf[k as usize] += Y1 - DOWN1;
                    *info_mut(p) = Y_HERE;
                    OptionSec612::Found
                    // End section 613
                }
            },

            (NONE_SEEN, Z_OK)
            | (Y_SEEN, YZ_OK)
            | (Y_SEEN, Z_OK) => {
                if location(p) < self.dvi_gone {
                    OptionSec612::NotFound
                }
                else {
                    // Section 614
                    let mut k = location(p) - self.dvi_offset;
                    if k < 0 {
                        k += DVI_BUF_SIZE;
                    }
                    self.dvi_buf[k as usize] += Z1 - DOWN1;
                    *info_mut(p) = Z_HERE;
                    OptionSec612::Found
                    // End section 614
                }
            },

            (NONE_SEEN, Y_HERE)
            | (NONE_SEEN, Z_HERE)
            | (Y_SEEN, Z_HERE)
            | (Z_SEEN, Y_HERE) => OptionSec612::Found,

            _ => OptionSec612::Continue, // Do nothing
        }
    }

    // Section 615
    fn prune_movements(&mut self, l: Integer) {
        while self.down_ptr != NULL {
            if location(self.down_ptr) < l {
                break; // Goto done
            }
            let p = self.down_ptr;
            self.down_ptr = link(p);
            self.free_node(p, MOVEMENT_NODE_SIZE);
        }

        // done:
        while self.right_ptr != NULL {
            if location(self.right_ptr) < l {
                break; // Exit
            }
            let p = self.right_ptr;
            self.right_ptr = link(p);
            self.free_node(p, MOVEMENT_NODE_SIZE);
        }
    }
}

// Section 616
macro_rules! synch_h {
    ($s:ident) => {
        if $s.cur_h != $s.dvi_h {
            $s.movement($s.cur_h - $s.dvi_h, RIGHT1)?;
            $s.dvi_h = $s.cur_h;
        }
    };
}

macro_rules! synch_v {
    ($s:ident) => {
        if $s.cur_v != $s.dvi_v {
            $s.movement($s.cur_v - $s.dvi_v, DOWN1)?;
            $s.dvi_v = $s.cur_v;
        }
    };
}

// Section 625
macro_rules! billion {
    () => {
        1_000_000_000.0
    };
}

macro_rules! vet_glue {
    ($g:expr) => {
        if $g > billion!() {
            billion!()
        }
        else if $g < -billion!() {
            -billion!()
        }
        else {
            $g
        }
    };
}

enum Goto {
    FinRule,
    MovePast,
    NextP
}

impl Global {
    // Section 619
    fn hlist_out(&mut self) -> TeXResult<()> {
        let mut cur_g = 0;
        let mut cur_glue = 0.0;
        let this_box = self.temp_ptr;
        let g_order = glue_order(this_box);
        let g_sign = glue_sign(this_box);
        let mut p = list_ptr(this_box);
        self.cur_s += 1;
        if self.cur_s > 0 {
            dvi_out!(self, PUSH);
        }
        if self.cur_s > self.max_push {
            self.max_push = self.cur_s;
        }
        let save_loc = self.dvi_offset + (self.dvi_ptr as Integer);
        let base_line = self.cur_v;
        let left_edge = self.cur_h;
        while p != NULL {
            // Section 620
            // reswitch:
            if self.is_char_node(p) {
                synch_h!(self);
                synch_v!(self);
                loop {
                    self.f = font(p);
                    self.c = character(p);
                    if self.f != (self.dvi_f as QuarterWord) {
                        // Section 621
                        if !self.font_used[self.f as usize] {
                            self.dvi_font_def(self.f);
                            self.font_used[self.f as usize] = true;
                        }
                        if self.f <= 64 + (FONT_BASE as QuarterWord) {
                            dvi_out!(self, (self.f as u8) + (FNT_NUM_0 as Integer - FONT_BASE - 1) as u8);
                        }
                        else {
                            dvi_out!(self, FNT1);
                            dvi_out!(self, (self.f as u8) - (FONT_BASE as u8) - 1);
                        }
                        self.dvi_f = self.f as usize;
                        // End section 621
                    }
                    if self.c >= 128 {
                        dvi_out!(self, SET1);
                    }
                    dvi_out!(self, self.c as u8);
                    self.cur_h += self.char_width(self.f, self.char_info(self.f, self.c));
                    p = link(p);
                    if !self.is_char_node(p) {
                        break;
                    }
                }
                self.dvi_h = self.cur_h;
            }
            else {
                // Section 622
                let goto = match r#type(p) {
                    HLIST_NODE
                    | VLIST_NODE => {
                        // Section 623
                        if list_ptr(p) == NULL {
                            self.cur_h += width(p);
                        }
                        else {
                            let save_h = self.dvi_h;
                            let save_v = self.dvi_v;
                            self.cur_v = base_line + shift_amount(p);
                            self.temp_ptr = p;
                            let edge = self.cur_h;
                            match r#type(p) {
                                VLIST_NODE => self.vlist_out()?,
                                _ => self.hlist_out()?
                            }
                            self.dvi_h = save_h;
                            self.dvi_v = save_v;
                            self.cur_h = edge + width(p);
                            self.cur_v = base_line;
                        }
                        Goto::NextP
                        // End section 623
                    },
        
                    RULE_NODE => {
                        self.rule_ht = height(p);
                        self.rule_dp = depth(p);
                        self.rule_wd = width(p);
                        Goto::FinRule
                    },
        
                    WHATSIT_NODE => {
                        // Section 1367
                        self.out_what(p)?;
                        // End section 1367
                        Goto::NextP
                    },
        
                    GLUE_NODE => {
                        // Section 625
                        self.g = glue_ptr(p);
                        self.rule_wd = width(self.g) - cur_g;
                        if g_sign != NORMAL {
                            if g_sign == STRETCHING {
                                if stretch_order(self.g) == g_order {
                                    cur_glue += stretch(self.g) as Real;
                                    cur_g = vet_glue!(glue_set(this_box) * cur_glue).round() as Scaled;
                                }
                            }
                            else if shrink_order(self.g) == g_order {
                                cur_glue -= shrink(self.g) as Real;
                                cur_g = vet_glue!(glue_set(this_box) * cur_glue).round() as Scaled;
                            }
                        }
                        self.rule_wd += cur_g;
                        if subtype(p) >= A_LEADERS {
                            self.sec626_output_leaders(p, left_edge, base_line)?
                        }
                        else {
                            Goto::MovePast
                        }
                        // End section 625
                    },
        
                    KERN_NODE
                    | MATH_NODE => {
                        self.cur_h += width(p);
                        Goto::NextP
                    },
        
                    LIGATURE_NODE => {
                        // Section 652
                        *mem_mut![LIG_TRICK as usize] = mem![lig_char!(p) as usize];
                        *link_mut(LIG_TRICK) = link(p);
                        p = LIG_TRICK;
                        continue; // Goto reswitch
                        // End section 652
                    },
        
                    _ => Goto::NextP, // Do nothing
                };
        
                if let Goto::FinRule = goto {
                    // fin_rule:
                    // Sction 624
                    if is_running!(self.rule_ht) {
                        self.rule_ht = height(this_box);
                    }
                    if is_running!(self.rule_dp) {
                        self.rule_dp = depth(this_box);
                    }
                    self.rule_ht += self.rule_dp;
                    if self.rule_ht > 0 && self.rule_wd > 0 {
                        synch_h!(self);
                        self.cur_v = base_line + self.rule_dp;
                        synch_v!(self);
                        dvi_out!(self, SET_RULE);
                        self.dvi_four(self.rule_ht);
                        self.dvi_four(self.rule_wd);
                        self.cur_v = base_line;
                        self.dvi_h += self.rule_wd;
                    }
                    // End section 624
                }
        
                match goto {
                    Goto::FinRule
                    | Goto::MovePast => {
                        // move_past:
                        self.cur_h += self.rule_wd;
                    },
                    Goto::NextP => (),
                }
        
                // next_p:
                p = link(p);
                // End section 622
            }
            // End section 620
        }
        self.prune_movements(save_loc);
        if self.cur_s > 0 {
            self.dvi_pop(save_loc);
        }
        self.cur_s -= 1;
        Ok(())
    }

    // Section 626
    fn sec626_output_leaders(&mut self, p: HalfWord, left_edge: Scaled, base_line: Scaled) -> TeXResult<Goto> {
        let leader_box = leader_ptr(p);
        if r#type(leader_box) == RULE_NODE {
            self.rule_ht = height(leader_box);
            self.rule_dp = depth(leader_box);
            return Ok(Goto::FinRule);
        }
        let leader_wd = width(leader_box);
        if leader_wd > 0 && self.rule_wd > 0 {
            self.rule_wd += 10;
            let edge = self.cur_h + self.rule_wd;
            let mut lx = 0;

            // Section 627
            if subtype(p) == A_LEADERS {
                let save_h = self.cur_h;
                self.cur_h = left_edge + leader_wd*((self.cur_h - left_edge) / leader_wd);
                if self.cur_h < save_h {
                    self.cur_h += leader_wd;
                }
            }
            else {
                self.lq = self.rule_wd / leader_wd;
                self.lr = self.rule_wd % leader_wd;
                if subtype(p) == C_LEADERS {
                    self.cur_h += self.lr / 2;
                }
                else {
                    lx = self.lr / (self.lq + 1);
                    self.cur_h += (self.lr - (self.lq - 1)*lx) / 2;
                }
            }
            // End section 627

            while self.cur_h + leader_wd <= edge {
                // Section 628
                self.cur_v = base_line + shift_amount(leader_box);
                synch_v!(self);
                let save_v = self.dvi_v;
                synch_h!(self);
                let save_h = self.dvi_h;
                self.temp_ptr = leader_box;
                let outer_doing_leaders = self.doing_leaders;
                self.doing_leaders = true;
                match r#type(leader_box) {
                    VLIST_NODE => self.vlist_out()?,
                    _ => self.hlist_out()?
                }
                self.doing_leaders = outer_doing_leaders;
                self.dvi_v = save_v;
                self.dvi_h = save_h;
                self.cur_v = base_line;
                self.cur_h = save_h + leader_wd + lx;
                // End section 628
            }
            self.cur_h = edge - 10;
            return Ok(Goto::NextP);
        }
        Ok(Goto::MovePast)
    }

    // Section 629
    fn vlist_out(&mut self) -> TeXResult<()> {
        let mut cur_g = 0;
        let mut cur_glue = 0.0;
        let this_box = self.temp_ptr;
        let g_order = glue_order(this_box);
        let g_sign = glue_sign(this_box);
        let mut p = list_ptr(this_box);
        self.cur_s += 1;
        if self.cur_s > 0 {
            dvi_out!(self, PUSH);
        }
        if self.cur_s > self.max_push {
            self.max_push = self.cur_s;
        }
        let save_loc = self.dvi_offset + (self.dvi_ptr as Integer);
        let left_edge = self.cur_h;
        self.cur_v -= height(this_box);
        let top_edge = self.cur_v;
        while p != NULL {
            // Section 630
            if self.is_char_node(p) {
                return Err(TeXError::Confusion("vlistout"));
            }
            
            // Section 631
            let mut goto = Goto::NextP;
            match r#type(p) {
                HLIST_NODE
                | VLIST_NODE => {
                    // Section 632
                    if list_ptr(p) == NULL {
                        self.cur_v += height(p) + depth(p);
                    }
                    else {
                        self.cur_v += height(p);
                        synch_v!(self);
                        let save_h = self.dvi_h;
                        let save_v = self.dvi_v;
                        self.cur_h = left_edge + shift_amount(p);
                        self.temp_ptr = p;
                        match r#type(p) {
                            VLIST_NODE => self.vlist_out()?,
                            _ => self.hlist_out()?
                        }
                        self.dvi_h = save_h;
                        self.dvi_v = save_v;
                        self.cur_v = save_v + depth(p);
                        self.cur_h = left_edge;
                    }
                    // End secction 632
                },
    
                RULE_NODE => {
                    self.rule_ht = height(p);
                    self.rule_dp = depth(p);
                    self.rule_wd = width(p);
                    goto = Goto::FinRule;
                },
    
                WHATSIT_NODE => self.out_what(p)?, // Section 1366
    
                GLUE_NODE => {
                    // Section 634
                    self.g = glue_ptr(p);
                    self.rule_ht = width(self.g) - cur_g;
                    if g_sign != NORMAL {
                        if g_sign == STRETCHING {
                            if stretch_order(self.g) == g_order {
                                cur_glue += stretch(self.g) as Real;
                                cur_g = vet_glue!(glue_set(this_box) * cur_glue).round() as Scaled;
                            }
                        }
                        else if shrink_order(self.g) == g_order {
                            cur_glue -= shrink(self.g) as Real;
                            cur_g = vet_glue!(glue_set(this_box) * cur_glue).round() as Scaled;
                        }
                    }
                    self.rule_ht += cur_g;
                    goto = if subtype(p) >= A_LEADERS {
                        self.sec635_output_leaders(p, left_edge, top_edge)?
                    }
                    else {
                        Goto::MovePast
                    };
                    // End section 634
                },
    
                KERN_NODE => self.cur_v += width(p),
    
                _ => (), // Do nothing
            }
    
            if let Goto::FinRule = goto {
                // Section 633
                if is_running!(self.rule_wd) {
                    self.rule_wd = width(this_box);
                }
                self.rule_ht += self.rule_dp;
                self.cur_v += self.rule_ht;
                if self.rule_ht > 0 && self.rule_wd > 0 {
                    synch_h!(self);
                    synch_v!(self);
                    dvi_out!(self, PUT_RULE);
                    self.dvi_four(self.rule_ht);
                    self.dvi_four(self.rule_wd);
                }
                // Goto next_p
                // End section 633
            }
            else if let Goto::MovePast = goto {
                // move_past:
                self.cur_v += self.rule_ht;
            }
            // End section 631
            
            // next_p:
            p = link(p);
            // End section 630
        }
        self.prune_movements(save_loc);
        if self.cur_s > 0 {
            self.dvi_pop(save_loc);
        }
        self.cur_s -= 1;
        Ok(())
    }

    // Section 635
    fn sec635_output_leaders(&mut self, p: HalfWord, left_edge: Scaled, top_edge: Scaled) -> TeXResult<Goto> {
        let leader_box = leader_ptr(p);
        if r#type(leader_box) == RULE_NODE {
            self.rule_wd = width(leader_box);
            self.rule_dp = 0;
            return Ok(Goto::FinRule);
        }
        let leader_ht = height(leader_box) + depth(leader_box);
        if leader_ht > 0 && self.rule_ht > 0 {
            self.rule_ht += 10;
            let edge = self.cur_v + self.rule_ht;
            let mut lx = 0;

            // Section 636
            if subtype(p) == A_LEADERS {
                let save_v = self.cur_v;
                self.cur_v = top_edge + leader_ht*((self.cur_v - top_edge) / leader_ht);
                if self.cur_v < save_v {
                    self.cur_v += leader_ht;
                }
            }
            else {
                self.lq = self.rule_ht / leader_ht;
                self.lr  = self.rule_ht % leader_ht;
                if subtype(p) == C_LEADERS {
                    self.cur_v += self.lr / 2;
                }
                else {
                    lx = self.lr / (self.lq + 1);
                    self.cur_v += (self.lr - (self.lq - 1)*lx) / 2;
                }
            }
            // End section 636

            while self.cur_v + leader_ht <= edge {
                // Section 637
                self.cur_h = left_edge + shift_amount(leader_box);
                synch_h!(self);
                let save_h = self.dvi_h;
                self.cur_v += height(leader_box);
                synch_v!(self);
                let save_v = self.dvi_v;
                self.temp_ptr = leader_box;
                let outer_doing_leaders = self.doing_leaders;
                self.doing_leaders = true;
                match r#type(leader_box) {
                    VLIST_NODE => self.vlist_out()?,
                    _ => self.hlist_out()?
                }
                self.doing_leaders = outer_doing_leaders;
                self.dvi_v = save_v;
                self.dvi_h = save_h;
                self.cur_h = left_edge;
                self.cur_v = save_v - height(leader_box) + leader_ht + lx;
                // End section 637
            }
            self.cur_v = edge - 10;
            return Ok(Goto::NextP);
        }
        Ok(Goto::MovePast)
    }

    // Section 638
    pub(crate) fn ship_out(&mut self, p: HalfWord) -> TeXResult<()> {
        if tracing_output() > 0 {
            self.print_nl("");
            self.print_ln();
            self.print("Completed box being shipped out");
        }
        if self.term_offset > MAX_PRINT_LINE - 9 {
            self.print_ln();
        }
        else if self.term_offset > 0 || self.file_offset > 0 {
            self.print_char(b' ');
        }
        self.print_char(b'[');
        let mut j = 9;
        while count(j) == 0 && j > 0 {
            j -= 1;
        }
        for k in 0..=j {
            self.print_int(count(k));
            if k < j {
                self.print_char(b'.');
            }
        }
        update_terminal!();
        if tracing_output() > 0 {
            self.print_char(b']');
            self.begin_diagnostic();
            self.show_box(p);
            self.end_diagnostic(true);
        }
        
        // Section 640
        // Section 641
        if height(p) > MAX_DIMEN
            || depth(p) > MAX_DIMEN
            || height(p) + depth(p) + v_offset() > MAX_DIMEN
            || width(p) + h_offset() > MAX_DIMEN
        {
            return Err(TeXError::HugePage);
        }
        if height(p) + depth(p) + v_offset() > self.max_v {
            self.max_v = height(p) + depth(p) + v_offset();
        }
        if width(p) + h_offset() > self.max_h {
            self.max_h = width(p) + h_offset();
        }
        // End secction 641

        // Section 532
        macro_rules! ensure_dvi_open {
            () => {
                if self.output_file_name == 0 {
                    if self.job_name == 0 {
                        self.open_log_file()?;
                    }
                    self.pack_job_name(EXT_DVI);
                    self.b_open_out(ByteFileOutSelector::DviFile)?;
                    self.output_file_name = self.make_name_string()?;
                }
            };
        }

        // Section 617
        self.dvi_h = 0;
        self.dvi_v = 0;
        self.cur_h = h_offset();
        self.dvi_f = NULL_FONT as usize;
        ensure_dvi_open!();
        if self.total_pages == 0 {
            dvi_out!(self, PRE);
            dvi_out!(self, ID_BYTE);
            self.dvi_four(25_400_000);
            self.dvi_four(473_628_672);
            self.prepare_mag()?;
            self.dvi_four(mag());
            let old_setting = self.selector;
            self.selector = NEW_STRING;
            self.print(" TeX output ");
            self.print_int(year());
            self.print_char(b'.');
            self.print_two(month());
            self.print_char(b'.');
            self.print_two(day());
            self.print_char(b':');
            self.print_two(time() / 60);
            self.print_two(time() % 60);
            self.selector = old_setting;
            dvi_out!(self, cur_length() as u8);
            for s in str_pool!(str_start(str_ptr()), pool_ptr()) {
                dvi_out!(self, *s);
            }
            *pool_ptr_mut() = str_start(str_ptr());
        }
        // End section 617

        let page_loc = self.dvi_offset + (self.dvi_ptr as Integer);
        dvi_out!(self, BOP);
        for k in 0..=9 {
            self.dvi_four(count(k));
        }
        self.dvi_four(self.last_bop);
        self.last_bop = page_loc;
        self.cur_v = height(p) + v_offset();
        self.temp_ptr = p;
        match r#type(p) {
            VLIST_NODE => self.vlist_out()?,
            _ => self.hlist_out()?
        }
        dvi_out!(self, EOP);
        self.total_pages += 1;
        self.cur_s = -1;
        // End section 640

        if tracing_output() <= 0 {
            self.print_char(b']');
        }
        self.dead_cycles = 0;
        update_terminal!();

        // Section 639
        #[cfg(feature = "stat")]
        {
            if tracing_stats() > 1 {
                self.print_nl("Memory usage before: ");
                self.print_int(self.var_used);
                self.print_char(b'&');
                self.print_int(self.dyn_used);
                self.print_char(b';');
            }
        }
        self.flush_node_list(p)?;
        #[cfg(feature = "stat")]
        {
            if tracing_stats() > 1 {
                self.print(" after: ");
                self.print_int(self.var_used);
                self.print_char(b'&');
                self.print_int(self.dyn_used);
                self.print("; still untouched: ");
                self.print_int(self.hi_mem_min - self.lo_mem_max - 1);
                self.print_ln();
            }
        }
        // End section 639
        Ok(())
    }

    // Section 642
    pub(crate) fn sec642_finish_the_dvi_file(&mut self) -> TeXResult<()> {
        while self.cur_s > -1 {
            if self.cur_s > 0 {
                dvi_out!(self, POP);
            }
            else {
                dvi_out!(self, EOP);
                self.total_pages += 1;
            }
            self.cur_s -= 1;
        }
        if self.total_pages == 0 {
            self.print_nl("No pages of output.");
        }
        else {
            dvi_out!(self, POST);
            self.dvi_four(self.last_bop);
            self.last_bop = self.dvi_offset + (self.dvi_ptr as Integer) - 5;
            self.dvi_four(25_400_000);
            self.dvi_four(473_628_672);
            self.prepare_mag()?;
            self.dvi_four(mag());
            self.dvi_four(self.max_v);
            self.dvi_four(self.max_h);
            dvi_out!(self, (self.max_push / 256) as u8);
            dvi_out!(self, (self.max_push % 256) as u8);
            dvi_out!(self, ((self.total_pages / 256) % 256) as u8);
            dvi_out!(self, (self.total_pages % 256) as u8);

            // Section 643
            while (self.font_ptr as Integer) > FONT_BASE {
                if self.font_used[self.font_ptr as usize] {
                    self.dvi_font_def(self.font_ptr);
                }
                self.font_ptr -= 1;
            }
            // End section 643

            dvi_out!(self, POST_POST);
            self.dvi_four(self.last_bop);
            dvi_out!(self, ID_BYTE);
            let mut k = 4 + (DVI_BUF_SIZE - self.dvi_ptr as Integer) % 4;
            while k > 0 {
                dvi_out!(self, 223);
                k -= 1;
            }

            // Section 599
            if self.dvi_limit == self.half_buf {
                self.write_dvi(self.half_buf, (DVI_BUF_SIZE - 1) as usize);
            }
            if self.dvi_ptr > 0 {
                self.write_dvi(0, self.dvi_ptr - 1);
            }
            // End section 599

            self.print_nl("Output written on ");
            self.slow_print(self.output_file_name);
            self.print(" (");
            self.print_int(self.total_pages);
            self.print(" page");
            if self.total_pages != 1 {
                self.print_char(b's');
            }
            self.print(", ");
            self.print_int(self.dvi_offset + self.dvi_ptr as Integer);
            self.print(" bytes).");
            self.dvi_file.close();
        }

        Ok(())
    }

    // Section 1368
    fn special_out(&mut self, p: HalfWord) -> TeXResult<()> {
        synch_h!(self);
        synch_v!(self);
        let old_setting = self.selector;
        self.selector = NEW_STRING;
        self.show_token_list(link(write_tokens(p)), NULL, POOL_SIZE - pool_ptr() as Integer);
        self.selector = old_setting;
        str_room(1)?;
        if cur_length() < 256 {
            dvi_out!(self, XXX1);
            dvi_out!(self, cur_length() as u8);
        }
        else {
            dvi_out!(self, XXX4);
            self.dvi_four(cur_length() as Integer);
        }
        for b in str_pool![str_start(str_ptr()), pool_ptr()] {
            dvi_out!(self, *b);
        }
        *pool_ptr_mut() = str_start(str_ptr());
        Ok(())
    }

    // Section 1370
    fn write_out(&mut self, p: HalfWord) -> TeXResult<()> {
        // Section 1371
        let mut q = self.get_avail()?;
        *info_mut(q) = RIGHT_BRACE_TOKEN + b'}' as HalfWord;
        let r = self.get_avail()?;
        *link_mut(q) = r;
        *info_mut(r) = END_WRITE_TOKEN;
        ins_list!(self, q);
        self.begin_token_list(write_tokens(p), WRITE_TEXT)?;
        q = self.get_avail()?;
        *info_mut(q) = LEFT_BRACE_TOKEN + b'{' as HalfWord;
        ins_list!(self, q);
        let old_mode = self.mode();
        *self.mode_mut() = 0;
        self.cur_cs = self.write_loc;
        _ = self.scan_toks(false, true)?;
        self.get_token()?;
        if self.cur_tok != END_WRITE_TOKEN {
            return Err(TeXError::UnbalancedWriteCmd);
        }
        *self.mode_mut() = old_mode;
        self.end_token_list()?;
        // End section 1371

        let old_setting = self.selector;
        let j = write_stream(p) as usize;
        if self.write_open[j] {
            self.selector = j as Integer;
        }
        else {
            if j == 17 && self.selector == TERM_AND_LOG {
                self.selector = LOG_ONLY;
            }
            self.print_nl("");
        }
        self.token_show(self.def_ref);
        self.print_ln();
        self.flush_list(self.def_ref);
        self.selector = old_setting;
        Ok(())
    }

    // Section 1373
    pub(crate) fn out_what(&mut self, p: HalfWord) -> TeXResult<()> {
        match subtype(p) as Integer {
            OPEN_NODE
            | WRITE_NODE
            | CLOSE_NODE => {
                // Section 1374
                if !self.doing_leaders {
                    let j = write_stream(p) as usize;
                    if subtype(p) == WRITE_NODE as QuarterWord {
                        self.write_out(p)?;
                    }
                    else {
                        if self.write_open[j] {
                            self.write_file[j].close();
                        }
                        if subtype(p) == CLOSE_NODE as QuarterWord {
                            self.write_open[j] = false;
                        }
                        else if j < 16 {
                            self.cur_name = open_name(p) as StrNum;
                            self.cur_area = open_area(p) as StrNum;
                            self.cur_ext = open_ext(p) as StrNum;
                            if self.cur_ext == EMPTY_STRING {
                                self.cur_ext = EXT_TEX;
                            }
                            self.pack_cur_name();
                            self.a_open_out(AlphaFileOutSelector::WriteFile(j))?;
                            self.write_open[j] = true;
                        }
                    }
                }
                // End section 1374
            },

            SPECIAL_NODE => self.special_out(p)?,

            LANGUAGE_NODE => (), // Do nothing

            _ => return Err(TeXError::Confusion("ext4"))
        }
        Ok(())
    }
}
