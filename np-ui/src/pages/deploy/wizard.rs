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
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-foreground ">
                    "Deploy Configuration"
                </h1>
                <A
                    href="/"
                    attr:class="text-muted-foreground hover:text-foreground :text-gray-200"
                >
                    "Cancel"
                </A>
            </div>

            // Error display
            <Show when=move || deploy_error.get().is_some()>
                <div class="bg-red-50900/20 border border-red-200800 rounded-md p-4">
                    <p class="text-sm text-red-600400">
                        {move || deploy_error.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Target Configuration
                <Card title="Target Machine".to_string()>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-foreground ">
                                "Host"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                placeholder="192.168.1.100"
                                on:input=move |ev| set_target_host.set(event_target_value(&ev))
                                prop:value=move || target_host.get()
                            />
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-foreground ">
                                    "Port"
                                </label>
                                <input
                                    type="number"
                                    class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                    on:input=move |ev| set_target_port.set(event_target_value(&ev))
                                    prop:value=move || target_port.get()
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-foreground ">
                                    "Username"
                                </label>
                                <input
                                    type="text"
                                    class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                    on:input=move |ev| set_target_user.set(event_target_value(&ev))
                                    prop:value=move || target_user.get()
                                />
                            </div>
                        </div>
                    </div>
                </Card>

                // Flake Configuration
                <Card title="NixOS Configuration".to_string()>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-foreground ">
                                "Flake Reference"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring font-mono"
                                placeholder="github:user/repo or /path/to/flake"
                                on:input=move |ev| set_flake_ref.set(event_target_value(&ev))
                                prop:value=move || flake_ref.get()
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-foreground ">
                                "Configuration Name"
                            </label>
                            <input
                                type="text"
                                class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring font-mono"
                                placeholder="myhost"
                                on:input=move |ev| set_configuration.set(event_target_value(&ev))
                                prop:value=move || configuration.get()
                            />
                        </div>

                        // Preview
                        <div class="bg-muted  rounded-md p-3">
                            <p class="text-xs text-muted-foreground  mb-1">"Full reference:"</p>
                            <p class="font-mono text-sm text-foreground ">
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
                        <label class="block text-sm font-medium text-foreground  mb-3">
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
                                class="rounded border-border text-primary focus:ring-ring"
                                on:change=move |ev| set_build_on_target.set(event_target_checked(&ev))
                                prop:checked=move || build_on_target.get()
                            />
                            <div>
                                <span class="text-sm font-medium text-foreground ">
                                    "Build on target"
                                </span>
                                <p class="text-xs text-muted-foreground">"Build directly on the remote machine"</p>
                            </div>
                        </label>

                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                class="rounded border-border text-primary focus:ring-ring"
                                on:change=move |ev| set_use_substitutes.set(event_target_checked(&ev))
                                prop:checked=move || use_substitutes.get()
                            />
                            <div>
                                <span class="text-sm font-medium text-foreground ">
                                    "Use substitutes"
                                </span>
                                <p class="text-xs text-muted-foreground">"Download from binary caches"</p>
                            </div>
                        </label>

                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                class="rounded border-border text-primary focus:ring-ring"
                                on:change=move |ev| set_rollback_on_failure.set(event_target_checked(&ev))
                                prop:checked=move || rollback_on_failure.get()
                            />
                            <div>
                                <span class="text-sm font-medium text-foreground ">
                                    "Auto-rollback"
                                </span>
                                <p class="text-xs text-muted-foreground">"Rollback if activation fails"</p>
                            </div>
                        </label>
                    </div>
                </div>
            </Card>

            // Command Preview
            <Card title="Command Preview".to_string()>
                <div class="bg-background rounded-lg p-4 font-mono text-sm text-green-400">
                    <span class="text-muted-foreground">"$ "</span>
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
                    attr:class="px-6 py-3 text-sm font-medium text-foreground  hover:bg-muted  rounded-md"
                >
                    "Cancel"
                </A>
                <button
                    class="px-6 py-3 text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 rounded-md disabled:opacity-50 disabled:cursor-not-allowed"
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
                    "border-primary bg-muted "
                } else {
                    "border-border hover:border-border:border-gray-600"
                }
            )
            on:click=move |_| set_action.set(action)
        >
            <div class="font-medium text-foreground ">
                {action.as_str()}
            </div>
            <div class="text-xs text-muted-foreground  mt-1">
                {action.description()}
            </div>
        </button>
    }
}

/// Deployment progress component (shown during deployment)
#[component]
pub fn DeployProgressPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-foreground ">
                "Deployment in Progress"
            </h1>

            <Card>
                <div class="space-y-6">
                    // Progress indicator
                    <div class="text-center">
                        <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10  mb-4">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"/>
                        </div>
                        <h2 class="text-lg font-medium text-foreground ">
                            "Building configuration..."
                        </h2>
                    </div>

                    // Progress bar
                    <div class="w-full bg-muted  rounded-full h-2.5">
                        <div class="bg-primary h-2.5 rounded-full transition-all duration-500" style="width: 30%"/>
                    </div>

                    // Phase info
                    <div class="flex justify-between text-sm text-muted-foreground">
                        <span>"Phase: Building"</span>
                        <span>"30%"</span>
                    </div>

                    // Terminal output
                    <div class="bg-background rounded-lg p-4 font-mono text-sm text-green-400 h-64 overflow-auto">
                        <p>"$ nixos-rebuild switch --flake .#myhost --target-host root@server"</p>
                        <p class="text-muted-foreground">"building '/nix/store/xxx-nixos-system.drv'..."</p>
                        <p class="animate-pulse">"_"</p>
                    </div>
                </div>
            </Card>
        </div>
    }
}
