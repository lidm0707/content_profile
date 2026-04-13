use crate::routes::Route;
use content_sdk::models::Content;
use dioxus::prelude::*;
use tracing::debug;

/// Props for the content list component
#[derive(Clone, PartialEq, Props)]
pub struct ContentListProps {
    /// The content items to display
    pub contents: Vec<Content>,
}

/// Content list component that displays content items in a grid layout
#[component]
pub fn ContentList(props: ContentListProps) -> Element {
    rsx! {
        div {
            class: "mt-8",

            if props.contents.is_empty() {
                div {
                    class: "text-center py-12",
                    p {
                        class: "text-gray-500 text-lg",
                        "No content found. Create your first content item!"
                    }
                }
            } else {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",

                    for content in props.contents.iter() {
                        ContentCard { content: content.clone() }
                    }
                }
            }
        }
    }
}

/// Individual content card component
#[component]
fn ContentCard(content: Content) -> Element {
    debug!(
        "Rendering content card - ID: {:?}, title: {}, status: {}",
        content.id, content.title, content.status
    );

    let status_color = match content.status.as_str() {
        "published" => "bg-green-100 text-green-800",
        "draft" => "bg-yellow-100 text-yellow-800",
        "archived" => "bg-gray-100 text-gray-800",
        _ => "bg-blue-100 text-blue-800",
    };

    let sync_status_color = if content.synced_at.is_some() {
        "bg-blue-100 text-blue-800"
    } else {
        "bg-gray-100 text-gray-600"
    };
    let sync_status = if content.synced_at.is_some() {
        "SYNCED"
    } else {
        "LOCAL"
    };

    rsx! {
        div {
            class: "bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-shadow duration-200",

            div {
                class: "p-6",

                div {
                    class: "flex items-center justify-between mb-2 gap-2",

                    span {
                        class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {status_color}",
                        "{content.status.to_uppercase()}"
                    }

                    span {
                        class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {sync_status_color}",
                        "{sync_status}"
                    }
                }

                h3 {
                    class: "text-lg font-medium text-gray-900 truncate",
                    "{content.title}"
                }

                p {
                    class: "mt-1 text-sm text-gray-500 line-clamp-3",
                    "{content.body.chars().take(150).collect::<String>()}"
                }

                div {
                    class: "mt-4 flex items-center justify-between",

                    div {
                        class: "text-xs text-gray-400",
                        if let Some(created_at) = content.created_at {
                            "Created: {created_at.format(\"%Y-%m-%d\")}"
                        } else {
                            "Created: N/A"
                        }
                    }

                    Link {
                        to: Route::ContentEdit { id: content.id.unwrap_or(0) },
                        class: "text-indigo-600 hover:text-indigo-900 text-sm font-medium",
                        onclick: move |_| {
                            debug!(
                                "Edit clicked for content - ID: {:?}, title: {}",
                                content.id,
                                content.title
                            );
                        },
                        "Edit →"
                    }
                }
            }
        }
    }
}
