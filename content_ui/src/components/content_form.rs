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

/// Props for content form component
#[derive(Clone, PartialEq, Props)]
pub struct ContentFormProps {
    /// Optional content for editing (None for creating new content)
    pub content: ReadSignal<Option<Content>>,
    /// Callback when form is submitted successfully (includes selected tag IDs)
    pub on_submit: EventHandler<(ContentRequest, Vec<i32>)>,
    /// Callback when form is cancelled
    pub on_cancel: EventHandler<()>,
}

/// Component for edit mode with toolbar and textarea
#[component]
fn EditModeBodyEditor(
    body: Signal<String>,
    is_submitting: Signal<bool>,
    handle_format_bold: EventHandler<MouseEvent>,
    handle_format_italic: EventHandler<MouseEvent>,
    handle_format_heading: EventHandler<MouseEvent>,
    handle_format_link: EventHandler<MouseEvent>,
    handle_format_image: EventHandler<MouseEvent>,
    handle_format_code: EventHandler<MouseEvent>,
    handle_format_code_block: EventHandler<MouseEvent>,
    handle_format_unordered_list: EventHandler<MouseEvent>,
    handle_format_ordered_list: EventHandler<MouseEvent>,
    handle_format_blockquote: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            class: "mb-2 border border-gray-300 rounded-t-md bg-gray-50 p-2 flex flex-wrap gap-1",
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_bold,
                disabled: *is_submitting.read(),
                title: "Bold",
                "B"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_italic,
                disabled: *is_submitting.read(),
                title: "Italic",
                "I"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_heading,
                disabled: *is_submitting.read(),
                title: "Heading",
                "H2"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_link,
                disabled: *is_submitting.read(),
                title: "Link",
                "🔗"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_image,
                disabled: *is_submitting.read(),
                title: "Image",
                "🖼️"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_code,
                disabled: *is_submitting.read(),
                title: "Inline Code",
                "</>"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_code_block,
                disabled: *is_submitting.read(),
                title: "Code Block",
                "Code"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_unordered_list,
                disabled: *is_submitting.read(),
                title: "Bullet List",
                "•"
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_ordered_list,
                disabled: *is_submitting.read(),
                title: "Numbered List",
                "1."
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_blockquote,
                disabled: *is_submitting.read(),
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
            disabled: *is_submitting.read()
        }
    }
}

/// Component for preview mode showing rendered markdown
#[component]
fn PreviewModeBodyEditor(body: Signal<String>) -> Element {
    rsx! {
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

/// Tags field component for managing content tags
/// Individual tag badge component
#[component]
fn TagBadge(tag_id: i32, tag: Tag, tag_to_remove: Signal<Option<(i32, String)>>) -> Element {
    let is_marked_for_removal = tag_to_remove().map(|(id, _)| id == tag_id).unwrap_or(false);
    debug!(
        "TagBadge render: id={}, name={}, is_marked_for_removal={}",
        tag_id, tag.name, is_marked_for_removal
    );

    rsx! {
        div {
            class: if is_marked_for_removal {
                "inline-flex items-center px-3 py-1.5 rounded-full text-sm font-medium bg-red-100 text-red-800 hover:bg-red-200 transition-colors duration-150 group"
            } else {
                "inline-flex items-center px-3 py-1.5 rounded-full text-sm font-medium bg-indigo-100 text-indigo-800 hover:bg-indigo-200 transition-colors duration-150 group"
            },

            span {
                class: "mr-1",
                "{tag.name}"
            }
            button {
                r#type: "button",
                class: "ml-1 inline-flex items-center justify-center w-4 h-4 rounded-full text-indigo-400 hover:text-red-600 hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-all duration-150",
                onclick: move |_| {
                    debug!("Tag remove clicked: id={}, name={}", tag_id, tag.name);
                    tag_to_remove.set(Some((tag_id, tag.name.clone())));
                },
                disabled: false,
                title: "Remove tag",

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

/// Tags field component for managing content tags
#[component]
fn TagsField(
    selected_tag_ids: Signal<Vec<i32>>,
    is_submitting: Signal<bool>,
    tag_to_remove: Signal<Option<(i32, String)>>,
    show_clear_all_confirmation: Signal<bool>,
    tags_loading: ReadSignal<bool>,
    tag_badges: ReadSignal<Vec<(i32, Tag)>>,
    available_tags: ReadSignal<Vec<Tag>>,
    show_tag_selector: Signal<bool>,
    available_tags_to_show: ReadSignal<Vec<Tag>>,
) -> Element {
    rsx! {
        div {
            div {
                "Tags"
            }

            div {
                class: "flex flex-wrap gap-2 mb-3 relative z-10",

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
                        TagBadge {
                            tag_id,
                            tag,
                            tag_to_remove,
                        }
                    }

                    if !tag_badges.read().is_empty() {
                        button {
                            r#type: "button",
                            class: "inline-flex items-center px-3 py-1.5 text-sm font-medium text-red-600 hover:text-red-700 hover:bg-red-50 rounded-full transition-colors duration-150",
                            onclick: move |_| {
                                show_clear_all_confirmation.set(true);
                            },
                            disabled: false,
                            //*is_submitting.read()
                            svg {
                                class: "w-4 h-4 mr-1",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",

                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    "stroke-width": 2,
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                }
                            }
                            "{show_clear_all_confirmation()}"
                            "Clear All"
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
                disabled: *is_submitting.read() || available_tags.read().is_empty(),

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
                                disabled: *is_submitting.read(),
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

/// Confirmation modal for removing a single tag
#[component]
fn RemoveTagConfirmationModal(
    tag_id: i32,
    tag_name: String,
    tag_to_remove: Signal<Option<(i32, String)>>,
    selected_tag_ids: Signal<Vec<i32>>,
    is_submitting: ReadSignal<bool>,
) -> Element {
    rsx! {
    div {
        class: "fixed inset-0 z-50 overflow-y-auto",
        div {
            class: "flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0",

            div {
                class: "fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity z-40",
                onclick: move |_| {
                    tag_to_remove.set(None);
                }
            }

            span {
                class: "hidden sm:inline-block sm:align-middle sm:h-screen",
                "​"
            }

            div {
                class: "inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full z-50",

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
                                    stroke_width: 2,
                                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                                }
                            }
                        }

                        div {
                            class: "mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left",

                            h3 {
                                class: "text-lg leading-6 font-medium text-gray-900",
                                "Remove Tag"
                            }

                            div {
                                class: "mt-2",

                                p {
                                    class: "text-sm text-gray-500",
                                    "Are you sure you want to remove the tag \"{tag_name}\"? This action can be undone by adding the tag back."
                                }
                            }
                        }
                    }
                }

                div {
                    class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",

                    button {
                        r#type: "button",
                        class: "w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm",
                        onclick: move |_| {
                            let mut ids = selected_tag_ids.write();
                            ids.retain(|id| *id != tag_id);
                            tag_to_remove.set(None);
                        },

                        "Remove"
                    }

                    button {
                        r#type: "button",
                        class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm",
                        onclick: move |_| {
                            tag_to_remove.set(None);
                        },

                        "Cancel"
                    }
                }
            }
        }
    }
    }
}

/// Confirmation modal for clearing all tags
#[component]
fn ClearAllTagsConfirmationModal(
    show_clear_all_confirmation: Signal<bool>,
    selected_tag_ids: Signal<Vec<i32>>,
    tag_badges: ReadSignal<Vec<(i32, Tag)>>,
    isSubmitting: ReadSignal<bool>,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 overflow-y-auto",
            div {
                class: "flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0",

                div {
                    class: "fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity z-40",
                    onclick: move |_| {
                        show_clear_all_confirmation.set(false);
                    }
                }

                span {
                    class: "hidden sm:inline-block sm:align-middle sm:h-screen",
                    "​"
                }

                div {
                    class: "inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full z-50",

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
                                        stroke_width: 2,
                                        d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                                    }
                                }
                            }

                            div {
                                class: "mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left",

                                h3 {
                                    class: "text-lg leading-6 font-medium text-gray-900",
                                    {"Clear All Tags"}
                                }

                                div {
                                    class: "mt-2",

                                    p {
                                        class: "text-sm text-gray-500",
                                        {"Are you sure you want to remove all ".to_string() + &tag_badges.read().len().to_string() + " tag(s)? This action can be undone by adding tags back."}
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",

                        button {
                            r#type: "button",
                            class: "w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm",
                            onclick: move |_| {
                                selected_tag_ids.set(Vec::new());
                                show_clear_all_confirmation.set(false);
                            },
                            disabled: *isSubmitting.read(),

                            "Clear All"
                        }

                        button {
                            r#type: "button",
                            class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm",
                            onclick: move |_| {
                                show_clear_all_confirmation.set(false);
                            },

                            "Cancel"
                        }
                    }
                }
            }
        }
    }
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
    let tag_to_remove = use_signal(|| None::<(i32, String)>);
    let show_clear_all_confirmation = use_signal(|| false);

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
        if let Some(content) = props.content.read().as_ref() {
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

        let selected_tag_ids_clone = selected_tag_ids.read().clone();
        debug!(
            "ContentRequest created with status: {}, tag_ids: {:?}",
            request.status, selected_tag_ids_clone
        );
        props.on_submit.call((request, selected_tag_ids_clone));
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
                                EditModeBodyEditor {
                                    body: body.clone(),
                                    is_submitting: isSubmitting.clone(),
                                    handle_format_bold: handle_format_bold,
                                    handle_format_italic: handle_format_italic,
                                    handle_format_heading: handle_format_heading,
                                    handle_format_link: handle_format_link,
                                    handle_format_image: handle_format_image,
                                    handle_format_code: handle_format_code,
                                    handle_format_code_block: handle_format_code_block,
                                    handle_format_unordered_list: handle_format_unordered_list,
                                    handle_format_ordered_list: handle_format_ordered_list,
                                    handle_format_blockquote: handle_format_blockquote,
                                }
                            } else {
                                PreviewModeBodyEditor {
                                    body: body.clone(),
                                }
                            }
                            }

                        TagsField {
                            selected_tag_ids,
                            is_submitting: isSubmitting,
                            tag_to_remove,
                            show_clear_all_confirmation,
                            tags_loading,
                            tag_badges,
                            available_tags,
                            show_tag_selector,
                            available_tags_to_show,
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

        // Remove tag confirmation modal
        if let Some((tag_id, tag_name)) = tag_to_remove.read().as_ref().cloned() {
            RemoveTagConfirmationModal {
                tag_id,
                tag_name,
                tag_to_remove,
                selected_tag_ids,
                is_submitting: isSubmitting,
            }
        }
     // Clear all tags confirmation modal
        if *show_clear_all_confirmation.read() {
            ClearAllTagsConfirmationModal {
                show_clear_all_confirmation,
                selected_tag_ids,
                tag_badges,
                isSubmitting: isSubmitting,
            }
        }
    }
}
