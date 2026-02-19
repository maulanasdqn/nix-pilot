use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::icons::*;

/// Navigation item in the sidebar
#[component]
fn NavItem(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <A
            href=href
            attr:class="flex items-center px-4 py-2 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
            <span class="mr-3 w-5 h-5">{icon}</span>
            <span>{label}</span>
        </A>
    }
}

/// Left sidebar navigation
#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 min-h-screen">
            <nav class="p-4 space-y-2">
                <NavItem href="/" label="Dashboard" icon=view! { <IconDashboard size=IconSize::Md /> } />
                <NavItem href="/machines" label="Machines" icon=view! { <IconServer size=IconSize::Md /> } />
                <NavItem href="/install" label="Install NixOS" icon=view! { <IconInstall size=IconSize::Md /> } />
                <NavItem href="/flakes" label="Flakes" icon=view! { <IconFlake size=IconSize::Md /> } />
                <NavItem href="/deploy" label="Deploy" icon=view! { <IconDeploy size=IconSize::Md /> } />
                <NavItem href="/secrets" label="Secrets" icon=view! { <IconShield size=IconSize::Md /> } />
                <NavItem href="/nix" label="Nix Operations" icon=view! { <IconTerminal size=IconSize::Md /> } />
                <NavItem href="/settings" label="Settings" icon=view! { <IconSettings size=IconSize::Md /> } />
            </nav>
        </aside>
    }
}
