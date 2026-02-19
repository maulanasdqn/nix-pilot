use leptos::prelude::*;

/// 404 Not Found page
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-96">
            <h1 class="text-6xl font-bold text-gray-300 dark:text-gray-600">
                "404"
            </h1>
            <p class="mt-4 text-xl text-gray-600 dark:text-gray-400">
                "Page not found"
            </p>
            <a
                href="/"
                class="mt-6 inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
            >
                "Go to Dashboard"
            </a>
        </div>
    }
}
