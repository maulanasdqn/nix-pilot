use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::common::Card;

/// Service state for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Unknown,
}

impl ServiceState {
    fn badge_class(&self) -> &'static str {
        match self {
            Self::Active => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
            Self::Inactive => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
            Self::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
            Self::Activating | Self::Deactivating => {
                "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
            }
            Self::Unknown => "bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400",
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Failed => "failed",
            Self::Activating => "activating",
            Self::Deactivating => "deactivating",
            Self::Unknown => "unknown",
        }
    }
}

/// Mock service info for placeholder
#[derive(Clone)]
struct MockServiceInfo {
    name: String,
    description: Option<String>,
    state: ServiceState,
    sub_state: String,
    enabled: bool,
}

/// Service list page for a machine
#[component]
pub fn ServiceListPage() -> impl IntoView {
    let params = use_params_map();
    let machine_id = move || params.read().get("machine_id").unwrap_or_default();

    // Filter state
    let (filter, set_filter) = signal(String::new());
    let (show_only_failed, set_show_only_failed) = signal(false);
    let (show_only_active, set_show_only_active) = signal(false);

    // In a real app, this would fetch from the API
    // For now, we'll show some mock services
    let services = vec![
        MockServiceInfo {
            name: "sshd.service".to_string(),
            description: Some("OpenSSH Daemon".to_string()),
            state: ServiceState::Active,
            sub_state: "running".to_string(),
            enabled: true,
        },
        MockServiceInfo {
            name: "nginx.service".to_string(),
            description: Some("A high performance web server".to_string()),
            state: ServiceState::Active,
            sub_state: "running".to_string(),
            enabled: true,
        },
        MockServiceInfo {
            name: "postgresql.service".to_string(),
            description: Some("PostgreSQL database server".to_string()),
            state: ServiceState::Inactive,
            sub_state: "dead".to_string(),
            enabled: false,
        },
        MockServiceInfo {
            name: "failed-example.service".to_string(),
            description: Some("Example failed service".to_string()),
            state: ServiceState::Failed,
            sub_state: "failed".to_string(),
            enabled: true,
        },
    ];

    let total_services = services.len();
    let services_for_filter = services.clone();

    let filtered_services = move || {
        let search = filter.get().to_lowercase();
        let only_failed = show_only_failed.get();
        let only_active = show_only_active.get();

        services_for_filter
            .iter()
            .filter(|s| {
                let matches_search = search.is_empty()
                    || s.name.to_lowercase().contains(&search)
                    || s.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search))
                        .unwrap_or(false);

                let matches_failed = !only_failed || s.state == ServiceState::Failed;
                let matches_active = !only_active || s.state == ServiceState::Active;

                matches_search && matches_failed && matches_active
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    // Clone for different usages in the view
    let filtered_services_for_table = filtered_services.clone();
    let filtered_services_for_empty = filtered_services.clone();
    let filtered_services_for_count = filtered_services.clone();

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href=move || format!("/machines/{}", machine_id())
                        attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        "<- Back to Machine"
                    </A>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        "Services"
                    </h1>
                </div>
                <button
                    class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md transition-colors"
                    title="Refresh service list"
                >
                    "Refresh"
                </button>
            </div>

            // Filters
            <Card>
                <div class="flex flex-col sm:flex-row gap-4">
                    <div class="flex-1">
                        <input
                            type="text"
                            placeholder="Filter services..."
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500"
                            prop:value=filter
                            on:input=move |ev| set_filter.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex items-center space-x-4">
                        <label class="flex items-center space-x-2 cursor-pointer">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-red-600 focus:ring-red-500"
                                prop:checked=show_only_failed
                                on:change=move |ev| set_show_only_failed.set(event_target_checked(&ev))
                            />
                            <span class="text-sm text-gray-700 dark:text-gray-300">"Failed only"</span>
                        </label>
                        <label class="flex items-center space-x-2 cursor-pointer">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-green-600 focus:ring-green-500"
                                prop:checked=show_only_active
                                on:change=move |ev| set_show_only_active.set(event_target_checked(&ev))
                            />
                            <span class="text-sm text-gray-700 dark:text-gray-300">"Active only"</span>
                        </label>
                    </div>
                </div>
            </Card>

            // Service list
            <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead class="bg-gray-50 dark:bg-gray-900">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                "Service"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                "Status"
                            </th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                "Enabled"
                            </th>
                            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                "Actions"
                            </th>
                        </tr>
                    </thead>
                    <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                        <For
                            each=filtered_services_for_table
                            key=|s| s.name.clone()
                            let:service
                        >
                            <ServiceRow
                                machine_id=machine_id()
                                name=service.name.clone()
                                description=service.description.clone()
                                state=service.state
                                sub_state=service.sub_state.clone()
                                enabled=service.enabled
                            />
                        </For>
                    </tbody>
                </table>

                // Empty state
                <Show when=move || filtered_services_for_empty().is_empty()>
                    <div class="text-center py-12">
                        <p class="text-gray-500 dark:text-gray-400">
                            "No services match your filter criteria."
                        </p>
                    </div>
                </Show>
            </div>

            // Summary
            <div class="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400">
                <span>
                    "Showing " {move || filtered_services_for_count().len()} " of " {total_services} " services"
                </span>
                <span>
                    "Machine: " {machine_id}
                </span>
            </div>
        </div>
    }
}

/// Service row component
#[component]
fn ServiceRow(
    #[prop(into)] machine_id: String,
    #[prop(into)] name: String,
    #[prop(into)] description: Option<String>,
    state: ServiceState,
    #[prop(into)] sub_state: String,
    enabled: bool,
) -> impl IntoView {
    let detail_href = format!("/services/{}/{}", machine_id, name);
    let logs_href = format!("/services/{}/{}/logs", machine_id, name);
    let display_name = name.clone();

    view! {
        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-6 py-4">
                <div class="flex flex-col">
                    <A
                        href=detail_href
                        attr:class="text-sm font-medium text-indigo-600 dark:text-indigo-400 hover:text-indigo-500 font-mono"
                    >
                        {display_name}
                    </A>
                    {description.map(|d| view! {
                        <span class="text-sm text-gray-500 dark:text-gray-400">{d}</span>
                    })}
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex items-center space-x-2">
                    <span class=format!(
                        "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                        state.badge_class()
                    )>
                        {state.as_str()}
                    </span>
                    <span class="text-xs text-gray-500 dark:text-gray-400 font-mono">
                        "(" {sub_state} ")"
                    </span>
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <span class=format!(
                    "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                    if enabled {
                        "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
                    } else {
                        "bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400"
                    }
                )>
                    {if enabled { "enabled" } else { "disabled" }}
                </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div class="flex items-center justify-end space-x-2">
                    {match state {
                        ServiceState::Active => view! {
                            <button
                                class="text-yellow-600 hover:text-yellow-900 dark:text-yellow-400"
                                title="Restart service"
                            >
                                "Restart"
                            </button>
                            <button
                                class="text-red-600 hover:text-red-900 dark:text-red-400"
                                title="Stop service"
                            >
                                "Stop"
                            </button>
                        }.into_any(),
                        ServiceState::Inactive | ServiceState::Failed => view! {
                            <button
                                class="text-green-600 hover:text-green-900 dark:text-green-400"
                                title="Start service"
                            >
                                "Start"
                            </button>
                        }.into_any(),
                        _ => view! {
                            <span class="text-gray-400">"..."</span>
                        }.into_any(),
                    }}
                    <A
                        href=logs_href
                        attr:class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400"
                    >
                        "Logs"
                    </A>
                </div>
            </td>
        </tr>
    }
}
