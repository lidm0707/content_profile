use content_sdk::utils::markdown::{build_image_size_fragment, split_image_size};
use dioxus::prelude::*;

pub const BODY_TEXTAREA_ID: &str = "content-body-textarea";
pub const FENCE_DELIM: &str = "```";
pub const CODE_BLOCK_PLACEHOLDER: &str = "code";

/// Placeholder label inserted by the link button; selected after insertion so
/// typing replaces it.
pub const LINK_PLACEHOLDER: &str = "Link text";

/// Insert markdown at a byte-aware cursor position, returning the new body and
/// the byte offset where the inserted snippet ends (so we can restore the
/// caret there).
///
/// Falls back to appending (current behaviour) when no cursor position is
/// available, so we never lose data.
pub fn insert_at_cursor(
    current_body: &str,
    markdown: &str,
    cursor: Option<(usize, usize)>,
) -> (String, usize) {
    let Some((start, end)) =
        cursor.filter(|(s, e)| *s <= current_body.len() && *e <= current_body.len())
    else {
        let new_body = append_markdown(current_body, markdown);
        let new_caret = new_body.len();
        return (new_body, new_caret);
    };

    let mut new_body = String::with_capacity(current_body.len() + markdown.len() + 2);
    new_body.push_str(&current_body[..start]);
    new_body.push_str(markdown);
    new_body.push_str(&current_body[end..]);
    let new_caret = start + markdown.len();
    (new_body, new_caret)
}

fn append_markdown(current_body: &str, markdown: &str) -> String {
    if current_body.trim().is_empty() {
        markdown.to_string()
    } else {
        format!("{}\n\n{}", current_body, markdown)
    }
}

/// A markdown image `![alt](url)` located in the body, with any existing
/// `#img=...` size fragment split out of the URL. Used by the image-size
/// editor to know what to rewrite.
#[derive(Clone, Debug)]
pub struct ImageTarget {
    /// Inclusive byte offset of the leading `!`.
    pub start: usize,
    /// Byte offset just past the closing `)`.
    pub end: usize,
    pub alt: String,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl ImageTarget {
    /// Render this target back to markdown with the given size applied.
    /// `(None, None)` produces a plain `![alt](url)` (responsive).
    pub fn to_markdown(&self, width: Option<u32>, height: Option<u32>) -> String {
        let frag = build_image_size_fragment(width, height);
        format!("![{alt}]({url}{frag})", alt = self.alt, url = self.url)
    }
}

/// Parse the `![alt](url)` image starting at byte offset `bang` (which must
/// point at `![`). Returns `None` when the syntax is incomplete.
fn parse_image_at(body: &str, bang: usize) -> Option<ImageTarget> {
    let alt_start = bang + 2;
    // Need `](` after the alt.
    let close = body[alt_start..].find("](").map(|i| i + alt_start)?;
    let url_start = close + 2;
    // URL ends at the next `)`.
    let url_end = body[url_start..].find(')').map(|i| i + url_start)?;
    let full_end = url_end + 1;
    let alt = body[alt_start..close].to_string();
    let raw_url = &body[url_start..url_end];
    let (url, width, height) = split_image_size(raw_url);
    Some(ImageTarget {
        start: bang,
        end: full_end,
        alt,
        url,
        width,
        height,
    })
}

/// Find the `![alt](url)` image whose byte range contains `cursor`. Scans
/// backwards from the cursor for the nearest `![` and tries to parse a full
/// image; if the cursor isn't inside that one, keeps looking earlier. Returns
/// `None` when the cursor isn't inside any image.
pub fn find_image_under_cursor(body: &str, cursor: usize) -> Option<ImageTarget> {
    let cur = cursor.min(body.len());
    let mut search_end = cur;
    loop {
        let bang = body[..search_end].rfind("![")?;
        if let Some(target) = parse_image_at(body, bang)
            && cur >= target.start
            && cur <= target.end
        {
            return Some(target);
        }
        // Cursor is past this image — try the previous `![`.
        if bang == 0 {
            return None;
        }
        search_end = bang;
    }
}

/// Find the first `![alt](url)` image in the body. Used as a fallback by the
/// image-size tool when the caret isn't on any image.
pub fn find_first_image(body: &str) -> Option<ImageTarget> {
    let bang = body.find("![")?;
    parse_image_at(body, bang)
}

/// Capture the textarea selection synchronously from the DOM.
///
/// The DOM `selectionStart`/`selectionEnd` are UTF-16 code-unit offsets, but
/// Rust strings are indexed by bytes. We convert UTF-16 code units → byte
/// offsets so the caller can safely slice `&str`.
pub fn read_cursor_pos() -> Option<(usize, usize)> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlTextAreaElement;

    let document = web_sys::window()?.document()?;
    let el = document.get_element_by_id(BODY_TEXTAREA_ID)?;
    let ta: HtmlTextAreaElement = el.dyn_into().ok()?;
    let start = ta.selection_start().ok().flatten()? as usize;
    let end = ta.selection_end().ok().flatten()? as usize;
    let value = ta.value();
    Some((
        utf16_offset_to_byte(&value, start),
        utf16_offset_to_byte(&value, end),
    ))
}

/// Convert a UTF-16 code-unit offset into `s` to a Rust byte offset.
/// Clamps to the string length so out-of-range values never panic.
fn utf16_offset_to_byte(s: &str, utf16_offset: usize) -> usize {
    let mut units = 0usize;
    for (byte_idx, ch) in s.char_indices() {
        if units >= utf16_offset {
            return byte_idx;
        }
        units += ch.len_utf16();
    }
    s.len()
}

/// Move the textarea selection to `[start, end)` (Rust byte offsets into
/// `body`) and refocus it.
///
/// Must run *after* Dioxus reconciles the controlled `value` back into the
/// DOM, otherwise the browser snaps the caret to the end.
pub fn restore_selection_after_render(start: usize, end: usize) {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlTextAreaElement;

    let (utf16_start, utf16_end) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(BODY_TEXTAREA_ID))
        .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok())
        .map(|ta| {
            let value = ta.value();
            (
                byte_offset_to_utf16(&value, start),
                byte_offset_to_utf16(&value, end),
            )
        })
        .unwrap_or((start, end));

    let script = format!(
        "setTimeout(() => {{ const ta = document.getElementById('{id}'); if (!ta) return; ta.focus(); ta.setSelectionRange({s}, {e}); }}, 0);",
        id = BODY_TEXTAREA_ID,
        s = utf16_start,
        e = utf16_end,
    );
    let _ = document::eval(script.as_str());
}

/// Place the caret (collapsed selection) at `caret`. See
/// [`restore_selection_after_render`].
pub fn restore_cursor_after_render(caret: usize) {
    restore_selection_after_render(caret, caret);
}

/// Convert a Rust byte offset into `s` to a UTF-16 code-unit offset the DOM
/// API expects. Clamps to the string length.
fn byte_offset_to_utf16(s: &str, byte_offset: usize) -> usize {
    let mut units = 0usize;
    for (byte_idx, ch) in s.char_indices() {
        if byte_idx >= byte_offset {
            return units;
        }
        units += ch.len_utf16();
    }
    units
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_at_cursor_splices_inside_body() {
        let (new_body, caret) = insert_at_cursor("hello world", "X", Some((5, 5)));
        assert_eq!(new_body, "helloX world");
        assert_eq!(caret, 6);
    }

    #[test]
    fn insert_at_cursor_replaces_selection() {
        let (new_body, caret) = insert_at_cursor("hello world", "X", Some((0, 5)));
        assert_eq!(new_body, "X world");
        assert_eq!(caret, 1);
    }

    #[test]
    fn insert_at_cursor_appends_without_cursor() {
        let (new_body, caret) = insert_at_cursor("abc", "X", None);
        assert_eq!(new_body, "abc\n\nX");
        assert_eq!(caret, new_body.len());
    }

    #[test]
    fn insert_at_cursor_appends_on_out_of_range_cursor() {
        let (new_body, caret) = insert_at_cursor("abc", "X", Some((10, 12)));
        assert_eq!(new_body, "abc\n\nX");
        assert_eq!(caret, new_body.len());
    }

    #[test]
    fn insert_at_cursor_into_empty_body() {
        let (new_body, caret) = insert_at_cursor("", "X", None);
        assert_eq!(new_body, "X");
        assert_eq!(caret, 1);
    }

    #[test]
    fn utf16_byte_roundtrip_multibyte() {
        // emoji (surrogate pair) + CJK (3-byte) + ascii
        let s = "a\u{1F600}\u{4F60}b";
        // Only char boundaries are meaningful offsets (DOM never reports
        // mid-char positions); mid-boundary bytes snap forward to the next
        // char start.
        for byte in s.char_indices().map(|(i, _)| i).chain([s.len()]) {
            let utf16 = byte_offset_to_utf16(s, byte);
            assert_eq!(utf16_offset_to_byte(s, utf16), byte);
        }
    }

    #[test]
    fn utf16_offset_clamps_to_len() {
        assert_eq!(utf16_offset_to_byte("ab", 99), 2);
        assert_eq!(byte_offset_to_utf16("ab", 99), 2);
    }

    #[test]
    fn find_image_plain() {
        let body = "before ![alt](http://x/y.png) after";
        let start = body.find("![");
        let t = find_image_under_cursor(body, body.find("y.png").unwrap()).unwrap();
        assert_eq!(t.alt, "alt");
        assert_eq!(t.url, "http://x/y.png");
        assert_eq!(Some(t.start), start);
        assert_eq!(t.width, None);
    }

    #[test]
    fn find_image_with_size_fragment() {
        let body = "![pic](http://x/y.png#img=200x100)";
        let t = find_image_under_cursor(body, 2).unwrap();
        assert_eq!(t.url, "http://x/y.png");
        assert_eq!(t.width, Some(200));
        assert_eq!(t.height, Some(100));
        assert_eq!(t.to_markdown(None, None), "![pic](http://x/y.png)");
        assert_eq!(
            t.to_markdown(Some(10), None),
            "![pic](http://x/y.png#img=10)"
        );
    }

    #[test]
    fn find_image_none_outside() {
        assert!(find_image_under_cursor("no images here", 5).is_none());
    }

    #[test]
    fn find_first_image_picks_earliest() {
        let body = "![one](a.png) text ![two](b.png)";
        let t = find_first_image(body).unwrap();
        assert_eq!(t.alt, "one");
        assert_eq!(t.url, "a.png");
        assert!(find_first_image("no image").is_none());
    }
}
