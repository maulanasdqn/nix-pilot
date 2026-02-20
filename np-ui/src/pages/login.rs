use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    message: String,
}

async fn do_login(username: String, password: String) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;

    let body = serde_json::json!({
        "username": username,
        "password": password
    });

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/auth/login", &opts)
        .map_err(|_| "Failed to create request")?;

    request
        .headers()
        .set("Content-Type", "application/json")
        .ok();

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let data: LoginResponse =
        serde_wasm_bindgen::from_value(json).map_err(|e| format!("Deserialize failed: {:?}", e))?;

    if data.success {
        if let Some(token) = data.token {
            // Store token in localStorage
            let storage = window
                .local_storage()
                .map_err(|_| "No storage")?
                .ok_or("No storage")?;
            storage
                .set_item("np_token", &token)
                .map_err(|_| "Failed to store token")?;
            Ok(token)
        } else {
            Err("No token in response".to_string())
        }
    } else {
        Err(data.message)
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let u = username.get();
        let p = password.get();

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match do_login(u, p).await {
                Ok(_) => {
                    // Redirect to dashboard
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
            <Card>
                <div class="w-96 p-6">
                    <div class="text-center mb-8">
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            "Nix Pilot"
                        </h1>
                        <p class="text-gray-500 dark:text-gray-400 mt-2">
                            "Sign in to manage your NixOS systems"
                        </p>
                    </div>

                    {move || error.get().map(|e| view! {
                        <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
                            {e}
                        </div>
                    })}

                    <form on:submit=on_submit>
                        <div class="mb-4">
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Username"
                            </label>
                            <input
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-800 dark:border-gray-600 dark:text-white"
                                placeholder="admin"
                                prop:value=move || username.get()
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div class="mb-6">
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Password"
                            </label>
                            <input
                                type="password"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-800 dark:border-gray-600 dark:text-white"
                                placeholder="Enter your password"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            class="w-full py-2 px-4 bg-indigo-600 hover:bg-indigo-700 text-white font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                        </button>
                    </form>
                </div>
            </Card>
        </div>
    }
}
