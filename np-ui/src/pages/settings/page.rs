use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;
use crate::components::icons::*;

fn load_settings() -> (String, String, bool, String) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let api_url = storage.get_item("np_api_url").ok().flatten().unwrap_or_else(|| "http://localhost:3000".to_string());
            let theme = storage.get_item("np_theme").ok().flatten().unwrap_or_else(|| "system".to_string());
            let auto_refresh = storage.get_item("np_auto_refresh").ok().flatten().map(|v| v == "true").unwrap_or(true);
            let refresh_interval = storage.get_item("np_refresh_interval").ok().flatten().unwrap_or_else(|| "30".to_string());
            return (api_url, theme, auto_refresh, refresh_interval);
        }
    }
    ("http://localhost:3000".to_string(), "system".to_string(), true, "30".to_string())
}

fn save_settings(api_url: &str, theme: &str, auto_refresh: bool, refresh_interval: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;

    storage.set_item("np_api_url", api_url).map_err(|_| "Failed to save")?;
    storage.set_item("np_theme", theme).map_err(|_| "Failed to save")?;
    storage.set_item("np_auto_refresh", &auto_refresh.to_string()).map_err(|_| "Failed to save")?;
    storage.set_item("np_refresh_interval", refresh_interval).map_err(|_| "Failed to save")?;

    Ok(())
}

/// Settings page
#[component]
pub fn SettingsPage() -> impl IntoView {
    let (saved_api_url, saved_theme, saved_auto_refresh, saved_refresh_interval) = load_settings();

    let (api_url, set_api_url) = signal(saved_api_url);
    let (theme, set_theme) = signal(saved_theme);
    let (auto_refresh, set_auto_refresh) = signal(saved_auto_refresh);
    let (refresh_interval, set_refresh_interval) = signal(saved_refresh_interval);
    let (saving, set_saving) = signal(false);
    let (save_msg, set_save_msg) = signal(Option::<String>::None);

    let on_save = move |_| {
        set_saving.set(true);
        match save_settings(&api_url.get(), &theme.get(), auto_refresh.get(), &refresh_interval.get()) {
            Ok(()) => {
                set_save_msg.set(Some("Settings saved successfully".to_string()));
                set_saving.set(false);
            }
            Err(e) => {
                set_save_msg.set(Some(format!("Failed to save: {}", e)));
                set_saving.set(false);
            }
        }
    };

    view! {
        <div class="space-y-6">
            // Header
            <div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Settings"
                </h1>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                    "Configure Nix Pilot preferences"
                </p>
            </div>

            {move || save_msg.get().map(|msg| view! {
                <div class="p-3 bg-green-100 border border-green-400 text-green-700 rounded">
                    {msg}
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // API Configuration
                <Card title="API Configuration".to_string()>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "API URL"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                prop:value=api_url
                                on:input=move |ev| set_api_url.set(event_target_value(&ev))
                            />
                            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                "The URL of the Nix Pilot API server"
                            </p>
                        </div>

                        <div class="flex items-center justify-between">
                            <div>
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Connection Status"
                                </span>
                            </div>
                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                                <IconStatusDot size=IconSize::Sm color="text-green-500".to_string() />
                                <span class="ml-1">"Connected"</span>
                            </span>
                        </div>
                    </div>
                </Card>

                // Appearance
                <Card title="Appearance".to_string()>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Theme"
                            </label>
                            <select
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                prop:value=theme
                                on:change=move |ev| set_theme.set(event_target_value(&ev))
                            >
                                <option value="system">"System"</option>
                                <option value="light">"Light"</option>
                                <option value="dark">"Dark"</option>
                            </select>
                        </div>
                    </div>
                </Card>

                // Data Refresh
                <Card title="Data Refresh".to_string()>
                    <div class="space-y-4">
                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                                prop:checked=auto_refresh
                                on:change=move |ev| set_auto_refresh.set(event_target_checked(&ev))
                            />
                            <div>
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Auto-refresh data"
                                </span>
                                <p class="text-xs text-gray-500 dark:text-gray-400">
                                    "Automatically refresh machine status and service data"
                                </p>
                            </div>
                        </label>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Refresh Interval (seconds)"
                            </label>
                            <input
                                type="number"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                prop:value=refresh_interval
                                on:input=move |ev| set_refresh_interval.set(event_target_value(&ev))
                                disabled=move || !auto_refresh.get()
                            />
                        </div>
                    </div>
                </Card>

                // SSH Keys Link
                <Card title="SSH Keys".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Manage SSH keys for connecting to remote machines."
                        </p>
                        <A
                            href="/secrets/keys"
                            attr:class="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600"
                        >
                            <span class="mr-2"><IconKey size=IconSize::Sm /></span>
                            "Manage SSH Keys"
                        </A>
                    </div>
                </Card>

                // Age Keys Link
                <Card title="Encryption Keys".to_string()>
                    <div class="space-y-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Manage age keys for SOPS secret encryption."
                        </p>
                        <A
                            href="/secrets/keys"
                            attr:class="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600"
                        >
                            <span class="mr-2"><IconShield size=IconSize::Sm /></span>
                            "Manage Age Keys"
                        </A>
                    </div>
                </Card>

                // About
                <Card title="About".to_string()>
                    <div class="space-y-3">
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-500 dark:text-gray-400">"Version"</span>
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"0.1.0"</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-500 dark:text-gray-400">"Build"</span>
                            <span class="text-sm font-mono text-gray-900 dark:text-gray-100">"dev"</span>
                        </div>
                        <div class="pt-3 border-t border-gray-200 dark:border-gray-700">
                            <p class="text-xs text-gray-500 dark:text-gray-400">
                                "Nix Pilot - A web UI for managing NixOS deployments, flakes, and services."
                            </p>
                        </div>
                    </div>
                </Card>
            </div>

            // Save button
            <div class="flex justify-end">
                <button
                    class="px-6 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md disabled:opacity-50"
                    on:click=on_save
                    disabled=move || saving.get()
                >
                    {move || if saving.get() { "Saving..." } else { "Save Settings" }}
                </button>
            </div>
        </div>
    }
}
