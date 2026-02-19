use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Add/Register flake page
#[component]
pub fn AddFlakePage() -> impl IntoView {
    // Form state
    let (name, set_name) = signal(String::new());
    let (path, set_path) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (is_loading, set_is_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error.set(None);

        // In a real app, this would call the API to register the flake
        // and then navigate to the flake detail page
        let _ = name.get(); // Use the values
        let _ = path.get();
        let _ = description.get();
    };

    view! {
        <div class="max-w-2xl mx-auto space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Register Flake"
                </h1>
                <A
                    href="/flakes"
                    attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Cancel"
                </A>
            </div>

            <Card>
                <form on:submit=on_submit class="space-y-6">
                    // Error display
                    <Show when=move || error.get().is_some()>
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4">
                            <p class="text-sm text-red-600 dark:text-red-400">
                                {move || error.get().unwrap_or_default()}
                            </p>
                        </div>
                    </Show>

                    // Name
                    <div>
                        <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Name"
                        </label>
                        <input
                            type="text"
                            id="name"
                            required=true
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            placeholder="my-nixos-config"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                        />
                        <p class="mt-1 text-sm text-gray-500">
                            "A friendly name to identify this flake"
                        </p>
                    </div>

                    // Path
                    <div>
                        <label for="path" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Path"
                        </label>
                        <input
                            type="text"
                            id="path"
                            required=true
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm font-mono"
                            placeholder="/home/user/nixos-config"
                            prop:value=path
                            on:input=move |ev| set_path.set(event_target_value(&ev))
                        />
                        <p class="mt-1 text-sm text-gray-500">
                            "The absolute path to the flake directory (must contain flake.nix)"
                        </p>
                    </div>

                    // Description
                    <div>
                        <label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Description (optional)"
                        </label>
                        <textarea
                            id="description"
                            rows="3"
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            placeholder="My NixOS configuration flake..."
                            prop:value=description
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                        />
                    </div>

                    // Flake reference preview
                    <div class="bg-gray-50 dark:bg-gray-800 rounded-md p-4">
                        <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Flake Reference"
                        </h4>
                        <p class="font-mono text-sm text-gray-600 dark:text-gray-400">
                            {move || {
                                let p = path.get();
                                if p.is_empty() {
                                    "path:/path/to/flake".to_string()
                                } else {
                                    format!("path:{}", p)
                                }
                            }}
                        </p>
                    </div>

                    // Info box
                    <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-4">
                        <h4 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-2">
                            "What happens when you register a flake?"
                        </h4>
                        <ul class="text-sm text-blue-700 dark:text-blue-300 list-disc list-inside space-y-1">
                            <li>"The flake metadata will be parsed and cached"</li>
                            <li>"You can view and manage flake inputs"</li>
                            <li>"Deploy configurations from this flake to machines"</li>
                            <li>"Track input updates and lock file changes"</li>
                        </ul>
                    </div>

                    // Submit Button
                    <div class="flex justify-end space-x-3">
                        <A
                            href="/flakes"
                            attr:class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-600"
                        >
                            "Cancel"
                        </A>
                        <button
                            type="submit"
                            disabled=move || is_loading.get()
                            class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                        >
                            {move || if is_loading.get() { "Registering..." } else { "Register Flake" }}
                        </button>
                    </div>
                </form>
            </Card>
        </div>
    }
}
