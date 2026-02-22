use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use leptos::wasm_bindgen::JsCast;

use crate::components::icons::IconFlake;

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

    let opts = web_sys::RequestInit::new();
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
                        let _ = window.location().set_href("/dashboard");
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
        <div class="min-h-screen flex items-center justify-center bg-background">
            <div class="w-full max-w-sm">
                <div class="rounded-xl border border-border bg-card p-8">
                    <div class="text-center mb-8">
                        <div class="flex justify-center mb-4">
                            <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-primary text-primary-foreground">
                                <IconFlake />
                            </div>
                        </div>
                        <h1 class="text-2xl font-bold text-foreground">
                            "Nix Pilot"
                        </h1>
                        <p class="text-muted-foreground mt-2">
                            "Sign in to manage your NixOS systems"
                        </p>
                    </div>

                    {move || error.get().map(|e| view! {
                        <div class="mb-4 p-3 bg-destructive/10 border border-destructive/20 text-destructive rounded-lg text-sm">
                            {e}
                        </div>
                    })}

                    <form on:submit=on_submit class="space-y-4">
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">
                                "Username"
                            </label>
                            <input
                                type="text"
                                class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background"
                                placeholder="admin"
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                prop:value=move || username.get()
                                required
                            />
                        </div>

                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">
                                "Password"
                            </label>
                            <input
                                type="password"
                                class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background"
                                placeholder="Enter your password"
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                prop:value=move || password.get()
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            class="inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
