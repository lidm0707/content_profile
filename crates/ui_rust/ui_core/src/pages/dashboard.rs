use crate::components::{ContentTable, Pagination, StatCard, TagPills};
use crate::routes::Route;
use content_sdk::contexts::{ContentContext, TagContext};
use content_sdk::models::{Content, Session};
use content_sdk::utils::config::{AppMode, Config};
use dioxus::prelude::*;
use dioxus_router::Navigator;

/// Tags section component - renders all available tags via the shared
/// `TagPills` component.
fn render_tags_section(
    tags_result: Option<Result<Vec<content_sdk::models::Tag>, String>>,
    active_tag: String,
    navigator: Navigator,
) -> Element {
    match tags_result {
        Some(Ok(all_tags)) => {
            rsx! {
                TagPills {
                    tags: all_tags,
                    active_tag,
                    on_click: move |name: String| {
                        navigator.push(Route::ContentList { tag: name });
                    },
                }
            }
        }
        Some(Err(_)) => {
            rsx! {
                div {
                    class: "text-center py-8 bg-red-50 rounded-lg shadow",
                    p {
                        class: "text-red-600",
                        "Failed to load tags. Please try again later."
                    }
                }
            }
        }
        None => {
            rsx! {
                div {
                    class: "flex items-center justify-center py-8 bg-white rounded-lg shadow",
                    svg {
                        class: "animate-spin h-8 w-8 text-indigo-600",
                        fill: "none",
                        view_box: "0 0 24 24",

                        circle {
                            class: "opacity-25",
                            cx: "12",
                            cy: "12",
                            r: "10",
                            stroke: "currentColor",
                            "stroke-width": "4",
                        }

                        path {
                            class: "opacity-75",
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                        }
                    }
                }
            }
        }
    }
}

/// Props for dashboard header component
#[derive(Clone, PartialEq, Props)]
struct DashboardHeaderProps {
    is_office_mode: bool,
    is_supabase_mode: bool,
    on_refresh: EventHandler<MouseEvent>,
}

/// Dashboard header component - displays title, mode badge, and action buttons
#[component]
fn DashboardHeader(props: DashboardHeaderProps) -> Element {
    let _navigator = use_navigator();
    rsx! {
        div {
            class: "md:flex md:items-center md:justify-between py-8",

            div {
                class: "flex-1 min-w-0",

                h2 {
                    class: "text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate flex items-center gap-3",
                    "Content Dashboard"

                    if props.is_office_mode {
                        span {
                            class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800",
                            "Office Mode"
                        }
                    } else if props.is_supabase_mode {
                        span {
                            class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800",
                            "Supabase Mode"
                        }
                    }
                }

                p {
                    class: "mt-1 text-sm text-gray-500",
                    "Manage and organize all your content items"
                }
            }

            div {
                class: "mt-4 flex md:mt-0 md:ml-4 space-x-3",

                button {
                    onclick: props.on_refresh,
                    class: "inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",

                    svg {
                        class: "-ml-1 mr-2 h-5 w-5 text-gray-500",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            "stroke-width": "2",
                            d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15",
                        }
                    }
                    "Refresh"
                }
            }
        }
    }
}

/// Dashboard page component - main content management interface
#[component]
pub fn Dashboard() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let session_checked: Signal<bool> = use_context();

    // Redirect only once the restore probe has finished and found no
    // session — a None before that just means "still checking".
    use_effect(move || {
        if session_checked() && session().is_none() {
            navigator.push(Route::Login {});
        }
    });

    let content_context: ContentContext = use_context();
    let mut current_page = use_signal(|| 1);
    let page_size = 9;

    let contents_context = content_context.clone();
    let mut contents = use_resource(move || {
        let content_context = contents_context.clone();
        let page = current_page();
        async move {
            content_context
                .get_paginated_content(&[], page, page_size)
                .await
        }
    });

    let mut error_message = use_signal(|| None::<String>);
    let mut contents_data = use_signal(Vec::<Content>::new);
    let tag_context: TagContext = use_context();
    let mut tags = use_resource(move || {
        let tag_context = tag_context.clone();
        async move { tag_context.get_all_tags().await }
    });

    let config = use_context::<Memo<Config>>();
    let mode = config.read().mode;
    let is_office_mode = mode == AppMode::Office;
    let is_supabase_mode = mode == AppMode::Supabase;

    use_effect(move || {
        if let Some(result) = contents.read().as_ref() {
            match result {
                Ok(data) => {
                    error_message.set(None);
                    contents_data.set(data.data.clone());
                }
                Err(err) => {
                    error_message.set(Some(err.clone()));
                }
            }
        }
    });

    let handle_refresh = move |_| {
        contents.restart();
        tags.restart();
    };

    rsx! {
        // Page header with mode indicator
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
            DashboardHeader {
                is_office_mode,
                is_supabase_mode,
                on_refresh: handle_refresh,
            }
        }


        // Stats — compact inline pills, wrap on small screens.
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",

            div {
                class: "flex flex-wrap items-center gap-2 mb-8",

                StatCard {
                    label: "Total".to_string(),
                    value: contents.read().as_ref().and_then(|r| r.as_ref().ok()).map(|r| r.total_items).unwrap_or(0).to_string(),
                    value_color: "text-gray-900".to_string(),
                    icon_bg: "bg-gray-100".to_string(),
                    icon_path: "M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z".to_string(),
                }

                StatCard {
                    label: "Published".to_string(),
                    value: contents_data.read().iter().filter(|c| c.status == "published").count().to_string(),
                    value_color: "text-green-600".to_string(),
                    icon_bg: "bg-green-100".to_string(),
                    icon_path: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z".to_string(),
                }

                StatCard {
                    label: "Drafts".to_string(),
                    value: contents_data.read().iter().filter(|c| c.status == "draft").count().to_string(),
                    value_color: "text-yellow-600".to_string(),
                    icon_bg: "bg-yellow-100".to_string(),
                    icon_path: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z".to_string(),
                }

                StatCard {
                    label: "Local Only".to_string(),
                    value: contents_data.read().iter().filter(|c| c.synced_at.is_none()).count().to_string(),
                    value_color: "text-gray-600".to_string(),
                    icon_bg: "bg-gray-100".to_string(),
                    icon_path: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4".to_string(),
                }

                StatCard {
                    label: "Synced".to_string(),
                    value: contents_data.read().iter().filter(|c| c.synced_at.is_some()).count().to_string(),
                    value_color: "text-blue-600".to_string(),
                    icon_bg: "bg-blue-100".to_string(),
                    icon_path: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12".to_string(),
                }
            }


        }

        // Tags section
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8",

            h2 {
                class: "text-lg leading-6 font-medium text-gray-900 mb-4",
                "Tags"
            }

            {render_tags_section(tags(), String::new(), navigator)}
        }



        // Content list or loading/error state
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",

            if contents.read().is_none() {
                // Initial loading state
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
            } else if let Some(result) = contents.read().as_ref() {
                if result.is_err() {
                    // Error state
                    if let Some(error) = error_message.read().as_ref() {
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
                                "{error}"
                            }

                            button {
                                onclick: handle_refresh,
                                class: "mt-4 inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                                "Try again"
                            }
                        }
                    }
                } else {
                    // Content list
                    ContentTable {
                        contents: contents_data.read().clone(),
                        // The Dashboard pages server-side below; the table's
                        // own footer would be a second, clashing control.
                        show_pagination: false,
                        on_edit: move |id: i32| {
                            navigator.push(Route::ContentEdit { id });
                        },
                    }

                    // Server-side pagination controls (same visual style
                    // as the table's internal pagination).
                    {
                        let total = contents.read().as_ref().and_then(|r| r.as_ref().ok()).map(|r| r.total_items as usize).unwrap_or(0);
                        let current = current_page() as usize;
                        let max_page = if total > 0 {
                            total.div_ceil(page_size as usize)
                        } else {
                            1
                        };

                        if max_page > 1 {
                            Some(rsx! {
                                div {
                                    class: "mt-2",
                                    Pagination {
                                        current_page: current,
                                        total_pages: max_page,
                                        total_items: total,
                                        page_size: page_size as usize,
                                        on_prev: move |_| {
                                            if current_page() > 1 {
                                                current_page -= 1;
                                            }
                                        },
                                        on_next: move |_| {
                                            if (current_page() as usize) < max_page {
                                                current_page += 1;
                                            }
                                        },
                                    }
                                }
                            })
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }
}
