use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Spinner(#[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("size-4 animate-spin", class);

    view! {
        <svg
            class=merged_class
            role="status"
            aria-label="Loading"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
        >
            <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
            ></circle>
            <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            ></path>
        </svg>
    }
}

#[component]
pub fn SpinnerCircle(#[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("size-4 animate-spin", class);

    view! {
        <svg
            class=merged_class
            role="status"
            aria-label="Loading"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
        >
            <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
            ></circle>
            <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            ></path>
        </svg>
    }
}
