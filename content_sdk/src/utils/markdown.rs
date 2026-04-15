use crate::models::Tag;
use pulldown_cmark::{Parser, html};

const FRONTMATTER_START: &str = "---";
const FRONTMATTER_END: &str = "---";
const TAGS_KEY: &str = "tags";

/// Converts markdown text to HTML, properly handling newlines
///
/// # Arguments
/// * `markdown` - The markdown text to convert
///
/// # Returns
/// * HTML string with proper newline handling
///
/// # Examples
/// ```ignore
/// let html = render_markdown_to_html("Hello\n\nWorld");
/// // Returns: "<p>Hello</p>\n<p>World</p>"
/// ```
pub fn render_markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
        // Single newline within text should be preserved in paragraph
        assert!(html.contains("<p>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_render_markdown_to_html_double_newline() {
        let markdown = "Hello\n\nWorld";
        let html = render_markdown_to_html(markdown);
        // Double newline should create separate paragraphs
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
}
