use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::routes;
use crate::state::AppState;

/// Build the API router
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // Health check
        .route("/health", get(routes::health_check))
        // Machines
        .route("/machines", get(routes::list_machines))
        .route("/machines", post(routes::create_machine))
        .route("/machines/{id}", get(routes::get_machine))
        .route("/machines/{id}", put(routes::update_machine))
        .route("/machines/{id}", delete(routes::delete_machine))
        .route("/machines/{id}/test", post(routes::test_machine_connection))
        // Registered flakes
        .route("/flakes", get(routes::list_flakes))
        .route("/flakes", post(routes::create_flake))
        .route("/flakes/metadata", post(routes::get_metadata_by_path))
        .route("/flakes/{id}", get(routes::get_flake))
        .route("/flakes/{id}", put(routes::update_flake))
        .route("/flakes/{id}", delete(routes::delete_flake))
        .route("/flakes/{id}/refresh", post(routes::refresh_flake_metadata))
        .route("/flakes/{id}/outputs", get(routes::get_flake_outputs))
        .route("/flakes/{id}/inputs/{input_name}", put(routes::update_flake_input))
        .route("/flakes/{id}/lock/update", post(routes::update_flake_lock))
        // Installation jobs
        .route("/install/jobs", get(routes::list_jobs))
        .route("/install/jobs", post(routes::create_job))
        .route("/install/jobs/{id}", get(routes::get_job))
        .route("/install/jobs/{id}", delete(routes::delete_job))
        .route("/install/jobs/{id}/start", post(routes::start_job))
        .route("/install/jobs/{id}/cancel", post(routes::cancel_job))
        .route("/install/vm-test", post(routes::start_vm_test))
        // Deployment jobs
        .route("/deploy/jobs", get(routes::list_deploy_jobs))
        .route("/deploy/jobs", post(routes::create_deploy_job))
        .route("/deploy/jobs/{id}", get(routes::get_deploy_job))
        .route("/deploy/jobs/{id}", delete(routes::delete_deploy_job))
        .route("/deploy/jobs/{id}/start", post(routes::start_deploy_job))
        .route("/deploy/jobs/{id}/cancel", post(routes::cancel_deploy_job))
        .route("/deploy/generations", post(routes::list_generations))
        .route("/deploy/rollback", post(routes::rollback))
        // Services
        .route("/machines/{id}/services", get(routes::list_services))
        .route("/machines/{id}/services/failed", get(routes::list_failed_services))
        .route("/machines/{id}/services/{service}", get(routes::get_service))
        .route("/machines/{id}/services/{service}/action", post(routes::service_action))
        .route("/machines/{id}/services/{service}/logs", get(routes::get_service_logs))
        .route("/machines/{id}/logs", get(routes::get_system_logs))
        // Nix operations (raw, for unregistered flakes)
        .route("/nix/flake/metadata", post(routes::get_flake_metadata))
        .route("/nix/flake/show/{flake_ref}", get(routes::get_flake_show))
        .route("/nix/eval", post(routes::eval_expression))
        // Store operations
        .route("/nix/store/info", get(routes::get_store_info))
        .route("/nix/path-info", post(routes::get_path_info))
        .route("/nix/closure-size", post(routes::get_closure_size))
        // Search and analysis
        .route("/nix/search", post(routes::search_packages))
        .route("/nix/why-depends", post(routes::why_depends))
        .route("/nix/derivation/show", post(routes::show_derivation))
        // WebSocket endpoints for streaming output
        .route("/ws/flakes/{id}/lock/update", get(routes::ws_flake_lock_update))
        .route("/ws/nix/build", get(routes::ws_nix_build))
        .route("/ws/nix/gc", get(routes::ws_nix_gc))
        .route("/ws/install/jobs/{id}", get(routes::ws_job_output))
        .route("/ws/install/jobs/{id}/start", get(routes::ws_start_job))
        .route("/ws/deploy/jobs/{id}", get(routes::ws_deploy_job_output))
        .route("/ws/deploy/jobs/{id}/start", get(routes::ws_start_deploy_job))
        .route("/ws/deploy/rollback", get(routes::ws_rollback))
        // WebSocket for service log streaming
        .route("/ws/machines/{id}/services/{service}/logs", get(routes::ws_stream_service_logs))
        .route("/ws/machines/{id}/logs", get(routes::ws_stream_system_logs))
        // WebSocket for nix store operations
        .route("/ws/nix/flake/check", get(routes::ws_flake_check))
        .route("/ws/nix/store/optimise", get(routes::ws_store_optimise))
        .route("/ws/nix/store/verify", get(routes::ws_store_verify))
        .route("/ws/nix/store/repair", get(routes::ws_store_repair))
        // Secrets management
        .route("/secrets/keys", get(routes::list_keys))
        .route("/secrets/keys", post(routes::generate_key))
        .route("/secrets/keys/import", post(routes::import_key))
        .route("/secrets/keys/public", get(routes::get_public_keys))
        .route("/secrets", get(routes::list_secrets))
        .route("/secrets", post(routes::create_secret))
        .route("/secrets/{id}", get(routes::get_secret))
        .route("/secrets/{id}", put(routes::update_secret))
        .route("/secrets/{id}", delete(routes::delete_secret))
        .route("/secrets/{id}/value", get(routes::get_secret_value))
        .route("/secrets/config/sops", post(routes::update_sops_config))
        .route("/secrets/config/sops-nix", get(routes::generate_sops_nix_config));

    Router::new()
        .nest("/api", api_routes)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
