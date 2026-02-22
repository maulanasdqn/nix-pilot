use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use leptos::wasm_bindgen::JsCast;

use crate::api::check_response_status;
use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AgeKeyInfo {
    pub public_key: String,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct KeyListResponse {
    pub keys: Vec<AgeKeyInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct KeyResponse {
    pub public_key: String,
    pub comment: Option<String>,
}

async fn fetch_keys() -> Result<Vec<AgeKeyInfo>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/secrets/keys", &opts)
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

    let response: KeyListResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(response.keys)
}

async fn generate_key(comment: Option<String>) -> Result<KeyResponse, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({ "comment": comment });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/secrets/keys", &opts)
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

async fn import_key(private_key: String, comment: Option<String>) -> Result<KeyResponse, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let body = serde_json::json!({
        "private_key": private_key,
        "comment": comment
    });

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));

    let request = web_sys::Request::new_with_str_and_init("/api/secrets/keys/import", &opts)
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

async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();

    wasm_bindgen_futures::JsFuture::from(clipboard.write_text(text))
        .await
        .map_err(|_| "Copy failed")?;

    Ok(())
}

/// Age keys management page
#[component]
pub fn KeysPage() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (keys, set_keys) = signal(Vec::<AgeKeyInfo>::new());
    let (action_msg, set_action_msg) = signal(Option::<String>::None);

    // Modal state for generating/importing keys
    let (show_generate_modal, set_show_generate_modal) = signal(false);
    let (show_import_modal, set_show_import_modal) = signal(false);
    let (key_comment, set_key_comment) = signal(String::new());
    let (import_key_value, set_import_key_value) = signal(String::new());
    let (generating, set_generating) = signal(false);
    let (importing, set_importing) = signal(false);

    // Fetch keys on mount
    leptos::task::spawn_local(async move {
        match fetch_keys().await {
            Ok(k) => {
                set_keys.set(k);
                set_loading.set(false);
            }
            Err(e) => {
                set_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });

    let on_generate = move |_| {
        let comment = key_comment.get();
        let comment_opt = if comment.is_empty() { None } else { Some(comment) };

        set_generating.set(true);
        leptos::task::spawn_local(async move {
            match generate_key(comment_opt).await {
                Ok(key) => {
                    set_action_msg.set(Some(format!("Key generated: {}", key.public_key)));
                    set_show_generate_modal.set(false);
                    set_key_comment.set(String::new());
                    set_generating.set(false);
                    // Refresh keys list
                    if let Ok(k) = fetch_keys().await {
                        set_keys.set(k);
                    }
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Generate failed: {}", e)));
                    set_generating.set(false);
                }
            }
        });
    };

    let on_import = move |_| {
        let private_key = import_key_value.get();
        if private_key.is_empty() {
            set_action_msg.set(Some("Please enter a private key".to_string()));
            return;
        }

        let comment = key_comment.get();
        let comment_opt = if comment.is_empty() { None } else { Some(comment) };

        set_importing.set(true);
        leptos::task::spawn_local(async move {
            match import_key(private_key, comment_opt).await {
                Ok(key) => {
                    set_action_msg.set(Some(format!("Key imported: {}", key.public_key)));
                    set_show_import_modal.set(false);
                    set_import_key_value.set(String::new());
                    set_key_comment.set(String::new());
                    set_importing.set(false);
                    // Refresh keys list
                    if let Ok(k) = fetch_keys().await {
                        set_keys.set(k);
                    }
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Import failed: {}", e)));
                    set_importing.set(false);
                }
            }
        });
    };

    let on_copy = move |public_key: String| {
        leptos::task::spawn_local(async move {
            match copy_to_clipboard(&public_key).await {
                Ok(()) => {
                    set_action_msg.set(Some("Public key copied to clipboard".to_string()));
                }
                Err(e) => {
                    set_action_msg.set(Some(format!("Copy failed: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <A
                        href="/secrets"
                        attr:class="text-sm text-muted-foreground  hover:text-foreground:text-muted-foreground flex items-center"
                    >
                        "<- Back to Secrets"
                    </A>
                    <h1 class="text-2xl font-bold text-foreground  mt-2">
                        "Age Keys"
                    </h1>
                    <p class="text-sm text-muted-foreground ">
                        "Manage age encryption keys for SOPS secrets"
                    </p>
                </div>
                <div class="flex items-center space-x-3">
                    <button
                        on:click=move |_| set_show_import_modal.set(true)
                        class="inline-flex items-center px-4 py-2 border border-border  bg-background hover:bg-muted  text-foreground  text-sm font-medium rounded-md transition-colors"
                    >
                        "Import Key"
                    </button>
                    <button
                        on:click=move |_| set_show_generate_modal.set(true)
                        class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors"
                    >
                        "+ Generate Key"
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

            // Info about age keys
            <div class="bg-blue-50900/20 border border-blue-200800 rounded-lg p-4">
                <div class="flex">
                    <div class="flex-shrink-0">
                        <span class="text-blue-400 text-lg">"i"</span>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-blue-800200">
                            "About Age Keys"
                        </h3>
                        <div class="mt-2 text-sm text-blue-700300">
                            <p>
                                "Age is a modern file encryption tool used by SOPS. Each machine that needs to decrypt secrets "
                                "must have access to an age private key. Public keys are used to encrypt secrets."
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            // Loading state
            <Show when=move || loading.get()>
                <Card>
                    <div class="text-center py-12">
                        <p class="text-muted-foreground">"Loading keys..."</p>
                    </div>
                </Card>
            </Show>

            // Keys list
            <Show when=move || !loading.get()>
                {move || {
                    let current_keys = keys.get();
                    if current_keys.is_empty() {
                        view! {
                            <Card>
                                <div class="text-center py-12">
                                    <div class="text-muted-foreground text-5xl mb-4">"K"</div>
                                    <h3 class="text-lg font-medium text-foreground  mb-2">
                                        "No age keys configured"
                                    </h3>
                                    <p class="text-muted-foreground  mb-4">
                                        "Generate or import an age key to start encrypting secrets."
                                    </p>
                                    <button
                                        on:click=move |_| set_show_generate_modal.set(true)
                                        class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors"
                                    >
                                        "Generate Your First Key"
                                    </button>
                                </div>
                            </Card>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-4">
                                {current_keys.into_iter().enumerate().map(|(i, key)| {
                                    let public_key = key.public_key.clone();
                                    let public_key_for_copy = public_key.clone();
                                    let truncated_key = format!(
                                        "{}...{}",
                                        &public_key[..12.min(public_key.len())],
                                        &public_key[public_key.len().saturating_sub(8)..]
                                    );
                                    let is_default = i == 0;

                                    view! {
                                        <Card>
                                            <div class="flex items-start justify-between">
                                                <div class="space-y-2">
                                                    <div class="flex items-center space-x-2">
                                                        <code class="text-sm font-mono text-foreground ">
                                                            {truncated_key}
                                                        </code>
                                                        {is_default.then(|| view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-500/20 text-green-400900200">
                                                                "Default"
                                                            </span>
                                                        })}
                                                    </div>
                                                    {key.comment.map(|c| view! {
                                                        <p class="text-sm text-muted-foreground ">{c}</p>
                                                    })}
                                                    <p class="text-xs text-muted-foreground  font-mono">
                                                        {public_key.clone()}
                                                    </p>
                                                </div>
                                                <div class="flex items-center space-x-2">
                                                    <button
                                                        class="px-3 py-1 text-sm font-medium text-primary hover:text-primary/80 border border-indigo-300 rounded"
                                                        on:click=move |_| on_copy(public_key_for_copy.clone())
                                                    >
                                                        "Copy"
                                                    </button>
                                                </div>
                                            </div>
                                        </Card>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </Show>

            // Usage instructions
            <Card title="Using Age Keys".to_string()>
                <div class="space-y-4">
                    <div>
                        <h4 class="font-medium text-foreground  mb-2">
                            "Key Storage"
                        </h4>
                        <p class="text-sm text-muted-foreground ">
                            "Private keys are stored at "
                            <code class="bg-muted  px-1 rounded">"~/.config/sops/age/keys.txt"</code>
                            " (default SOPS location)."
                        </p>
                    </div>

                    <div>
                        <h4 class="font-medium text-foreground  mb-2">
                            "Deploying to NixOS"
                        </h4>
                        <pre class="bg-muted  rounded p-3 text-xs overflow-x-auto font-mono">
{r#"# Copy the private key to your server:
scp ~/.config/sops/age/keys.txt root@server:/var/lib/sops-nix/key.txt

# In your NixOS configuration:
sops.age.keyFile = "/var/lib/sops-nix/key.txt";"#}
                        </pre>
                    </div>

                    <div>
                        <h4 class="font-medium text-foreground  mb-2">
                            "Adding to .sops.yaml"
                        </h4>
                        <pre class="bg-muted  rounded p-3 text-xs overflow-x-auto font-mono">
{r#"# Add your public key to .sops.yaml:
creation_rules:
  - path_regex: secrets/.*\.yaml$
    key_groups:
      - age:
          - age1your-public-key-here"#}
                        </pre>
                    </div>
                </div>
            </Card>

            // Generate key modal
            <Show when=move || show_generate_modal.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-background rounded-lg shadow-xl w-full max-w-md p-6">
                        <h3 class="text-lg font-medium text-foreground  mb-4">
                            "Generate New Age Key"
                        </h3>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-foreground  mb-1">
                                    "Comment (optional)"
                                </label>
                                <input
                                    type="text"
                                    on:input=move |ev| set_key_comment.set(event_target_value(&ev))
                                    prop:value=move || key_comment.get()
                                    class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                                    placeholder="e.g., production-server-1"
                                />
                            </div>
                        </div>

                        <div class="flex justify-end space-x-3 mt-6">
                            <button
                                on:click=move |_| {
                                    set_show_generate_modal.set(false);
                                    set_key_comment.set(String::new());
                                }
                                class="px-4 py-2 border border-border  rounded-md shadow-sm text-sm font-medium text-foreground  bg-background hover:bg-muted "
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=on_generate
                                disabled=move || generating.get()
                                class="px-4 py-2 bg-primary hover:bg-primary/90 rounded-md shadow-sm text-sm font-medium text-primary-foreground disabled:opacity-50"
                            >
                                {move || if generating.get() { "Generating..." } else { "Generate" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Import key modal
            <Show when=move || show_import_modal.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-background rounded-lg shadow-xl w-full max-w-md p-6">
                        <h3 class="text-lg font-medium text-foreground  mb-4">
                            "Import Age Key"
                        </h3>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-foreground  mb-1">
                                    "Private Key"
                                </label>
                                <textarea
                                    on:input=move |ev| set_import_key_value.set(event_target_value(&ev))
                                    prop:value=move || import_key_value.get()
                                    rows=3
                                    class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary font-mono text-sm"
                                    placeholder="AGE-SECRET-KEY-1..."
                                />
                                <p class="mt-1 text-xs text-muted-foreground ">
                                    "Paste your age private key (starts with AGE-SECRET-KEY-)"
                                </p>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-foreground  mb-1">
                                    "Comment (optional)"
                                </label>
                                <input
                                    type="text"
                                    on:input=move |ev| set_key_comment.set(event_target_value(&ev))
                                    prop:value=move || key_comment.get()
                                    class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                                    placeholder="e.g., imported from backup"
                                />
                            </div>
                        </div>

                        <div class="flex justify-end space-x-3 mt-6">
                            <button
                                on:click=move |_| {
                                    set_show_import_modal.set(false);
                                    set_import_key_value.set(String::new());
                                    set_key_comment.set(String::new());
                                }
                                class="px-4 py-2 border border-border  rounded-md shadow-sm text-sm font-medium text-foreground  bg-background hover:bg-muted "
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=on_import
                                disabled=move || importing.get()
                                class="px-4 py-2 bg-primary hover:bg-primary/90 rounded-md shadow-sm text-sm font-medium text-primary-foreground disabled:opacity-50"
                            >
                                {move || if importing.get() { "Importing..." } else { "Import" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
