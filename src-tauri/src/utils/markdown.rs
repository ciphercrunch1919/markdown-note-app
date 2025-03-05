use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd, html};
use regex::Regex;
use ammonia::clean;

// Renders Markdown content to HTML.
pub fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Sanitize the HTML output
    sanitize_html(&html_output)
}

// Extracts Wikilinks ([[wikilink]]) from Markdown content.
pub fn extract_links(content: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|link| link.as_str().trim().to_string()))
        .collect()
}

// Parses a Markdown file and returns its content.
pub fn parse_markdown_file(file_path: &str) -> Result<String, String> {
    std::fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading file '{}': {}", file_path, e))
}

// Extracts text-only content from Markdown (without formatting).
pub fn extract_plain_text(content: &str) -> String {
    let parser = Parser::new(content);
    let mut plain_text = String::new();
    let mut in_header = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_header = true;
            }
            Event::End(TagEnd::Heading(level)) => {
                in_header = false;
                plain_text.push('\n'); // Add newline after header
            }
            Event::Text(text) => plain_text.push_str(&text),
            Event::Code(code) => plain_text.push_str(&code),
            Event::SoftBreak | Event::HardBreak => plain_text.push('\n'),
            _ => {}
        }
    }

    plain_text
}

// Sanitizes HTML to prevent XSS attacks.
pub fn sanitize_html(html: &str) -> String {
    clean(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown() {
        let md_content = "# Title\nThis is **bold**.";
        let html_content = render_markdown(md_content);
        assert!(html_content.contains("<h1>Title</h1>"));
        assert!(html_content.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_extract_links() {
        let md_content = "This note links to [[AnotherNote]] and [[TestNote]].";
        let links = extract_links(md_content);
        assert_eq!(links, vec!["AnotherNote", "TestNote"]);
    }

    #[test]
    fn test_extract_plain_text() {
        let md_content = "# Title\nThis is **bold**.";
        let plain_text = extract_plain_text(md_content);
        assert_eq!(plain_text, "Title\nThis is bold.");
    }

    #[test]
    fn test_parse_markdown_file() {
        let file_path = "test.md";
        let content = "# Test File\nThis is a test.";
        std::fs::write(file_path, content).unwrap();

        let parsed_content = parse_markdown_file(file_path).unwrap();
        assert_eq!(parsed_content, content);

        std::fs::remove_file(file_path).unwrap();
    }
}