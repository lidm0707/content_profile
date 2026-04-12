use crate::components::{
    ContentList as ContentListComponent, NotificationCard, NotificationVariant, StatCard,
};
use crate::contexts::TagContext;
use crate::contexts::UserContext;
use crate::models::{Content, Tag};
use crate::routes::Route;
use crate::services::{ContentService, SyncService};
use crate::utils::config::{Config, get_config};

use dioxus::prelude::*;
use dioxus_router::Navigator;

/// Tags section component - displays all available tags
fn render_tags_section(
    tags_result: Option<Result<Vec<Tag>, String>>,
    navigator: Navigator,
) -> Element {
    match tags_result {
        Some(Ok(all_tags)) => {
            if all_tags.is_empty() {
                rsx! {
                    div {
                        class: "text-center py-8 bg-white rounded-lg shadow",
                        p {
                            class: "text-gray-500",
                            "No tags found. Create your first tag to get started."
                        }
                    }
                }
            } else {
                rsx! {
                    div {
                        class: "flex flex-wrap gap-2",
                        for tag in all_tags {
                            button {
                                class: "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-indigo-100 text-indigo-800 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors",
                                onclick: move |_| {
                                    navigator.push(Route::ContentList {
                                        tag: tag.name.clone(),
                                    });
                                },
                                "{tag.name}"
                            }
                        }
                    }
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
    config: Config,
    on_sync: EventHandler<MouseEvent>,
    sync_result: Resource<Result<(), String>>,
    on_refresh: EventHandler<MouseEvent>,
}

/// Dashboard header component - displays title, mode badge, and action buttons
#[component]
fn DashboardHeader(props: DashboardHeaderProps) -> Element {
    let navigator = use_navigator();
    rsx! {
        div {
            class: "md:flex md:items-center md:justify-between py-8",

            div {
                class: "flex-1 min-w-0",

                h2 {
                    class: "text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate flex items-center gap-3",
                    "Content Dashboard"

                    if props.config.is_office_mode() {
                        span {
                            class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800",
                            "Office Mode"
                        }
                    } else if props.config.is_supabase_mode() {
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

                if props.config.is_office_mode() || props.config.is_sync_enabled() {
                    button {
                        onclick: props.on_sync,
                        disabled: props.sync_result.read().is_none(),
                        class: "inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed",

                        if props.sync_result.read().is_none() {
                            svg {
                                class: "-ml-1 mr-2 h-5 w-5 text-gray-500 animate-spin",
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
                                    d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                }
                            }
                            "Syncing..."
                        } else {
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
                            "Sync"
                        }
                    }
                }

                button {
                    onclick: props.on_refresh,
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

                Link {
                    to: Route::ContentEdit { id: 0 },
                    class: "inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",

                    svg {
                        class: "-ml-1 mr-2 h-5 w-5",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",

                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            "stroke-width": 2,
                            d: "M12 6v6m0 0v6m0-6h6m-6 0H6"
                        }
                    }
                    "Create Content"
                }
            }
        }
    }
}

/// Dashboard page component - main content management interface
#[component]
pub fn Dashboard() -> Element {
    let navigator = use_navigator();

    use_effect(move || {
        if !UserContext::has_valid_saved_session() {
            navigator.push(Route::Login {});
        }
    });

    let config = get_config();
    let content_service = ContentService::new();

    // let refresh_trigger: Signal<u64> = use_context::<Signal<u64>>();
    let mut contents = use_resource(move || {
        let content_service = content_service.clone();
        async move { content_service.get_all_content().await }
    });

    let mut error_message = use_signal(|| None::<String>);
    let mut contents_data = use_signal(Vec::<Content>::new);
    let tag_context: TagContext = use_context();
    let tags = use_resource(move || {
        let tag_context = tag_context.clone();
        async move { tag_context.get_all_tags().await }
    });
    let mut sync_result = use_resource(move || async move {
        let mut sync_service = SyncService::new();
        sync_service.sync_bidirectional().await
    });

    use_effect(move || {
        if let Some(result) = contents.read().as_ref() {
            match result {
                Ok(data) => {
                    error_message.set(None);
                    contents_data.set(data.clone());
                }
                Err(err) => {
                    error_message.set(Some(err.clone()));
                }
            }
        }
    });

    let handle_refresh = move |_| {
        contents.restart();
    };

    let handle_sync = move |_| {
        sync_result.restart();
    };

    rsx! {
        // Page header with mode indicator
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
            DashboardHeader {
                config,
                on_sync: handle_sync,
                sync_result: sync_result,
                on_refresh: handle_refresh,
            }
        }

        // Sync status notification
        if let Some(Ok(())) = sync_result.read().as_ref() {
            NotificationCard {
                variant: NotificationVariant::Success,
                message: "Sync completed successfully".to_string(),
                on_dismiss: move |_| sync_result.restart(),
            }
        }

        if let Some(Err(err)) = sync_result.read().as_ref() {
            NotificationCard {
                variant: NotificationVariant::Error,
                message: format!("Sync failed: {}", err),
                on_dismiss: move |_| sync_result.restart(),
            }
        }

        // Stats cards
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",

            div {
                class: "grid grid-cols-1 gap-5 sm:grid-cols-3 mb-8",

                StatCard {
                    label: "Total Content".to_string(),
                    value: contents_data.read().len().to_string(),
                    value_color: "text-gray-900".to_string(),
                }

                StatCard {
                    label: "Published".to_string(),
                    value: contents_data.read().iter().filter(|c| c.status == "published").count().to_string(),
                    value_color: "text-green-600".to_string(),
                }

                StatCard {
                    label: "Drafts".to_string(),
                    value: contents_data.read().iter().filter(|c| c.status == "draft").count().to_string(),
                    value_color: "text-yellow-600".to_string(),
                }
                StatCard {
                    label: "Local Only".to_string(),
                    value: contents_data.read().iter().filter(|c| c.synced_at.is_none()).count().to_string(),
                    value_color: "text-gray-600".to_string(),
                }
                StatCard {
                    label: "Synced".to_string(),
                    value: contents_data.read().iter().filter(|c| c.synced_at.is_some()).count().to_string(),
                    value_color: "text-blue-600".to_string(),
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

            {render_tags_section(tags(), navigator)}
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
                    ContentListComponent {
                        contents: contents_data.read().clone()
                    }
                }
            }
        }
    }
}
