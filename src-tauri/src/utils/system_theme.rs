/// Parse GNOME's `org.gnome.desktop.interface color-scheme` value.
pub fn parse_color_scheme(value: &str) -> Option<bool> {
    let normalized = value.trim().trim_matches(['\'', '"']).to_ascii_lowercase();

    match normalized.as_str() {
        "prefer-dark" | "dark" => Some(true),
        "prefer-light" | "light" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_color_scheme;

    #[test]
    fn parses_gnome_color_scheme_values() {
        assert_eq!(parse_color_scheme("'prefer-dark'"), Some(true));
        assert_eq!(parse_color_scheme("  'prefer-light'  "), Some(false));
        assert_eq!(parse_color_scheme("'default'"), None);
    }
}
