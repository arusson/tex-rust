use tex_rust::{
    constants::SPOTLESS,
    datastructures::end_line_char,
    initialization::fix_date_and_time,
    strings::{
        get_strings_started, init_pool_ptr_set, init_str_ptr_set,
        pool_ptr, str_ptr
    },
    Global, Integer, end_line_char_inactive
};

// Part 51: The main program

const PRELOADED_FORMAT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/preloaded_format.fmt"));

fn help() {
    println!("Usage: tex-rust [-ini] TEXNAME[.tex] [-fmt=FMTNAME[.fmt]]");
    println!();
    println!("  Run TeX on TEXNAME, usually to create TEXNAME.dvi.");
    println!("  Contrary to original TeX, there is no prompt input if no file is provided.");
    println!("  A format file can be given as input (plain.fmt will be assumed by default).");
    println!();
    println!("  For dumping format, INITEX is available with `-ini` option.");
    println!("  Note that a format could still be supplied to construct a new format");
    println!("  on top of another.");
    println!();
    println!("-fmt=FMTNAME  use FMTNAME as format file instead of plain");
    println!("                (equal sign and file extension are optional)");
    println!("-ini          INITEX mode for dumping formats")
}

fn main() {
    // Parsing arguments from command line
    let args: Vec<String> = std::env::args().collect();
    let mut input_fname = "";
    let mut format_fname = "";
    let mut ini = false;
    let mut n = 1;
    while n < args.len() {
        if args[n] == "-h"
            || args[n] == "--help"
            || args[n] == "-help"
        {
            help();
            return;
        }
        if args[n] == "-ini" {
            ini = true;
        }
        else if args[n].starts_with("-fmt=") {
            if format_fname.is_empty() {
                format_fname = args[n].split_at(5).1;
            }
        }
        else if args[n] == "-fmt" {
            if n + 1 < args.len() {
                if format_fname.is_empty() {
                    format_fname = &args[n + 1];
                }
                n += 1;
            }
        }
        else if input_fname.is_empty() {
            input_fname = &args[n];
        }
        n += 1;
    }
    if input_fname.is_empty() {
        help();
        return;
    }

    // Section 1332
    let mut global = Global::default();
    global.initex_mode = ini;

    macro_rules! manage_error {
        (global.$f:ident($($args:expr),*)) => {
            if let Err(texerror) = global.$f($($args),*) {
                if global.error(texerror).is_err() {
                    println!("Ouch---the error procedure produced an error!");
                }
                return;
            }
        };

        ($f:ident($($args:expr),*)) => {
            if let Err(texerror) = $f($($args),*) {
                if global.error(texerror).is_err() {
                    println!("Ouch---the error procedure produced an error!");
                }
                return;
            }
        };
    }

    // Section 14
    let bad = global.check_constant_values_for_consistency();
    if bad > 0 {
        println!("Ouch---my internal constants have been clobbered!");
        println!("---case {bad}");
        return;
    }
    // End section 14

    global.initialize();

    // Load format file from preloaded format in the binary
    if !global.initex_mode && !PRELOADED_FORMAT.is_empty() {
        global.fmt_file.set_preloaded(PRELOADED_FORMAT);
        if !global.load_fmt_file() {
            return; // Final end
        }
    }

    if global.initex_mode {
        manage_error!(get_strings_started());    
        manage_error!(global.init_prim());
        init_str_ptr_set(str_ptr());
        init_pool_ptr_set(pool_ptr());
        fix_date_and_time();
    }

    // Start of TeX
    global.initialize_output_routines();
    
    // Section 1337
    global.initialize_input_routines();
    if global.format_ident == 0 || !format_fname.is_empty() {
        if global.format_ident != 0 {
            global.initialize(); // erase preloaded format
        }
        manage_error!(global.open_fmt_file(format_fname));
        if !global.load_fmt_file() {
            return; // Final end
        }
        global.fmt_file.close();
    }

    // We copy input filename at the beginning of `buffer`.
    // loc already points to the start of the buffer, but we must
    // set last and limit accordingly to the length of the name.
    // An end of line is added at the end of the name,
    // except if end_line_char_inactive is true which might
    // happen if the format modified the end of line character.
    let len = input_fname.len();
    global.buffer[..len].copy_from_slice(input_fname.as_bytes());
    // We expect filenames are only ASCII.
    *global.limit_mut() = len as Integer;
    *global.limit_mut() = if end_line_char_inactive!() {
         (len - 1) as Integer
    }
    else {
        global.buffer[len] = end_line_char() as u8;
        len as Integer
    };
    global.last = len as Integer;
    global.first = global.last + 1;

    fix_date_and_time();
    global.sec75_initialize_print_selector();

    manage_error!(global.start_input()); // \input assumed
    // End section 1337

    global.history = SPOTLESS; // ready to go!

    manage_error!(global.main_control());  // come to life
    manage_error!(global.final_cleanup()); // prepare for death

    // End of TeX
    manage_error!(global.close_files_and_terminate());
}
