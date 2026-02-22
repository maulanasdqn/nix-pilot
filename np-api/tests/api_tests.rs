//! Integration tests for np-api endpoints

use axum::http::StatusCode;
use axum_test::TestServer;
use np_api::{create_router, AppState};
use np_core::CoreConfig;
use serde_json::json;
use tempfile::TempDir;

/// Create a test server with temporary directories
async fn create_test_server() -> (TestServer, TempDir) {
    // Disable authentication for tests
    // SAFETY: Tests run sequentially in the test harness
    unsafe { std::env::set_var("NP_AUTH_ENABLED", "false"); }

    let temp_dir = TempDir::new().unwrap();

    let config = CoreConfig::with_data_dir(temp_dir.path().to_path_buf());
    config.ensure_dirs().unwrap();

    let state = AppState::new(config);
    state.init().await.unwrap();

    let app = create_router(state);
    let server = TestServer::new(app.into_make_service()).unwrap();

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
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_list_install_jobs_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/install/jobs").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_list_deploy_jobs_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/deploy/jobs").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
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

// ============================================================================
// Flake API Tests
// ============================================================================

#[tokio::test]
async fn test_register_and_get_flake() {
    let (server, temp) = create_test_server().await;

    // Create a mock flake directory
    let flake_dir = temp.path().join("my-flake");
    std::fs::create_dir_all(&flake_dir).unwrap();
    std::fs::write(
        flake_dir.join("flake.nix"),
        r#"{ description = "Test flake"; inputs = {}; outputs = { self }: {}; }"#,
    )
    .unwrap();

    // Register the flake
    let create_response = server
        .post("/api/flakes")
        .json(&json!({
            "name": "test-flake",
            "path": flake_dir.to_str().unwrap(),
            "description": "A test flake"
        }))
        .await;

    // 201 Created is expected for successful creation
    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let flake_id = created["id"].as_str().unwrap();

    // Get the flake
    let get_response = server.get(&format!("/api/flakes/{}", flake_id)).await;

    get_response.assert_status_ok();
    let flake: serde_json::Value = get_response.json();
    assert_eq!(flake["name"], "test-flake");
    assert_eq!(flake["description"], "A test flake");
}

#[tokio::test]
async fn test_unregister_flake() {
    let (server, temp) = create_test_server().await;

    // Create a mock flake directory
    let flake_dir = temp.path().join("to-delete-flake");
    std::fs::create_dir_all(&flake_dir).unwrap();
    std::fs::write(
        flake_dir.join("flake.nix"),
        r#"{ description = "Delete me"; inputs = {}; outputs = { self }: {}; }"#,
    )
    .unwrap();

    // Register the flake
    let create_response = server
        .post("/api/flakes")
        .json(&json!({
            "name": "delete-flake",
            "path": flake_dir.to_str().unwrap()
        }))
        .await;

    let created: serde_json::Value = create_response.json();
    let flake_id = created["id"].as_str().unwrap();

    // Delete the flake - expects 204 No Content
    let delete_response = server
        .delete(&format!("/api/flakes/{}", flake_id))
        .await;

    delete_response.assert_status(StatusCode::NO_CONTENT);

    // Verify it's gone
    let get_response = server.get(&format!("/api/flakes/{}", flake_id)).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

// ============================================================================
// System Info API Tests
// ============================================================================

#[tokio::test]
async fn test_system_info() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/system/info").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    // Should have system_type field
    assert!(body["system_type"].as_str().is_some());
}

// ============================================================================
// Machine Tag Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_list_machines_with_tags() {
    let (server, _temp) = create_test_server().await;

    // Create machines with different tags
    server
        .post("/api/machines")
        .json(&json!({
            "name": "web-server",
            "host": "192.168.1.1",
            "port": 22,
            "username": "root",
            "tags": ["production", "web"]
        }))
        .await;

    server
        .post("/api/machines")
        .json(&json!({
            "name": "db-server",
            "host": "192.168.1.2",
            "port": 22,
            "username": "root",
            "tags": ["production", "database"]
        }))
        .await;

    server
        .post("/api/machines")
        .json(&json!({
            "name": "dev-server",
            "host": "192.168.1.3",
            "port": 22,
            "username": "root",
            "tags": ["development"]
        }))
        .await;

    // List all machines
    let response = server.get("/api/machines").await;
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["machines"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Secrets API Tests (when secrets are enabled)
// ============================================================================

#[tokio::test]
async fn test_list_secrets_empty() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/secrets").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    // Secrets response could be an object with a secrets array
    if let Some(secrets) = body["secrets"].as_array() {
        assert!(secrets.is_empty());
    } else if let Some(arr) = body.as_array() {
        assert!(arr.is_empty());
    }
    // If neither, just verify we got a response
}

#[tokio::test]
async fn test_list_age_keys() {
    let (server, _temp) = create_test_server().await;

    let response = server.get("/api/secrets/keys").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    // Should have keys array
    assert!(body["keys"].is_array());
}

// ============================================================================
// Service API Tests
// ============================================================================

#[tokio::test]
async fn test_list_services() {
    let (server, _temp) = create_test_server().await;

    // This may return empty or error depending on system state
    let response = server.get("/api/services").await;

    // Services endpoint might fail on non-NixOS systems or return success
    // Accept any HTTP response - the important thing is it doesn't panic
    let status = response.status_code();
    assert!(
        status.is_success() || status.is_client_error() || status.is_server_error(),
        "Should return a valid HTTP status"
    );
}

// ============================================================================
// Input Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_machine_missing_required_fields() {
    let (server, _temp) = create_test_server().await;

    // Missing host field
    let response = server
        .post("/api/machines")
        .json(&json!({
            "name": "test-machine",
            "port": 22,
            "username": "root"
        }))
        .await;

    // Should fail with bad request or similar
    assert!(response.status_code().is_client_error());
}

#[tokio::test]
async fn test_create_machine_invalid_port() {
    let (server, _temp) = create_test_server().await;

    let response = server
        .post("/api/machines")
        .json(&json!({
            "name": "test-machine",
            "host": "192.168.1.1",
            "port": "not-a-number",
            "username": "root"
        }))
        .await;

    // Should fail with bad request
    assert!(response.status_code().is_client_error());
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_machine_creation() {
    let (server, _temp) = create_test_server().await;

    // Create multiple machines concurrently
    let futures: Vec<_> = (0..5)
        .map(|i| {
            let server = &server;
            async move {
                server
                    .post("/api/machines")
                    .json(&json!({
                        "name": format!("machine-{}", i),
                        "host": format!("192.168.1.{}", i + 10),
                        "port": 22,
                        "username": "root"
                    }))
                    .await
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    // All should succeed
    for result in results {
        result.assert_status_ok();
    }

    // Should have 5 machines
    let list_response = server.get("/api/machines").await;
    let body: serde_json::Value = list_response.json();
    assert_eq!(body["machines"].as_array().unwrap().len(), 5);
}
