use crate::models::Tag;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag as MdTag, TagEnd, html};

const FRONTMATTER_START: &str = "---";
const FRONTMATTER_END: &str = "---";
const TAGS_KEY: &str = "tags";
/// URL-fragment marker that signals an image size hint, e.g.
/// `![alt](photo.png#img=200x100)`. Picked so it almost never collides with a
/// real page anchor on an image URL.
const IMG_SIZE_MARKER: &str = "img=";

/// CSS class wrapping every rendered markdown tree.
/// See `ui_core/assets/markdown.css` for the styles.
pub const MARKDOWN_CONTAINER_CLASS: &str = "md-render";

/// Converts markdown to HTML with correct whitespace and indentation handling.
///
/// Unlike a blind `\n -> "  \n"` replace, this walks the parser event stream so:
/// - Inside **code blocks** (fenced ``` or indented), line breaks and indentation
///   are preserved verbatim — no spurious `<br>` injected.
/// - Outside code blocks, a single newline becomes a hard line break (`<br>`)
///   so prose wraps the way the author typed it.
/// - GFM tables, strikethrough, and task lists are enabled.
pub fn render_markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let transformed = SoftBreakToHardOutsideCode::new(parser);
    let transformed = SizedImages::new(transformed);

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed);
    html_output
}

/// Iterator adapter that converts `SoftBreak` into `HardBreak` only while the
/// cursor is **not** inside a code block. Inside code, soft breaks are kept as
/// soft breaks (rendered as plain newlines by `pulldown-cmark::html`), so the
/// author's indentation and line wrapping survive intact.
struct SoftBreakToHardOutsideCode<I> {
    inner: I,
    in_code: bool,
}

impl<I> SoftBreakToHardOutsideCode<I> {
    fn new(inner: I) -> Self {
        Self {
            inner,
            in_code: false,
        }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for SoftBreakToHardOutsideCode<I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ev = self.inner.next()?;
        match ev {
            Event::Start(MdTag::CodeBlock(_)) => {
                self.in_code = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                self.in_code = false;
            }
            Event::SoftBreak if !self.in_code => return Some(Event::HardBreak),
            _ => {}
        }
        Some(ev)
    }
}

/// Parsed `#img=...` size spec stripped from an image URL.
///
/// `width` and/or `height` may be `None` (e.g. `#img=200` sets only width).
struct ImageSize {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
}

/// If `dest` carries a trailing `#img=WxH` size fragment, split it into the
/// clean URL and the dimensions. Recognised forms:
///
/// - `#img=200x100` → width 200, height 100
/// - `#img=200`     → width only
/// - `#img=x100`    → height only
///
/// Returns `None` when there is no `#img=` marker or the spec is empty/invalid,
/// so plain images and ordinary page anchors pass through untouched.
fn parse_image_size(dest: &str) -> Option<ImageSize> {
    let hash_idx = dest.find('#')?;
    let (url, frag) = dest.split_at(hash_idx);
    let spec = frag[1..].strip_prefix(IMG_SIZE_MARKER)?;
    if spec.is_empty() {
        return None;
    }
    let (width, height) = match spec.split_once('x') {
        Some((w, h)) => (
            if w.is_empty() {
                None
            } else {
                Some(w.parse::<u32>().ok()?)
            },
            if h.is_empty() {
                None
            } else {
                Some(h.parse::<u32>().ok()?)
            },
        ),
        None => (Some(spec.parse::<u32>().ok()?), None),
    };
    if width.is_none() && height.is_none() {
        return None;
    }
    Some(ImageSize {
        url: url.to_string(),
        width,
        height,
    })
}

/// Escape a string for safe interpolation into a double-quoted HTML attribute.
fn escape_html_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Build an `<img>` element with explicit width/height inline styles so the
/// size always wins over the responsive `.md-render img` stylesheet.
fn build_sized_img_html(url: &str, alt: &str, width: Option<u32>, height: Option<u32>) -> String {
    let mut style = String::new();
    if let Some(w) = width {
        style.push_str(&format!("width:{w}px;"));
    }
    if let Some(h) = height {
        style.push_str(&format!("height:{h}px;"));
    }
    format!(
        "<img src=\"{url}\" alt=\"{alt}\" style=\"{style}\" loading=\"lazy\">",
        url = escape_html_attr(url),
        alt = escape_html_attr(alt),
        style = style,
    )
}

/// Split a markdown image URL into `(clean_url, width, height)`. If the URL
/// carries a `#img=WxH` fragment, it is stripped and the dimensions parsed
/// out; otherwise the URL is returned unchanged with both dims as `None`.
///
/// Used by the editor UI to read the current size of an existing image.
pub fn split_image_size(dest: &str) -> (String, Option<u32>, Option<u32>) {
    match parse_image_size(dest) {
        Some(s) => (s.url, s.width, s.height),
        None => (dest.to_string(), None, None),
    }
}

/// Build a `#img=...` fragment string for the given dimensions, or an empty
/// string when both are `None` (meaning "no size, render responsive").
///
/// Formats:
/// - `(Some(w), Some(h))` → `#img={w}x{h}`
/// - `(Some(w), None)`    → `#img={w}`
/// - `(None, Some(h))`    → `#img=x{h}`
/// - `(None, None)`       → `""`
pub fn build_image_size_fragment(width: Option<u32>, height: Option<u32>) -> String {
    match (width, height) {
        (Some(w), Some(h)) => format!("#img={w}x{h}"),
        (Some(w), None) => format!("#img={w}"),
        (None, Some(h)) => format!("#img=x{h}"),
        (None, None) => String::new(),
    }
}

/// Iterator adapter that replaces image nodes carrying a `#img=WxH` size
/// fragment with a single `<img>` element that has explicit width/height
/// styles. Images without the marker pass through unchanged (pulldown-cmark
/// already renders them as `<img>`).
struct SizedImages<I> {
    inner: I,
    /// When set, we are inside a sized image's alt sub-tree, buffering text
    /// until the matching `End(Image)`.
    pending: Option<PendingImage>,
}

struct PendingImage {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    alt: String,
}

impl<I> SizedImages<I> {
    fn new(inner: I) -> Self {
        Self {
            inner,
            pending: None,
        }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for SizedImages<I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        loop {
            let ev = self.inner.next()?;

            if let Some(pending) = self.pending.as_mut() {
                match ev {
                    Event::Text(s) | Event::Code(s) | Event::InlineHtml(s) => {
                        pending.alt.push_str(&s)
                    }
                    Event::End(TagEnd::Image) => {
                        let PendingImage {
                            url,
                            width,
                            height,
                            alt,
                        } = self.pending.take().expect("pending set in branch");
                        let html_str = build_sized_img_html(&url, &alt, width, height);
                        return Some(Event::InlineHtml(CowStr::from(html_str)));
                    }
                    // Any inline formatting inside the alt is ignored — we
                    // only need the flattened text for `alt=`.
                    _ => {}
                }
                continue;
            }

            if let Event::Start(MdTag::Image { dest_url, .. }) = &ev
                && let Some(size) = parse_image_size(dest_url)
            {
                self.pending = Some(PendingImage {
                    url: size.url,
                    width: size.width,
                    height: size.height,
                    alt: String::new(),
                });
                continue;
            }

            return Some(ev);
        }
    }
}

pub fn add_tag_frontmarkter(content: &str, tags: &[Tag]) -> String {
    if tags.is_empty() {
        return content.to_string();
    }

    let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
    let frontmatter = format_tag_frontmatter(&tag_names);

    if content.starts_with(FRONTMATTER_START) {
        let lines: Vec<&str> = content.lines().collect();

        // Find the tags section in existing frontmatter
        let tags_start = lines
            .iter()
            .position(|line| *line == format!("{}:", TAGS_KEY));

        if let Some(tags_idx) = tags_start {
            // Find the end of the tags section (next line that's not indented or a new key)
            let tags_end = tags_idx
                + 1
                + lines[tags_idx + 1..]
                    .iter()
                    .take_while(|line| line.starts_with("  ") || line.is_empty())
                    .count();

            // Build new tags lines
            let new_tag_lines: Vec<String> = tag_names
                .iter()
                .map(|name| format!("  - {}", name))
                .collect();

            // Replace old tags with new tags
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            new_lines.splice(tags_idx + 1..tags_end, new_tag_lines);

            return new_lines.join("\n");
        }
    }

    format!("{}\n{}", frontmatter, content)
}

pub fn format_tag_frontmatter(tag_names: &[String]) -> String {
    let tags_yaml = tag_names
        .iter()
        .map(|name| format!("  - {}", name))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{}\n{}:\n{}\n{}",
        FRONTMATTER_START, TAGS_KEY, tags_yaml, FRONTMATTER_END
    )
}

pub fn parse_tag_frontmatter(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first().is_none_or(|l| *l != FRONTMATTER_START) {
        return Vec::new();
    }

    let mut tags = Vec::new();
    let mut in_tags_section = false;

    for line in lines.iter().skip(1) {
        if *line == FRONTMATTER_END {
            break;
        }

        if line.starts_with(TAGS_KEY) {
            in_tags_section = true;
            continue;
        }

        if in_tags_section && let Some(tag_name) = parse_tag_line(line) {
            tags.push(tag_name);
        }
    }

    tags
}

fn parse_tag_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("-")
        .map(|stripped| stripped.trim().to_string())
}

pub fn strip_frontmatter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first().is_none_or(|l| *l != FRONTMATTER_START) {
        return content.to_string();
    }

    if let Some(end_idx) = lines[1..].iter().position(|line| *line == FRONTMATTER_END) {
        lines[(end_idx + 2)..].join("\n")
    } else {
        content.to_string()
    }
}

pub fn update_tags_in_frontmatter(content: &str, new_tags: &[Tag]) -> String {
    let existing_tags = parse_tag_frontmatter(content);
    let new_tag_names: Vec<String> = new_tags.iter().map(|t| t.name.clone()).collect();

    if existing_tags == new_tag_names {
        return content.to_string();
    }

    let content_without_frontmatter = strip_frontmatter(content);
    add_tag_frontmarkter(&content_without_frontmatter, new_tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tag_frontmatter() {
        let tags = vec!["technology".to_string(), "programming".to_string()];
        let result = format_tag_frontmatter(&tags);
        assert!(result.contains("tags:"));
        assert!(result.contains("- technology"));
        assert!(result.contains("- programming"));
    }

    #[test]
    fn test_add_tag_frontmatter() {
        let content = "Hello world";
        let tags = vec![Tag {
            id: Some(1),
            name: "technology".to_string(),
            slug: "technology".to_string(),
            parent_id: None,
            created_at: None,
            updated_at: None,
            synced_at: None,
        }];
        let result = add_tag_frontmarkter(content, &tags);
        assert!(result.starts_with("---"));
        assert!(result.contains("tags:"));
        assert!(result.contains("- technology"));
        assert!(result.ends_with("Hello world"));
    }

    #[test]
    fn test_parse_tag_frontmatter() {
        let content = r#"---
tags:
  - technology
  - programming
---

Hello world"#;
        let result = parse_tag_frontmatter(content);
        assert_eq!(result, vec!["technology", "programming"]);
    }

    #[test]
    fn test_update_tags_in_frontmatter() {
        let content = r#"---
tags:
  - old-tag
---

Hello world"#;
        let new_tags = vec![Tag {
            id: Some(1),
            name: "new-tag".to_string(),
            slug: "new-tag".to_string(),
            parent_id: None,
            created_at: None,
            updated_at: None,
            synced_at: None,
        }];

        // Debug output
        let parsed = parse_tag_frontmatter(content);
        println!("Parsed tags: {:?}", parsed);

        let stripped = strip_frontmatter(content);
        println!("Stripped content: {:?}", stripped);
        println!("Stripped contains '---': {}", stripped.contains("---"));

        let result = update_tags_in_frontmatter(content, &new_tags);
        println!("Result: {:?}", result);
        println!("Result contains 'new-tag': {}", result.contains("new-tag"));
        println!("Result contains 'old-tag': {}", result.contains("old-tag"));

        assert!(result.contains("- new-tag"));
        assert!(!result.contains("- old-tag"));
    }

    #[test]
    fn test_render_markdown_to_html_single_newline() {
        let markdown = "Hello\nWorld";
        let html = render_markdown_to_html(markdown);
        println!("Single newline HTML output: {}", html);
        // Single newline should create line break (<br>)
        assert!(html.contains("<p>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<br />"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_render_markdown_to_html_double_newline() {
        let markdown = "Hello\n\nWorld";
        let html = render_markdown_to_html(markdown);
        // Double newline should create separate paragraphs
        assert!(html.contains("<p>"));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn test_render_markdown_to_html_with_bold() {
        let markdown = "**Bold** text";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<strong>Bold</strong>"));
    }

    #[test]
    fn test_render_markdown_to_html_with_link() {
        let markdown = "[Link](https://example.com)";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("Link"));
    }

    #[test]
    fn test_render_fenced_code_block_preserves_newlines() {
        // Code blocks must keep the author's line breaks verbatim — no injected <br>.
        let markdown = "```rust\nfn main() {\n    let x = 1;\n}\n```";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<pre>"), "expected <pre> wrapper");
        assert!(html.contains("<code"), "expected <code> element");
        assert!(!html.contains("<br"), "code blocks must not contain <br>");
        assert!(html.contains("fn main()"));
        assert!(html.contains("let x = 1;"));
    }

    #[test]
    fn test_render_indented_code_block_no_br() {
        // Indented (4-space) code block — also must not get <br> injections.
        let markdown = "    line one\n    line two\n    line three";
        let html = render_markdown_to_html(markdown);
        assert!(
            html.contains("<pre"),
            "expected indented code block as <pre>"
        );
        assert!(
            !html.contains("<br"),
            "indented code block must not contain <br>"
        );
    }

    #[test]
    fn test_render_nested_list_indent_preserved() {
        let markdown = "- top\n  - nested\n    - deeper\n- back";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>top"));
        assert!(html.contains("nested"));
        assert!(html.contains("deeper"));
        // Two levels of nesting => at least two nested <ul> opens.
        assert_eq!(html.matches("<ul").count(), 3);
    }

    #[test]
    fn test_render_gfm_task_list() {
        let markdown = "- [x] done\n- [ ] todo";
        let html = render_markdown_to_html(markdown);
        assert!(
            html.contains("<input"),
            "task list must render checkbox inputs"
        );
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("checked"), "checked task should be marked");
        assert!(html.contains("done"));
        assert!(html.contains("todo"));
    }

    #[test]
    fn test_render_table() {
        let markdown = "| Name | Age |\n| --- | --- |\n| Ada | 36 |";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<td>Ada</td>"));
    }

    #[test]
    fn test_render_blockquote_and_hr() {
        let markdown = "> wisdom\n\n---\n\nafter";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("wisdom"));
        assert!(html.contains("<hr"));
    }

    /// Regression: leading whitespace inside fenced code blocks must survive
    /// in the HTML output (the visible indent problem). CSS handles display,
    /// but the bytes must contain the spaces.
    #[test]
    fn test_code_block_leading_whitespace_survives() {
        let markdown = "```rust\n       let state = if led.is_set_low() {\n                Level::High\n            } else {\n                Level::Low\n            };\n```";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<pre"));
        // 7-space indent on first line must be in the output verbatim.
        assert!(
            html.contains("       let state"),
            "leading whitespace lost; got: {html}"
        );
        // 16-space indent on the inner line must also survive.
        assert!(html.contains("                Level::High"));
        // And the relative indent of `} else {`.
        assert!(html.contains("            } else {"));
    }

    #[test]
    fn test_parse_image_size_width_and_height() {
        let s = parse_image_size("photo.png#img=200x100").unwrap();
        assert_eq!(s.url, "photo.png");
        assert_eq!(s.width, Some(200));
        assert_eq!(s.height, Some(100));
    }

    #[test]
    fn test_parse_image_size_width_only() {
        let s = parse_image_size("photo.png#img=320").unwrap();
        assert_eq!(s.url, "photo.png");
        assert_eq!(s.width, Some(320));
        assert_eq!(s.height, None);
    }

    #[test]
    fn test_parse_image_size_height_only() {
        let s = parse_image_size("photo.png#img=x240").unwrap();
        assert_eq!(s.url, "photo.png");
        assert_eq!(s.width, None);
        assert_eq!(s.height, Some(240));
    }

    #[test]
    fn test_parse_image_size_none_without_marker() {
        assert!(parse_image_size("photo.png").is_none());
        // A real page anchor must not be misread as a size spec.
        assert!(parse_image_size("page.html#section").is_none());
        // Empty / malformed specs are rejected.
        assert!(parse_image_size("photo.png#img=").is_none());
        assert!(parse_image_size("photo.png#img=abc").is_none());
        assert!(parse_image_size("photo.png#img=x").is_none());
    }

    #[test]
    fn test_split_image_size_strips_fragment() {
        let (url, w, h) = split_image_size("photo.png#img=200x100");
        assert_eq!(url, "photo.png");
        assert_eq!(w, Some(200));
        assert_eq!(h, Some(100));
    }

    #[test]
    fn test_split_image_size_no_marker_passes_through() {
        let (url, w, h) = split_image_size("photo.png");
        assert_eq!(url, "photo.png");
        assert_eq!(w, None);
        assert_eq!(h, None);
    }

    #[test]
    fn test_build_image_size_fragment_all_combinations() {
        assert_eq!(
            build_image_size_fragment(Some(200), Some(100)),
            "#img=200x100"
        );
        assert_eq!(build_image_size_fragment(Some(200), None), "#img=200");
        assert_eq!(build_image_size_fragment(None, Some(100)), "#img=x100");
        assert_eq!(build_image_size_fragment(None, None), "");
    }

    #[test]
    fn test_split_and_build_roundtrip() {
        for (w, h) in [
            (Some(200u32), Some(100u32)),
            (Some(200), None),
            (None, Some(100)),
        ] {
            let frag = build_image_size_fragment(w, h);
            let url = format!("photo.png{frag}");
            let (clean, w2, h2) = split_image_size(&url);
            assert_eq!(clean, "photo.png");
            assert_eq!(w2, w);
            assert_eq!(h2, h);
        }
    }

    #[test]
    fn test_render_image_with_size_fragment() {
        let markdown = "![logo](photo.png#img=200x100)";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<img"), "expected <img>; got: {html}");
        // Fragment must be stripped from the src.
        assert!(html.contains("src=\"photo.png\""));
        assert!(!html.contains("#img="));
        assert!(html.contains("alt=\"logo\""));
        assert!(html.contains("width:200px"));
        assert!(html.contains("height:100px"));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_render_image_width_only_fragment() {
        let markdown = "![pic](/a/b.png#img=320)";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("src=\"/a/b.png\""));
        assert!(html.contains("width:320px"));
        assert!(
            !html.contains("height:"),
            "no height style when only width set; got: {html}"
        );
    }

    #[test]
    fn test_render_image_without_size_marker_unchanged() {
        let markdown = "![plain](photo.png)";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"photo.png\""));
        // Plain images must not gain inline size styles.
        assert!(!html.contains("style="));
    }

    #[test]
    fn test_render_image_escapes_alt_and_url() {
        // Alt and URL carrying HTML metacharacters must be escaped so they
        // can't break out of the attribute.
        let markdown = "![a & b <c>](u.png?q=1&b=2#img=50x50)";
        let html = render_markdown_to_html(markdown);
        assert!(html.contains("&amp;"));
        assert!(html.contains("&lt;c&gt;"));
        // No raw, unescaped angle brackets from the alt.
        assert!(!html.contains("<c>"));
    }
}
