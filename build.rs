use cmake::Config;

fn main() {
    // TODO Randomly compilable now.
    let mut config = Config::new("libs/pipy");

    // Build `pipy` as a shared library **should edit the CMakeLists.txt to support shared library**
    config.define("PIPY_SHARED", "On");
    config.define("CMAKE_BUILD_PARALLEL_LEVEL", "4");

    // build
    let dst = config.build();
    // ** `cargo:rustc-*` format is used to pass information to the cargo build system
    // add the path to the library to the linker search path
    println!("cargo:rustc-link-search={}/build/", dst.display());

    // according to `pipy bind_.pdf`, didn't know reason temporarily
    println!("cargo:rustc-link-lib=pipy");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=yajl_s");
    println!("cargo:rustc-link-lib=brotlienc");
    println!("cargo:rustc-link-lib=brotlidec");
    println!("cargo:rustc-link-lib=expat");
    println!("cargo:rustc-link-lib=yaml");
    println!("cargo:rustc-link-lib=leveldb");
    println!("cargo:rustc-link-lib=z");
}
