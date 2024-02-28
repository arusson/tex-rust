use std::env;
use std::fs::{File, metadata};
use std::process::Command;

// Change the filename below if you use a different format
const PRELOADED_FORMAT: &str = "plain.fmt";

fn main() {
    let path = env::var("OUT_DIR").unwrap() + "/preloaded_format.fmt";

    println!("cargo:rustc-env=PRELOADED_FORMAT={PRELOADED_FORMAT}");
    match metadata(PRELOADED_FORMAT) {
        Ok(_) => {
            _ = Command::new("cp")
                .args([PRELOADED_FORMAT, &path])
                .status()
                .unwrap();
        },
        Err(_) => {
            _ = File::create(path)
                .expect("cannot create empty file")
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=plain.fmt");
}
