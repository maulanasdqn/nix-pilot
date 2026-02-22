use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::api::check_response_status;
use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RegisteredFlake {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DetectedFlake {
    path: String,
    name: String,
    description: Option<String>,
    is_registered: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlakeListResponse {
    #[serde(default)]
    flakes: Vec<RegisteredFlake>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DetectFlakesResponse {
    flakes: Vec<DetectedFlake>,
}

async fn fetch_flakes() -> Result<Vec<RegisteredFlake>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/flakes", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    check_response_status(resp.status(), resp.ok())?;

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    // API returns array directly
    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
}

async fn detect_flakes() -> Result<Vec<DetectedFlake>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/flakes/detect", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    check_response_status(resp.status(), resp.ok())?;

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let response: DetectFlakesResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(response.flakes)
}

async fn register_flake(path: String, name: String) -> Result<RegisteredFlake, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({
        "path": path,
        "name": name
    });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/flakes", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }
    request.headers().set("Content-Type", "application/json").ok();

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    check_response_status(resp.status(), resp.ok())?;

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
}

/// Flake list page
#[component]
pub fn FlakeListPage() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (flakes, set_flakes) = signal(Vec::<RegisteredFlake>::new());
    let (detected, set_detected) = signal(Vec::<DetectedFlake>::new());
    let (detecting, set_detecting) = signal(false);
    let (action_msg, set_action_msg) = signal(Option::<String>::None);

    // Fetch flakes on mount
    leptos::task::spawn_local(async move {
        match fetch_flakes().await {
            Ok(f) => {
                set_flakes.set(f);
                set_loading.set(false);
            }
            Err(e) => {
                set_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });

    let on_detect = move |_| {
        set_detecting.set(true);
        set_action_msg.set(Some("Scanning for flakes...".to_string()));
        leptos::task::spawn_local(async move {
            match detect_flakes().await {
                Ok(d) => {
                    let count = d.len();
                    set_detected.set(d);
                    set_detecting.set(false);
                    if count > 0 {
                        set_action_msg.set(Some(format!("Found {} flake(s)", count)));
                    } else {
                        set_action_msg.set(Some("No flakes found at common locations".to_string()));
                    }
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Detection failed: {}", e)));
                    set_detecting.set(false);
                }
            }
        });
    };

    let on_register = move |path: String, name: String| {
        set_action_msg.set(Some(format!("Registering {}...", name)));
        leptos::task::spawn_local(async move {
            match register_flake(path, name.clone()).await {
                Ok(flake) => {
                    set_flakes.update(|f| f.push(flake));
                    set_detected.update(|d| {
                        d.retain(|f| f.name != name);
                    });
                    set_action_msg.set(Some(format!("Registered {} successfully", name)));
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Failed to register: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-foreground">
                    "Flakes"
                </h1>
                <div class="flex space-x-3">
                    <button
                        class="inline-flex items-center px-4 py-2 bg-secondary hover:bg-secondary/80 text-secondary-foreground text-sm font-medium rounded-md transition-colors disabled:opacity-50"
                        on:click=on_detect
                        disabled=move || detecting.get()
                    >
                        {move || if detecting.get() { "Detecting..." } else { "Detect Flakes" }}
                    </button>
                    <A
                        href="/flakes/add"
                        attr:class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors"
                    >
                        "+ Register Flake"
                    </A>
                </div>
            </div>

            {move || error.get().map(|e| view! {
                <div class="p-4 bg-red-500/10 border border-red-500/20 text-red-400 rounded-lg">
                    {e}
                </div>
            })}

            {move || action_msg.get().map(|msg| view! {
                <div class="p-4 bg-blue-500/10 border border-blue-500/20 text-blue-400 rounded-lg">
                    {msg}
                </div>
            })}

            // Detected flakes section
            <Show when=move || !detected.get().is_empty()>
                <Card title="Detected Flakes".to_string()>
                    <p class="text-sm text-muted-foreground mb-4">
                        "Flakes found on your system that can be registered:"
                    </p>
                    <div class="space-y-3">
                        {move || detected.get().into_iter().filter(|d| !d.is_registered).map(|d| {
                            let path = d.path.clone();
                            let name = d.name.clone();
                            let path_for_register = path.clone();
                            let name_for_register = name.clone();
                            let on_register_clone = on_register.clone();
                            view! {
                                <div class="flex items-center justify-between p-4 bg-muted/50 rounded-lg">
                                    <div class="flex-1">
                                        <div class="font-medium text-foreground">{name}</div>
                                        <div class="text-sm text-muted-foreground font-mono">{path}</div>
                                        {d.description.map(|desc| view! {
                                            <div class="text-sm text-muted-foreground mt-1">{desc}</div>
                                        })}
                                    </div>
                                    <button
                                        class="px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md"
                                        on:click=move |_| on_register_clone(path_for_register.clone(), name_for_register.clone())
                                    >
                                        "Register"
                                    </button>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </Card>
            </Show>

            // Loading state
            <Show when=move || loading.get()>
                <div class="text-center py-12">
                    <p class="text-muted-foreground">"Loading flakes..."</p>
                </div>
            </Show>

            // Registered flakes section
            <Show when=move || !loading.get()>
                {move || {
                    let registered_flakes = flakes.get();
                    if registered_flakes.is_empty() {
                        view! {
                            <Card>
                                <div class="text-center py-12">
                                    <div class="text-muted-foreground text-5xl mb-4">"*"</div>
                                    <h3 class="text-lg font-medium text-foreground mb-2">
                                        "No flakes registered"
                                    </h3>
                                    <p class="text-muted-foreground mb-4">
                                        "Click \"Detect Flakes\" to find flakes on your system, or register one manually."
                                    </p>
                                    <div class="flex justify-center space-x-3">
                                        <button
                                            class="inline-flex items-center px-4 py-2 bg-secondary hover:bg-secondary/80 text-secondary-foreground text-sm font-medium rounded-md"
                                            on:click=on_detect
                                            disabled=move || detecting.get()
                                        >
                                            {move || if detecting.get() { "Detecting..." } else { "Detect Flakes" }}
                                        </button>
                                        <A
                                            href="/flakes/add"
                                            attr:class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md"
                                        >
                                            "Register Manually"
                                        </A>
                                    </div>
                                </div>
                            </Card>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-4">
                                <h2 class="text-lg font-semibold text-foreground">"Registered Flakes"</h2>
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    {registered_flakes.into_iter().map(|flake| {
                                        let href = format!("/flakes/{}", flake.id);
                                        view! {
                                            <A href=href attr:class="block">
                                                <Card class="hover:bg-muted/50 transition-colors cursor-pointer h-full".to_string()>
                                                    <div class="space-y-2">
                                                        <h3 class="text-lg font-medium text-foreground">
                                                            {flake.name.clone()}
                                                        </h3>
                                                        {flake.description.map(|desc| view! {
                                                            <p class="text-sm text-muted-foreground line-clamp-2">
                                                                {desc}
                                                            </p>
                                                        })}
                                                        <div class="text-sm text-muted-foreground font-mono truncate">
                                                            {flake.path}
                                                        </div>
                                                    </div>
                                                </Card>
                                            </A>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Show>
        </div>
    }
}
