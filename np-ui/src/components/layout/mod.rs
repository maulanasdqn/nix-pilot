mod navbar;
mod sidebar;

pub use navbar::Navbar;
pub use sidebar::Sidebar;

use leptos::prelude::*;

/// Main layout component with navbar and sidebar
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
            <Navbar/>
            <div class="flex">
                <Sidebar/>
                <main class="flex-1 p-6">
                    {children()}
                </main>
            </div>
        </div>
    }
}
