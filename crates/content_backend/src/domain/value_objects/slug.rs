pub const SEPARATOR: char = '-';

pub fn generate_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                SEPARATOR
            }
        })
        .collect::<String>()
        .split(SEPARATOR)
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}
