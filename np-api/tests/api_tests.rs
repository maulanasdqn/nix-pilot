//! Integration tests for np-api endpoints

use axum::http::StatusCode;
use axum_test::TestServer;
use np_api::{create_router, AppState};
use np_core::CoreConfig;
use serde_json::json;
use tempfile::TempDir;

/// Create a test server with temporary directories
async fn create_test_server() -> (TestServer, TempDir) {
    let temp_dir = TempDir::new().unwrap();

    let config = CoreConfig {
        nix_path: std::path::PathBuf::from("nix"),
        machines_dir: temp_dir.path().join("machines"),
        flakes_dir: temp_dir.path().join("flakes"),
        jobs_dir: temp_dir.path().join("jobs"),
        command_timeout_secs: 60,
        max_concurrent_jobs: 2,
    };

    // Create directories
    std::fs::create_dir_all(&config.machines_dir).unwrap();
    std::fs::create_dir_all(&config.flakes_dir).unwrap();
    std::fs::create_dir_all(&config.jobs_dir).unwrap();

    let state = AppState::new(config);
    state.init().await.unwrap();

    let app = create_router(state);
    let server = TestServer::new(app).unwrap();

    (server, temp_dir)
}

#[tokio::test]
async fn test_health_check() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/health").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_list_machines_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/machines").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["machines"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_and_get_machine() {
    let (server, _temp) = create_test_server().await;

    // Create a machine
    let create_response = server
        .post("/api/machines")
        .json(&json!({
            "name": "test-machine",
            "host": "192.168.1.100",
            "port": 22,
            "username": "root",
            "auth_method": { "type": "agent" }
        }))
        .await;

    create_response.assert_status_ok();
    let created: serde_json::Value = create_response.json();
    let machine_id = created["machine"]["id"].as_str().unwrap();

    // Get the machine
    let get_response = server.get(&format!("/api/machines/{}", machine_id)).await;

    get_response.assert_status_ok();
    let machine: serde_json::Value = get_response.json();
    assert_eq!(machine["machine"]["name"], "test-machine");
    assert_eq!(machine["machine"]["target"]["host"], "192.168.1.100");
}

#[tokio::test]
async fn test_update_machine() {
    let (server, _temp) = create_test_server().await;

    // Create a machine
    let create_response = server
        .post("/api/machines")
        .json(&json!({
            "name": "original-name",
            "host": "192.168.1.100",
            "port": 22,
            "username": "root"
        }))
        .await;

    let created: serde_json::Value = create_response.json();
    let machine_id = created["machine"]["id"].as_str().unwrap();

    // Update the machine
    let update_response = server
        .put(&format!("/api/machines/{}", machine_id))
        .json(&json!({
            "name": "updated-name",
            "description": "A test machine"
        }))
        .await;

    update_response.assert_status_ok();
    let updated: serde_json::Value = update_response.json();
    assert_eq!(updated["machine"]["name"], "updated-name");
    assert_eq!(updated["machine"]["description"], "A test machine");
}

#[tokio::test]
async fn test_delete_machine() {
    let (server, _temp) = create_test_server().await;

    // Create a machine
    let create_response = server
        .post("/api/machines")
        .json(&json!({
            "name": "to-delete",
            "host": "192.168.1.100",
            "port": 22,
            "username": "root"
        }))
        .await;

    let created: serde_json::Value = create_response.json();
    let machine_id = created["machine"]["id"].as_str().unwrap();

    // Delete the machine
    let delete_response = server
        .delete(&format!("/api/machines/{}", machine_id))
        .await;

    delete_response.assert_status_ok();

    // Verify it's gone
    let get_response = server.get(&format!("/api/machines/{}", machine_id)).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_machine() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/machines/nonexistent-id").await;

    response.assert_status(StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().is_some());
    assert!(body["code"].as_str().is_some());
}

#[tokio::test]
async fn test_list_flakes_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/flakes").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["flakes"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_install_jobs_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/install/jobs").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["jobs"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_deploy_jobs_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/deploy/jobs").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["jobs"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_nix_eval_expression() {
    let (server, _temp) = create_test_server().await;

    // This will only work if nix is installed
    let response = server
        .post("/api/nix/eval")
        .json(&json!({
            "expr": "1 + 1"
        }))
        .await;

    // If nix is not installed, we expect an error
    // If it is, we expect success with result = 2
    if response.status_code() == StatusCode::OK {
        let body: serde_json::Value = response.json();
        assert_eq!(body["result"], 2);
    }
}

#[tokio::test]
async fn test_error_response_format() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/machines/invalid-id").await;

    response.assert_status(StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();

    // Check error response structure
    assert!(body["error"].as_str().is_some(), "Should have 'error' field");
    assert!(body["code"].as_str().is_some(), "Should have 'code' field");
}
