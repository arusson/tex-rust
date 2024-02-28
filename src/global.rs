use crate::constants::*;
use crate::datastructures::{
    InStateRecord, InputFile, LineStack, ListStateRecord, MemoryWord, Status,
    link, new_line_char_mut, subtype
};
use crate::error::TeXResult;
use crate::io::{
    AlphaFileIn, AlphaFileOut, ByteFileIn, ByteFileOut
};
use crate::parser::{
    TrieOpHash, TrieTaken, if_line_field
};
use crate::breaker::{Array1to6, HyfArray};
use crate::{
    ASCIICode, HalfWord, Integer, QuarterWord, Scaled, SmallNumber, StrNum
};

#[cfg(feature = "stat")]
use crate::{
    datastructures::tracing_stats,
    strings::{init_pool_ptr, init_str_ptr, pool_ptr, str_ptr}
};

pub struct Global {
    pub initex_mode: bool,
    // Section 26
    pub(crate) name_of_file: String,

    // Section 30
    pub buffer: [ASCIICode; (BUF_SIZE + 1) as usize],
    pub(crate) buffer_string: String,
    pub first: Integer,
    pub last: Integer,
    pub(crate) max_buf_stack: Integer,

    // Section 54
    pub(crate) log_file: AlphaFileOut,
    pub(crate) selector: Integer,
    pub(crate) dig: [u8; 23],
    pub(crate) tally: Integer,
    pub(crate) term_offset: Integer,
    pub(crate) file_offset: Integer,
    pub(crate) trick_buf: [u8; (ERROR_LINE + 1) as usize],
    pub(crate) trick_count: Integer,
    pub(crate) first_count: Integer,

    // Section 73
    // We only use two modes:
    // - batch mode: no output on terminal
    // - error_stop_mode: stops at first opportunity
    // Any error stops the execution.
    pub(crate) interaction: Integer,

    // Section 76
    // `deletions_allowed` and `error_count` are not supported
    // (`error_count` would stop at 1).
    pub(crate) set_box_allowed: bool,
    pub history: usize,

    // Section 96
    pub(crate) interrupt: bool,
    pub(crate) ok_to_interrupt: bool,

    // Section 115
    pub(crate) temp_ptr: HalfWord,

    // Section 116
    pub(crate) lo_mem_max: HalfWord,
    pub(crate) hi_mem_min: HalfWord,

    // Section 117
    pub(crate) var_used: Integer,
    pub(crate) dyn_used: Integer,

    // Section 118
    pub(crate) avail: HalfWord,
    pub(crate) mem_end: HalfWord,

    // Section 124
    pub(crate) rover: HalfWord,

    // Section 165
    #[cfg(feature = "debug")]
    pub(crate) free: [bool; (MEM_MAX - MEM_MIN  +1) as usize],
    #[cfg(feature = "debug")]
    pub(crate) was_free: [bool; (MEM_MAX - MEM_MIN  + 1) as usize],
    #[cfg(feature = "debug")]
    pub(crate) was_mem_end: HalfWord,
    #[cfg(feature = "debug")]
    pub(crate) was_lo_max: HalfWord,
    #[cfg(feature = "debug")]
    pub(crate) was_hi_min: HalfWord,
    #[cfg(feature = "debug")]
    pub(crate) panicking: bool,

    // Section 173
    pub(crate) font_in_short_display: QuarterWord,

    // Section 181
    pub(crate) depth_threshold: Integer,
    pub(crate) breadth_max: Integer,

    // Section 213
    pub(crate) nest: [ListStateRecord; (NEST_SIZE + 1) as usize],
    pub(crate) nest_ptr: usize,
    pub(crate) max_nest_stack: usize,
    pub(crate) cur_list: ListStateRecord,
    pub(crate) shown_mode: Integer, // -MMODE..MMODE

    // Section 246
    pub(crate) old_setting: Integer,

    // Section 256
    pub(crate) hash_used: HalfWord,
    pub(crate) no_new_control_sequence: bool,
    pub(crate) cs_count: Integer,

    // Section 271
    pub(crate) save_stack: [MemoryWord; (SAVE_SIZE + 1) as usize],
    pub(crate) save_ptr: usize,
    pub(crate) max_save_stack: Integer,
    pub(crate) cur_level: QuarterWord,
    pub(crate) cur_group: Integer,
    pub(crate) cur_boundary: Integer,

    // Sectin 286
    pub(crate) mag_set: Integer,

    // Section 297
    pub(crate) cur_cmd: QuarterWord,
    pub(crate) cur_chr: HalfWord,
    pub(crate) cur_cs: HalfWord,
    pub(crate) cur_tok: HalfWord,

    // Section 301
    pub(crate) input_stack: [InStateRecord; (STACK_SIZE + 1) as usize],
    pub(crate) input_ptr: usize,
    pub(crate) max_in_stack: usize,
    pub(crate) cur_input: InStateRecord,

    // Section 304
    pub(crate) in_open: usize,
    pub(crate) open_parens: usize,
    pub(crate) input_file: InputFile,
    pub(crate) line: Integer,
    pub(crate) line_stack: LineStack,

    // Section 305
    pub(crate) scanner_status: Status,
    pub(crate) warning_index: HalfWord,
    pub(crate) def_ref: HalfWord,

    // Section 308
    pub(crate) param_stack: [HalfWord; (PARAM_SIZE + 1) as usize],
    pub(crate) param_ptr: usize,
    pub(crate) max_param_stack: usize,

    // Section 309
    pub(crate) align_state: Integer,

    // Section 310
    pub(crate) base_ptr: usize,

    // Section 333
    pub(crate) par_loc: HalfWord,
    pub(crate) par_token: HalfWord,
    
    // Section 361
    pub(crate) force_eof: bool,

    // Section 382
    pub(crate) cur_mark: [HalfWord; (SPLIT_BOT_MARK_CODE + 1) as usize],

    // Section 387
    pub(crate) long_state: QuarterWord,

    // Section 388
    pub(crate) pstack: [HalfWord; 9],

    // Section 410
    pub(crate) cur_val: Integer,
    pub(crate) cur_val_level: Integer,

    // Section 438
    pub(crate) radix: Integer,

    // Section 447
    pub(crate) cur_order: QuarterWord,

    // Section 480
    pub(crate) read_file: [AlphaFileIn; 16],
    pub(crate) read_open: [usize; 17],

    // Section 489
    pub(crate) cond_ptr: HalfWord,
    pub(crate) if_limit: QuarterWord,
    pub(crate) cur_if: QuarterWord,
    pub(crate) if_line: Integer,

    // Section 493
    pub(crate) skip_line: Integer,

    // Section 512
    pub(crate) cur_name: StrNum,
    pub(crate) cur_area: StrNum,
    pub(crate) cur_ext: StrNum,

    // Section 513
    pub(crate) area_delimiter: usize,
    pub(crate) ext_delimiter: usize,

    // Section 527
    pub(crate) name_in_progress: bool,
    pub(crate) job_name: StrNum,
    pub(crate) log_opened: bool,

    // Section 532
    pub(crate) dvi_file: ByteFileOut,
    pub(crate) output_file_name: StrNum,
    pub(crate) log_name: StrNum,

    // Section 539
    pub(crate) tfm_file: ByteFileIn,

    // Section 549
    pub(crate) font_info: [MemoryWord; (FONT_MEM_SIZE + 1) as usize],
    pub(crate) fmem_ptr: usize,
    pub(crate) font_ptr: QuarterWord,
    pub(crate) font_check: [MemoryWord; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_size: [Scaled; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_dsize: [Scaled; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_params: [usize; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_name: [StrNum; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_area: [StrNum; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_bc: [u8; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_ec: [u8; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_glue: [HalfWord; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_used: [bool; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) hyphen_char: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) skew_char: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) bchar_label: [usize; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_bchar: [usize; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) font_false_bchar: [usize; (FONT_MAX - FONT_BASE + 1) as usize],

    // Section 550
    pub(crate) char_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) width_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) height_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) depth_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) italic_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) lig_kern_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) kern_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) exten_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],
    pub(crate) param_base: [Integer; (FONT_MAX - FONT_BASE + 1) as usize],

    // Section 555
    pub(crate) null_character: MemoryWord,

    // Section 592
    pub(crate) total_pages: Integer,
    pub(crate) max_v: Scaled,
    pub(crate) max_h: Scaled,
    pub(crate) max_push: Integer,
    pub(crate) last_bop: Integer,
    pub(crate) dead_cycles: Integer,
    pub(crate) doing_leaders: bool,
    pub(crate) c: QuarterWord,
    pub(crate) f: QuarterWord,
    pub(crate) rule_ht: Scaled,
    pub(crate) rule_dp: Scaled,
    pub(crate) rule_wd: Scaled,
    pub(crate) g: HalfWord,
    pub(crate) lq: Integer,
    pub(crate) lr: Integer,

    // Section 595
    pub(crate) dvi_buf: [u8; (DVI_BUF_SIZE + 1) as usize],
    pub(crate) half_buf: usize,
    pub(crate) dvi_limit: usize,
    pub(crate) dvi_ptr: usize,
    pub(crate) dvi_offset: Integer,
    pub(crate) dvi_gone: Integer,

    // Section 605
    pub(crate) down_ptr: HalfWord,
    pub(crate) right_ptr: HalfWord,

    // Section 616
    pub(crate) dvi_h: Scaled,
    pub(crate) dvi_v: Scaled,
    pub(crate) cur_h: Scaled,
    pub(crate) cur_v: Scaled,
    pub(crate) dvi_f: usize,
    pub(crate) cur_s: Integer,

    // Section 646
    pub(crate) total_stretch: [Scaled; 4],
    pub(crate) total_shrink: [Scaled; 4],
    pub(crate) last_badness: Integer,

    // Section 647
    pub(crate) adjust_tail: HalfWord,

    // Section 661
    pub(crate) pack_begin_line: Integer,

    // Section 684
    pub(crate) empty_field: MemoryWord,
    pub(crate) null_delimiter: MemoryWord,

    // Section 719
    pub(crate) cur_mlist: HalfWord,
    pub(crate) cur_style: QuarterWord,
    pub(crate) cur_size: QuarterWord,
    pub(crate) cur_mu: Scaled,
    pub(crate) mlist_penalties: bool,

    // Section 724
    pub(crate) cur_f: QuarterWord,
    pub(crate) cur_c: QuarterWord,
    pub(crate) cur_i: MemoryWord,

    // Section 770
    pub(crate) cur_align: HalfWord,
    pub(crate) cur_span: HalfWord,
    pub(crate) cur_loop: HalfWord,
    pub(crate) align_ptr: HalfWord,
    pub(crate) cur_head: HalfWord,
    pub(crate) cur_tail: HalfWord,

    // Section 814
    pub(crate) just_box: HalfWord,

    // Section 821
    pub(crate) passive: HalfWord,
    pub(crate) printed_node: HalfWord,
    pub(crate) pass_number: HalfWord,

    // Section 823
    pub(crate) active_width: Array1to6,
    pub(crate) cur_active_width: Array1to6,
    pub(crate) background: Array1to6,
    pub(crate) break_width: Array1to6,

    // Section 828
    pub(crate) cur_p: HalfWord,
    pub(crate) second_pass: bool,
    pub(crate) final_pass: bool,
    pub(crate) threshold: Integer,

    // Section 833
    pub(crate) minimal_demerits: [Integer; 4],
    pub(crate) minimum_demerits: Integer,
    pub(crate) best_place: [HalfWord; 4],
    pub(crate) best_pl_line: [HalfWord; 4],

    // Section 839
    pub(crate) disc_width: Scaled,

    // Section 847
    pub(crate) easy_line: HalfWord,
    pub(crate) last_special_line: HalfWord,
    pub(crate) first_width: Scaled,
    pub(crate) second_width: Scaled,
    pub(crate) first_indent: Scaled,
    pub(crate) second_indent: Scaled,

    // Section 872
    pub(crate) best_bet: HalfWord,
    pub(crate) fewest_demerits: Integer,
    pub(crate) best_line: HalfWord,
    pub(crate) actual_looseness: Integer,
    pub(crate) line_diff: Integer,

    // Section 892
    pub(crate) hc: [QuarterWord; 66],
    pub(crate) hn: SmallNumber,
    pub(crate) ha: HalfWord,
    pub(crate) hb: HalfWord,
    pub(crate) hf: QuarterWord,
    pub(crate) hu: [QuarterWord; 64],
    pub(crate) hyf_char: Integer,
    pub(crate) cur_lang: ASCIICode,
    pub(crate) init_cur_lang: ASCIICode,
    pub(crate) l_hyf: Integer,
    pub(crate) r_hyf: Integer,
    pub(crate) init_l_hyf: Integer,
    pub(crate) init_r_hyf: Integer,
    pub(crate) hyf_bchar: HalfWord,

    // Section 900
    pub(crate) hyf: [QuarterWord; 65],
    pub(crate) init_list: HalfWord,
    pub(crate) init_lig: bool,
    pub(crate) init_lft: bool,

    // Section 905
    pub(crate) hyphen_passed: QuarterWord,

    // Section 907
    pub(crate) cur_l: HalfWord,
    pub(crate) cur_r: HalfWord,
    pub(crate) cur_q: HalfWord,
    pub(crate) lig_stack: HalfWord,
    pub(crate) ligature_present: bool,
    pub(crate) lft_hit: bool,
    pub(crate) rt_hit: bool,

    // Section 921
    pub(crate) trie: [MemoryWord; (TRIE_SIZE + 1) as usize],
    pub(crate) hyf_distance: HyfArray,
    pub(crate) hyf_num: HyfArray,
    pub(crate) hyf_next: HyfArray,
    pub(crate) op_start: [usize; 256],

    // Section 926
    pub(crate) hyph_word: [StrNum; (HYPH_SIZE + 1) as usize],
    pub(crate) hyph_list: [HalfWord; (HYPH_SIZE + 1) as usize],
    pub(crate) hyph_count: HalfWord,

    // Section 943
    pub(crate) trie_op_hash: TrieOpHash,
    pub(crate) trie_used: [QuarterWord; 256],
    pub(crate) trie_op_lang: [u8; (TRIE_OP_SIZE + 1) as usize],
    pub(crate) trie_op_val: [QuarterWord; (TRIE_OP_SIZE + 1) as usize],
    pub(crate) trie_op_ptr: usize,

    // Section 947
    pub(crate) trie_c: [u8; (TRIE_SIZE + 1) as usize],
    pub(crate) trie_o: [QuarterWord; (TRIE_SIZE + 1) as usize],
    pub(crate) trie_l: [usize; (TRIE_SIZE + 1) as usize],
    pub(crate) trie_r: [usize; (TRIE_SIZE + 1) as usize],
    pub(crate) trie_ptr: usize,
    pub(crate) trie_hash: [usize; (TRIE_SIZE + 1) as usize],

    // Section 950
    pub(crate) trie_taken: TrieTaken,
    pub(crate) trie_min: [usize; 256],
    pub(crate) trie_max: usize,
    pub(crate) trie_not_ready: bool,

    // Section 971
    pub(crate) best_height_plus_depth: Scaled,

    // Section 980
    pub(crate) page_tail: HalfWord,
    pub(crate) page_contents: SmallNumber,
    pub(crate) page_max_depth: Scaled,
    pub(crate) best_page_break: HalfWord,
    pub(crate) least_page_cost: Integer,
    pub(crate) best_size: Scaled,

    // Section 982
    pub(crate) page_so_far: [Scaled; 8],
    pub(crate) last_glue: HalfWord,
    pub(crate) last_penalty: Integer,
    pub(crate) last_kern: Scaled,
    pub(crate) insert_penalties: Integer,

    // Section 989
    pub(crate) output_active: bool,

    // Section 1032
    pub(crate) main_f: QuarterWord,
    pub(crate) main_i: MemoryWord,
    pub(crate) main_j: MemoryWord,
    pub(crate) main_k: Integer,
    pub(crate) main_p: HalfWord,
    pub(crate) main_s: Integer,
    pub(crate) bchar: HalfWord,
    pub(crate) false_bchar: HalfWord,
    pub(crate) cancel_boundary: bool,
    pub(crate) ins_disc: bool,

    // Section 1074
    pub(crate) cur_box: HalfWord,

    // Section 1266
    pub(crate) after_token: HalfWord,

    // Section 1299
    pub format_ident: StrNum,

    // Section 1305
    pub fmt_file: ByteFileIn,
    pub(crate) fmt_file_out: ByteFileOut,
    
    // Section 1342
    pub(crate) write_file: [AlphaFileOut; 16],
    pub(crate) write_open: [bool; 18],

    // Section 1345
    pub(crate) write_loc: HalfWord
}

impl Global {
    // Section 1333
    pub fn close_files_and_terminate(&mut self) -> TeXResult<()> {
        // Section 1378
        for k in 0..=15 {
            if self.write_open[k] {
                self.write_file[k].close();
            }
        }
        // End section 1378

        *new_line_char_mut() = -1;

        #[cfg(feature = "stat")]
        macro_rules! wlog_ln {
            ($s:expr) => {
                self.log_file.write_ln($s);
            };
        }

        #[cfg(feature = "stat")]
        macro_rules! wlog {
            ($s:expr) => {
                self.log_file.write_str($s);
            };
        }

        #[cfg(feature = "stat")]
        if tracing_stats() > 0 {
            // Section 1334
            if self.log_opened {
                wlog_ln!(" ");
                wlog_ln!("Here is how much of TeX's memory you used:");
                wlog_ln!(&format!(" {} string", str_ptr() - init_str_ptr()));
                if str_ptr() != init_str_ptr() + 1 {
                    wlog!("s");
                }
                wlog_ln!(&format!(" out of {}", MAX_STRINGS - init_str_ptr() as Integer));
                wlog_ln!(&format!(" {} string characters out of {}", pool_ptr() - init_pool_ptr(), POOL_SIZE - init_pool_ptr() as Integer));
                wlog_ln!(&format!(" {} words of memory out of {}", self.lo_mem_max - MEM_MIN + self.mem_end - self.hi_mem_min + 2, self.mem_end + 1 - MEM_MIN));
                wlog_ln!(&format!(" {} multiletter control sequences out of {}", self.cs_count, HASH_SIZE));
                wlog!(&format!(" {} words of font info for {} font", self.fmem_ptr, self.font_ptr as Integer - FONT_BASE));
                if (self.font_ptr as Integer) != FONT_BASE + 1 {
                    wlog!("s");
                }
                wlog_ln!(&format!(", out of {} for {}", FONT_MEM_SIZE, FONT_MAX - FONT_BASE));
                wlog!(&format!(" {} hyphenation exception", self.hyph_count));
                if self.hyph_count != 1 {
                    wlog!("s");
                }
                wlog_ln!(&format!(" out of {}", HYPH_SIZE));
                wlog_ln!(
                    &format!(" {}i, {}n, {}p, {}b, {}s stack positions out of {}i, {}n, {}p, {}b, {}s",
                        self.max_in_stack,
                        self.max_nest_stack,
                        self.max_param_stack,
                        self.max_buf_stack + 1,
                        self.max_save_stack + 6,
                        STACK_SIZE,
                        NEST_SIZE,
                        PARAM_SIZE,
                        BUF_SIZE,
                        SAVE_SIZE
                    )
                );
            }
            // End section 1334
        }

        self.sec642_finish_the_dvi_file()?;
        if self.log_opened {
            self.log_file.write_cr();
            self.log_file.close();
            self.selector -= 2;
            if self.selector == TERM_ONLY {
                self.print_nl("Transcript written on ");
                self.slow_print(self.log_name);
                self.print_char(b'.');
                self.print_ln();
            }
        }
        Ok(())
    }

    // Section 1335
    pub fn final_cleanup(&mut self) -> TeXResult<()> {
        let c = self.cur_chr;
        if c != 1 {
            *new_line_char_mut() = -1;
        }
        if self.job_name == 0 {
            self.open_log_file()?;
        }
        while self.input_ptr > 0 {
            if self.state() == TOKEN_LIST {
                self.end_token_list()?;
            }
            else {
                self.end_file_reading();
            }
        }
        while self.open_parens > 0 {
            self.print(" )");
            self.open_parens -= 1;
        }
        if self.cur_level > LEVEL_ONE {
            self.print_nl("(");
            self.print_esc("end occurred ");
            self.print("inside a group at level ");
            self.print_int((self.cur_level - LEVEL_ONE) as Integer);
            self.print_char(b')');
        }
        while self.cond_ptr != NULL {
            self.print_nl("(");
            self.print_esc("end occurred ");
            self.print("when ");
            self.print_cmd_chr(IF_TEST, self.cur_if as HalfWord);
            if self.if_line != 0 {
                self.print(" on line ");
                self.print_int(self.if_line);
            }
            self.print(" was incomplete)");
            self.if_line = if_line_field(self.cond_ptr);
            self.cur_if = subtype(self.cond_ptr);
            self.temp_ptr = self.cond_ptr;
            self.cond_ptr = link(self.cond_ptr);
            self.free_node(self.temp_ptr, IF_NODE_SIZE);
        }

        if self.history != SPOTLESS
            && (self.history == WARNING_ISSUED || self.interaction < ERROR_STOP_MODE)
            && self.selector == TERM_AND_LOG
        {
            self.selector = TERM_ONLY;
            self.print_nl("(see the transcript file for additional information)");
            self.selector = TERM_AND_LOG;
        }
        
        if c == 1 {
            if self.initex_mode {
                for c in TOP_MARK_CODE..=SPLIT_BOT_MARK_CODE {
                    if self.cur_mark[c as usize] != NULL {
                        self.delete_token_ref(self.cur_mark[c as usize]);
                    }
                }
                if self.last_glue != MAX_HALFWORD {
                    self.delete_glue_ref(self.last_glue);
                }
                self.store_fmt_file()?;
            }
            else {
                self.print_nl("(\\dump is performed only by INITEX)");
            }
        }
        Ok(())
    }
}
