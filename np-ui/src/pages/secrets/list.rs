use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Secrets list page - displays all stored secrets
#[component]
pub fn SecretsListPage() -> impl IntoView {
    // In a real app, this would fetch from the API
    // For now, we'll show the empty state
    let secrets: Vec<()> = vec![];

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

            // Secrets list or empty state
            {if secrets.is_empty() {
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
                        // Secret cards would be rendered here
                    </div>
                }.into_any()
            }}

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

/// Secret card component for displaying a single secret
#[component]
fn SecretCard(
    #[prop(into)] id: String,
    #[prop(into)] name: String,
    #[prop(into)] secret_type: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(into, optional)] environment: Option<String>,
    #[prop(into)] tags: Vec<String>,
) -> impl IntoView {
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
                        class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        attr:title="Copy secret path"
                    >
                        "C"
                    </button>
                    <A
                        href=format!("/secrets/{}", id)
                        attr:class="p-2 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400"
                        attr:title="View details"
                    >
                        ">"
                    </A>
                </div>
            </div>
        </Card>
    }
}
