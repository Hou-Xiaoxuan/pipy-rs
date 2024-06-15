use std::thread;

use pipy_rs;

#[tokio::test]
pub async fn start_ztm_agent() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )
    .unwrap();
    let port = 6060;
    let pipy = pipy_rs::PipyRepo::new(port);
    pipy.start();

    let agent_files = vec!["api.js", "db.js", "main.js", "mesh.js", "options.js"];
    let agent_path = "tests/data/agent";

    let api_client = pipy_rs::api_client::ApiClient::new("127.0.0.1", port);
    let agent_name = "ztm_agent";
    api_client.create_codebase(agent_name).await.unwrap();
    for file in agent_files {
        let file_path = format!("{}/{}", agent_path, file);
        let file_content = std::fs::read(file_path).unwrap();
        api_client
            .update_file(agent_name, file, file_content)
            .await
            .unwrap();
    }
    api_client.publish_changes(agent_name).await.unwrap();
    let _ = api_client.get_codebase(agent_name).await.unwrap();
    api_client.start_repo(agent_name).await.unwrap();
    tracing::info!("start ztm agent");

    // test curl localhost:7777
    let resp = reqwest::get("http://127.0.0.1:7777/api/version")
        .await
        .unwrap();
    tracing::debug!("resp: {:?}", resp);
    assert!(resp.status().is_success());
    tracing::info!("test ztm agent success");
}
