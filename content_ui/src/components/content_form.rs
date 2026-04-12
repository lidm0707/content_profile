use crate::models::{Content, ContentRequest, STATUS_DRAFT, STATUS_PUBLISHED, Tag};
use crate::utils::markdown::update_tags_in_frontmatter;
use crate::utils::{
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
    /// Available tags for selection
    pub available_tags: Signal<Vec<Tag>>,
    /// Callback when form is submitted successfully
    pub on_submit: EventHandler<ContentRequest>,
    /// Callback when form is cancelled
    pub on_cancel: EventHandler<()>,
}

/// Content form component for creating and editing content
#[component]
pub fn ContentForm(props: ContentFormProps) -> Element {
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
    let mut selected_tag_ids = use_signal(|| {
        props
            .content
            .read()
            .as_ref()
            .and_then(|c| c.tags.clone())
            .unwrap_or_default()
    });
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
            selected_tag_ids.set(content.tags.clone().unwrap_or_default());
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

        let selected_tags: Vec<Tag> = props
            .available_tags
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
            tags: Some(selected_tag_ids.read().clone()),
        };

        debug!("ContentRequest created with status: {}", request.status);
        props.on_submit.call(request);
        isSubmitting.set(false);
    };

    let parent_tags: Vec<Tag> = props
        .available_tags
        .read()
        .iter()
        .filter(|t| t.parent_id.is_none())
        .cloned()
        .collect();

    let child_tags_map: std::collections::HashMap<i32, Vec<Tag>> = props
        .available_tags
        .read()
        .iter()
        .filter(|t| t.parent_id.is_some())
        .fold(std::collections::HashMap::new(), |mut acc, tag| {
            if let Some(parent_id) = tag.parent_id {
                acc.entry(parent_id)
                    .or_insert_with(Vec::new)
                    .push(tag.clone());
            }
            acc
        });

    let orphan_tags: Vec<Tag> = props
        .available_tags
        .read()
        .iter()
        .filter(|t| t.parent_id.is_some() && !parent_tags.iter().any(|p| p.id == t.parent_id))
        .cloned()
        .collect();

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
                            label {
                                class: "block text-sm font-medium text-gray-700 mb-2",
                                "Tags"
                            }

                            if props.available_tags.read().is_empty() {
                                p {
                                    class: "text-sm text-gray-500",
                                    "No tags available. Create tags first."
                                }
                            } else {
                                div {
                                    class: "space-y-3",

                                    for parent_tag in parent_tags {
                                        div {
                                            class: "flex items-center",

                                            input {
                                                r#type: "checkbox",
                                                id: "tag-{parent_tag.id.unwrap()}",
                                                checked: selected_tag_ids.read().contains(&parent_tag.id.unwrap()),
                                                class: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded",
                                                onchange: move |e: Event<FormData>| {
                                                    let checked = e.checked();
                                                    let parent_id = parent_tag.id.unwrap();
                                                    let mut ids = selected_tag_ids.write();
                                                    if checked {
                                                        ids.push(parent_id);
                                                    } else {
                                                        ids.retain(|id| *id != parent_id);
                                                    }
                                                },
                                                disabled: *isSubmitting.read()
                                            }

                                            label {
                                                r#for: "tag-{parent_tag.id.unwrap()}",
                                                class: "ml-2 block text-sm text-gray-900 font-medium",
                                                "{parent_tag.name}"
                                            }
                                        }

                                        if let Some(child_tags) = child_tags_map.get(&parent_tag.id.unwrap()).cloned() {
                                            div {
                                                class: "ml-6 space-y-2 mt-2",

                                                for child_tag in child_tags {
                                                    div {
                                                        class: "flex items-center",

                                                        input {
                                                            r#type: "checkbox",
                                                            id: "tag-{child_tag.id.unwrap()}",
                                                            checked: selected_tag_ids.read().contains(&child_tag.id.unwrap()),
                                                            class: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded",
                                                            onchange: move |e: Event<FormData>| {
                                                                let checked = e.checked();
                                                                let child_id = child_tag.id.unwrap();
                                                                let mut ids = selected_tag_ids.write();
                                                                if checked {
                                                                    ids.push(child_id);
                                                                } else {
                                                                    ids.retain(|id| *id != child_id);
                                                                }
                                                            },
                                                            disabled: *isSubmitting.read()
                                                        }

                                                        label {
                                                            r#for: "tag-{child_tag.id.unwrap()}",
                                                            class: "ml-2 block text-sm text-gray-700",
                                                            "{child_tag.name}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if !orphan_tags.is_empty() {
                                        div {
                                            class: "mt-4 border-t pt-4",

                                            for orphan_tag in orphan_tags {
                                                div {
                                                    class: "flex items-center",

                                                    input {
                                                        r#type: "checkbox",
                                                        id: "tag-{orphan_tag.id.unwrap()}",
                                                        checked: selected_tag_ids.read().contains(&orphan_tag.id.unwrap()),
                                                        class: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded",
                                                        onchange: move |e: Event<FormData>| {
                                                            let checked = e.checked();
                                                            let orphan_id = orphan_tag.id.unwrap();
                                                            let mut ids = selected_tag_ids.write();
                                                            if checked {
                                                                ids.push(orphan_id);
                                                            } else {
                                                                ids.retain(|id| *id != orphan_id);
                                                            }
                                                        },
                                                        disabled: *isSubmitting.read()
                                                    }

                                                    label {
                                                        r#for: "tag-{orphan_tag.id.unwrap()}",
                                                        class: "ml-2 block text-sm text-gray-700",
                                                        "{orphan_tag.name}"
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
