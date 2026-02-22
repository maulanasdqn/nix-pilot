use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::*;

mod components {
    use super::*;
    clx! {Tabs, div, "flex flex-wrap"}
    clx! {TabsContent, div,
        "tabs__item-content",
        "order-last",
        "hidden w-full",
    }
}

pub use components::*;

use crate::components::hooks::use_random::use_random_id;

/* ========================================================== */
/*                     ✨ FUNCTIONS ✨                        */
/* ========================================================== */

#[component]
pub fn TabsTrigger(
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] checked: Option<bool>,
    #[prop(into, optional)] name: Option<String>,
    children: Children,
) -> impl IntoView {
    let random_id = use_random_id();
    let id_for_label = random_id.clone();

    let class_label = tw_merge!(
        "tabs__item-label",
        "order-1",
        "font-bold cursor-pointer block p-4 mr-[0.2rem] border-b-[0.2rem] border-transparent",
        class
    );

    view! {
        <input
            class="hidden tabs__item-input"
            type="radio"
            name=name.unwrap_or("tabs".to_string())
            id=random_id
            checked=checked
        />
        <label class=class_label for=id_for_label>
            {children()}
        </label>
    }
}