use leptos::prelude::*;

/// Card component for containing content (shadcn style)
#[component]
pub fn Card(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base_classes = "rounded-xl border border-border bg-card text-card-foreground";
    let classes = match class {
        Some(c) => format!("{} {}", base_classes, c),
        None => base_classes.to_string(),
    };

    view! {
        <div class=classes>
            {title.map(|t| view! {
                <div class="px-6 py-4 border-b border-border">
                    <h3 class="text-lg font-semibold text-foreground">
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
