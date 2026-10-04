use dioxus::prelude::*;

/// Props for the StatCard component.
///
/// Compact pill style: small colored icon + count + label, all inline.
/// Designed to fit several stats in a single row.
#[derive(Clone, PartialEq, Props)]
pub struct StatCardProps {
    /// The label for the stat (e.g. "Published").
    pub label: String,
    /// The value to display (e.g. "9").
    pub value: String,
    /// Tailwind text color class for the icon + value (e.g. "text-green-600").
    #[props(default = "text-gray-900".to_string())]
    pub value_color: String,
    /// Tailwind background tint class for the icon chip (e.g. "bg-green-100").
    #[props(default = "bg-gray-100".to_string())]
    pub icon_bg: String,
    /// Heroicon-style SVG path `d` attribute. Rendered inside a 20x20 viewBox.
    /// Pass `None` (default) to render a simple filled circle dot.
    #[props(default = String::new())]
    pub icon_path: String,
}

/// Stat card component — compact inline pill with icon + count + label.
#[component]
pub fn StatCard(props: StatCardProps) -> Element {
    // Fallback icon if none provided: a small dot.
    let path = if props.icon_path.is_empty() {
        "M10 10m-3 0a3 3 0 106 0 3 3 0 106 0".to_string()
    } else {
        props.icon_path.clone()
    };

    rsx! {
        div {
            class: "inline-flex items-center gap-2 px-3 py-1.5 bg-white rounded-full border border-gray-200 shadow-sm",

            // Icon chip
            span {
                class: "inline-flex items-center justify-center w-6 h-6 rounded-full {props.icon_bg}",
                svg {
                    class: "w-3.5 h-3.5 {props.value_color}",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "stroke-width": 2,
                        d: "{path}",
                    }
                }
            }

            // Value
            span {
                class: "text-sm font-semibold {props.value_color}",
                "{props.value}"
            }

            // Label
            span {
                class: "text-xs text-gray-500",
                "{props.label}"
            }
        }
    }
}
