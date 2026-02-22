use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use leptos::wasm_bindgen::JsCast;

use crate::api::check_response_status;
use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ServiceInfo {
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ServicesResponse {
    services: Vec<ServiceInfo>,
}

async fn fetch_services() -> Result<Vec<ServiceInfo>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/system/services", &opts)
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

    let data: ServicesResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(data.services)
}

async fn service_action(service: String, action: String) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({ "action": action });

    let mut opts = web_sys::RequestInit::new();
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

/// Service list page - local system services
#[component]
pub fn ServiceListPage() -> impl IntoView {
    let (filter, set_filter) = signal(String::new());
    let (show_only_failed, set_show_only_failed) = signal(false);
    let (show_only_active, set_show_only_active) = signal(false);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (services, set_services) = signal(Vec::<ServiceInfo>::new());
    let (action_msg, set_action_msg) = signal(Option::<String>::None);

    // Fetch services on mount
    leptos::task::spawn_local(async move {
        match fetch_services().await {
            Ok(s) => {
                set_services.set(s);
                set_loading.set(false);
            }
            Err(e) => {
                set_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });

    let refresh = move |_| {
        set_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match fetch_services().await {
                Ok(s) => {
                    set_services.set(s);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    let do_action = move |service: String, action: String| {
        set_action_msg.set(Some(format!("{}ing {}...", action, service)));
        leptos::task::spawn_local(async move {
            match service_action(service.clone(), action.clone()).await {
                Ok(()) => {
                    set_action_msg.set(Some(format!("{} {} successful", action, service)));
                    // Refresh services list
                    if let Ok(s) = fetch_services().await {
                        set_services.set(s);
                    }
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Failed to {} {}: {}", action, service, e)));
                }
            }
        });
    };

    let filtered_services = move || {
        let search = filter.get().to_lowercase();
        let only_failed = show_only_failed.get();
        let only_active = show_only_active.get();

        services.get()
            .into_iter()
            .filter(|s| {
                let matches_search = search.is_empty()
                    || s.name.to_lowercase().contains(&search)
                    || s.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search))
                        .unwrap_or(false);

                let is_failed = s.active_state == "failed";
                let is_active = s.active_state == "active";
                let matches_failed = !only_failed || is_failed;
                let matches_active = !only_active || is_active;

                matches_search && matches_failed && matches_active
            })
            .collect::<Vec<_>>()
    };

    let total_services = move || services.get().len();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-foreground">
                        "System Services"
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        "Manage systemd services on this machine"
                    </p>
                </div>
                <button
                    class="inline-flex items-center px-4 py-2 bg-secondary hover:bg-secondary/80 text-secondary-foreground text-sm font-medium rounded-md transition-colors disabled:opacity-50"
                    on:click=refresh
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Loading..." } else { "Refresh" }}
                </button>
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

            <Card>
                <div class="flex flex-col sm:flex-row gap-4">
                    <div class="flex-1">
                        <input
                            type="text"
                            placeholder="Filter services..."
                            class="w-full px-3 py-2 border border-border rounded-md bg-background text-foreground focus:ring-ring focus:border-ring"
                            on:input=move |ev| set_filter.set(event_target_value(&ev))
                            prop:value=move || filter.get()
                        />
                    </div>
                    <div class="flex items-center space-x-4">
                        <label class="flex items-center space-x-2 cursor-pointer">
                            <input
                                type="checkbox"
                                on:change=move |ev| set_show_only_failed.set(event_target_checked(&ev))
                                prop:checked=move || show_only_failed.get()
                            />
                            <span class="text-sm text-foreground">"Failed only"</span>
                        </label>
                        <label class="flex items-center space-x-2 cursor-pointer">
                            <input
                                type="checkbox"
                                on:change=move |ev| set_show_only_active.set(event_target_checked(&ev))
                                prop:checked=move || show_only_active.get()
                            />
                            <span class="text-sm text-foreground">"Active only"</span>
                        </label>
                    </div>
                </div>
            </Card>

            <div class="bg-card shadow rounded-lg overflow-hidden border border-border">
                <table class="min-w-full divide-y divide-border">
                    <thead class="bg-muted">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                                "Service"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                                "Status"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                                "Enabled"
                            </th>
                            <th class="px-6 py-3 text-right text-xs font-medium text-muted-foreground uppercase tracking-wider">
                                "Actions"
                            </th>
                        </tr>
                    </thead>
                    <tbody class="bg-card divide-y divide-border">
                        {move || {
                            filtered_services().into_iter().map(|service| {
                                let name = service.name.clone();
                                let detail_href = format!("/services/{}", name);
                                let logs_href = format!("/services/{}/logs", name);
                                let is_active = service.active_state == "active";
                                let is_failed = service.active_state == "failed";

                                let name_for_restart = name.clone();
                                let name_for_stop = name.clone();
                                let name_for_start = name.clone();

                                let do_action_restart = do_action.clone();
                                let do_action_stop = do_action.clone();
                                let do_action_start = do_action.clone();

                                view! {
                                    <tr class="hover:bg-muted/50">
                                        <td class="px-6 py-4">
                                            <div class="flex flex-col">
                                                <A
                                                    href=detail_href
                                                    attr:class="text-sm font-medium text-foreground hover:text-primary font-mono"
                                                >
                                                    {name.clone()}
                                                </A>
                                                {service.description.map(|d| view! {
                                                    <span class="text-sm text-muted-foreground">{d}</span>
                                                })}
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div class="flex items-center space-x-2">
                                                <span class={format!(
                                                    "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                                                    if is_active { "bg-green-500/20 text-green-400" }
                                                    else if is_failed { "bg-red-500/20 text-red-400" }
                                                    else { "bg-muted text-muted-foreground" }
                                                )}>
                                                    {service.active_state.clone()}
                                                </span>
                                                <span class="text-xs text-muted-foreground font-mono">
                                                    "(" {service.sub_state.clone()} ")"
                                                </span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <span class={format!(
                                                "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                                                if service.enabled { "bg-blue-500/20 text-blue-400" } else { "bg-muted text-muted-foreground" }
                                            )}>
                                                {if service.enabled { "enabled" } else { "disabled" }}
                                            </span>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                            <div class="flex items-center justify-end space-x-2">
                                                {if is_active {
                                                    view! {
                                                        <button
                                                            class="text-yellow-400 hover:text-yellow-300"
                                                            on:click=move |_| do_action_restart(name_for_restart.clone(), "restart".to_string())
                                                        >
                                                            "Restart"
                                                        </button>
                                                        <button
                                                            class="text-red-400 hover:text-red-300"
                                                            on:click=move |_| do_action_stop(name_for_stop.clone(), "stop".to_string())
                                                        >
                                                            "Stop"
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            class="text-green-400 hover:text-green-300"
                                                            on:click=move |_| do_action_start(name_for_start.clone(), "start".to_string())
                                                        >
                                                            "Start"
                                                        </button>
                                                    }.into_any()
                                                }}
                                                <A
                                                    href=logs_href
                                                    attr:class="text-primary hover:text-primary/80"
                                                >
                                                    "Logs"
                                                </A>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </tbody>
                </table>

                <Show when=move || !loading.get() && filtered_services().is_empty()>
                    <div class="text-center py-12">
                        <p class="text-muted-foreground">
                            "No services match your filter criteria."
                        </p>
                    </div>
                </Show>
            </div>

            <div class="flex items-center justify-between text-sm text-muted-foreground">
                <span>
                    "Showing " {move || filtered_services().len()} " of " {total_services} " services"
                </span>
            </div>
        </div>
    }
}
