use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;

use crate::components::common::Card;

/// Add secret page - form for creating a new encrypted secret
#[component]
pub fn AddSecretPage() -> impl IntoView {
    // Form state
    let (name, set_name) = signal(String::new());
    let (secret_type, set_secret_type) = signal("text".to_string());
    let (value, set_value) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (environment, set_environment) = signal(String::new());
    let (tags, set_tags) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (submitting, set_submitting) = signal(false);

    // Handle form submission
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);
        set_error.set(None);

        // In a real app, this would call the API
        // For now, just simulate
        set_submitting.set(false);
    };

    view! {
        <div class="space-y-6 max-w-2xl mx-auto">
            // Header
            <div>
                <A
                    href="/secrets"
                    attr:class="text-sm text-muted-foreground  hover:text-foreground:text-muted-foreground flex items-center"
                >
                    "<- Back to Secrets"
                </A>
                <h1 class="text-2xl font-bold text-foreground  mt-2">
                    "Add Secret"
                </h1>
                <p class="text-sm text-muted-foreground ">
                    "Create a new SOPS-encrypted secret"
                </p>
            </div>

            // Error display
            {move || error.get().map(|err| view! {
                <div class="bg-red-50900/20 border border-red-200800 rounded-lg p-4">
                    <p class="text-sm text-red-600400">{err}</p>
                </div>
            })}

            // Form
            <form on:submit=on_submit>
                <Card>
                    <div class="space-y-6">
                        // Name
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Name"
                                <span class="text-red-500">" *"</span>
                            </label>
                            <input
                                type="text"
                                required=true
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                prop:value=move || name.get()
                                class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                                placeholder="e.g., database-password"
                            />
                            <p class="mt-1 text-xs text-muted-foreground ">
                                "A unique identifier for this secret (no spaces)"
                            </p>
                        </div>

                        // Type
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Type"
                            </label>
                            <select
                                on:change=move |ev| set_secret_type.set(event_target_value(&ev))
                                prop:value=move || secret_type.get()
                                class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                            >
                                <option value="text">"Text (password, API key, token)"</option>
                                <option value="ssh_key">"SSH Private Key"</option>
                                <option value="certificate">"TLS Certificate"</option>
                                <option value="tls_key">"TLS Private Key"</option>
                                <option value="env_file">"Environment File"</option>
                                <option value="json">"JSON Data"</option>
                                <option value="yaml">"YAML Data"</option>
                            </select>
                        </div>

                        // Value
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Value"
                                <span class="text-red-500">" *"</span>
                            </label>
                            <textarea
                                required=true
                                on:input=move |ev| set_value.set(event_target_value(&ev))
                                prop:value=move || value.get()
                                rows=6
                                class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary font-mono text-sm"
                                placeholder="Enter your secret value..."
                            />
                            <p class="mt-1 text-xs text-muted-foreground ">
                                "This value will be encrypted with age before storage"
                            </p>
                        </div>

                        // Description
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Description"
                            </label>
                            <input
                                type="text"
                                on:input=move |ev| set_description.set(event_target_value(&ev))
                                prop:value=move || description.get()
                                class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                                placeholder="Optional description of this secret"
                            />
                        </div>

                        // Environment
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Environment"
                            </label>
                            <select
                                on:change=move |ev| set_environment.set(event_target_value(&ev))
                                prop:value=move || environment.get()
                                class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                            >
                                <option value="">"(none)"</option>
                                <option value="production">"Production"</option>
                                <option value="staging">"Staging"</option>
                                <option value="development">"Development"</option>
                            </select>
                        </div>

                        // Tags
                        <div>
                            <label class="block text-sm font-medium text-foreground  mb-1">
                                "Tags"
                            </label>
                            <input
                                type="text"
                                on:input=move |ev| set_tags.set(event_target_value(&ev))
                                prop:value=move || tags.get()
                                class="w-full px-3 py-2 border border-border  rounded-md shadow-sm bg-background text-foreground  focus:ring-ring focus:border-primary"
                                placeholder="comma-separated tags"
                            />
                            <p class="mt-1 text-xs text-muted-foreground ">
                                "Optional tags for organizing secrets (e.g., database, api, auth)"
                            </p>
                        </div>
                    </div>
                </Card>

                // Submit buttons
                <div class="flex justify-end space-x-3 mt-6">
                    <A
                        href="/secrets"
                        attr:class="px-4 py-2 border border-border  rounded-md shadow-sm text-sm font-medium text-foreground  bg-background hover:bg-muted "
                    >
                        "Cancel"
                    </A>
                    <button
                        type="submit"
                        disabled=move || submitting.get()
                        class="px-4 py-2 bg-primary hover:bg-primary/90 disabled:opacity-50 rounded-md shadow-sm text-sm font-medium text-primary-foreground"
                    >
                        {move || if submitting.get() { "Creating..." } else { "Create Secret" }}
                    </button>
                </div>
            </form>

            // Security notice
            <div class="bg-yellow-50900/20 border border-yellow-200800 rounded-lg p-4">
                <div class="flex">
                    <div class="flex-shrink-0">
                        <span class="text-yellow-400 text-lg">"!"</span>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-yellow-800200">
                            "Security Notice"
                        </h3>
                        <div class="mt-2 text-sm text-yellow-700300">
                            <ul class="list-disc list-inside space-y-1">
                                <li>"Secrets are encrypted client-side before transmission"</li>
                                <li>"Only machines with the corresponding age private key can decrypt"</li>
                                <li>"Make sure you have at least one age key configured"</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
