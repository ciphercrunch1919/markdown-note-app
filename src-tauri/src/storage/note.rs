use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::io::{self, Error, ErrorKind};
use tantivy::Result as TantivyResult;
use regex::Regex;

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

        if !Path::new(vault_path).exists() {
            println!("📂 Creating vault directory: {}", vault_path);
            file_operations::create_directory(vault_path)?;
        }
    
        let note_path = format!("{}/{}.md", vault_path, safe_title);
        println!("📝 Creating note at: {}", note_path);
    
        file_operations::write_to_file(&note_path, &clean_content)?;
    
        if !Path::new(&note_path).exists() {
            return Err(Error::new(ErrorKind::Other, "❌ File was not created"));
        }
    
        let verify_content = file_operations::read_from_file(&note_path)?;
        if verify_content.is_empty() {
            return Err(Error::new(ErrorKind::Other, "❌ File was created but is empty"));
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
        let content = Self::read_note(vault_path, title)
            .map_err(|e| tantivy::TantivyError::IoError(std::sync::Arc::new(e)))?;

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
    use std::fs;

    const TEST_VAULT: &str = "test_notes";
    const TEST_NOTE: &str = "test_note";

    fn setup() {
        cleanup();
        fs::create_dir_all(TEST_VAULT).unwrap();
    }

    fn cleanup() {
        fs::read_dir(TEST_VAULT).ok().and_then(|entries| {
            for entry in entries {
                fs::remove_file(entry.unwrap().path()).ok();
            }
            fs::remove_dir(TEST_VAULT).ok()
        });
        fs::remove_dir_all(TEST_VAULT).ok();
    }

    #[test]
    fn test_create_and_read_note() {
        setup();

        let file_name = format!("{}_create", TEST_NOTE);
        let content = "This is a test note.";
        let expected_clean_content = "This is a test note.";

        let result = Note::create_note(TEST_VAULT, &file_name, content);
        assert!(result.is_ok(), "❌ Note creation should succeed");

        let read_result = Note::read_note(TEST_VAULT, &file_name);
        assert!(read_result.is_ok(), "❌ Reading the note should succeed");

        let read_content = read_result.unwrap().trim().to_string();
        assert_eq!(read_content, expected_clean_content, "❌ Read content should match sanitized input");

    }

    #[test]
    fn test_extract_links() {
        setup();

        let file_name = format!("{}_extract", TEST_NOTE);
        let md_content = "This note links to [[AnotherNote]] and [[TestNote]].";
        let result = Note::create_note(TEST_VAULT, &file_name, md_content);
        assert!(result.is_ok(), "❌ Creating note with links should succeed");

        let content = Note::read_note(TEST_VAULT, &file_name).unwrap();
        let links = Note::extract_links(&content);
        assert!(!links.is_empty(), "❌ Extracting links should succeed");

        let extracted_links = links;
        println!("🔗 Extracted Links: {:?}", extracted_links);
        assert_eq!(extracted_links, vec!["AnotherNote", "TestNote"], "❌ Extracted links should match expected values");

    }

    #[test]
    fn test_render_html() {
        setup();

        let file_name = format!("{}_render", TEST_NOTE);
        let md_content = "# Title\n This is **bold**.";
        let result = Note::create_note(TEST_VAULT, &file_name, md_content);
        assert!(result.is_ok(), "❌ Creating markdown note should succeed");

        let html_result = Note::render_html(TEST_VAULT, &file_name);
        assert!(html_result.is_ok(), "❌ Rendering markdown to HTML should succeed");

        let html_content = html_result.unwrap();
        println!("🖥️ Rendered HTML: {}", html_content);

        assert!(html_content.contains("<h1>Title</h1>"), "❌ Markdown header should be converted");
        assert!(html_content.contains("<strong>bold</strong>"), "❌ Bold text should be formatted");

    }

    #[test]
    fn test_delete_note() {
        setup();

        let file_name = format!("{}_delete", TEST_NOTE);
        let result = Note::create_note(TEST_VAULT, &file_name, "Test content");
        assert!(result.is_ok(), "❌ Creating note should succeed");

        let delete_result = Note::delete_note(TEST_VAULT, &file_name);
        assert!(delete_result.is_ok(), "❌ Deleting note should succeed");

        let read_result = Note::read_note(TEST_VAULT, &file_name);
        assert!(read_result.is_err(), "❌ Reading a deleted note should fail");

    }
}