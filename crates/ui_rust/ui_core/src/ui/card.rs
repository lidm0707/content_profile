use dioxus::prelude::*;

/// Shared props for [Card].
#[derive(Clone, PartialEq, Props)]
pub struct CardProps {
    children: Element,
}

/// Shared panel surface used by pages and forms.
#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        div {
            class: "bg-white shadow rounded-lg overflow-hidden",
            {props.children}
        }
    }
}

/// Shared props for [PageHeader].
#[derive(Clone, PartialEq, Props)]
pub struct PageHeaderProps {
    /// Page title.
    title: String,
    /// One-line subtitle under the title.
    subtitle: String,
    /// Optional right-aligned actions (buttons).
    actions: Option<Element>,
}

/// Shared page heading block: title + subtitle + optional actions row.
#[component]
pub fn PageHeader(props: PageHeaderProps) -> Element {
    rsx! {
        div {
            class: "flex items-start justify-between gap-4 mb-6",

            div {
                h2 {
                    class: "text-3xl font-extrabold tracking-wide text-gray-900",
                    "{props.title}"
                }
                p {
                    class: "mt-1 text-sm text-gray-500",
                    "{props.subtitle}"
                }
            }

            if let Some(actions) = props.actions {
                div {
                    class: "shrink-0",
                    {actions}
                }
            }
        }
    }
}
