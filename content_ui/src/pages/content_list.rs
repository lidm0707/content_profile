use crate::components::ContentList as ContentListComponent;
use crate::routes::Route;
use content_sdk::contexts::{ContentContext, TagContext};
use content_sdk::models::Content;
use dioxus::prelude::*;

/// Props for the content list page
#[derive(Clone, PartialEq, Props)]
pub struct ContentListProps {
    /// Tag to filter content by (empty string = show all content)
    pub tag: String,
}

/// Content list page - displays content filtered by tag
#[component]
pub fn ContentList(props: ContentListProps) -> Element {
    let tag_context: TagContext = use_context();
    let navigator = use_navigator();
    let content_context: ContentContext = use_context();

    let mut contents = use_resource(move || {
        let content_context = content_context.clone();
        async move { content_context.get_all_content().await }
    });

    let tag_context_for_tags = tag_context.clone();
    let mut all_tags = use_resource(move || {
        let tag_context_for_tags_closure = tag_context_for_tags.clone();
        async move { tag_context_for_tags_closure.get_all_tags().await }
    });

    let mut filtered_contents = use_signal(Vec::<Content>::new);
    let mut loading_tag_filter = use_signal(|| false);

    let tag_name = props.tag.clone();
    let tag_name_for_effect = tag_name.clone();
    use_effect(move || {
        if let Some(Ok(contents_data)) = contents.read().as_ref() {
            if !tag_name_for_effect.is_empty() {
                loading_tag_filter.set(true);
                let tag_context = tag_context.clone();
                let contents_data = contents_data.clone();
                let tag_name = tag_name_for_effect.clone();

                spawn(async move {
                    let filtered =
                        filter_content_by_tag(tag_context, contents_data, &tag_name).await;
                    filtered_contents.set(filtered);
                    loading_tag_filter.set(false);
                });
            } else {
                filtered_contents.set(contents_data.clone());
                loading_tag_filter.set(false);
            }
        }
    });

    let tag_name_for_handlers1 = tag_name.clone();
    let tag_name_for_handlers2 = tag_name.clone();

    rsx! {
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",

            div {
                class: "md:flex md:items-center md:justify-between py-8",

                div {
                    class: "flex-1 min-w-0",

                    if !tag_name_for_handlers1.is_empty() {
                        h2 {
                            class: "text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate flex items-center gap-3",
                            "Content tagged with '{tag_name_for_handlers1}'"

                            span {
                                class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800",
                                "Filtered"
                            }
                        }

                        p {
                            class: "mt-1 text-sm text-gray-500",
                            "Showing all content items tagged with '{tag_name_for_handlers1}'"
                        }
                    } else {
                        h2 {
                            class: "text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate",
                            "All Content"
                        }

                        p {
                            class: "mt-1 text-sm text-gray-500",
                            "Browse all content items in the system"
                        }
                    }
                }

                div {
                    class: "mt-4 flex md:mt-0 md:ml-4 space-x-3",

                    button {
                        onclick: move |_: MouseEvent| {
                            contents.restart();
                            if !tag_name_for_handlers1.is_empty() {
                                all_tags.restart();
                            }
                        },
                        class: "inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",

                        svg {
                            class: "-ml-1 mr-2 h-5 w-5 text-gray-500",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",

                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "stroke-width": 2,
                                d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                            }
                        }
                        "Refresh"
                    }

                    if !tag_name_for_handlers1.is_empty() {
                        button {
                            onclick: move |_| {
                                navigator.push(Route::Dashboard {});
                            },
                            class: "inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                            "Back to Dashboard"
                        }
                    }
                }
            }

            if contents.read().is_none() {
                div {
                    class: "flex items-center justify-center py-12",

                    svg {
                        class: "animate-spin h-10 w-10 text-indigo-600",
                        fill: "none",
                        view_box: "0 0 24 24",

                        circle {
                            class: "opacity-25",
                            cx: "12",
                            cy: "12",
                            r: "10",
                            stroke: "currentColor",
                            "stroke-width": 4
                        }

                        path {
                            class: "opacity-75",
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                        }
                    }
                }
            } else if let Some(Ok(_)) = contents.read().as_ref() {
                if *loading_tag_filter.read() {
                    div {
                        class: "flex items-center justify-center py-12",
                        "Filtering by tag..."
                    }
                } else {
                    ContentListComponent {
                        contents: filtered_contents.read().clone()
                    }
                }
            } else if let Some(Err(err)) = contents.read().as_ref() {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-6 text-center",

                    svg {
                        class: "mx-auto h-12 w-12 text-red-400",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",

                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            "stroke-width": 2,
                            d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                        }
                    }

                    h3 {
                        class: "mt-2 text-sm font-medium text-gray-900",
                        "Error loading content"
                    }

                    p {
                        class: "mt-1 text-sm text-gray-500",
                        "{err}"
                    }

                    button {
                        onclick: move |_: MouseEvent| {
                            contents.restart();
                            if !tag_name_for_handlers2.is_empty() {
                                all_tags.restart();
                            }
                        },
                        class: "mt-4 inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                        "Try again"
                    }
                }
            }
        }
    }
}

async fn filter_content_by_tag(
    tag_context: TagContext,
    contents: Vec<Content>,
    tag_name: &str,
) -> Vec<Content> {
    let mut filtered = Vec::new();

    for content in contents {
        let content_id = content.id.unwrap_or(0);
        if content_id == 0 {
            continue;
        }

        if let Ok(tags) = tag_context.get_tags_for_content(content_id).await {
            let has_tag = tags.iter().any(|tag| tag.name == tag_name);
            if has_tag {
                filtered.push(content);
            }
        }
    }

    filtered
}
