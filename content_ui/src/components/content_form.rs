use content_sdk::ContentTagsContext;
use content_sdk::TagContext;
use content_sdk::models::{Content, ContentRequest, STATUS_DRAFT, STATUS_PUBLISHED, Tag};
use content_sdk::utils::markdown::update_tags_in_frontmatter;
use content_sdk::utils::{
    format_blockquote, format_bold, format_code, format_code_block, format_heading, format_image,
    format_italic, format_link, format_ordered_list, format_unordered_list,
    render_markdown_to_html,
};
use dioxus::prelude::*;
use tracing::debug;

/// Props for the content form component
#[derive(Clone, PartialEq, Props)]
pub struct ContentFormProps {
    /// Optional content for editing (None for creating new content)
    pub content: ReadSignal<Option<Content>>,
    /// Callback when form is submitted successfully
    pub on_submit: EventHandler<ContentRequest>,
    /// Callback when form is cancelled
    pub on_cancel: EventHandler<()>,
}

/// Content form component for creating and editing content
#[component]
pub fn ContentForm(props: ContentFormProps) -> Element {
    let tag_context = use_context::<TagContext>();
    let content_tags_context = use_context::<ContentTagsContext>();

    debug!("ContentForm rendered");

    let mut title = use_signal(|| {
        props
            .content
            .read()
            .as_ref()
            .map(|c| c.title.clone())
            .unwrap_or_default()
    });
    let mut slug = use_signal(|| {
        props
            .content
            .read()
            .as_ref()
            .map(|c| c.slug.clone())
            .unwrap_or_default()
    });
    let mut body = use_signal(|| {
        props
            .content
            .read()
            .as_ref()
            .map(|c| c.body.clone())
            .unwrap_or_default()
    });
    let mut selected_tag_ids = use_signal(Vec::<i32>::new);
    let mut status = use_signal(|| {
        let status_value = props
            .content
            .read()
            .as_ref()
            .map(|c| c.status.clone())
            .unwrap_or_else(|| STATUS_DRAFT.to_string());
        debug!("Status signal initialized with value: {}", status_value);
        status_value
    });
    let mut isSubmitting = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut isPreviewMode = use_signal(|| false);

    // Fetch available_tags using resource
    let available_tags_resource = use_resource(move || {
        let context = tag_context.clone();
        async move {
            debug!("Fetching all available tags");
            match context.get_all_tags().await {
                Ok(tags) => {
                    debug!(
                        "Successfully fetched {} available tags: {:?}",
                        tags.len(),
                        tags
                    );
                    tags
                }
                Err(e) => {
                    warn!("Failed to fetch available tags: {}", e);
                    vec![]
                }
            }
        }
    });

    // Fetch content_tags using resource
    let content_tags_resource = use_resource(move || {
        let content_id = props.content.read().as_ref().and_then(|c| c.id);
        let context = content_tags_context.clone();
        async move {
            debug!("Fetching content_tags for content_id: {:?}", content_id);
            if let Some(id) = content_id {
                match context.tag_service().get_content_tags_for_content(id).await {
                    Ok(tags) => {
                        debug!(
                            "Successfully fetched {} content_tags: {:?}",
                            tags.len(),
                            tags
                        );
                        let tag_ids: Vec<i32> = tags.iter().map(|t| t.tag_id).collect();
                        debug!("Extracted tag_ids: {:?}", tag_ids);
                        tag_ids
                    }
                    Err(e) => {
                        warn!("Failed to fetch content tags: {}", e);
                        vec![]
                    }
                }
            } else {
                debug!("No content_id available, returning empty tags");
                vec![]
            }
        }
    });

    // Initialize selected_tag_ids when resource completes
    use_effect(move || {
        if let Some(tag_ids) = content_tags_resource.read().as_ref() {
            debug!("Initializing selected_tag_ids from resource: {:?}", tag_ids);
            selected_tag_ids.set(tag_ids.clone());
        }
    });

    // Get available_tags from resource
    let available_tags = use_memo(move || {
        available_tags_resource
            .read()
            .as_ref()
            .map_or(Vec::<Tag>::new(), |result| result.clone())
    });

    // Check if tags are still loading
    let tags_loading = use_memo(move || {
        available_tags_resource.read().is_none() || content_tags_resource.read().is_none()
    });

    // Pre-compute tag badges for rendering
    let tag_badges = use_memo(move || {
        let selected_ids = selected_tag_ids.read();
        let tags = available_tags.read();

        if selected_ids.is_empty() {
            vec![]
        } else {
            selected_ids
                .iter()
                .filter_map(|&tag_id| {
                    tags.iter().find(|t| t.id == Some(tag_id)).map(|tag| {
                        debug!("Found tag for id {}: {}", tag_id, tag.name);
                        (tag_id, tag.clone())
                    })
                })
                .collect::<Vec<(i32, Tag)>>()
        }
    });

    let is_editing = props.content.read().is_some();
    let title_text = if is_editing {
        "Edit Content".to_string()
    } else {
        "Create New Content".to_string()
    };
    let button_text = if is_editing {
        "Update Content".to_string()
    } else {
        "Create Content".to_string()
    };

    use_effect(move || {
        if let Some(content) = props.content.as_ref() {
            title.set(content.title.clone());
            slug.set(content.slug.clone());
            body.set(content.body.clone());
            status.set(content.status.clone());
            debug!(
                "Content effect updated - title: {}, status: {}",
                content.title, content.status
            );
        }
    });

    // Auto-generate slug from title
    let handle_title_change = move |e: Event<FormData>| {
        let new_title = e.value();
        *title.write() = new_title.clone();
        if slug.read().is_empty() {
            slug.write().clone_from(&Content::generate_slug(&new_title));
        }
    };

    // Formatting handlers
    let handle_format_bold = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_bold(&current_body);
    };

    let handle_format_italic = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_italic(&current_body);
    };

    let handle_format_code = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_code(&current_body);
    };

    let handle_format_code_block = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_code_block(&current_body);
    };

    let handle_format_heading = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_heading(&current_body, 2);
    };

    let handle_format_link = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_link(&current_body, "https://");
    };

    let handle_format_unordered_list = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_unordered_list(&current_body);
    };

    let handle_format_ordered_list = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_ordered_list(&current_body, 1);
    };

    let handle_format_blockquote = move |_| {
        let current_body = body.read().clone();
        *body.write() = format_blockquote(&current_body);
    };

    let handle_format_image = move |_| {
        let current_body = body.read().clone();
        let image_markdown = format_image("Alt text", "https://");
        *body.write() = if current_body.trim().is_empty() {
            image_markdown
        } else {
            format!("{}\n\n{}", current_body, image_markdown)
        };
    };

    let handle_submit = move |_| {
        if title.read().is_empty() {
            error_message.set(Some("Title is required".to_string()));
            return;
        }

        if body.read().is_empty() {
            error_message.set(Some("Body is required".to_string()));
            return;
        }

        isSubmitting.set(true);

        let selected_tags: Vec<Tag> = available_tags
            .read()
            .iter()
            .filter(|t| selected_tag_ids.read().contains(&t.id.unwrap()))
            .cloned()
            .collect();

        let updated_body = update_tags_in_frontmatter(&body.read(), &selected_tags);

        let current_status = status.read().clone();
        debug!(
            "Form submission - title: {}, slug: {}, status: {}, tags: {:?}",
            title.read(),
            slug.read(),
            current_status,
            selected_tag_ids.read()
        );

        let request = ContentRequest {
            id: props.content.read().as_ref().and_then(|c| c.id),
            title: title.read().clone(),
            slug: slug.read().clone(),
            body: updated_body,
            status: current_status,
        };

        debug!("ContentRequest created with status: {}", request.status);
        props.on_submit.call(request);
        isSubmitting.set(false);
    };

    let mut show_tag_selector = use_signal(|| false);

    let available_tags_to_show = use_memo(move || {
        let selected = selected_tag_ids.read();
        let tags = available_tags.read();
        let filtered = tags
            .iter()
            .filter(|tag| !selected.contains(&tag.id.unwrap()))
            .cloned()
            .collect::<Vec<Tag>>();
        debug!(
            "available_tags_to_show computed: {} tags (filtered from {} total tags)",
            filtered.len(),
            tags.len()
        );
        filtered
    });

    rsx! {
        div {
            class: "bg-white shadow rounded-lg",

            div {
                class: "px-4 py-5 sm:p-6",

                h3 {
                    class: "text-lg leading-6 font-medium text-gray-900 mb-4",
                    "{title_text}"
                }

                if let Some(error) = error_message.read().as_ref() {
                    div {
                        class: "mb-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded relative",
                        "{error}"
                    }
                }

                form {
                    onsubmit: move |e| {
                        e.prevent_default();
                    },

                    div {
                        class: "space-y-6",

                        // Title field
                        div {
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                "Title"
                            }
                            input {
                                r#type: "text",
                                value: "{title}",
                                class: "mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                                oninput: handle_title_change,
                                disabled: *isSubmitting.read()
                            }
                        }

                        // Slug field
                        div {
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                "Slug"
                            }
                            input {
                                r#type: "text",
                                value: "{slug}",
                                class: "mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                                oninput: move |e: Event<FormData>| {
                                    *slug.write() = e.value();
                                },
                                disabled: *isSubmitting.read()
                            }
                            p {
                                class: "mt-1 text-xs text-gray-500",
                                "URL-friendly version of the title (auto-generated from title if empty)"
                            }
                        }

                        // Status field
                        div {
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                "Status"
                            }
                            select {
                                value: "{status}",
                                class: "mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md",
                                onchange: move |e: Event<FormData>| {
                                    *status.write() = e.value();
                                },
                                disabled: *isSubmitting.read(),

                                option {
                                    value: STATUS_DRAFT,
                                    "Draft"
                                }
                                option {
                                    value: STATUS_PUBLISHED,
                                    "Published"
                                }
                            }
                        }

                        // Body field
                        div {
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                "Content"
                            }

                            // Preview/Edit toggle
                            div {
                                class: "mb-2 flex items-center space-x-2",

                                button {
                                    r#type: "button",
                                    class: if *isPreviewMode.read() {
                                        "px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
                                    } else {
                                        "px-3 py-1.5 text-sm border border-indigo-500 bg-indigo-50 text-indigo-700 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
                                    },
                                    onclick: move |_| {
                                        isPreviewMode.set(false);
                                    },
                                    disabled: *isSubmitting.read(),
                                    "Edit"
                                }

                                button {
                                    r#type: "button",
                                    class: if *isPreviewMode.read() {
                                        "px-3 py-1.5 text-sm border border-indigo-500 bg-indigo-50 text-indigo-700 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
                                    } else {
                                        "px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
                                    },
                                    onclick: move |_| {
                                        isPreviewMode.set(true);
                                    },
                                    disabled: *isSubmitting.read(),
                                    "Preview"
                                }
                            }

                            // Formatting toolbar (only shown in edit mode)
                            if !*isPreviewMode.read() {

                            // Formatting toolbar
                            div {
                                class: "mb-2 border border-gray-300 rounded-t-md bg-gray-50 p-2 flex flex-wrap gap-1",

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_bold,
                                    disabled: *isSubmitting.read(),
                                    title: "Bold",
                                    "B"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_italic,
                                    disabled: *isSubmitting.read(),
                                    title: "Italic",
                                    "I"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_heading,
                                    disabled: *isSubmitting.read(),
                                    title: "Heading",
                                    "H2"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_link,
                                    disabled: *isSubmitting.read(),
                                    title: "Link",
                                    "🔗"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_image,
                                    disabled: *isSubmitting.read(),
                                    title: "Image",
                                    "🖼️"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_code,
                                    disabled: *isSubmitting.read(),
                                    title: "Inline Code",
                                    "&lt;/&gt;"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_code_block,
                                    disabled: *isSubmitting.read(),
                                    title: "Code Block",
                                    "Code"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_unordered_list,
                                    disabled: *isSubmitting.read(),
                                    title: "Bullet List",
                                    "•"
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_ordered_list,
                                    disabled: *isSubmitting.read(),
                                    title: "Numbered List",
                                    "1."
                                }

                                button {
                                    r#type: "button",
                                    class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                                    onclick: handle_format_blockquote,
                                    disabled: *isSubmitting.read(),
                                    title: "Blockquote",
                                    "Quote"
                                }
                            }

                            textarea {
                                value: "{body}",
                                class: "mt-0 block w-full border border-gray-300 border-t-0 rounded-b-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono",
                                rows: 8,
                                oninput: move |e: Event<FormData>| {
                                    *body.write() = e.value();
                                },
                                disabled: *isSubmitting.read()
                            }
                        } else {
                            // Preview mode - show rendered markdown
                            div {
                                class: "mt-0 block w-full border border-gray-300 rounded-md shadow-sm py-3 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm min-h-[200px] bg-gray-50",
                                if body.read().trim().is_empty() {
                                    p {
                                        class: "text-gray-400 italic",
                                        "No content to preview"
                                    }
                                } else {
                                    div {
                                        class: "prose prose-sm max-w-none",
                                        dangerous_inner_html: render_markdown_to_html(&body.read()),
                                    }
                                }
                            }
                        }
                        }

                        // Tags field
                        div {
                            div {
                                // class: "block text-sm font-medium text-gray-700 mb-2",
                                "Tags"
                            }

                            div {
                                class: "flex flex-wrap gap-2 mb-3",

                                if *tags_loading.read() {
                                    span {
                                        class: "text-sm text-gray-500",
                                        "Loading tags..."
                                    }
                                } else if tag_badges.read().is_empty() {
                                    span {
                                        class: "text-sm text-gray-500",
                                        "No tags selected"
                                    }
                                } else {
                                    for (tag_id, tag) in tag_badges.read().iter().cloned() {
                                        div {
                                            class: "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-indigo-100 text-indigo-800",

                                            span {
                                                "{tag.name}"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ml-2 inline-flex items-center justify-center w-4 h-4 rounded-full text-indigo-400 hover:text-indigo-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                                                onclick: move |_| {
                                                    let mut ids = selected_tag_ids.write();
                                                    ids.retain(|id| *id != tag_id);
                                                },
                                                disabled: *isSubmitting.read(),

                                                svg {
                                                    class: "w-3 h-3",
                                                    fill: "none",
                                                    view_box: "0 0 24 24",
                                                    stroke: "currentColor",

                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        "stroke-width": 2,
                                                        d: "M6 18L18 6M6 6l12 12"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            button {
                                r#type: "button",
                                class: "inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                                onclick: move |_| {
                                    *show_tag_selector.write() = !show_tag_selector();
                                },
                                disabled: *isSubmitting.read() || available_tags.read().is_empty(),

                                svg {
                                    class: "-ml-0.5 mr-2 h-4 w-4 text-gray-500",
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

                                "Add Tag"
                            }

                            if *show_tag_selector.read() {
                                div {
                                    class: "mt-3 p-3 border border-gray-200 rounded-md bg-gray-50",

                                    div {
                                        class: "max-h-48 overflow-y-auto space-y-1",
                                        for tag in available_tags_to_show().iter().cloned() {
                                            button {
                                                r#type: "button",
                                                class: "w-full text-left px-3 py-2 rounded-md text-sm text-gray-700 hover:bg-white hover:shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                                onclick: move |_| {
                                                    let mut ids = selected_tag_ids.write();
                                                    ids.push(tag.id.unwrap());
                                                    *show_tag_selector.write() = false;
                                                },
                                                disabled: *isSubmitting.read(),
                                                "{tag.name}"
                                            }
                                        }
                                    }

                                    button {
                                        r#type: "button",
                                        class: "mt-2 text-sm text-gray-500 hover:text-gray-700",
                                        onclick: move |_| {
                                            *show_tag_selector.write() = false;
                                        },

                                        "Cancel"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Form actions
            div {
                class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",

                button {
                    r#type: "button",
                    class: "w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-indigo-600 text-base font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:ml-3 sm:w-auto sm:text-sm",
                    onclick: handle_submit,
                    disabled: *isSubmitting.read(),

                    if *isSubmitting.read() {
                        "Saving..."
                    } else {
                        "{button_text}"
                    }
                }

                button {
                    r#type: "button",
                    class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm",
                    onclick: move |_| {
                        props.on_cancel.call(());
                    },
                    disabled: *isSubmitting.read(),
                    "Cancel"
                }
            }
        }
    }
}
