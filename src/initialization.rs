use crate::breaker::{
    Array1to6, HyfArray, line_number_mut
};
use crate::constants::*;
use crate::datastructures::{
    EQTB, HASH, MEM, XEQ_LEVEL, InStateRecord, InputFile, LineStack,
    ListStateRecord, MemoryWord, Status, box_mut, cat_code_mut, cur_font_mut,
    day_mut, del_code_mut, end_line_char_mut, eq_level_mut, eq_type_mut,
    equiv_mut, escape_char_mut, glue_ref_count_mut, hang_after_mut, info_mut,
    lc_code_mut, link_mut, llink_mut, mag, mag_mut, math_code_mut,
    max_dead_cycles_mut, month_mut, next_mut, node_size_mut, par_shape_ptr_mut,
    rlink_mut, sf_code_mut, shrink_mut, shrink_order_mut, stretch_mut,
    stretch_order_mut, subtype_mut, text_mut, time_mut, tolerance_mut, type_mut,
    uc_code_mut, year_mut
};
use crate::error::{TeXError, TeXResult};
use crate::io::{
    AlphaFileIn, AlphaFileOut, ByteFileIn, ByteFileOut
};
use crate::parser::{TrieOpHash, TrieTaken};
use crate::strings::str_ptr;
use crate::{
    Global, HalfWord, Integer, QuarterWord, SmallNumber,
    eqtb, eqtb_mut, hash, hash_mut, hi, page_depth, update_terminal
};

use std::io::Write;

// Initialization
impl Default for Global {
    fn default() -> Self {
        Self {
            initex_mode: false,

            // Section 26
            name_of_file: String::new(),

            // Section 30
            buffer: [0; (BUF_SIZE + 1) as usize],
            buffer_string: String::new(),
            first: 0,
            last: 0,
            max_buf_stack: 0,

            // Section 54
            log_file: AlphaFileOut::new(),
            selector: 0,
            dig: [0; 23],
            tally: 0,
            term_offset: 0,
            file_offset: 0,
            trick_buf: [0; (ERROR_LINE + 1) as usize],
            trick_count: 0,
            first_count: 0,

            // Section 73
            interaction: 0,

            // Section 76
            set_box_allowed: false,
            history: 0,

            // Section 96
            interrupt: false,
            ok_to_interrupt: false,

            // Section 115
            temp_ptr: 0,

            // Section 116
            lo_mem_max: 0,
            hi_mem_min: 0,

            // Section 117
            var_used: 0,
            dyn_used: 0,

            // Section 118
            avail: 0,
            mem_end: 0,

            // Section 124
            rover: 0,

            // Section 165
            #[cfg(feature = "debug")]
            free: [false; (MEM_MAX - MEM_MIN  +1) as usize],
            #[cfg(feature = "debug")]
            was_free: [false; (MEM_MAX - MEM_MIN  + 1) as usize],
            #[cfg(feature = "debug")]
            was_mem_end: 0,
            #[cfg(feature = "debug")]
            was_lo_max: 0,
            #[cfg(feature = "debug")]
            was_hi_min: 0,
            #[cfg(feature = "debug")]
            panicking: false,

            // Section 173
            font_in_short_display: 0,

            // Section 181
            depth_threshold: 0,
            breadth_max: 0,

            // Section 213
            nest: [ListStateRecord::default(); (NEST_SIZE + 1) as usize],
            nest_ptr: 0,
            max_nest_stack: 0,
            cur_list: ListStateRecord::default(),
            shown_mode: 0, // -MMODE..MMODE

            // Section 246
            old_setting: 0,

            // Section 256
            hash_used: 0,
            no_new_control_sequence: false,
            cs_count: 0,

            // Section 271
            save_stack: [MemoryWord::ZERO; (SAVE_SIZE + 1) as usize],
            save_ptr: 0,
            max_save_stack: 0,
            cur_level: 0,
            cur_group: 0,
            cur_boundary: 0,

            // Sectin 286
            mag_set: 0,

            // Section 297
            cur_cmd: 0, // instead of u8
            cur_chr: 0,
            cur_cs: 0,
            cur_tok: 0,

            // Section 301
            input_stack: [InStateRecord::default(); (STACK_SIZE + 1) as usize],
            input_ptr: 0,
            max_in_stack: 0,
            cur_input: InStateRecord::default(),

            // Section 304
            in_open: 0,
            open_parens: 0,
            input_file: InputFile([AlphaFileIn::INIT; MAX_IN_OPEN as usize]),
            line: 0,
            line_stack: LineStack([0; MAX_IN_OPEN as usize]),

            // Section 305
            scanner_status: Status::Normal,
            warning_index: 0,
            def_ref: 0,

            // Section 308
            param_stack: [0; (PARAM_SIZE + 1) as usize],
            param_ptr: 0,
            max_param_stack: 0,

            // Section 309
            align_state: 0,

            // Section 310
            base_ptr: 0,

            // Section 333
            par_loc: 0,
            par_token: 0,
            
            // Section 361
            force_eof: false,

            // Section 382
            cur_mark: [0; (SPLIT_BOT_MARK_CODE + 1) as usize],

            // Section 387
            long_state: 0,

            // Section 388
            pstack: [0; 9],

            // Section 410
            cur_val: 0,
            cur_val_level: 0,

            // Section 438
            radix: 0,

            // Section 447
            cur_order: 0,

            // Section 480
            read_file: [AlphaFileIn::INIT; 16],
            read_open: [0; 17],

            // Section 489
            cond_ptr: 0,
            if_limit: 0,
            cur_if: 0,
            if_line: 0,

            // Section 493
            skip_line: 0,

            // Section 512
            cur_name: 0,
            cur_area: 0,
            cur_ext: 0,

            // Section 513
            area_delimiter: 0,
            ext_delimiter: 0,

            // Section 527
            name_in_progress: false,
            job_name: 0,
            log_opened: false,

            // Section 532
            dvi_file: ByteFileOut::new(),
            output_file_name: 0,
            log_name: 0,

            // Section 539
            tfm_file: ByteFileIn::new(),

            // Section 549
            font_info: [MemoryWord::ZERO; (FONT_MEM_SIZE + 1) as usize],
            fmem_ptr: 0,
            font_ptr: 0,
            font_check: [MemoryWord::ZERO; (FONT_MAX - FONT_BASE + 1) as usize],
            font_size: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_dsize: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_params: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_name: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_area: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_bc: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_ec: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_glue: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_used: [false; (FONT_MAX - FONT_BASE + 1) as usize],
            hyphen_char: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            skew_char: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            bchar_label: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_bchar: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            font_false_bchar: [0; (FONT_MAX - FONT_BASE + 1) as usize],

            // Section 550
            char_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            width_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            height_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            depth_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            italic_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            lig_kern_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            kern_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            exten_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],
            param_base: [0; (FONT_MAX - FONT_BASE + 1) as usize],

            // Section 555
            null_character: MemoryWord::ZERO,

            // Section 592
            total_pages: 0,
            max_v: 0,
            max_h: 0,
            max_push: 0,
            last_bop: 0,
            dead_cycles: 0,
            doing_leaders: false,
            c: 0,
            f: 0,
            rule_ht: 0,
            rule_dp: 0,
            rule_wd: 0,
            g: 0,
            lq: 0,
            lr: 0,

            // Section 595
            dvi_buf: [0; (DVI_BUF_SIZE + 1) as usize],
            half_buf: 0,
            dvi_limit: 0,
            dvi_ptr: 0,
            dvi_offset: 0,
            dvi_gone: 0,

            // Section 605
            down_ptr: 0,
            right_ptr: 0,

            // Section 616
            dvi_h: 0,
            dvi_v: 0,
            cur_h: 0,
            cur_v: 0,
            dvi_f: 0,
            cur_s: 0,

            // Section 646
            total_stretch: [0; 4],
            total_shrink: [0; 4],
            last_badness: 0,

            // Section 647
            adjust_tail: 0,

            // Section 661
            pack_begin_line: 0,

            // Section 684
            empty_field: MemoryWord::ZERO,
            null_delimiter: MemoryWord::ZERO,

            // Section 719
            cur_mlist: 0,
            cur_style: 0,
            cur_size: 0,
            cur_mu: 0,
            mlist_penalties: false,

            // Section 724
            cur_f: 0,
            cur_c: 0,
            cur_i: MemoryWord::ZERO,

            // Section 770
            cur_align: 0,
            cur_span: 0,
            cur_loop: 0,
            align_ptr: 0,
            cur_head: 0,
            cur_tail: 0,

            // Section 814
            just_box: 0,

            // Section 821
            passive: 0,
            printed_node: 0,
            pass_number: 0,

            // Section 823
            active_width: Array1to6::default(),
            cur_active_width: Array1to6::default(),
            background: Array1to6::default(),
            break_width: Array1to6::default(),

            // Section 828
            cur_p: 0,
            second_pass: false,
            final_pass: false,
            threshold: 0,

            // Section 833
            minimal_demerits: [0; 4],
            minimum_demerits: 0,
            best_place: [0; 4],
            best_pl_line: [0; 4],

            // Section 839
            disc_width: 0,

            // Section 847
            easy_line: 0,
            last_special_line: 0,
            first_width: 0,
            second_width: 0,
            first_indent: 0,
            second_indent: 0,

            // Section 872
            best_bet: 0,
            fewest_demerits: 0,
            best_line: 0,
            actual_looseness: 0,
            line_diff: 0,

            // Section 892
            hc: [0; 66],
            hn: 0,
            ha: 0,
            hb: 0,
            hf: 0,
            hu: [0; 64],
            hyf_char: 0,
            cur_lang: 0,
            init_cur_lang: 0,
            l_hyf: 0,
            r_hyf: 0,
            init_l_hyf: 0,
            init_r_hyf: 0,
            hyf_bchar: 0,

            // Section 900
            hyf: [0; 65],
            init_list: 0,
            init_lig: false,
            init_lft: false,

            // Section 905
            hyphen_passed: 0,

            // Section 907
            cur_l: 0,
            cur_r: 0,
            cur_q: 0,
            lig_stack: 0,
            ligature_present: false,
            lft_hit: false,
            rt_hit: false,

            // Section 921
            // hyf_distance, hyf_num, hyf_next:
            // we start indexing with 1, value in index 0 is unused.
            trie: [MemoryWord::ZERO; (TRIE_SIZE + 1) as usize],
            hyf_distance: HyfArray::default(),
            hyf_num: HyfArray::default(),
            hyf_next: HyfArray::default(),
            op_start: [0; 256],

            // Section 926
            hyph_word: [0; (HYPH_SIZE + 1) as usize],
            hyph_list: [0; (HYPH_SIZE + 1) as usize],
            hyph_count: 0,

            // Section 943
            trie_op_hash: TrieOpHash::new(),
            trie_used: [0; 256],
            trie_op_lang: [0; (TRIE_OP_SIZE + 1) as usize],
            trie_op_val: [0; (TRIE_OP_SIZE + 1) as usize],
            trie_op_ptr: 0,

            // Section 947
            trie_c: [0; (TRIE_SIZE + 1) as usize],
            trie_o: [0; (TRIE_SIZE + 1) as usize],
            trie_l: [0; (TRIE_SIZE + 1) as usize],
            trie_r: [0; (TRIE_SIZE + 1) as usize],
            trie_ptr: 0,
            trie_hash: [0; (TRIE_SIZE + 1) as usize],

            // Section 950
            trie_taken: TrieTaken::default(),
            trie_min: [0; 256],
            trie_max: 0,
            trie_not_ready: false,

            // Section 971
            best_height_plus_depth: 0,

            // Section 980
            page_tail: 0,
            page_contents: 0,
            page_max_depth: 0,
            best_page_break: 0,
            least_page_cost: 0,
            best_size: 0,

            // Section 982
            page_so_far: [0; 8],
            last_glue: 0,
            last_penalty: 0,
            last_kern: 0,
            insert_penalties: 0,

            // Section 989
            output_active: false,

            // Section 1032
            main_f: 0,
            main_i: MemoryWord::ZERO,
            main_j: MemoryWord::ZERO,
            main_k: 0,
            main_p: 0,
            main_s: 0,
            bchar: 0,
            false_bchar: 0,
            cancel_boundary: false,
            ins_disc: false,

            // Section 1074
            cur_box: 0,

            // Section 1266
            after_token: 0,

            // Section 1299
            format_ident: 0,

            // Section 1305
            fmt_file: ByteFileIn::new(),
            fmt_file_out: ByteFileOut::new(),
            
            // Section 1342
            write_file: [AlphaFileOut::INIT; 16],
            write_open: [false; 18],

            // Section 1345
            write_loc: 0,
        }
    }
}

impl Global {
    pub fn initialize_output_routines(&mut self) {
        // Section 55
        self.selector = TERM_ONLY;
        self.tally = 0;
        self.term_offset = 0;
        self.file_offset = 0;
        // End section 55

        // Section 61
        print!("{BANNER}");
        if self.format_ident == 0 {
            println!(" (no format preloaded)");
        }
        else {
            self.slow_print(self.format_ident);
            self.print_ln();
        }
        update_terminal!();
        // End section 61

        // Section 528
        self.job_name = 0;
        self.name_in_progress = false;
        self.log_opened = false;
        // End section 528

        // Section 533
        self.output_file_name = 0;
        // End section 533
    }

    // Section 288
    pub(crate) fn prepare_mag(&mut self) -> TeXResult<()> {
        if self.mag_set > 0 && mag() != self.mag_set {
            return Err(TeXError::IncompatibleMag);
        }
        if mag() <= 0 || mag() > 32768 {
            return Err(TeXError::IllegalMag(mag()));
        }
        self.mag_set = mag();
        Ok(())
    }

    fn sec164_initialize_table_entries(&mut self) {
        // Section 164
        unsafe {
            for word in MEM[(MEM_BOT + 1) as usize..=(LO_MEM_STAT_MAX as usize)].iter_mut() {
                word.sc = 0;
            }
        }
        let mut k = MEM_BOT;
        while k <= LO_MEM_STAT_MAX {
            *glue_ref_count_mut(k) = NULL + 1;
            *stretch_order_mut(k) = NORMAL;
            *shrink_order_mut(k) = NORMAL;
            k += GLUE_SPEC_SIZE;
        }

        *stretch_mut(FIL_GLUE) = UNITY;
        *stretch_order_mut(FIL_GLUE) = FIL;

        *stretch_mut(FILL_GLUE) = UNITY;
        *stretch_order_mut(FILL_GLUE) = FILL;

        *stretch_mut(SS_GLUE) = UNITY;
        *stretch_order_mut(SS_GLUE) = FIL;

        *shrink_mut(SS_GLUE) = UNITY;
        *shrink_order_mut(SS_GLUE) = FIL;

        *stretch_mut(FIL_NEG_GLUE) = -UNITY;
        *stretch_order_mut(FIL_NEG_GLUE) = FIL;

        self.rover = LO_MEM_STAT_MAX + 1;
        *link_mut(self.rover) = EMPTY_FLAG;
        *node_size_mut(self.rover) = 1000;
        *llink_mut(self.rover) = self.rover;
        *rlink_mut(self.rover) = self.rover;
        self.lo_mem_max = self.rover + 1000;
        *link_mut(self.lo_mem_max) = NULL;
        *info_mut(self.lo_mem_max) = NULL;
        unsafe {
            MEM[(HI_MEM_STAT_MIN as usize)..=(MEM_TOP as usize)].fill(MEM[self.lo_mem_max as usize]);
        }

        // Section 790
        *info_mut(OMIT_TEMPLATE) = END_TEMPLATE_TOKEN;
        // End section 790

        // Section 797
        *link_mut(END_SPAN) = (MAX_QUARTERWORD as HalfWord) + 1;
        *info_mut(END_SPAN) = NULL;
        // End section 797

        // Section 820
        *type_mut(LAST_ACTIVE) = HYPHENATED;
        *line_number_mut(LAST_ACTIVE) = MAX_HALFWORD;
        *subtype_mut(LAST_ACTIVE) = 0;
        // End section 820

        // Section 981
        *subtype_mut(PAGE_INS_HEAD) = 255;
        *type_mut(PAGE_INS_HEAD) = SPLIT_UP;
        *link_mut(PAGE_INS_HEAD) = PAGE_INS_HEAD;
        // End section 981

        // Section 988
        *type_mut(PAGE_HEAD) = GLUE_NODE;
        *subtype_mut(PAGE_HEAD) = NORMAL;
        // End section 988
        // End section 790

        self.avail = NULL;
        self.mem_end = MEM_TOP;
        self.hi_mem_min = HI_MEM_STAT_MIN;
        self.var_used = LO_MEM_STAT_MAX + 1 - MEM_BOT;
        self.dyn_used = HI_MEM_STAT_USAGE;
        // End section 164

        // Section 222
        *eq_type_mut(UNDEFINED_CONTROL_SEQUENCE) = UNDEFINED_CS;
        *equiv_mut(UNDEFINED_CONTROL_SEQUENCE) = NULL;
        *eq_level_mut(UNDEFINED_CONTROL_SEQUENCE) = LEVEL_ZERO;
        for k in ACTIVE_BASE..UNDEFINED_CONTROL_SEQUENCE {
            *eqtb_mut![k as usize] = eqtb![UNDEFINED_CONTROL_SEQUENCE as usize];
        }
        // End section 222

        // Section 228
        *equiv_mut(GLUE_BASE) = ZERO_GLUE;
        *eq_level_mut(GLUE_BASE) = LEVEL_ONE;
        *eq_type_mut(GLUE_BASE) = GLUE_REF;
        for k in (GLUE_BASE + 1)..LOCAL_BASE {
            *eqtb_mut![k as usize] = eqtb![GLUE_BASE as usize];
        }
        *glue_ref_count_mut(ZERO_GLUE) += LOCAL_BASE - GLUE_BASE;
        // End section 228

        // Section 232
        *par_shape_ptr_mut() = NULL;
        *eq_type_mut(PAR_SHAPE_LOC) = SHAPE_REF;
        *eq_level_mut(PAR_SHAPE_LOC) = LEVEL_ONE;
        for k in OUTPUT_ROUTINE_LOC..=(TOKS_BASE + 255) {
            *eqtb_mut![k as usize] = eqtb![UNDEFINED_CONTROL_SEQUENCE as usize];
        }
        *box_mut(0) = NULL;
        *eq_type_mut(BOX_BASE) = BOX_REF;
        *eq_level_mut(BOX_BASE) = LEVEL_ONE;
        for k in (BOX_BASE + 1)..=(BOX_BASE + 255) {
            *eqtb_mut![k as usize] = eqtb![BOX_BASE as usize];
        }
        *cur_font_mut() = NULL_FONT;
        *eq_type_mut(CUR_FONT_LOC) = DATA ;
        *eq_level_mut(CUR_FONT_LOC) = LEVEL_ONE;
        for k in MATH_FONT_BASE..=(MATH_FONT_BASE + 47) {
            *eqtb_mut![k as usize] = eqtb![CUR_FONT_LOC as usize];
        }
        *equiv_mut(CAT_CODE_BASE) = 0;
        *eq_type_mut(CAT_CODE_BASE) = DATA;
        *eq_level_mut(CAT_CODE_BASE) = LEVEL_ONE;
        for k in (CAT_CODE_BASE + 1)..INT_BASE {
            *eqtb_mut![k as usize] = eqtb![CAT_CODE_BASE as usize];
        }
        for k in 0..=255 {
            *cat_code_mut(k) = OTHER_CHAR as HalfWord;
            *math_code_mut(k) = hi!(k);
            *sf_code_mut(k) = 1000;
        }
        *cat_code_mut(CARRIAGE_RETURN) = CAR_RET as HalfWord;
        *cat_code_mut(b' ' as HalfWord) = SPACER as HalfWord;
        *cat_code_mut(b'\\' as HalfWord) = ESCAPE as HalfWord;
        *cat_code_mut(b'%' as HalfWord) = COMMENT as HalfWord;
        *cat_code_mut(INVALID_CODE) = INVALID_CHAR as HalfWord;
        *cat_code_mut(NULL_CODE) = IGNORE as HalfWord;
        for k in b'0'..=b'9' {
            *math_code_mut(k as HalfWord) = hi!(k as Integer + VAR_CODE);
        }
        for k in (b'A' as HalfWord)..=(b'Z' as HalfWord) {
            *cat_code_mut(k) = LETTER as HalfWord;
            *cat_code_mut(k + (b'a' - b'A') as HalfWord) = LETTER as HalfWord;
            *math_code_mut(k) = hi!(k + VAR_CODE as HalfWord + 256);
            *math_code_mut(k + (b'a' - b'A') as HalfWord) = hi!((k + (b'a' - b'A') as HalfWord + VAR_CODE + 256));
            *lc_code_mut(k) = k + (b'a' - b'A') as HalfWord;
            *lc_code_mut(k + (b'a' - b'A') as HalfWord) = k + (b'a' - b'A') as HalfWord;
            *uc_code_mut(k) = k;
            *uc_code_mut(k + (b'a' - b'A') as HalfWord) = k;
            *sf_code_mut(k) = 999;
        }
        // End section 232

        // Section 240
        for k in INT_BASE..DEL_CODE_BASE {
            *eqtb_mut![k as usize].int_mut() = 0;
        }
        *mag_mut() = 1000;
        *tolerance_mut() = 10000;
        *hang_after_mut() = 1;
        *max_dead_cycles_mut() = 25;
        *escape_char_mut() = b'\\' as Integer;
        *end_line_char_mut() = CARRIAGE_RETURN;
        for k in 0..=255 {
            *del_code_mut(k) = -1;
        }
        *del_code_mut(b'.' as HalfWord) = 0;
        // End section 240

        // Section 250
        for k in DIMEN_BASE..=EQTB_SIZE {
            *eqtb_mut![k as usize].sc_mut() = 0;
        }
        // End section 250

        // Section 258
        self.hash_used = FROZEN_CONTROL_SEQUENCE;
        self.cs_count = 0;
        *eq_type_mut(FROZEN_DONT_EXPAND) = DONT_EXPAND;
        *text_mut(FROZEN_DONT_EXPAND) = NOTEXPANDED_STRING as HalfWord;
        // End section 258

        // Section 552
        self.font_ptr = NULL_FONT as QuarterWord;
        self.fmem_ptr = 7;
        self.font_name[NULL_FONT as usize] = NULLFONT_STRING;
        self.font_area[NULL_FONT as usize] = EMPTY_STRING;
        self.hyphen_char[NULL_FONT as usize] = b'-' as Integer;
        self.skew_char[NULL_FONT as usize] = -1;
        self.bchar_label[NULL_FONT as usize] = NON_ADDRESS as usize;
        self.font_bchar[NULL_FONT as usize] = NON_CHAR as usize;
        self.font_false_bchar[NULL_FONT as usize] = NON_CHAR as usize;
        self.font_bc[NULL_FONT as usize] = 1;
        self.font_ec[NULL_FONT as usize] = 0;
        self.font_size[NULL_FONT as usize] = 0;
        self.font_dsize[NULL_FONT as usize] = 0;
        self.char_base[NULL_FONT as usize] = 0;
        self.width_base[NULL_FONT as usize] = 0;
        self.height_base[NULL_FONT as usize] = 0;
        self.depth_base[NULL_FONT as usize] = 0;
        self.italic_base[NULL_FONT as usize] = 0;
        self.lig_kern_base[NULL_FONT as usize] = 0;
        self.kern_base[NULL_FONT as usize] = 0;
        self.exten_base[NULL_FONT as usize] = 0;
        self.font_glue[NULL_FONT as usize] = NULL;
        self.font_params[NULL_FONT as usize] = 7;
        self.param_base[NULL_FONT as usize] = -1;
        for k in 0..=6 {
            *self.font_info[k].sc_mut() = 0;
        }
        // End section 552

        // Section 946
        for k in -TRIE_OP_SIZE..=TRIE_OP_SIZE {
            self.trie_op_hash[k] = 0;
        }
        self.trie_used.fill(MIN_QUARTERWORD);
        self.trie_op_ptr = 0;
        // End section 946

        // Section 951
        self.trie_not_ready = true;
        self.trie_l[0] = 0; // trie_root
        self.trie_c[0] = 0;
        self.trie_ptr = 0;
        // End section 951

        // Section 1216
        *text_mut(FROZEN_PROTECTION) = INACCESSIBLE_STRING as HalfWord;
        // End section 1216

        // Section 1301
        self.format_ident = INITEX_IDENT_STRING;
        // End Section 1301

        // Section 1369
        *text_mut(END_WRITE) = ENDWRITE_STRING as HalfWord;
        *eq_level_mut(END_WRITE) = LEVEL_ONE;
        *eq_type_mut(END_WRITE) = OUTER_CALL;
        *equiv_mut(END_WRITE) = NULL;
        // End section 1369
    }
}

// Source: https://howardhinnant.github.io/date_algorithms.html
fn epoch_to_date(mut days: u64) -> (Integer, Integer, Integer) {
    days += 719_468;
    let era = days / 146_097;
    let doe = days % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);

    let mp = (5 * doy + 2 ) / 153;
    let d = (doy - (153 * mp  +2 ) / 5 + 1) as Integer;
    let m = if mp < 10 {
        (mp + 3) as Integer
    }
    else {
        (mp - 9) as Integer
    };

    let y = if m <= 2 {
        (yoe + era * 400 + 1) as Integer
    }
    else {
        (yoe + era * 400) as Integer
    };

    (y, m, d)
}

// Section 241
// The time is UTC.
pub fn fix_date_and_time() {
    let now = std::time::SystemTime::now();
    match now.duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => {
            let seconds = time.as_secs();
            let days = seconds / 86_400;
            *time_mut() = ((seconds % 86_400) / 60) as Integer;
            (*year_mut(), *month_mut(), *day_mut()) = epoch_to_date(days);
        },
        Err(_) => {
            // Default date and time if there is a problem
            *time_mut() = 12 * 60;
            *day_mut() = 14;
            *month_mut() = 7;
            *year_mut() = 1789;   
        }
    }
}

impl Global {
    // Section 21
    fn sec21_set_initial_values_key_variables(&mut self) {
        // Section 74
        self.interaction = ERROR_STOP_MODE;
        // End section 74

        // Section 77
        self.set_box_allowed = true;
        // End section 77

        // Section 97
        self.interrupt = false;
        self.ok_to_interrupt = true;
        // End section 97
        
        // Section 166
        #[cfg(feature = "debug")]
        {
            self.was_mem_end = MEM_MIN;
            self.was_lo_max = MEM_MIN;
            self.was_hi_min = MEM_MAX;
            self.panicking = false;
        }

        // Section 215
        self.nest_ptr = 0;
        self.max_nest_stack = 0;
        *self.mode_mut() = VMODE;
        *self.head_mut() = CONTRIB_HEAD;
        *self.tail_mut() = CONTRIB_HEAD;
        *self.prev_depth_mut() = IGNORE_DEPTH;
        *self.mode_line_mut() = 0;
        *self.prev_graf_mut() = 0;
        self.shown_mode = 0;
        // Section 991
        self.page_contents = EMPTY as SmallNumber;
        self.page_tail = PAGE_HEAD;
        *link_mut(PAGE_HEAD) = NULL;
        self.last_glue = MAX_HALFWORD;
        self.last_penalty = 0;
        self.last_kern = 0;
        page_depth!(self) = 0;
        self.page_max_depth = 0;
        // End section 991
        // End section 215

        // Section 254
        for k in INT_BASE..=EQTB_SIZE {
            unsafe {
                XEQ_LEVEL[k as usize] = LEVEL_ONE;
            }
        }

        // Section 257
        self.no_new_control_sequence = true;
        *next_mut(HASH_BASE) = 0;
        *text_mut(HASH_BASE) = 0;
        for k in (HASH_BASE + 1)..UNDEFINED_CONTROL_SEQUENCE {
            *hash_mut![k as usize] = hash![HASH_BASE as usize];
        }

        // Section 272
        self.save_ptr = 0;
        self.cur_level = LEVEL_ONE;
        self.cur_group = BOTTOM_LEVEL;
        self.cur_boundary = 0;
        self.max_save_stack = 0;
        // End section 272

        // Section 287
        self.mag_set = 0;
        // End section 287

        // Section 383
        *self.top_mark_mut() = NULL;
        *self.first_mark_mut() = NULL;
        *self.bot_mark_mut() = NULL;
        *self.split_first_mark_mut() = NULL;
        *self.split_bot_mark_mut() = NULL;

        // Section 439
        self.cur_val = 0;
        self.cur_val_level = INT_VAL;
        self.radix = 0;
        self.cur_order = NORMAL;
        // End section 439

        // Section 481
        self.read_open[0..=16].fill(CLOSED);
        // End section 481

        // Section 490
        self.cond_ptr = NULL;
        self.if_limit = NORMAL;
        self.cur_if = 0;
        self.if_line = 0;

        // Section 551
        self.font_used.fill(false);

        // Section 556
        *self.null_character.qqqq_b0_mut() = MIN_QUARTERWORD;
        *self.null_character.qqqq_b1_mut() = MIN_QUARTERWORD;
        *self.null_character.qqqq_b2_mut() = MIN_QUARTERWORD;
        *self.null_character.qqqq_b3_mut() = MIN_QUARTERWORD;

        // Section 593
        self.total_pages = 0;
        self.max_v = 0;
        self.max_h = 0;
        self.max_push = 0;
        self.last_bop = -1;
        self.doing_leaders = false;
        self.dead_cycles = 0;
        self.cur_s = -1;

        // Section 596
        self.half_buf = (DVI_BUF_SIZE / 2) as usize;
        self.dvi_limit = DVI_BUF_SIZE as usize;
        self.dvi_ptr = 0;
        self.dvi_offset = 0;
        self.dvi_gone = 0;

        // Section 606
        self.down_ptr = NULL;
        self.right_ptr = NULL;

        // Section 648
        self.adjust_tail = NULL;
        self.last_badness = 0;

        // Section 662
        self.pack_begin_line = 0;
        // End section 662

        // Section 685
        *self.empty_field.hh_rh_mut() = EMPTY;
        *self.empty_field.hh_lh_mut() = NULL;
        *self.null_delimiter.qqqq_b0_mut() = 0;
        *self.null_delimiter.qqqq_b1_mut() = MIN_QUARTERWORD;
        *self.null_delimiter.qqqq_b2_mut() = 0;
        *self.null_delimiter.qqqq_b3_mut() = MIN_QUARTERWORD;

        // Section 771
        self.align_ptr = NULL;
        self.cur_align = NULL;
        self.cur_span = NULL;
        self.cur_loop = NULL;
        self.cur_head = NULL;
        self.cur_tail = NULL;
        // End section 771

        // Section 928
        self.hyph_word.fill(0);
        self.hyph_list.fill(NULL);
        self.hyph_count = 0;
        // End section 928

        // Section 990
        self.output_active = false;
        self.insert_penalties = 0;

        // Section 1033
        self.ligature_present = false;
        self.cancel_boundary = false;
        self.lft_hit = false;
        self.rt_hit = false;
        self.ins_disc = false;
        // End section 1033

        // Section 1267
        self.after_token = 0;
        // End section 1267

        // Section 1300
        self.format_ident = 0;
        // End section 1300

        // Section 1343
        self.write_open.fill(false);
    }

    // Section 331
    pub fn initialize_input_routines(&mut self) {
        self.input_ptr = 0;
        self.max_in_stack = 0;
        self.in_open = 0;
        self.open_parens = 0;
        self.max_buf_stack = 0;
        self.param_ptr = 0;
        self.max_param_stack = 0;
        self.buffer.fill(0);
        self.scanner_status = Status::Normal;
        self.warning_index = NULL;
        *self.state_mut() = NEW_LINE;
        *self.start_mut() = 0;
        *self.index_mut() = 0;
        self.line = 0;
        *self.name_mut() = 0;
        self.force_eof = false;
        self.align_state = 1_000_000;
        *self.loc_mut() = 0;
        self.last = 0;
        *self.limit_mut() = 0;
        self.first = 1;
    }
}

impl Global {
    fn primitives_to_hash(&mut self) -> TeXResult<()> {
        // Section 226
        self.primitive(b"lineskip", ASSIGN_GLUE, GLUE_BASE + LINE_SKIP_CODE)?;
        self.primitive(b"baselineskip", ASSIGN_GLUE, GLUE_BASE + BASELINE_SKIP_CODE)?;
        self.primitive(b"parskip", ASSIGN_GLUE, GLUE_BASE + PAR_SKIP_CODE)?;
        self.primitive(b"abovedisplayskip", ASSIGN_GLUE, GLUE_BASE + ABOVE_DISPLAY_SKIP_CODE)?;
        self.primitive(b"belowdisplayskip", ASSIGN_GLUE, GLUE_BASE + BELOW_DISPLAY_SKIP_CODE)?;
        self.primitive(b"abovedisplayshortskip", ASSIGN_GLUE, GLUE_BASE + ABOVE_DISPLAY_SHORT_SKIP_CODE)?;
        self.primitive(b"belowdisplayshortskip", ASSIGN_GLUE, GLUE_BASE + BELOW_DISPLAY_SHORT_SKIP_CODE)?;
        self.primitive(b"leftskip", ASSIGN_GLUE, GLUE_BASE + LEFT_SKIP_CODE)?;
        self.primitive(b"rightskip", ASSIGN_GLUE, GLUE_BASE + RIGHT_SKIP_CODE)?;
        self.primitive(b"topskip", ASSIGN_GLUE, GLUE_BASE + TOP_SKIP_CODE)?;
        self.primitive(b"splittopskip", ASSIGN_GLUE, GLUE_BASE + SPLIT_TOP_SKIP_CODE)?;
        self.primitive(b"tabskip", ASSIGN_GLUE, GLUE_BASE + TAB_SKIP_CODE)?;
        self.primitive(b"spaceskip", ASSIGN_GLUE, GLUE_BASE + SPACE_SKIP_CODE)?;
        self.primitive(b"xspaceskip", ASSIGN_GLUE, GLUE_BASE + XSPACE_SKIP_CODE)?;
        self.primitive(b"parfillskip", ASSIGN_GLUE, GLUE_BASE + PAR_FILL_SKIP_CODE)?;
        self.primitive(b"thinmuskip", ASSIGN_MU_GLUE, GLUE_BASE + THIN_MU_SKIP_CODE)?;
        self.primitive(b"medmuskip", ASSIGN_MU_GLUE, GLUE_BASE + MED_MU_SKIP_CODE)?;
        self.primitive(b"thickmuskip", ASSIGN_MU_GLUE, GLUE_BASE + THICK_MU_SKIP_CODE)?;

        // Section 230
        self.primitive(b"output", ASSIGN_TOKS, OUTPUT_ROUTINE_LOC)?;
        self.primitive(b"everypar", ASSIGN_TOKS, EVERY_PAR_LOC)?;
        self.primitive(b"everymath", ASSIGN_TOKS, EVERY_MATH_LOC)?;
        self.primitive(b"everydisplay", ASSIGN_TOKS, EVERY_DISPLAY_LOC)?;
        self.primitive(b"everyhbox", ASSIGN_TOKS, EVERY_HBOX_LOC)?;
        self.primitive(b"everyvbox", ASSIGN_TOKS, EVERY_VBOX_LOC)?;
        self.primitive(b"everyjob", ASSIGN_TOKS, EVERY_JOB_LOC)?;
        self.primitive(b"everycr", ASSIGN_TOKS, EVERY_CR_LOC)?;
        self.primitive(b"errhelp", ASSIGN_TOKS, ERR_HELP_LOC)?;

        // Section 238
        self.primitive(b"pretolerance", ASSIGN_INT, INT_BASE + PRETOLERANCE_CODE)?;
        self.primitive(b"tolerance", ASSIGN_INT, INT_BASE + TOLERANCE_CODE)?;
        self.primitive(b"linepenalty", ASSIGN_INT, INT_BASE + LINE_PENALTY_CODE)?;
        self.primitive(b"hyphenpenalty", ASSIGN_INT, INT_BASE + HYPHEN_PENALTY_CODE)?;
        self.primitive(b"exhyphenpenalty", ASSIGN_INT, INT_BASE + EX_HYPHEN_PENALTY_CODE)?;
        self.primitive(b"clubpenalty", ASSIGN_INT, INT_BASE + CLUB_PENALTY_CODE)?;
        self.primitive(b"widowpenalty", ASSIGN_INT, INT_BASE + WIDOW_PENALTY_CODE)?;
        self.primitive(b"displaywidowpenalty", ASSIGN_INT, INT_BASE + DISPLAY_WIDOW_PENALTY_CODE)?;
        self.primitive(b"brokenpenalty", ASSIGN_INT, INT_BASE + BROKEN_PENALTY_CODE)?;
        self.primitive(b"binoppenalty", ASSIGN_INT, INT_BASE + BIN_OP_PENALTY_CODE)?;
        self.primitive(b"relpenalty", ASSIGN_INT, INT_BASE + REL_PENALTY_CODE)?;
        self.primitive(b"predisplaypenalty", ASSIGN_INT, INT_BASE + PRE_DISPLAY_PENALTY_CODE)?;
        self.primitive(b"postdisplaypenalty", ASSIGN_INT, INT_BASE + POST_DISPLAY_PENALTY_CODE)?;
        self.primitive(b"interlinepenalty", ASSIGN_INT, INT_BASE + INTER_LINE_PENALTY_CODE)?;
        self.primitive(b"doublehyphendemerits", ASSIGN_INT, INT_BASE + DOUBLE_HYPHEN_DEMERITS_CODE)?;
        self.primitive(b"finalhyphendemerits", ASSIGN_INT, INT_BASE + FINAL_HYPHEN_DEMERITS_CODE)?;
        self.primitive(b"adjdemerits", ASSIGN_INT, INT_BASE + ADJ_DEMERITS_CODE)?;
        self.primitive(b"mag", ASSIGN_INT, INT_BASE + MAG_CODE)?;
        self.primitive(b"delimiterfactor", ASSIGN_INT, INT_BASE + DELIMITER_FACTOR_CODE)?;
        self.primitive(b"looseness", ASSIGN_INT, INT_BASE + LOOSENESS_CODE)?;
        self.primitive(b"time", ASSIGN_INT, INT_BASE + TIME_CODE)?;
        self.primitive(b"day", ASSIGN_INT, INT_BASE + DAY_CODE)?;
        self.primitive(b"month", ASSIGN_INT, INT_BASE + MONTH_CODE)?;
        self.primitive(b"year", ASSIGN_INT, INT_BASE + YEAR_CODE)?;
        self.primitive(b"showboxbreadth", ASSIGN_INT, INT_BASE + SHOW_BOX_BREADTH_CODE)?;
        self.primitive(b"showboxdepth", ASSIGN_INT, INT_BASE + SHOW_BOX_DEPTH_CODE)?;
        self.primitive(b"hbadness", ASSIGN_INT, INT_BASE + HBADNESS_CODE)?;
        self.primitive(b"vbadness", ASSIGN_INT, INT_BASE + VBADNESS_CODE)?;
        self.primitive(b"pausing", ASSIGN_INT, INT_BASE + PAUSING_CODE)?;
        self.primitive(b"tracingonline", ASSIGN_INT, INT_BASE + TRACING_ONLINE_CODE)?;
        self.primitive(b"tracingmacros", ASSIGN_INT, INT_BASE + TRACING_MACROS_CODE)?;
        self.primitive(b"tracingstats", ASSIGN_INT, INT_BASE + TRACING_STATS_CODE)?;
        self.primitive(b"tracingparagraphs", ASSIGN_INT, INT_BASE + TRACING_PARAGRAPHS_CODE)?;
        self.primitive(b"tracingpages", ASSIGN_INT, INT_BASE + TRACING_PAGES_CODE)?;
        self.primitive(b"tracingoutput", ASSIGN_INT, INT_BASE + TRACING_OUTPUT_CODE)?;
        self.primitive(b"tracinglostchars", ASSIGN_INT, INT_BASE + TRACING_LOST_CHARS_CODE)?;
        self.primitive(b"tracingcommands", ASSIGN_INT, INT_BASE + TRACING_COMMANDS_CODE)?;
        self.primitive(b"tracingrestores", ASSIGN_INT, INT_BASE + TRACING_RESTORES_CODE)?;
        self.primitive(b"uchyph", ASSIGN_INT, INT_BASE + UC_HYPH_CODE)?;
        self.primitive(b"outputpenalty", ASSIGN_INT, INT_BASE + OUTPUT_PENALTY_CODE)?;
        self.primitive(b"maxdeadcycles", ASSIGN_INT, INT_BASE + MAX_DEAD_CYCLES_CODE)?;
        self.primitive(b"hangafter", ASSIGN_INT, INT_BASE + HANG_AFTER_CODE)?;
        self.primitive(b"floatingpenalty", ASSIGN_INT, INT_BASE + FLOATING_PENALTY_CODE)?;
        self.primitive(b"globaldefs", ASSIGN_INT, INT_BASE + GLOBAL_DEFS_CODE)?;
        self.primitive(b"fam", ASSIGN_INT, INT_BASE + CUR_FAM_CODE)?;
        self.primitive(b"escapechar", ASSIGN_INT, INT_BASE + ESCAPE_CHAR_CODE)?;
        self.primitive(b"defaulthyphenchar", ASSIGN_INT, INT_BASE + DEFAULT_HYPHEN_CHAR_CODE)?;
        self.primitive(b"defaultskewchar", ASSIGN_INT, INT_BASE + DEFAULT_SKEW_CHAR_CODE)?;
        self.primitive(b"endlinechar", ASSIGN_INT, INT_BASE + END_LINE_CHAR_CODE)?;
        self.primitive(b"newlinechar", ASSIGN_INT, INT_BASE + NEW_LINE_CHAR_CODE)?;
        self.primitive(b"language", ASSIGN_INT, INT_BASE + LANGUAGE_CODE)?;
        self.primitive(b"lefthyphenmin", ASSIGN_INT, INT_BASE + LEFT_HYPHEN_MIN_CODE)?;
        self.primitive(b"righthyphenmin", ASSIGN_INT, INT_BASE + RIGHT_HYPHEN_MIN_CODE)?;
        self.primitive(b"holdinginserts", ASSIGN_INT, INT_BASE + HOLDING_INSERTS_CODE)?;
        self.primitive(b"errorcontextlines", ASSIGN_INT, INT_BASE + ERROR_CONTEXT_LINES_CODE)?;

        // Section 248
        self.primitive(b"parindent", ASSIGN_DIMEN, DIMEN_BASE + PAR_INDENT_CODE)?;
        self.primitive(b"mathsurround", ASSIGN_DIMEN, DIMEN_BASE + MATH_SURROUND_CODE)?;
        self.primitive(b"lineskiplimit", ASSIGN_DIMEN, DIMEN_BASE + LINE_SKIP_LIMIT_CODE)?;
        self.primitive(b"hsize", ASSIGN_DIMEN, DIMEN_BASE + HSIZE_CODE)?;
        self.primitive(b"vsize", ASSIGN_DIMEN, DIMEN_BASE + VSIZE_CODE)?;
        self.primitive(b"maxdepth", ASSIGN_DIMEN, DIMEN_BASE + MAX_DEPTH_CODE)?;
        self.primitive(b"splitmaxdepth", ASSIGN_DIMEN, DIMEN_BASE + SPLIT_MAX_DEPTH_CODE)?;
        self.primitive(b"boxmaxdepth", ASSIGN_DIMEN, DIMEN_BASE + BOX_MAX_DEPTH_CODE)?;
        self.primitive(b"hfuzz", ASSIGN_DIMEN, DIMEN_BASE + HFUZZ_CODE)?;
        self.primitive(b"vfuzz", ASSIGN_DIMEN, DIMEN_BASE + VFUZZ_CODE)?;
        self.primitive(b"delimitershortfall", ASSIGN_DIMEN, DIMEN_BASE + DELIMITER_SHORTFALL_CODE)?;
        self.primitive(b"nulldelimiterspace", ASSIGN_DIMEN, DIMEN_BASE + NULL_DELIMITER_SPACE_CODE)?;
        self.primitive(b"scriptspace", ASSIGN_DIMEN, DIMEN_BASE + SCRIPT_SPACE_CODE)?;
        self.primitive(b"predisplaysize", ASSIGN_DIMEN, DIMEN_BASE + PRE_DISPLAY_SIZE_CODE)?;
        self.primitive(b"displaywidth", ASSIGN_DIMEN, DIMEN_BASE + DISPLAY_WIDTH_CODE)?;
        self.primitive(b"displayindent", ASSIGN_DIMEN, DIMEN_BASE + DISPLAY_INDENT_CODE)?;
        self.primitive(b"overfullrule", ASSIGN_DIMEN, DIMEN_BASE + OVERFULL_RULE_CODE)?;
        self.primitive(b"hangindent", ASSIGN_DIMEN, DIMEN_BASE + HANG_INDENT_CODE)?;
        self.primitive(b"hoffset", ASSIGN_DIMEN, DIMEN_BASE + H_OFFSET_CODE)?;
        self.primitive(b"voffset", ASSIGN_DIMEN, DIMEN_BASE + V_OFFSET_CODE)?;
        self.primitive(b"emergencystretch", ASSIGN_DIMEN, DIMEN_BASE + EMERGENCY_STRETCH_CODE)?;

        // Section 265
        self.primitive(b" ", EX_SPACE, 0)?;
        self.primitive(b"/", ITAL_CORR, 0)?;
        self.primitive(b"accent", ACCENT, 0)?;
        self.primitive(b"advance", ADVANCE, 0)?;
        self.primitive(b"afterassignment", AFTER_ASSIGNMENT, 0)?;
        self.primitive(b"aftergroup", AFTER_GROUP, 0)?;
        self.primitive(b"begingroup", BEGIN_GROUP, 0)?;
        self.primitive(b"char", CHAR_NUM, 0)?;
        self.primitive(b"csname", CS_NAME, 0)?;
        self.primitive(b"delimiter", DELIM_NUM, 0)?;
        self.primitive(b"divide", DIVIDE, 0)?;
        self.primitive(b"endcsname", END_CS_NAME, 0)?;
        self.primitive(b"endgroup", END_GROUP, 0)?;
        *text_mut(FROZEN_END_GROUP) = (str_ptr() - 1) as HalfWord; // "endgroup"
        *eqtb_mut![FROZEN_END_GROUP as usize] = eqtb![self.cur_val as usize];

        self.primitive(b"expandafter", EXPAND_AFTER, 0)?;
        self.primitive(b"font", DEF_FONT, 0)?;
        self.primitive(b"fontdimen", ASSIGN_FONT_DIMEN, 0)?;
        self.primitive(b"halign", HALIGN, 0)?;
        self.primitive(b"hrule", HRULE, 0)?;
        self.primitive(b"ignorespaces", IGNORE_SPACES, 0)?;
        self.primitive(b"insert", INSERT, 0)?;
        self.primitive(b"mark", MARK, 0)?;
        self.primitive(b"mathaccent", MATH_ACCENT, 0)?;
        self.primitive(b"mathchar", MATH_CHAR_NUM, 0)?;
        self.primitive(b"mathchoice", MATH_CHOICE, 0)?;
        self.primitive(b"multiply", MULTIPLY, 0)?;
        self.primitive(b"noalign", NO_ALIGN, 0)?;
        self.primitive(b"noboundary", NO_BOUNDARY, 0)?;
        self.primitive(b"noexpand", NO_EXPAND, 0)?;
        self.primitive(b"nonscript", NON_SCRIPT, 0)?;
        self.primitive(b"omit", OMIT, 0)?;
        self.primitive(b"parshape", SET_SHAPE, 0)?;
        self.primitive(b"penalty", BREAK_PENALTY, 0)?;
        self.primitive(b"prevgraph", SET_PREV_GRAF, 0)?;
        self.primitive(b"radical", RADICAL, 0)?;
        self.primitive(b"read", READ_TO_CS, 0)?;
        self.primitive(b"relax", RELAX, 256)?;
        *text_mut(FROZEN_RELAX) = (str_ptr() - 1) as HalfWord; // "relax"
        *eqtb_mut![FROZEN_RELAX as usize] = eqtb![self.cur_val as usize];

        self.primitive(b"setbox", SET_BOX, 0)?;
        self.primitive(b"the", THE, 0)?;
        self.primitive(b"toks", TOKS_REGISTER, 0)?;
        self.primitive(b"vadjust", VADJUST, 0)?;
        self.primitive(b"valign", VALIGN, 0)?;
        self.primitive(b"vcenter", VCENTER, 0)?;
        self.primitive(b"vrule", VRULE, 0)?;

        // Section 334
        self.primitive(b"par", PAR_END, 256)?;
        self.par_loc = self.cur_val;
        self.par_token = CS_TOKEN_FLAG + self.par_loc;

        // Section 376
        self.primitive(b"input", INPUT, 0)?;
        self.primitive(b"endinput", INPUT, 1)?;

        // Section 384
        self.primitive(b"topmark", TOP_BOT_MARK, TOP_MARK_CODE)?;
        self.primitive(b"firstmark", TOP_BOT_MARK, FIRST_MARK_CODE)?;
        self.primitive(b"botmark", TOP_BOT_MARK, BOT_MARK_CODE)?;
        self.primitive(b"splitfirstmark", TOP_BOT_MARK, SPLIT_FIRST_MARK_CODE)?;
        self.primitive(b"splitbotmark", TOP_BOT_MARK, SPLIT_BOT_MARK_CODE)?;

        // Section 411
        self.primitive(b"count", REGISTER, INT_VAL)?;
        self.primitive(b"dimen", REGISTER, DIMEN_VAL)?;
        self.primitive(b"skip", REGISTER, GLUE_VAL)?;
        self.primitive(b"muskip", REGISTER, MU_VAL)?;

        // Section 416
        self.primitive(b"spacefactor", SET_AUX, HMODE)?;
        self.primitive(b"prevdepth", SET_AUX, VMODE)?;
        self.primitive(b"deadcycles", SET_PAGE_INT, 0)?;
        self.primitive(b"insertpenalties", SET_PAGE_INT, 1)?;
        self.primitive(b"wd", SET_BOX_DIMEN, WIDTH_OFFSET)?;
        self.primitive(b"ht", SET_BOX_DIMEN, HEIGHT_OFFSET)?;
        self.primitive(b"dp", SET_BOX_DIMEN, DEPTH_OFFSET)?;
        self.primitive(b"lastpenalty", LAST_ITEM, INT_VAL)?;
        self.primitive(b"lastkern", LAST_ITEM, DIMEN_VAL)?;
        self.primitive(b"lastskip", LAST_ITEM, GLUE_VAL)?;
        self.primitive(b"inputlineno", LAST_ITEM, INPUT_LINE_NO_CODE)?;
        self.primitive(b"badness", LAST_ITEM, BADNESS_CODE)?;

        // Section 468
        self.primitive(b"number", CONVERT, NUMBER_CODE)?;
        self.primitive(b"romannumeral", CONVERT, ROMAN_NUMERAL_CODE)?;
        self.primitive(b"string", CONVERT, STRING_CODE)?;
        self.primitive(b"meaning", CONVERT, MEANING_CODE)?;
        self.primitive(b"fontname", CONVERT, FONT_NAME_CODE)?;
        self.primitive(b"jobname", CONVERT, JOB_NAME_CODE)?;

        // Section 487
        self.primitive(b"if", IF_TEST, IF_CHAR_CODE)?;
        self.primitive(b"ifcat", IF_TEST, IF_CAT_CODE)?;
        self.primitive(b"ifnum", IF_TEST, IF_INT_CODE)?;
        self.primitive(b"ifdim", IF_TEST, IF_DIM_CODE)?;
        self.primitive(b"ifodd", IF_TEST, IF_ODD_CODE)?;
        self.primitive(b"ifvmode", IF_TEST, IF_VMODE_CODE)?;
        self.primitive(b"ifhmode", IF_TEST, IF_HMODE_CODE)?;
        self.primitive(b"ifmmode", IF_TEST, IF_MMODE_CODE)?;
        self.primitive(b"ifinner", IF_TEST, IF_INNER_CODE)?;
        self.primitive(b"ifvoid", IF_TEST, IF_VOID_CODE)?;
        self.primitive(b"ifhbox", IF_TEST, IF_HBOX_CODE)?;
        self.primitive(b"ifvbox", IF_TEST, IF_VBOX_CODE)?;
        self.primitive(b"ifx", IF_TEST, IFX_CODE)?;
        self.primitive(b"ifeof", IF_TEST, IF_EOF_CODE)?;
        self.primitive(b"iftrue", IF_TEST, IF_TRUE_CODE)?;
        self.primitive(b"iffalse", IF_TEST, IF_FALSE_CODE)?;
        self.primitive(b"ifcase", IF_TEST, IF_CASE_CODE)?;

        // Section 491
        self.primitive(b"fi", FI_OR_ELSE, FI_CODE)?;
        *text_mut(FROZEN_FI) = (str_ptr() - 1) as HalfWord; // "fi"
        *eqtb_mut![FROZEN_FI as usize] = eqtb![self.cur_val as usize];
        self.primitive(b"or", FI_OR_ELSE, OR_CODE)?;
        self.primitive(b"else", FI_OR_ELSE, ELSE_CODE)?;

        // Section 553
        self.primitive(b"nullfont", SET_FONT, NULL_FONT)?;
        *text_mut(FROZEN_NULL_FONT) = (str_ptr() - 1) as HalfWord; // "nullfont"
        *eqtb_mut![FROZEN_NULL_FONT as usize] = eqtb![self.cur_val as usize];

        // Section 780
        self.primitive(b"span", TAB_MARK, SPAN_CODE)?;
        self.primitive(b"cr", CAR_RET, CR_CODE)?;
        *text_mut(FROZEN_CR) = (str_ptr() - 1) as HalfWord; // "cr"
        *eqtb_mut![FROZEN_CR as usize] = eqtb![self.cur_val as usize];
        self.primitive(b"crcr", CAR_RET, CR_CR_CODE)?;
        *text_mut(FROZEN_END_TEMPLATE) = ENDTEMPLATE_STRING as HalfWord;
        *text_mut(FROZEN_ENDV) = ENDTEMPLATE_STRING as HalfWord;
        *eq_type_mut(FROZEN_ENDV) = ENDV;
        *equiv_mut(FROZEN_ENDV) = NULL_LIST;
        *eq_level_mut(FROZEN_ENDV) = LEVEL_ONE;
        *eqtb_mut![FROZEN_END_TEMPLATE as usize] = eqtb![FROZEN_ENDV as usize];
        *eq_type_mut(FROZEN_END_TEMPLATE) = END_TEMPLATE;

        // Section 983
        self.primitive(b"pagegoal", SET_PAGE_DIMEN, 0)?;
        self.primitive(b"pagetotal", SET_PAGE_DIMEN, 1)?;
        self.primitive(b"pagestretch", SET_PAGE_DIMEN, 2)?;
        self.primitive(b"pagefilstretch", SET_PAGE_DIMEN, 3)?;
        self.primitive(b"pagefillstretch", SET_PAGE_DIMEN, 4)?;
        self.primitive(b"pagefilllstretch", SET_PAGE_DIMEN, 5)?;
        self.primitive(b"pageshrink", SET_PAGE_DIMEN, 6)?;
        self.primitive(b"pagedepth", SET_PAGE_DIMEN, 7)?;

        // Section 1052
        self.primitive(b"end", STOP, 0)?;
        self.primitive(b"dump", STOP, 1)?;

        // Section 1058
        self.primitive(b"hskip", HSKIP, SKIP_CODE)?;
        self.primitive(b"hfil", HSKIP, FIL_CODE)?;
        self.primitive(b"hfill", HSKIP, FILL_CODE)?;
        self.primitive(b"hss", HSKIP, SS_CODE)?;
        self.primitive(b"hfilneg", HSKIP, FIL_NEG_CODE)?;
        self.primitive(b"vskip", VSKIP, SKIP_CODE)?;
        self.primitive(b"vfil", VSKIP, FIL_CODE)?;
        self.primitive(b"vfill", VSKIP, FILL_CODE)?;
        self.primitive(b"vss", VSKIP, SS_CODE)?;
        self.primitive(b"vfilneg", VSKIP, FIL_NEG_CODE)?;
        self.primitive(b"mskip", MSKIP, MSKIP_CODE)?;
        self.primitive(b"kern", KERN, EXPLICIT as HalfWord)?;
        self.primitive(b"mkern", MKERN, MU_GLUE as HalfWord)?;

        // Section 1071
        self.primitive(b"moveleft", HMOVE, 1)?;
        self.primitive(b"moveright", HMOVE, 0)?;
        self.primitive(b"raise", VMOVE, 1)?;
        self.primitive(b"lower", VMOVE, 0)?;
        
        self.primitive(b"box", MAKE_BOX, BOX_CODE)?;
        self.primitive(b"copy", MAKE_BOX, COPY_CODE)?;
        self.primitive(b"lastbox", MAKE_BOX, LAST_BOX_CODE)?;
        self.primitive(b"vsplit", MAKE_BOX, VSPLIT_CODE)?;
        self.primitive(b"vtop", MAKE_BOX, VTOP_CODE)?;
        self.primitive(b"vbox", MAKE_BOX, VTOP_CODE + VMODE)?;
        self.primitive(b"hbox", MAKE_BOX, VTOP_CODE + HMODE)?;
        self.primitive(b"shipout", LEADER_SHIP, (A_LEADERS - 1) as HalfWord)?;
        self.primitive(b"leaders", LEADER_SHIP, A_LEADERS as HalfWord)?;
        self.primitive(b"cleaders", LEADER_SHIP, C_LEADERS as HalfWord)?;
        self.primitive(b"xleaders", LEADER_SHIP, X_LEADERS as HalfWord)?;

        // Section 1088
        self.primitive(b"indent", START_PAR, 1)?;
        self.primitive(b"noindent", START_PAR, 0)?;

        // Section 1107
        self.primitive(b"unpenalty", REMOVE_ITEM, PENALTY_NODE as HalfWord)?;
        self.primitive(b"unkern", REMOVE_ITEM, KERN_NODE as HalfWord)?;
        self.primitive(b"unskip", REMOVE_ITEM, GLUE_NODE as HalfWord)?;
        self.primitive(b"unhbox", UN_HBOX, BOX_CODE)?;
        self.primitive(b"unhcopy", UN_HBOX, COPY_CODE)?;
        self.primitive(b"unvbox", UN_VBOX, BOX_CODE)?;
        self.primitive(b"unvcopy", UN_VBOX, COPY_CODE)?;

        // Section 1114
        self.primitive(b"-", DISCRETIONARY, 1)?;
        self.primitive(b"discretionary", DISCRETIONARY, 0)?;

        // Section 1141
        self.primitive(b"eqno", EQ_NO, 0)?;
        self.primitive(b"leqno", EQ_NO, 1)?;

        // Section 1156
        self.primitive(b"mathord", MATH_COMP, ORD_NOAD as HalfWord)?;
        self.primitive(b"mathop", MATH_COMP, OP_NOAD as HalfWord)?;
        self.primitive(b"mathbin", MATH_COMP, BIN_NOAD as HalfWord)?;
        self.primitive(b"mathrel", MATH_COMP, REL_NOAD as HalfWord)?;
        self.primitive(b"mathopen", MATH_COMP, OPEN_NOAD as HalfWord)?;
        self.primitive(b"mathclose", MATH_COMP, CLOSE_NOAD as HalfWord)?;
        self.primitive(b"mathpunct", MATH_COMP, PUNCT_NOAD as HalfWord)?;
        self.primitive(b"mathinner", MATH_COMP, INNER_NOAD as HalfWord)?;
        self.primitive(b"underline", MATH_COMP, UNDER_NOAD as HalfWord)?;
        self.primitive(b"overline", MATH_COMP, OVER_NOAD as HalfWord)?;
        self.primitive(b"displaylimits", LIMIT_SWITCH, NORMAL as HalfWord)?;
        self.primitive(b"limits", LIMIT_SWITCH, LIMITS as HalfWord)?;
        self.primitive(b"nolimits", LIMIT_SWITCH, NO_LIMITS as HalfWord)?;

        // Section 1169
        self.primitive(b"displaystyle", MATH_STYLE, DISPLAY_STYLE as HalfWord)?;
        self.primitive(b"textstyle", MATH_STYLE, TEXT_STYLE as HalfWord)?;
        self.primitive(b"scriptstyle", MATH_STYLE, SCRIPT_STYLE as HalfWord)?;
        self.primitive(b"scriptscriptstyle", MATH_STYLE, SCRIPT_SCRIPT_STYLE as HalfWord)?;

        // Secion 1178
        self.primitive(b"above", ABOVE, ABOVE_CODE)?;
        self.primitive(b"over", ABOVE, OVER_CODE)?;
        self.primitive(b"atop", ABOVE, ATOP_CODE)?;
        self.primitive(b"abovewithdelims", ABOVE, DELIMITED_ABOVE_CODE)?;
        self.primitive(b"overwithdelims", ABOVE, DELIMITED_OVER_CODE)?;
        self.primitive(b"atopwithdelims", ABOVE, DELIMITED_ATOP_CODE)?;

        // Section 1188
        self.primitive(b"left", LEFT_RIGHT, LEFT_NOAD as HalfWord)?;
        self.primitive(b"right", LEFT_RIGHT, RIGHT_NOAD as HalfWord)?;
        *text_mut(FROZEN_RIGHT) = (str_ptr() - 1) as HalfWord; // "right"
        *eqtb_mut![FROZEN_RIGHT as usize] = eqtb![self.cur_val as usize];

        // Section 1208
        self.primitive(b"long", PREFIX, 1)?;
        self.primitive(b"outer", PREFIX, 2)?;
        self.primitive(b"global", PREFIX, 4)?;
        self.primitive(b"def", DEF, 0)?;
        self.primitive(b"gdef", DEF, 1)?;
        self.primitive(b"edef", DEF, 2)?;
        self.primitive(b"xdef", DEF, 3)?;

        // Section 1219
        self.primitive(b"let", LET, NORMAL as HalfWord)?;
        self.primitive(b"futurelet", LET, (NORMAL + 1) as HalfWord)?;

        // Section 1222
        self.primitive(b"chardef", SHORTHAND_DEF, CHAR_DEF_CODE)?;
        self.primitive(b"mathchardef", SHORTHAND_DEF, MATH_CHAR_DEF_CODE)?;
        self.primitive(b"countdef", SHORTHAND_DEF, COUNT_DEF_CODE)?;
        self.primitive(b"dimendef", SHORTHAND_DEF, DIMEN_DEF_CODE)?;
        self.primitive(b"skipdef", SHORTHAND_DEF, SKIP_DEF_CODE)?;
        self.primitive(b"muskipdef", SHORTHAND_DEF, MU_SKIP_DEF_CODE)?;
        self.primitive(b"toksdef", SHORTHAND_DEF, TOKS_DEF_CODE)?;

        // Section 1230
        self.primitive(b"catcode", DEF_CODE, CAT_CODE_BASE)?;
        self.primitive(b"mathcode", DEF_CODE, MATH_CODE_BASE)?;
        self.primitive(b"lccode", DEF_CODE, LC_CODE_BASE)?;
        self.primitive(b"uccode", DEF_CODE, UC_CODE_BASE)?;
        self.primitive(b"sfcode", DEF_CODE, SF_CODE_BASE)?;
        self.primitive(b"delcode", DEF_CODE, DEL_CODE_BASE)?;
        self.primitive(b"textfont", DEF_FAMILY, MATH_FONT_BASE)?;
        self.primitive(b"scriptfont", DEF_FAMILY, MATH_FONT_BASE + SCRIPT_SIZE as HalfWord)?;
        self.primitive(b"scriptscriptfont", DEF_FAMILY, MATH_FONT_BASE + SCRIPT_SCRIPT_SIZE as HalfWord)?;

        // Section 1250
        self.primitive(b"hyphenation", HYPH_DATA, 0)?;
        self.primitive(b"patterns", HYPH_DATA, 1)?;

        // Section 1254
        self.primitive(b"hyphenchar", ASSIGN_FONT_INT, 0)?;
        self.primitive(b"skewchar", ASSIGN_FONT_INT, 1)?;

        // Section 1262
        // \nonstopmode and \scrollmode are merged with \batch_mode
        self.primitive(b"batchmode", SET_INTERACTION, BATCH_MODE)?;
        self.primitive(b"nonstopmode", SET_INTERACTION, BATCH_MODE)?;
        self.primitive(b"scrollmode", SET_INTERACTION, BATCH_MODE)?;
        self.primitive(b"errorstopmode", SET_INTERACTION, ERROR_STOP_MODE)?;

        // Section 1272
        self.primitive(b"openin", IN_STREAM, 1)?;
        self.primitive(b"closein", IN_STREAM, 0)?;

        // Section 1277
        self.primitive(b"message", MESSAGE, 0)?;
        self.primitive(b"errmessage", MESSAGE, 1)?;

        // Section 1286
        self.primitive(b"lowercase", CASE_SHIFT, LC_CODE_BASE)?;
        self.primitive(b"uppercase", CASE_SHIFT, UC_CODE_BASE)?;

        // Section 1291
        self.primitive(b"show", XRAY, SHOW_CODE)?;
        self.primitive(b"showbox", XRAY, SHOW_BOX_CODE)?;
        self.primitive(b"showthe", XRAY, SHOW_THE_CODE)?;
        self.primitive(b"showlists", XRAY, SHOW_LISTS)?;

        // Section 1344
        self.primitive(b"openout", EXTENSION, OPEN_NODE)?;
        self.primitive(b"write", EXTENSION, WRITE_NODE)?;
        self.write_loc = self.cur_val;
        self.primitive(b"closeout", EXTENSION, CLOSE_NODE)?;
        self.primitive(b"special", EXTENSION, SPECIAL_NODE)?;
        self.primitive(b"immediate", EXTENSION, IMMEDIATE_CODE)?;
        self.primitive(b"setlanguage", EXTENSION, SET_LANGUAGE_CODE)?;

        Ok(())
    }
}

impl Global {
    // The goal of this function is to verify that changes
    // in constant values is still consistent for TeX.
    // But the Rust compiler will raise an error since some comparisons
    // are obviously false for default values.
    // Example: `MEM_MIN` is 0, so MEM_MIN > 0 is always false.
    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn check_constant_values_for_consistency(&mut self) -> Integer {
        // Section 14
        let mut bad = 0;
        if HALF_ERROR_LINE < 30 || HALF_ERROR_LINE > ERROR_LINE - 15 {
            bad = 1;
        }
        if MAX_PRINT_LINE < 60 {
            bad = 2
        }
        if DVI_BUF_SIZE % 8 != 0 {
            bad = 3
        }
        if MEM_BOT + 1100 > MEM_TOP {
            bad = 4
        }
        if HASH_PRIME > HASH_SIZE {
            bad = 5
        }
        if MAX_IN_OPEN >= 128 {
            bad = 6
        }
        if MEM_TOP < 256 + 11 {
            bad = 7
        }
        // End section 14

        // Section 111
        if self.initex_mode && (MEM_MIN != MEM_BOT || MEM_MAX != MEM_TOP) {
            bad = 10;
        }
        if MEM_MIN > MEM_BOT || MEM_MAX < MEM_TOP {
            bad = 10;
        }
        if MIN_QUARTERWORD > 0 || MAX_QUARTERWORD < 127 {
            bad = 11;
        }
        if MIN_HALFWORD > 0 || MAX_HALFWORD < 32767 {
            bad = 12;
        }
        if (MIN_QUARTERWORD as Integer) < MIN_HALFWORD
            || (MAX_QUARTERWORD as Integer) > MAX_HALFWORD
        {
            bad = 13;
        }
        if MEM_MIN < MIN_HALFWORD
            || MEM_MAX >= MAX_HALFWORD
            || (MEM_BOT - MEM_MIN) > MAX_HALFWORD + 1
        {
            bad = 14;
        }
        if FONT_BASE < (MIN_QUARTERWORD as Integer)
            || FONT_MAX > (MAX_QUARTERWORD as Integer)
        {
            bad = 15;
        }
        if FONT_MAX > FONT_BASE + 256 {
            bad = 16;
        }
        if SAVE_SIZE > MAX_HALFWORD
            || MAX_STRINGS > MAX_HALFWORD
        {
            bad = 17;
        }
        if BUF_SIZE > MAX_HALFWORD {
            bad = 18;
        }
        if MAX_QUARTERWORD - MIN_QUARTERWORD < 255 {
            bad = 19;
        }
        // End section 111

        // Section 290
        if CS_TOKEN_FLAG + UNDEFINED_CONTROL_SEQUENCE > MAX_HALFWORD {
            bad = 21;
        }
        // End section 290

        // Section 1249
        if 2*MAX_HALFWORD < MEM_TOP - MEM_MIN {
            bad = 41;
        }
        // End section 1249

        bad
    }

    // Section 4
    pub fn initialize(&mut self) {
        // Section 8
        self.sec21_set_initial_values_key_variables();

        if self.initex_mode {
            self.sec164_initialize_table_entries();
        }
        // End section 8
    }

    // Section 1336
    pub fn init_prim(&mut self) -> TeXResult<()> {
        self.no_new_control_sequence = false;
        self.primitives_to_hash()?;
        self.no_new_control_sequence = true;
        Ok(())
    }
}
