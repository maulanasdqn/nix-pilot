use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;

use crate::components::common::Card;

/// Service logs page with live streaming
#[component]
pub fn ServiceLogsPage() -> impl IntoView {
    let params = use_params_map();
    let machine_id = move || params.read().get("machine_id").unwrap_or_default();
    let service_name = move || params.read().get("service").unwrap_or_default();

    // Log state
    let (logs, set_logs) = signal(Vec::<String>::new());
    let (is_streaming, set_is_streaming) = signal(false);
    let (auto_scroll, set_auto_scroll) = signal(true);
    let (filter, set_filter) = signal(String::new());

    // Log options
    let (lines_to_fetch, set_lines_to_fetch) = signal(100u32);

    // Mock logs for demonstration
    let mock_logs = vec![
        "2024-01-15T10:30:00+0000 sshd[1234]: Server listening on 0.0.0.0 port 22.".to_string(),
        "2024-01-15T10:30:00+0000 sshd[1234]: Server listening on :: port 22.".to_string(),
        "2024-01-15T10:35:22+0000 sshd[1234]: Accepted publickey for user from 192.168.1.100 port 54321 ssh2: RSA SHA256:xxxxx".to_string(),
        "2024-01-15T10:35:22+0000 sshd[1234]: pam_unix(sshd:session): session opened for user user(uid=1000) by (uid=0)".to_string(),
        "2024-01-15T10:40:15+0000 sshd[1234]: Received disconnect from 192.168.1.100 port 54321:11: disconnected by user".to_string(),
        "2024-01-15T10:40:15+0000 sshd[1234]: Disconnected from user user 192.168.1.100 port 54321".to_string(),
        "2024-01-15T10:40:15+0000 sshd[1234]: pam_unix(sshd:session): session closed for user user".to_string(),
        "2024-01-15T11:00:00+0000 sshd[1234]: Accepted publickey for admin from 10.0.0.50 port 12345 ssh2: ED25519 SHA256:yyyyy".to_string(),
    ];

    // Load initial logs
    Effect::new(move |_| {
        set_logs.set(mock_logs.clone());
    });

    let toggle_streaming = move |_| {
        let current = is_streaming.get();
        set_is_streaming.set(!current);

        if !current {
            // Would connect to WebSocket here
            // For demo, just append mock logs periodically
        }
    };

    let clear_logs = move |_| {
        set_logs.set(Vec::new());
    };

    let download_logs = move |_| {
        let current_logs = logs.get();
        if current_logs.is_empty() {
            return;
        }

        let content = current_logs.join("\n");
        let service = service_name();
        let filename = format!("{}-logs.txt", service);

        // Create blob and download
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&wasm_bindgen::JsValue::from_str(&content));

                if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&blob_parts) {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        if let Ok(a) = document.create_element("a") {
                            let _ = a.set_attribute("href", &url);
                            let _ = a.set_attribute("download", &filename);
                            if let Some(html_element) = a.dyn_ref::<web_sys::HtmlElement>() {
                                html_element.click();
                            }
                            let _ = web_sys::Url::revoke_object_url(&url);
                        }
                    }
                }
            }
        }
    };

    let filtered_logs = move || {
        let search = filter.get().to_lowercase();
        if search.is_empty() {
            logs.get()
        } else {
            logs.get()
                .into_iter()
                .filter(|line| line.to_lowercase().contains(&search))
                .collect()
        }
    };

    // Log line coloring based on content
    let get_log_class = |line: &str| -> &'static str {
        if line.contains("error") || line.contains("Error") || line.contains("ERROR") || line.contains("failed") {
            "text-red-400"
        } else if line.contains("warn") || line.contains("Warn") || line.contains("WARNING") {
            "text-yellow-400"
        } else if line.contains("debug") || line.contains("Debug") || line.contains("DEBUG") {
            "text-gray-500"
        } else {
            "text-green-400"
        }
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <A
                        href=move || format!("/services/{}/{}", machine_id(), service_name())
                        attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        "<- Back to Service"
                    </A>
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            "Service Logs"
                        </h1>
                        <p class="text-sm text-gray-500 dark:text-gray-400 font-mono">
                            {service_name}
                        </p>
                    </div>
                </div>
                <div class="flex items-center space-x-3">
                    // Streaming indicator
                    <Show when=move || is_streaming.get()>
                        <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                            <span class="w-2 h-2 mr-2 bg-green-500 rounded-full animate-pulse"></span>
                            "Live"
                        </span>
                    </Show>
                </div>
            </div>

            // Controls
            <Card>
                <div class="flex flex-col md:flex-row gap-4">
                    // Filter
                    <div class="flex-1">
                        <input
                            type="text"
                            placeholder="Filter logs..."
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-indigo-500 focus:border-indigo-500 font-mono text-sm"
                            prop:value=filter
                            on:input=move |ev| set_filter.set(event_target_value(&ev))
                        />
                    </div>

                    // Lines selector
                    <div class="flex items-center space-x-2">
                        <label class="text-sm text-gray-500 dark:text-gray-400">"Lines:"</label>
                        <select
                            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 text-sm"
                            prop:value=move || lines_to_fetch.get().to_string()
                            on:change=move |ev| {
                                if let Ok(n) = event_target_value(&ev).parse() {
                                    set_lines_to_fetch.set(n);
                                }
                            }
                        >
                            <option value="50">"50"</option>
                            <option value="100">"100"</option>
                            <option value="250">"250"</option>
                            <option value="500">"500"</option>
                            <option value="1000">"1000"</option>
                        </select>
                    </div>

                    // Auto-scroll toggle
                    <label class="flex items-center space-x-2 cursor-pointer">
                        <input
                            type="checkbox"
                            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                            prop:checked=auto_scroll
                            on:change=move |ev| set_auto_scroll.set(event_target_checked(&ev))
                        />
                        <span class="text-sm text-gray-700 dark:text-gray-300">"Auto-scroll"</span>
                    </label>

                    // Action buttons
                    <div class="flex items-center space-x-2">
                        <button
                            class=move || format!(
                                "inline-flex items-center px-4 py-2 text-sm font-medium rounded-md transition-colors {}",
                                if is_streaming.get() {
                                    "bg-red-600 hover:bg-red-700 text-white"
                                } else {
                                    "bg-green-600 hover:bg-green-700 text-white"
                                }
                            )
                            on:click=toggle_streaming
                        >
                            {move || if is_streaming.get() { "Stop" } else { "Stream" }}
                        </button>
                        <button
                            class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md transition-colors"
                            on:click=clear_logs
                        >
                            "Clear"
                        </button>
                        <button
                            class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                            on:click=download_logs
                        >
                            "Download"
                        </button>
                    </div>
                </div>
            </Card>

            // Log viewer
            <div class="bg-gray-900 rounded-lg shadow-lg overflow-hidden">
                // Log header
                <div class="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
                    <span class="text-sm text-gray-400">
                        {move || format!("{} lines", filtered_logs().len())}
                        {move || {
                            let total = logs.get().len();
                            let filtered = filtered_logs().len();
                            if filtered < total {
                                format!(" (filtered from {})", total)
                            } else {
                                String::new()
                            }
                        }}
                    </span>
                    <span class="text-sm text-gray-500">
                        "journalctl -u " {service_name} " -n " {move || lines_to_fetch.get()}
                    </span>
                </div>

                // Log content
                <div
                    class="p-4 font-mono text-sm overflow-auto"
                    style="max-height: 600px; min-height: 400px;"
                >
                    <Show
                        when=move || !filtered_logs().is_empty()
                        fallback=|| view! {
                            <p class="text-gray-500 text-center py-8">
                                "No log entries to display."
                            </p>
                        }
                    >
                        <For
                            each=filtered_logs
                            key=|l| l.clone()
                            let:log
                        >
                            {
                                let log_class = get_log_class(&log);
                                view! {
                                    <p class=format!("whitespace-pre-wrap break-all py-0.5 hover:bg-gray-800 {}", log_class)>
                                        {log}
                                    </p>
                                }
                            }
                        </For>
                    </Show>
                </div>
            </div>

            // Footer info
            <div class="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400">
                <span>
                    "Machine: " {machine_id}
                </span>
                <span>
                    "Service: " {service_name}
                </span>
            </div>
        </div>
    }
}
