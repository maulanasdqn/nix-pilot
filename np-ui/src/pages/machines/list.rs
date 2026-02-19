use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Machine status badge component
#[component]
fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let (bg_class, text) = match status.as_str() {
        "online" => ("bg-green-100 text-green-800", "Online"),
        "offline" => ("bg-red-100 text-red-800", "Offline"),
        "auth_failed" => ("bg-yellow-100 text-yellow-800", "Auth Failed"),
        _ => ("bg-gray-100 text-gray-800", "Unknown"),
    };

    view! {
        <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", bg_class)>
            {text}
        </span>
    }
}

/// Machine list page
#[component]
pub fn MachineListPage() -> impl IntoView {
    // In a real app, this would fetch from the API
    // For now, we'll show the empty state
    let machines: Vec<()> = vec![];

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Machines"
                </h1>
                <A
                    href="/machines/add"
                    attr:class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                >
                    "+ Add Machine"
                </A>
            </div>

            {if machines.is_empty() {
                view! {
                    <Card>
                        <div class="text-center py-12">
                            <div class="text-gray-400 text-5xl mb-4">"🖥️"</div>
                            <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                                "No machines configured"
                            </h3>
                            <p class="text-gray-500 dark:text-gray-400 mb-4">
                                "Add a machine to start managing NixOS systems remotely."
                            </p>
                            <A
                                href="/machines/add"
                                attr:class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                            >
                                "Add Your First Machine"
                            </A>
                        </div>
                    </Card>
                }.into_any()
            } else {
                view! {
                    <div class="bg-white dark:bg-gray-800 shadow overflow-hidden rounded-lg">
                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                            <thead class="bg-gray-50 dark:bg-gray-900">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        "Name"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        "Host"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        "Status"
                                    </th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        "Last Seen"
                                    </th>
                                    <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        "Actions"
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                                // Machine rows would be rendered here
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
