use content_profile::models::Tag;

fn main() {
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
    
    // Test parse_tag_frontmatter
    let parsed = content_profile::utils::markdown::parse_tag_frontmatter(content);
    println!("Parsed tags: {:?}", parsed);
    
    // Test strip_frontmatter
    let stripped = content_profile::utils::markdown::strip_frontmatter(content);
    println!("Stripped content: {}", stripped);
    println!("Stripped contains '---': {}", stripped.contains("---"));
    println!("Stripped contains 'old-tag': {}", stripped.contains("old-tag"));
    
    // Test add_tag_frontmarkter
    let result = content_profile::utils::markdown::add_tag_frontmarkter(&stripped, &new_tags);
    println!("Result: {}", result);
    println!("Result contains 'new-tag': {}", result.contains("new-tag"));
    println!("Result contains 'old-tag': {}", result.contains("old-tag"));
    
    // Test update_tags_in_frontmatter
    let final_result = content_profile::utils::markdown::update_tags_in_frontmatter(content, &new_tags);
    println!("\nFinal result: {}", final_result);
    println!("Final contains 'new-tag': {}", final_result.contains("new-tag"));
    println!("Final contains 'old-tag': {}", final_result.contains("old-tag"));
}
