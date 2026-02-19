use leptos::prelude::*;

/// Top navigation bar
#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
            <div class="px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <span class="text-xl font-bold text-indigo-600 dark:text-indigo-400">
                                "Nix Pilot"
                            </span>
                        </div>
                    </div>
                    <div class="flex items-center space-x-4">
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                            "NixOS Management"
                        </span>
                    </div>
                </div>
            </div>
        </nav>
    }
}
