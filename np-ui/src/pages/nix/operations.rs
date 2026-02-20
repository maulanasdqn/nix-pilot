use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::components::common::Card;

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

    let mut opts = web_sys::RequestInit::new();
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

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

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

    let mut opts = web_sys::RequestInit::new();
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

    if !resp.ok() {
        return Err(format!("Search failed: {}", resp.status()));
    }

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

    let mut opts = web_sys::RequestInit::new();
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

    if !resp.ok() {
        return Err(format!("Path info failed: {}", resp.status()));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
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
    let (active_operation, set_active_operation) = signal(Option::<&'static str>::None);
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

    let start_operation = move |op: &'static str| {
        set_active_operation.set(Some(op));
        set_operation_output.set(vec![format!("Starting {}...", op)]);
        set_operation_running.set(true);
        set_operation_output.update(|lines| {
            lines.push("Note: This operation requires WebSocket connection.".to_string());
            lines.push("Please use the nix-pilot CLI for full streaming support.".to_string());
        });
        set_operation_running.set(false);
    };

    let clear_output = move |_| {
        set_operation_output.set(Vec::new());
        set_active_operation.set(None);
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Nix Operations"
                </h1>
            </div>

            {move || error_msg.get().map(|e| view! {
                <div class="p-3 bg-red-100 border border-red-400 text-red-700 rounded">
                    {e}
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Store Info
                <Card title="Store Info".to_string()>
                    <dl class="space-y-3">
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Store URL"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                {store_url}
                            </dd>
                        </div>
                        {move || store_version.get().map(|v| view! {
                            <div>
                                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Nix Version"</dt>
                                <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
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
                                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
                                prop:value=search_query
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
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
                                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
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
                                    <p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
                                        "Enter a search term to find packages"
                                    </p>
                                }
                            >
                                <ul class="divide-y divide-gray-200 dark:divide-gray-700">
                                    <For
                                        each=move || search_results.get()
                                        key=|r| r.attr_path.clone()
                                        let:result
                                    >
                                        <li class="py-2">
                                            <div class="font-mono text-sm text-indigo-600 dark:text-indigo-400">
                                                {result.attr_path.clone()}
                                            </div>
                                            {result.description.clone().map(|d| view! {
                                                <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
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
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Remove unused store paths to free disk space."
                        </p>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Delete older than"
                            </label>
                            <select
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
                                prop:value=gc_older_than
                                on:change=move |ev| set_gc_older_than.set(event_target_value(&ev))
                            >
                                <option value="">"All unused"</option>
                                <option value="1d">"1 day"</option>
                                <option value="7d">"7 days"</option>
                                <option value="30d">"30 days"</option>
                                <option value="90d">"90 days"</option>
                            </select>
                        </div>
                        <button
                            class="w-full px-4 py-2 bg-red-600 hover:bg-red-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get()
                            on:click=move |_| start_operation("gc")
                        >
                            "Run Garbage Collection"
                        </button>
                    </div>
                </Card>

                // Store Optimise
                <Card title="Store Optimise".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Deduplicate files in the Nix store using hard links."
                        </p>
                        <button
                            class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get()
                            on:click=move |_| start_operation("optimise")
                        >
                            "Optimise Store"
                        </button>
                    </div>
                </Card>

                // Store Verify
                <Card title="Store Verify".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Verify the integrity of store paths."
                        </p>
                        <button
                            class="w-full px-4 py-2 bg-yellow-600 hover:bg-yellow-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get()
                            on:click=move |_| start_operation("verify")
                        >
                            "Verify Store"
                        </button>
                    </div>
                </Card>

                // Flake Check
                <Card title="Flake Check".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Run checks defined in a flake."
                        </p>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Flake reference"
                            </label>
                            <input
                                type="text"
                                placeholder=". or github:user/repo"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                prop:value=flake_ref
                                on:input=move |ev| set_flake_ref.set(event_target_value(&ev))
                            />
                        </div>
                        <button
                            class="w-full px-4 py-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                            disabled=move || operation_running.get() || flake_ref.get().is_empty()
                            on:click=move |_| start_operation("flake-check")
                        >
                            "Run Flake Check"
                        </button>
                    </div>
                </Card>

                // Path Info
                <Card title="Path Info".to_string() class="md:col-span-2".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Get information about a store path including size and references."
                        </p>
                        <div class="flex space-x-2">
                            <input
                                type="text"
                                placeholder="/nix/store/..."
                                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                prop:value=path_input
                                on:input=move |ev| set_path_input.set(event_target_value(&ev))
                            />
                            <button
                                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                                disabled=move || path_input.get().is_empty() || getting_path_info.get()
                                on:click=on_get_path_info
                            >
                                {move || if getting_path_info.get() { "Loading..." } else { "Get Info" }}
                            </button>
                        </div>

                        // Path info display
                        {move || path_info.get().map(|info| view! {
                            <div class="bg-gray-50 dark:bg-gray-800 rounded-md p-4 space-y-2">
                                <div class="flex justify-between">
                                    <span class="text-sm text-gray-500">"NAR Size:"</span>
                                    <span class="font-mono text-sm">{format_bytes(info.nar_size)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-sm text-gray-500">"Closure Size:"</span>
                                    <span class="font-mono text-sm">{format_bytes(info.closure_size)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-sm text-gray-500">"References:"</span>
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
                                <Show when=move || operation_running.get()>
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                        <span class="w-2 h-2 mr-1.5 bg-blue-500 rounded-full animate-pulse"></span>
                                        "Running"
                                    </span>
                                </Show>
                            </div>
                            <button
                                class="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                                on:click=clear_output
                            >
                                "Clear"
                            </button>
                        </div>
                        <div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400 max-h-80 overflow-auto">
                            <For
                                each=move || operation_output.get()
                                key=|l| l.clone()
                                let:line
                            >
                                <p class="whitespace-pre-wrap">{line}</p>
                            </For>
                            <Show when=move || operation_running.get()>
                                <p class="animate-pulse">"_"</p>
                            </Show>
                        </div>
                    </div>
                </Card>
            </Show>
        </div>
    }
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
