use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use leptos::wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::api::check_response_status;
use crate::components::common::Card;

/// Output line from streaming command
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OutputLine {
    content: String,
    #[serde(default)]
    is_stderr: bool,
}

/// WebSocket message types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    Output(OutputLine),
    Started { command: String },
    Completed { exit_code: i32 },
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchResult {
    #[serde(default)]
    attr_path: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PathInfo {
    #[serde(default)]
    nar_size: u64,
    #[serde(default)]
    closure_size: u64,
    #[serde(default)]
    references: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoreInfo {
    #[serde(default)]
    store_url: String,
    #[serde(default)]
    version: Option<String>,
}

async fn fetch_store_info() -> Result<StoreInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/nix/store/info", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
}

async fn search_packages(query: String) -> Result<Vec<SearchResult>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({ "query": query });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/nix/search", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }
    request.headers().set("Content-Type", "application/json").ok();

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let response: SearchResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(response.results)
}

async fn get_path_info(path: String) -> Result<PathInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({ "path": path });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/nix/path-info", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }
    request.headers().set("Content-Type", "application/json").ok();

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
}

/// Build WebSocket URL with auth token
fn build_ws_url(path: &str) -> String {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().ok().flatten();
    let token = storage.and_then(|s| s.get_item("np_token").ok().flatten()).unwrap_or_default();

    let location = window.location();
    let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
    let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
    let host = location.host().unwrap_or_else(|_| "localhost:8080".to_string());
    format!("{}//{}/api{}?token={}", ws_protocol, host, path, token)
}

/// Nix operations page
#[component]
pub fn NixOperationsPage() -> impl IntoView {
    // Store info state
    let (store_url, set_store_url) = signal("Loading...".to_string());
    let (store_version, set_store_version) = signal(Option::<String>::None);

    // Search state
    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<SearchResult>::new());
    let (is_searching, set_is_searching) = signal(false);

    // Path info state
    let (path_input, set_path_input) = signal(String::new());
    let (path_info, set_path_info) = signal(Option::<PathInfo>::None);
    let (getting_path_info, set_getting_path_info) = signal(false);

    // Operation state
    let (active_operation, set_active_operation) = signal(Option::<String>::None);
    let (operation_output, set_operation_output) = signal(Vec::<String>::new());
    let (operation_running, set_operation_running) = signal(false);

    // GC options
    let (gc_older_than, set_gc_older_than) = signal("7d".to_string());

    // Flake check options
    let (flake_ref, set_flake_ref) = signal(String::new());

    // Error state
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Fetch store info on mount
    leptos::task::spawn_local(async move {
        match fetch_store_info().await {
            Ok(info) => {
                set_store_url.set(info.store_url);
                set_store_version.set(info.version);
            }
            Err(e) => {
                set_error_msg.set(Some(format!("Failed to fetch store info: {}", e)));
            }
        }
    });

    let on_search = move |_| {
        let query = search_query.get();
        if query.is_empty() {
            return;
        }
        set_is_searching.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            match search_packages(query).await {
                Ok(results) => {
                    set_search_results.set(results);
                    set_is_searching.set(false);
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Search failed: {}", e)));
                    set_is_searching.set(false);
                }
            }
        });
    };

    let on_get_path_info = move |_| {
        let path = path_input.get();
        if path.is_empty() {
            return;
        }
        set_getting_path_info.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            match get_path_info(path).await {
                Ok(info) => {
                    set_path_info.set(Some(info));
                    set_getting_path_info.set(false);
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Path info failed: {}", e)));
                    set_getting_path_info.set(false);
                }
            }
        });
    };

    // Start garbage collection via WebSocket
    let start_gc = move |_| {
        let older_than = gc_older_than.get();
        set_active_operation.set(Some("Garbage Collection".to_string()));
        set_operation_output.set(vec![]);
        set_operation_running.set(true);

        let ws_url = build_ws_url("/ws/nix/gc");
        let ws = match web_sys::WebSocket::new(&ws_url) {
            Ok(ws) => ws,
            Err(_) => {
                set_operation_output.update(|lines| {
                    lines.push("Error: Failed to create WebSocket connection".to_string());
                });
                set_operation_running.set(false);
                return;
            }
        };

        // Clone for onopen closure
        let ws_clone = ws.clone();
        let older_than_clone = older_than.clone();

        // On open: send the request
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let request = serde_json::json!({
                "delete_older_than": if older_than_clone.is_empty() { None } else { Some(&older_than_clone) }
            });
            let msg = serde_json::to_string(&request).unwrap_or_default();
            let _ = ws_clone.send_with_str(&msg);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        setup_ws_handlers(ws, set_operation_output, set_operation_running);
    };

    // Start store optimise via WebSocket
    let start_optimise = move |_| {
        set_active_operation.set(Some("Store Optimise".to_string()));
        set_operation_output.set(vec![]);
        set_operation_running.set(true);

        let ws_url = build_ws_url("/ws/nix/store/optimise");
        let ws = match web_sys::WebSocket::new(&ws_url) {
            Ok(ws) => ws,
            Err(_) => {
                set_operation_output.update(|lines| {
                    lines.push("Error: Failed to create WebSocket connection".to_string());
                });
                set_operation_running.set(false);
                return;
            }
        };

        // Clone for onopen closure
        let ws_clone = ws.clone();

        // On open: send empty request to start
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let _ = ws_clone.send_with_str("{}");
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        setup_ws_handlers(ws, set_operation_output, set_operation_running);
    };

    // Start store verify via WebSocket
    let start_verify = move |_| {
        set_active_operation.set(Some("Store Verify".to_string()));
        set_operation_output.set(vec![]);
        set_operation_running.set(true);

        let ws_url = build_ws_url("/ws/nix/store/verify");
        let ws = match web_sys::WebSocket::new(&ws_url) {
            Ok(ws) => ws,
            Err(_) => {
                set_operation_output.update(|lines| {
                    lines.push("Error: Failed to create WebSocket connection".to_string());
                });
                set_operation_running.set(false);
                return;
            }
        };

        // Clone for onopen closure
        let ws_clone = ws.clone();

        // On open: send empty request to start
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let _ = ws_clone.send_with_str("{}");
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        setup_ws_handlers(ws, set_operation_output, set_operation_running);
    };

    // Start flake check via WebSocket
    let start_flake_check = move |_| {
        let flake = flake_ref.get();
        if flake.is_empty() {
            return;
        }
        set_active_operation.set(Some("Flake Check".to_string()));
        set_operation_output.set(vec![]);
        set_operation_running.set(true);

        let ws_url = build_ws_url("/ws/nix/flake/check");
        let ws = match web_sys::WebSocket::new(&ws_url) {
            Ok(ws) => ws,
            Err(_) => {
                set_operation_output.update(|lines| {
                    lines.push("Error: Failed to create WebSocket connection".to_string());
                });
                set_operation_running.set(false);
                return;
            }
        };

        // Clone for onopen closure
        let ws_clone = ws.clone();
        let flake_clone = flake.clone();

        // On open: send the request with flake ref
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let request = serde_json::json!({
                "flake_ref": flake_clone
            });
            let msg = serde_json::to_string(&request).unwrap_or_default();
            let _ = ws_clone.send_with_str(&msg);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        setup_ws_handlers(ws, set_operation_output, set_operation_running);
    };

    let clear_output = move |_| {
        set_operation_output.set(Vec::new());
        set_active_operation.set(None);
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-foreground ">
                    "Nix Operations"
                </h1>
            </div>

            {move || error_msg.get().map(|e| view! {
                <div class="p-3 bg-red-500/20 border border-red-500/30 text-red-700 rounded">
                    {e}
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Store Info
                <Card title="Store Info".to_string()>
                    <dl class="space-y-3">
                        <div>
                            <dt class="text-sm font-medium text-muted-foreground ">"Store URL"</dt>
                            <dd class="mt-1 text-sm text-foreground  font-mono">
                                {move || store_url.get()}
                            </dd>
                        </div>
                        {move || store_version.get().map(|v| view! {
                            <div>
                                <dt class="text-sm font-medium text-muted-foreground ">"Nix Version"</dt>
                                <dd class="mt-1 text-sm text-foreground  font-mono">
                                    {v}
                                </dd>
                            </div>
                        })}
                    </dl>
                </Card>

                // Package Search
                <Card title="Package Search".to_string()>
                    <div class="space-y-4">
                        <div class="flex space-x-2">
                            <input
                                type="text"
                                placeholder="Search packages (e.g., 'ripgrep')"
                                class="flex-1 px-3 py-2 border border-border  rounded-md bg-background text-foreground  focus:ring-ring focus:border-primary"
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                                prop:value=move || search_query.get()
                                on:keypress=move |ev: web_sys::KeyboardEvent| {
                                    if ev.key() == "Enter" {
                                        ev.prevent_default();
                                        let query = search_query.get();
                                        if query.is_empty() {
                                            return;
                                        }
                                        set_is_searching.set(true);
                                        set_error_msg.set(None);
                                        leptos::task::spawn_local(async move {
                                            match search_packages(query).await {
                                                Ok(results) => {
                                                    set_search_results.set(results);
                                                    set_is_searching.set(false);
                                                }
                                                Err(e) => {
                                                    set_error_msg.set(Some(format!("Search failed: {}", e)));
                                                    set_is_searching.set(false);
                                                }
                                            }
                                        });
                                    }
                                }
                            />
                            <button
                                class="px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md disabled:opacity-50"
                                disabled=move || is_searching.get() || search_query.get().is_empty()
                                on:click=on_search
                            >
                                {move || if is_searching.get() { "Searching..." } else { "Search" }}
                            </button>
                        </div>

                        // Search results
                        <div class="max-h-48 overflow-auto">
                            <Show
                                when=move || !search_results.get().is_empty()
                                fallback=|| view! {
                                    <p class="text-sm text-muted-foreground  text-center py-4">
                                        "Enter a search term to find packages"
                                    </p>
                                }
                            >
                                <ul class="divide-y divide-border">
                                    <For
                                        each=move || search_results.get()
                                        key=|r| r.attr_path.clone()
                                        let:result
                                    >
                                        <li class="py-2">
                                            <div class="font-mono text-sm text-primary">
                                                {result.attr_path.clone()}
                                            </div>
                                            {result.description.clone().map(|d| view! {
                                                <p class="text-xs text-muted-foreground  truncate">
                                                    {d}
                                                </p>
                                            })}
                                        </li>
                                    </For>
                                </ul>
                            </Show>
                        </div>
                    </div>
                </Card>
            </div>

            // Operations Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                // Garbage Collection
                <Card title="Garbage Collection".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-muted-foreground ">
                            "Remove unused store paths to free disk space."
                        </p>
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Delete older than"
                            </label>
                            <select
                                class="w-full px-3 py-2 border border-border  rounded-md bg-background text-foreground "
                                on:change=move |ev| set_gc_older_than.set(event_target_value(&ev))
                                prop:value=move || gc_older_than.get()
                            >
                                <option value="">"All unused"</option>
                                <option value="1d">"1 day"</option>
                                <option value="7d">"7 days"</option>
                                <option value="30d">"30 days"</option>
                                <option value="90d">"90 days"</option>
                            </select>
                        </div>
                        <button
                            class="w-full px-4 py-2 bg-destructive text-destructive-foreground hover:bg-destructive/90 text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get()
                            on:click=start_gc
                        >
                            {move || if operation_running.get() && active_operation.get().as_deref() == Some("Garbage Collection") {
                                "Running..."
                            } else {
                                "Run Garbage Collection"
                            }}
                        </button>
                    </div>
                </Card>

                // Store Optimise
                <Card title="Store Optimise".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-muted-foreground ">
                            "Deduplicate files in the Nix store using hard links."
                        </p>
                        <button
                            class="w-full px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get()
                            on:click=start_optimise
                        >
                            {move || if operation_running.get() && active_operation.get().as_deref() == Some("Store Optimise") {
                                "Running..."
                            } else {
                                "Optimise Store"
                            }}
                        </button>
                    </div>
                </Card>

                // Store Verify
                <Card title="Store Verify".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-muted-foreground ">
                            "Verify the integrity of store paths."
                        </p>
                        <button
                            class="w-full px-4 py-2 bg-secondary text-secondary-foreground hover:bg-secondary/80 text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get()
                            on:click=start_verify
                        >
                            {move || if operation_running.get() && active_operation.get().as_deref() == Some("Store Verify") {
                                "Running..."
                            } else {
                                "Verify Store"
                            }}
                        </button>
                    </div>
                </Card>

                // Flake Check
                <Card title="Flake Check".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-muted-foreground ">
                            "Run checks defined in a flake."
                        </p>
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Flake reference"
                            </label>
                            <input
                                type="text"
                                placeholder=". or github:user/repo"
                                class="w-full px-3 py-2 border border-border  rounded-md bg-background text-foreground  font-mono text-sm"
                                on:input=move |ev| set_flake_ref.set(event_target_value(&ev))
                                prop:value=move || flake_ref.get()
                            />
                        </div>
                        <button
                            class="w-full px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get() || flake_ref.get().is_empty()
                            on:click=start_flake_check
                        >
                            {move || if operation_running.get() && active_operation.get().as_deref() == Some("Flake Check") {
                                "Running..."
                            } else {
                                "Run Flake Check"
                            }}
                        </button>
                    </div>
                </Card>

                // Path Info
                <Card title="Path Info".to_string() class="md:col-span-2".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-muted-foreground ">
                            "Get information about a store path including size and references."
                        </p>
                        <div class="flex space-x-2">
                            <input
                                type="text"
                                placeholder="/nix/store/..."
                                class="flex-1 px-3 py-2 border border-border  rounded-md bg-background text-foreground  font-mono text-sm"
                                on:input=move |ev| set_path_input.set(event_target_value(&ev))
                                prop:value=move || path_input.get()
                            />
                            <button
                                class="px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md disabled:opacity-50"
                                disabled=move || path_input.get().is_empty() || getting_path_info.get()
                                on:click=on_get_path_info
                            >
                                {move || if getting_path_info.get() { "Loading..." } else { "Get Info" }}
                            </button>
                        </div>

                        // Path info display
                        {move || path_info.get().map(|info| view! {
                            <div class="bg-muted  rounded-md p-4 space-y-2">
                                <div class="flex justify-between">
                                    <span class="text-sm text-muted-foreground">"NAR Size:"</span>
                                    <span class="font-mono text-sm">{format_bytes(info.nar_size)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-sm text-muted-foreground">"Closure Size:"</span>
                                    <span class="font-mono text-sm">{format_bytes(info.closure_size)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-sm text-muted-foreground">"References:"</span>
                                    <span class="font-mono text-sm">{info.references}</span>
                                </div>
                            </div>
                        })}
                    </div>
                </Card>
            </div>

            // Operation Output
            <Show when=move || active_operation.get().is_some()>
                <Card title="Operation Output".to_string()>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center space-x-2">
                                <span class="text-sm text-muted-foreground">
                                    {move || active_operation.get().unwrap_or_default()}
                                </span>
                                <Show when=move || operation_running.get()>
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                        <span class="w-2 h-2 mr-1.5 bg-blue-500 rounded-full animate-pulse"></span>
                                        "Running"
                                    </span>
                                </Show>
                            </div>
                            <button
                                class="text-sm text-muted-foreground hover:text-foreground"
                                on:click=clear_output
                            >
                                "Clear"
                            </button>
                        </div>
                        <div class="bg-zinc-900 rounded-lg p-4 font-mono text-sm max-h-80 overflow-auto">
                            <For
                                each=move || operation_output.get()
                                key=|l| l.clone()
                                let:output_line
                            >
                                {
                                    let text = output_line.clone();
                                    let is_error = text.starts_with("Error:") || text.contains("error:");
                                    let is_success = text.contains("completed") || text.contains("success");
                                    let class = if is_error {
                                        "whitespace-pre-wrap text-red-400"
                                    } else if is_success {
                                        "whitespace-pre-wrap text-green-400"
                                    } else {
                                        "whitespace-pre-wrap text-gray-300"
                                    };
                                    view! { <p class=class>{text}</p> }
                                }
                            </For>
                            <Show when=move || operation_running.get()>
                                <p class="animate-pulse text-green-400">"_"</p>
                            </Show>
                        </div>
                    </div>
                </Card>
            </Show>
        </div>
    }
}

/// Setup WebSocket message handlers
fn setup_ws_handlers(
    ws: web_sys::WebSocket,
    set_output: WriteSignal<Vec<String>>,
    set_running: WriteSignal<bool>,
) {
    // On message: handle streaming output
    let set_output_msg = set_output.clone();
    let set_running_msg = set_running.clone();
    let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
        if let Some(text) = e.data().as_string() {
            if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                match msg {
                    WsMessage::Started { command } => {
                        set_output_msg.update(|lines| {
                            lines.push(format!("$ {}", command));
                        });
                    }
                    WsMessage::Output(line) => {
                        set_output_msg.update(|lines| {
                            lines.push(line.content);
                        });
                    }
                    WsMessage::Completed { exit_code } => {
                        set_output_msg.update(|lines| {
                            if exit_code == 0 {
                                lines.push("Operation completed successfully!".to_string());
                            } else {
                                lines.push(format!("Operation finished with exit code: {}", exit_code));
                            }
                        });
                        set_running_msg.set(false);
                    }
                    WsMessage::Error { message } => {
                        set_output_msg.update(|lines| {
                            lines.push(format!("Error: {}", message));
                        });
                        set_running_msg.set(false);
                    }
                }
            }
        }
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // On error
    let set_output_err = set_output.clone();
    let set_running_err = set_running.clone();
    let onerror = Closure::wrap(Box::new(move |_: web_sys::Event| {
        set_output_err.update(|lines| {
            lines.push("WebSocket error occurred".to_string());
        });
        set_running_err.set(false);
    }) as Box<dyn FnMut(web_sys::Event)>);
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    // On close
    let set_running_close = set_running.clone();
    let onclose = Closure::wrap(Box::new(move |_: web_sys::CloseEvent| {
        set_running_close.set(false);
    }) as Box<dyn FnMut(web_sys::CloseEvent)>);
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}
