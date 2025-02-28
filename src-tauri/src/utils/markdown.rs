use pulldown_cmark::{Parser, Options, Event, Tag, html};
use regex::Regex;

// Renders Markdown content to HTML.
pub fn render_markdown(content: &str) -> String {
    let parser = Parser::new(content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

// Extracts Wikilinks ([[wikilink]]) from Markdown content.
pub fn extract_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    for cap in re.captures_iter(content) {
        if let Some(link) = cap.get(1) {
            links.push(link.as_str().to_string());
        }
    }

    links
}

// Parses a Markdown file and returns its plain text content.
pub fn parse_markdown_file(file_path: &str) -> Result<String, String> {
    std::fs::read_to_string(file_path).map_err(|e| format!("Error reading file: {}", e))
}

// Extracts text-only content from Markdown (without formatting).
pub fn extract_plain_text(content: &str) -> String {
    let parser = Parser::new(content);
    let mut plain_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) => plain_text.push_str(&text),
            Event::SoftBreak | Event::HardBreak => plain_text.push('\n'),
            _ => {}
        }
    }

    plain_text
}