/// a test demo for pipy
use libc::{c_char, c_int};
use std::{
    ffi::CString,
    sync::{atomic, Arc},
    thread,
};

pub mod api_client;
mod util;
#[link(name = "pipy", kind = "dylib")]
extern "C" {
    pub fn pipy_main(argc: c_int, argv: *const *const c_char) -> c_int;

    pub fn pipy_exit(force: c_int);
}
/// start pipy in repo mod with given port, default port is 6060
pub fn start_pipy_repo(port: Option<u16>) -> Pipy {
    let port = port.unwrap_or(6060);
    let pipy = Pipy::new(port);
    pipy.start();
    pipy
}

pub struct Pipy {
    port: u16,
    is_started: Arc<atomic::AtomicBool>,
}
impl Pipy {
    pub fn new(port: u16) -> Self {
        Pipy {
            port,
            is_started: Arc::new(atomic::AtomicBool::new(false)),
        }
    }
    pub fn start(&self) {
        let port = self.port;
        let is_started = self.is_started.clone();
        tracing::info!("start pipy with port: {}", port);
        thread::spawn(move || {
            let mut args: Vec<CString> = vec![];
            args.push(CString::new("pipy-rs").unwrap());
            args.push(CString::new(format!("--admin-port={}", port)).unwrap());
            let c_args: Vec<*const c_char> = args
                .iter()
                .map(|arg| <CString as Clone>::clone(&arg).into_raw() as *const c_char)
                .collect();
            is_started.store(true, atomic::Ordering::SeqCst);
            unsafe {
                pipy_main(c_args.len() as c_int, c_args.as_ptr());
            }
        });
        thread::sleep(std::time::Duration::from_secs(1)); // wait for pipy to start
    }
    pub fn exit(&self) {
        if self.is_started.load(atomic::Ordering::SeqCst) {
            unsafe {
                pipy_exit(1);
            }
            self.is_started.store(false, atomic::Ordering::SeqCst);
            thread::sleep(std::time::Duration::from_secs(1)); // wait for pipy to exit
            tracing::info!("exit pipy");
        }
    }
}
impl Drop for Pipy {
    fn drop(&mut self) {
        self.exit();
    }
}

#[cfg(test)]
mod tests {
    use util::init_logger;

    use super::*;
    #[tokio::test]
    async fn test_start_pipy_repo() {
        init_logger("info");
        let port = 6060;
        let client = api_client::ApiClient::new("127.0.0.1", port);
        let _ = start_pipy_repo(Some(port));

        client.create_codebase("test1").await.unwrap();
        client.create_codebase("test2").await.unwrap();

        let codebase_list = client.get_codebase_list().await;
        assert!(codebase_list.is_ok());
        let codebase_list = codebase_list.unwrap();
        tracing::info!("codebase_list: {:?}", codebase_list);
        assert!(codebase_list.contains(&"test1".to_string()));
        assert!(codebase_list.contains(&"test2".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_start_pipy_repo() {
        init_logger("info");
        let port_1 = 6001;
        let port_2 = 6002;
        let client_1 = api_client::ApiClient::new("127.0.0.1", port_1);
        let client_2 = api_client::ApiClient::new("127.0.0.1", port_2);
        let _ = start_pipy_repo(Some(port_1));
        let _ = start_pipy_repo(Some(port_2));

        client_1.create_codebase("test1").await.unwrap();
        client_2.create_codebase("test2").await.unwrap();

        let codebase_list_1 = client_1.get_codebase_list().await;
        assert!(codebase_list_1.is_ok());
        assert!(codebase_list_1.unwrap().contains(&"test1".to_string()));

        let codebase_list_2 = client_2.get_codebase_list().await;
        assert!(codebase_list_2.is_ok());
        assert!(codebase_list_2.unwrap().contains(&"test2".to_string()));
    }
}
