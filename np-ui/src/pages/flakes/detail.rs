use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::common::Card;

/// Flake detail page
#[component]
pub fn FlakeDetailPage() -> impl IntoView {
    let params = use_params_map();
    let flake_id = move || params.read().get("id").unwrap_or_default();

    // State for update modal
    let (show_update_modal, set_show_update_modal) = signal(false);
    let (selected_input, set_selected_input) = signal(Option::<String>::None);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href="/flakes"
                        attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        {"\u{2190} Back"}
                    </A>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        "Flake Details"
                    </h1>
                </div>
                <div class="flex space-x-3">
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md"
                        on:click=move |_| set_show_update_modal.set(true)
                    >
                        "Update All Inputs"
                    </button>
                    <button class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md">
                        "Refresh Metadata"
                    </button>
                    <button class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-md">
                        "Unregister"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Flake Info
                <Card title="Information".to_string()>
                    <dl class="space-y-4">
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Name"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                "Loading..."
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Path"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                "Loading..."
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Description"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                "No description"
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Last Modified"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                "Unknown"
                            </dd>
                        </div>
                    </dl>
                </Card>

                // Inputs
                <Card title="Inputs".to_string() class="lg:col-span-2".to_string()>
                    <div class="space-y-4">
                        // Placeholder inputs - in real app, these would be fetched
                        <FlakeInputRow
                            name="nixpkgs".to_string()
                            url="github:NixOS/nixpkgs/nixos-24.05".to_string()
                            locked_rev="abc123...".to_string()
                            on_update=Callback::new(move |name: String| {
                                set_selected_input.set(Some(name.clone()));
                                set_show_update_modal.set(true);
                            })
                        />
                        <FlakeInputRow
                            name="home-manager".to_string()
                            url="github:nix-community/home-manager".to_string()
                            locked_rev="def456...".to_string()
                            on_update=Callback::new(move |name: String| {
                                set_selected_input.set(Some(name.clone()));
                                set_show_update_modal.set(true);
                            })
                        />
                    </div>
                </Card>
            </div>

            // Outputs section
            <Card title="Outputs".to_string()>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <OutputCategory
                        name="NixOS Configurations"
                        items=vec!["desktop".to_string(), "server".to_string()]
                    />
                    <OutputCategory
                        name="Packages"
                        items=vec!["x86_64-linux: 3 packages".to_string()]
                    />
                    <OutputCategory
                        name="Dev Shells"
                        items=vec!["default".to_string()]
                    />
                    <OutputCategory
                        name="NixOS Modules"
                        items=vec!["custom-module".to_string()]
                    />
                </div>
            </Card>

            // Quick Actions
            <Card title="Quick Actions".to_string()>
                <div class="flex flex-wrap gap-4">
                    <A
                        href=move || format!("/deploy?flake={}", flake_id())
                        attr:class="inline-flex items-center px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium rounded-md transition-colors"
                    >
                        "Deploy Configuration"
                    </A>
                    <A
                        href=move || format!("/install?flake={}", flake_id())
                        attr:class="inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md transition-colors"
                    >
                        "Install to Machine"
                    </A>
                    <button class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md transition-colors">
                        "Build"
                    </button>
                </div>
            </Card>

            <p class="text-sm text-gray-500">
                "Flake ID: " {flake_id}
            </p>

            // Update Modal
            <Show when=move || show_update_modal.get()>
                <UpdateInputModal
                    input_name=selected_input
                    on_close=Callback::new(move |_: ()| {
                        set_show_update_modal.set(false);
                        set_selected_input.set(None);
                    })
                />
            </Show>
        </div>
    }
}

/// Component for displaying a flake input row
#[component]
fn FlakeInputRow(
    #[prop(into)] name: String,
    #[prop(into)] url: String,
    #[prop(into)] locked_rev: String,
    on_update: Callback<String>,
) -> impl IntoView {
    let name_clone = name.clone();

    view! {
        <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-2">
                    <span class="font-medium text-gray-900 dark:text-gray-100">{name.clone()}</span>
                </div>
                <p class="text-sm text-gray-500 dark:text-gray-400 font-mono truncate">
                    {url}
                </p>
                <p class="text-xs text-gray-400 dark:text-gray-500 font-mono">
                    "Locked: " {locked_rev}
                </p>
            </div>
            <div class="flex space-x-2 ml-4">
                <button
                    class="px-3 py-1 text-sm font-medium text-indigo-600 hover:text-indigo-800 dark:text-indigo-400"
                    on:click=move |_| on_update.run(name_clone.clone())
                >
                    "Update"
                </button>
                <button class="px-3 py-1 text-sm font-medium text-gray-600 hover:text-gray-800 dark:text-gray-400">
                    "Edit"
                </button>
            </div>
        </div>
    }
}

/// Component for displaying output category
#[component]
fn OutputCategory(
    #[prop(into)] name: String,
    items: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <h4 class="font-medium text-gray-900 dark:text-gray-100 mb-2">{name}</h4>
            <ul class="text-sm text-gray-500 dark:text-gray-400 space-y-1">
                {items.into_iter().map(|item| view! {
                    <li class="font-mono">{item}</li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

/// Modal for updating flake inputs
#[component]
fn UpdateInputModal(
    input_name: ReadSignal<Option<String>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let title = move || {
        match input_name.get() {
            Some(name) => format!("Update '{}'", name),
            None => "Update All Inputs".to_string(),
        }
    };

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center">
            // Backdrop
            <div
                class="absolute inset-0 bg-black bg-opacity-50"
                on:click=move |_| on_close.run(())
            />

            // Modal content
            <div class="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 p-6">
                <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100 mb-4">
                    {title}
                </h2>

                <div class="space-y-4">
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "This will run 'nix flake update' to update the lock file."
                    </p>

                    // Progress area (placeholder)
                    <div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400 h-32 overflow-auto">
                        "$ nix flake update"
                        <br/>
                        "Ready to update..."
                    </div>
                </div>

                <div class="flex justify-end space-x-3 mt-6">
                    <button
                        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md"
                        on:click=move |_| on_close.run(())
                    >
                        "Cancel"
                    </button>
                    <button class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md">
                        "Update"
                    </button>
                </div>
            </div>
        </div>
    }
}
