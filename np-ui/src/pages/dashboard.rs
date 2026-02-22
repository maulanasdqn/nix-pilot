use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::api::check_response_status;
use crate::components::icons::*;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(default)]
    hostname: String,
    #[serde(default)]
    nixos_version: String,
    #[serde(default)]
    kernel_version: String,
    #[serde(default)]
    uptime: String,
    #[serde(default)]
    system_type: String,
    #[serde(default)]
    os: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DashboardStats {
    #[serde(default)]
    flake_count: usize,
    #[serde(default)]
    active_services: usize,
    #[serde(default)]
    failed_services: usize,
    #[serde(default)]
    secret_count: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FlakeListResponse {
    #[serde(default)]
    flakes: Vec<FlakeBasic>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FlakeBasic {
    #[serde(default)]
    id: String,
}

async fn fetch_system_info() -> Result<SystemInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/system/info", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(ref t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

    // Handle 401 - logout and redirect
    check_response_status(resp.status(), resp.ok())?;

    if !resp.ok() {
        return Ok(SystemInfo {
            hostname: "localhost".to_string(),
            nixos_version: "Unknown".to_string(),
            kernel_version: "Unknown".to_string(),
            uptime: "Unknown".to_string(),
            system_type: "unknown".to_string(),
            os: "unknown".to_string(),
        });
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))
}

async fn fetch_dashboard_stats() -> Result<DashboardStats, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut stats = DashboardStats::default();

    // Fetch flakes
    {
        let opts = web_sys::RequestInit::new();
        opts.set_method("GET");

        let request = web_sys::Request::new_with_str_and_init("/api/flakes", &opts)
            .map_err(|_| "Failed to create request")?;

        if let Some(ref t) = token {
            request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
        }

        let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| "Fetch failed")?;

        let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;

        if resp.ok() {
            if let Ok(json) = wasm_bindgen_futures::JsFuture::from(resp.json().unwrap()).await {
                if let Ok(data) = serde_wasm_bindgen::from_value::<FlakeListResponse>(json) {
                    stats.flake_count = data.flakes.len();
                }
            }
        }
    }

    Ok(stats)
}

/// Dashboard page - local system overview
#[component]
pub fn DashboardPage() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (stats, set_stats) = signal(DashboardStats::default());
    let (system_info, set_system_info) = signal(SystemInfo::default());

    // Fetch data on mount
    leptos::task::spawn_local(async move {
        // Fetch system info
        if let Ok(info) = fetch_system_info().await {
            set_system_info.set(info);
        }
        // Fetch dashboard stats
        if let Ok(s) = fetch_dashboard_stats().await {
            set_stats.set(s);
        }
        set_loading.set(false);
    });

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-foreground">
                        "Dashboard"
                    </h1>
                    <p class="text-muted-foreground">
                        "Manage your local Nix system"
                    </p>
                </div>
            </div>

            // System Info Card
            <Card>
                <CardHeader>
                    <CardTitle>"System Information"</CardTitle>
                    <CardDescription>"Current system status"</CardDescription>
                </CardHeader>
                <CardContent>
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <div class="animate-pulse h-20 bg-muted rounded"></div> }
                    >
                        {move || {
                            let info = system_info.get();
                            let version_label = match info.system_type.as_str() {
                                "nix-darwin" => "nix-darwin",
                                "nixos" => "NixOS Version",
                                _ => "System",
                            };
                            view! {
                                <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                                    <div class="space-y-1">
                                        <p class="text-sm text-muted-foreground">"Hostname"</p>
                                        <p class="text-lg font-medium text-foreground font-mono">
                                            {if info.hostname.is_empty() { "localhost".to_string() } else { info.hostname.clone() }}
                                        </p>
                                    </div>
                                    <div class="space-y-1">
                                        <p class="text-sm text-muted-foreground">{version_label}</p>
                                        <p class="text-lg font-medium text-foreground">
                                            {if info.nixos_version.is_empty() { "Unknown".to_string() } else { info.nixos_version.clone() }}
                                        </p>
                                    </div>
                                    <div class="space-y-1">
                                        <p class="text-sm text-muted-foreground">"Kernel"</p>
                                        <p class="text-lg font-medium text-foreground">
                                            {if info.kernel_version.is_empty() { "Unknown".to_string() } else { info.kernel_version.clone() }}
                                        </p>
                                    </div>
                                    <div class="space-y-1">
                                        <p class="text-sm text-muted-foreground">"Uptime"</p>
                                        <p class="text-lg font-medium text-foreground">
                                            {if info.uptime.is_empty() { "Unknown".to_string() } else { info.uptime.clone() }}
                                        </p>
                                    </div>
                                </div>
                            }
                        }}
                    </Show>
                </CardContent>
            </Card>

            // Stats Grid
            <Show when=move || !loading.get()>
                {move || {
                    let s = stats.get();
                    view! {
                        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                            <StatCard
                                title="Services"
                                value="View All"
                                subtitle="manage systemd services"
                                href="/services"
                                icon=view! { <IconService size=IconSize::Md /> }
                            />
                            <StatCard
                                title="Flakes"
                                value=s.flake_count.to_string()
                                subtitle="registered"
                                href="/flakes"
                                icon=view! { <IconFlake size=IconSize::Md /> }
                            />
                            <StatCard
                                title="Rebuild"
                                value="nixos-rebuild"
                                subtitle="switch, boot, test"
                                href="/rebuild"
                                icon=view! { <IconDeploy size=IconSize::Md /> }
                            />
                            <StatCard
                                title="Secrets"
                                value="SOPS"
                                subtitle="encrypted secrets"
                                href="/secrets"
                                icon=view! { <IconShield size=IconSize::Md /> }
                            />
                        </div>
                    }
                }}
            </Show>

            // Main Content Grid
            <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-7">
                // Quick Actions - takes 4 columns
                <Card class="lg:col-span-4">
                    <CardHeader>
                        <CardTitle>"Quick Actions"</CardTitle>
                        <CardDescription>"Common tasks and operations"</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div class="grid gap-3 sm:grid-cols-2">
                            <ActionButton
                                href="/rebuild"
                                label="Rebuild System"
                                description="Run nixos-rebuild"
                                icon=view! { <IconDeploy size=IconSize::Sm /> }
                            />
                            <ActionButton
                                href="/services"
                                label="Manage Services"
                                description="Start, stop, restart services"
                                icon=view! { <IconService size=IconSize::Sm /> }
                            />
                            <ActionButton
                                href="/flakes"
                                label="Flakes"
                                description="Manage flake configurations"
                                icon=view! { <IconFlake size=IconSize::Sm /> }
                            />
                            <ActionButton
                                href="/flakes/add"
                                label="Add Flake"
                                description="Register a new flake"
                                icon=view! { <IconPlus size=IconSize::Sm /> }
                            />
                            <ActionButton
                                href="/nix"
                                label="Nix Operations"
                                description="GC, search, optimize"
                                icon=view! { <IconTerminal size=IconSize::Sm /> }
                            />
                            <ActionButton
                                href="/secrets"
                                label="Secrets"
                                description="SOPS encrypted secrets"
                                icon=view! { <IconShield size=IconSize::Sm /> }
                            />
                        </div>
                    </CardContent>
                </Card>

                // System Status - takes 3 columns
                <Card class="lg:col-span-3">
                    <CardHeader>
                        <CardTitle>"System Status"</CardTitle>
                        <CardDescription>"Current system health"</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div class="space-y-4">
                            <div class="flex items-center gap-3 p-3 rounded-lg bg-green-500/10 border border-green-500/20">
                                <div class="flex h-8 w-8 items-center justify-center rounded-full bg-green-500/20">
                                    <IconCheck size=IconSize::Sm />
                                </div>
                                <div>
                                    <p class="text-sm font-medium text-green-400">"System Online"</p>
                                    <p class="text-xs text-muted-foreground">"All core services running"</p>
                                </div>
                            </div>
                            <div class="text-sm text-muted-foreground">
                                <p>"Use the Services page to monitor individual systemd services and view logs."</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>

            // Feature Overview
            <Card>
                <CardHeader>
                    <CardTitle>"Features"</CardTitle>
                    <CardDescription>"What you can do with Nix Pilot"</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        <FeatureCard
                            title="System Rebuild"
                            description="Rebuild your NixOS configuration with switch, boot, test, or dry-run modes."
                            icon=view! { <IconDeploy size=IconSize::Sm /> }
                        />
                        <FeatureCard
                            title="Service Management"
                            description="Start, stop, restart systemd services. View logs in real-time."
                            icon=view! { <IconService size=IconSize::Sm /> }
                        />
                        <FeatureCard
                            title="Flake Management"
                            description="Register, update, and manage flake configurations."
                            icon=view! { <IconFlake size=IconSize::Sm /> }
                        />
                        <FeatureCard
                            title="Nix Operations"
                            description="Garbage collection, store optimization, package search."
                            icon=view! { <IconTerminal size=IconSize::Sm /> }
                        />
                        <FeatureCard
                            title="Secret Management"
                            description="SOPS-encrypted secrets with age encryption for NixOS."
                            icon=view! { <IconShield size=IconSize::Sm /> }
                        />
                        <FeatureCard
                            title="Configuration"
                            description="Manage Nix Pilot settings and preferences."
                            icon=view! { <IconSettings size=IconSize::Sm /> }
                        />
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}

/// Stat card component
#[component]
fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] subtitle: String,
    #[prop(into)] href: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <A
            href=href
            attr:class="group block rounded-xl border border-border bg-card p-6 hover:bg-accent transition-colors"
        >
            <div class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                    <p class="text-sm font-medium text-muted-foreground">{title}</p>
                    <div class="text-muted-foreground">
                        {icon}
                    </div>
                </div>
                <p class="text-2xl font-bold text-foreground">{value}</p>
                <p class="text-xs text-muted-foreground">{subtitle}</p>
            </div>
        </A>
    }
}

/// Action button component
#[component]
fn ActionButton(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    #[prop(into)] description: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <A
            href=href
            attr:class="group flex items-center gap-4 rounded-lg border border-border p-4 hover:bg-accent transition-colors"
        >
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
                {icon}
            </div>
            <div class="space-y-0.5">
                <p class="text-sm font-medium text-foreground">{label}</p>
                <p class="text-xs text-muted-foreground">{description}</p>
            </div>
        </A>
    }
}

/// Feature card component
#[component]
fn FeatureCard(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-border p-4 hover:bg-accent/50 transition-colors">
            <div class="flex items-center gap-3 mb-2">
                <div class="flex h-8 w-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
                    {icon}
                </div>
                <h3 class="text-sm font-medium text-foreground">{title}</h3>
            </div>
            <p class="text-sm text-muted-foreground leading-relaxed">{description}</p>
        </div>
    }
}
