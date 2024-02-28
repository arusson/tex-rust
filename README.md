# $\bf \TeX$ in Rust

$\rm\TeX$ is a program written by Donald E. Knuth.

This project is a manual conversion of $\rm\TeX$ into the Rust language for pedagogic purpose.
The goal was to understand how the program works by rewriting it.
At the same time, it was useful to continue learning Rust.

## Conversion to Rust

The original code was written in [`WEB`](https://ctan.gutenberg-asso.fr/info/knuth/webman.pdf) where Pascal is the base language.
However, features very specific to Pascal were deliberately not much used, so adaptation to other languages is feasible without changing too much of the base code.

Here is listed how a few things were adapted.

### Sections numbering

The section numbers from [*TeX: The Program*](https://mirrors.ctan.org/info/knuth-pdf/tex/tex.pdf) are kept in the source code, so the original code can still be referred to for comparison.

For example, the `round_decimals` function is define in section 102:
```rust
impl Global {
    // Section 102
    pub(crate) fn round_decimals(&self, mut k: usize) -> Scaled {
        let mut a = 0;
        while k > 0 {
            k -= 1;
            a = (a + (self.dig[k] as Integer)*TWO) / 10;
        }
        (a + 1) / 2
    }
}
```

When a section is part of another, it is enclosed with comments.
For example, section 854 is part of section 851 (itself part of section 829 that defines the `try_break` procedure):

```rust
let node_r_stays_active = if b > INF_BAD || pi == EJECT_PENALTY {
    // Section 854
    if self.final_pass
        && self.minimum_demerits == AWFUL_BAD
        && link(r) == LAST_ACTIVE
        && prev_r == ACTIVE
    {
        artificial_demerits = true;
    }
    else if b > self.threshold {
        break 'block; // Goto deactivate
    }
    false
    // End section 854
}
else {
    prev_r = r;
    if b > self.threshold {
        continue 'sec829; // Goto continue
    }
    true
};
```

### Global variables

$\rm\TeX$ uses a lot of global variables.
In Rust, mutable global variables needs to be used with the `unsafe` keyword, and alternatives would require too much verbosity, so global variables are (almost) all defined in a `struct` named `Global` (declared in [global.rs](./src/global.rs)).

As a consquence, most of the functions are defined in `impl Global`, and the keyword `self` appears a lot.

There are a few exceptions such as the memory array `MEM` declared as `static mut` in [memory.rs](./src/datastructures/memory.rs).
Two macros `mem!` and `mem_mut!` are used to access members with `unsafe`.

Other tables are declared as `static mut`:
`EQTB` and `XEQ_LEVEL` (the equivalent tables), `HASH` (the hash table), and `POOL` (the string pool).

### Goto statements

Backwards goto can be handled using `loop` and `continue`, which is the case most of the time.

Forwards goto are sometimes handled with [`break` from labeled blocks](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html#break-from-labeled-blocks).
It works as a `break` from a loop, but with a block of code delimited by braces.

Another form of goto is manged with an `enum`, in particular for some parts that have many goto (see the [`main_control`](./src/builder/chief.rs) procedure).

### Errors

$\rm\TeX$ allows errors to be accumulated, proposing a fix such as `Missing $ inserted` when something that should be in math mode has been read outside of math mode (or vice-versa):
For this specific example, it means a `$` token has been added, and the user can decide to keep it, insert its own choice of tokens, delete tokens, or ask for help (which prints more details about the error).

For this implementation, when there is en error, the program prints the error and the help message, then it stops.
So the error messages were rewritten.
For example, the original `Missing $ inserted` message is:

```
! Missing $ inserted.
<inserted text> 
                $
<to be read again> 
                   ^
l.1 Hello x^
            2$.
? h
I've inserted a begin-math/end-math symbol since I think
you left one out. Proceed, with fingers crossed.

? 
```
The inserted text is presented, the next token to be read again (after the insertion) is presented, then the context line where we can see precisely where the problem was detected.
The help message is given only if the user types `h`.

In this Rust implementation, it becomes:
```
! Missing $.
l.1 Hello x^
            2$.
Either you forgot opening or closing math mode with $,
or a math character/control sequence is used outside
of math mode (or vice versa).
```
The user is invited to fix it, and run the program again.

Al the errors are listed as en `enum` named `TeXError` in [error.rs](./src/error.rs).
They are treated in the `error` procedure where all the help messages are written.
Any function where an error can occur returns `TeXResult<T>` (which is defined as `Result<T, TeXError>`) that returns `Ok` (with the return value if there is one), or `Err` with a `TeXError` (the error goes up to the `main` function where `error` is called).

### Command line

This implementation does not treat the command line arguments as the input buffer.

The user must type at least the input filename (with or without extension), and two arguments are optional:
- `-ini`: the INITEX mode, to dump a format;
- `-fmt`: followed by the filename of the input format (such as `plain.fmt`, again the extension is optional).

So there is no prompt `**`, but there is still the prompt `*` available.
For example, `plain.tex` does not have the `\dump` command at the end, so it has to be written when the prompt appears when running `tex-rust -ini plain`.

Except for this, the usage stays the same.

### Strings

An external pool file is not used to store the strings of the source code.
Instead, almost all of them are static strings except a few that are added in the string pool with `put_string`:

```rust
// Add a string in the pool
pub(crate) fn put_string(s: &[u8]) -> TeXResult<StrNum> {
    unsafe {
        str_room(s.len())?;
        POOL.pool[POOL.pool_ptr..(POOL.pool_ptr + s.len())].copy_from_slice(s);
        POOL.pool_ptr += s.len();
        make_string()
    }
}
```

The behavior of the string pool has not been changed.

### Memory words

A memory word is expanded to 64 bits defined as a `union` in [memory.rs](./src/datastructures/memory.rs):
```rust
#[derive(Clone, Copy)]
pub(crate) union MemoryWord {
    pub(crate) int: Integer,
    pub(crate) sc: Scaled,
    pub(crate) gr: GlueRatio,
    pub(crate) hh: [HalfWord; 2],
    pub(crate) qqqq: [QuarterWord; 4],
    pub(crate) word: u64
}
```

- `Integer`, `Scaled` and `HalfWord` are all `i32`, so they fit nicely into each other (it avoids many casts);
- `GlueRatio` is a `f64`;
- `QuarterWord` is a `u16` with the full range;
- `HalfWord` is an integer between `-0x3fff_ffff` and `0x3fff_ffff` (same as LuaTeX).

Since `MemoryWord` is defined as a `union`, methods to access the value depending of the type it represents have been defined (a direct access needs `unsafe`):
`.int()`, `.sc()`, `.hh_b0()`, `.hh_b1()`, etc., and their mutable versions `.int_mut()`, `.sc_mut()`, `.hh_b0()`, `.hh_b1()`, etc.

### Features

Some parts of the original $\rm\TeX$ can be activated or disabled at compilation time:
- Code between **init** and **tini**: for INITEX;
- Code between **stat** and **tats**: for statistics;
- Code between **debug** and **gubed**: for debugging.

The first one has been integrated as an argument to the command line.
The other two are features, both disabled by default.
Either you provide them manually with `cargo build --features debug,stat` or by editing the [Cargo.toml](Cargo.toml) file.

### Preloaded format

Section 1331 of $\TeX$*: The Program* explains a trick to get a production version of $\rm\TeX$ with a format already loaded.

To my understanding, such a trick is not used anymore for TeXlive binaries.
Instead, the name of the program you run is used to determine the format, then the file is found and loaded.
For example, `optex` is a symlink to the `luatex` binary, and the file `optex.fmt` (which is somewhere on the installation directory) is loaded.

Instead of trying to reproduce this, this Rust implementation allows the user to **embed** directly the format file in the binary.

More details are given [below](#embedding-a-format-file).

## Compilation

As any Rust project, the compilation is easy (with or without `--release`)
```
cargo build --release
```

The binary in `target/release/` (or `target/debug/`) is named `tex-rust`.

Two features are available and can be added with `-F` or `--features`: `debug` and `stat`.

A `Makefile` is provided to compile a version of the program with the `plain` format preloaded.
See below for an explanation.

### Embedding a format file

The file `build.rs` is used to customize the compilation.
By default, it will look for a file `plain.fmt` in the main directory.
If found, it will be embedded in the binary, and the program can be used without having to give a format as input.

However, to create a format  you need the compiled program, so a virgin version must be produced first.

The principle is as follows:
- Compile the program a first time (in debug to be faster);
- Run the program on the source file to dump the format;
- Compile the program again (make sure that the format file in `build.rs` matches with yours).

For example, to produce a $\rm\TeX$ version with the `plain` format (if the file `plain.tex` is in the current folder or in `TeXinputs/`):
```
cargo run -- -ini plain
```
The file `plain.fmt` has been generated.
Compile again in release:
```
cargo build --release
```

The binary in the folder `target/release` is ready to be used with the `plain` format embedded.

## Usage

There are two main usages:
- Dumping a format with the `-ini` option;
- Generating a DVI file.

The command line is:
```
tex-rust [-ini] TEXNAME[.tex] [-fmt=FMTNAME[.fmt]]
```

### Dumping a format file

To dump a format, use the option `-ini` and specify your input file (extension `.tex` is optional):
```
./tex-rust -ini TEXNAME
```
You input file must include the command `\dump`, otherwise a prompt will appear where you can add lines, until you type `\dump`.

Example for `plain` format:
```
./tex-rust -ini plain
```

> The font metric files used by the format must be available while dumping.
> Those must be in the folder `TeXfonts/`.
> The fonts needed for the `plain` format (mainly Computer Modern in several sizes and styles) are already present.
> Any missing font will produce an error.
> Once a format is produced, all necessary data for $\rm\TeX$ are included, so the font metric files won't be necessary.
> 
> If the format needs auxiliary input files (for instance, the `plain` format needs the file `hyphen.tex`), make sure those are available: put them in the folder `TeXinputs/` and the program will know to look there.

### Generating a DVI file

To create a DVI document, you must specify your input file (extension `.tex` optional).
The format file can be submitted too, but it is not mandatory.
If no format is supplied, there are a few possibilities:
- a format is preloaded in the binary and will be used (see [Embedding a format file](#embedding-a-format-file));
- no format is preloaded, then the `plain.fmt` file will be searched in the current folder or in `TeXformats/`: an error will be returned if not found.

If a format is supplied at the command line, then the preloaded format (if there is one) will be erased with the new one.

The command line is:
```
./tex-rust TEXNAME [-fmt=FMTNAME]
```

For example, suppose you input is `paper.tex` and the binary has `plain` preloaded (or `plain.fmt` is available):
```
./tex-rust paper
```

> If your input uses a font that is not included in the format (for example with `\font\libertine=LinLibertineT-tosf-t1`), make sure that it is present in `TeXfonts/`.
> 
> A DVI file can be converted in PDF with the program `dvipdf`, available with TeXlive.

## TRIP test

Since the first error stops the program, then the TRIP test cannot be applied.

However, the resulting DVI files from the [examples](examples/) have been compared to the ones obtained with the [C version](https://github.com/arusson/tex-c) that passes the TRIP test, with the same results (using `dvitype` for comparison), except for the date which might differ depending of the time you run the tests.

> It has also been tested on `tex.tex` (i.e., $\TeX$*: The Program*) for identical results too.
> To test it yourself:
> - Get [`tex.web`](https://ctan.org/tex-archive/systems/knuth/dist/tex), [`webmac.tex`](https://ctan.org/tex-archive/systems/knuth/dist/lib) and [`logo10.tfm`](https://ctan.org/tex-archive/fonts/mflogo/tfm) from CTAN;
> - Use `weave` to get `tex.tex` from `tex.web`.
> - Put `webmac.tex` in `TeXinputs/` and `logo10.tfm` in `TeXfonts/`;
> - Run `./tex-rust tex`.

## License

This work is released under the [MIT license](LICENSE).

The original $\rm\TeX$ program was created by Donald E. Knuth and released under his usual license: https://ctan.org/license/knuth.

Font metric files, `plain.tex` and `hyphen.tex` were copied from CTAN and have not been modified:
[Computer Modern](https://ctan.org/tex-archive/fonts/cm/tfm), [manfnt](https://ctan.org/tex-archive/fonts/manual/tfm), [knuth-lib](https://ctan.org/tex-archive/systems/knuth/dist/lib).
These files are also under the Knuth license.

$\rm\TeX$ is a trademark of the American Mathematical Society.
