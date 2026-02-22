mod navbar;
mod sidebar;

pub use navbar::Navbar;
pub use sidebar::Sidebar;

use leptos::prelude::*;

/// Main layout component with sidebar navigation (shadcn style)
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-background text-foreground">
            <div class="flex h-screen">
                <Sidebar/>
                <div class="flex-1 flex flex-col min-h-0">
                    <Navbar/>
                    <main class="flex-1 p-6 overflow-auto">
                        <div class="mx-auto max-w-7xl">
                            {children()}
                        </div>
                    </main>
                </div>
            </div>
        </div>
    }
}
