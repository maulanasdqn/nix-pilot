use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;
use crate::components::icons::*;

/// Dashboard page - main overview
#[component]
pub fn DashboardPage() -> impl IntoView {
    // Mock data - in real app, this would come from API
    let machine_count = 0;
    let online_count = 0;
    let flake_count = 0;
    let recent_deploys = 0;
    let failed_services = 0;

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        "Dashboard"
                    </h1>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Nix Pilot - NixOS Management"
                    </p>
                </div>
                <div class="flex items-center space-x-2">
                    <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                        <IconStatusDot size=IconSize::Sm color="text-green-500".to_string() />
                        <span class="ml-2">"System Ready"</span>
                    </span>
                </div>
            </div>

            // Stats Grid
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <StatCard
                    title="Machines"
                    value=machine_count
                    subtitle=format!("{} online", online_count)
                    color="indigo"
                    href="/machines"
                    icon=view! { <IconServer size=IconSize::Lg /> }
                />
                <StatCard
                    title="Flakes"
                    value=flake_count
                    subtitle="registered"
                    color="purple"
                    href="/flakes"
                    icon=view! { <IconFlake size=IconSize::Lg /> }
                />
                <StatCard
                    title="Deployments"
                    value=recent_deploys
                    subtitle="this week"
                    color="green"
                    href="/deploy"
                    icon=view! { <IconDeploy size=IconSize::Lg /> }
                />
                <StatCard
                    title="Failed Services"
                    value=failed_services
                    subtitle="across all machines"
                    color="gray"
                    href="/machines"
                    icon=view! { <IconWarning size=IconSize::Lg /> }
                />
            </div>

            // Main Content Grid
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Quick Actions
                <Card title="Quick Actions".to_string()>
                    <div class="grid grid-cols-2 gap-3">
                        <ActionButton
                            href="/machines/add"
                            label="Add Machine"
                            description="Register a new NixOS machine"
                            color="indigo"
                            icon=view! { <IconPlus size=IconSize::Lg /> }
                        />
                        <ActionButton
                            href="/install"
                            label="Install NixOS"
                            description="Install via nixos-anywhere"
                            color="green"
                            icon=view! { <IconInstall size=IconSize::Lg /> }
                        />
                        <ActionButton
                            href="/deploy"
                            label="Deploy"
                            description="Deploy a configuration"
                            color="blue"
                            icon=view! { <IconDeploy size=IconSize::Lg /> }
                        />
                        <ActionButton
                            href="/flakes/add"
                            label="Register Flake"
                            description="Add a flake to manage"
                            color="purple"
                            icon=view! { <IconFlake size=IconSize::Lg /> }
                        />
                        <ActionButton
                            href="/nix"
                            label="Nix Operations"
                            description="GC, search, and more"
                            color="gray"
                            icon=view! { <IconTerminal size=IconSize::Lg /> }
                        />
                        <ActionButton
                            href="/machines"
                            label="Services"
                            description="Manage systemd services"
                            color="yellow"
                            icon=view! { <IconService size=IconSize::Lg /> }
                        />
                        <ActionButton
                            href="/secrets"
                            label="Secrets"
                            description="SOPS encrypted secrets"
                            color="red"
                            icon=view! { <IconShield size=IconSize::Lg /> }
                        />
                    </div>
                </Card>

                // Recent Activity
                <Card title="Recent Activity".to_string()>
                    <div class="space-y-4">
                        // Placeholder for recent activity
                        <div class="text-center py-8">
                            <div class="text-gray-300 dark:text-gray-600 mb-2">
                                <IconClock size=IconSize::Xl />
                            </div>
                            <p class="text-gray-500 dark:text-gray-400">
                                "No recent activity"
                            </p>
                            <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
                                "Deployments and installations will appear here"
                            </p>
                        </div>
                    </div>
                </Card>
            </div>

            // Feature Overview
            <Card title="Features".to_string()>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <FeatureCard
                        title="Flake Management"
                        description="Register, update, and manage flake inputs. View and edit flake configurations."
                        icon=view! { <IconFlake size=IconSize::Lg /> }
                    />
                    <FeatureCard
                        title="nixos-anywhere"
                        description="Install NixOS on any machine with just SSH access. Supports disk-image mode."
                        icon=view! { <IconInstall size=IconSize::Lg /> }
                    />
                    <FeatureCard
                        title="Deployments"
                        description="Deploy configurations with nixos-rebuild. Support for switch, boot, and test modes."
                        icon=view! { <IconDeploy size=IconSize::Lg /> }
                    />
                    <FeatureCard
                        title="Service Management"
                        description="Start, stop, restart services. Stream logs in real-time via journalctl."
                        icon=view! { <IconService size=IconSize::Lg /> }
                    />
                    <FeatureCard
                        title="Nix Operations"
                        description="Garbage collection, store optimization, package search, and more."
                        icon=view! { <IconTerminal size=IconSize::Lg /> }
                    />
                    <FeatureCard
                        title="SSH Connections"
                        description="Manage SSH connections with support for agent, key file, and password auth."
                        icon=view! { <IconLink size=IconSize::Lg /> }
                    />
                    <FeatureCard
                        title="Secret Management"
                        description="SOPS-encrypted secrets with age encryption. Compatible with sops-nix for NixOS deployments."
                        icon=view! { <IconShield size=IconSize::Lg /> }
                    />
                </div>
            </Card>
        </div>
    }
}

/// Stat card component
#[component]
fn StatCard(
    #[prop(into)] title: String,
    value: usize,
    #[prop(into)] subtitle: String,
    #[prop(into)] color: String,
    #[prop(into)] href: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    let bg_color = match color.as_str() {
        "indigo" => "bg-indigo-50 dark:bg-indigo-900/20",
        "purple" => "bg-purple-50 dark:bg-purple-900/20",
        "green" => "bg-green-50 dark:bg-green-900/20",
        "red" => "bg-red-50 dark:bg-red-900/20",
        "yellow" => "bg-yellow-50 dark:bg-yellow-900/20",
        "blue" => "bg-blue-50 dark:bg-blue-900/20",
        _ => "bg-gray-50 dark:bg-gray-800",
    };

    let text_color = match color.as_str() {
        "indigo" => "text-indigo-600 dark:text-indigo-400",
        "purple" => "text-purple-600 dark:text-purple-400",
        "green" => "text-green-600 dark:text-green-400",
        "red" => "text-red-600 dark:text-red-400",
        "yellow" => "text-yellow-600 dark:text-yellow-400",
        "blue" => "text-blue-600 dark:text-blue-400",
        _ => "text-gray-600 dark:text-gray-400",
    };

    view! {
        <A
            href=href
            attr:class=format!(
                "block p-4 rounded-lg {} hover:ring-2 hover:ring-offset-2 hover:ring-indigo-500 transition-all",
                bg_color
            )
        >
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-500 dark:text-gray-400">{title}</p>
                    <p class=format!("text-3xl font-bold {}", text_color)>{value}</p>
                    <p class="text-xs text-gray-400 dark:text-gray-500">{subtitle}</p>
                </div>
                <div class=format!("{}", text_color)>
                    {icon}
                </div>
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
    #[prop(into)] color: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    let icon_bg = match color.as_str() {
        "indigo" => "bg-indigo-100 text-indigo-600 dark:bg-indigo-900 dark:text-indigo-400",
        "purple" => "bg-purple-100 text-purple-600 dark:bg-purple-900 dark:text-purple-400",
        "green" => "bg-green-100 text-green-600 dark:bg-green-900 dark:text-green-400",
        "blue" => "bg-blue-100 text-blue-600 dark:bg-blue-900 dark:text-blue-400",
        "yellow" => "bg-yellow-100 text-yellow-600 dark:bg-yellow-900 dark:text-yellow-400",
        "red" => "bg-red-100 text-red-600 dark:bg-red-900 dark:text-red-400",
        _ => "bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400",
    };

    view! {
        <A
            href=href
            attr:class="flex items-center p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
        >
            <div class=format!("flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center {}", icon_bg)>
                {icon}
            </div>
            <div class="ml-3">
                <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{label}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{description}</p>
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
        <div class="p-4 rounded-lg bg-gray-50 dark:bg-gray-800">
            <div class="flex items-center space-x-3 mb-2">
                <span class="flex-shrink-0 w-8 h-8 rounded-lg bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-indigo-600 dark:text-indigo-400">
                    {icon}
                </span>
                <h3 class="font-medium text-gray-900 dark:text-gray-100">{title}</h3>
            </div>
            <p class="text-sm text-gray-500 dark:text-gray-400">{description}</p>
        </div>
    }
}
