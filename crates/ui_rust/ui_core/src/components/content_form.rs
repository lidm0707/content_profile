#[cfg(target_arch = "wasm32")]
use crate::app::KEY_GOOGLE_FOLDER_ID;
#[cfg(target_arch = "wasm32")]
use crate::components::editor::cursor::read_cursor_pos;
use crate::components::editor::cursor::{
    CODE_BLOCK_PLACEHOLDER, FENCE_DELIM, ImageTarget, LINK_PLACEHOLDER, find_first_image,
    find_image_under_cursor, insert_at_cursor, restore_cursor_after_render,
    restore_selection_after_render,
};
use crate::components::editor::image_size_modal::ImageSizeModal;
use crate::components::editor::tags_ui::{
    ClearAllTagsConfirmationModal, RemoveTagConfirmationModal, TagsField,
};
use crate::components::editor::toolbar::{EditModeBodyEditor, PreviewModeBodyEditor};
use content_sdk::ContentTagsContext;
use content_sdk::TagContext;
#[cfg(target_arch = "wasm32")]
use content_sdk::contexts::UserContext;
use content_sdk::models::{Content, ContentRequest, STATUS_DRAFT, STATUS_PUBLISHED, Tag};
#[cfg(target_arch = "wasm32")]
use content_sdk::services::BackendApiService;
#[cfg(target_arch = "wasm32")]
use content_sdk::services::drive::upload_with_token as drive_upload_with_token;
use content_sdk::utils::config::Config;
#[cfg(target_arch = "wasm32")]
use content_sdk::utils::format_image;
use content_sdk::utils::markdown::update_tags_in_frontmatter;
use content_sdk::utils::{
    format_blockquote, format_bold, format_code, format_code_block, format_heading, format_italic,
    format_link, format_ordered_list, format_table, format_unordered_list,
};
use dioxus::prelude::*;
use tracing::{debug, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::ui::{Button, ButtonVariant, TextField};

#[cfg(target_arch = "wasm32")]
fn guess_mime_from_name(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else {
        "application/octet-stream"
    }
}

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

/// Content form component for creating and editing content
#[component]
pub fn ContentForm(props: ContentFormProps) -> Element {
    let tag_context = use_context::<TagContext>();
    let content_tags_context = use_context::<ContentTagsContext>();
    let content_tags_context_for_resource = content_tags_context.clone();

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
    let mut is_submitting = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut isPreviewMode = use_signal(|| false);
    let mut is_uploading_image = use_signal(|| false);
    // Last known selection on the body textarea: `(selection_start, selection_end)`
    // in UTF-16 code-unit offsets (matches the DOM API).
    // `None` means "no recorded position" — we fall back to appending at the end.
    let cursor_pos: Signal<Option<(usize, usize)>> = use_signal(|| None);
    let tag_to_remove = use_signal(|| None::<(i32, String)>);
    let show_clear_all_confirmation = use_signal(|| false);
    // Image-size editor state.
    let mut image_size_open = use_signal(|| false);
    let mut image_size_width = use_signal(String::new);
    let mut image_size_height = use_signal(String::new);
    let mut image_size_target = use_signal(|| None::<ImageTarget>);

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
        let context = content_tags_context_for_resource.clone();
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

    use_effect(move || {
        if let Some(content) = props.content.read().as_ref() {
            title.set(content.title.clone());
            slug.set(content.slug.clone());
            body.set(content.body.clone());
            debug!(
                "Content effect updated - title: {}, status: {}",
                content.title, content.status
            );
        }
    });

    // Auto-generate slug from title
    let mut handle_title_change_value = move |new_title: String| {
        *title.write() = new_title.clone();
        if slug.read().is_empty() {
            slug.write().clone_from(&Content::generate_slug(&new_title));
        }
    };

    // Inserts `markdown` into `body` at the stored cursor position (or appends
    // when we never recorded one) and schedules a DOM caret restore.
    let mut insert_markdown = move |markdown: String| {
        let current_body = body.read().clone();
        let pos = *cursor_pos.read();
        let (new_body, caret) = insert_at_cursor(&current_body, &markdown, pos);
        *body.write() = new_body;
        restore_cursor_after_render(caret);
    };

    // Inserts a fenced code block at the cursor and lands the caret *inside* it
    // (selecting the placeholder so a single keystroke replaces it). The fence
    // is padded onto its own line(s) so markdown always parses it as a block,
    // never as inline text glued to the surrounding line.
    let mut insert_code_block = move || {
        let current_body = body.read().clone();
        let pos = *cursor_pos.read();

        // Splice point: stored cursor, else append at the end.
        let (splice_start, splice_end) = match pos {
            Some((s, e)) if s <= current_body.len() && e <= current_body.len() => (s, e),
            _ => (current_body.len(), current_body.len()),
        };

        // Pad so the opening/closing fence each sit on their own line.
        let need_leading_newline =
            splice_start > 0 && !current_body[..splice_start].ends_with('\n');
        let need_trailing_newline =
            splice_end < current_body.len() && !current_body[splice_end..].starts_with('\n');

        // format_code_block produces "```\n{placeholder}\n```". The placeholder
        // starts right after "```\n".
        let fence = format_code_block(CODE_BLOCK_PLACEHOLDER);
        let open_len = FENCE_DELIM.len() + 1; // "```" + '\n'
        let ph_start = open_len;
        let ph_end = open_len + CODE_BLOCK_PLACEHOLDER.len();

        let mut new_body = String::with_capacity(current_body.len() + fence.len() + 2);
        new_body.push_str(&current_body[..splice_start]);
        if need_leading_newline {
            new_body.push('\n');
        }
        let insert_start = new_body.len();
        new_body.push_str(&fence);
        new_body.push_str(&current_body[splice_end..]);
        if need_trailing_newline {
            new_body.insert(insert_start + fence.len(), '\n');
        }

        *body.write() = new_body;
        restore_selection_after_render(insert_start + ph_start, insert_start + ph_end);
    };

    // Formatting handlers — insert at cursor, fall back to append.
    let handle_format_bold = move |_| {
        insert_markdown(format_bold("bold text"));
    };

    let handle_format_italic = move |_| {
        insert_markdown(format_italic("italic text"));
    };

    let handle_format_code = move |_| {
        insert_markdown(format_code("code"));
    };

    let handle_format_code_block = move |_| {
        insert_code_block();
    };

    let handle_format_heading = move |_| {
        insert_markdown(format_heading("Heading", 2));
    };

    let handle_format_link = move |_| {
        let md = format_link(LINK_PLACEHOLDER, "https://");
        let current_body = body.read().clone();
        let pos = *cursor_pos.read();
        let (new_body, caret) = insert_at_cursor(&current_body, &md, pos);
        *body.write() = new_body;
        // Select the placeholder label so typing replaces it; the start sits
        // just past the opening `[`.
        let label_start = caret - md.len() + 1;
        restore_selection_after_render(label_start, label_start + LINK_PLACEHOLDER.len());
    };

    let handle_format_unordered_list = move |_| {
        insert_markdown(format_unordered_list("List item"));
    };

    let handle_format_ordered_list = move |_| {
        insert_markdown(format_ordered_list("List item", 1));
    };

    let handle_format_blockquote = move |_| {
        insert_markdown(format_blockquote("Quote text"));
    };

    // Image-size editor.
    //
    // Open: locate the image under the caret (falling back to the first
    // image in the body) and pre-fill the modal with its current size. If
    // the body has no images at all, surface an error instead.
    let handle_open_image_size = move |_| {
        let current_body = body.read().clone();
        let caret = (*cursor_pos.read())
            .map(|(s, _)| s)
            .unwrap_or(current_body.len());
        let found = find_image_under_cursor(&current_body, caret)
            .or_else(|| find_first_image(&current_body));
        match found {
            Some(target) => {
                image_size_width.set(target.width.map(|w| w.to_string()).unwrap_or_default());
                image_size_height.set(target.height.map(|h| h.to_string()).unwrap_or_default());
                image_size_target.set(Some(target));
                image_size_open.set(true);
            }
            None => {
                error_message.set(Some(
                    "Place the cursor inside an image (e.g. ![..](..)) first".to_string(),
                ));
            }
        }
    };

    // Parse a numeric text field into Option<u32>. Empty → None; invalid → None.
    let parse_size_field = move |s: &str| -> Option<u32> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            trimmed.parse::<u32>().ok()
        }
    };

    // Apply the entered size to the targeted image: rewrite the `![alt](url)`
    // span with a fresh `#img=...` fragment and restore the caret just past it.
    let handle_apply_image_size = move |_| {
        let Some(target) = image_size_target.read().clone() else {
            image_size_open.set(false);
            return;
        };
        let width = parse_size_field(&image_size_width.read());
        let height = parse_size_field(&image_size_height.read());

        let current_body = body.read().clone();
        let new_md = target.to_markdown(width, height);
        // Bounds may have shifted if the body changed while the modal was
        // open; clamp to be safe.
        let start = target.start.min(current_body.len());
        let end = target.end.min(current_body.len()).max(start);
        let mut new_body = String::with_capacity(current_body.len() + new_md.len());
        new_body.push_str(&current_body[..start]);
        let caret = start + new_md.len();
        new_body.push_str(&new_md);
        new_body.push_str(&current_body[end..]);
        *body.write() = new_body;

        image_size_open.set(false);
        restore_cursor_after_render(caret);
    };

    // Strip the size fragment so the image renders responsively again.
    let handle_remove_image_size = move |_| {
        let Some(target) = image_size_target.read().clone() else {
            image_size_open.set(false);
            return;
        };
        let current_body = body.read().clone();
        let new_md = target.to_markdown(None, None);
        let start = target.start.min(current_body.len());
        let end = target.end.min(current_body.len()).max(start);
        let mut new_body = String::with_capacity(current_body.len() + new_md.len());
        new_body.push_str(&current_body[..start]);
        let caret = start + new_md.len();
        new_body.push_str(&new_md);
        new_body.push_str(&current_body[end..]);
        *body.write() = new_body;

        image_size_open.set(false);
        image_size_width.set(String::new());
        image_size_height.set(String::new());
        restore_cursor_after_render(caret);
    };

    let handle_cancel_image_size = move |_| {
        image_size_open.set(false);
    };

    let config = use_context::<Memo<Config>>();

    // Holds the Drive access token served by the backend (stored per user in
    // pg). The click handler fetches it, then a second click opens the file
    // picker inside a real user gesture.
    let mut gdrive_token: Signal<Option<String>> = use_signal(|| None);
    // Cursor position captured at image-button click time, so the async
    // upload path can insert at the original caret instead of the end.
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut pending_image_pos: Signal<Option<(usize, usize)>> = use_signal(|| None);

    let handle_trigger_image_upload = move |_| {
        // Re-entrancy guard. `is_uploading_image` only disables the button
        // after a re-render, so a fast second click would call
        // `requestAccessToken` twice (GIS blocks the extra popup) — and the
        // synchronous wasm re-entry inside GIS then collides with Dioxus
        // signal borrows → "RefCell already borrowed" panic.
        if is_uploading_image() {
            return;
        }

        // Token already fetched in a previous click? Open the file picker
        // directly. This runs inside a real user gesture, so the picker is
        // allowed to open (a programmatic click after an await is NOT:
        // transient user activation has expired by then).
        #[cfg(target_arch = "wasm32")]
        if gdrive_token.read().is_some() {
            // Click the hidden input synchronously via web_sys. `document::eval`
            // may not run inside the same activation window in Firefox, and a
            // file picker opened without transient activation is silently
            // blocked ("Opening multiple popups was blocked").
            if let Some(input) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("gdrive-image-input"))
            {
                if let Ok(el) = input.dyn_into::<web_sys::HtmlInputElement>() {
                    el.click();
                }
            }
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Fetch the Drive access token from the backend (the refresh
            // token lives in pg, keyed by the logged-in user). The consent
            // popup is only ever needed once — via Settings → Connect Google
            // Drive — never inside this upload flow.
            pending_image_pos.set(read_cursor_pos());
            is_uploading_image.set(true);
            error_message.set(None);

            let api = BackendApiService::new(Some(config.read().clone()));
            let mut gdrive_token_signal = gdrive_token;
            let mut error_message_signal = error_message;
            let mut is_uploading_signal = is_uploading_image;

            spawn(async move {
                match api.get_google_access_token().await {
                    Ok(token) => {
                        gdrive_token_signal.set(Some(token));
                        // Transient user activation expired during the fetch,
                        // so a programmatic file-picker click here would be
                        // blocked by the browser. Keep the token and ask the
                        // user to click the button once more — that click
                        // opens the picker within a real gesture (see the
                        // token branch above).
                        is_uploading_signal.set(false);
                        error_message_signal.set(Some(
                            "Google Drive ready — click the image button again to choose a file"
                                .to_string(),
                        ));
                    }
                    Err(msg) => {
                        error!("google access token fetch failed: {msg}");
                        let hint = if msg.contains("not connected") {
                            "Google Drive not connected — connect it in Settings → Google"
                        } else {
                            "Image upload failed"
                        };
                        error_message_signal.set(Some(format!("{hint}: {msg}")));
                        is_uploading_signal.set(false);
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (config, gdrive_token);
            is_uploading_image.set(false);
        }
    };

    let handle_image_file_selected = move |e: Event<FormData>| {
        let token = match gdrive_token.read().clone() {
            Some(t) if !t.is_empty() => t,
            _ => {
                error_message.set(Some(
                    "No OAuth token — click the image button first".to_string(),
                ));
                is_uploading_image.set(false);
                return;
            }
        };

        let files = e.files();
        if files.is_empty() {
            warn!("image file selected but no files attached");
            is_uploading_image.set(false);
            // Clear stale token so the next click re-acquires.
            gdrive_token.set(None);
            return;
        }
        let file_engine = files[0].clone();

        let file_name = file_engine.name();
        debug!("image file selected: {file_name}");

        #[cfg(target_arch = "wasm32")]
        let mut body_signal = body;
        #[cfg(target_arch = "wasm32")]
        let mut error_message_signal = error_message;
        #[cfg(target_arch = "wasm32")]
        let mut is_uploading_signal = is_uploading_image;
        #[cfg(target_arch = "wasm32")]
        let mut gdrive_token_signal = gdrive_token;
        #[cfg(target_arch = "wasm32")]
        let mut pending_pos_signal = pending_image_pos;
        #[cfg(target_arch = "wasm32")]
        let user_context_for_upload = use_context::<UserContext>();

        #[cfg(target_arch = "wasm32")]
        spawn(async move {
            let bytes = match file_engine.read_bytes().await {
                Ok(b) => b,
                Err(e) => {
                    let msg = format!("Failed to read file: {e}");
                    error!("{msg}");
                    error_message_signal.set(Some(msg));
                    is_uploading_signal.set(false);
                    gdrive_token_signal.set(None);
                    return;
                }
            };

            let mime = guess_mime_from_name(&file_name);
            let alt_text = file_name
                .rsplit('.')
                .next()
                .map(|ext| {
                    let base = file_name.trim_end_matches(ext).trim_end_matches('.');
                    if base.is_empty() { "image" } else { base }
                })
                .unwrap_or("image");

            // Folder id comes only from the per-user pg setting
            // (Settings → Google). Empty means upload to Drive root.
            let folder_id = match user_context_for_upload.load_settings().await {
                Ok(settings) => settings
                    .get(KEY_GOOGLE_FOLDER_ID)
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
                Err(e) => {
                    warn!("settings load failed, uploading to Drive root: {e}");
                    None
                }
            };

            match drive_upload_with_token(
                &token,
                bytes.as_ref(),
                mime,
                &file_name,
                folder_id.as_deref(),
            )
            .await
            {
                Ok(url) => {
                    let current_body = body_signal.read().clone();
                    let markdown = format_image(alt_text, &url);
                    let pos = *pending_pos_signal.read();
                    let (new_body, caret) = insert_at_cursor(&current_body, &markdown, pos);
                    *body_signal.write() = new_body;
                    restore_cursor_after_render(caret);
                    debug!("image inserted: ![{alt_text}]({url})");
                }
                Err(msg) => {
                    error!("drive upload failed: {msg}");
                    error_message_signal.set(Some(format!("Image upload failed: {msg}")));
                    gdrive_token_signal.set(None);
                }
            }
            // Keep the token on success so the next upload skips the OAuth
            // popup; clear it on failure (e.g. expired/revoked token) so the
            // next click re-authenticates.
            pending_pos_signal.set(None);
            is_uploading_signal.set(false);
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (
                token,
                file_engine,
                body,
                error_message,
                is_uploading_image,
                gdrive_token,
                pending_image_pos,
            );
        }
    };

    let handle_format_table = move |_| {
        let markdown = format_table(
            &["Header 1", "Header 2", "Header 3"],
            &[
                &["Cell 1", "Cell 2", "Cell 3"],
                &["Cell 4", "Cell 5", "Cell 6"],
            ],
        );
        insert_markdown(markdown);
    };

    let handle_submit = move |status: &'static str| {
        if title.read().is_empty() {
            error_message.set(Some("Title is required".to_string()));
            return;
        }

        if body.read().is_empty() {
            error_message.set(Some("Body is required".to_string()));
            return;
        }

        is_submitting.set(true);

        let selected_tags: Vec<Tag> = available_tags
            .read()
            .iter()
            .filter(|t| selected_tag_ids.read().contains(&t.id.unwrap()))
            .cloned()
            .collect();

        let updated_body = update_tags_in_frontmatter(&body.read(), &selected_tags);

        debug!(
            "Form submission - title: {}, slug: {}, status: {}, tags: {:?}",
            title.read(),
            slug.read(),
            status,
            selected_tag_ids.read()
        );

        let request = ContentRequest {
            id: props.content.read().as_ref().and_then(|c| c.id),
            title: title.read().clone(),
            slug: slug.read().clone(),
            body: updated_body,
            status: status.to_string(),
        };

        let selected_tag_ids_clone = selected_tag_ids.read().clone();
        debug!(
            "ContentRequest created with status: {}, tag_ids: {:?}",
            request.status, selected_tag_ids_clone
        );
        props.on_submit.call((request, selected_tag_ids_clone));
        is_submitting.set(false);
    };

    let handle_save_draft = {
        let mut handle_submit = handle_submit;
        move |_| handle_submit(STATUS_DRAFT)
    };
    let handle_publish = {
        let mut handle_submit = handle_submit;
        move |_| handle_submit(STATUS_PUBLISHED)
    };

    let show_tag_selector = use_signal(|| false);

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
        // hidden file input for Google Drive image upload
        input {
            id: "gdrive-image-input",
            r#type: "file",
            accept: "image/*",
            class: "hidden",
            onchange: handle_image_file_selected,
        }

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
                        class: "grid grid-cols-1 lg:grid-cols-3 gap-6",

                        // LEFT — the content editor takes the main column and
                        // stretches to (nearly) the full screen height.
                        div {
                            class: "lg:col-span-2 flex flex-col",

                            div {
                                class: "mb-2 flex items-center justify-between",
                                label {
                                    class: "block text-sm font-medium text-gray-700",
                                    "Content"
                                }

                                // Preview/Edit toggle
                                div {
                                    class: "flex items-center space-x-2",

                                    if *isPreviewMode.read() {
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            disabled: *is_submitting.read(),
                                            onclick: move |_| {
                                                isPreviewMode.set(false);
                                            },
                                            "Edit"
                                        }
                                    } else {
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            disabled: *is_submitting.read(),
                                            onclick: move |_| {
                                                isPreviewMode.set(false);
                                            },
                                            "Edit"
                                        }
                                    }

                                    if *isPreviewMode.read() {
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            disabled: *is_submitting.read(),
                                            onclick: move |_| {
                                                isPreviewMode.set(true);
                                            },
                                            "Preview"
                                        }
                                    } else {
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            disabled: *is_submitting.read(),
                                            onclick: move |_| {
                                                isPreviewMode.set(true);
                                            },
                                            "Preview"
                                        }
                                    }
                                }
                            }

                            // Formatting toolbar (only shown in edit mode)
                            if !*isPreviewMode.read() {
                                EditModeBodyEditor {
                                    body,
                                    is_submitting,
                                    is_uploading_image,
                                    handle_format_bold: handle_format_bold,
                                    handle_format_italic: handle_format_italic,
                                    handle_format_heading: handle_format_heading,
                                    handle_format_link: handle_format_link,
                                    on_upload_image: handle_trigger_image_upload,
                                    on_edit_image_size: handle_open_image_size,
                                    handle_format_code: handle_format_code,
                                    handle_format_code_block: handle_format_code_block,
                                    handle_format_unordered_list: handle_format_unordered_list,
                                    handle_format_ordered_list: handle_format_ordered_list,
                                    handle_format_blockquote: handle_format_blockquote,
                                    handle_format_table: handle_format_table,
                                    cursor_pos,
                                }
                            } else {
                                PreviewModeBodyEditor {
                                    body,
                                }
                            }
                        }

                        // RIGHT — metadata sidebar.
                        div {
                            class: "space-y-6",

                            // Title field
                            TextField {
                                label: "Title".to_string(),
                                value: title.read().clone(),
                                oninput: move |v: String| handle_title_change_value(v),
                                disabled: *is_submitting.read(),
                            }

                            // Slug field
                            TextField {
                                label: "Slug".to_string(),
                                value: slug.read().clone(),
                                hint: "URL-friendly version of the title (auto-generated from title if empty)".to_string(),
                                oninput: move |v: String| {
                                    *slug.write() = v;
                                },
                                disabled: *is_submitting.read(),
                            }

                            // Status field removed — the action buttons below
                            // decide it: Save Draft stays local, Publish pushes.

                            TagsField {
                                selected_tag_ids,
                                is_submitting: is_submitting,
                                tag_to_remove,
                                show_clear_all_confirmation,
                                tags_loading,
                                tag_badges,
                                available_tags,
                                show_tag_selector,
                                available_tags_to_show,
                            }

                            // Actions — pinned under the sidebar fields.
                            div {
                                class: "flex flex-col gap-3 pt-2",

                                Button {
                                    variant: ButtonVariant::Primary,
                                    disabled: *is_submitting.read(),
                                    onclick: handle_save_draft,

                                    if *is_submitting.read() {
                                        "Saving..."
                                    } else {
                                        "Save Draft"
                                    }
                                }

                                Button {
                                    variant: ButtonVariant::Secondary,
                                    disabled: *is_submitting.read(),
                                    onclick: handle_publish,
                                    "Publish"
                                }

                                Button {
                                    variant: ButtonVariant::Ghost,
                                    disabled: *is_submitting.read(),
                                    onclick: move |_| {
                                        props.on_cancel.call(());
                                    },
                                    "Cancel"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove tag confirmation modal
        { debug!("ContentForm render - tag_to_remove value: {:?}", tag_to_remove.read()); }
        if let Some((tag_id, tag_name)) = tag_to_remove.read().as_ref().cloned() {
            { debug!("Rendering RemoveTagConfirmationModal with tag_id: {}, tag_name: {}", tag_id, tag_name); }
            RemoveTagConfirmationModal {
                tag_id,
                tag_name,
                tag_to_remove,
                selected_tag_ids,
                is_submitting: is_submitting,
                content_id: props.content.read().as_ref().and_then(|c| c.id),
                content_tags_context: content_tags_context.clone(),
            }
        }
     // Clear all tags confirmation modal
        if *show_clear_all_confirmation.read() {
            ClearAllTagsConfirmationModal {
                show_clear_all_confirmation,
                selected_tag_ids,
                tag_badges,
                is_submitting: is_submitting,
            }
        }

        // Image size editor — shown when the user clicked the Img↔ toolbar
        // button while the caret was on an image.
        if *image_size_open.read()
            && let Some(target) = image_size_target.read().clone()
        {
            ImageSizeModal {
                width: image_size_width,
                height: image_size_height,
                alt: target.alt.clone(),
                url: target.url.clone(),
                on_apply: handle_apply_image_size,
                on_remove: handle_remove_image_size,
                on_cancel: handle_cancel_image_size,
            }
        }
    }
}
