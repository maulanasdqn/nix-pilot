use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::app::logout;
use crate::components::icons::*;

/// User profile dropdown component
#[component]
fn ProfileDropdown() -> impl IntoView {
    let (open, set_open) = signal(false);

    view! {
        <div class="relative">
            <button
                type="button"
                class="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-accent transition-colors"
                on:click=move |_| set_open.update(|v| *v = !*v)
            >
                <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-medium">
                    "A"
                </div>
                <div class="hidden sm:block text-left">
                    <p class="text-sm font-medium text-foreground">"Admin"</p>
                    <p class="text-xs text-muted-foreground">"admin@nix.local"</p>
                </div>
                <IconChevronDown size=IconSize::Sm class="hidden sm:block text-muted-foreground".to_string() />
            </button>

            // Backdrop
            <div
                class=move || if open.get() { "fixed inset-0 z-40" } else { "hidden" }
                on:click=move |_| set_open.set(false)
            ></div>

            // Dropdown menu
            <div class=move || {
                if open.get() {
                    "absolute right-0 mt-2 w-56 rounded-md border border-border bg-card shadow-lg z-50"
                } else {
                    "hidden"
                }
            }>
                <div class="p-2">
                    <div class="px-2 py-2 border-b border-border mb-2">
                        <p class="text-sm font-medium text-foreground">"Admin"</p>
                        <p class="text-xs text-muted-foreground">"admin@nix.local"</p>
                    </div>

                    <button
                        type="button"
                        class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                        on:click=move |_| {
                            set_open.set(false);
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/settings");
                            }
                        }
                    >
                        <IconSettings size=IconSize::Sm />
                        "Settings"
                    </button>

                    <div class="my-1 border-t border-border"></div>

                    <button
                        type="button"
                        class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-sm text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-colors"
                        on:click=move |_| {
                            set_open.set(false);
                            logout();
                        }
                    >
                        <IconLogout size=IconSize::Sm />
                        "Logout"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Get page title from pathname
fn get_page_title(pathname: &str) -> &'static str {
    match pathname {
        "/" | "/dashboard" => "Dashboard",
        "/services" => "Services",
        "/flakes" => "Flakes",
        "/flakes/add" => "Add Flake",
        "/rebuild" => "Rebuild",
        "/nix" => "Nix Operations",
        "/secrets" => "Secrets",
        "/secrets/keys" => "Age Keys",
        "/settings" => "Settings",
        path if path.starts_with("/services/") => "Service Details",
        path if path.starts_with("/flakes/") => "Flake Details",
        _ => "Nix Pilot",
    }
}

/// Top navigation bar (shadcn style)
#[component]
pub fn Navbar(
    #[prop(into)] on_menu_click: Callback<()>,
) -> impl IntoView {
    let location = use_location();

    let page_title = move || {
        let pathname = location.pathname.get();
        get_page_title(&pathname)
    };

    view! {
        <header class="sticky top-0 z-30 h-14 border-b border-border bg-background">
            <div class="flex h-full items-center justify-between px-4 sm:px-6">
                // Left side - Mobile menu + Page context
                <div class="flex items-center gap-3">
                    // Mobile menu button
                    <button
                        type="button"
                        class="md:hidden inline-flex items-center justify-center rounded-md h-9 w-9 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                        on:click=move |_| on_menu_click.run(())
                    >
                        <IconMenu size=IconSize::Sm />
                    </button>

                    // Mobile logo (shown when sidebar is hidden)
                    <div class="md:hidden flex items-center gap-2">
                        <div class="flex h-7 w-7 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                            <IconFlake size=IconSize::Xs />
                        </div>
                        <span class="font-semibold text-foreground text-sm">"Nix Pilot"</span>
                    </div>

                    // Desktop title (dynamic based on current route)
                    <h1 class="hidden md:block text-lg font-semibold text-foreground">
                        {page_title}
                    </h1>
                </div>

                // Right side - Actions
                <div class="flex items-center gap-1 sm:gap-2">
                    // Search button (hidden on very small screens)
                    <button
                        type="button"
                        class="hidden sm:inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-3 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                    >
                        <IconSearch size=IconSize::Sm />
                        <span class="ml-2 hidden lg:inline-flex">"Search..."</span>
                        <kbd class="ml-4 hidden lg:inline-flex pointer-events-none h-5 select-none items-center gap-1 rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
                            <span class="text-xs">"⌘"</span>"K"
                        </kbd>
                    </button>

                    // Divider (hidden on mobile)
                    <div class="hidden md:block h-6 w-px bg-border"></div>

                    // Notifications
                    <button
                        type="button"
                        class="relative inline-flex items-center justify-center rounded-md h-9 w-9 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                        title="Notifications"
                    >
                        <IconBell size=IconSize::Sm />
                        // Notification dot
                        <span class="absolute top-1.5 right-1.5 flex h-2 w-2">
                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                            <span class="relative inline-flex rounded-full h-2 w-2 bg-primary"></span>
                        </span>
                    </button>

                    // Theme toggle
                    <button
                        type="button"
                        class="inline-flex items-center justify-center rounded-md h-9 w-9 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                        title="Toggle theme"
                    >
                        <IconMoon size=IconSize::Sm />
                    </button>

                    // User profile dropdown
                    <ProfileDropdown />
                </div>
            </div>
        </header>
    }
}
