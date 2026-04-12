use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct NotificationCardProps {
    pub variant: NotificationVariant,
    pub message: String,
    #[props(default = |_| {})]
    pub on_dismiss: EventHandler<MouseEvent>,
}

#[derive(Clone, PartialEq)]
pub enum NotificationVariant {
    Success,
    Error,
    Info,
    Warning,
}

#[component]
pub fn NotificationCard(props: NotificationCardProps) -> Element {
    let (bg_color, border_color, icon_color, text_color, icon_path) = match props.variant {
        NotificationVariant::Success => (
            "bg-green-50",
            "border-green-200",
            "text-green-500",
            "text-green-800",
            "M5 13l4 4L19 7",
        ),
        NotificationVariant::Error => (
            "bg-red-50",
            "border-red-200",
            "text-red-500",
            "text-red-800",
            "M6 18L18 6M6 6l12 12",
        ),
        NotificationVariant::Info => (
            "bg-blue-50",
            "border-blue-200",
            "text-blue-500",
            "text-blue-800",
            "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
        ),
        NotificationVariant::Warning => (
            "bg-yellow-50",
            "border-yellow-200",
            "text-yellow-500",
            "text-yellow-800",
            "M12 9v2m0 3l.01-.01.01-.01.01c-1.08 0-2-.08-2.42V7a2 2 0 01-2-2h-1l-2 7a2 2 0 01-2 2H3m-2-7a2 2 0 01-2-2h3a2 2 0 012-2zm0-8a1 1 0 110-2 1 1 0 110-2 1 1 0 110 2 1 1 0 110 2zm-2 9V7h2v2h-2z",
        ),
    };

    rsx! {
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mb-4",

            div {
                class: "{bg_color} border {border_color} rounded-lg p-4 flex items-center",

                svg {
                    class: "h-5 w-5 {icon_color} mr-2",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",

                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "stroke-width": 2,
                        d: "{icon_path}",
                    }
                }

                span {
                    class: "text-sm {text_color}",
                    "{props.message}"
                }

                button {
                    class: "ml-auto text-{icon_color} hover:opacity-80",
                    onclick: props.on_dismiss,
                    "×"
                }
            }
        }
    }
}
