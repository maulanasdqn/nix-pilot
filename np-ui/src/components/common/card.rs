use leptos::prelude::*;

/// Card component for containing content
#[component]
pub fn Card(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base_classes = "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700";
    let classes = match class {
        Some(c) => format!("{} {}", base_classes, c),
        None => base_classes.to_string(),
    };

    view! {
        <div class=classes>
            {title.map(|t| view! {
                <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">
                        {t}
                    </h3>
                </div>
            })}
            <div class="p-6">
                {children()}
            </div>
        </div>
    }
}
