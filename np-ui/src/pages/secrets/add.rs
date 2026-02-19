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
                    attr:class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 flex items-center"
                >
                    "<- Back to Secrets"
                </A>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mt-2">
                    "Add Secret"
                </h1>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                    "Create a new SOPS-encrypted secret"
                </p>
            </div>

            // Error display
            {move || error.get().map(|err| view! {
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                    <p class="text-sm text-red-600 dark:text-red-400">{err}</p>
                </div>
            })}

            // Form
            <form on:submit=on_submit>
                <Card>
                    <div class="space-y-6">
                        // Name
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Name"
                                <span class="text-red-500">" *"</span>
                            </label>
                            <input
                                type="text"
                                required=true
                                prop:value=move || name.get()
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
                                placeholder="e.g., database-password"
                            />
                            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                "A unique identifier for this secret (no spaces)"
                            </p>
                        </div>

                        // Type
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Type"
                            </label>
                            <select
                                prop:value=move || secret_type.get()
                                on:change=move |ev| set_secret_type.set(event_target_value(&ev))
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
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
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Value"
                                <span class="text-red-500">" *"</span>
                            </label>
                            <textarea
                                required=true
                                prop:value=move || value.get()
                                on:input=move |ev| set_value.set(event_target_value(&ev))
                                rows=6
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500 font-mono text-sm"
                                placeholder="Enter your secret value..."
                            />
                            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                "This value will be encrypted with age before storage"
                            </p>
                        </div>

                        // Description
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Description"
                            </label>
                            <input
                                type="text"
                                prop:value=move || description.get()
                                on:input=move |ev| set_description.set(event_target_value(&ev))
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
                                placeholder="Optional description of this secret"
                            />
                        </div>

                        // Environment
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Environment"
                            </label>
                            <select
                                prop:value=move || environment.get()
                                on:change=move |ev| set_environment.set(event_target_value(&ev))
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
                            >
                                <option value="">"(none)"</option>
                                <option value="production">"Production"</option>
                                <option value="staging">"Staging"</option>
                                <option value="development">"Development"</option>
                            </select>
                        </div>

                        // Tags
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Tags"
                            </label>
                            <input
                                type="text"
                                prop:value=move || tags.get()
                                on:input=move |ev| set_tags.set(event_target_value(&ev))
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
                                placeholder="comma-separated tags"
                            />
                            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                "Optional tags for organizing secrets (e.g., database, api, auth)"
                            </p>
                        </div>
                    </div>
                </Card>

                // Submit buttons
                <div class="flex justify-end space-x-3 mt-6">
                    <A
                        href="/secrets"
                        attr:class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700"
                    >
                        "Cancel"
                    </A>
                    <button
                        type="submit"
                        disabled=move || submitting.get()
                        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-indigo-400 rounded-md shadow-sm text-sm font-medium text-white"
                    >
                        {move || if submitting.get() { "Creating..." } else { "Create Secret" }}
                    </button>
                </div>
            </form>

            // Security notice
            <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                <div class="flex">
                    <div class="flex-shrink-0">
                        <span class="text-yellow-400 text-lg">"!"</span>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                            "Security Notice"
                        </h3>
                        <div class="mt-2 text-sm text-yellow-700 dark:text-yellow-300">
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
