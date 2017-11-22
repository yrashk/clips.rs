use std::process::Command;
use std::path::Path;
use std::env;

fn main() {

    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let clips_dir = format!("{}/clips_core_source_630", dir);
    let clips_core_dir = format!("{}/core", clips_dir);

    Command::new("make").args(&["-f", "../makefiles/makefile.lib"])
        .current_dir(&Path::new(&clips_core_dir))
        .status().unwrap();

    println!("cargo:rustc-link-search={}", clips_core_dir);
    println!("cargo:rustc-link-lib=static=clips");
}