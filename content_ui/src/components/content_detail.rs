use crate::contexts::{ContentContext, TagContext};
use crate::models::{Content, STATUS_ARCHIVED, STATUS_DRAFT, STATUS_PUBLISHED, Tag};
use crate::routes::Route;
use dioxus::prelude::*;

/// Props for the tag button component
#[derive(Clone, PartialEq, Props)]
pub struct TagButtonProps {
    pub tag: Tag,
    pub content_id: i32,
    pub on_add: EventHandler<()>,
}

/// Tag button component for adding a tag to content
#[component]
fn TagButton(props: TagButtonProps) -> Element {
    let tag_context = use_context::<TagContext>();

    rsx! {
        button {
            onclick: move |_| {
                let content_id = props.content_id;
                let tag_id = props.tag.id.unwrap_or(0);
                let tag_context = tag_context.clone();
                spawn(async move {
                    let _ = tag_context.add_tag_to_content(
                        crate::models::ContentTagRequest {
                            content_id,
                            tag_id,
                        }
                    ).await;
                });
                props.on_add.call(());
            },
            class: "w-full text-left px-4 py-2 rounded-md hover:bg-indigo-50 focus:outline-none focus:bg-indigo-100 transition-colors",
            "{props.tag.name}"
        }
    }
}

/// Props for the content detail component
#[derive(Clone, PartialEq, Props)]
pub struct ContentDetailProps {
    /// The content item to display
    pub content: Content,
}

/// Content detail component for viewing a single content item in detail
#[component]
pub fn ContentDetail(props: ContentDetailProps) -> Element {
    let content = props.content;
    let content_context: ContentContext = use_context();
    let tag_context: TagContext = use_context();
    let navigator = use_navigator();
    let mut show_delete_dialog = use_signal(|| false);

    // Load tags for this content
    let content_id = content.id.unwrap_or(0);
    let tag_context_for_tags = tag_context.clone();
    let tags_resource = use_resource(move || {
        let tag_context = tag_context_for_tags.clone();
        async move {
            if content_id > 0 {
                tag_context
                    .get_tags_for_content(content_id)
                    .await
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    // Load all available tags
    let tag_context_for_all_tags = tag_context.clone();
    let all_tags_resource = use_resource(move || {
        let tag_context = tag_context_for_all_tags.clone();
        async move { tag_context.get_all_tags().await.unwrap_or_default() }
    });

    // Extract tag data into owned values for use in RSX
    let available_tags = use_memo(move || {
        all_tags_resource
            .read()
            .as_ref()
            .cloned()
            .unwrap_or_default()
    });

    let current_content_tags =
        use_memo(move || tags_resource.read().as_ref().cloned().unwrap_or_default());

    let mut show_tag_dialog = use_signal(|| false);

    // Format dates for display
    let created_at_display = content
        .created_at
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let updated_at_display = content
        .updated_at
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let status_color = match content.status.as_str() {
        STATUS_PUBLISHED => "bg-green-100 text-green-800",
        STATUS_DRAFT => "bg-yellow-100 text-yellow-800",
        STATUS_ARCHIVED => "bg-gray-100 text-gray-800",
        _ => "bg-blue-100 text-blue-800",
    };

    let formatted_body = format_content_body(&content.body);

    rsx! {
        div {
            class: "bg-white shadow overflow-hidden sm:rounded-lg",

            div {
                class: "px-4 py-5 sm:px-6",

                div {
                    class: "flex items-start justify-between",

                    div {
                        class: "flex-1",

                        h1 {
                            class: "text-3xl font-bold text-gray-900",
                            "{content.title}"
                        }

                        div {
                            class: "mt-2 flex items-center space-x-4",

                            span {
                                class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {status_color}",
                                "{content.status.to_uppercase()}"
                            }

                            p {
                                class: "text-sm text-gray-500",
                                "Slug: {content.slug}"
                            }
                        }
                    }

                    Link {
                        to: Route::ContentEdit { id: content.id.unwrap_or(0) },
                        class: "inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                        "Edit Content"
                    }

                    button {
                        onclick: move |_| {
                            show_delete_dialog.set(true);
                        },
                        class: "ml-2 inline-flex items-center px-3 py-2 border border-red-300 shadow-sm text-sm leading-4 font-medium rounded-md text-red-700 bg-white hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500",
                        "Delete"
                    }

                    button {
                        onclick: move |_| {
                            show_tag_dialog.set(true);
                        },
                        class: "ml-2 inline-flex items-center px-3 py-2 border border-indigo-300 shadow-sm text-sm leading-4 font-medium rounded-md text-indigo-700 bg-white hover:bg-indigo-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                        "Add Tag"
                    }
                }
            }

            div {
                class: "border-t border-gray-200 px-4 py-5 sm:p-0",

                dl {
                    class: "sm:divide-y sm:divide-gray-200",

                    div {
                        class: "py-4 sm:py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6",

                        dt {
                            class: "text-sm font-medium text-gray-500",
                            "Created At"
                        }

                        dd {
                            class: "mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2",
                            "{created_at_display}"
                        }
                    }

                    div {
                        class: "py-4 sm:py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6",

                        dt {
                            class: "text-sm font-medium text-gray-500",
                            "Updated At"
                        }

                        dd {
                            class: "mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2",
                            "{updated_at_display}"
                        }
                    }

                    div {
                        class: "py-4 sm:py-5 sm:px-6",

                        dt {
                            class: "text-sm font-medium text-gray-500",
                            "Content Body"
                        }

                        dd {
                            class: "mt-1 text-sm text-gray-900 sm:mt-0",
                            div {
                                class: "prose prose-sm max-w-none",
                                dangerous_inner_html: formatted_body,
                            }
                        }

                        // Tag selection dialog
                        if show_tag_dialog() {
                            div {
                                class: "fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50",

                                div {
                                    class: "bg-white rounded-lg shadow-xl max-w-md w-full mx-4",

                                    div {
                                        class: "px-4 py-5 sm:p-6",

                                        h3 {
                                            class: "text-lg leading-6 font-medium text-gray-900",
                                            "Add Tag"
                                        }

                                        p {
                                            class: "mt-2 text-sm text-gray-500",
                                            "Select a tag to add to '{content.title}'"
                                        }
                                    }

                                    div {
                                        class: "px-4 py-3 sm:px-6",

                                        if !available_tags().is_empty() {
                                            div {
                                                class: "space-y-2 max-h-60 overflow-y-auto",

                                                for tag in available_tags() {
                                                    if !current_content_tags().iter().any(|t| t.id == tag.id) {
                                                        TagButton {
                                                            tag: tag.clone(),
                                                            content_id,
                                                            on_add: move |_| show_tag_dialog.set(false),
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            p {
                                                class: "text-sm text-gray-500",
                                                "No tags available"
                                            }
                                        }
                                    }

                                    div {
                                        class: "bg-gray-50 px-4 py-3 sm:px-6",

                                        button {
                                            onclick: move |_| {
                                                show_tag_dialog.set(false);
                                            },
                                            class: "w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:w-auto sm:text-sm",
                                            "Cancel"
                                        }
                                    }
                                }
                            }
                        }

                        // Delete confirmation dialog
                        if show_delete_dialog() {
                            div {
                                class: "fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50",

                                div {
                                    class: "bg-white rounded-lg shadow-xl max-w-md w-full mx-4",

                                    div {
                                        class: "px-4 py-5 sm:p-6",

                                        h3 {
                                            class: "text-lg leading-6 font-medium text-gray-900",
                                            "Delete Content"
                                        }

                                        p {
                                            class: "mt-2 text-sm text-gray-500",
                                            "Are you sure you want to delete '{content.title}'? This action cannot be undone."
                                        }
                                    }

                                    div {
                                        class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",

                                        button {
                                            onclick: move |_| {
                                                let id = content.id.unwrap_or(0);
                                                let mut content_context = content_context.clone();
                                                let navigator = navigator;
                                                spawn(async move {
                                                    if let Ok(_) = content_context.delete_content(id).await {
                                                        navigator.push(Route::Dashboard {});
                                                    }
                                                });
                                            },
                                            class: "w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm",
                                            "Delete"
                                        }

                                        button {
                                            onclick: move |_| {
                                                show_delete_dialog.set(false);
                                            },
                                            class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm",
                                            "Cancel"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Formats the content body for display (simple line breaks to paragraphs)
fn format_content_body(body: &str) -> String {
    body.split('\n')
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!("<p>{}</p>", line))
        .collect::<Vec<String>>()
        .join("")
}
