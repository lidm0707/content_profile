use crate::components::editor::cursor::{BODY_TEXTAREA_ID, read_cursor_pos};
use content_sdk::utils::{MARKDOWN_CONTAINER_CLASS, render_markdown_to_html};
use dioxus::prelude::*;

/// Component for edit mode with toolbar and textarea
#[component]
pub fn EditModeBodyEditor(
    body: Signal<String>,
    is_submitting: Signal<bool>,
    is_uploading_image: Signal<bool>,
    handle_format_bold: EventHandler<MouseEvent>,
    handle_format_italic: EventHandler<MouseEvent>,
    handle_format_heading: EventHandler<MouseEvent>,
    handle_format_link: EventHandler<MouseEvent>,
    on_upload_image: EventHandler<()>,
    on_edit_image_size: EventHandler<()>,
    handle_format_code: EventHandler<MouseEvent>,
    handle_format_code_block: EventHandler<MouseEvent>,
    handle_format_unordered_list: EventHandler<MouseEvent>,
    handle_format_ordered_list: EventHandler<MouseEvent>,
    handle_format_blockquote: EventHandler<MouseEvent>,
    handle_format_table: EventHandler<MouseEvent>,
    cursor_pos: Signal<Option<(usize, usize)>>,
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
                onclick: move |_| on_upload_image.call(()),
                disabled: *is_submitting.read() || *is_uploading_image.read(),
                title: if *is_uploading_image.read() { "Uploading..." } else { "Upload image to Google Drive" },
                if *is_uploading_image.read() {
                    { "⏳" }
                } else {
                    { "🖼️" }
                }
            }
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: move |_| on_edit_image_size.call(()),
                disabled: *is_submitting.read(),
                title: "Edit image size (place cursor on an image first)",
                "Img↔"
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
            button {
                r#type: "button",
                class: "px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50",
                onclick: handle_format_table,
                disabled: *is_submitting.read(),
                title: "Table",
                "Table"
            }
        }
        textarea {
            id: BODY_TEXTAREA_ID,
            value: "{body}",
            class: "mt-0 block w-full border border-gray-300 border-t-0 rounded-b-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono",
            rows: 8,
            oninput: move |e: Event<FormData>| {
                *body.write() = e.value();
                *cursor_pos.write() = read_cursor_pos();
            },
            // Capture the selection whenever the user clicks / arrows inside
            // the textarea and whenever focus leaves it (so the next toolbar
            // click still knows where the caret was).
            onclick: move |_| {
                *cursor_pos.write() = read_cursor_pos();
            },
            onkeyup: move |_| {
                *cursor_pos.write() = read_cursor_pos();
            },
            onblur: move |_| {
                *cursor_pos.write() = read_cursor_pos();
            },
            disabled: *is_submitting.read()
        }
    }
}

/// Component for preview mode showing rendered markdown
#[component]
pub fn PreviewModeBodyEditor(body: Signal<String>) -> Element {
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
                        class: "{MARKDOWN_CONTAINER_CLASS}",
                        dangerous_inner_html: render_markdown_to_html(&body.read()),
                    }
            }
        }
    }
}
