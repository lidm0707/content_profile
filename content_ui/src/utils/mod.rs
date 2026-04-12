pub mod config;
pub mod markdown;

use pulldown_cmark::{Parser, html};

/// Renders Markdown to HTML using pulldown-cmark
pub fn render_markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Wraps text with markdown syntax
pub fn wrap_with_markdown(text: &str, before: &str, after: &str) -> String {
    format!("{}{}{}", before, text, after)
}

/// Formats text as bold (**text**)
pub fn format_bold(text: &str) -> String {
    wrap_with_markdown(text, "**", "**")
}

/// Formats text as italic (*text*)
pub fn format_italic(text: &str) -> String {
    wrap_with_markdown(text, "*", "*")
}

/// Formats text as inline code (`code`)
pub fn format_code(text: &str) -> String {
    wrap_with_markdown(text, "`", "`")
}

/// Formats text as a code block (```)
pub fn format_code_block(text: &str) -> String {
    format!("```\n{}\n```", text)
}

/// Formats text as a heading (# Heading)
pub fn format_heading(text: &str, level: u8) -> String {
    let hashes = "#".repeat(level as usize);
    format!("{} {}", hashes, text)
}

/// Formats text as a markdown link ([text](url))
pub fn format_link(text: &str, url: &str) -> String {
    format!("[{}]({})", text, url)
}

/// Formats text as a markdown image (![alt text](image_url))
pub fn format_image(alt_text: &str, image_url: &str) -> String {
    format!("![{}]({})", alt_text, image_url)
}

/// Formats text as a list item (- item)
pub fn format_unordered_list(text: &str) -> String {
    format!("- {}", text)
}

/// Formats text as an ordered list item (1. item)
pub fn format_ordered_list(text: &str, number: u32) -> String {
    format!("{}. {}", number, text)
}

/// Formats text as a blockquote (> text)
pub fn format_blockquote(text: &str) -> String {
    text.lines()
        .map(|line| format!("> {}", line))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bold() {
        assert_eq!(format_bold("hello"), "**hello**");
    }

    #[test]
    fn test_format_italic() {
        assert_eq!(format_italic("hello"), "*hello*");
    }

    #[test]
    fn test_format_code() {
        assert_eq!(format_code("hello"), "`hello`");
    }

    #[test]
    fn test_format_heading() {
        assert_eq!(format_heading("Hello", 1), "# Hello");
        assert_eq!(format_heading("Hello", 2), "## Hello");
    }

    #[test]
    fn test_format_link() {
        assert_eq!(
            format_link("Example", "https://example.com"),
            "[Example](https://example.com)"
        );
    }

    #[test]
    fn test_format_image() {
        assert_eq!(
            format_image("My Image", "https://example.com/image.jpg"),
            "![My Image](https://example.com/image.jpg)"
        );
    }

    #[test]
    fn test_format_blockquote() {
        assert_eq!(format_blockquote("hello\nworld"), "> hello\n> world");
    }
}
