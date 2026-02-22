use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use leptos::wasm_bindgen::JsCast;

use crate::api::check_response_status;
use crate::components::common::Card;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct MachineInfo {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    username: String,
    #[serde(default)]
    auth_method: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    system_info: Option<String>,
    #[serde(default)]
    last_seen: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct MachineStatus {
    #[serde(default)]
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestResponse {
    #[serde(default)]
    machine: MachineStatus,
}

async fn fetch_machine(id: String) -> Result<MachineInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let url = format!("/api/machines/{}", id);
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

async fn delete_machine(id: String) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("DELETE");

    let url = format!("/api/machines/{}", id);
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

async fn test_machine_connection(id: String) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");

    let url = format!("/api/machines/{}/test", id);
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

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let response: TestResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(response.machine.status)
}

/// Machine detail page
#[component]
pub fn MachineDetailPage() -> impl IntoView {
    let params = use_params_map();
    let machine_id = move || params.read().get("id").unwrap_or_default();

    // Machine data state
    let (loading, set_loading) = signal(true);
    let (machine, set_machine) = signal(Option::<MachineInfo>::None);
    let (error, set_error) = signal(Option::<String>::None);

    // Action states
    let (deleting, set_deleting) = signal(false);
    let (testing, set_testing) = signal(false);
    let (status_msg, set_status_msg) = signal(Option::<String>::None);
    let (show_terminal_info, set_show_terminal_info) = signal(false);

    // Fetch machine data on mount
    let id = machine_id();
    if !id.is_empty() {
        leptos::task::spawn_local(async move {
            match fetch_machine(id).await {
                Ok(m) => {
                    set_machine.set(Some(m));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    }

    let on_delete = move |_| {
        let id = machine_id();
        if id.is_empty() {
            return;
        }

        // Confirm deletion
        let window = web_sys::window().unwrap();
        let confirmed = window.confirm_with_message("Are you sure you want to delete this machine?").unwrap_or(false);
        if !confirmed {
            return;
        }

        set_deleting.set(true);
        leptos::task::spawn_local(async move {
            match delete_machine(id).await {
                Ok(()) => {
                    // Redirect to machines list
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/machines");
                    }
                }
                Err(e) => {
                    set_status_msg.set(Some(format!("Delete failed: {}", e)));
                    set_deleting.set(false);
                }
            }
        });
    };

    let on_test = move |_| {
        let id = machine_id();
        if id.is_empty() {
            return;
        }

        set_testing.set(true);
        set_status_msg.set(Some("Testing connection...".to_string()));
        leptos::task::spawn_local(async move {
            match test_machine_connection(id).await {
                Ok(status) => {
                    set_status_msg.set(Some(format!("Status: {}", status)));
                    set_testing.set(false);
                }
                Err(e) => {
                    set_status_msg.set(Some(format!("Test failed: {}", e)));
                    set_testing.set(false);
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href="/machines"
                        attr:class="text-muted-foreground hover:text-foreground :text-gray-200"
                    >
                        {"\u{2190} Back"}
                    </A>
                    <h1 class="text-2xl font-bold text-foreground ">
                        "Machine Details"
                    </h1>
                </div>
                <div class="flex space-x-3">
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md disabled:opacity-50"
                        on:click=on_test
                        disabled=move || testing.get() || loading.get()
                    >
                        {move || if testing.get() { "Testing..." } else { "Test Connection" }}
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-md disabled:opacity-50"
                        on:click=on_delete
                        disabled=move || deleting.get() || loading.get()
                    >
                        {move || if deleting.get() { "Deleting..." } else { "Delete" }}
                    </button>
                </div>
            </div>

            {move || error.get().map(|e| view! {
                <div class="p-3 bg-red-500/20 border border-red-500/30 text-red-700 rounded">
                    {e}
                </div>
            })}

            {move || status_msg.get().map(|msg| view! {
                <div class="p-3 bg-blue-100 border border-blue-400 text-blue-700 rounded">
                    {msg}
                </div>
            })}

            <Show when=move || loading.get()>
                <div class="text-center py-12">
                    <p class="text-muted-foreground">"Loading machine details..."</p>
                </div>
            </Show>

            <Show when=move || !loading.get() && machine.get().is_some()>
                {move || {
                    let m = machine.get().unwrap();
                    view! {
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            // Connection Info
                            <Card title="Connection".to_string()>
                                <dl class="space-y-4">
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Name"</dt>
                                        <dd class="mt-1 text-sm text-foreground  font-medium">
                                            {m.name.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Status"</dt>
                                        <dd class="mt-1 text-sm text-foreground ">
                                            <span class=move || {
                                                let status = m.status.clone();
                                                let base = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium";
                                                if status == "online" || status == "connected" {
                                                    format!("{} bg-green-500/20 text-green-400", base)
                                                } else if status == "offline" || status == "disconnected" {
                                                    format!("{} bg-red-100 text-red-800", base)
                                                } else {
                                                    format!("{} bg-muted text-foreground", base)
                                                }
                                            }>
                                                {if m.status.is_empty() { "Unknown".to_string() } else { m.status.clone() }}
                                            </span>
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Host"</dt>
                                        <dd class="mt-1 text-sm text-foreground  font-mono">
                                            {m.host.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Port"</dt>
                                        <dd class="mt-1 text-sm text-foreground  font-mono">
                                            {m.port.to_string()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Username"</dt>
                                        <dd class="mt-1 text-sm text-foreground  font-mono">
                                            {m.username.clone()}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Auth Method"</dt>
                                        <dd class="mt-1 text-sm text-foreground ">
                                            {if m.auth_method.is_empty() { "SSH Agent".to_string() } else { m.auth_method.clone() }}
                                        </dd>
                                    </div>
                                </dl>
                            </Card>

                            // System Info
                            <Card title="System Info".to_string()>
                                <dl class="space-y-4">
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"System"</dt>
                                        <dd class="mt-1 text-sm text-foreground  font-mono">
                                            {m.system_info.clone().unwrap_or_else(|| "Not available - test connection first".to_string())}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground ">"Last Seen"</dt>
                                        <dd class="mt-1 text-sm text-foreground ">
                                            {m.last_seen.clone().unwrap_or_else(|| "Never".to_string())}
                                        </dd>
                                    </div>
                                </dl>
                            </Card>

                            // Quick Actions
                            <Card title="Quick Actions".to_string() class="lg:col-span-2".to_string()>
                                <div class="flex flex-wrap gap-4">
                                    <A
                                        href=move || format!("/services/{}", machine_id())
                                        attr:class="inline-flex items-center px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 text-sm font-medium rounded-md transition-colors"
                                    >
                                        "View Services"
                                    </A>
                                    <A
                                        href=move || format!("/deploy?machine={}", machine_id())
                                        attr:class="inline-flex items-center px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium rounded-md transition-colors"
                                    >
                                        "Deploy Configuration"
                                    </A>
                                    <button
                                        class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md transition-colors"
                                        on:click=move |_| set_show_terminal_info.set(!show_terminal_info.get())
                                    >
                                        {move || if show_terminal_info.get() { "Hide Terminal Info" } else { "Open Terminal" }}
                                    </button>
                                </div>
                            </Card>
                        </div>

                        // Terminal connection info (shown conditionally below)

                        <p class="text-sm text-muted-foreground">
                            "Machine ID: " {machine_id}
                        </p>
                    }
                }}
            </Show>

            <Show when=move || !loading.get() && machine.get().is_none() && error.get().is_none()>
                <div class="text-center py-12">
                    <p class="text-muted-foreground">"Machine not found"</p>
                </div>
            </Show>

            // Terminal connection info - outside of main closure to avoid ownership issues
            <Show when=move || show_terminal_info.get() && machine.get().is_some()>
                {move || {
                    let m = machine.get().unwrap();
                    let ssh_cmd = format!("ssh {}@{} -p {}", m.username, m.host, m.port);
                    view! {
                        <Card title="Terminal Connection".to_string()>
                            <div class="space-y-4">
                                <p class="text-sm text-muted-foreground ">
                                    "Use the following command to connect to this machine via SSH:"
                                </p>
                                <div class="bg-background rounded-lg p-4">
                                    <code class="text-green-400 font-mono text-sm">
                                        {ssh_cmd}
                                    </code>
                                </div>
                                <p class="text-xs text-muted-foreground ">
                                    "Web-based terminal coming soon in a future release."
                                </p>
                            </div>
                        </Card>
                    }
                }}
            </Show>
        </div>
    }
}
