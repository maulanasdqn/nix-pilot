//! np-ui - Web UI for nix-pilot
//!
//! This crate provides the Leptos-based web interface for managing
//! NixOS deployments, flakes, and services.

#![recursion_limit = "512"]

pub mod api;
pub mod app;
pub mod components;
pub mod pages;
pub mod state;

pub use app::App;

/// Client-side rendering entry point for WASM
/// Used when deploying as static files without SSR
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    // Use mount_to_body for CSR (no server-rendered HTML)
    leptos::mount::mount_to_body(App);
}
