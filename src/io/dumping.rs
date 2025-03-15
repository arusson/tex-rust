use crate::constants::*;
use crate::datastructures::{
    eqtb, eqtb_mut, hash, hash_mut, mem, mem_mut, MemoryWord, day, eq_level, eq_type, equiv,
    font_id_text, link, month, node_size, rlink, text, tracing_stats_mut, year
};
use crate::error::{TeXError, TeXResult};
use crate::io::{ByteFileInSelector, ByteFileOutSelector};
use crate::strings::{
    POOL, init_pool_ptr_set, init_str_ptr_set, make_string, pool_ptr,
    pool_ptr_set, str_ptr, str_ptr_set, str_room, str_start, str_start_mut
};
use crate::{
    Global, Integer, QuarterWord, StrNum, str_pool, str_pool_mut
};

// Part 50: Dumping and undumping the tables

impl Global {
    // Section 524
    // Format file should be supplied on command line, or plain used as default.
    // The string `s` is supplied on the command line,
    // but it falls back the defaukt (which is plain.fmt) if empty.
    pub fn open_fmt_file(&mut self, s: &str) -> TeXResult<()> {
        let fmt_fname = if s.is_empty() {
            TEX_FORMAT_DEFAULT.to_string()
        }
        else {
            // Add ".fmt" if the extension is not present.
            if !s.ends_with(".fmt") {
                s.to_string() + ".fmt"
            }
            else {
                s.to_string()
            }
        };

        // Try first without area
        self.name_of_file.clear();
        self.name_of_file.push_str(&fmt_fname);
        if !self.b_open_in(ByteFileInSelector::FmtFile) {
            // Second try with area
            self.name_of_file.clear();
            self.name_of_file.push_str(TEX_FORMAT_AREA);
            self.name_of_file.push_str(&fmt_fname);
            if !self.b_open_in(ByteFileInSelector::FmtFile) {
                return Err(TeXError::CantFindFormat);
            }
        }
        Ok(())
    }

    // Section 1302
    pub(crate) fn store_fmt_file(&mut self) -> TeXResult<()> {
        // Section 1304
        if self.save_ptr != 0 {
            return Err(TeXError::CantDumpInGroup);
        }
        // End section 1304

        // Section 1328
        self.selector = NEW_STRING;
        self.print(" (preloaded format=");
        self.print_strnumber(self.job_name);
        self.print_char(b' ');
        self.print_int(year());
        self.print_char(b'.');
        self.print_int(month());
        self.print_char(b'.');
        self.print_int(day());
        self.print_char(b')');
        self.selector = match self.interaction {
            BATCH_MODE => LOG_ONLY,
            _ => TERM_AND_LOG,
        };
        str_room(1)?;
        self.format_ident = make_string()?;
        self.pack_job_name(EXT_FMT);

        self.b_open_out(ByteFileOutSelector::FmtFile)?;
        self.print_nl("Beginning to dump on file ");
        self.print(&self.name_of_file.clone());
        self.print_nl("");
        self.slow_print(self.format_ident);
        // End section 1328

        // Section 1305
        macro_rules! dump_wd {
            ($w:expr) => {
                self.fmt_file_out.write_wd($w);
            };
        }

        macro_rules! dump_int {
            ($x:expr) => {
                self.fmt_file_out.write_int($x);
            };
        }
        // End section 1305

        // Section 1307
        dump_int!(0); // @$ = 0, external pool file is not used
        dump_int!(MEM_BOT);
        dump_int!(MEM_TOP);
        dump_int!(EQTB_SIZE);
        dump_int!(HASH_PRIME);
        dump_int!(HYPH_SIZE);
        // End section 1307

        // Section 1309
        dump_int!(pool_ptr() as Integer);
        dump_int!(str_ptr() as Integer);
        for k in 0..=str_ptr() {
            dump_int!(str_start(k) as Integer);
        }
        let mut k = 0;

        macro_rules! dump_four_ASCII {
            () => {
                let w = MemoryWord { qqqq: [
                    str_pool![k] as QuarterWord,
                    str_pool![k + 1] as QuarterWord,
                    str_pool![k + 2] as QuarterWord,
                    str_pool![k + 3] as QuarterWord
                ]};
                dump_wd!(w);
            };
        }

        while k + 4 < pool_ptr() {
            dump_four_ASCII!();
            k += 4;
        }
        k = pool_ptr() - 4;
        dump_four_ASCII!();
        self.print_ln();
        self.print_int(str_ptr() as Integer);
        self.print(" strings of total length ");
        self.print_int(pool_ptr() as Integer);
        // End section 1309

        // Section 1311
        self.sort_avail()?;
        self.var_used = 0;
        dump_int!(self.lo_mem_max);
        dump_int!(self.rover);
        let mut p = MEM_BOT;
        let mut q = self.rover;
        let mut x = 0;
        loop {
            for k in p..=(q + 1) {
                dump_wd!(mem(k as usize));
            }
            x += q + 2 - p;
            self.var_used += q - p;
            p = q + node_size(q);
            q = rlink(q);
            if q == self.rover {
                break;
            }
        }
        self.var_used += self.lo_mem_max - p;
        self.dyn_used = self.mem_end + 1 - self.hi_mem_min;
        for k in p..=self.lo_mem_max {
            dump_wd!(mem(k as usize));
        }
        x += self.lo_mem_max + 1 - p;
        dump_int!(self.hi_mem_min);
        dump_int!(self.avail);
        for k in self.hi_mem_min..=self.mem_end {
            dump_wd!(mem(k as usize));
        }
        x += self.mem_end + 1 - self.hi_mem_min;
        p = self.avail;
        while p != NULL {
            self.dyn_used -= 1;
            p = link(p);
        }
        dump_int!(self.var_used);
        dump_int!(self.dyn_used);
        self.print_ln();
        self.print_int(x);
        self.print(" memory locations dumped; current usage is ");
        self.print_int(self.var_used);
        self.print_char(b'&');
        self.print_int(self.dyn_used);
        // End section 1311
        
        // Section 1313
        // Section 1315
        let mut k = ACTIVE_BASE;
        loop {
            let mut j = k;
            let mut found1 = false;
            while j < INT_BASE - 1 {
                if equiv(j) == equiv(j + 1)
                    && eq_type(j) == eq_type(j + 1)
                    && eq_level(j) == eq_level(j + 1)
                {
                    found1 = true;
                    break;
                }
                j += 1;
            }
            let l = if found1 {
                j += 1;
                let l = j;
                while j < INT_BASE - 1 {
                    if equiv(j) != equiv(j + 1)
                        || eq_type(j) != eq_type(j + 1)
                        || eq_level(j) != eq_level(j + 1)
                    {
                        break; // Goto done1
                    }
                    j += 1;
                }
                l
            }
            else {
                INT_BASE
            };

            // done1:
            dump_int!(l - k);
            while k < l {
                dump_wd!(eqtb(k as usize));
                k += 1;
            }
            k = j + 1;
            dump_int!(k - l);

            if k == INT_BASE {
                break;
            }
        }
        // End section 1315

        // Section 1316
        loop {
            let mut j = k;
            let mut found2 = false;
            while j < EQTB_SIZE {
                if eqtb(j as usize).int() == eqtb((j + 1) as usize).int() {
                    found2 = true;
                    break;
                }
                j += 1;
            }
            let l = if found2 {
                j += 1;
                let l = j;
                while j < EQTB_SIZE {
                    if eqtb(j as usize).int() != eqtb((j + 1) as usize).int() {
                        break; // Goto done2
                    }
                    j += 1;
                }
                l
            }
            else {
                EQTB_SIZE + 1
            };

            // done2:
            dump_int!(l - k);
            while k < l {
                dump_wd!(eqtb(k as usize));
                k += 1;
            }
            k = j + 1;
            dump_int!(k - l);
            
            if k > EQTB_SIZE {
                break;
            }
        }
        // End section 1316

        dump_int!(self.par_loc);
        dump_int!(self.write_loc);

        // Section 1318
        dump_int!(self.hash_used);
        self.cs_count = FROZEN_CONTROL_SEQUENCE - 1 - self.hash_used;
        for p in HASH_BASE..=self.hash_used {
            if text(p) != 0 {
                dump_int!(p);
                dump_wd!(hash(p as usize));
                self.cs_count += 1;
            }
        }
        for p in (self.hash_used + 1)..UNDEFINED_CONTROL_SEQUENCE {
            dump_wd!(hash(p as usize));
        }
        dump_int!(self.cs_count);
        self.print_ln();
        self.print_int(self.cs_count);
        self.print(" multiletter control sequences");
        // End section 1318
        // End section 1313

        // Section 1320
        dump_int!(self.fmem_ptr as Integer);
        for k in 0..self.fmem_ptr {
            dump_wd!(self.font_info[k]);
        }
        dump_int!(self.font_ptr as Integer);
        for k in (NULL_FONT as usize)..=(self.font_ptr as usize) {
            // Section 1322
            dump_wd!(self.font_check[k]);
            dump_int!(self.font_size[k]);
            dump_int!(self.font_dsize[k]);
            dump_int!(self.font_params[k] as Integer);
            dump_int!(self.hyphen_char[k]);
            dump_int!(self.skew_char[k]);
            dump_int!(self.font_name[k] as Integer);
            dump_int!(self.font_area[k] as Integer);
            dump_int!(self.font_bc[k] as Integer);
            dump_int!(self.font_ec[k] as Integer);
            dump_int!(self.char_base[k]);
            dump_int!(self.width_base[k]);
            dump_int!(self.height_base[k]);
            dump_int!(self.depth_base[k]);
            dump_int!(self.italic_base[k]);
            dump_int!(self.lig_kern_base[k]);
            dump_int!(self.kern_base[k]);
            dump_int!(self.exten_base[k]);
            dump_int!(self.param_base[k]);
            dump_int!(self.font_glue[k]);
            dump_int!(self.bchar_label[k] as Integer);
            dump_int!(self.font_bchar[k] as Integer);
            dump_int!(self.font_false_bchar[k] as Integer);
            self.print_nl("\\font");
            self.print_esc_strnumber(font_id_text(k as QuarterWord) as StrNum);
            self.print_char(b'=');
            self.print_file_name(self.font_name[k], self.font_area[k], EMPTY_STRING);
            if self.font_size[k] != self.font_dsize[k] {
                self.print(" at ");
                self.print_scaled(self.font_size[k]);
                self.print("pt");
            }
            // End section 1322
        }
        self.print_ln();
        self.print_int((self.fmem_ptr - 7) as Integer);
        self.print(" words of font info for ");
        self.print_int(self.font_ptr as Integer - FONT_BASE);
        self.print(" preloaded font");
        if self.font_ptr as Integer != FONT_BASE + 1 {
            self.print_char(b's');
        }
        // End section 1320

        // Section 1324
        dump_int!(self.hyph_count);
        for k in 0..=HYPH_SIZE {
            if self.hyph_word[k as usize] != 0 {
                dump_int!(k);
                dump_int!(self.hyph_word[k as usize] as Integer);
                dump_int!(self.hyph_list[k as usize]);
            }
        }
        self.print_ln();
        self.print_int(self.hyph_count);
        self.print(" hyphenation exception");
        if self.hyph_count != 1 {
            self.print_char(b's');
        }
        if self.trie_not_ready {
            self.init_trie()?;
        }
        dump_int!(self.trie_max as Integer);
        for k in 0..=self.trie_max {
            dump_wd!(self.trie[k]);
        }
        dump_int!(self.trie_op_ptr as Integer);
        for k in 1..=self.trie_op_ptr {
            dump_int!(self.hyf_distance[k] as Integer);
            dump_int!(self.hyf_num[k] as Integer);
            dump_int!(self.hyf_next[k] as Integer);
        }
        self.print_nl("Hyphenation trie of length ");
        self.print_int(self.trie_max as Integer);
        self.print(" has ");
        self.print_int(self.trie_op_ptr as Integer);
        self.print(" op");
        if self.trie_op_ptr != 1 {
            self.print_char(b's');
        }
        self.print(" out of ");
        self.print_int(TRIE_OP_SIZE);
        for k in (0..=255).rev() {
            if self.trie_used[k] > MIN_QUARTERWORD {
                self.print_nl("  ");
                self.print_int(self.trie_used[k] as Integer);
                self.print(" for language ");
                self.print_int(k as Integer);
                dump_int!(k as Integer);
                dump_int!(self.trie_used[k] as Integer);
            }
        }
        // End section 1324
        
        // Section 1326
        dump_int!(self.interaction);
        dump_int!(self.format_ident as Integer);
        dump_int!(69069);
        *tracing_stats_mut() = 0;
        // End section 1326

        // Section 1329
        self.fmt_file_out.close();
        // End section 1329

        Ok(())
    }
}

impl Global {
    // Section 1303
    pub fn load_fmt_file(&mut self) -> bool {
        let bad_fmt = 'load_fmt: {
            // Section 1306
            macro_rules! undump {
                ($min:expr, $max:expr) => {
                    match self.fmt_file.read_int().filter(|&x| ($min..=$max).contains(&x)) {
                        Some(x) => x,
                        None => break 'load_fmt false,
                    }
                };
            }

            macro_rules! undump_size {
                ($min:expr, $max:expr, $s:expr) => {
                    match self.fmt_file.read_int() {
                        Some(x) if x < $min => break 'load_fmt false,
                        Some(x) if x > $max => {
                            println!("---! Must increase the {}", $s);
                            break 'load_fmt false;
                        },
                        Some(x) => x,
                        None => break 'load_fmt false,
                    }
                };
            }

            macro_rules! undump_wd {
                () => {
                    match self.fmt_file.read_wd() {
                        Some(w) => w,
                        None => break 'load_fmt false,
                    }
                };
            }

            macro_rules! undump_int {
                () => {
                    match self.fmt_file.read_int() {
                        Some(x) => x,
                        None => break 'load_fmt false,
                    }
                };
            }
            // End section 1306

            // Section 1308
            macro_rules! check_constant {
                ($x:expr) => {
                    match self.fmt_file.read_int() {
                        Some(x) if x != $x => break 'load_fmt false,
                        None => break 'load_fmt false,
                        _ => ()
                    }
                };
            }

            check_constant!(0);
            check_constant!(MEM_BOT);
            check_constant!(MEM_TOP);
            check_constant!(EQTB_SIZE);
            check_constant!(HASH_PRIME);
            check_constant!(HYPH_SIZE);
            // End section 1308

            // Section 1310
            pool_ptr_set(undump_size!(0, POOL_SIZE, "string pool size") as usize);
            str_ptr_set(undump_size!(0, MAX_STRINGS, "max strings") as usize);
            for k in 0..=str_ptr() {
                *str_start_mut(k) = undump!(0, pool_ptr() as Integer) as usize;
            }
            let mut k = 0;

            macro_rules! undump_four_ASCII {
                () => {
                    match self.fmt_file.read_wd() {
                        Some(w) => {
                            *str_pool_mut![k] = w.qqqq_b0() as u8;
                            *str_pool_mut![k + 1] = w.qqqq_b1() as u8;
                            *str_pool_mut![k + 2] = w.qqqq_b2() as u8;
                            *str_pool_mut![k + 3] = w.qqqq_b3() as u8;
                        },
                        None => break 'load_fmt false,
                    }
                };
            }

            while k + 4 < pool_ptr() {
                undump_four_ASCII!();
                k += 4;
            }
            k = pool_ptr() - 4;
            undump_four_ASCII!();
            init_str_ptr_set(str_ptr());
            init_pool_ptr_set(pool_ptr());
            // End section 1310

            // Section 1312
            self.lo_mem_max = undump!(LO_MEM_STAT_MAX + 1000, HI_MEM_STAT_MIN - 1);
            self.rover = undump!(LO_MEM_STAT_MAX + 1, self.lo_mem_max);
            let mut p = MEM_BOT;
            let mut q = self.rover;
            loop {
                for k in p..=(q + 1) {
                    *mem_mut(k as usize) = undump_wd!();
                }
                p = q + node_size(q);
                if p > self.lo_mem_max || (q >= rlink(q) && rlink(q) != self.rover) {
                    break 'load_fmt false;
                }
                q = rlink(q);
                if q == self.rover {
                    break;
                }
            }
            for k in p..=self.lo_mem_max {
                *mem_mut(k as usize) = undump_wd!();
            }
            // This is never true: if MEM_MIN < MEM_BOT - 2
            self.hi_mem_min = undump!(self.lo_mem_max + 1, HI_MEM_STAT_MIN);
            self.avail = undump!(NULL, MEM_TOP);
            self.mem_end = MEM_TOP;
            for k in self.hi_mem_min..=self.mem_end {
                *mem_mut(k as usize) = undump_wd!();
            }
            self.var_used = undump_int!();
            self.dyn_used = undump_int!();
            // End section 1312

            // Section 1314
            // Section 1317
            let mut k = ACTIVE_BASE;
            loop {
                let mut x = undump_int!();
                if x < 1 || k + x > EQTB_SIZE + 1 {
                    break 'load_fmt false;
                }
                for j in k..(k + x) {
                    *eqtb_mut(j as usize) = undump_wd!();
                }
                k += x;
                x = undump_int!();
                if x < 0 || k + x > EQTB_SIZE + 1 {
                    break 'load_fmt false;
                }
                for j in k..(k + x) {
                    *eqtb_mut(j as usize) = eqtb((k - 1) as usize);
                }
                k += x;

                if k > EQTB_SIZE {
                    break;
                }
            }
            // End section 1317

            self.par_loc = undump!(HASH_BASE, FROZEN_CONTROL_SEQUENCE);
            self.par_token = CS_TOKEN_FLAG + self.par_loc;
            self.write_loc = undump!(HASH_BASE, FROZEN_CONTROL_SEQUENCE);

            // Section 1319
            self.hash_used = undump!(HASH_BASE, FROZEN_CONTROL_SEQUENCE);
            p = HASH_BASE - 1;
            loop {
                p = undump!(p + 1, self.hash_used);
                *hash_mut(p as usize) = undump_wd!();
                if p == self.hash_used {
                    break;
                }
            }
            for p in (self.hash_used + 1)..UNDEFINED_CONTROL_SEQUENCE {
                *hash_mut(p as usize) = undump_wd!();
            }
            self.cs_count = undump_int!();
            // End section 1319
            // End section 1314

            // Section 1321
            self.fmem_ptr = undump_size!(7, FONT_MEM_SIZE, "font meme size") as usize;
            for k in 0..self.fmem_ptr {
                self.font_info[k] = undump_wd!();
            }
            self.font_ptr = undump_size!(FONT_BASE, FONT_MAX, "font max") as QuarterWord;
            for k in (NULL_FONT as usize)..=(self.font_ptr as usize) {
                // Section 1323
                self.font_check[k] = undump_wd!();
                self.font_size[k] = undump_int!();
                self.font_dsize[k] = undump_int!();
                self.font_params[k] = undump!(MIN_HALFWORD, MAX_HALFWORD) as usize;
                self.hyphen_char[k] = undump_int!();
                self.skew_char[k] = undump_int!();
                self.font_name[k] = undump!(0, str_ptr() as Integer) as usize;
                self.font_area[k] = undump!(0, str_ptr() as Integer) as usize;
                self.font_bc[k] = undump!(0, 255) as u8;
                self.font_ec[k] = undump!(0, 255) as u8;
                self.char_base[k] = undump_int!();
                self.width_base[k] = undump_int!();
                self.height_base[k] = undump_int!();
                self.depth_base[k] = undump_int!();
                self.italic_base[k] = undump_int!();
                self.lig_kern_base[k] = undump_int!();
                self.kern_base[k] = undump_int!();
                self.exten_base[k] = undump_int!();
                self.param_base[k] = undump_int!();
                self.font_glue[k] = undump!(MIN_HALFWORD, self.lo_mem_max);
                self.bchar_label[k] = undump!(0, (self.fmem_ptr - 1) as Integer) as usize;
                self.font_bchar[k] = undump!(MIN_QUARTERWORD as Integer, NON_CHAR) as usize;
                self.font_false_bchar[k] = undump!(MIN_QUARTERWORD as Integer, NON_CHAR) as usize;
                // End section 1323
            }
            // End section 1321

            // Section 1325
            self.hyph_count = undump!(0, HYPH_SIZE);
            for _ in 1..=self.hyph_count {
                let j = undump!(0, HYPH_SIZE) as usize;
                self.hyph_word[j] = undump!(0, str_ptr() as Integer) as usize;
                self.hyph_list[j] = undump!(MIN_HALFWORD, MAX_HALFWORD);
            }
            let mut j = undump_size!(0, TRIE_SIZE, "trie size") as usize;
            if self.initex_mode {
                self.trie_max = j;
            }
            for k in 0..=j {
                self.trie[k] = undump_wd!();
            }

            j = undump_size!(0, TRIE_OP_SIZE, "trie op size") as usize;
            if self.initex_mode {
                self.trie_op_ptr = j;
            }
            for k in 1..=j {
                self.hyf_distance[k] = undump!(0, 63) as QuarterWord;
                self.hyf_num[k] = undump!(0, 63) as QuarterWord;
                self.hyf_next[k] = undump!(MIN_QUARTERWORD as Integer, MAX_QUARTERWORD as Integer) as QuarterWord;
            }

            if self.initex_mode {
                for k in 0..=255 {
                    self.trie_used[k] = MIN_QUARTERWORD;
                }
            }
            let mut k = 256;
            while j > 0 {
                k = undump!(0, k - 1);
                let x = undump!(1, j as Integer) as usize;
                if self.initex_mode {
                    self.trie_used[k as usize] = x as QuarterWord;
                }
                j -= x;
                self.op_start[k as usize] = j as usize;
            }
            if self.initex_mode {
                self.trie_not_ready = false;
            }
            // End section 1325

            // Section 1327
            // Interaction is restricted to BATCH_MODE and ERROR_STOP_MODE only.
            // If NONSTOP_MODE or SCROLL_MODE are met, it falls back to BATCH_MODE.
            self.interaction = match undump!(0, 3) {
                ERROR_STOP_MODE => ERROR_STOP_MODE,
                _ => BATCH_MODE,
            };
            self.format_ident = undump!(0, str_ptr() as Integer) as usize;
            undump_int!() != 69069
            // End section 1327
        };
        
        if bad_fmt {
            println!("(Fatal format file error; I'm stymied)");
        }
        !bad_fmt
    }
}
