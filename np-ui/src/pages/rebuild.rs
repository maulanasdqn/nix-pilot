use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};

use crate::components::common::Card;
use crate::components::ui::spinner::Spinner;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(default)]
    system_type: String,
}

async fn fetch_system_info() -> Result<SystemInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/system/info", &opts)
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

    serde_wasm_bindgen::from_value(json).map_err(|e| format!("Deserialize failed: {:?}", e))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RebuildResponse {
    success: bool,
    job_id: Option<String>,
    message: String,
    #[serde(default)]
    system_type: String,
    #[serde(default)]
    command: String,
}

async fn start_rebuild(action: &str, flake_path: Option<String>) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({
        "action": action,
        "flake_path": flake_path,
    });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/system/rebuild", &opts)
        .map_err(|_| "Failed to create request")?;

    request.headers().set("Content-Type", "application/json").ok();
    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let data: RebuildResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    if data.success {
        data.job_id.ok_or("No job ID returned".to_string())
    } else {
        Err(data.message)
    }
}

/// Rebuild page - NixOS/nix-darwin system rebuild operations
#[component]
pub fn RebuildPage() -> impl IntoView {
    let (action, set_action) = signal("switch".to_string());
    let (flake_path, set_flake_path) = signal(String::new());
    let (is_running, set_is_running) = signal(false);
    let (output, set_output) = signal(Vec::<String>::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (system_info, set_system_info) = signal(SystemInfo::default());

    // Fetch system info on mount
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(info) = fetch_system_info().await {
                set_system_info.set(info);
            }
        });
    });

    let is_darwin = move || system_info.get().system_type == "nix-darwin";
    let system_name = move || if is_darwin() { "nix-darwin" } else { "NixOS" };

    let on_rebuild = move |_| {
        let action_val = action.get();
        let flake = if flake_path.get().is_empty() {
            None
        } else {
            Some(flake_path.get())
        };

        set_is_running.set(true);
        set_error.set(None);
        set_success.set(None);
        set_output.set(vec!["Starting rebuild...".to_string()]);

        leptos::task::spawn_local(async move {
            match start_rebuild(&action_val, flake).await {
                Ok(job_id) => {
                    set_output.update(|o| o.push(format!("Job started: {}", job_id)));
                    set_success.set(Some(format!("Rebuild ({}) started successfully", action_val)));
                    set_is_running.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_is_running.set(false);
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            // Header
            <div>
                <h1 class="text-2xl font-bold text-foreground">
                    "Rebuild System"
                </h1>
                <p class="text-sm text-muted-foreground">
                    {move || format!("Rebuild your {} configuration", system_name())}
                </p>
            </div>

            // Error/Success messages
            {move || error.get().map(|e| view! {
                <div class="p-4 bg-red-500/10 border border-red-500/20 text-red-400 rounded-lg">
                    {e}
                </div>
            })}

            {move || success.get().map(|s| view! {
                <div class="p-4 bg-green-500/10 border border-green-500/20 text-green-400 rounded-lg">
                    {s}
                </div>
            })}

            // Rebuild options
            <Card title="Rebuild Options".to_string()>
                <div class="space-y-6">
                    // Action selection
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-2">
                            "Rebuild Action"
                        </label>
                        <select
                            class="w-full px-3 py-2 border border-border rounded-md bg-background text-foreground focus:ring-ring focus:border-ring"
                            on:change=move |ev| set_action.set(event_target_value(&ev))
                            prop:value=move || action.get()
                        >
                            <Show
                                when=move || is_darwin()
                                fallback=|| view! {
                                    <option value="switch">"switch - Build and activate immediately"</option>
                                    <option value="boot">"boot - Build and activate on next boot"</option>
                                    <option value="test">"test - Build and activate, don't add to boot menu"</option>
                                    <option value="build">"build - Build only, don't activate"</option>
                                    <option value="dry-build">"dry-build - Show what would be built"</option>
                                    <option value="dry-activate">"dry-activate - Show what would change"</option>
                                }
                            >
                                <option value="switch">"switch - Build and activate immediately"</option>
                                <option value="build">"build - Build only, don't activate"</option>
                                <option value="check">"check - Verify configuration builds"</option>
                                <option value="changelog">"changelog - Show recent changes"</option>
                            </Show>
                        </select>
                        <p class="mt-1 text-xs text-muted-foreground">
                            "Choose how to apply the configuration"
                        </p>
                    </div>

                    // Flake path (optional)
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-2">
                            "Flake Path (optional)"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-border rounded-md bg-background text-foreground focus:ring-ring focus:border-ring"
                            placeholder=move || if is_darwin() { "~/.config/nix-darwin or github:user/repo" } else { "/etc/nixos or github:user/repo" }
                            on:input=move |ev| set_flake_path.set(event_target_value(&ev))
                            prop:value=move || flake_path.get()
                        />
                        <p class="mt-1 text-xs text-muted-foreground">
                            "Leave empty to use system default configuration"
                        </p>
                    </div>

                    // Action buttons
                    <div class="flex items-center gap-4">
                        <button
                            class="px-6 py-2 bg-primary hover:bg-primary/90 text-primary-foreground font-medium rounded-md disabled:opacity-50 transition-colors inline-flex items-center gap-2"
                            on:click=on_rebuild
                            disabled=move || is_running.get()
                        >
                            <Show when=move || is_running.get() fallback=|| "Start Rebuild">
                                <Spinner class="size-4" />" Rebuilding..."
                            </Show>
                        </button>
                    </div>
                </div>
            </Card>

            // Output
            <Show when=move || !output.get().is_empty()>
                <Card title="Output".to_string()>
                    <div class="bg-muted rounded-md p-4 font-mono text-sm max-h-96 overflow-auto">
                        {move || output.get().into_iter().map(|line| {
                            view! {
                                <div class="text-muted-foreground">{line}</div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </Card>
            </Show>

            // Quick actions
            <Card title="Quick Reference".to_string()>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="font-medium text-foreground mb-2">"Common Actions"</h4>
                        <Show
                            when=move || is_darwin()
                            fallback=|| view! {
                                <ul class="space-y-2 text-sm text-muted-foreground">
                                    <li class="flex items-start gap-2">
                                        <span class="font-medium text-foreground w-24">"switch"</span>
                                        <span>"Build, activate now, add to boot menu"</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="font-medium text-foreground w-24">"boot"</span>
                                        <span>"Build, activate on reboot"</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="font-medium text-foreground w-24">"test"</span>
                                        <span>"Build and test without persisting"</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="font-medium text-foreground w-24">"build"</span>
                                        <span>"Build only, no activation"</span>
                                    </li>
                                </ul>
                            }
                        >
                            <ul class="space-y-2 text-sm text-muted-foreground">
                                <li class="flex items-start gap-2">
                                    <span class="font-medium text-foreground w-24">"switch"</span>
                                    <span>"Build and activate immediately"</span>
                                </li>
                                <li class="flex items-start gap-2">
                                    <span class="font-medium text-foreground w-24">"build"</span>
                                    <span>"Build only, no activation"</span>
                                </li>
                                <li class="flex items-start gap-2">
                                    <span class="font-medium text-foreground w-24">"check"</span>
                                    <span>"Verify configuration builds"</span>
                                </li>
                                <li class="flex items-start gap-2">
                                    <span class="font-medium text-foreground w-24">"changelog"</span>
                                    <span>"Show recent changes"</span>
                                </li>
                            </ul>
                        </Show>
                    </div>
                    <div>
                        <h4 class="font-medium text-foreground mb-2">"CLI Equivalent"</h4>
                        <Show
                            when=move || is_darwin()
                            fallback=|| view! {
                                <pre class="bg-muted rounded p-3 text-xs overflow-x-auto font-mono">
{r#"# Using nixos-rebuild
sudo nixos-rebuild switch

# With flakes
sudo nixos-rebuild switch \
  --flake /etc/nixos#hostname

# Dry run
sudo nixos-rebuild dry-build"#}
                                </pre>
                            }
                        >
                            <pre class="bg-muted rounded p-3 text-xs overflow-x-auto font-mono">
{r#"# Using darwin-rebuild
darwin-rebuild switch

# With flakes
darwin-rebuild switch \
  --flake ~/.config/nix-darwin

# Check configuration
darwin-rebuild check"#}
                            </pre>
                        </Show>
                    </div>
                </div>
            </Card>
        </div>
    }
}
