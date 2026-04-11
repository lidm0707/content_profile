use crate::models::Content;
use crate::routes::Route;
use dioxus::prelude::*;

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
        "published" => "bg-green-100 text-green-800",
        "draft" => "bg-yellow-100 text-yellow-800",
        "archived" => "bg-gray-100 text-gray-800",
        _ => "bg-blue-100 text-blue-800",
    };

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
                                dangerous_inner_html: "{format_content_body(&content.body)}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Formats the content body for display (simple line breaks to paragraphs)
#[allow(dead_code)]
fn format_content_body(body: &str) -> String {
    body.split('\n')
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!("<p>{}</p>", line))
        .collect::<Vec<String>>()
        .join("")
}
