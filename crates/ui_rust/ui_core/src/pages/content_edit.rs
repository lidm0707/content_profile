use crate::components::ContentForm;
use crate::routes::Route;
use content_sdk::contexts::ContentContext;
use content_sdk::contexts::ContentTagsContext;
use content_sdk::models::{Content, ContentRequest};
use dioxus::prelude::*;
use tracing::{debug, info, warn};

/// Content edit page component - handles both creating and editing content
#[component]
pub fn ContentEdit(id: i32) -> Element {
    let navigate = use_navigator();
    let is_editing = id != 0;
    let mut refresh_trigger = use_context::<Signal<u64>>();
    let content_context = use_context::<ContentContext>();
    let content_tags_context = use_context::<ContentTagsContext>();

    // Clone contexts for use in resource closures
    let content_context_for_content_resource = content_context.clone();
    // Resource to fetch content if editing
    let content_resource = use_resource(move || {
        let content_context = content_context_for_content_resource.clone();
        async move {
            debug!(
                "Loading content resource - is_editing: {}, id: {}",
                is_editing, id
            );
            if is_editing {
                info!("Fetching content by ID: {}", id);
                let result = content_context.get_content_by_id(id).await;
                match &result {
                    Ok(Some(content)) => debug!("Successfully loaded content: {}", content.title),
                    Ok(None) => warn!("Content not found with ID: {}", id),
                    Err(e) => warn!("Failed to load content: {}", e),
                }
                result
            } else {
                debug!("Creating new content - skipping fetch");
                Ok(None)
            }
        }
    });

    // State management
    let mut current_content = use_signal(|| None::<Content>);
    let mut is_submitting = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);

    let page_title = if is_editing {
        "Edit Content".to_string()
    } else {
        "Create New Content".to_string()
    };

    let page_subtitle = if is_editing {
        "Update your content details".to_string()
    } else {
        "Fill in the details below to create new content".to_string()
    };

    debug!(
        "ContentEdit page initialized - is_editing: {}, id: {}, title: {}",
        is_editing, id, page_title
    );

    // Update current_content when content_resource loads
    use_effect(move || {
        if let Some(result) = content_resource.read().as_ref() {
            match result {
                Ok(Some(content)) => {
                    current_content.set(Some(content.clone()));
                    debug!(
                        "Content loaded for editing - ID: {}, title: {}, status: {}",
                        content.id.unwrap_or(0),
                        content.title,
                        content.status
                    );
                }
                Ok(None) if !is_editing => {
                    debug!("Initializing for new content creation - status will default to draft");
                }
                Ok(None) => {
                    error_message.set(Some("Content not found".to_string()));
                    warn!("Content not found with ID: {}", id);
                }
                Err(err) => {
                    error_message.set(Some(format!("Failed to load content: {}", err)));
                    warn!("Failed to load content with ID {}: {}", id, err);
                }
            }
        }
    });

    let handle_form_submit = move |(request, selected_tag_ids): (ContentRequest, Vec<i32>)| {
        is_submitting.set(true);
        error_message.set(None);
        success_message.set(None);

        let _current_content_for_spawn = current_content.read().clone();
        let navigate_for_spawn = navigate;
        let mut content_context_for_spawn = content_context.clone();
        let mut content_tags_context_for_spawn = content_tags_context.clone();
        let is_editing_for_spawn = is_editing;

        // Spawn an async task to handle the submission
        async move {
            let result = if let Some(id) = request.id {
                content_context_for_spawn.update_content(id, request).await
            } else {
                let request = ContentRequest {
                    id: None,
                    ..request
                };
                content_context_for_spawn.create_content(request).await
            };

            match result {
                Ok(content) => {
                    let content_id = content.id.unwrap();

                    // Sync tags with content_tags table
                    if let Err(err) = content_tags_context_for_spawn
                        .update_content_tags(content_id, selected_tag_ids)
                        .await
                    {
                        warn!("Failed to sync tags: {}", err);
                        error_message.set(Some(format!(
                            "Content saved but failed to sync tags: {}",
                            err
                        )));
                    } else {
                        success_message.set(Some(if is_editing_for_spawn {
                            "Content updated successfully!".to_string()
                        } else {
                            "Content created successfully!".to_string()
                        }));
                    }

                    // Trigger content refresh for dashboard
                    *refresh_trigger.write() += 1;

                    // Navigate back to dashboard after a short delay
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    navigate_for_spawn.push(Route::Dashboard {});
                }
                Err(err) => {
                    error_message.set(Some(format!("Failed to save content: {}", err)));
                }
            }

            is_submitting.set(false);
        }
    };

    let handle_cancel = move |_| {
        navigate.push(Route::Dashboard {});
    };

    rsx! {
        div {

            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",

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
                if is_editing && content_resource.read().is_none() {
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
                    // Content form
                    ContentForm {
                        content: current_content.read().clone(),
                        on_submit: handle_form_submit,
                        on_cancel: handle_cancel
                    }
                }
            }
        }
    }
}
