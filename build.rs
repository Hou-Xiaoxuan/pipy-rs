use std::env;

use cmake::Config;

fn main() {
    let profile = env::var("PROFILE").unwrap();

    let mut config = Config::new("libs/pipy");

    // TODO: change to static library
    config.define("PIPY_SHARED", "ON");
    // set to use clang/clang++ to compile
    config.define("CMAKE_C_COMPILER", "clang");
    config.define("CMAKE_CXX_COMPILER", "clang++");

    std::env::set_var("CMAKE_BUILD_PARALLEL_LEVEL", "4");

    // use `tcmalloc` if it exists
    // because pipy will use `tcmalloc` if it exists, and rust must unify the memory allocator
    if exist_tcmalloc() {
        println!("cargo:rustc-cfg=feature=\"use_tcmalloc\"");
    }

    if profile == "release" {
        config.define("CMAKE_BUILD_TYPE", "Release");
    } else {
        config.define("CMAKE_BUILD_TYPE", "Debug");
    }
    // add target to build
    config.no_build_target(true);

    // build
    let dst = config.build();

    // ** `cargo:rustc-*` format is used to pass information to the cargo build system
    // parse to `rustc` to look for dynamic library, used in running
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}/build,-rpath,$ORIGIN",
        dst.display()
    );
    // add the path to the library to the linker search path, used in build
    println!("cargo:rustc-link-search={}/build", dst.display());

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

// check if tcmalloc library exists in the system
fn exist_tcmalloc() -> bool {
    // TODO: look for a way to check if tcmalloc exists or libpipy use tcmalloc
    false
}
