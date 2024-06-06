use api::ApiError;

pub struct ApiClient {
    host: String,
    port: u16,
}
impl ApiClient {
    pub fn new(host: &str, port: u16) -> Self {
        ApiClient {
            host: host.to_string(),
            port,
        }
    }
    pub async fn get_codebase_list(&self) -> Result<Vec<String>, ApiError> {
        api::get_codebase_list(&self.host, self.port).await
    }
    pub async fn create_codebase(&self, codebase_name: &str) -> Result<(), ApiError> {
        api::create_codebase(&self.host, self.port, codebase_name).await
    }
    pub async fn get_codebase(&self, codebase_name: &str) -> Result<api::Codebase, ApiError> {
        api::get_codebase(&self.host, self.port, codebase_name).await
    }
    pub async fn get_file(
        &self,
        codebase_name: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, ApiError> {
        api::get_file(&self.host, self.port, codebase_name, file_name).await
    }
    pub async fn update_file(
        &self,
        codebase_name: &str,
        file_name: &str,
        data: Vec<u8>,
    ) -> Result<(), ApiError> {
        api::update_file(&self.host, self.port, codebase_name, file_name, data).await
    }
    pub async fn publish_changes(&self, codebase_name: &str) -> Result<(), ApiError> {
        api::publish_changes(&self.host, self.port, codebase_name).await
    }
    pub async fn start_repo(&self, codebase_name: &str) -> Result<(), ApiError> {
        api::start_repo(&self.host, self.port, codebase_name).await
    }
    pub async fn current_repo(&self) -> Result<Option<String>, ApiError> {
        api::current_repo(&self.host, self.port).await
    }
    pub async fn stop_repo(&self) -> Result<(), ApiError> {
        api::stop_repo(&self.host, self.port).await
    }
}

pub mod api {
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ApiError {
        #[error("reqwest error: {0}")]
        ReqwestError(#[from] reqwest::Error),
        #[error("serde_json error: {0}")]
        SerdeJsonError(#[from] serde_json::Error),
        #[error("error: {0} not found")]
        NotFountError(String),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Codebase {
        pub version: String,
        pub path: String,
        pub main: String,            // entry script
        pub files: Vec<String>,      // file list
        pub edit_files: Vec<String>, // files that have been modified but not submitted
        pub erased_files: Vec<String>,
        pub base_files: Vec<String>,
        pub derived: Vec<String>,
        pub instances: Vec<String>,
    }

    /// GET /api/v1/repo
    pub async fn get_codebase_list(host: &str, port: u16) -> Result<Vec<String>, ApiError> {
        let url = format!("http://{}:{}/api/v1/repo", host, port);
        let resp = reqwest::get(&url).await?;
        tracing::debug!("Response: {:?}", resp);
        // split the response by '\n'
        let codebase_list: Vec<String> = resp
            .text()
            .await?
            .split('\n')
            .map(|s| s[1..].to_string()) // remove prefix '/'
            .collect();
        Ok(codebase_list)
    }

    /// POST /api/v1/repo/[CODEBASE]
    /// TODO: support create form a base codebase
    pub async fn create_codebase(
        host: &str,
        port: u16,
        codebase_name: &str,
    ) -> Result<(), ApiError> {
        let url = format!("http://{}:{}/api/v1/repo/{}", host, port, codebase_name);
        let resp = reqwest::Client::new().post(&url).send().await?;
        tracing::debug!("Response: {:?}", resp);
        resp.error_for_status()?;
        Ok(())
    }

    /// GET /api/v1/repo/[CODEBASE]
    pub async fn get_codebase(
        host: &str,
        port: u16,
        codebase_name: &str,
    ) -> Result<Codebase, ApiError> {
        let code_base_list = get_codebase_list(host, port).await?;
        if !code_base_list.contains(&codebase_name.to_string())
            && !code_base_list.contains(&format!("/{}", codebase_name))
        {
            return Err(ApiError::NotFountError(codebase_name.to_string()));
        }

        let url = format!("http://{}:{}/api/v1/repo/{}", host, port, codebase_name);
        let resp = reqwest::get(&url).await?;
        tracing::debug!("Response: {:?}", resp);
        let data = resp.bytes().await?;
        let codebase: Codebase = serde_json::from_slice(&data)?;
        Ok(codebase)
    }

    /// GET /api/v1/repo/[CODEBASE]/[FILE_NAME]
    pub async fn get_file(
        host: &str,
        port: u16,
        codebase_name: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, ApiError> {
        let codebase_info = get_codebase(host, port, codebase_name).await?;
        if !codebase_info.files.contains(&file_name.to_string()) {
            return Err(ApiError::NotFountError(file_name.to_string()));
        }

        let url = format!(
            "http://{}:{}/api/v1/repo/{}/{}",
            host, port, codebase_name, file_name
        );
        let resp = reqwest::get(&url).await?;
        tracing::debug!("Response: {:?}", resp);
        let data = resp.bytes().await?;
        Ok(data.to_vec())
    }

    /// POST /api/v1/repo/[CODEBASE]/[FILE_NAME]
    pub async fn update_file(
        host: &str,
        port: u16,
        codebase_name: &str,
        file_name: &str,
        data: Vec<u8>,
    ) -> Result<(), ApiError> {
        let _ = get_codebase(host, port, codebase_name).await?;

        let url = format!(
            "http://{}:{}/api/v1/repo/{}/{}",
            host, port, codebase_name, file_name
        );
        let resp = reqwest::Client::new().post(&url).body(data).send().await?;
        tracing::debug!("Response: {:?}", resp);
        resp.error_for_status()?;
        Ok(())
    }

    /// POST /api/v1/repo/[CODEBASE]
    /// $ curl -X POST http://localhost:6060/api/v1/repo/hello --data '{"version": '2'}'
    pub async fn publish_changes(
        host: &str,
        port: u16,
        codebase_name: &str,
    ) -> Result<(), ApiError> {
        let codebase_info = get_codebase(host, port, codebase_name).await?;
        let version_now: usize = codebase_info.version.parse().unwrap();
        let url = format!("http://{}:{}/api/v1/repo/{}", host, port, codebase_name);
        let body = format!(r#"{{"version": "{}"}}"#, version_now + 1);
        let resp = reqwest::Client::new().post(&url).body(body).send().await?;
        tracing::debug!("Response: {:?}", resp);
        resp.error_for_status()?;
        Ok(())
    }

    /// POST /api/v1/program
    /// $ curl -X POST http://localhost:6060/api/v1/program --data '/repo_name'
    pub async fn start_repo(host: &str, port: u16, codebase_name: &str) -> Result<(), ApiError> {
        let _ = get_codebase(host, port, codebase_name).await?;

        let url = format!("http://{}:{}/api/v1/program", host, port);
        let body = format!(r#""/{}""#, codebase_name);
        let resp = reqwest::Client::new().post(&url).body(body).send().await?;
        tracing::debug!("Response: {:?}", resp);
        resp.error_for_status()?;
        Ok(())
    }

    /// GET /api/v1/program
    /// look up the running program
    pub async fn current_repo(host: &str, port: u16) -> Result<Option<String>, ApiError> {
        let url = format!("http://{}:{}/api/v1/program", host, port);
        let resp = reqwest::get(&url).await?;
        tracing::debug!("Response: {:?}", resp);
        let data = resp.text().await?;
        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data[1..].to_string())) // TODO: test if only have one running program
        }
    }

    /// DELETE /api/v1/program
    /// stop the running program
    pub async fn stop_repo(host: &str, port: u16) -> Result<(), ApiError> {
        let url = format!("http://{}:{}/api/v1/program", host, port);
        let resp = reqwest::Client::new().delete(&url).send().await?;
        tracing::debug!("Response: {:?}", resp);
        resp.error_for_status()?;
        Ok(())
    }
}
mod tests {
    use libc::exit;

    use crate::{start_pipy_repo, util::init_logger};

    use super::api::*;

    #[test]
    fn test_codebase_serde() {
        let correct = r#"
                {
                    "version": "0.1",
                    "path": "/test",
                    "main": "main.py",
                    "files": ["main.py", "util.py"],
                    "edit_files": ["main.py"],
                    "erased_files": [],
                    "base_files": [],
                    "derived": [],
                    "instances": []
                }"#;
        serde_json::from_str::<Codebase>(correct).expect("correct json failed");

        let extra_field = r#"
                {
                    "version": "0.1",
                    "path": "/test",
                    "main": "main.py",
                    "files": ["main.py", "util.py"],
                    "edit_files": ["main.py"],
                    "erased_files": [],
                    "base_files": [],
                    "derived": [],
                    "instances": [],
                    "extra_field": "extra"
                }"#;
        serde_json::from_str::<Codebase>(extra_field).expect("extra json failed");

        let missing_field = r#"
                {
                    "version": "0.1",
                    "path": "/test",
                    "main": "main.py",
                    "files": ["main.py", "util.py"],
                    "edit_files": ["main.py"],
                    "erased_files": [],
                    "base_files": [],
                    "derived": []
                }"#;
        serde_json::from_str::<Codebase>(missing_field).expect_err("missing json failed");
    }

    #[tokio::test]
    async fn test_api() {
        init_logger();

        start_pipy_repo(Some(6060));
        unimplemented!();
        unsafe {
            exit(0); // exit the test. Otherwise, the `pipy-main` thread will report `panic!`, wait for a better solution
        }
    }
}
