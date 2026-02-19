use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::common::Card;

/// Service detail page
#[component]
pub fn ServiceDetailPage() -> impl IntoView {
    let params = use_params_map();
    let machine_id = move || params.read().get("machine_id").unwrap_or_default();
    let service_name = move || params.read().get("service").unwrap_or_default();

    // Action state
    let (is_loading, set_is_loading) = signal(false);
    let (action_error, set_action_error) = signal(Option::<String>::None);

    // In a real app, this would fetch from the API
    let service_status = "active";
    let service_sub_state = "running";
    let service_enabled = true;
    let service_description = "OpenSSH Daemon";
    let main_pid = Some(1234u32);
    let memory_bytes = Some(45_000_000u64);
    let started_at = Some("2024-01-15 10:30:00 UTC");
    let unit_file_path = Some("/etc/systemd/system/sshd.service");

    // Dependencies (mock)
    let requires = vec!["network.target".to_string()];
    let wanted_by = vec!["multi-user.target".to_string()];
    let after = vec!["network.target".to_string(), "sshd-keygen.target".to_string()];
    let before = vec!["shutdown.target".to_string()];

    // Recent logs (mock)
    let recent_logs = vec![
        "2024-01-15T10:30:00+0000 sshd[1234]: Server listening on 0.0.0.0 port 22.".to_string(),
        "2024-01-15T10:30:00+0000 sshd[1234]: Server listening on :: port 22.".to_string(),
        "2024-01-15T10:35:22+0000 sshd[1234]: Accepted publickey for user from 192.168.1.100".to_string(),
    ];

    let format_bytes = move |bytes: u64| {
        if bytes >= 1_000_000_000 {
            format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            format!("{:.2} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.2} KB", bytes as f64 / 1_000.0)
        } else {
            format!("{} B", bytes)
        }
    };

    let perform_action = move |action: &'static str| {
        set_is_loading.set(true);
        set_action_error.set(None);
        // In a real app, this would call the API
        // For now, just simulate
        set_is_loading.set(false);
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href=move || format!("/services/{}", machine_id())
                        attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        "<- Back to Services"
                    </A>
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 font-mono">
                            {service_name}
                        </h1>
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            {service_description}
                        </p>
                    </div>
                </div>
                <div class="flex items-center space-x-3">
                    // Status badge
                    <span class=format!(
                        "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {}",
                        match service_status {
                            "active" => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
                            "inactive" => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
                            "failed" => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                            _ => "bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400",
                        }
                    )>
                        {service_status} " (" {service_sub_state} ")"
                    </span>
                </div>
            </div>

            // Error display
            <Show when=move || action_error.get().is_some()>
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4">
                    <p class="text-sm text-red-600 dark:text-red-400">
                        {move || action_error.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>

            // Quick Actions
            <Card title="Actions".to_string()>
                <div class="flex flex-wrap gap-3">
                    <button
                        class="inline-flex items-center px-4 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                        disabled=move || is_loading.get() || service_status == "active"
                        on:click=move |_| perform_action("start")
                    >
                        "Start"
                    </button>
                    <button
                        class="inline-flex items-center px-4 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                        disabled=move || is_loading.get() || service_status != "active"
                        on:click=move |_| perform_action("stop")
                    >
                        "Stop"
                    </button>
                    <button
                        class="inline-flex items-center px-4 py-2 bg-yellow-600 hover:bg-yellow-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                        disabled=move || is_loading.get() || service_status != "active"
                        on:click=move |_| perform_action("restart")
                    >
                        "Restart"
                    </button>
                    <button
                        class="inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                        disabled=move || is_loading.get() || service_status != "active"
                        on:click=move |_| perform_action("reload")
                    >
                        "Reload"
                    </button>
                    <div class="border-l border-gray-300 dark:border-gray-600 mx-2"></div>
                    <button
                        class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                        disabled=move || is_loading.get() || service_enabled
                        on:click=move |_| perform_action("enable")
                    >
                        "Enable"
                    </button>
                    <button
                        class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors"
                        disabled=move || is_loading.get() || !service_enabled
                        on:click=move |_| perform_action("disable")
                    >
                        "Disable"
                    </button>
                </div>
            </Card>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Service Info
                <Card title="Service Info".to_string()>
                    <dl class="space-y-4">
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"State"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                {service_status} " (" {service_sub_state} ")"
                            </dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Enabled"</dt>
                            <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                {if service_enabled { "Yes" } else { "No" }}
                            </dd>
                        </div>
                        {main_pid.map(|pid| view! {
                            <div>
                                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Main PID"</dt>
                                <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono">
                                    {pid}
                                </dd>
                            </div>
                        })}
                        {memory_bytes.map(|bytes| view! {
                            <div>
                                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Memory"</dt>
                                <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                    {format_bytes(bytes)}
                                </dd>
                            </div>
                        })}
                        {started_at.map(|time| view! {
                            <div>
                                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Started At"</dt>
                                <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100">
                                    {time}
                                </dd>
                            </div>
                        })}
                        {unit_file_path.map(|path| view! {
                            <div>
                                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">"Unit File"</dt>
                                <dd class="mt-1 text-sm text-gray-900 dark:text-gray-100 font-mono break-all">
                                    {path}
                                </dd>
                            </div>
                        })}
                    </dl>
                </Card>

                // Dependencies
                <Card title="Dependencies".to_string()>
                    <div class="space-y-4">
                        <div>
                            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">"Requires"</h4>
                            <div class="flex flex-wrap gap-2">
                                <For
                                    each=move || requires.clone()
                                    key=|d| d.clone()
                                    let:dep
                                >
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 font-mono">
                                        {dep}
                                    </span>
                                </For>
                            </div>
                        </div>
                        <div>
                            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">"Wanted By"</h4>
                            <div class="flex flex-wrap gap-2">
                                <For
                                    each=move || wanted_by.clone()
                                    key=|d| d.clone()
                                    let:dep
                                >
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 font-mono">
                                        {dep}
                                    </span>
                                </For>
                            </div>
                        </div>
                        <div>
                            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">"After"</h4>
                            <div class="flex flex-wrap gap-2">
                                <For
                                    each=move || after.clone()
                                    key=|d| d.clone()
                                    let:dep
                                >
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200 font-mono">
                                        {dep}
                                    </span>
                                </For>
                            </div>
                        </div>
                        <div>
                            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">"Before"</h4>
                            <div class="flex flex-wrap gap-2">
                                <For
                                    each=move || before.clone()
                                    key=|d| d.clone()
                                    let:dep
                                >
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200 font-mono">
                                        {dep}
                                    </span>
                                </For>
                            </div>
                        </div>
                    </div>
                </Card>
            </div>

            // Recent Logs
            <Card title="Recent Logs".to_string()>
                <div class="space-y-3">
                    <div class="flex items-center justify-between">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Last " {recent_logs.len()} " log entries"
                        </p>
                        <A
                            href=move || format!("/services/{}/{}/logs", machine_id(), service_name())
                            attr:class="text-sm text-indigo-600 hover:text-indigo-500 dark:text-indigo-400"
                        >
                            "View Full Logs ->"
                        </A>
                    </div>
                    <div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400 max-h-64 overflow-auto">
                        <For
                            each=move || recent_logs.clone()
                            key=|l| l.clone()
                            let:log
                        >
                            <p class="whitespace-pre-wrap break-all">{log}</p>
                        </For>
                    </div>
                </div>
            </Card>
        </div>
    }
}
