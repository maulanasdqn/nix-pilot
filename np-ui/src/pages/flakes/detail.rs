use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use leptos::wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::api::check_response_status;
use crate::components::common::Card;
use crate::components::ui::spinner::Spinner;

use std::collections::HashMap;

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

/// Request to send over WebSocket
#[derive(Clone, Debug, Serialize)]
struct UpdateLockWsRequest {
    input: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FlakeInputRef {
    #[serde(rename = "type", default)]
    input_type: String,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(rename = "ref", default)]
    git_ref: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlakeInputLocked {
    #[serde(default)]
    rev: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlakeInput {
    #[serde(default)]
    name: String,
    #[serde(default)]
    original: FlakeInputRef,
    #[serde(default)]
    locked: Option<FlakeInputLocked>,
    #[serde(default)]
    follows: Option<Vec<String>>,
}

impl FlakeInputRef {
    fn to_display_url(&self) -> String {
        match self.input_type.as_str() {
            "github" => format!(
                "github:{}/{}{}",
                self.owner.as_deref().unwrap_or(""),
                self.repo.as_deref().unwrap_or(""),
                self.git_ref.as_ref().map(|r| format!("/{}", r)).unwrap_or_default()
            ),
            "follows" => "follows".to_string(),
            _ => self.url.clone().unwrap_or_else(|| self.input_type.clone()),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FlakeMetadata {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    inputs: HashMap<String, FlakeInput>,
    #[serde(default)]
    last_modified: Option<String>,
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
    metadata: Option<FlakeMetadata>,
    #[serde(default)]
    updated_at: Option<String>,
}

impl FlakeInfo {
    fn get_inputs(&self) -> Vec<FlakeInput> {
        self.metadata
            .as_ref()
            .map(|m| m.inputs.values().cloned().collect())
            .unwrap_or_default()
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
            .or_else(|| self.metadata.as_ref().and_then(|m| m.description.clone()))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FlakeOutputEntry {
    #[serde(rename = "type", default)]
    output_type: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FlakeOutputsRaw {
    #[serde(rename = "nixosConfigurations", default)]
    nixos_configurations: Option<HashMap<String, FlakeOutputEntry>>,
    #[serde(rename = "darwinConfigurations", default)]
    darwin_configurations: Option<HashMap<String, FlakeOutputEntry>>,
    #[serde(default)]
    packages: Option<HashMap<String, HashMap<String, FlakeOutputEntry>>>,
    #[serde(rename = "devShells", default)]
    dev_shells: Option<HashMap<String, HashMap<String, FlakeOutputEntry>>>,
    #[serde(rename = "nixosModules", default)]
    nixos_modules: Option<HashMap<String, FlakeOutputEntry>>,
}

#[derive(Clone, Debug, Default)]
struct FlakeOutputs {
    nixos_configurations: Vec<String>,
    darwin_configurations: Vec<String>,
    packages: Vec<String>,
    dev_shells: Vec<String>,
    nixos_modules: Vec<String>,
}

impl From<FlakeOutputsRaw> for FlakeOutputs {
    fn from(raw: FlakeOutputsRaw) -> Self {
        Self {
            nixos_configurations: raw.nixos_configurations
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default(),
            darwin_configurations: raw.darwin_configurations
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default(),
            packages: raw.packages
                .map(|m| {
                    m.values()
                        .flat_map(|inner| inner.keys().cloned())
                        .collect()
                })
                .unwrap_or_default(),
            dev_shells: raw.dev_shells
                .map(|m| {
                    m.values()
                        .flat_map(|inner| inner.keys().cloned())
                        .collect()
                })
                .unwrap_or_default(),
            nixos_modules: raw.nixos_modules
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default(),
        }
    }
}

async fn fetch_flake(id: String) -> Result<FlakeInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
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

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

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

    let opts = web_sys::RequestInit::new();
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

    // Handle 401 - logout and redirect
    if resp.status() == 401 {
        crate::api::logout_and_redirect();
        return Err("Session expired. Please login again.".to_string());
    }

    if !resp.ok() {
        return Ok(FlakeOutputs::default());
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    // Deserialize to raw format first, then convert
    let raw: FlakeOutputsRaw = serde_wasm_bindgen::from_value(json).unwrap_or_default();
    Ok(FlakeOutputs::from(raw))
}

async fn refresh_metadata(id: String) -> Result<FlakeInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
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

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

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

    let opts = web_sys::RequestInit::new();
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

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

    Ok(())
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

        // Build WebSocket URL with auth token
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().ok().flatten();
        let token = storage.and_then(|s| s.get_item("np_token").ok().flatten()).unwrap_or_default();

        let location = window.location();
        let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let host = location.host().unwrap_or_else(|_| "localhost:8080".to_string());
        let ws_url = format!("{}//{}/api/ws/flakes/{}/lock/update?token={}", ws_protocol, host, id, token);

        // Create WebSocket
        let ws = match web_sys::WebSocket::new(&ws_url) {
            Ok(ws) => ws,
            Err(_) => {
                set_update_output.update(|lines| {
                    lines.push("Error: Failed to create WebSocket connection".to_string());
                });
                set_updating.set(false);
                return;
            }
        };

        // Clone for onopen closure
        let ws_clone = ws.clone();
        let input_clone = input.clone();

        // On open: send the update request
        let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let request = UpdateLockWsRequest { input: input_clone.clone() };
            let msg = serde_json::to_string(&request).unwrap_or_default();
            let _ = ws_clone.send_with_str(&msg);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // On message: handle streaming output
        let set_update_output_msg = set_update_output.clone();
        let set_updating_msg = set_updating.clone();
        let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Some(text) = e.data().as_string() {
                if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                    match msg {
                        WsMessage::Started { command } => {
                            set_update_output_msg.update(|lines| {
                                lines.push(format!("Starting: {}", command));
                            });
                        }
                        WsMessage::Output(line) => {
                            set_update_output_msg.update(|lines| {
                                let prefix = if line.is_stderr { "[stderr] " } else { "" };
                                lines.push(format!("{}{}", prefix, line.content));
                            });
                        }
                        WsMessage::Completed { exit_code } => {
                            set_update_output_msg.update(|lines| {
                                if exit_code == 0 {
                                    lines.push("Update completed successfully!".to_string());
                                } else {
                                    lines.push(format!("Update finished with exit code: {}", exit_code));
                                }
                            });
                            set_updating_msg.set(false);
                        }
                        WsMessage::Error { message } => {
                            set_update_output_msg.update(|lines| {
                                lines.push(format!("Error: {}", message));
                            });
                            set_updating_msg.set(false);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // On error
        let set_update_output_err = set_update_output.clone();
        let set_updating_err = set_updating.clone();
        let onerror = Closure::wrap(Box::new(move |_: web_sys::Event| {
            set_update_output_err.update(|lines| {
                lines.push("WebSocket error occurred".to_string());
            });
            set_updating_err.set(false);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        // On close
        let set_updating_close = set_updating.clone();
        let onclose = Closure::wrap(Box::new(move |_: web_sys::CloseEvent| {
            set_updating_close.set(false);
        }) as Box<dyn FnMut(web_sys::CloseEvent)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
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
                        attr:class="text-muted-foreground hover:text-foreground :text-gray-200"
                    >
                        {"\u{2190} Back"}
                    </A>
                    <h1 class="text-2xl font-bold text-foreground ">
                        "Flake Details"
                    </h1>
                </div>
                <div class="flex space-x-3">
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md disabled:opacity-50 inline-flex items-center gap-2"
                        on:click=on_update_all
                        disabled=move || loading.get()
                    >
                        "Update All Inputs"
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md disabled:opacity-50 inline-flex items-center gap-2"
                        on:click=on_refresh
                        disabled=move || refreshing.get() || loading.get()
                    >
                        <Show when=move || refreshing.get() fallback=|| "Refresh Metadata">
                            <Spinner class="size-4" />" Refreshing..."
                        </Show>
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-md disabled:opacity-50 inline-flex items-center gap-2"
                        on:click=on_unregister
                        disabled=move || unregistering.get() || loading.get()
                    >
                        <Show when=move || unregistering.get() fallback=|| "Unregister">
                            <Spinner class="size-4" />" Unregistering..."
                        </Show>
                    </button>
                </div>
            </div>

            {move || error.get().map(|e| view! {
                <div class="p-3 bg-red-500/20 border border-red-500/30 text-red-700 rounded">
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
                    <p class="text-muted-foreground">"Loading flake details..."</p>
                </div>
            </Show>

            <Show when=move || !loading.get() && flake.get().is_some()>
                {move || {
                    let f = flake.get().unwrap();
                    let f_clone = f.clone();
                    let inputs = f.get_inputs();
                    view! {
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                            // Flake Info
                            <Card title="Information".to_string()>
                                <dl class="space-y-4">
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Name"</dt>
                                        <dd class="mt-1 text-sm text-foreground ">
                                            {f.name.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Path"</dt>
                                        <dd class="mt-1 text-sm text-foreground  font-mono">
                                            {f.path.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Description"</dt>
                                        <dd class="mt-1 text-sm text-foreground ">
                                            {f.get_description().unwrap_or_else(|| "No description".to_string())}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Last Updated"</dt>
                                        <dd class="mt-1 text-sm text-foreground ">
                                            {f.updated_at.clone().unwrap_or_else(|| "Unknown".to_string())}
                                        </dd>
                                    </div>
                                </dl>
                            </Card>

                            // Inputs
                            <Card title="Inputs".to_string() class="lg:col-span-2".to_string()>
                                <div class="space-y-4">
                                    {if inputs.is_empty() {
                                        view! {
                                            <p class="text-muted-foreground ">"No inputs found. Click \"Refresh Metadata\" to load inputs."</p>
                                        }.into_any()
                                    } else {
                                        inputs.into_iter().map(|input| {
                                            let name = input.name.clone();
                                            let name_for_update = name.clone();
                                            let url = input.original.to_display_url();
                                            let locked_rev = input.locked.as_ref().and_then(|l| l.rev.clone());
                                            let follows = input.follows.clone();
                                            view! {
                                                <div class="flex items-center justify-between p-3 bg-muted  rounded-lg">
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-center space-x-2">
                                                            <span class="font-medium text-foreground ">{name.clone()}</span>
                                                            {follows.map(|f| view! {
                                                                <span class="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                                                                    "follows: " {f.join(".")}
                                                                </span>
                                                            })}
                                                        </div>
                                                        <p class="text-sm text-muted-foreground  font-mono truncate">
                                                            {url}
                                                        </p>
                                                        <p class="text-xs text-muted-foreground  font-mono">
                                                            "Locked: " {locked_rev.unwrap_or_else(|| "N/A".to_string())}
                                                        </p>
                                                    </div>
                                                    <div class="flex space-x-2 ml-4">
                                                        <button
                                                            class="px-3 py-1 text-sm font-medium text-primary hover:text-primary/80"
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
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                                        <OutputCategory
                                            name="NixOS Configurations"
                                            items=o.nixos_configurations.clone()
                                        />
                                        <OutputCategory
                                            name="Darwin Configurations"
                                            items=o.darwin_configurations.clone()
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
                                    attr:class="inline-flex items-center px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 text-sm font-medium rounded-md transition-colors"
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

                        <p class="text-sm text-muted-foreground">
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
                    <div class="relative bg-background rounded-lg shadow-xl max-w-lg w-full mx-4 p-6">
                        <h2 class="text-lg font-bold text-foreground  mb-4">
                            {move || match selected_input.get() {
                                Some(name) => format!("Update '{}'", name),
                                None => "Update All Inputs".to_string(),
                            }}
                        </h2>

                        <div class="space-y-4">
                            <p class="text-sm text-muted-foreground ">
                                "This will run 'nix flake update' to update the lock file."
                            </p>

                            // Progress area
                            <div class="bg-background rounded-lg p-4 font-mono text-sm text-green-400 h-32 overflow-auto">
                                {move || update_output.get().into_iter().map(|line| view! {
                                    <div>{line}</div>
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>

                        <div class="flex justify-end space-x-3 mt-6">
                            <button
                                class="px-4 py-2 text-sm font-medium text-foreground  hover:bg-muted  rounded-md"
                                on:click=close_modal
                            >
                                "Close"
                            </button>
                            <button
                                class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 rounded-md disabled:opacity-50 inline-flex items-center gap-2"
                                on:click=do_update
                                disabled=move || updating.get()
                            >
                                <Show when=move || updating.get() fallback=|| "Update">
                                    <Spinner class="size-4" />" Updating..."
                                </Show>
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
        <div class="p-4 bg-muted  rounded-lg">
            <h4 class="font-medium text-foreground  mb-2">{name}</h4>
            <ul class="text-sm text-muted-foreground  space-y-1">
                {if items.is_empty() {
                    view! { <li class="text-muted-foreground italic">"None"</li> }.into_any()
                } else {
                    items.into_iter().map(|item| view! {
                        <li class="font-mono">{item}</li>
                    }).collect::<Vec<_>>().into_any()
                }}
            </ul>
        </div>
    }
}
