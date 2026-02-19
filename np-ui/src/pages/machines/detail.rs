use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::common::Card;

/// Machine detail page
#[component]
pub fn MachineDetailPage() -> impl IntoView {
    let params = use_params_map();
    let machine_id = move || params.read().get("id").unwrap_or_default();

    // In a real app, this would fetch the machine from the API
    // For now, we'll show a placeholder

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href="/machines"
                        attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        "← Back"
                    </A>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        "Machine Details"
                    </h1>
                </div>
                <div class="flex space-x-3">
                    <button class="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md">
                        "Test Connection"
                    </button>
                    <button class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-md">
                        "Delete"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Connection Info
                <Card title="Connection".to_string()>
                    <dl class="space-y-4">
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Status"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                                    "Unknown"
                                </span>
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Host"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                "Loading..."
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Port"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                "22"
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Username"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                "Loading..."
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Auth Method"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                "SSH Agent"
                            </dd>
                        </div>
                    </dl>
                </Card>

                // System Info
                <Card title="System Info".to_string()>
                    <dl class="space-y-4">
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"System"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                "Not available - test connection first"
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Last Seen"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                "Never"
                            </dd>
                        </div>
                    </dl>
                </Card>

                // Quick Actions
                <Card title="Quick Actions".to_string() class="lg:col-span-2".to_string()>
                    <div class="flex flex-wrap gap-4">
                        <A
                            href=move || format!("/services/{}", machine_id())
                            attr:class="inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md transition-colors"
                        >
                            "View Services"
                        </A>
                        <A
                            href=move || format!("/deploy?machine={}", machine_id())
                            attr:class="inline-flex items-center px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium rounded-md transition-colors"
                        >
                            "Deploy Configuration"
                        </A>
                        <button class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md transition-colors">
                            "Open Terminal"
                        </button>
                    </div>
                </Card>
            </div>

            <p class="text-sm text-gray-500">
                "Machine ID: " {machine_id}
            </p>
        </div>
    }
}
