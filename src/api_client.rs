///! Pipy Repo RESTful API Client
///! according to the API doc: https://flomesh.io/pipy/docs/en/operating/repo/3-api
///! some details may be different, please refer to pipy code in `pipy/src/admin-service.cpp`
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
    #[serde(rename_all = "camelCase")]
    pub struct Codebase {
        pub version: String,
        pub path: String,
        pub main: String,            // entry script
        pub files: Vec<String>,      // file list
        pub edit_files: Vec<String>, // files that have been modified but not submitted
        pub erased_files: Vec<String>,
        pub base_files: Vec<String>,
        pub derived: Vec<String>,
        // pub instances: Vec<String>, // TODO: didn't know schema, ignore temporarily
    }

    /// GET /api/v1/repo
    pub async fn get_codebase_list(host: &str, port: u16) -> Result<Vec<String>, ApiError> {
        let url = format!("http://{}:{}/api/v1/repo", host, port);
        let resp = reqwest::get(&url).await?;
        tracing::debug!("get_codebase_list: {:?}", resp);
        // split the response by '\n'
        let test = resp.text().await?;
        if test.is_empty() {
            return Ok(vec![]);
        } else {
            let codebase_list = test
                .split('\n')
                .map(|s| s[1..].to_string()) // remove prefix '/'
                .collect();
            Ok(codebase_list)
        }
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
        tracing::debug!("create_codebase: {:?}", resp);
        if let Err(e) = resp.error_for_status_ref() {
            tracing::debug!("create_codebase Error, body: {:?}", resp.text().await?);
            return Err(e.into());
        }
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
        tracing::debug!("get_codebase: {:?}", resp);
        let data = resp.bytes().await?;
        tracing::debug!("get_codebase data: {:?}", data);
        let codebase: Codebase = serde_json::from_slice(&data)?;
        Ok(codebase)
    }

    /// GET /api/v1/repo-files/[CODEBASE]/[FILE_NAME]
    pub async fn get_file(
        host: &str,
        port: u16,
        codebase_name: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, ApiError> {
        let codebase_info = get_codebase(host, port, codebase_name).await?;
        if !codebase_info.files.contains(&file_name.to_string())
            && !codebase_info.files.contains(&format!("/{}", file_name))
        {
            return Err(ApiError::NotFountError(file_name.to_string()));
        }

        let url = format!(
            "http://{}:{}/api/v1/repo-files/{}/{}",
            host, port, codebase_name, file_name
        );
        let resp = reqwest::get(&url).await?;
        tracing::debug!("get_file: {:?}", resp);
        let data = resp.bytes().await?;
        Ok(data.to_vec())
    }

    /// POST /api/v1/repo-files/[CODEBASE]/[FILE_NAME]
    pub async fn update_file(
        host: &str,
        port: u16,
        codebase_name: &str,
        file_name: &str,
        data: Vec<u8>,
    ) -> Result<(), ApiError> {
        let _ = get_codebase(host, port, codebase_name).await?;

        let url = format!(
            "http://{}:{}/api/v1/repo-files/{}/{}",
            host, port, codebase_name, file_name
        );
        let resp = reqwest::Client::new().post(&url).body(data).send().await?;
        tracing::debug!("update_file: {:?}", resp);
        if let Err(e) = resp.error_for_status_ref() {
            tracing::debug!("update_file Error, body: {:?}", resp.text().await?);
            return Err(e.into());
        }
        Ok(())
    }

    /// POST /api/v1/repo/[CODEBASE]
    /// $ curl -X PATCH http://localhost:6060/api/v1/repo/hello --data '{"version": '2'}'
    pub async fn publish_changes(
        host: &str,
        port: u16,
        codebase_name: &str,
    ) -> Result<(), ApiError> {
        let codebase_info = get_codebase(host, port, codebase_name).await?;
        let version_now: usize = codebase_info.version.parse().unwrap();
        let url = format!("http://{}:{}/api/v1/repo/{}", host, port, codebase_name);
        let body = format!(r#"{{"version": "{}"}}"#, version_now + 1);
        let resp = reqwest::Client::new().patch(&url).body(body).send().await?;
        tracing::debug!("publish_changes: {:?}", resp);
        if let Err(e) = resp.error_for_status_ref() {
            tracing::debug!("publish_changes Error, body: {:?}", resp.text().await?);
            return Err(e.into());
        }
        Ok(())
    }

    /// POST /api/v1/program
    /// $ curl -X POST http://localhost:6060/api/v1/program --data '/repo_name'
    pub async fn start_repo(host: &str, port: u16, codebase_name: &str) -> Result<(), ApiError> {
        let _ = get_codebase(host, port, codebase_name).await?;

        let url = format!("http://{}:{}/api/v1/program", host, port);
        let body = format!(r#"/{}"#, codebase_name);
        let resp = reqwest::Client::new().post(&url).body(body).send().await?;
        tracing::debug!("start_repo: {:?}", resp);
        if let Err(e) = resp.error_for_status_ref() {
            tracing::debug!("start_repo Error, body: {:?}", resp.text().await?);
            return Err(e.into());
        }
        Ok(())
    }

    /// GET /api/v1/program
    /// look up the running program
    pub async fn current_repo(host: &str, port: u16) -> Result<Option<String>, ApiError> {
        let url = format!("http://{}:{}/api/v1/program", host, port);
        let resp = reqwest::get(&url).await?;
        tracing::debug!("current_repo: {:?}", resp);
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
        tracing::debug!("stop_repo: {:?}", resp);
        if let Err(e) = resp.error_for_status_ref() {
            tracing::debug!("stop_repo Error, body: {:?}", resp.text().await?);
            return Err(e.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{api_client::api::ApiError, start_pipy_repo, util::init_logger};

    use super::{api::Codebase, ApiClient};

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
        init_logger("info");
        let pipy_port = 6060;
        start_pipy_repo(Some(pipy_port));

        let repo_name = "hello";
        let client = ApiClient::new("127.0.0.1", pipy_port);
        matches!(
            client.get_codebase(repo_name).await.err().unwrap(),
            ApiError::NotFountError(_)
        );

        // create codebase
        client.create_codebase(repo_name).await.unwrap();
        let _ = client.get_codebase(repo_name).await.unwrap();

        // update `main.js` to a simple http server
        let default_main_js = client.get_file(repo_name, "main.js").await.unwrap();
        let default_main_js_code = String::from_utf8_lossy(&default_main_js);
        tracing::info!("default_main_js: {:?}", default_main_js_code);
        let main_js = r#"pipy().listen(8080).serveHTTP(new Message('Hello world!'))"#;
        client
            .update_file(repo_name, "main.js", main_js.as_bytes().to_vec())
            .await
            .unwrap();
        let codebase = client.get_codebase(repo_name).await.unwrap();
        assert!(!codebase.edit_files.is_empty(), "edit_file failed");

        // publish changes
        client.publish_changes(repo_name).await.unwrap(); // TODO: update_file seems not work, api may be wrong
        let codebase = client.get_codebase(repo_name).await.unwrap();
        assert!(codebase.edit_files.is_empty(), "publish_changes failed");

        // start the repo
        let running_repo = client.current_repo().await.unwrap();
        assert!(running_repo.is_none(), "should not have running repo");
        client.start_repo(repo_name).await.unwrap();

        let resp = reqwest::get("http://127.0.0.1:8080")
            .await
            .expect("repo not started")
            .text()
            .await
            .unwrap();
        assert_eq!(resp, "Hello world!");

        // replace the repo with another one
        let another_repo_name = "world";
        client.create_codebase(another_repo_name).await.unwrap();
        client.start_repo(another_repo_name).await.unwrap();
        let running_repo = client.current_repo().await.unwrap();
        assert_eq!(running_repo.unwrap(), another_repo_name);

        // stop the repo
        client.stop_repo().await.unwrap();
        let running_repo = client.current_repo().await.unwrap();
        assert!(running_repo.is_none(), "should not have running repo");

        unsafe {
            libc::exit(0); // exit the test. Otherwise, the `pipy-main` thread will report `panic!`, wait for a better solution
        }
    }
}
