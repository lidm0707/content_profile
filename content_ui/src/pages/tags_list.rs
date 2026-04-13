use crate::contexts::TagContext;
use crate::routes::Route;
use content_sdk::models::Tag;
use dioxus::prelude::*;
use tracing::{debug, warn};

/// Page header component with title and create button
#[component]
fn TagsListHeader(on_create: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            class: "md:flex md:items-center md:justify-between mb-6",

            div {
                class: "flex-1 min-w-0",

                h2 {
                    class: "text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate",
                    "Tags Management"
                }

                p {
                    class: "mt-1 text-sm text-gray-500",
                    "Manage your content tags and hierarchies"
                }
            }

            div {
                class: "mt-4 flex md:mt-0 md:ml-4 space-x-3",

                button {
                    onclick: on_create,
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
                            d: "M12 4v16m8-8H4"
                        }
                    }

                    "Create New Tag"
                }
            }
        }
    }
}

/// Search bar component
#[component]
fn SearchBar(query: Signal<String>, on_input: EventHandler<Event<FormData>>) -> Element {
    rsx! {
        div {
            class: "mb-6",

            div {
                class: "relative",

                input {
                    r#type: "text",
                    value: "{query}",
                    oninput: on_input,
                    class: "block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                    placeholder: "Search tags by name or slug..."
                }

                div {
                    class: "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none",

                    svg {
                        class: "h-5 w-5 text-gray-400",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",

                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            "stroke-width": 2,
                            d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        }
                    }
                }
            }
        }
    }
}

/// Message alert component for success and error messages
#[component]
fn MessageAlert(message: Option<String>, message_type: String) -> Element {
    let base_class = if message_type == "success" {
        "bg-green-50"
    } else {
        "bg-red-50"
    };

    let title_color = if message_type == "success" {
        "text-green-800"
    } else {
        "text-red-800"
    };

    let body_color = if message_type == "success" {
        "text-green-700"
    } else {
        "text-red-700"
    };

    let icon_color = if message_type == "success" {
        "text-green-400"
    } else {
        "text-red-400"
    };

    let icon_path = if message_type == "success" {
        "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
    } else {
        "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
    };

    let title = if message_type == "success" {
        "Success"
    } else {
        "Error"
    };

    rsx! {
        if let Some(msg) = message {
            div {
                class: "mb-6 rounded-md {base_class} p-4",

                div {
                    class: "flex",

                    div {
                        class: "flex-shrink-0",

                        svg {
                            class: "h-5 w-5 {icon_color}",
                            view_box: "0 0 20 20",
                            fill: "currentColor",

                            path {
                                fill_rule: "evenodd",
                                d: "{icon_path}",
                                clip_rule: "evenodd"
                            }
                        }
                    }

                    div {
                        class: "ml-3",

                        h3 {
                            class: "text-sm font-medium {title_color}",
                            "{title}"
                        }

                        div {
                            class: "mt-2 text-sm {body_color}",
                            "{msg}"
                        }
                    }
                }
            }
        }
    }
}

/// Loading spinner component
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
                    "stroke-width": 4
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

/// Empty state component for no tags or no search results
#[component]
fn EmptyState(empty_type: String, on_create: EventHandler<MouseEvent>) -> Element {
    if empty_type == "no_results" {
        rsx! {
            div {
                class: "text-center py-12",

                svg {
                    class: "mx-auto h-12 w-12 text-gray-400",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",

                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "stroke-width": 2,
                        d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    }
                }

                h3 {
                    class: "mt-2 text-sm font-medium text-gray-900",
                    "No tags found"
                }

                p {
                    class: "mt-1 text-sm text-gray-500",
                    "Try adjusting your search query"
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "text-center py-12",

                svg {
                    class: "mx-auto h-12 w-12 text-gray-400",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",

                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "stroke-width": 2,
                        d: "M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"
                    }
                }

                h3 {
                    class: "mt-2 text-sm font-medium text-gray-900",
                    "No tags yet"
                }

                p {
                    class: "mt-1 text-sm text-gray-500",
                    "Get started by creating your first tag"
                }

                div {
                    class: "mt-6",

                    button {
                        onclick: on_create,
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
                                d: "M12 4v16m8-8H4"
                            }
                        }

                        "Create Your First Tag"
                    }
                }
            }
        }
    }
}

/// Delete confirmation dialog component
#[component]
fn DeleteConfirmDialog(
    tag: Tag,
    on_confirm: EventHandler<i32>,
    on_close: EventHandler<MouseEvent>,
    is_deleting: bool,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50",

            div {
                class: "bg-white rounded-lg shadow-xl transform transition-all max-w-md w-full mx-4",

                div {
                    class: "bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4",

                    div {
                        class: "sm:flex sm:items-start",

                        div {
                            class: "mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10",

                            svg {
                                class: "h-6 w-6 text-red-600",
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
                        }

                        div {
                            class: "mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left",

                            h3 {
                                class: "text-lg leading-6 font-medium text-gray-900",
                                "Delete Tag"
                            }

                            div {
                                class: "mt-2",

                                p {
                                    class: "text-sm text-gray-500",
                                    "Are you sure you want to delete the tag \"{tag.name}\"? This will also remove it from any content pieces that use it."
                                }
                            }
                        }
                    }
                }

                div {
                    class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",

                    button {
                        r#type: "button",
                        onclick: move |_| {
                            if let Some(id) = tag.id {
                                on_confirm(id);
                            }
                        },
                        disabled: is_deleting,
                        class: "w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed",

                        if is_deleting {
                            svg {
                                class: "animate-spin -ml-1 mr-2 h-4 w-4 text-white",
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

                        "Delete"
                    }

                    button {
                        r#type: "button",
                        onclick: on_close,
                        disabled: is_deleting,
                        class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed",
                        "Cancel"
                    }
                }
            }
        }
    }
}

/// Tags list table component with header and tag rows
#[component]
fn TagsListTable(
    root_tags: ReadSignal<Vec<Tag>>,
    child_tags: ReadSignal<Vec<Tag>>,
    handle_edit: EventHandler<i32>,
    handle_delete: EventHandler<i32>,
) -> Element {
    rsx! {
        div {
            class: "bg-white shadow rounded-lg overflow-hidden",

            // Table header
            div {
                class: "hidden sm:block bg-gray-50 px-6 py-3 border-b border-gray-200",

                div {
                    class: "grid grid-cols-12 gap-4",

                    div {
                        class: "col-span-5 text-xs font-medium text-gray-500 uppercase tracking-wider",
                        "Tag Name"
                    }

                    div {
                        class: "col-span-4 text-xs font-medium text-gray-500 uppercase tracking-wider",
                        "Slug"
                    }

                    div {
                        class: "col-span-1 text-xs font-medium text-gray-500 uppercase tracking-wider",
                        "Parent"
                    }

                    div {
                        class: "col-span-2 text-right text-xs font-medium text-gray-500 uppercase tracking-wider",
                        "Actions"
                    }
                }
            }

            // Render root tags
            for tag in root_tags().iter() {
                TagRow {
                    tag: tag.clone(),
                    child_tags: child_tags().iter().filter(|t| t.parent_id == tag.id).cloned().collect(),
                    all_children: child_tags().clone(),
                    handle_edit,
                    handle_delete,
                    depth: 0
                }
            }
        }
    }
}

/// Tags list page component - displays all tags with management options
#[component]
pub fn TagsList() -> Element {
    let navigate = use_navigator();

    let tag_context = use_context::<TagContext>();
    let tag_context_clone = tag_context.clone();

    // Resource to fetch all tags
    let tags_resource: Resource<Result<Vec<Tag>, String>> = use_resource(move || {
        let tag_context = tag_context_clone.clone();
        async move { tag_context.get_all_tags().await }
    });

    // State management
    let mut tags = use_signal(Vec::<Tag>::new);
    let mut show_delete_dialog = use_signal(|| None::<i32>);
    let mut is_deleting = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    let mut search_query = use_signal(|| String::new());

    // Update tags when resource loads
    use_effect(move || {
        if let Some(result) = tags_resource.read().as_ref() {
            match result {
                Ok(tag_list) => {
                    tags.set(tag_list.clone());
                    debug!("Loaded {} tags", tag_list.len());
                }
                Err(err) => {
                    error_message.set(Some(format!("Failed to load tags: {}", err)));
                    warn!("Failed to load tags: {}", err);
                }
            }
        }
    });

    // Filter tags based on search query
    let filtered_tags = use_memo(move || {
        let query = search_query.read().to_lowercase();
        let all_tags = tags.read().clone();

        if query.is_empty() {
            all_tags
        } else {
            all_tags
                .iter()
                .filter(|tag| {
                    tag.name.to_lowercase().contains(&query)
                        || tag.slug.to_lowercase().contains(&query)
                })
                .cloned()
                .collect()
        }
    });

    // Organize tags by parent for hierarchical display
    let root_tags = use_memo(move || {
        let all_tags = filtered_tags.read().clone();
        all_tags
            .iter()
            .filter(|tag| tag.parent_id.is_none())
            .cloned()
            .collect::<Vec<Tag>>()
    });

    let child_tags = use_memo(move || {
        let all_tags = filtered_tags.read().clone();
        all_tags
            .iter()
            .filter(|tag| tag.parent_id.is_some())
            .cloned()
            .collect::<Vec<Tag>>()
    });

    let handle_edit = move |tag_id: i32| {
        navigate.push(Route::TagsEdit { id: tag_id });
    };

    let handle_delete_confirm = move |tag_id: i32| {
        is_deleting.set(true);
        error_message.set(None);
        success_message.set(None);

        let navigate_for_spawn = navigate;
        let tag_context_for_spawn = tag_context.clone();

        spawn(async move {
            match tag_context_for_spawn.tag_service().delete_tag(tag_id).await {
                Ok(_) => {
                    success_message.set(Some("Tag deleted successfully!".to_string()));

                    // Navigate back after a short delay
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate_for_spawn.push(Route::TagsList {});
                }
                Err(err) => {
                    error_message.set(Some(format!("Failed to delete tag: {}", err)));
                }
            }

            is_deleting.set(false);
        });
    };

    let handle_create_new = move |_| {
        navigate.push(Route::TagsEdit { id: 0 });
    };

    let handle_search = move |e: Event<FormData>| {
        search_query.set(e.value());
    };

    let handle_delete_click = move |tag_id: i32| {
        show_delete_dialog.set(Some(tag_id));
    };

    let handle_close_dialog = move |_| {
        show_delete_dialog.set(None);
    };

    let root_tags_signal: ReadSignal<Vec<Tag>> = root_tags.into();
    let child_tags_signal: ReadSignal<Vec<Tag>> = child_tags.into();

    rsx! {
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",

            TagsListHeader { on_create: handle_create_new }
            SearchBar { query: search_query, on_input: handle_search }
            MessageAlert { message: success_message.read().clone(), message_type: "success".to_string() }
            MessageAlert { message: error_message.read().clone(), message_type: "error".to_string() }

            if tags_resource.read().is_none() {
                LoadingSpinner {}
            } else if root_tags().is_empty() && child_tags().is_empty() && !tags().is_empty() {
                EmptyState { empty_type: "no_results".to_string(), on_create: handle_create_new }
            } else if tags().is_empty() {
                EmptyState { empty_type: "no_tags".to_string(), on_create: handle_create_new }
            } else {
                TagsListTable {
                    root_tags: root_tags_signal,
                    child_tags: child_tags_signal,
                    handle_edit,
                    handle_delete: handle_delete_click,
                }
            }

            if let Some(tag_id) = show_delete_dialog.read().cloned() {
                if let Some(tag) = tags().iter().find(|t| t.id == Some(tag_id)) {
                    DeleteConfirmDialog {
                        tag: tag.clone(),
                        on_confirm: handle_delete_confirm,
                        on_close: handle_close_dialog,
                        is_deleting: *is_deleting.read(),
                    }
                }
            }
        }
    }
}

#[component]
fn TagRow(
    tag: Tag,
    child_tags: Vec<Tag>,
    all_children: Vec<Tag>,
    handle_edit: EventHandler<i32>,
    handle_delete: EventHandler<i32>,
    depth: usize,
) -> Element {
    let indent_class = if depth > 0 {
        format!("pl-{} md:pl-{}", depth * 4, depth * 6)
    } else {
        "px-6".to_string()
    };

    let parent_tag = if let Some(parent_id) = tag.parent_id {
        all_children.iter().find(|t| t.id == Some(parent_id))
    } else {
        None
    };

    rsx! {
        div {
            class: "{indent_class} py-4 border-b border-gray-200 last:border-b-0 hover:bg-gray-50",

            // Mobile view
            div {
                class: "sm:hidden",

                div {
                    class: "flex items-center justify-between",

                    div {
                        class: "flex-1",

                        div {
                            class: "flex items-center",

                            div {
                                class: "flex-shrink-0 h-2 w-2 rounded-full bg-indigo-600"
                            }

                            p {
                                class: "ml-3 text-sm font-medium text-indigo-600 truncate",
                                "{tag.name}"
                            }
                        }

                        if let Some(parent) = parent_tag {
                            p {
                                class: "ml-3 mt-1 text-xs text-gray-500",
                                "Parent: {parent.name}"
                            }
                        }
                    }

                    div {
                        class: "ml-4 flex items-center space-x-2",

                        button {
                            onclick: move |_| {
                                if let Some(id) = tag.id {
                                    handle_edit(id);
                                }
                            },
                            class: "text-indigo-600 hover:text-indigo-900 text-sm font-medium",
                            "Edit"
                        }

                        button {
                            onclick: move |_| {
                                if let Some(id) = tag.id {
                                    handle_delete(id);
                                }
                            },
                            class: "text-red-600 hover:text-red-900 text-sm font-medium",
                            "Delete"
                        }
                    }
                }

                div {
                    class: "mt-2 sm:mt-0",

                    p {
                        class: "text-sm text-gray-500",
                        "{tag.slug}"
                    }
                }
            }

            // Desktop view
            div {
                class: "hidden sm:block",

                div {
                    class: "grid grid-cols-12 gap-4 items-center",

                    // Tag name
                    div {
                        class: "col-span-5 flex items-center",

                        div {
                            class: "flex-shrink-0 h-2 w-2 rounded-full bg-indigo-600"
                        }

                        p {
                            class: "ml-3 text-sm font-medium text-gray-900 truncate",
                            "{tag.name}"
                        }
                    }

                    // Slug
                    div {
                        class: "col-span-4",

                        p {
                            class: "text-sm text-gray-500 truncate",
                            "{tag.slug}"
                        }
                    }

                    // Parent
                    div {
                        class: "col-span-1",

                        if let Some(parent) = parent_tag {
                            p {
                                class: "text-sm text-gray-900",
                                "{parent.name}"
                            }
                        } else {
                            p {
                                class: "text-sm text-gray-400",
                                "None"
                            }
                        }
                    }

                    // Actions
                    div {
                        class: "col-span-2 text-right",

                        div {
                            class: "flex items-center justify-end space-x-2",

                            button {
                                onclick: move |_| {
                                    if let Some(id) = tag.id {
                                        handle_edit(id);
                                    }
                                },
                                class: "text-indigo-600 hover:text-indigo-900 text-sm font-medium",
                                "Edit"
                            }

                            button {
                                onclick: move |_| {
                                    if let Some(id) = tag.id {
                                        handle_delete(id);
                                    }
                                },
                                class: "text-red-600 hover:text-red-900 text-sm font-medium",
                                "Delete"
                            }
                        }
                    }
                }
            }

            // Render child tags recursively
            for child in child_tags.iter() {
                TagRow {
                    tag: child.clone(),
                    child_tags: all_children.iter().filter(|t| t.parent_id == child.id).cloned().collect(),
                    all_children: all_children.clone(),
                    handle_edit,
                    handle_delete,
                    depth: depth + 1
                }
            }
        }
    }
}
