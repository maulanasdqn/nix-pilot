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
                <h1 class="text-2xl font-bold text-foreground ">
                    "Register Flake"
                </h1>
                <A
                    href="/flakes"
                    attr:class="text-muted-foreground hover:text-foreground :text-gray-200"
                >
                    "Cancel"
                </A>
            </div>

            <Card>
                <form on:submit=on_submit class="space-y-6">
                    // Error display
                    <Show when=move || error.get().is_some()>
                        <div class="bg-red-50900/20 border border-red-200800 rounded-md p-4">
                            <p class="text-sm text-red-600400">
                                {move || error.get().unwrap_or_default()}
                            </p>
                        </div>
                    </Show>

                    // Name
                    <div>
                        <label for="name" class="block text-sm font-medium text-foreground ">
                            "Name"
                        </label>
                        <input
                            type="text"
                            id="name"
                            required=true
                            class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                            placeholder="my-nixos-config"
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                            prop:value=move || name.get()
                        />
                        <p class="mt-1 text-sm text-muted-foreground">
                            "A friendly name to identify this flake"
                        </p>
                    </div>

                    // Path
                    <div>
                        <label for="path" class="block text-sm font-medium text-foreground ">
                            "Path"
                        </label>
                        <input
                            type="text"
                            id="path"
                            required=true
                            class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring font-mono"
                            placeholder="/home/user/nixos-config"
                            on:input=move |ev| set_path.set(event_target_value(&ev))
                            prop:value=move || path.get()
                        />
                        <p class="mt-1 text-sm text-muted-foreground">
                            "The absolute path to the flake directory (must contain flake.nix)"
                        </p>
                    </div>

                    // Description
                    <div>
                        <label for="description" class="block text-sm font-medium text-foreground ">
                            "Description (optional)"
                        </label>
                        <textarea
                            id="description"
                            rows="3"
                            class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                            placeholder="My NixOS configuration flake..."
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            prop:value=move || description.get()
                        />
                    </div>

                    // Flake reference preview
                    <div class="bg-muted  rounded-md p-4">
                        <h4 class="text-sm font-medium text-foreground  mb-2">
                            "Flake Reference"
                        </h4>
                        <p class="font-mono text-sm text-muted-foreground ">
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
                    <div class="bg-blue-50900/20 border border-blue-200800 rounded-md p-4">
                        <h4 class="text-sm font-medium text-blue-800200 mb-2">
                            "What happens when you register a flake?"
                        </h4>
                        <ul class="text-sm text-blue-700300 list-disc list-inside space-y-1">
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
                            attr:class="px-4 py-2 text-sm font-medium text-foreground bg-card border border-border rounded-md hover:bg-muted    "
                        >
                            "Cancel"
                        </A>
                        <button
                            type="submit"
                            disabled=move || is_loading.get()
                            class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground border border-transparent rounded-md hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-ring disabled:opacity-50"
                        >
                            {move || if is_loading.get() { "Registering..." } else { "Register Flake" }}
                        </button>
                    </div>
                </form>
            </Card>
        </div>
    }
}
