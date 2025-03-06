use regex::Regex;

// Sanitizes a string for use as a filename by removing illegal characters.
pub fn sanitize_filename(name: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9-_]").unwrap();
    re.replace_all(name, "").to_string()
}

// Trims and normalizes whitespace in a string.
pub fn normalize_whitespace(input: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(input.trim(), " ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("valid_name"), "valid_name");
        assert_eq!(sanitize_filename("invalid name.txt"), "invalidnametxt");
        assert_eq!(sanitize_filename("123@file!"), "123file");
        assert_eq!(sanitize_filename("hello_world"), "hello_world");
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("   hello    world   "), "hello world");
        assert_eq!(normalize_whitespace("singleword"), "singleword");
        assert_eq!(normalize_whitespace("multiple    spaces   here"), "multiple spaces here");
    }
}
