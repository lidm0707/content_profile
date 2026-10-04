use content_sdk::ContentTagsContext;
use content_sdk::models::Tag;
use dioxus::prelude::*;
use tracing::{debug, error};

/// Tags field component for managing content tags
/// Individual tag badge component
#[component]
pub fn TagBadge(tag_id: i32, tag: Tag, tag_to_remove: Signal<Option<(i32, String)>>) -> Element {
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
                class: "ml-1 flex items-center justify-center w-5 h-5 rounded-full text-indigo-400 hover:text-red-600 hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-all duration-150 cursor-pointer",
                onclick: move |_| {
                    debug!("Tag remove clicked: id={}, name={}", tag_id, tag.name);
                    tag_to_remove.set(Some((tag_id, tag.name.clone())));
                },
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
pub fn TagsField(
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
pub fn RemoveTagConfirmationModal(
    tag_id: i32,
    tag_name: String,
    tag_to_remove: Signal<Option<(i32, String)>>,
    selected_tag_ids: Signal<Vec<i32>>,
    is_submitting: ReadSignal<bool>,
    content_id: Option<i32>,
    content_tags_context: ContentTagsContext,
) -> Element {
    debug!(
        "RemoveTagConfirmationModal rendered - tag_id: {}, tag_name: {}, content_id: {:?}",
        tag_id, tag_name, content_id
    );
    let mut is_removing = use_signal(|| false);
    rsx! {
    div {
        class: "fixed inset-0 z-50 flex items-center justify-center overflow-y-auto",
        div {
            class: "fixed inset-0 bg-gray-500/30 transition-opacity z-40",
            onclick: move |_| {
                tag_to_remove.set(None);
            }
        }

        div {
            class: "relative bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all max-w-lg w-full mx-4 z-50",

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
                            is_removing.set(true);
                            let mut context_for_spawn = content_tags_context.clone();
                            let tag_id_for_spawn = tag_id;
                            let mut tag_to_remove_for_spawn = tag_to_remove;
                            let mut is_removing_for_spawn = is_removing;

                            spawn(async move {
                                if let Some(content_id) = content_id
                                    && let Err(err) = context_for_spawn.remove_tag_from_content(content_id, tag_id_for_spawn).await
                                {
                                    error!("Failed to remove tag from content: {}", err);
                                    is_removing_for_spawn.set(false);
                                    return;
                                }

                                tag_to_remove_for_spawn.set(None);
                                is_removing_for_spawn.set(false);
                            });

                            let mut ids = selected_tag_ids.write();
                            ids.retain(|id| *id != tag_id);
                        },
                        disabled: *is_removing.read(),

                        if *is_removing.read() {
                            "Removing..."
                        } else {
                            "Remove"
                        }
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

/// Confirmation modal for clearing all tags
#[component]
pub fn ClearAllTagsConfirmationModal(
    show_clear_all_confirmation: Signal<bool>,
    selected_tag_ids: Signal<Vec<i32>>,
    tag_badges: ReadSignal<Vec<(i32, Tag)>>,
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
                        show_clear_all_confirmation.set(false);
                    }
                }

                span {
                    class: "hidden sm:inline-block sm:align-middle sm:h-screen",
                    " "
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
                            disabled: *is_submitting.read(),

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
