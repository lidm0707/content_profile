use content_sdk::models::Tag;
use dioxus::prelude::*;

/// Props for the [`TagPills`] component.
#[derive(Clone, PartialEq, Props)]
pub struct TagPillsProps {
    /// All available tags to render as clickable pills.
    pub tags: Vec<Tag>,
    /// Name of the currently-active tag (empty string = no filter active).
    /// The active pill is highlighted differently from the rest.
    #[props(default = String::new())]
    pub active_tag: String,
    /// Called with the tag name when a pill is clicked. The parent decides
    /// what to do (typically: navigate to `/content/list/{name}`).
    pub on_click: EventHandler<String>,
}

/// Inline row of tag pills. Used on both the Dashboard and the filtered
/// ContentList page so users can switch tags without going back.
#[component]
pub fn TagPills(props: TagPillsProps) -> Element {
    if props.tags.is_empty() {
        return rsx! {
            p {
                class: "text-sm text-gray-400",
                "No tags yet."
            }
        };
    }

    let active = props.active_tag.clone();

    rsx! {
        div {
            class: "flex flex-wrap gap-2",

            for tag in props.tags.iter() {
                {
                    let name = tag.name.clone();
                    let name_for_click = name.clone();
                    let is_active = name == active;
                    let class = if is_active {
                        "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-indigo-600 text-white shadow-sm"
                    } else {
                        "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-indigo-100 text-indigo-800 hover:bg-indigo-200 transition-colors"
                    };

                    rsx! {
                        button {
                            key: "{name}",
                            class: "{class}",
                            onclick: move |_| {
                                props.on_click.call(name_for_click.clone());
                            },
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}
