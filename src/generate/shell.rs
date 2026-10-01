pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

pub fn sh_words(items: &[String]) -> String {
    items.iter().map(|s| sh_quote(s)).collect::<Vec<_>>().join(" ")
}

pub fn iso_label(name: &str) -> String {
    let label: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
        .take(32)
        .collect();
    if label.is_empty() {
        "VOID_LIVE".into()
    } else {
        label
    }
}

pub fn menu_title(name: &str) -> String {
    let t: String = name.chars().filter(|c| !matches!(c, '"' | '\'' | '\\' | '$' | '`' | '\n' | '{' | '}')).collect();
    if t.trim().is_empty() {
        "Void Linux live".into()
    } else {
        t
    }
}
