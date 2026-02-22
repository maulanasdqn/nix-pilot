use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::api::check_response_status;
use crate::components::common::Card;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ServiceDetail {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    load_state: String,
    #[serde(default)]
    active_state: String,
    #[serde(default)]
    sub_state: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    main_pid: Option<u32>,
    #[serde(default)]
    memory_bytes: Option<u64>,
    #[serde(default)]
    started_at: Option<String>,
    #[serde(default)]
    unit_file_path: Option<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    wanted_by: Vec<String>,
    #[serde(default)]
    after: Vec<String>,
    #[serde(default)]
    before: Vec<String>,
    #[serde(default)]
    recent_logs: Vec<String>,
}

async fn fetch_service_detail(service: &str) -> Result<ServiceDetail, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let url = format!("/api/system/services/{}", service);
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

async fn service_action(service: &str, action: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({ "action": action });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let url = format!("/api/system/services/{}/action", service);
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

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

    Ok(())
}

/// Service detail page - local system
#[component]
pub fn ServiceDetailPage() -> impl IntoView {
    let params = use_params_map();
    let service_name = move || params.read().get("service").unwrap_or_default();

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (service, set_service) = signal(ServiceDetail::default());
    let (action_loading, set_action_loading) = signal(false);
    let (action_msg, set_action_msg) = signal(Option::<String>::None);

    // Fetch service detail on mount
    let svc = service_name();
    if !svc.is_empty() {
        leptos::task::spawn_local(async move {
            match fetch_service_detail(&svc).await {
                Ok(s) => {
                    set_service.set(s);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    }

    let format_bytes = move |bytes: u64| {
        if bytes >= 1_000_000_000 {
            format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            format!("{:.2} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.2} KB", bytes as f64 / 1_000.0)
        } else {
            format!("{} B", bytes)
        }
    };

    let perform_action = move |action: &'static str| {
        let svc = service_name();
        if svc.is_empty() {
            return;
        }
        set_action_loading.set(true);
        set_action_msg.set(Some(format!("{}ing {}...", action, svc)));

        leptos::task::spawn_local(async move {
            match service_action(&svc, action).await {
                Ok(()) => {
                    set_action_msg.set(Some(format!("{} successful", action)));
                    // Refresh service detail
                    if let Ok(s) = fetch_service_detail(&svc).await {
                        set_service.set(s);
                    }
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Failed: {}", e)));
                }
            }
            set_action_loading.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href="/services"
                        attr:class="text-muted-foreground hover:text-foreground"
                    >
                        "\u{2190} Back to Services"
                    </A>
                    <div>
                        <h1 class="text-2xl font-bold text-foreground font-mono">
                            {service_name}
                        </h1>
                        <p class="text-sm text-muted-foreground">
                            {move || service.get().description.unwrap_or_default()}
                        </p>
                    </div>
                </div>
                <div class="flex items-center space-x-3">
                    {move || {
                        let s = service.get();
                        let status_class = match s.active_state.as_str() {
                            "active" => "bg-green-500/20 text-green-400",
                            "inactive" => "bg-muted text-foreground",
                            "failed" => "bg-red-500/20 text-red-400",
                            _ => "bg-muted text-muted-foreground",
                        };
                        view! {
                            <span class=format!(
                                "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {}",
                                status_class
                            )>
                                {s.active_state.clone()} " (" {s.sub_state.clone()} ")"
                            </span>
                        }
                    }}
                </div>
            </div>

            // Error display
            {move || error.get().map(|e| view! {
                <div class="bg-red-500/10 border border-red-500/20 rounded-md p-4">
                    <p class="text-sm text-red-400">{e}</p>
                </div>
            })}

            // Action message
            {move || action_msg.get().map(|msg| view! {
                <div class="bg-blue-500/10 border border-blue-500/20 rounded-md p-4">
                    <p class="text-sm text-blue-400">{msg}</p>
                </div>
            })}

            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                </div>
            </Show>

            <Show when=move || !loading.get()>
                // Quick Actions
                <Card title="Actions".to_string()>
                    {move || {
                        let s = service.get();
                        let is_active = s.active_state == "active";
                        let is_enabled = s.enabled;

                        view! {
                            <div class="flex flex-wrap gap-3">
                                <button
                                    class="inline-flex items-center px-4 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                                    disabled=move || action_loading.get() || is_active
                                    on:click=move |_| perform_action("start")
                                >
                                    "Start"
                                </button>
                                <button
                                    class="inline-flex items-center px-4 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                                    disabled=move || action_loading.get() || !is_active
                                    on:click=move |_| perform_action("stop")
                                >
                                    "Stop"
                                </button>
                                <button
                                    class="inline-flex items-center px-4 py-2 bg-yellow-600 hover:bg-yellow-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                                    disabled=move || action_loading.get() || !is_active
                                    on:click=move |_| perform_action("restart")
                                >
                                    "Restart"
                                </button>
                                <button
                                    class="inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                                    disabled=move || action_loading.get() || !is_active
                                    on:click=move |_| perform_action("reload")
                                >
                                    "Reload"
                                </button>
                                <div class="border-l border-border mx-2"></div>
                                <button
                                    class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 disabled:opacity-50 text-primary-foreground text-sm font-medium rounded-md transition-colors"
                                    disabled=move || action_loading.get() || is_enabled
                                    on:click=move |_| perform_action("enable")
                                >
                                    "Enable"
                                </button>
                                <button
                                    class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                                    disabled=move || action_loading.get() || !is_enabled
                                    on:click=move |_| perform_action("disable")
                                >
                                    "Disable"
                                </button>
                            </div>
                        }
                    }}
                </Card>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    // Service Info
                    <Card title="Service Info".to_string()>
                        {move || {
                            let s = service.get();
                            view! {
                                <dl class="space-y-4">
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground">"State"</dt>
                                        <dd class="mt-1 text-sm text-foreground">
                                            {s.active_state.clone()} " (" {s.sub_state.clone()} ")"
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-sm font-medium text-muted-foreground">"Enabled"</dt>
                                        <dd class="mt-1 text-sm text-foreground">
                                            {if s.enabled { "Yes" } else { "No" }}
                                        </dd>
                                    </div>
                                    {s.main_pid.map(|pid| view! {
                                        <div>
                                            <dt class="text-sm font-medium text-muted-foreground">"Main PID"</dt>
                                            <dd class="mt-1 text-sm text-foreground font-mono">
                                                {pid}
                                            </dd>
                                        </div>
                                    })}
                                    {s.memory_bytes.map(|bytes| view! {
                                        <div>
                                            <dt class="text-sm font-medium text-muted-foreground">"Memory"</dt>
                                            <dd class="mt-1 text-sm text-foreground">
                                                {format_bytes(bytes)}
                                            </dd>
                                        </div>
                                    })}
                                    {s.started_at.clone().map(|time| view! {
                                        <div>
                                            <dt class="text-sm font-medium text-muted-foreground">"Started At"</dt>
                                            <dd class="mt-1 text-sm text-foreground">
                                                {time}
                                            </dd>
                                        </div>
                                    })}
                                    {s.unit_file_path.clone().map(|path| view! {
                                        <div>
                                            <dt class="text-sm font-medium text-muted-foreground">"Unit File"</dt>
                                            <dd class="mt-1 text-sm text-foreground font-mono break-all">
                                                {path}
                                            </dd>
                                        </div>
                                    })}
                                </dl>
                            }
                        }}
                    </Card>

                    // Dependencies
                    <Card title="Dependencies".to_string()>
                        {move || {
                            let s = service.get();
                            let requires = s.requires.clone();
                            let wanted_by = s.wanted_by.clone();
                            let after = s.after.clone();
                            let before = s.before.clone();
                            let has_deps = !requires.is_empty() || !wanted_by.is_empty() || !after.is_empty() || !before.is_empty();

                            view! {
                                <div class="space-y-4">
                                    {if !requires.is_empty() {
                                        Some(view! {
                                            <div>
                                                <h4 class="text-sm font-medium text-muted-foreground mb-2">"Requires"</h4>
                                                <div class="flex flex-wrap gap-2">
                                                    {requires.iter().map(|dep| view! {
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-blue-500/20 text-blue-400 font-mono">
                                                            {dep.clone()}
                                                        </span>
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }}
                                    {if !wanted_by.is_empty() {
                                        Some(view! {
                                            <div>
                                                <h4 class="text-sm font-medium text-muted-foreground mb-2">"Wanted By"</h4>
                                                <div class="flex flex-wrap gap-2">
                                                    {wanted_by.iter().map(|dep| view! {
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-green-500/20 text-green-400 font-mono">
                                                            {dep.clone()}
                                                        </span>
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }}
                                    {if !after.is_empty() {
                                        Some(view! {
                                            <div>
                                                <h4 class="text-sm font-medium text-muted-foreground mb-2">"After"</h4>
                                                <div class="flex flex-wrap gap-2">
                                                    {after.iter().map(|dep| view! {
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-muted text-foreground font-mono">
                                                            {dep.clone()}
                                                        </span>
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }}
                                    {if !before.is_empty() {
                                        Some(view! {
                                            <div>
                                                <h4 class="text-sm font-medium text-muted-foreground mb-2">"Before"</h4>
                                                <div class="flex flex-wrap gap-2">
                                                    {before.iter().map(|dep| view! {
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-muted text-foreground font-mono">
                                                            {dep.clone()}
                                                        </span>
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }}
                                    {if !has_deps {
                                        Some(view! {
                                            <p class="text-sm text-muted-foreground">"No dependency information available"</p>
                                        })
                                    } else {
                                        None
                                    }}
                                </div>
                            }
                        }}
                    </Card>
                </div>

                // Recent Logs
                <Card title="Recent Logs".to_string()>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <p class="text-sm text-muted-foreground">
                                "Last " {move || service.get().recent_logs.len()} " log entries"
                            </p>
                            <A
                                href=move || format!("/services/{}/logs", service_name())
                                attr:class="text-sm text-primary hover:text-primary/80"
                            >
                                "View Full Logs \u{2192}"
                            </A>
                        </div>
                        <div class="bg-muted rounded-lg p-4 font-mono text-sm max-h-64 overflow-auto">
                            {move || {
                                let logs = service.get().recent_logs;
                                if logs.is_empty() {
                                    view! {
                                        <p class="text-muted-foreground">"No recent logs available"</p>
                                    }.into_any()
                                } else {
                                    logs.into_iter().map(|line| {
                                        let line_class = if line.contains("error") || line.contains("Error") || line.contains("ERROR") {
                                            "text-red-400"
                                        } else if line.contains("warn") || line.contains("Warn") || line.contains("WARNING") {
                                            "text-yellow-400"
                                        } else {
                                            "text-green-400"
                                        };
                                        view! {
                                            <p class=format!("whitespace-pre-wrap break-all {}", line_class)>{line}</p>
                                        }
                                    }).collect::<Vec<_>>().into_any()
                                }
                            }}
                        </div>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
