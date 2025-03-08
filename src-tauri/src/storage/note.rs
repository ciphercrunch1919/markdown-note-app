use serde::{Serialize, Deserialize};
use std::path::Path;
use std::io::{self, Error, ErrorKind};
use tantivy::Result as TantivyResult;
use std::fs;
use std::sync::Arc;
use nanoid::nanoid;

use crate::utils::{file_operations, string_utils, markdown};
use crate::storage::vault::Vault;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
}

impl Note {
    #[allow(dead_code)]
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
        }
    }

    // Generates a file name from the first three words of the content.
    pub fn generate_file_name(content: &str) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        let file_name = if words.len() >= 3 {
            format!("{}-{}-{}", words[0], words[1], words[2])
        } else {
            content.to_string()
        };
        string_utils::sanitize_filename(&file_name)
    }

    // Creates a new Markdown note.
    pub fn create_note(&self, vault: &Vault) -> io::Result<()> {
        // Generate a default title if the provided title is empty
        let title = if self.title.trim().is_empty() {
            let id = nanoid!(); // Generates a unique ID like "abc123xyz"
            format!("untitled_{}", id)
        } else {
            self.title.clone()
        };

        // Generate the file name from the first three words of the content
        let file_name = Self::generate_file_name(&self.content);
        let clean_content = string_utils::normalize_whitespace(&self.content);

        println!("🛠️ Debug: Title - '{}'", title);
        println!("🛠️ Debug: File name - '{}'", file_name);
        println!("🛠️ Debug: Clean content - '{}'", clean_content);

        // Ensure the vault directory exists
        if !Path::new(&vault.path).exists() {
            println!("📂 Creating vault directory: {}", vault.path);
            file_operations::create_directory(&vault.path)?;
        }

        let note_path = format!("{}/{}.md", vault.path, file_name);
        println!("📝 Creating note at: {}", note_path);

        // Write the note content to the file
        file_operations::write_to_file(&note_path, &clean_content)?;

        // Verify that the file was created
        if !Path::new(&note_path).exists() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("❌ File was not created: {}", note_path),
                )
            );
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
        let _ = self.index_note_in_vault(vault);

        Ok(())
    }

    // Reads a Markdown note as raw content.
    pub fn read_note(vault: &Vault, file_name: &str) -> io::Result<String> {
        let safe_file_name = string_utils::sanitize_filename(file_name);
        let note_path = format!("{}/{}.md", vault.path, safe_file_name);

        println!("📖 Debug: Attempting to read note from {}", note_path);

        if !Path::new(&note_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "❌ Note file does not exist"));
        }

        let content = file_operations::read_from_file(&note_path)?;
        println!("📖 Debug: Read content - '{}'", content);
        Ok(content)
    }

    // Deletes a Markdown note.
    pub fn delete_note(&self, vault: &Vault) -> io::Result<()> {
        let file_name = Self::generate_file_name(&self.content);
        let safe_file_name = string_utils::sanitize_filename(&file_name);
        let note_path = format!("{}/{}.md", vault.path, safe_file_name);

        if Path::new(&note_path).exists() {
            println!("🗑️ Deleting note: {}", note_path);
            file_operations::delete_file(&note_path)?;
        }

        if Path::new(&note_path).exists() {
            println!("❌ File still exists after deletion: {}", note_path);
            return Err(Error::new(ErrorKind::Other, "❌ File was not deleted"));
        }

        // Delete the note from the Tantivy index
        vault.delete_note_index(&safe_file_name)
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

        println!("✅ Note successfully deleted: {}", note_path);
        Ok(())
    }

    // Lists all notes in a vault.
    pub fn list_notes(vault: &Vault) -> io::Result<Vec<String>> {
        let paths = fs::read_dir(&vault.path)?;
        Ok(paths
            .filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().ok()).flatten())
            .filter(|name| name.ends_with(".md"))
            .map(|name| name.trim_end_matches(".md").to_string())
            .collect())
    }

    // Converts a Markdown note into HTML.
    pub fn render_html(&self, vault: &Vault) -> Result<String, String> {
        let file_name = Self::generate_file_name(&self.content);
        let content = Self::read_note(vault, &file_name).map_err(|e| e.to_string())?;
        Ok(markdown::render_markdown(&content))
    }

    // Indexes a note in the vault.
    pub fn index_note_in_vault(&self, vault: &Vault) -> TantivyResult<()> {
        let file_name = Self::generate_file_name(&self.content);
        let content = Self::read_note(vault, &file_name)
            .map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;

        println!("🔍 Debug: Indexing note '{}' in vault '{}'", file_name, vault.name);

        vault.index_note(&file_name, &content)
    }

    #[allow(dead_code)]
    pub fn rename_note(&mut self, vault: &Vault, new_title: &str) -> io::Result<()> {
        // Generate the old file name from the first three words of the content
        let old_file_name = Self::generate_file_name(&self.content);
        let old_file_path = format!("{}/{}.md", vault.path, old_file_name);

        // Update the title
        self.title = new_title.to_string();

        // Generate the new file name from the first three words of the content
        let new_file_name = Self::generate_file_name(&self.content);
        let new_file_path = format!("{}/{}.md", vault.path, new_file_name);

        // Rename the file
        file_operations::rename_file(&old_file_path, &new_file_path)?;

        // Update the index (if applicable)
        vault.delete_note_index(&old_file_name)
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        vault.index_note(&new_file_name, &self.content)
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

        println!("✅ Note successfully renamed to: {}", new_file_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;

    const TEST_VAULT: &str = "test_notes";
    const TEST_NOTE: &str = "test_note";

    static VAULT: Mutex<OnceLock<Vault>> = Mutex::new(OnceLock::new());

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

        // Verify the content
        let read_content = read_result.unwrap().trim().to_string();
        assert_eq!(read_content, expected_clean_content, "❌ Read content should match sanitized input")
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

        // Verify the extracted links
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

        // Verify the rendered HTML
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

    #[test]
    fn test_rename_note() {
        setup();

        let content = "This is a test note.";
        let new_title = "Renamed Note";

        // Create the note
        let mut note = Note {
            title: "Test Note".to_string(),
            content: content.to_string(),
        };
        let binding = VAULT.lock().unwrap();
        let vault = binding.get().unwrap();
        note.create_note(vault).unwrap();

        // Rename the note
        note.rename_note(vault, new_title).unwrap();

        // Verify the new file exists
        let new_file_name = Note::generate_file_name(&content); // Use Note::generate_file_name
        let new_file_path = format!("{}/{}.md", vault.path, new_file_name);
        assert!(Path::new(&new_file_path).exists());

        // Verify the old file no longer exists
        let old_file_name = Note::generate_file_name(&content); // Use Note::generate_file_name
        let old_file_path = format!("{}/{}.md", vault.path, old_file_name);
        assert!(!Path::new(&old_file_path).exists());

        cleanup();
    }
}