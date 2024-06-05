/// a test demo for pipy
use libc::{c_char, c_int};

pub mod api;
mod util;
#[link(name = "pipy", kind = "dylib")]
extern "C" {
    pub fn pipy_main(argc: c_int, argv: *const *const c_char) -> c_int;
}
