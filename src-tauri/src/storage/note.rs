use serde::{Serialize, Deserialize};
use std::path::Path;
use std::io::{self, Error, ErrorKind};
use tantivy::Result as TantivyResult;
use regex::Regex;
use std::fs;
use std::sync::Arc;

use crate::utils::{file_operations, string_utils, markdown};
use crate::storage::vault::Vault;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
}

impl Note {
    // Creates a new Markdown note.
    pub fn create_note(vault_path: &str, title: &str, content: &str) -> io::Result<()> {
        let safe_title = string_utils::sanitize_filename(title);
        let clean_content = string_utils::normalize_whitespace(content);
    
        println!("🛠️ Debug: Safe title - '{}'", safe_title);
        println!("🛠️ Debug: Clean content - '{}'", clean_content);
    
        // Check if the vault directory exists
        if !Path::new(vault_path).exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("❌ Vault directory does not exist: {}", vault_path),
            ));
        }
    
        let note_path = format!("{}/{}.md", vault_path, safe_title);
        println!("📝 Creating note at: {}", note_path);
    
        // Write the note content to the file
        file_operations::write_to_file(&note_path, &clean_content)?;
    
        // Verify that the file was created
        if !Path::new(&note_path).exists() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("❌ File was not created: {}", note_path),
            ));
        }
    
        // Verify that the file is not empty
        let verify_content = file_operations::read_from_file(&note_path)?;
        println!("🛠️ Debug: Verify content - '{}'", verify_content);
    
        if verify_content.is_empty() {
            return Err(Error::new(
                ErrorKind::Other,
                "❌ File was created but is empty",
            ));
        }
    
        println!("✅ Note successfully created: {}", note_path);
        Ok(())
    }

    // Reads a Markdown note as raw content.
    pub fn read_note(vault_path: &str, title: &str) -> io::Result<String> {
        let safe_title = string_utils::sanitize_filename(title);
        let note_path = format!("{}/{}.md", vault_path, safe_title);
        
        println!("📖 Debug: Attempting to read note from {}", note_path);

        if !Path::new(&note_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "❌ Note file does not exist"));
        }

        let content = file_operations::read_from_file(&note_path)?;
        println!("📖 Debug: Read content - '{}'", content);
        Ok(content)
    }

    // Deletes a Markdown note.
    pub fn delete_note(vault_path: &str, title: &str) -> io::Result<()> {
        let safe_title = string_utils::sanitize_filename(title);
        let note_path = format!("{}/{}.md", vault_path, safe_title);
    
        if Path::new(&note_path).exists() {
            println!("🗑️ Deleting note: {}", note_path);
            file_operations::delete_file(&note_path)?;
        }
    
        if Path::new(&note_path).exists() {
            println!("❌ File still exists after deletion: {}", note_path);
            return Err(Error::new(ErrorKind::Other, "❌ File was not deleted"));
        }
    
        println!("✅ Note successfully deleted: {}", note_path);
        Ok(())
    }

    // Lists all notes in a vault.
    pub fn list_notes(vault_path: &str) -> io::Result<Vec<String>> {
        let paths = fs::read_dir(vault_path)?;
        Ok(paths
            .filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().ok()).flatten())
            .filter(|name| name.ends_with(".md"))
            .map(|name| name.trim_end_matches(".md").to_string())
            .collect())
    }

    // Converts a Markdown note into HTML.
    pub fn render_html(vault_path: &str, title: &str) -> std::result::Result<String, String> {
        let content = Self::read_note(vault_path, title).map_err(|e| e.to_string())?;
        Ok(markdown::render_markdown(&content))
    }

    // Extracts wikilinks ([[Link]]) from a note.
    pub fn extract_links(content: &str) -> Vec<String> {
        let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        let matches: Vec<String> = re
            .captures_iter(content)
            .map(|cap| cap[1].trim().to_string()) // Trim whitespace
            .collect();
        
        println!("🔗 Debug: Extracted links - {:?}", matches);
        matches
    }

    // Indexes a note in the vault.
    pub fn index_note_in_vault(vault_path: &str, title: &str) -> TantivyResult<()> {
        let content = Note::read_note(vault_path, title)
            .map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;
    
        let vault_name = std::path::Path::new(vault_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| tantivy::TantivyError::InvalidArgument("Invalid vault path".into()))?;
    
        println!("🔍 Debug: Indexing note '{}' in vault '{}'", title, vault_name);
    
        let vault = Vault::create_vault(vault_name, vault_path)?;
        vault.index_note(title, &content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VAULT: &str = "test_notes";
    const TEST_NOTE: &str = "test_note";

    fn setup() {
        // Clean up any existing test directory
        cleanup();
    
        // Create the test directory
        file_operations::create_directory(TEST_VAULT).unwrap();
    }
    
    fn cleanup() {
        if Path::new(TEST_VAULT).exists() {
            file_operations::delete_directory(TEST_VAULT).unwrap();
        }
    }

    #[test]
    fn test_create_and_read_note() {
        setup();

        let content = "This is a test note.";
        let expected_clean_content = "This is a test note.";

        // Create the note
        let result = Note::create_note(TEST_VAULT, TEST_NOTE, content);
        assert!(result.is_ok(), "❌ Note creation should succeed");

        // Read the note
        let read_result = Note::read_note(TEST_VAULT, TEST_NOTE);
        assert!(read_result.is_ok(), "❌ Reading the note should succeed");

        // Verify the content
        let read_content = read_result.unwrap().trim().to_string();
        assert_eq!(read_content, expected_clean_content, "❌ Read content should match sanitized input");

        cleanup();
    }

    #[test]
    fn test_extract_links() {
        setup();

        let md_content = "This note links to [[AnotherNote]] and [[TestNote]].";
        let result = Note::create_note(TEST_VAULT, "test_note_links", md_content); // Use a unique title
        assert!(result.is_ok(), "❌ Creating note with links should succeed");

        // Read the note
        let content = Note::read_note(TEST_VAULT, "test_note_links").unwrap();

        // Extract links
        let links = Note::extract_links(&content);
        assert!(!links.is_empty(), "❌ Extracting links should succeed");

        // Verify the extracted links
        let extracted_links = links;
        println!("🔗 Extracted Links: {:?}", extracted_links);
        assert_eq!(extracted_links, vec!["AnotherNote", "TestNote"], "❌ Extracted links should match expected values");

        cleanup();
    }

    #[test]
    fn test_render_html() {
        setup();

        let md_content = "# Title\nThis is **bold**."; // Add a newline after the header
        let result = Note::create_note(TEST_VAULT, "test_note_html", md_content);
        assert!(result.is_ok(), "❌ Creating markdown note should succeed");

        // Render the note to HTML
        let html_result = Note::render_html(TEST_VAULT, "test_note_html");
        assert!(html_result.is_ok(), "❌ Rendering markdown to HTML should succeed");

        // Verify the rendered HTML
        let html_content = html_result.unwrap();
        println!("🖥️ Rendered HTML: {}", html_content);

        assert!(html_content.contains("<h1>Title</h1>"), "❌ Markdown header should be converted");
        assert!(html_content.contains("<strong>bold</strong>"), "❌ Bold text should be formatted");

        cleanup();
    }

    #[test]
    fn test_delete_note() {
        setup();

        let result = Note::create_note(TEST_VAULT, TEST_NOTE, "Test content");
        assert!(result.is_ok(), "❌ Creating note should succeed");

        let delete_result = Note::delete_note(TEST_VAULT, TEST_NOTE);
        assert!(delete_result.is_ok(), "❌ Deleting note should succeed");

        let read_result = Note::read_note(TEST_VAULT, TEST_NOTE);
        assert!(read_result.is_err(), "❌ Reading a deleted note should fail");

        cleanup();
    }
}