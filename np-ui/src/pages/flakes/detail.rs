use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlakeInput {
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    locked_rev: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlakeInfo {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    inputs: Vec<FlakeInput>,
    #[serde(default)]
    last_updated: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlakeOutputs {
    #[serde(default)]
    nixos_configurations: Vec<String>,
    #[serde(default)]
    packages: Vec<String>,
    #[serde(default)]
    dev_shells: Vec<String>,
    #[serde(default)]
    nixos_modules: Vec<String>,
}

async fn fetch_flake(id: String) -> Result<FlakeInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let url = format!("/api/flakes/{}", id);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
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

async fn fetch_outputs(id: String) -> Result<FlakeOutputs, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let url = format!("/api/flakes/{}/outputs", id);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    if !resp.ok() {
        return Ok(FlakeOutputs::default());
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json).unwrap_or_default()
}

async fn refresh_metadata(id: String) -> Result<FlakeInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("POST");

    let url = format!("/api/flakes/{}/refresh", id);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    if !resp.ok() {
        return Err(format!("Refresh failed: {}", resp.status()));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
}

async fn unregister_flake(id: String) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("DELETE");

    let url = format!("/api/flakes/{}", id);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Unregister failed: {}", resp.status()))
    }
}

async fn update_lock(id: String, input: Option<String>) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({ "input": input });

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let url = format!("/api/flakes/{}/lock/update", id);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
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
        return Err(format!("Update failed: {}", resp.status()));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let message = js_sys::Reflect::get(&json, &"message".into())
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "Update started".to_string());

    Ok(message)
}

impl Default for FlakeOutputs {
    fn default() -> Self {
        Self {
            nixos_configurations: vec![],
            packages: vec![],
            dev_shells: vec![],
            nixos_modules: vec![],
        }
    }
}

/// Flake detail page
#[component]
pub fn FlakeDetailPage() -> impl IntoView {
    let params = use_params_map();
    let flake_id = move || params.read().get("id").unwrap_or_default();

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (flake, set_flake) = signal(Option::<FlakeInfo>::None);
    let (outputs, set_outputs) = signal(FlakeOutputs::default());
    let (action_msg, set_action_msg) = signal(Option::<String>::None);
    let (refreshing, set_refreshing) = signal(false);
    let (unregistering, set_unregistering) = signal(false);

    // State for update modal
    let (show_update_modal, set_show_update_modal) = signal(false);
    let (selected_input, set_selected_input) = signal(Option::<String>::None);
    let (updating, set_updating) = signal(false);
    let (update_output, set_update_output) = signal(Vec::<String>::new());

    // Fetch flake on mount
    let id = flake_id();
    if !id.is_empty() {
        let id_clone = id.clone();
        let id_for_outputs = id.clone();
        leptos::task::spawn_local(async move {
            match fetch_flake(id_clone).await {
                Ok(f) => {
                    set_flake.set(Some(f));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
        leptos::task::spawn_local(async move {
            if let Ok(o) = fetch_outputs(id_for_outputs).await {
                set_outputs.set(o);
            }
        });
    }

    let on_refresh = move |_| {
        let id = flake_id();
        if id.is_empty() {
            return;
        }
        set_refreshing.set(true);
        set_action_msg.set(Some("Refreshing metadata...".to_string()));
        leptos::task::spawn_local(async move {
            match refresh_metadata(id).await {
                Ok(f) => {
                    set_flake.set(Some(f));
                    set_action_msg.set(Some("Metadata refreshed successfully".to_string()));
                    set_refreshing.set(false);
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Refresh failed: {}", e)));
                    set_refreshing.set(false);
                }
            }
        });
    };

    let on_unregister = move |_| {
        let id = flake_id();
        if id.is_empty() {
            return;
        }

        let window = web_sys::window().unwrap();
        let confirmed = window.confirm_with_message("Are you sure you want to unregister this flake?").unwrap_or(false);
        if !confirmed {
            return;
        }

        set_unregistering.set(true);
        leptos::task::spawn_local(async move {
            match unregister_flake(id).await {
                Ok(()) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/flakes");
                    }
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Unregister failed: {}", e)));
                    set_unregistering.set(false);
                }
            }
        });
    };

    let on_update_all = move |_| {
        set_selected_input.set(None);
        set_update_output.set(vec!["Ready to update all inputs...".to_string()]);
        set_show_update_modal.set(true);
    };

    let do_update = move |_| {
        let id = flake_id();
        if id.is_empty() {
            return;
        }
        let input = selected_input.get();
        set_updating.set(true);
        set_update_output.update(|lines| {
            lines.push(format!("$ nix flake update{}", input.as_ref().map(|i| format!(" {}", i)).unwrap_or_default()));
        });
        leptos::task::spawn_local(async move {
            match update_lock(id, input).await {
                Ok(msg) => {
                    set_update_output.update(|lines| {
                        lines.push(msg);
                        lines.push("Update completed!".to_string());
                    });
                    set_updating.set(false);
                }
                Err(e) => {
                    set_update_output.update(|lines| {
                        lines.push(format!("Error: {}", e));
                    });
                    set_updating.set(false);
                }
            }
        });
    };

    let close_modal = move |_| {
        set_show_update_modal.set(false);
        set_selected_input.set(None);
        set_update_output.set(vec![]);
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href="/flakes"
                        attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        {"\u{2190} Back"}
                    </A>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        "Flake Details"
                    </h1>
                </div>
                <div class="flex space-x-3">
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md disabled:opacity-50"
                        on:click=on_update_all
                        disabled=move || loading.get()
                    >
                        "Update All Inputs"
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md disabled:opacity-50"
                        on:click=on_refresh
                        disabled=move || refreshing.get() || loading.get()
                    >
                        {move || if refreshing.get() { "Refreshing..." } else { "Refresh Metadata" }}
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-md disabled:opacity-50"
                        on:click=on_unregister
                        disabled=move || unregistering.get() || loading.get()
                    >
                        {move || if unregistering.get() { "Unregistering..." } else { "Unregister" }}
                    </button>
                </div>
            </div>

            {move || error.get().map(|e| view! {
                <div class="p-3 bg-red-100 border border-red-400 text-red-700 rounded">
                    {e}
                </div>
            })}

            {move || action_msg.get().map(|msg| view! {
                <div class="p-3 bg-blue-100 border border-blue-400 text-blue-700 rounded">
                    {msg}
                </div>
            })}

            <Show when=move || loading.get()>
                <div class="text-center py-12">
                    <p class="text-gray-500">"Loading flake details..."</p>
                </div>
            </Show>

            <Show when=move || !loading.get() && flake.get().is_some()>
                {move || {
                    let f = flake.get().unwrap();
                    let f_clone = f.clone();
                    let inputs = f.inputs.clone();
                    view! {
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                            // Flake Info
                            <Card title="Information".to_string()>
                                <dl class="space-y-4">
                                    <div>
                                        <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Name"</dt>
                                        <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                            {f.name.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Path"</dt>
                                        <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                            {f.path.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Description"</dt>
                                        <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                            {f.description.clone().unwrap_or_else(|| "No description".to_string())}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Last Updated"</dt>
                                        <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                            {f.last_updated.clone().unwrap_or_else(|| "Unknown".to_string())}
                                        </dd>
                                    </div>
                                </dl>
                            </Card>

                            // Inputs
                            <Card title="Inputs".to_string() class="lg:col-span-2".to_string()>
                                <div class="space-y-4">
                                    {if inputs.is_empty() {
                                        view! {
                                            <p class="text-gray-500 dark:text-gray-400">"No inputs found"</p>
                                        }.into_any()
                                    } else {
                                        inputs.into_iter().map(|input| {
                                            let name = input.name.clone();
                                            let name_for_update = name.clone();
                                            view! {
                                                <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-center space-x-2">
                                                            <span class="font-medium text-gray-900 dark:text-gray-100">{name.clone()}</span>
                                                        </div>
                                                        <p class="text-sm text-gray-500 dark:text-gray-400 font-mono truncate">
                                                            {input.url.clone()}
                                                        </p>
                                                        <p class="text-xs text-gray-400 dark:text-gray-500 font-mono">
                                                            "Locked: " {input.locked_rev.clone().unwrap_or_else(|| "N/A".to_string())}
                                                        </p>
                                                    </div>
                                                    <div class="flex space-x-2 ml-4">
                                                        <button
                                                            class="px-3 py-1 text-sm font-medium text-indigo-600 hover:text-indigo-800 dark:text-indigo-400"
                                                            on:click=move |_| {
                                                                set_selected_input.set(Some(name_for_update.clone()));
                                                                set_update_output.set(vec![format!("Ready to update {}...", name_for_update)]);
                                                                set_show_update_modal.set(true);
                                                            }
                                                        >
                                                            "Update"
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>().into_any()
                                    }}
                                </div>
                            </Card>
                        </div>

                        // Outputs section
                        <Card title="Outputs".to_string()>
                            {move || {
                                let o = outputs.get();
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                                        <OutputCategory
                                            name="NixOS Configurations"
                                            items=o.nixos_configurations.clone()
                                        />
                                        <OutputCategory
                                            name="Packages"
                                            items=o.packages.clone()
                                        />
                                        <OutputCategory
                                            name="Dev Shells"
                                            items=o.dev_shells.clone()
                                        />
                                        <OutputCategory
                                            name="NixOS Modules"
                                            items=o.nixos_modules.clone()
                                        />
                                    </div>
                                }
                            }}
                        </Card>

                        // Quick Actions
                        <Card title="Quick Actions".to_string()>
                            <div class="flex flex-wrap gap-4">
                                <A
                                    href=move || format!("/deploy?flake={}", flake_id())
                                    attr:class="inline-flex items-center px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium rounded-md transition-colors"
                                >
                                    "Deploy Configuration"
                                </A>
                                <A
                                    href=move || format!("/install?flake={}", flake_id())
                                    attr:class="inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md transition-colors"
                                >
                                    "Install to Machine"
                                </A>
                                <A
                                    href=move || format!("/nix/build?flake={}", f_clone.path)
                                    attr:class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md transition-colors"
                                >
                                    "Build"
                                </A>
                            </div>
                        </Card>

                        <p class="text-sm text-gray-500">
                            "Flake ID: " {flake_id}
                        </p>
                    }
                }}
            </Show>

            // Update Modal
            <Show when=move || show_update_modal.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center">
                    // Backdrop
                    <div
                        class="absolute inset-0 bg-black bg-opacity-50"
                        on:click=close_modal
                    />

                    // Modal content
                    <div class="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 p-6">
                        <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100 mb-4">
                            {move || match selected_input.get() {
                                Some(name) => format!("Update '{}'", name),
                                None => "Update All Inputs".to_string(),
                            }}
                        </h2>

                        <div class="space-y-4">
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                "This will run 'nix flake update' to update the lock file."
                            </p>

                            // Progress area
                            <div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400 h-32 overflow-auto">
                                {move || update_output.get().into_iter().map(|line| view! {
                                    <div>{line}</div>
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>

                        <div class="flex justify-end space-x-3 mt-6">
                            <button
                                class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md"
                                on:click=close_modal
                            >
                                "Close"
                            </button>
                            <button
                                class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md disabled:opacity-50"
                                on:click=do_update
                                disabled=move || updating.get()
                            >
                                {move || if updating.get() { "Updating..." } else { "Update" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Component for displaying output category
#[component]
fn OutputCategory(
    #[prop(into)] name: String,
    items: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <h4 class="font-medium text-gray-900 dark:text-gray-100 mb-2">{name}</h4>
            <ul class="text-sm text-gray-500 dark:text-gray-400 space-y-1">
                {if items.is_empty() {
                    view! { <li class="text-gray-400 italic">"None"</li> }.into_any()
                } else {
                    items.into_iter().map(|item| view! {
                        <li class="font-mono">{item}</li>
                    }).collect::<Vec<_>>().into_any()
                }}
            </ul>
        </div>
    }
}
