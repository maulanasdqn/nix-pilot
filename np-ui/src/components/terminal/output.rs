use leptos::prelude::*;

/// A line of terminal output
#[derive(Debug, Clone, PartialEq)]
pub struct OutputLine {
    pub id: usize,
    pub stream: String,
    pub content: String,
    pub timestamp: String,
}

/// Terminal-like output display component
#[component]
pub fn TerminalOutput(
    #[prop(into)] lines: Signal<Vec<OutputLine>>,
    #[prop(default = true)] auto_scroll: bool,
    #[prop(default = "max-h-96".to_string(), into)] max_height: String,
) -> impl IntoView {
    let container_ref = NodeRef::<leptos::html::Div>::new();

    // Auto-scroll effect
    Effect::new(move |_| {
        let _ = lines.get();
        if auto_scroll {
            if let Some(el) = container_ref.get() {
                let _ = el.set_scroll_top(el.scroll_height());
            }
        }
    });

    let container_class = format!(
        "bg-gray-900 text-gray-100 font-mono text-sm p-4 rounded-lg overflow-auto {}",
        max_height
    );

    view! {
        <div
            node_ref=container_ref
            class=container_class
        >
            <For
                each=move || lines.get()
                key=|line| line.id
                children=move |line| {
                    let content_class = if line.stream == "stderr" {
                        "text-red-400"
                    } else {
                        "text-gray-100"
                    };

                    view! {
                        <div class=content_class>
                            <span class="text-gray-500 mr-2 select-none">{line.timestamp}</span>
                            <span>{line.content}</span>
                        </div>
                    }
                }
            />
        </div>
    }
}
