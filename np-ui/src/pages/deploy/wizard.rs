use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Deploy action type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployActionType {
    Switch,
    Boot,
    Test,
    Build,
    DryBuild,
}

impl DeployActionType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Switch => "switch",
            Self::Boot => "boot",
            Self::Test => "test",
            Self::Build => "build",
            Self::DryBuild => "dry-build",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Switch => "Build, activate, and add to boot menu",
            Self::Boot => "Build and add to boot menu (activate on reboot)",
            Self::Test => "Build and activate (don't add to boot menu)",
            Self::Build => "Build only (don't activate)",
            Self::DryBuild => "Show what would be built (dry run)",
        }
    }
}

/// Deploy wizard page
#[component]
pub fn DeployWizardPage() -> impl IntoView {
    // Form state
    let (target_host, set_target_host) = signal(String::new());
    let (target_port, set_target_port) = signal("22".to_string());
    let (target_user, set_target_user) = signal("root".to_string());
    let (flake_ref, set_flake_ref) = signal(String::new());
    let (configuration, set_configuration) = signal(String::new());
    let (action, set_action) = signal(DeployActionType::Switch);
    let (build_on_target, set_build_on_target) = signal(false);
    let (use_substitutes, set_use_substitutes) = signal(true);
    let (rollback_on_failure, set_rollback_on_failure) = signal(true);

    // Deployment state
    let (is_deploying, set_is_deploying) = signal(false);
    let (deploy_error, set_deploy_error) = signal(Option::<String>::None);

    let can_deploy = move || {
        !target_host.get().is_empty()
            && !flake_ref.get().is_empty()
            && !configuration.get().is_empty()
            && !is_deploying.get()
    };

    let start_deploy = move |_| {
        set_is_deploying.set(true);
        set_deploy_error.set(None);
        // In a real app, this would call the API
    };

    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Deploy Configuration"
                </h1>
                <A
                    href="/"
                    attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Cancel"
                </A>
            </div>

            // Error display
            <Show when=move || deploy_error.get().is_some()>
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4">
                    <p class="text-sm text-red-600 dark:text-red-400">
                        {move || deploy_error.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Target Configuration
                <Card title="Target Machine".to_string()>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Host"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                placeholder="192.168.1.100"
                                prop:value=target_host
                                on:input=move |ev| set_target_host.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Port"
                                </label>
                                <input
                                    type="number"
                                    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                    prop:value=target_port
                                    on:input=move |ev| set_target_port.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Username"
                                </label>
                                <input
                                    type="text"
                                    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                    prop:value=target_user
                                    on:input=move |ev| set_target_user.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </div>
                </Card>

                // Flake Configuration
                <Card title="NixOS Configuration".to_string()>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Flake Reference"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm font-mono"
                                placeholder="github:user/repo or /path/to/flake"
                                prop:value=flake_ref
                                on:input=move |ev| set_flake_ref.set(event_target_value(&ev))
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Configuration Name"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm font-mono"
                                placeholder="myhost"
                                prop:value=configuration
                                on:input=move |ev| set_configuration.set(event_target_value(&ev))
                            />
                        </div>

                        // Preview
                        <div class="bg-gray-50 dark:bg-gray-800 rounded-md p-3">
                            <p class="text-xs text-gray-500 dark:text-gray-400 mb-1">"Full reference:"</p>
                            <p class="font-mono text-sm text-gray-700 dark:text-gray-300">
                                {move || {
                                    let f = flake_ref.get();
                                    let c = configuration.get();
                                    if f.is_empty() || c.is_empty() {
                                        ".#nixosConfigurations.hostname".to_string()
                                    } else {
                                        format!("{}#nixosConfigurations.{}", f, c)
                                    }
                                }}
                            </p>
                        </div>
                    </div>
                </Card>
            </div>

            // Deployment Options
            <Card title="Deployment Options".to_string()>
                <div class="space-y-6">
                    // Action selection
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                            "Deployment Action"
                        </label>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                            <ActionOption
                                action=DeployActionType::Switch
                                current=action
                                set_action=set_action
                            />
                            <ActionOption
                                action=DeployActionType::Boot
                                current=action
                                set_action=set_action
                            />
                            <ActionOption
                                action=DeployActionType::Test
                                current=action
                                set_action=set_action
                            />
                            <ActionOption
                                action=DeployActionType::Build
                                current=action
                                set_action=set_action
                            />
                            <ActionOption
                                action=DeployActionType::DryBuild
                                current=action
                                set_action=set_action
                            />
                        </div>
                    </div>

                    // Advanced options
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                                prop:checked=build_on_target
                                on:change=move |ev| set_build_on_target.set(event_target_checked(&ev))
                            />
                            <div>
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Build on target"
                                </span>
                                <p class="text-xs text-gray-500">"Build directly on the remote machine"</p>
                            </div>
                        </label>

                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                                prop:checked=use_substitutes
                                on:change=move |ev| set_use_substitutes.set(event_target_checked(&ev))
                            />
                            <div>
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Use substitutes"
                                </span>
                                <p class="text-xs text-gray-500">"Download from binary caches"</p>
                            </div>
                        </label>

                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                                prop:checked=rollback_on_failure
                                on:change=move |ev| set_rollback_on_failure.set(event_target_checked(&ev))
                            />
                            <div>
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    "Auto-rollback"
                                </span>
                                <p class="text-xs text-gray-500">"Rollback if activation fails"</p>
                            </div>
                        </label>
                    </div>
                </div>
            </Card>

            // Command Preview
            <Card title="Command Preview".to_string()>
                <div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400">
                    <span class="text-gray-500">"$ "</span>
                    <span>"nixos-rebuild "</span>
                    <span class="text-yellow-400">{move || action.get().as_str()}</span>
                    <span>" --flake "</span>
                    <span class="text-blue-400">
                        {move || {
                            let f = flake_ref.get();
                            let c = configuration.get();
                            let flake_part = if f.is_empty() { ".".to_string() } else { f };
                            let config_part = if c.is_empty() { "hostname".to_string() } else { c };
                            format!("{}#nixosConfigurations.{}", flake_part, config_part)
                        }}
                    </span>
                    <span>" --target-host "</span>
                    <span class="text-purple-400">
                        {move || format!("{}@{}", target_user.get(), target_host.get())}
                    </span>
                    {move || build_on_target.get().then(|| view! {
                        <span>" --build-host "</span>
                        <span class="text-purple-400">
                            {format!("{}@{}", target_user.get(), target_host.get())}
                        </span>
                    })}
                    {move || (!use_substitutes.get()).then(|| view! {
                        <span>" --no-substitutes"</span>
                    })}
                </div>
            </Card>

            // Deploy button
            <div class="flex justify-end space-x-4">
                <A
                    href="/"
                    attr:class="px-6 py-3 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md"
                >
                    "Cancel"
                </A>
                <button
                    class="px-6 py-3 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled=move || !can_deploy()
                    on:click=start_deploy
                >
                    {move || if is_deploying.get() {
                        "Deploying..."
                    } else {
                        "Start Deployment"
                    }}
                </button>
            </div>
        </div>
    }
}

/// Action option component
#[component]
fn ActionOption(
    action: DeployActionType,
    current: ReadSignal<DeployActionType>,
    set_action: WriteSignal<DeployActionType>,
) -> impl IntoView {
    let is_selected = move || current.get() == action;

    view! {
        <button
            type="button"
            class=move || format!(
                "p-3 rounded-lg border-2 text-left transition-colors {}",
                if is_selected() {
                    "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
                } else {
                    "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                }
            )
            on:click=move |_| set_action.set(action)
        >
            <div class="font-medium text-gray-900 dark:text-gray-100">
                {action.as_str()}
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {action.description()}
            </div>
        </button>
    }
}

/// Deployment progress component (shown during deployment)
#[component]
pub fn DeployProgressPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                "Deployment in Progress"
            </h1>

            <Card>
                <div class="space-y-6">
                    // Progress indicator
                    <div class="text-center">
                        <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-indigo-100 dark:bg-indigo-900 mb-4">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"/>
                        </div>
                        <h2 class="text-lg font-medium text-gray-900 dark:text-gray-100">
                            "Building configuration..."
                        </h2>
                    </div>

                    // Progress bar
                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
                        <div class="bg-indigo-600 h-2.5 rounded-full transition-all duration-500" style="width: 30%"/>
                    </div>

                    // Phase info
                    <div class="flex justify-between text-sm text-gray-500">
                        <span>"Phase: Building"</span>
                        <span>"30%"</span>
                    </div>

                    // Terminal output
                    <div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400 h-64 overflow-auto">
                        <p>"$ nixos-rebuild switch --flake .#myhost --target-host root@server"</p>
                        <p class="text-gray-400">"building '/nix/store/xxx-nixos-system.drv'..."</p>
                        <p class="animate-pulse">"_"</p>
                    </div>
                </div>
            </Card>
        </div>
    }
}
