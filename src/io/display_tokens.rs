use crate::constants::*;
use crate::datastructures::{info, link};
use crate::{
    Global, HalfWord, Integer, QuarterWord, StrNum,
};

impl Global {
    // Section 292
    pub(crate) fn show_token_list(&mut self, mut p: Integer, q: Integer, l: Integer) {
        let mut match_chr = b'#';
        let mut n = b'0';
        self.tally = 0;
        while p != NULL && self.tally < l {
            if p == q {
                // Section 320
                self.set_trick_count();
                // End section 320
            }

            // Section 293
            if p < self.hi_mem_min || p > self.mem_end {
                return self.print_esc("CLOBBERED.");
            }
            if info(p) >= CS_TOKEN_FLAG {
                self.print_cs((info(p)) - CS_TOKEN_FLAG);
            }
            else {
                let m = (info(p) / 256) as QuarterWord;
                let c = info(p) % 256;
                if info(p) < 0 {
                    self.print_esc("BAD.");
                }
                else {
                    // Section 294
                    match m {
                        LEFT_BRACE
                        | RIGHT_BRACE
                        | MATH_SHIFT
                        | TAB_MARK
                        | SUP_MARK
                        | SUB_MARK
                        | SPACER
                        | LETTER
                        | OTHER_CHAR => self.print_strnumber(c as StrNum),

                        MAC_PARAM => {
                            self.print_strnumber(c as StrNum);
                            self.print_strnumber(c as StrNum);
                        },

                        OUT_PARAM => {
                            self.print_strnumber(match_chr as StrNum);
                            if c <= 9 {
                                self.print_char(c as u8 + b'0');
                            }
                            else {
                                return self.print_char(b'!');
                            }
                        },

                        MATCH => {
                            match_chr = c as u8;
                            self.print_strnumber(c as StrNum);
                            n += 1;
                            self.print_char(n);
                            if n > b'9' {
                                return;
                            }
                        },

                        END_MATCH => self.print("->"),
                        
                        _ => self.print_esc("BAD."),
                    }
                    // End Section 294
                }
            }
            // End section 293
            p = link(p);
        }
        if p != NULL {
            self.print_esc("ETC.");
        }
    }

    // Section 295
    pub(crate) fn token_show(&mut self, p: HalfWord) {
        if p != NULL {
            self.show_token_list(link(p), NULL, 10_000_000);
        }
    }

    // Section 296
    pub(crate) fn print_meaning(&mut self) {
        self.print_cmd_chr(self.cur_cmd, self.cur_chr);
        if self.cur_cmd >= CALL {
            self.print_char(b':');
            self.print_ln();
            self.token_show(self.cur_chr);
        }
        else if self.cur_cmd == TOP_BOT_MARK {
            self.print_char(b':');
            self.print_ln();
            self.token_show(self.cur_mark[self.cur_chr as usize]);
        }
    }

    // Section 298
    pub(crate) fn print_cmd_chr(&mut self, cmd: QuarterWord, chr_code: HalfWord) {
        macro_rules! chr_cmd {
            ($s:expr) => {
                {
                    self.print($s);
                    self.print_strnumber(chr_code as StrNum);
                }
            };
        }
        
        match cmd {
            LEFT_BRACE => chr_cmd!("begin-group character "),
            
            RIGHT_BRACE => chr_cmd!("end-group character "),
            
            MATH_SHIFT => chr_cmd!("math shift character "),
            
            MAC_PARAM => chr_cmd!("macro parameter character "),
            
            SUP_MARK => chr_cmd!("superscript character "),
            
            SUB_MARK => chr_cmd!("subscript character "),
            
            ENDV => chr_cmd!("end of alignment template"),
            
            SPACER => chr_cmd!("blank space "),
            
            LETTER => chr_cmd!("the letter "),
            
            OTHER_CHAR => chr_cmd!("the character "),
            
            // Section 227
            ASSIGN_GLUE
            | ASSIGN_MU_GLUE => {
                if chr_code < SKIP_BASE {
                    self.print_skip_param(chr_code - GLUE_BASE);
                }
                else if chr_code < MU_SKIP_BASE {
                    self.print_esc("skip");
                    self.print_int(chr_code - SKIP_BASE);
                }
                else {
                    self.print_esc("muskip");
                    self.print_int(chr_code - MU_SKIP_BASE);
                }
            },
            // End section 227

            // Section 231
            ASSIGN_TOKS => {
                if chr_code >= TOKS_BASE {
                    self.print_esc("toks");
                    self.print_int(chr_code - TOKS_BASE);
                }
                else {
                    match chr_code {
                        OUTPUT_ROUTINE_LOC => self.print_esc("output"),
                        EVERY_PAR_LOC => self.print_esc("everypar"),
                        EVERY_MATH_LOC => self.print_esc("everymath"),
                        EVERY_DISPLAY_LOC => self.print_esc("everydisplay"),
                        EVERY_HBOX_LOC => self.print_esc("everyhbox"),
                        EVERY_VBOX_LOC => self.print_esc("everyvbox"),
                        EVERY_JOB_LOC => self.print_esc("everyjob"),
                        EVERY_CR_LOC => self.print_esc("everycr"),
                        _ => self.print_esc("errhelp"),
                    };
                }
            },
            // End section 231

            // Section 239
            ASSIGN_INT => {
                if chr_code < COUNT_BASE {
                    self.print_param(chr_code - INT_BASE);
                }
                else {
                    self.print_esc("count");
                    self.print_int(chr_code - COUNT_BASE);
                }
            },
            // End section 239
            
            // Section 249
            ASSIGN_DIMEN => {
                if chr_code < SCALED_BASE {
                    self.print_length_param(chr_code - DIMEN_BASE);
                }
                else {
                    self.print_esc("dimen");
                    self.print_int(chr_code - SCALED_BASE);
                }
            },
            // End section 249
            
            // Section 266
            ACCENT => self.print_esc("accent"),
            
            ADVANCE => self.print_esc("advance"),
            
            AFTER_ASSIGNMENT => self.print_esc("afterassignment"),
            
            AFTER_GROUP => self.print_esc("aftergroup"),

            ASSIGN_FONT_DIMEN => self.print_esc("fontdimen"),
            
            BEGIN_GROUP => self.print_esc("begingroup"),
            
            BREAK_PENALTY => self.print_esc("penalty"),
            
            CHAR_NUM => self.print_esc("char"),
            
            CS_NAME => self.print_esc("csname"),
            
            DEF_FONT => self.print_esc("font"),
            
            DELIM_NUM => self.print_esc("delimiter"),
            
            DIVIDE => self.print_esc("divide"),
            
            END_CS_NAME => self.print_esc("endcsname"),
            
            END_GROUP => self.print_esc("endgroup"),
            
            EX_SPACE => self.print_esc(" "),
            
            EXPAND_AFTER => self.print_esc("expandafter"),
            
            HALIGN => self.print_esc("halign"),
            
            HRULE => self.print_esc("hrule"),
            
            IGNORE_SPACES => self.print_esc("ignorespaces"),
            
            INSERT => self.print_esc("insert"),
            
            ITAL_CORR => self.print_esc("/"),
            
            MARK => self.print_esc("mark"),
            
            MATH_ACCENT => self.print_esc("mathaccent"),
            
            MATH_CHAR_NUM => self.print_esc("mathchar"),
            
            MATH_CHOICE => self.print_esc("mathchoice"),
            
            MULTIPLY => self.print_esc("multiply"),
            
            NO_ALIGN => self.print_esc("noalign"),
            
            NO_BOUNDARY => self.print_esc("noboundary"),
            
            NO_EXPAND => self.print_esc("noexpand"),
            
            NON_SCRIPT => self.print_esc("nonscript"),
            
            OMIT => self.print_esc("omit"),
            
            RADICAL => self.print_esc("radical"),
            
            READ_TO_CS => self.print_esc("read"),
            
            RELAX => self.print_esc("relax"),
            
            SET_BOX => self.print_esc("setbox"),
            
            SET_PREV_GRAF => self.print_esc("prevgraf"),
            
            SET_SHAPE => self.print_esc("parshape"),
            
            THE => self.print_esc("the"),
            
            TOKS_REGISTER => self.print_esc("toks"),
            
            VADJUST => self.print_esc("vadjust"),
            
            VALIGN => self.print_esc("valign"),
            
            VCENTER => self.print_esc("vcenter"),
            
            VRULE => self.print_esc("vrule"),
            // End section 266
            
            // Section 335
            PAR_END => self.print_esc("par"),
            // End section 335
            
            // Section 377
            INPUT => {
                if chr_code == 0 {
                    self.print_esc("input");
                }
                else {
                    self.print_esc("endinput");
                }
            },
            // End section 377

            // Section 385
            TOP_BOT_MARK => {
                match chr_code {
                    FIRST_MARK_CODE => self.print_esc("firstmark"),
                    BOT_MARK_CODE => self.print_esc("botmark"),
                    SPLIT_FIRST_MARK_CODE => self.print_esc("splitfirstmark"),
                    SPLIT_BOT_MARK_CODE => self.print_esc("splitbotmark"),
                    _ => self.print_esc("topmark"),
                }
            },
            // End section 385
            
            // Section 412
            REGISTER => {
                match chr_code {
                    INT_VAL => self.print_esc("count"),
                    DIMEN_VAL => self.print_esc("dimen"),
                    GLUE_VAL => self.print_esc("skip"),
                    _ => self.print_esc("muskip"),
                }
            },
            // End section 412
            
            // Section 417
            SET_AUX => {
                if chr_code == VMODE {
                    self.print_esc("prevdepth");
                }
                else {
                    self.print_esc("spacefactor");
                }
            },
            
            SET_PAGE_INT => {
                if chr_code == 0 {
                    self.print_esc("deadcycles");
                }
                else {
                    self.print_esc("insertpenalties");
                }
            },
            
            SET_BOX_DIMEN => {
                match chr_code {
                    WIDTH_OFFSET => self.print_esc("wd"),
                    HEIGHT_OFFSET => self.print_esc("ht"),
                    _ => self.print_esc("dp"),
                }
            },
            
            LAST_ITEM => {
                match chr_code {
                    INT_VAL => self.print_esc("lastpenalty"),
                    DIMEN_VAL => self.print_esc("lastkern"),
                    GLUE_VAL => self.print_esc("lastskip"),
                    INPUT_LINE_NO_CODE => self.print_esc("inputlineno"),
                    _ => self.print_esc("badness"),
                }
            },
            // End section 417
            
            // Section 469
            CONVERT => {
                match chr_code {
                    NUMBER_CODE => self.print_esc("number"),
                    ROMAN_NUMERAL_CODE => self.print_esc("romannumeral"),
                    STRING_CODE => self.print_esc("string"),
                    MEANING_CODE => self.print_esc("meaning"),
                    FONT_NAME_CODE => self.print_esc("fontname"),
                    _ => self.print_esc("jobname"),
                }
            },
            // End section 469
            
            // Section 488
            IF_TEST => {
                match chr_code {
                    IF_CAT_CODE => self.print_esc("ifcat"),
                    IF_INT_CODE => self.print_esc("ifnum"),
                    IF_DIM_CODE => self.print_esc("ifdim"),
                    IF_ODD_CODE => self.print_esc("ifodd"),
                    IF_VMODE_CODE => self.print_esc("ifvmode"),
                    IF_HMODE_CODE => self.print_esc("ifhmode"),
                    IF_MMODE_CODE => self.print_esc("ifmmode"),
                    IF_INNER_CODE => self.print_esc("ifinner"),
                    IF_VOID_CODE => self.print_esc("ifvoid"),
                    IF_HBOX_CODE => self.print_esc("ifhbox"),
                    IF_VBOX_CODE => self.print_esc("ifvbox"),
                    IFX_CODE => self.print_esc("ifx"),
                    IF_EOF_CODE => self.print_esc("ifeof"),
                    IF_TRUE_CODE => self.print_esc("iftrue"),
                    IF_FALSE_CODE => self.print_esc("iffalse"),
                    IF_CASE_CODE => self.print_esc("ifcase"),
                    _ => self.print_esc("if"),
                }
            },
            // End section 488
            
            // Section 492
            FI_OR_ELSE => {
                match chr_code {
                    FI_CODE => self.print_esc("fi"),
                    OR_CODE => self.print_esc("or"),
                    _ => self.print_esc("else"),
                }
            },
            // End section 492
            
            // Section 781
            TAB_MARK => {
                match chr_code {
                    SPAN_CODE => self.print_esc("span"),
                    _ => chr_cmd!("alignment tab character "),
                }
            },
            
            CAR_RET => {
                match chr_code {
                    CR_CODE => self.print_esc("cr"),
                    _ => self.print_esc("crcr"),
                }
            },
            // End section 781
            
            // Section 984
            SET_PAGE_DIMEN => {
                match chr_code {
                    0 => self.print_esc("pagegoal"),
                    1 => self.print_esc("pagetotal"),
                    2 => self.print_esc("pagestretch"),
                    3 => self.print_esc("pagefilstretch"),
                    4 => self.print_esc("pagefillstretch"),
                    5 => self.print_esc("pagefilllstretch"),
                    6 => self.print_esc("pageshrink"),
                    _ => self.print_esc("pagedepth"),
                }
            },
            // End section 984
            
            // Section 1053
            STOP => {
                match chr_code {
                    1 => self.print_esc("dump"),
                    _ => self.print_esc("end"),
                }
            },
            // End section 1053
            
            // Section 1059
            HSKIP => {
                match chr_code {
                    SKIP_CODE => self.print_esc("hskip"),
                    FIL_CODE => self.print_esc("hfil"),
                    FILL_CODE => self.print_esc("hfill"),
                    SS_CODE => self.print_esc("hss"),
                    _ => self.print_esc("hfilneg"),
                }
            },
            
            VSKIP => {
                match chr_code {
                    SKIP_CODE => self.print_esc("vskip"),
                    FIL_CODE => self.print_esc("vfil"),
                    FILL_CODE => self.print_esc("vfill"),
                    SS_CODE => self.print_esc("vss"),
                    _ => self.print_esc("vfilneg"),
                }
            },
            
            MSKIP => self.print_esc("mskip"),
            
            KERN => self.print_esc("kern"),
            
            MKERN => self.print_esc("mkern"),
            // End section 1059
            
            // Section 1072
            HMOVE => {
                match chr_code {
                    1 => self.print_esc("moveleft"),
                    _ => self.print_esc("moveright"),
                }
            },
            
            VMOVE => {
                match chr_code {
                    1 => self.print_esc("raise"),
                    _ => self.print_esc("lower"),
                }
            },
            
            MAKE_BOX => {
                match chr_code {
                    BOX_CODE => self.print_esc("box"),
                    COPY_CODE => self.print_esc("copy"),
                    LAST_BOX_CODE => self.print_esc("lastbox"),
                    VSPLIT_CODE => self.print_esc("vsplit"),
                    VTOP_CODE => self.print_esc("vtop"),
                    VTOP_CODE_PLUS_VMODE => self.print_esc("vbox"),
                    _ => self.print_esc("hbox"),
                }
            },
            
            LEADER_SHIP => {
                match chr_code as QuarterWord {
                    A_LEADERS => self.print_esc("leaders"),
                    C_LEADERS => self.print_esc("cleaders"),
                    X_LEADERS => self.print_esc("xleaders"),
                    _ => self.print_esc("shipout"),
                }
            },
            // End section 1072
            
            // Section 1089
            START_PAR => {
                if chr_code == 0 {
                    self.print_esc("noindent");
                }
                else {
                    self.print_esc("indent");
                }
            }
            // End section 1089
            
            // Section 1108
            REMOVE_ITEM => {
                match chr_code as QuarterWord {
                    GLUE_NODE => self.print_esc("unskip"),
                    KERN_NODE => self.print_esc("unkern"),
                    _ => self.print_esc("unpenalty"),
                }
            },
            
            UN_HBOX => {
                if chr_code == COPY_CODE {
                    self.print_esc("unhcopy");
                }
                else {
                    self.print_esc("unhbox");
                }
            },
            
            UN_VBOX => {
                if chr_code == COPY_CODE {
                    self.print_esc("unvcopy");
                }
                else {
                    self.print_esc("unvbox");
                }
            },
            // End section 1108
            
            // Section 1115
            DISCRETIONARY => {
                if chr_code == 1 {
                    self.print_esc("-");
                }
                else {
                    self.print_esc("discretionary");
                }
            },
            // End section 1115

            // Section 1143
            EQ_NO => {
                if chr_code == 1 {
                    self.print_esc("leqno");
                }
                else {
                    self.print_esc("eqno");
                }
            },
            // End section 1143

            // Section 1157
            MATH_COMP => {
                match chr_code as QuarterWord {
                    ORD_NOAD => self.print_esc("mathord"),
                    OP_NOAD => self.print_esc("mathop"),
                    BIN_NOAD => self.print_esc("mathbin"),
                    REL_NOAD => self.print_esc("mathrel"),
                    OPEN_NOAD => self.print_esc("mathopen"),
                    CLOSE_NOAD => self.print_esc("mathclose"),
                    PUNCT_NOAD => self.print_esc("mathpunct"),
                    INNER_NOAD => self.print_esc("mathinner"),
                    UNDER_NOAD => self.print_esc("underline"),
                    _ => self.print_esc("overline"),
                }
            },

            LIMIT_SWITCH => {
                match chr_code as QuarterWord {
                    LIMITS => self.print_esc("limits"),
                    NO_LIMITS => self.print_esc("nolimits"),
                    _ => self.print_esc("displaylimits"),
                }
            }
            // End section 1157

            // Section 1170
            MATH_STYLE => self.print_style(chr_code),
            // End section 1170

            // Section 1179
            ABOVE => {
                match chr_code {
                    OVER_CODE => self.print_esc("over"),
                    ATOP_CODE => self.print_esc("atop"),
                    DELIMITED_ABOVE_CODE => self.print_esc("abovewithdelims"),
                    DELIMITED_OVER_CODE => self.print_esc("overwithdelims"),
                    DELIMITED_ATOP_CODE => self.print_esc("atopwithdelims"),
                    _ => self.print_esc("above"),
                }
            },
            // End section 1179

            // Section 1189
            LEFT_RIGHT => {
                if chr_code == LEFT_NOAD as HalfWord {
                    self.print_esc("left");
                }
                else {
                    self.print_esc("right");
                }
            },
            // End section 1189

            // Section 1209
            PREFIX => {
                match chr_code {
                    1 => self.print_esc("long"),
                    2 => self.print_esc("outer"),
                    _ => self.print_esc("global"),
                }
            },

            DEF => {
                match chr_code {
                    0 => self.print_esc("def"),
                    1 => self.print_esc("gdef"),
                    2 => self.print_esc("edef"),
                    _ => self.print_esc("xdef"),
                }
            },
            // End section 1209

            // Section 1220
            LET => {
                match chr_code as QuarterWord {
                    NORMAL => self.print_esc("let"),
                    _ => self.print_esc("futurelet"),
                }
            },
            // End section 1220

            // Section 1223
            SHORTHAND_DEF => {
                match chr_code {
                    CHAR_DEF_CODE => self.print_esc("chardef"),
                    MATH_CHAR_DEF_CODE => self.print_esc("mathchardef"),
                    COUNT_DEF_CODE => self.print_esc("countdef"),
                    DIMEN_DEF_CODE => self.print_esc("dimendef"),
                    SKIP_DEF_CODE => self.print_esc("skipdef"),
                    MU_SKIP_DEF_CODE => self.print_esc("muskipdef"),
                    _ => self.print_esc("toksdef"),
                }
            },

            CHAR_GIVEN => {
                self.print_esc("char");
                self.print_hex(chr_code);
            },

            MATH_GIVEN => {
                self.print_esc("mathchar");
                self.print_hex(chr_code);
            },
            // End section 1223

            // Section 1231
            DEF_CODE => {
                match chr_code {
                    CAT_CODE_BASE => self.print_esc("catcode"),
                    MATH_CODE_BASE => self.print_esc("mathcode"),
                    LC_CODE_BASE => self.print_esc("lccode"),
                    UC_CODE_BASE => self.print_esc("uccode"),
                    SF_CODE_BASE => self.print_esc("sfcode"),
                    _ => self.print_esc("delcode"),
                }
            },

            DEF_FAMILY => self.print_size(chr_code - MATH_FONT_BASE),
            // End section 1231

            // Section 1251
            HYPH_DATA => {
                if chr_code == 1 {
                    self.print_esc("patterns");
                }
                else {
                    self.print_esc("hyphenation");
                }
            },
            // End section 1251

            // Section 1255
            ASSIGN_FONT_INT => {
                if chr_code == 0 {
                    self.print_esc("hyphenchar");
                }
                else {
                    self.print_esc("skewchar");
                }
            },
            // End section 1255

            // Section 1261
            SET_FONT => {
                self.print("select font ");
                self.slow_print(self.font_name[chr_code as usize]);
                if self.font_size[chr_code as usize] != self.font_dsize[chr_code as usize] {
                    self.print(" at ");
                    self.print_scaled(self.font_size[chr_code as usize]);
                    self.print("pt");
                }
            },
            // End section 1261

            // Section 1263
            SET_INTERACTION => {
                match chr_code {
                    ERROR_STOP_MODE => self.print_esc("errorstopmode"),
                    _ => self.print_esc("batchmode"),
                }
            },
            // End section 1263

            // Section 1273
            IN_STREAM => {
                if chr_code == 0 {
                    self.print_esc("closein");
                }
                else {
                    self.print_esc("openin");
                }
            },
            // End section 1273

            // Section 1278
            MESSAGE => {
                if chr_code == 0 {
                    self.print_esc("message");
                }
                else {
                    self.print_esc("errmessage");
                }
            },
            // End section 1278

            // Section 1287
            CASE_SHIFT => {
                if chr_code == LC_CODE_BASE {
                    self.print_esc("lowercase");
                }
                else {
                    self.print_esc("uppercase");
                }
            },
            // End section 1287

            // Section 1292
            XRAY => {
                match chr_code {
                    SHOW_BOX_CODE => self.print_esc("showbox"),
                    SHOW_THE_CODE => self.print_esc("showthe"),
                    SHOW_LISTS => self.print_esc("showlists"),
                    _ => self.print_esc("show"),
                }
            },
            // End section 1292

            // Section 1295
            UNDEFINED_CS => self.print("undefined"),
            
            CALL => self.print("macro"),
            
            LONG_CALL => self.print_esc("long macro"),
            
            OUTER_CALL => self.print_esc("outer macro"),
            
            LONG_OUTER_CALL => {
                self.print_esc("long");
                self.print_esc("outer macro");
            },

            END_TEMPLATE => self.print_esc("outer endtemplate"),
            // End section 1295
            
            // Section 1346
            EXTENSION => {
                match chr_code {
                    OPEN_NODE => self.print_esc("openout"),
                    WRITE_NODE => self.print_esc("write"),
                    CLOSE_NODE => self.print_esc("closeout"),
                    SPECIAL_NODE => self.print_esc("special"),
                    IMMEDIATE_CODE => self.print_esc("immediate"),
                    SET_LANGUAGE_CODE    => self.print_esc("setlanguage"),
                    _ => self.print("[unknown extension!]"),
                }
            }
            // End section 1346

            _ => self.print("[unknown command code]"),
        }
    }

    // Section 299
    pub(crate) fn show_cur_cmd_chr(&mut self) {
        self.begin_diagnostic();
        self.print_nl("{");
        if self.mode() != self.shown_mode {
            self.print_mode(self.mode());
            self.print(": ");
            self.shown_mode = self.mode();
        }
        self.print_cmd_chr(self.cur_cmd, self.cur_chr);
        self.print("}");
        self.end_diagnostic(false);
    }
}
