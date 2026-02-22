use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Install wizard step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    SelectTarget,
    SelectFlake,
    ConfigureOptions,
    Review,
    Installing,
}

impl WizardStep {
    fn number(&self) -> u8 {
        match self {
            Self::SelectTarget => 1,
            Self::SelectFlake => 2,
            Self::ConfigureOptions => 3,
            Self::Review => 4,
            Self::Installing => 5,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::SelectTarget => "Select Target",
            Self::SelectFlake => "Select Configuration",
            Self::ConfigureOptions => "Options",
            Self::Review => "Review",
            Self::Installing => "Installing",
        }
    }
}

/// Install wizard page
#[component]
pub fn InstallWizardPage() -> impl IntoView {
    // Current step
    let (step, set_step) = signal(WizardStep::SelectTarget);

    // Form state
    let (target_host, set_target_host) = signal(String::new());
    let (target_port, set_target_port) = signal("22".to_string());
    let (target_user, set_target_user) = signal("root".to_string());
    let (auth_method, set_auth_method) = signal("agent".to_string());
    let (flake_ref, set_flake_ref) = signal(String::new());
    let (configuration, set_configuration) = signal(String::new());
    let (use_kexec, set_use_kexec) = signal(true);
    let (wipe_disks, set_wipe_disks) = signal(false);

    // Installation state
    let (job_id, set_job_id) = signal(Option::<String>::None);
    let (is_installing, set_is_installing) = signal(false);

    let can_proceed = move || {
        match step.get() {
            WizardStep::SelectTarget => !target_host.get().is_empty(),
            WizardStep::SelectFlake => !flake_ref.get().is_empty() && !configuration.get().is_empty(),
            WizardStep::ConfigureOptions => true,
            WizardStep::Review => true,
            WizardStep::Installing => false,
        }
    };

    let next_step = move |_| {
        let current = step.get();
        let next = match current {
            WizardStep::SelectTarget => WizardStep::SelectFlake,
            WizardStep::SelectFlake => WizardStep::ConfigureOptions,
            WizardStep::ConfigureOptions => WizardStep::Review,
            WizardStep::Review => WizardStep::Installing,
            WizardStep::Installing => WizardStep::Installing,
        };
        set_step.set(next);

        // Start installation when entering Installing step
        if next == WizardStep::Installing {
            set_is_installing.set(true);
            // In a real app, this would call the API to start the installation
        }
    };

    let prev_step = move |_| {
        let current = step.get();
        let prev = match current {
            WizardStep::SelectTarget => WizardStep::SelectTarget,
            WizardStep::SelectFlake => WizardStep::SelectTarget,
            WizardStep::ConfigureOptions => WizardStep::SelectFlake,
            WizardStep::Review => WizardStep::ConfigureOptions,
            WizardStep::Installing => WizardStep::Review,
        };
        set_step.set(prev);
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-foreground ">
                    "Install NixOS"
                </h1>
                <A
                    href="/"
                    attr:class="text-muted-foreground hover:text-foreground  "
                >
                    "Cancel"
                </A>
            </div>

            // Step indicator
            <WizardStepIndicator current_step=step />

            // Step content
            <Card>
                <div class="min-h-[400px]">
                    <Show when=move || step.get() == WizardStep::SelectTarget>
                        <SelectTargetStep
                            host=target_host
                            set_host=set_target_host
                            port=target_port
                            set_port=set_target_port
                            user=target_user
                            set_user=set_target_user
                            auth_method=auth_method
                            set_auth_method=set_auth_method
                        />
                    </Show>

                    <Show when=move || step.get() == WizardStep::SelectFlake>
                        <SelectFlakeStep
                            flake_ref=flake_ref
                            set_flake_ref=set_flake_ref
                            configuration=configuration
                            set_configuration=set_configuration
                        />
                    </Show>

                    <Show when=move || step.get() == WizardStep::ConfigureOptions>
                        <ConfigureOptionsStep
                            use_kexec=use_kexec
                            set_use_kexec=set_use_kexec
                            wipe_disks=wipe_disks
                            set_wipe_disks=set_wipe_disks
                        />
                    </Show>

                    <Show when=move || step.get() == WizardStep::Review>
                        <ReviewStep
                            host=target_host
                            port=target_port
                            user=target_user
                            flake_ref=flake_ref
                            configuration=configuration
                            use_kexec=use_kexec
                            wipe_disks=wipe_disks
                        />
                    </Show>

                    <Show when=move || step.get() == WizardStep::Installing>
                        <InstallingStep job_id=job_id />
                    </Show>
                </div>

                // Navigation buttons
                <div class="flex justify-between mt-6 pt-6 border-t border-border">
                    <button
                        class="px-4 py-2 text-sm font-medium text-foreground  hover:bg-muted  rounded-md disabled:opacity-50"
                        disabled=move || step.get() == WizardStep::SelectTarget || step.get() == WizardStep::Installing
                        on:click=prev_step
                    >
                        {"\u{2190} Previous"}
                    </button>

                    <button
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 rounded-md disabled:opacity-50"
                        disabled=move || !can_proceed() || step.get() == WizardStep::Installing
                        on:click=next_step
                    >
                        {move || if step.get() == WizardStep::Review { "Start Installation" } else { "Next \u{2192}" }}
                    </button>
                </div>
            </Card>
        </div>
    }
}

/// Step indicator component
#[component]
fn WizardStepIndicator(current_step: ReadSignal<WizardStep>) -> impl IntoView {
    let steps = vec![
        WizardStep::SelectTarget,
        WizardStep::SelectFlake,
        WizardStep::ConfigureOptions,
        WizardStep::Review,
        WizardStep::Installing,
    ];

    view! {
        <div class="flex items-center justify-center">
            {steps.into_iter().enumerate().map(|(idx, step)| {
                let is_current = move || current_step.get() == step;
                let is_completed = move || current_step.get().number() > step.number();

                view! {
                    <>
                        {if idx > 0 {
                            Some(view! {
                                <div class=move || format!(
                                    "w-12 h-1 mx-2 {}",
                                    if is_completed() { "bg-primary" } else { "bg-muted " }
                                )/>
                            })
                        } else {
                            None
                        }}

                        <div class="flex flex-col items-center">
                            <div class=move || format!(
                                "w-10 h-10 rounded-full flex items-center justify-center text-sm font-medium {}",
                                if is_current() {
                                    "bg-primary text-primary-foreground"
                                } else if is_completed() {
                                    "bg-primary text-primary-foreground"
                                } else {
                                    "bg-muted  text-foreground "
                                }
                            )>
                                {step.number()}
                            </div>
                            <span class="mt-2 text-xs text-muted-foreground ">
                                {step.name()}
                            </span>
                        </div>
                    </>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Step 1: Select target
#[component]
fn SelectTargetStep(
    host: ReadSignal<String>,
    set_host: WriteSignal<String>,
    port: ReadSignal<String>,
    set_port: WriteSignal<String>,
    user: ReadSignal<String>,
    set_user: WriteSignal<String>,
    auth_method: ReadSignal<String>,
    set_auth_method: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h2 class="text-lg font-medium text-foreground  mb-2">
                    "Target Machine"
                </h2>
                <p class="text-sm text-muted-foreground ">
                    "Enter the SSH connection details for the machine where you want to install NixOS."
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                    <label class="block text-sm font-medium text-foreground ">
                        "Host"
                    </label>
                    <input
                        type="text"
                        class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                        placeholder="192.168.1.100 or hostname.example.com"
                        on:input=move |ev| set_host.set(event_target_value(&ev))
                        prop:value=move || host.get()
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-foreground ">
                        "Port"
                    </label>
                    <input
                        type="number"
                        class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                        on:input=move |ev| set_port.set(event_target_value(&ev))
                        prop:value=move || port.get()
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-foreground ">
                        "Username"
                    </label>
                    <input
                        type="text"
                        class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                        on:input=move |ev| set_user.set(event_target_value(&ev))
                        prop:value=move || user.get()
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-foreground ">
                        "Authentication"
                    </label>
                    <select
                        class="mt-1 block w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                        on:change=move |ev| set_auth_method.set(event_target_value(&ev))
                        prop:value=move || auth_method.get()
                    >
                        <option value="agent">"SSH Agent"</option>
                        <option value="key">"SSH Key File"</option>
                    </select>
                </div>
            </div>

            <div class="bg-yellow-50900/20 border border-yellow-200800 rounded-md p-4">
                <h4 class="text-sm font-medium text-yellow-800200">
                    "Requirements"
                </h4>
                <ul class="mt-2 text-sm text-yellow-700300 list-disc list-inside space-y-1">
                    <li>"Target machine must be accessible via SSH"</li>
                    <li>"Root access required (or sudo without password)"</li>
                    <li>"Target should be booted into a Linux live environment (e.g., NixOS installer, SystemRescue)"</li>
                </ul>
            </div>
        </div>
    }
}

/// Step 2: Select flake
#[component]
fn SelectFlakeStep(
    flake_ref: ReadSignal<String>,
    set_flake_ref: WriteSignal<String>,
    configuration: ReadSignal<String>,
    set_configuration: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h2 class="text-lg font-medium text-foreground  mb-2">
                    "NixOS Configuration"
                </h2>
                <p class="text-sm text-muted-foreground ">
                    "Select the flake containing your NixOS configuration."
                </p>
            </div>

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
                    <p class="mt-1 text-sm text-muted-foreground">
                        "Can be a GitHub reference, local path, or any valid flake URL"
                    </p>
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
                    <p class="mt-1 text-sm text-muted-foreground">
                        "The name in nixosConfigurations (e.g., \"myhost\" for nixosConfigurations.myhost)"
                    </p>
                </div>
            </div>

            // Preview
            <div class="bg-muted  rounded-md p-4">
                <h4 class="text-sm font-medium text-foreground  mb-2">
                    "Full Flake Reference"
                </h4>
                <p class="font-mono text-sm text-muted-foreground ">
                    {move || {
                        let f = flake_ref.get();
                        let c = configuration.get();
                        if f.is_empty() || c.is_empty() {
                            "github:user/repo#nixosConfigurations.hostname".to_string()
                        } else {
                            format!("{}#nixosConfigurations.{}", f, c)
                        }
                    }}
                </p>
            </div>
        </div>
    }
}

/// Step 3: Configure options
#[component]
fn ConfigureOptionsStep(
    use_kexec: ReadSignal<bool>,
    set_use_kexec: WriteSignal<bool>,
    wipe_disks: ReadSignal<bool>,
    set_wipe_disks: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h2 class="text-lg font-medium text-foreground  mb-2">
                    "Installation Options"
                </h2>
                <p class="text-sm text-muted-foreground ">
                    "Configure how the installation should proceed."
                </p>
            </div>

            <div class="space-y-4">
                <label class="flex items-start space-x-3">
                    <input
                        type="checkbox"
                        class="mt-1 rounded border-border text-primary focus:ring-ring"
                        on:change=move |ev| set_use_kexec.set(event_target_checked(&ev))
                        prop:checked=move || use_kexec.get()
                    />
                    <div>
                        <span class="text-sm font-medium text-foreground ">
                            "Use kexec"
                        </span>
                        <p class="text-sm text-muted-foreground">
                            "Boot into the NixOS installer using kexec (faster, recommended)"
                        </p>
                    </div>
                </label>

                <label class="flex items-start space-x-3">
                    <input
                        type="checkbox"
                        class="mt-1 rounded border-border text-primary focus:ring-ring"
                        on:change=move |ev| set_wipe_disks.set(event_target_checked(&ev))
                        prop:checked=move || wipe_disks.get()
                    />
                    <div>
                        <span class="text-sm font-medium text-foreground ">
                            "Wipe disks"
                        </span>
                        <p class="text-sm text-muted-foreground">
                            "Completely wipe target disks before installation (DESTRUCTIVE)"
                        </p>
                    </div>
                </label>
            </div>

            <div class="bg-red-50900/20 border border-red-200800 rounded-md p-4">
                <h4 class="text-sm font-medium text-red-800200">
                    "Warning"
                </h4>
                <p class="mt-2 text-sm text-red-700300">
                    "This installation will format disks according to your disko configuration. All existing data on target disks will be permanently lost. Make sure you have backups of any important data."
                </p>
            </div>
        </div>
    }
}

/// Step 4: Review
#[component]
fn ReviewStep(
    host: ReadSignal<String>,
    port: ReadSignal<String>,
    user: ReadSignal<String>,
    flake_ref: ReadSignal<String>,
    configuration: ReadSignal<String>,
    use_kexec: ReadSignal<bool>,
    wipe_disks: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h2 class="text-lg font-medium text-foreground  mb-2">
                    "Review Installation"
                </h2>
                <p class="text-sm text-muted-foreground ">
                    "Please review the installation settings before proceeding."
                </p>
            </div>

            <dl class="divide-y divide-border">
                <div class="py-3 flex justify-between">
                    <dt class="text-sm font-medium text-muted-foreground ">"Target"</dt>
                    <dd class="text-sm text-foreground  font-mono">
                        {move || format!("{}@{}:{}", user.get(), host.get(), port.get())}
                    </dd>
                </div>
                <div class="py-3 flex justify-between">
                    <dt class="text-sm font-medium text-muted-foreground ">"Configuration"</dt>
                    <dd class="text-sm text-foreground  font-mono">
                        {move || format!("{}#nixosConfigurations.{}", flake_ref.get(), configuration.get())}
                    </dd>
                </div>
                <div class="py-3 flex justify-between">
                    <dt class="text-sm font-medium text-muted-foreground ">"Use kexec"</dt>
                    <dd class="text-sm text-foreground ">
                        {move || if use_kexec.get() { "Yes" } else { "No" }}
                    </dd>
                </div>
                <div class="py-3 flex justify-between">
                    <dt class="text-sm font-medium text-muted-foreground ">"Wipe disks"</dt>
                    <dd class="text-sm text-foreground ">
                        {move || if wipe_disks.get() { "Yes" } else { "No" }}
                    </dd>
                </div>
            </dl>

            <div class="bg-blue-50900/20 border border-blue-200800 rounded-md p-4">
                <h4 class="text-sm font-medium text-blue-800200">
                    "What will happen"
                </h4>
                <ol class="mt-2 text-sm text-blue-700300 list-decimal list-inside space-y-1">
                    <li>"Connect to target via SSH"</li>
                    {move || use_kexec.get().then(|| view! {
                        <li>"Boot into NixOS installer using kexec"</li>
                    })}
                    <li>"Format disks according to disko configuration"</li>
                    <li>"Install NixOS with the specified configuration"</li>
                    <li>"Reboot into the installed system"</li>
                </ol>
            </div>
        </div>
    }
}

/// Step 5: Installing
#[component]
fn InstallingStep(job_id: ReadSignal<Option<String>>) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="text-center">
                <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10  mb-4">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"/>
                </div>
                <h2 class="text-lg font-medium text-foreground ">
                    "Installing NixOS..."
                </h2>
                <p class="mt-2 text-sm text-muted-foreground ">
                    "This may take several minutes. Please don't close this page."
                </p>
            </div>

            // Progress bar
            <div class="w-full bg-muted  rounded-full h-2.5">
                <div class="bg-primary h-2.5 rounded-full transition-all duration-500" style="width: 25%"/>
            </div>

            // Terminal output
            <div class="bg-background rounded-lg p-4 font-mono text-sm text-green-400 h-64 overflow-auto">
                <p>"$ nixos-anywhere --flake .#nixosConfigurations.myhost root@192.168.1.100"</p>
                <p class="text-muted-foreground">"Connecting to target..."</p>
                <p class="animate-pulse">"_"</p>
            </div>

            // Status
            <div class="flex items-center justify-center space-x-2 text-sm text-muted-foreground ">
                <span>"Current phase: "</span>
                <span class="font-medium text-foreground ">"Connecting"</span>
            </div>
        </div>
    }
}
