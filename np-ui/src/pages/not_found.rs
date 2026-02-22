use leptos::prelude::*;

/// 404 Not Found page
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-96">
            <h1 class="text-6xl font-bold text-muted-foreground ">
                "404"
            </h1>
            <p class="mt-4 text-xl text-muted-foreground ">
                "Page not found"
            </p>
            <a
                href="/"
                class="mt-6 inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors"
            >
                "Go to Dashboard"
            </a>
        </div>
    }
}
