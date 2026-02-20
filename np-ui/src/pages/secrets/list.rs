use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SecretSummary {
    id: String,
    name: String,
    #[serde(default)]
    secret_type: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    environment: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SecretListResponse {
    secrets: Vec<SecretSummary>,
}

async fn fetch_secrets() -> Result<Vec<SecretSummary>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/secrets", &opts)
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

    let response: SecretListResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(response.secrets)
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

/// Secrets list page - displays all stored secrets
#[component]
pub fn SecretsListPage() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (secrets, set_secrets) = signal(Vec::<SecretSummary>::new());
    let (action_msg, set_action_msg) = signal(Option::<String>::None);

    // Fetch secrets on mount
    leptos::task::spawn_local(async move {
        match fetch_secrets().await {
            Ok(s) => {
                set_secrets.set(s);
                set_loading.set(false);
            }
            Err(e) => {
                set_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });

    let on_copy = move |secret_name: String| {
        let path = format!("config.sops.secrets.\"{}\".path", secret_name);
        leptos::task::spawn_local(async move {
            match copy_to_clipboard(&path).await {
                Ok(()) => {
                    set_action_msg.set(Some(format!("Copied path for '{}' to clipboard", secret_name)));
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
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        "Secrets"
                    </h1>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "SOPS-encrypted secrets for NixOS"
                    </p>
                </div>
                <div class="flex items-center space-x-3">
                    <A
                        href="/secrets/keys"
                        attr:class="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-md transition-colors"
                    >
                        "Manage Keys"
                    </A>
                    <A
                        href="/secrets/add"
                        attr:class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                    >
                        "+ Add Secret"
                    </A>
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

            // Info banner about SOPS/sops-nix
            <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                <div class="flex">
                    <div class="flex-shrink-0">
                        <span class="text-blue-400 text-lg">"i"</span>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-blue-800 dark:text-blue-200">
                            "Secrets are encrypted with SOPS"
                        </h3>
                        <div class="mt-2 text-sm text-blue-700 dark:text-blue-300">
                            <p>
                                "Secrets are encrypted using age encryption and stored in YAML files compatible with "
                                <a href="https://github.com/Mic92/sops-nix" class="underline" target="_blank">"sops-nix"</a>
                                ". They can be deployed to your NixOS machines securely."
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            // Loading state
            <Show when=move || loading.get()>
                <Card>
                    <div class="text-center py-12">
                        <p class="text-gray-500">"Loading secrets..."</p>
                    </div>
                </Card>
            </Show>

            // Secrets list or empty state
            <Show when=move || !loading.get()>
                {move || {
                    let current_secrets = secrets.get();
                    if current_secrets.is_empty() {
                        view! {
                            <Card>
                                <div class="text-center py-12">
                                    <div class="text-gray-400 text-5xl mb-4">"^"</div>
                                    <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                                        "No secrets stored"
                                    </h3>
                                    <p class="text-gray-500 dark:text-gray-400 mb-4">
                                        "Create encrypted secrets for your NixOS deployments."
                                    </p>
                                    <div class="flex justify-center space-x-4">
                                        <A
                                            href="/secrets/keys"
                                            attr:class="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-md transition-colors"
                                        >
                                            "Setup Age Keys"
                                        </A>
                                        <A
                                            href="/secrets/add"
                                            attr:class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                                        >
                                            "Add Your First Secret"
                                        </A>
                                    </div>
                                </div>
                            </Card>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-4">
                                {current_secrets.into_iter().map(|secret| {
                                    let id = secret.id.clone();
                                    let name = secret.name.clone();
                                    let name_for_copy = name.clone();
                                    let secret_type = secret.secret_type.clone();
                                    let description = secret.description.clone();
                                    let environment = secret.environment.clone();
                                    let tags = secret.tags.clone();

                                    let type_badge_class = match secret_type.as_str() {
                                        "text" => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                                        "ssh_key" => "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
                                        "certificate" | "tls_key" => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
                                        "env_file" => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
                                        _ => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
                                    };

                                    view! {
                                        <Card class="hover:border-indigo-500 transition-colors".to_string()>
                                            <div class="flex items-start justify-between">
                                                <div class="space-y-2">
                                                    <div class="flex items-center space-x-2">
                                                        <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">
                                                            {name.clone()}
                                                        </h3>
                                                        <span class=format!("inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {}", type_badge_class)>
                                                            {secret_type}
                                                        </span>
                                                    </div>

                                                    {description.map(|desc| view! {
                                                        <p class="text-sm text-gray-500 dark:text-gray-400">
                                                            {desc}
                                                        </p>
                                                    })}

                                                    <div class="flex items-center space-x-2">
                                                        {environment.map(|env| view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200">
                                                                {env}
                                                            </span>
                                                        })}

                                                        {tags.into_iter().map(|tag| view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400">
                                                                {tag}
                                                            </span>
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>

                                                <div class="flex items-center space-x-2">
                                                    <button
                                                        class="px-3 py-1 text-sm font-medium text-indigo-600 hover:text-indigo-800 dark:text-indigo-400 border border-indigo-300 dark:border-indigo-600 rounded"
                                                        on:click=move |_| on_copy(name_for_copy.clone())
                                                        title="Copy secret path"
                                                    >
                                                        "Copy"
                                                    </button>
                                                    <A
                                                        href=format!("/secrets/{}", id)
                                                        attr:class="px-3 py-1 text-sm font-medium text-gray-600 hover:text-gray-800 dark:text-gray-400 border border-gray-300 dark:border-gray-600 rounded"
                                                        attr:title="View details"
                                                    >
                                                        "View"
                                                    </A>
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

            // Quick reference card
            <Card title="Quick Reference".to_string()>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="font-medium text-gray-900 dark:text-gray-100 mb-2">
                            "Secret Types"
                        </h4>
                        <ul class="space-y-2 text-sm text-gray-600 dark:text-gray-400">
                            <li class="flex items-center">
                                <span class="w-20 font-medium">"Text"</span>
                                <span>"Passwords, API keys, tokens"</span>
                            </li>
                            <li class="flex items-center">
                                <span class="w-20 font-medium">"SSH Key"</span>
                                <span>"Private SSH keys"</span>
                            </li>
                            <li class="flex items-center">
                                <span class="w-20 font-medium">"TLS"</span>
                                <span>"Certificates and private keys"</span>
                            </li>
                            <li class="flex items-center">
                                <span class="w-20 font-medium">"Env File"</span>
                                <span>"Environment variable files"</span>
                            </li>
                        </ul>
                    </div>
                    <div>
                        <h4 class="font-medium text-gray-900 dark:text-gray-100 mb-2">
                            "Using with sops-nix"
                        </h4>
                        <pre class="bg-gray-100 dark:bg-gray-800 rounded p-3 text-xs overflow-x-auto font-mono">
{r#"# In your NixOS config:
sops.secrets."mySecret" = {
  sopsFile = ./secrets.yaml;
  owner = "myuser";
};

# Access in services:
config.sops.secrets."mySecret".path"#}
                        </pre>
                    </div>
                </div>
            </Card>
        </div>
    }
}
