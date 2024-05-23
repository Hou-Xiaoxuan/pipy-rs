/// a test demo for pipy
use libc::{c_char, c_int};
use std::ffi::CString;

#[link(name = "pipy", kind = "dylib")]
extern "C" {
    fn pipy_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() {
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect();

    let c_args: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();

    unsafe {
        pipy_main(c_args.len() as c_int, c_args.as_ptr());
    }
}
