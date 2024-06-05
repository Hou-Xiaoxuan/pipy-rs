use reqwest;

/// GET /api/v1/repo
pub async fn get_codebase_list(host: &str, port: u16) -> Result<Vec<String>, reqwest::Error> {
    let url = format!("http://{}:{}/api/v1/repo", host, port);
    let resp = reqwest::get(&url).await?;
    tracing::debug!("Response: {:?}", resp);
    // split the response by '\n'
    let codebase_list: Vec<String> = resp
        .text()
        .await?
        .split('\n')
        .map(|s| s.to_string())
        .collect();
    Ok(codebase_list)
}

/// POST /api/v1/repo/${codebase_name}
pub async fn create_codebase(
    host: &str,
    port: u16,
    codebase_name: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("http://{}:{}/api/v1/repo/{}", host, port, codebase_name);
    let resp = reqwest::Client::new().post(&url).send().await?;
    tracing::debug!("Response: {:?}", resp);
    resp.error_for_status()?;
    Ok(())
}
