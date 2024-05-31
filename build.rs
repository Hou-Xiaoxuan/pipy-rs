use std::env;

use cmake::Config;

fn main() {
    let profile = env::var("PROFILE").unwrap();

    let mut config = Config::new("libs/pipy");

    // TODO: change to static library
    config.define("PIPY_SHARED", "ON");
    config.define("CMAKE_BUILD_PARALLEL_LEVEL", "4"); // didn't work, compile too slow

    if profile == "release" {
        config.define("CMAKE_BUILD_TYPE", "Release");
    } else {
        config.define("CMAKE_BUILD_TYPE", "Debug");
    }

    // build
    let dst = config.build();

    // ** `cargo:rustc-*` format is used to pass information to the cargo build system
    // add the path to the library to the linker search path
    println!("cargo:rustc-link-search={}/build/", dst.display());

    // according to `pipy bind_.pdf`, didn't know reason temporarily
    // but in macos, if i use pipy in `lib.rs` but not in `main.rs`, below code cause error, didn't know reason, so comment it
    println!("cargo:rustc-link-lib=pipy");
    // println!("cargo:rustc-link-lib=stdc++");
    // println!("cargo:rustc-link-lib=ssl");
    // println!("cargo:rustc-link-lib=crypto");
    // println!("cargo:rustc-link-lib=yajl_s");
    // println!("cargo:rustc-link-lib=brotlienc");
    // println!("cargo:rustc-link-lib=brotlidec");
    // println!("cargo:rustc-link-lib=expat");
    // println!("cargo:rustc-link-lib=yaml");
    // println!("cargo:rustc-link-lib=leveldb");
    // println!("cargo:rustc-link-lib=z");
}
