use crate::arithmetic::xn_over_d;
use crate::constants::*;
use crate::datastructures::{
    MemoryWord, character_mut, default_hyphen_char, default_skew_char,
    font_mut, tracing_lost_chars,
};
use crate::error::{TeXError, TeXResult};
use crate::io::ByteFileInSelector;
use crate::{
    Global, HalfWord, Integer, QuarterWord, Scaled, StrNum
};

use std::cmp::Ordering::{Equal, Greater, Less};

// Part 30: Font metric data

impl MemoryWord {
    // Section 545
    pub(crate) fn skip_byte(&self) -> QuarterWord {
        self.qqqq_b0()
    }

    pub(crate) fn next_char(&self) -> QuarterWord {
        self.qqqq_b1()
    }

    pub(crate) fn op_byte(&self) -> QuarterWord {
        self.qqqq_b2()
    }

    pub(crate) fn rem_byte(&self) -> QuarterWord {
        self.qqqq_b3()
    }

    // Section 546
    pub(crate) fn ext_top(&self) -> QuarterWord {
        self.qqqq_b0()
    }

    pub(crate) fn ext_mid(&self) -> QuarterWord {
        self.qqqq_b1()
    }

    pub(crate) fn ext_bot(&self) -> QuarterWord {
        self.qqqq_b2()
    }

    pub(crate) fn ext_rep(&self) -> QuarterWord {
        self.qqqq_b3()
    }

    // Section 554
    pub(crate) fn char_exists(&self) -> bool {
        self.qqqq_b0() > MIN_QUARTERWORD
    }

    pub(crate) fn height_depth(&self) -> QuarterWord {
        self.qqqq_b1()
    }

    pub(crate) fn char_tag(&self) -> QuarterWord {
        self.qqqq_b2() % 4
    }
}

impl Global {
    // Section 554
    pub(crate) fn char_info(&self, f: QuarterWord, d: QuarterWord) -> MemoryWord {
        self.font_info[(self.char_base[f as usize] + d as Integer) as usize]
    }

    pub(crate) fn char_width(&self, f: QuarterWord, d: MemoryWord) -> Scaled {
        self.font_info[(self.width_base[f as usize] + d.qqqq_b0() as Integer) as usize].sc()
    }

    pub(crate) fn char_italic(&self, f: QuarterWord, d: MemoryWord) -> Scaled {
        self.font_info[(self.italic_base[f as usize] + (d.qqqq_b2() / 4) as Integer) as usize].sc()
    }

    pub(crate) fn char_height(&self, f: QuarterWord, d: QuarterWord) -> Scaled {
        self.font_info[(self.height_base[f as usize] + (d / 16) as Integer) as usize].sc()
    }
    
    pub(crate) fn char_depth(&self, f: QuarterWord, d: QuarterWord) -> Scaled {
        self.font_info[(self.depth_base[f as usize] + (d % 16) as Integer) as usize].sc()
    }

    // Section 557
    pub(crate) fn char_kern(&self, p: QuarterWord, q: MemoryWord) -> Scaled {
        self.font_info[(self.kern_base[p as usize]
            + 256 * (q.op_byte() as Integer)
            + (q.rem_byte() as Integer)) as usize]
            .sc()
    }

    pub(crate) fn lig_kern_start(&self, p: QuarterWord, q: MemoryWord) -> Integer {
        self.lig_kern_base[p as usize] + (q.rem_byte() as Integer)
    }

    pub(crate) fn lig_kern_restart(&self, p: QuarterWord, q: MemoryWord) -> Integer {
        self.lig_kern_base[p as usize]
            + 256 * (q.op_byte() as Integer)
            + (q.rem_byte() as Integer)
            + 32768
            - KERN_BASE_OFFSET
    }

    // Section 558
    fn param(&self, f: Integer, d: QuarterWord) -> Scaled {
        self.font_info[(f + self.param_base[d as usize]) as usize].sc()
    }

    pub(crate) fn slant(&self, d: QuarterWord) -> Scaled {
        self.param(SLANT_CODE, d)
    }

    pub(crate) fn space(&self, d: QuarterWord) -> Scaled {
        self.param(SPACE_CODE, d)
    }

    pub(crate) fn x_height(&mut self, d: QuarterWord) -> Scaled {
        self.param(X_HEIGHT_CODE, d)
    }
    
    pub(crate) fn quad(&mut self, d: QuarterWord) -> Scaled {
        self.param(QUAD_CODE, d)
    }

    pub(crate) fn extra_space(&mut self, d: QuarterWord) -> Scaled {
        self.param(EXTRA_SPACE_CODE, d)
    }

    // Section 560
    pub(crate) fn read_font_info(&mut self, u: HalfWord, nom: StrNum, aire: StrNum, s: Scaled) -> TeXResult<usize> {
        // Section 562
        // Section 563
        if aire == EMPTY_STRING {
            self.pack_file_name(nom, TEX_FONT_AREA, EXT_TFM);
        }
        else {
            self.pack_file_name(nom, aire, EXT_TFM);
        }
        if !self.b_open_in(ByteFileInSelector::TfmFile) {
            return Err(TeXError::TfmNotLoadable(false, u, s));
        }
        // End section 563

        // Section 565
        macro_rules! fbyte {
            () => {
                self.tfm_file
                    .fbyte()
                    .ok_or(TeXError::TfmNotLoadable(true, u, s))?   
            };
        }

        macro_rules! read_sixteen {
            () => {
                self.tfm_file
                    .read_sixteen()
                    .ok_or(TeXError::TfmNotLoadable(true, u, s))?
            };
        }

        macro_rules! abort {
            () => {
                return Err(TeXError::TfmNotLoadable(true, u, s))
            };
        }

        let (mut a, mut b, mut c, mut d): (u8, u8, u8, u8);

        macro_rules! store_four_quarters {
            () => {
                {
                    (a, b, c, d) = self.tfm_file
                        .read_four_quarters()
                        .ok_or(TeXError::TfmNotLoadable(true, u, s))?;
                    MemoryWord { qqqq: [a as QuarterWord, b as QuarterWord, c as QuarterWord, d as QuarterWord] }
                }
            };
        }

        let mut lf = read_sixteen!();
        let mut lh = read_sixteen!();
        let mut bc = read_sixteen!();
        let mut ec = read_sixteen!();

        if bc > ec + 1 || ec > 255 {
            abort!();
        }
        if bc > 255 {
            bc = 1;
            ec = 0;
        }

        let nw = read_sixteen!();
        let nh = read_sixteen!();
        let nd = read_sixteen!();
        let ni = read_sixteen!();
        let nl = read_sixteen!();
        let nk = read_sixteen!();
        let ne = read_sixteen!();
        let np = read_sixteen!();

        if lf != (6 + lh + (ec - bc + 1) + nw + nh + nd + ni + nl + nk + ne + np)
            || nw == 0
            || nh == 0
            || nd == 0
            || ni == 0
        {
            abort!()
        }
        // End section 565

        // Section 566
        lf -= 6 + lh;
        if np < 7 {
            lf += 7 - np;
        }
        if self.font_ptr == (FONT_MAX as QuarterWord)
            || (self.fmem_ptr as Integer) + lf > FONT_MEM_SIZE
        {
            return Err(TeXError::TfmNotLoaded(u, s));
        }
        let f = (self.font_ptr + 1) as usize;
        self.char_base[f] = self.fmem_ptr as Integer - bc;
        self.width_base[f] = self.char_base[f] + ec + 1;
        self.height_base[f] = self.width_base[f] + nw;
        self.depth_base[f] = self.height_base[f] + nh;
        self.italic_base[f] = self.depth_base[f] + nd;
        self.lig_kern_base[f] = self.italic_base[f] + ni;
        self.kern_base[f] = self.lig_kern_base[f] + nl - KERN_BASE_OFFSET;
        self.exten_base[f] = self.kern_base[f] + KERN_BASE_OFFSET + nk;
        self.param_base[f] = self.exten_base[f] + ne;
        // End section 566

        // Section 568
        if lh < 2 {
            abort!();
        }

        self.font_check[f] = store_four_quarters!();
        let mut z = read_sixteen!();
        z = z*256 + (fbyte!() as Scaled);
        z = z*16 + (fbyte!() / 16) as Scaled;
        if z < UNITY {
            abort!();
        }
        while lh > 2 {
            _ = fbyte!();
            _ = fbyte!();
            _ = fbyte!();
            _ = fbyte!();
            lh -= 1;
        }
        self.font_dsize[f] = z;
        if s != -1000 {
            z = match s.cmp(&0) {
                Greater | Equal => s,
                Less => xn_over_d(z, -s, 1000)?.0,
            };
        }
        self.font_size[f] = z;
        // End section 568

        // Section 570
        macro_rules! check_byte_range {
            ($d:expr) => {
                if ($d as HalfWord) < bc || ($d as HalfWord) > ec {
                    abort!();
                }
            };
        }

        // Section 569
        for k in self.fmem_ptr..(self.width_base[f] as usize) {
            self.font_info[k] = store_four_quarters!();
            if (a as HalfWord) >= nw
                || ((b / 16) as HalfWord) >= nh
                || ((b % 16) as HalfWord) >= nd
                || ((c / 4) as HalfWord) >= ni
            {
                abort!();
            }
            match (c % 4) as QuarterWord {
                LIG_TAG => {
                    if (d as HalfWord) >= nl {
                        abort!();
                    }
                },

                EXT_TAG => {
                    if (d as HalfWord) >= ne {
                        abort!();
                    }
                },

                LIST_TAG => {
                    // Section 570
                    check_byte_range!(d);
                    'block: {
                        while (d as Integer) < (k as Integer) + bc - (self.fmem_ptr as Integer) {
                            let qw = self.char_info(f as QuarterWord, d as QuarterWord);
                            if qw.char_tag() != LIST_TAG {
                                break 'block; // Goto not_found
                            }
                            d = qw.rem_byte() as u8;
                        }
                        if (d as Integer) == (k as Integer) + bc - (self.fmem_ptr as Integer) {
                            abort!();
                        }
                    }
                    // not_found:
                    // End section 570
                },

                _ => (), // Do nothing, no tag.
            }
        }
        // End section 569

        // Section 571
        // Section 572
        let mut alpha = 16;
        while z >= 0x80_0000 {
            z /= 2;
            alpha *= 2;
        }
        let beta = 256 / alpha;
        alpha *= z;
        // End section 572

        macro_rules! store_scaled {
            () => {
                {
                    (a, b, c, d) = self.tfm_file
                        .read_four_quarters()
                        .ok_or(TeXError::TfmNotLoadable(true, u, s))?;
                    let sw = ((((((d as Scaled)*z) / 256) + ((c as Scaled)*z)) / 256) + ((b as Scaled)*z)) / beta;
                    match a {
                        0 => sw,
                        255 => sw - alpha,
                        _ => abort!()
                    }
                }
            };
        }

        for k in self.width_base[f]..self.lig_kern_base[f] {
            *self.font_info[k as usize].sc_mut() = store_scaled!();
        }
        if self.font_info[self.width_base[f] as usize].sc() != 0
            || self.font_info[self.height_base[f] as usize].sc() != 0
            || self.font_info[self.depth_base[f] as usize].sc() != 0
            || self.font_info[self.italic_base[f] as usize].sc() != 0
        {
            abort!();
        }
        // End section 571

        // Section 573
        macro_rules! check_existence {
            ($d:expr) => {
                check_byte_range!($d);
                let qw = self.char_info(f as QuarterWord, $d as QuarterWord);
                if !qw.char_exists() {
                    abort!();
                }
            };
        }

        let mut bch_label = 0x7fff;
        let mut bchar: Integer = 256;
        if nl > 0 {
            for k in self.lig_kern_base[f]..(self.kern_base[f] + KERN_BASE_OFFSET as Integer) {
                self.font_info[k as usize] = store_four_quarters!();
                if a > 128 {
                    if 256 * (c as HalfWord) + (d as HalfWord) >= nl {
                        abort!();
                    }
                    if a == 255 && k == self.lig_kern_base[f] {
                        bchar = b as Integer;
                    }
                }
                else {
                    if (b as Integer) != bchar {
                        check_existence!(b);
                    }
                    if c < 128 {
                        check_existence!(d);
                    }
                    else if 256 * ((c as HalfWord) - 128) + (d as HalfWord) >= nk {
                        abort!();
                    }
                    if a < 128 && k - self.lig_kern_base[f] + (a as Integer) + 1 >= nl {
                        abort!();
                    }
                }
            }
            if a == 255 {
                bch_label = 256 * (c as HalfWord) + (d as HalfWord);
            }
        }
        for k in (self.kern_base[f] + KERN_BASE_OFFSET)..self.exten_base[f] {
            *self.font_info[k as usize].sc_mut() = store_scaled!();
        }
        // End section 573

        // Section 574
        for k in self.exten_base[f]..self.param_base[f] {
            self.font_info[k as usize] = store_four_quarters!();
            if a != 0 {
                check_existence!(a);
            }
            if b != 0 {
                check_existence!(b);
            }
            if c != 0 {
                check_existence!(c);
            }
            check_existence!(d);
        }
        // End section 574

        // Section 575
        for k in 1..=np {
            if k == 1 {
                let mut sw = fbyte!() as Scaled;
                if sw > 127 {
                    sw -= 256;
                }
                sw = sw*256 + (fbyte!() as Scaled);
                sw = sw*256 + (fbyte!() as Scaled);
                *self.font_info[self.param_base[f] as usize].sc_mut() = sw*16 + (fbyte!() / 16) as Scaled;
            }
            else {
                *self.font_info[(self.param_base[f] + k - 1) as usize].sc_mut() = store_scaled!();
            }
        }

        if np < 7 {
            for k in (np + 1)..=7 {
                *self.font_info[(self.param_base[f] + k - 1) as usize].sc_mut() = 0;
            }
        }
        // End section 575

        // Section 576
        self.font_params[f] = match np.cmp(&7) {
            Greater | Equal => np as usize,
            Less => 7,
        };

        self.hyphen_char[f] = default_hyphen_char();
        self.skew_char[f] = default_skew_char();
        self.bchar_label[f] = match bch_label.cmp(&nl) {
            Less => (bch_label + self.lig_kern_base[f]) as usize,
            Equal | Greater => NON_ADDRESS as usize,
        };

        self.font_bchar[f] = bchar as usize;
        self.font_false_bchar[f] = bchar as usize;

        if bchar <= ec && bchar >= bc {
            let qw = self.char_info(f as QuarterWord, bchar as QuarterWord);
            if qw.char_exists() {
                self.font_false_bchar[f] = NON_CHAR as usize;
            }
        }
        self.font_name[f] = nom;
        self.font_area[f] = aire;
        self.font_bc[f] = bc as u8;
        self.font_ec[f] = ec as u8;
        self.font_glue[f] = NULL;
        self.param_base[f] -= 1;
        self.fmem_ptr += lf as usize;
        self.font_ptr = f as QuarterWord;
        // End section 576
        // End section 562

        // bad_tfm: return Err early above.
        // done:
        self.tfm_file.close();
        Ok(f)
    }

    // Section 581
    pub(crate) fn char_warning(&mut self, f: QuarterWord, c: u8) {
        if tracing_lost_chars() > 0 {
            self.begin_diagnostic();
            self.print_nl("Missing character: There is no ");
            self.print_strnumber(c as usize);
            self.print(" in font ");
            self.print_strnumber(self.font_name[f as usize]);
            self.print_char(b'!');
            self.end_diagnostic(false);
        }
    }

    // Section 582
    pub(crate) fn new_character(&mut self, f: QuarterWord, c: u8) -> TeXResult<HalfWord> {
        if self.font_bc[f as usize] <= c
            && self.font_ec[f as usize] >= c
            && self.char_info(f, c as QuarterWord).char_exists()
        {
            let p = self.get_avail()?;
            *font_mut(p) = f;
            *character_mut(p) = c as QuarterWord;
            Ok(p)
        }
        else {
            self.char_warning(f, c);
            Ok(NULL)
        }
    }
}
