use crate::components::ContentList as ContentListComponent;
use crate::routes::Route;
use content_sdk::contexts::{ContentContext, ContentTagsContext, TagContext};
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
    let navigator = use_navigator();
    let content_context: ContentContext = use_context();
    let tag_context: TagContext = use_context();
    let content_tags_context: ContentTagsContext = use_context();

    let tag_name_for_resource = props.tag.clone();
    let tag_name = tag_name_for_resource.clone();
    let tag_name_for_resource = tag_name.clone();
    let tag_context_for_effect = tag_context.clone();
    let content_tags_context_for_effect = content_tags_context.clone();
    let content_context_for_effect = content_context.clone();

    let contents = use_resource(move || {
        let tag_name = tag_name_for_resource.clone();
        let tag_context = tag_context_for_effect.clone();
        let content_tags_context = content_tags_context_for_effect.clone();
        let content_context = content_context_for_effect.clone();

        async move {
            if tag_name.is_empty() {
                content_context.get_all_content().await
            } else {
                fetch_content_by_tag_name(
                    content_context,
                    content_tags_context,
                    tag_context,
                    &tag_name,
                )
                .await
            }
        }
    });

    let tag_name_for_handlers = props.tag.clone();

    rsx! {
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",

            {render_header_section(&tag_name_for_handlers, contents)}

            {render_content_section(contents)}
        }
    }
}

#[component]
fn HeaderSection(tag_name: String, contents: Resource<Result<Vec<Content>, String>>) -> Element {
    let navigator = use_navigator();
    rsx! {
        div {
            class: "md:flex md:items-center md:justify-between py-8",

            div {
                class: "flex-1 min-w-0",

                {render_title_section(&tag_name)}
            }

            {render_action_buttons(&tag_name, contents)}
        }
    }
}

fn render_title_section(tag_name: &str) -> Element {
    rsx! {
        if !tag_name.is_empty() {
            h2 {
                class: "text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate flex items-center gap-3",
                "Content tagged with '{tag_name}'"

                span {
                    class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800",
                    "Filtered"
                }
            }

            p {
                class: "mt-1 text-sm text-gray-500",
                "Showing all content items tagged with '{tag_name}'"
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
}

#[component]
fn ActionButtons(tag_name: String, contents: Resource<Result<Vec<Content>, String>>) -> Element {
    let navigator = use_navigator();
    rsx! {
        div {
            class: "mt-4 flex md:mt-0 md:ml-4 space-x-3",

            RefreshButton { contents }

            BackButton { tag_name: tag_name.clone() }
        }
    }
}

#[component]
fn RefreshButton(contents: Resource<Result<Vec<Content>, String>>) -> Element {
    rsx! {
        button {
            onclick: move |_: MouseEvent| {
                contents.restart();
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
                    stroke_width: 2,
                    d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                }
            }
            "Refresh"
        }
    }
}

#[component]
fn BackButton(tag_name: String) -> Element {
    let navigator = use_navigator();
    rsx! {
        if !tag_name.is_empty() {
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

fn render_header_section(
    tag_name: &str,
    contents: Resource<Result<Vec<Content>, String>>,
) -> Element {
    rsx! {
        HeaderSection {
            tag_name: tag_name.to_string(),
            contents,
        }
    }
}

fn render_action_buttons(
    tag_name: &str,
    contents: Resource<Result<Vec<Content>, String>>,
) -> Element {
    rsx! {
        ActionButtons {
            tag_name: tag_name.to_string(),
            contents,
        }
    }
}

fn render_content_section(contents: Resource<Result<Vec<Content>, String>>) -> Element {
    let contents_state = contents();

    match contents_state {
        None => rsx! { LoadingSpinner {} },
        Some(Ok(content_list)) => rsx! { ContentListComponent { contents: content_list } },
        Some(Err(err)) => rsx! { ErrorSection { error: err.clone(), contents } },
    }
}

#[component]
fn LoadingSpinner() -> Element {
    rsx! {
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
                    stroke_width: 4,
                }

                path {
                    class: "opacity-75",
                    fill: "currentColor",
                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                }
            }
        }
    }
}

#[component]
fn ErrorSection(error: String, contents: Resource<Result<Vec<Content>, String>>) -> Element {
    rsx! {
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
                    stroke_width: 2,
                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                }
            }

            h3 {
                class: "mt-2 text-sm font-medium text-gray-900",
                "Error loading content"
            }

            p {
                class: "mt-1 text-sm text-gray-500",
                "{error}"
            }

            button {
                onclick: move |_: MouseEvent| {
                    contents.restart();
                },
                class: "mt-4 inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                "Try again"
            }
        }
    }
}

async fn fetch_content_by_tag_name(
    content_context: ContentContext,
    content_tags_context: ContentTagsContext,
    tag_context: TagContext,
    tag_name: &str,
) -> Result<Vec<Content>, String> {
    let all_tags = tag_context.get_all_tags().await?;

    let tag = all_tags
        .iter()
        .find(|t| t.name == tag_name)
        .ok_or_else(|| format!("Tag '{}' not found", tag_name))?;

    let tag_id = tag
        .id
        .ok_or_else(|| format!("Tag '{}' has no ID", tag_name))?;

    let content_ids = content_tags_context.get_content_ids_for_tag(tag_id).await?;

    if content_ids.is_empty() {
        return Ok(Vec::new());
    }

    content_context.get_content_by_ids(&content_ids).await
}
