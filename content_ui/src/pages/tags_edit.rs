use crate::contexts::TagContext;
use crate::models::{Tag, TagRequest};
use crate::routes::Route;
use dioxus::prelude::*;
use tracing::{debug, info, warn};

/// Tag edit page component - handles both creating and editing tags
#[component]
pub fn TagsEdit(id: i32) -> Element {
    let navigate = use_navigator();
    let is_editing = id != 0;

    let tag_context = use_context::<TagContext>();
    let tag_service = tag_context.tag_service();
    let tag_service_for_resource = tag_service.clone();
    let tag_service_clone = tag_service.clone();

    // Resource to fetch tag if editing
    let tag_resource: Resource<Result<Option<Tag>, String>> = use_resource(move || {
        let tag_service = tag_service_clone.clone();
        async move {
            debug!(
                "Loading tag resource - is_editing: {}, id: {}",
                is_editing, id
            );
            if is_editing {
                info!("Fetching tag by ID: {}", id);
                let result = tag_service.get_tag_by_id(id).await;
                match &result {
                    Ok(Some(tag)) => debug!("Successfully loaded tag: {}", tag.name),
                    Ok(None) => warn!("Tag not found with ID: {}", id),
                    Err(e) => warn!("Failed to load tag: {}", e),
                }
                result
            } else {
                debug!("Creating new tag - skipping fetch");
                Ok(None)
            }
        }
    });

    // Resource to fetch all tags for parent selection
    let all_tags_resource: Resource<Result<Vec<Tag>, String>> = use_resource(move || {
        let tag_service = tag_service_for_resource.clone();
        async move { tag_service.get_all_tags().await }
    });

    // State management
    let mut current_tag = use_signal(|| None::<Tag>);
    let mut form_name = use_signal(|| String::new());
    let mut form_slug = use_signal(|| String::new());
    let mut form_parent_id = use_signal(|| None::<i32>);
    let mut is_submitting = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    let mut available_parent_tags = use_signal(Vec::<Tag>::new);

    // Update available parent tags when resource loads
    use_effect(move || {
        if let Some(result) = all_tags_resource.read().as_ref() {
            match result {
                Ok(tags) => {
                    available_parent_tags.set(tags.clone());
                }
                Err(_) => {
                    available_parent_tags.set(Vec::new());
                }
            }
        }
    });

    let page_title = if is_editing {
        "Edit Tag".to_string()
    } else {
        "Create New Tag".to_string()
    };

    let page_subtitle = if is_editing {
        "Update tag details".to_string()
    } else {
        "Fill in the details below to create a new tag".to_string()
    };

    // Update current_tag and form when tag_resource loads
    use_effect(move || {
        if let Some(result) = tag_resource.read().as_ref() {
            match result {
                Ok(Some(tag)) => {
                    current_tag.set(Some(tag.clone()));
                    form_name.set(tag.name.clone());
                    form_slug.set(tag.slug.clone());
                    form_parent_id.set(tag.parent_id);
                    debug!(
                        "Tag loaded for editing - ID: {}, name: {}, slug: {}",
                        tag.id.unwrap_or(0),
                        tag.name,
                        tag.slug
                    );
                }
                Ok(None) if !is_editing => {
                    debug!("Initializing for new tag creation");
                }
                Ok(None) => {
                    error_message.set(Some("Tag not found".to_string()));
                    warn!("Tag not found with ID: {}", id);
                }
                Err(err) => {
                    error_message.set(Some(format!("Failed to load tag: {}", err)));
                    warn!("Failed to load tag with ID {}: {}", id, err);
                }
            }
        }
    });

    // Auto-generate slug from name
    let handle_name_change = move |e: Event<FormData>| {
        let name = e.value();
        form_name.set(name.clone());

        let slug = name
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-");

        form_slug.set(slug);
    };

    let mut handle_submit = move |_| {
        is_submitting.set(true);
        error_message.set(None);
        success_message.set(None);

        let navigate_for_spawn = navigate;
        let mut tag_service_for_spawn = tag_service.clone();
        let name = form_name.cloned();
        let slug = form_slug.cloned();
        let parent_id = form_parent_id.cloned();
        let is_editing_for_spawn = is_editing;
        let id_for_spawn = id;

        spawn(async move {
            let request = TagRequest {
                id: if is_editing_for_spawn {
                    Some(id_for_spawn)
                } else {
                    None
                },
                name: name.clone(),
                slug: slug.clone(),
                parent_id,
            };

            let result = if is_editing_for_spawn {
                tag_service_for_spawn
                    .update_tag(id_for_spawn, request)
                    .await
            } else {
                tag_service_for_spawn.create_tag(request).await
            };

            match result {
                Ok(tag) => {
                    success_message.set(Some(if is_editing_for_spawn {
                        format!("Tag '{}' updated successfully!", tag.name)
                    } else {
                        format!("Tag '{}' created successfully!", tag.name)
                    }));

                    // Navigate back to dashboard after a short delay
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate_for_spawn.push(Route::Dashboard {});
                }
                Err(err) => {
                    error_message.set(Some(format!("Failed to save tag: {}", err)));
                }
            }

            is_submitting.set(false);
        });
    };

    let handle_cancel = move |_| {
        navigate.push(Route::Dashboard {});
    };

    rsx! {
        div {
            div {
                class: "max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8",

                // Page header
                div {
                    class: "mb-6",

                    div {
                        class: "flex items-center justify-between",

                        div {
                            h1 {
                                class: "text-2xl font-bold text-gray-900",
                                "{page_title}"
                            }

                            p {
                                class: "mt-1 text-sm text-gray-500",
                                "{page_subtitle}"
                            }
                        }

                        Link {
                            to: Route::Dashboard {},
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
                                    d: "M10 19l-7-7m0 0l7-7m-7 7h18"
                                }
                            }

                            "Back to Dashboard"
                        }
                    }
                }

                // Success message
                if let Some(message) = success_message.read().as_ref() {
                    div {
                        class: "mb-6 rounded-md bg-green-50 p-4",

                        div {
                            class: "flex",

                            div {
                                class: "flex-shrink-0",

                                svg {
                                    class: "h-5 w-5 text-green-400",
                                    view_box: "0 0 20 20",
                                    fill: "currentColor",

                                    path {
                                        fill_rule: "evenodd",
                                        d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z",
                                        clip_rule: "evenodd"
                                    }
                                }
                            }

                            div {
                                class: "ml-3",

                                h3 {
                                    class: "text-sm font-medium text-green-800",
                                    "Success"
                                }

                                div {
                                    class: "mt-2 text-sm text-green-700",
                                    "{message}"
                                }
                            }
                        }
                    }
                }

                // Error message
                if let Some(message) = error_message.read().as_ref() {
                    div {
                        class: "mb-6 rounded-md bg-red-50 p-4",

                        div {
                            class: "flex",

                            div {
                                class: "flex-shrink-0",

                                svg {
                                    class: "h-5 w-5 text-red-400",
                                    view_box: "0 0 20 20",
                                    fill: "currentColor",

                                    path {
                                        fill_rule: "evenodd",
                                        d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z",
                                        clip_rule: "evenodd"
                                    }
                                }
                            }

                            div {
                                class: "ml-3",

                                h3 {
                                    class: "text-sm font-medium text-red-800",
                                    "Error"
                                }

                                div {
                                    class: "mt-2 text-sm text-red-700",
                                    "{message}"
                                }
                            }
                        }
                    }
                }

                // Loading state when editing
                if is_editing && tag_resource.read().is_none() {
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
                } else {
                    // Tag form
                    div {
                        class: "bg-white shadow rounded-lg p-6",

                        form {
                            onsubmit: move |e: Event<FormData>| {
                                e.prevent_default();
                                handle_submit(());
                            },

                            // Name field
                            div {
                                class: "mb-6",

                                label {
                                    class: "block text-sm font-medium text-gray-700 mb-1",
                                    "Name"
                                }

                                input {
                                    r#type: "text",
                                    value: "{form_name}",
                                    oninput: handle_name_change,
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500",
                                    placeholder: "Enter tag name",
                                    disabled: *is_submitting.read()
                                }

                                p {
                                    class: "mt-1 text-xs text-gray-500",
                                    "The display name for your tag"
                                }
                            }

                            // Slug field
                            div {
                                class: "mb-6",

                                label {
                                    class: "block text-sm font-medium text-gray-700 mb-1",
                                    "Slug"
                                }

                                input {
                                    r#type: "text",
                                    value: "{form_slug}",
                                    oninput: move |e: Event<FormData>| form_slug.set(e.value()),
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500",
                                    placeholder: "tag-slug",
                                    disabled: *is_submitting.read()
                                }

                                p {
                                    class: "mt-1 text-xs text-gray-500",
                                    "URL-friendly version of the name"
                                }
                            }

                            // Parent tag field
                            div {
                                class: "mb-6",

                                label {
                                    class: "block text-sm font-medium text-gray-700 mb-1",
                                    "Parent Tag"
                                }

                                select {
                                    value: "{form_parent_id.read().map_or(String::new(), |id| id.to_string())}",
                                    onchange: move |e: Event<FormData>| {
                                        let value = e.value();
                                        if value.is_empty() {
                                            form_parent_id.set(None);
                                        } else {
                                            form_parent_id.set(value.parse().ok());
                                        }
                                    },
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500",
                                    disabled: *is_submitting.read(),

                                    option {
                                        value: "",
                                        "No parent (root tag)"
                                    }

                                    for tag in available_parent_tags.read().iter() {
                                        // Don't show the current tag as a parent option
                                        if tag.id != Some(id) {
                                            option {
                                                value: "{tag.id.map_or(String::new(), |id| id.to_string())}",
                                                selected: form_parent_id() == tag.id,
                                                "{tag.name}"
                                            }
                                        }
                                    }
                                }

                                p {
                                    class: "mt-1 text-xs text-gray-500",
                                    "Optional: Select a parent tag to create a hierarchy"
                                }
                            }

                            // Form actions
                            div {
                                class: "flex items-center justify-end space-x-3 pt-4 border-t border-gray-200",

                                button {
                                    r#type: "button",
                                    onclick: handle_cancel,
                                    class: "px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                                    disabled: *is_submitting.read(),
                                    "Cancel"
                                }

                                button {
                                    r#type: "submit",
                                    disabled: *is_submitting.read() || form_name.read().is_empty(),
                                    class: "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed",

                                    if *is_submitting.read() {
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

                                    {if is_editing { "Update Tag" } else { "Create Tag" }}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
