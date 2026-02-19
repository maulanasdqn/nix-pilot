use leptos::prelude::*;
use leptos_router::components::A;

/// Navigation item in the sidebar
#[component]
fn NavItem(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    #[prop(into)] icon: String,
) -> impl IntoView {
    view! {
        <A
            href=href
            attr:class="flex items-center px-4 py-2 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
            <span class="mr-3">{icon}</span>
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
                <NavItem href="/" label="Dashboard" icon="📊"/>
                <NavItem href="/machines" label="Machines" icon="🖥️"/>
                <NavItem href="/install" label="Install NixOS" icon="📦"/>
                <NavItem href="/flakes" label="Flakes" icon="❄️"/>
                <NavItem href="/deploy" label="Deploy" icon="🚀"/>
                <NavItem href="/settings" label="Settings" icon="⚙️"/>
            </nav>
        </aside>
    }
}
