use std::process::Command;
use std::path::Path;
use std::env;

fn main() {

    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let clips_dir = format!("{}/clips_core_source_640", dir);
    let clips_core_dir = format!("{}/core", clips_dir);

    Command::new("make")
        .current_dir(&Path::new(&clips_core_dir))
        .status().unwrap();

    println!("cargo:rustc-link-search={}", clips_core_dir);
    println!("cargo:rustc-link-lib=static=clips");
}
