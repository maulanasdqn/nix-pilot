use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Flake list page
#[component]
pub fn FlakeListPage() -> impl IntoView {
    // In a real app, this would fetch from the API
    // For now, we'll show the empty state
    let flakes: Vec<()> = vec![];

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-foreground ">
                    "Flakes"
                </h1>
                <A
                    href="/flakes/add"
                    attr:class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors"
                >
                    "+ Register Flake"
                </A>
            </div>

            {if flakes.is_empty() {
                view! {
                    <Card>
                        <div class="text-center py-12">
                            <div class="text-muted-foreground text-5xl mb-4">"*"</div>
                            <h3 class="text-lg font-medium text-foreground  mb-2">
                                "No flakes registered"
                            </h3>
                            <p class="text-muted-foreground  mb-4">
                                "Register a flake to manage its inputs and deploy configurations."
                            </p>
                            <A
                                href="/flakes/add"
                                attr:class="inline-flex items-center px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors"
                            >
                                "Register Your First Flake"
                            </A>
                        </div>
                    </Card>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        // Flake cards would be rendered here
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// Flake card component
#[component]
fn FlakeCard(
    #[prop(into)] id: String,
    #[prop(into)] name: String,
    #[prop(into)] path: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(into)] input_count: usize,
) -> impl IntoView {
    let href = format!("/flakes/{}", id);

    view! {
        <A href=href attr:class="block">
            <Card class="hover:border-primary transition-colors cursor-pointer".to_string()>
                <div class="space-y-3">
                    <div class="flex items-start justify-between">
                        <h3 class="text-lg font-medium text-foreground ">
                            {name}
                        </h3>
                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                            {input_count} " inputs"
                        </span>
                    </div>

                    {description.map(|desc| view! {
                        <p class="text-sm text-muted-foreground  line-clamp-2">
                            {desc}
                        </p>
                    })}

                    <div class="flex items-center text-sm text-muted-foreground ">
                        <span class="font-mono truncate">{path}</span>
                    </div>
                </div>
            </Card>
        </A>
    }
}
