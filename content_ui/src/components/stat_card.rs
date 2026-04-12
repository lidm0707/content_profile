use dioxus::prelude::*;

/// Props for the StatCard component
#[derive(Clone, PartialEq, Props)]
pub struct StatCardProps {
    /// The label for the stat
    pub label: String,
    /// The value to display
    pub value: String,
    /// Optional color class for the value text
    #[props(default = "text-gray-900".to_string())]
    pub value_color: String,
}

/// Stat card component - displays a statistic with label and value
#[component]
pub fn StatCard(props: StatCardProps) -> Element {
    rsx! {
        div {
            class: "bg-white overflow-hidden shadow rounded-lg",

            div {
                class: "px-4 py-5 sm:p-6",

                dt {
                    class: "text-sm font-medium text-gray-500 truncate",
                    "{props.label}"
                }

                dd {
                    class: "mt-1 text-3xl font-semibold {props.value_color}",
                    "{props.value}"
                }
            }
        }
    }
}
