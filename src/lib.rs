/// a test demo for pipy
use libc::{c_char, c_int};
use std::{ffi::CString, thread};

pub mod api;
mod util;
#[link(name = "pipy", kind = "dylib")]
extern "C" {
    pub fn pipy_main(argc: c_int, argv: *const *const c_char) -> c_int;
}
/// start pipy in repo mod with port, default port is 6060
pub fn start_pipy_repo(port: Option<usize>) {
    thread::spawn(move || {
        let mut args: Vec<CString> = vec![];
        args.push(CString::new("pipy-rs").unwrap());
        args.push(CString::new(format!("--admin-port={}", port.unwrap_or(6060))).unwrap());
        let c_args: Vec<*const c_char> = args
            .iter()
            .map(|arg| <CString as Clone>::clone(&arg).into_raw() as *const c_char)
            .collect();

        unsafe {
            pipy_main(c_args.len() as c_int, c_args.as_ptr());
        }
    });
}

#[cfg(test)]
mod tests {
    use util::{init_debug_logger, init_logger};

    use libc::exit;

    use super::*;
    #[tokio::test]
    async fn test_start_pipy_repo() {
        init_logger();

        start_pipy_repo(Some(6060));
        thread::sleep(std::time::Duration::from_secs(1)); // wait for pipy to start
        api::create_codebase("127.0.0.1", 6060, "test1")
            .await
            .unwrap();
        api::create_codebase("127.0.0.1", 6060, "test2")
            .await
            .unwrap();

        let codebase_list = api::get_codebase_list("127.0.0.1", 6060).await;
        assert!(codebase_list.is_ok());
        let codebase_list = codebase_list.unwrap();
        tracing::info!("codebase_list: {:?}", codebase_list);
        assert!(codebase_list.contains(&"/test1".to_string()));
        assert!(codebase_list.contains(&"/test2".to_string()));
        unsafe {
            exit(0); // exit the test. Otherwise, the `pipy-main` thread will report `panic!`, wait for a better solution
        }
    }
}
