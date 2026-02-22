use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::app::logout;
use crate::components::icons::*;

/// Navigation item in the sidebar
#[component]
fn NavItem(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    icon: impl IntoView + 'static,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
) -> impl IntoView {
    let location = use_location();
    let href_clone = href.clone();

    let is_active = move || {
        let path = location.pathname.get();
        if href_clone == "/" {
            path == "/"
        } else {
            path.starts_with(&href_clone)
        }
    };

    view! {
        <A
            href=href
            attr:class=move || {
                let base = "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-all";
                if is_active() {
                    format!("{} bg-accent text-accent-foreground", base)
                } else {
                    format!("{} text-muted-foreground hover:text-foreground hover:bg-accent/50", base)
                }
            }
            on:click=move |_| {
                if let Some(ref cb) = on_click {
                    cb();
                }
            }
        >
            <span class="flex-shrink-0">{icon}</span>
            <span>{label}</span>
        </A>
    }
}

/// Section header in sidebar
#[component]
fn SectionHeader(#[prop(into)] label: String) -> impl IntoView {
    view! {
        <h4 class="mb-1 px-3 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            {label}
        </h4>
    }
}

/// Left sidebar navigation (shadcn style)
#[component]
pub fn Sidebar(
    #[prop(default = false)] mobile: bool,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let close = move || on_close.run(());

    view! {
        <>
            // Logo/Brand
            <div class="flex h-14 items-center border-b border-border px-4 justify-between">
                <A href="/" attr:class="flex items-center gap-2 font-semibold text-foreground">
                    <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                        <IconFlake size=IconSize::Sm />
                    </div>
                    <span>"Nix Pilot"</span>
                </A>
                // Close button for mobile
                {if mobile {
                    Some(view! {
                        <button
                            type="button"
                            class="md:hidden p-2 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                            on:click=move |_| close()
                        >
                            <IconX size=IconSize::Sm />
                        </button>
                    })
                } else {
                    None
                }}
            </div>

            // Navigation
            <nav class="flex-1 overflow-auto p-4">
                <div class="space-y-6">
                    // Overview Section
                    <div class="space-y-1">
                        <SectionHeader label="Overview" />
                        <NavItem
                            href="/dashboard"
                            label="Dashboard"
                            icon=view! { <IconDashboard size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                        <NavItem
                            href="/services"
                            label="Services"
                            icon=view! { <IconService size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                        <NavItem
                            href="/flakes"
                            label="Flakes"
                            icon=view! { <IconFlake size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                    </div>

                    // Operations Section
                    <div class="space-y-1">
                        <SectionHeader label="Operations" />
                        <NavItem
                            href="/rebuild"
                            label="Rebuild"
                            icon=view! { <IconDeploy size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                        <NavItem
                            href="/nix"
                            label="Nix Operations"
                            icon=view! { <IconTerminal size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                    </div>

                    // Security Section
                    <div class="space-y-1">
                        <SectionHeader label="Security" />
                        <NavItem
                            href="/secrets"
                            label="Secrets"
                            icon=view! { <IconShield size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                    </div>

                    // System Section
                    <div class="space-y-1">
                        <SectionHeader label="System" />
                        <NavItem
                            href="/settings"
                            label="Settings"
                            icon=view! { <IconSettings size=IconSize::Sm /> }
                            on_click=Box::new(move || close())
                        />
                    </div>
                </div>
            </nav>

            // Footer
            <div class="border-t border-border p-4 space-y-2">
                <div class="flex items-center gap-3 rounded-lg bg-muted/50 px-3 py-2">
                    <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-primary">
                        <IconStatusDot size=IconSize::Sm color="text-green-500".to_string() />
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-sm font-medium text-foreground truncate">"System Ready"</p>
                        <p class="text-xs text-muted-foreground">"All services online"</p>
                    </div>
                </div>
                <button
                    class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-all"
                    on:click=move |_| logout()
                >
                    <IconLogout size=IconSize::Sm />
                    <span>"Logout"</span>
                </button>
            </div>
        </>
    }
}
