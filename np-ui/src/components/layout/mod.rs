mod navbar;
mod sidebar;

pub use navbar::Navbar;
pub use sidebar::Sidebar;

use leptos::prelude::*;

/// Mobile sidebar overlay
#[component]
fn MobileSidebar(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        // Backdrop
        <div
            class=move || {
                if open.get() {
                    "fixed inset-0 z-40 bg-black/50 transition-opacity duration-300 md:hidden"
                } else {
                    "fixed inset-0 z-40 bg-black/0 pointer-events-none transition-opacity duration-300 md:hidden"
                }
            }
            on:click=move |_| set_open.set(false)
        ></div>

        // Sidebar drawer
        <aside
            class=move || {
                let base = "fixed inset-y-0 left-0 z-50 w-72 flex flex-col border-r border-border bg-background transform transition-transform duration-300 ease-in-out md:hidden";
                if open.get() {
                    format!("{} translate-x-0", base)
                } else {
                    format!("{} -translate-x-full", base)
                }
            }
        >
            <Sidebar mobile=true on_close=move || set_open.set(false) />
        </aside>
    }
}

/// Main layout component with sidebar navigation (shadcn style)
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    // Mobile sidebar state
    let (mobile_open, set_mobile_open) = signal(false);

    view! {
        <div class="min-h-screen bg-background text-foreground">
            // Mobile sidebar overlay
            <MobileSidebar open=mobile_open set_open=set_mobile_open />

            <div class="flex h-screen">
                // Desktop sidebar
                <aside class="hidden md:flex w-64 flex-col border-r border-border bg-background flex-shrink-0">
                    <Sidebar mobile=false on_close=|| {} />
                </aside>

                <div class="flex-1 flex flex-col min-h-0 min-w-0">
                    <Navbar on_menu_click=move || set_mobile_open.update(|v| *v = !*v) />
                    <main class="flex-1 p-4 sm:p-6 overflow-auto">
                        <div class="mx-auto max-w-7xl">
                            {children()}
                        </div>
                    </main>
                </div>
            </div>
        </div>
    }
}
