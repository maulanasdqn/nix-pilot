use leptos::prelude::*;

/// Button variant styles
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
}

impl ButtonVariant {
    fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => {
                "bg-indigo-600 hover:bg-indigo-700 text-white focus:ring-indigo-500"
            }
            ButtonVariant::Secondary => {
                "bg-gray-200 hover:bg-gray-300 text-gray-800 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-200 focus:ring-gray-500"
            }
            ButtonVariant::Danger => {
                "bg-red-600 hover:bg-red-700 text-white focus:ring-red-500"
            }
        }
    }
}

/// Reusable button component
#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let base_classes = "inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed";

    let classes = format!("{} {}", base_classes, variant.classes());

    view! {
        <button
            class=classes
            disabled=disabled
            on:click=move |_| {
                if let Some(cb) = &on_click {
                    cb.run(());
                }
            }
        >
            {children()}
        </button>
    }
}
