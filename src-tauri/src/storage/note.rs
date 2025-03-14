use serde::{Serialize, Deserialize};
use std::path::Path;
use std::io::{self, Error, ErrorKind};
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

    pub fn generate_file_name(content: &str) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        let file_name = if words.len() >= 3 {
            format!("{}-{}-{}", words[0], words[1], words[2])
        } else {
            content.to_string()
        };
        string_utils::sanitize_filename(&file_name)
    }

    pub fn create_note(&self, vault: &mut Vault) -> io::Result<()> {
        let _title = if self.title.trim().is_empty() {
            let id = nanoid!();
            format!("untitled_{}", id)
        } else {
            self.title.clone()
        };

        let file_name = Self::generate_file_name(&self.content);
        let clean_content = string_utils::normalize_whitespace(&self.content);

        // Use file_operations::create_directory instead of std::fs::create_dir_all
        file_operations::create_directory(&vault.path)?;

        let note_path = format!("{}/{}.md", vault.path, file_name);
        // Use file_operations::write_to_file instead of std::fs::write
        file_operations::write_to_file(&note_path, &clean_content)?;

        if !Path::new(&note_path).exists() {
            return Err(Error::new(ErrorKind::Other, format!("❌ File was not created: {}", note_path)));
        }

        // Use file_operations::read_from_file instead of std::fs::read_to_string
        let verify_content = file_operations::read_from_file(&note_path)?;
        if verify_content.is_empty() {
            return Err(Error::new(ErrorKind::Other, "❌ File was created but is empty"));
        }

        Ok(())
    }

    pub fn read_note(vault: &Vault, file_name: &str) -> io::Result<String> {
        let safe_file_name = string_utils::sanitize_filename(file_name);
        let note_path = format!("{}/{}.md", vault.path, safe_file_name);

        if !Path::new(&note_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "❌ Note file does not exist"));
        }

        // Use file_operations::read_from_file instead of std::fs::read_to_string
        file_operations::read_from_file(&note_path)
    }

    pub fn delete_note(&self, vault: &mut Vault) -> io::Result<()> {
        let file_name = Self::generate_file_name(&self.content);
        let note_path = format!("{}/{}.md", vault.path, file_name);

        if Path::new(&note_path).exists() {
            // Use file_operations::delete_file instead of std::fs::remove_file
            file_operations::delete_file(&note_path)?;
        } else {
            return Err(Error::new(ErrorKind::NotFound, "❌ Note file does not exist"));
        }

        Ok(())
    }

    pub fn list_notes(vault: &Vault) -> io::Result<Vec<String>> {
        // Use file_operations::read_dir (if implemented) or keep using std::fs::read_dir
        let paths = std::fs::read_dir(&vault.path)?;
        Ok(paths
            .filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().ok()))
            .flatten()
            .filter(|name| name.ends_with(".md"))
            .map(|name| name.trim_end_matches(".md").to_string())
            .collect())
    }

    pub fn render_html(&self, vault: &Vault) -> Result<String, String> {
        let file_name = Self::generate_file_name(&self.content);
        let content = Self::read_note(vault, &file_name).map_err(|e| e.to_string())?;
        Ok(markdown::render_markdown(&content))
    }

    #[allow(dead_code)]
    pub fn rename_note(&mut self, vault: &mut Vault, new_title: &str) -> io::Result<()> {
        let old_file_name = Self::generate_file_name(&self.content);
        let old_file_path = format!("{}/{}.md", vault.path, old_file_name);

        self.title = new_title.to_string();

        let new_file_name = Self::generate_file_name(&self.content);
        let new_file_path = format!("{}/{}.md", vault.path, new_file_name);

        // Use file_operations::rename_file instead of std::fs::rename
        file_operations::rename_file(&old_file_path, &new_file_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::vault::Vault;

    #[test]
    fn test_create_note() {
        let vault_name = format!("test_vault_{}", nanoid!());
        let mut vault = Vault::create_vault(&vault_name).unwrap();
        let note = Note::new("Test Note", "This is a test note content.");
        assert!(note.create_note(&mut vault).is_ok());

        // Cleanup
        vault.delete_vault().expect("Failed to delete vault");
    }

    #[test]
    fn test_read_note() {
        let vault_name = format!("test_vault_{}", nanoid!());
        let mut vault = Vault::create_vault(&vault_name).unwrap();

        let title = "Test Note";
        let content = "This is a test note content.";

        let note = Note::new(title, content);
        note.create_note(&mut vault).unwrap();

        let file_name = Note::generate_file_name(content);
        let note_path = format!("{}/{}.md", vault.path, file_name);

        assert!(Path::new(&note_path).exists(), "❌ Note file does not exist");

        let read_content = Note::read_note(&vault, &file_name).unwrap();
        assert_eq!(read_content, content);

        // Cleanup
        vault.delete_vault().expect("Failed to delete vault");
    }

    #[test]
    fn test_delete_note() {
        let vault_name = format!("test_vault_{}", nanoid!());
        let mut vault = Vault::create_vault(&vault_name).unwrap();
        let note = Note::new("Test Note", "This is a test note content.");
        note.create_note(&mut vault).unwrap();
        assert!(note.delete_note(&mut vault).is_ok());

        // Cleanup
        vault.delete_vault().expect("Failed to delete vault");
    }
}