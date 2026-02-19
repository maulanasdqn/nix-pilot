//! np-ui - Web UI for nix-pilot
//!
//! This crate provides the Leptos-based web interface for managing
//! NixOS deployments, flakes, and services.

pub mod api;
pub mod app;
pub mod components;
pub mod pages;
pub mod state;

pub use app::App;

/// Hydration entry point for WASM
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
