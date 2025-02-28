use tantivy::{Index, schema::{Schema, TextOptions, TEXT}, doc};
use std::path::Path;
use std::sync::Arc;

use crate::utils::{file_operations, string_utils};

#[derive(Debug)]
pub struct Vault {
    pub name: String,
    pub path: String,
    index: Index,
}

impl PartialEq for Vault {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.path == other.path
    }
}

impl Vault {
    // Creates a new vault and Tantivy index.
    pub fn create_vault(name: &str, base_path: &str) -> tantivy::Result<Self> {
        let safe_name = string_utils::sanitize_filename(name);
        let vault_path = format!("{}/{}", base_path, safe_name);

        // Ensure base directory exists
        if !Path::new(base_path).exists() {
            file_operations::create_directory(base_path).map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;
        }

        // Ensure vault directory exists
        if !Path::new(&vault_path).exists() {
            file_operations::create_directory(&vault_path).map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;
        }

        let schema = Self::create_schema();
        let index = Index::create_in_dir(&vault_path, schema.clone())?;

        Ok(Self {
            name: safe_name,
            path: vault_path,
            index,
        })
    }

    // Lists all vaults in the base directory.
    pub fn list_vaults(base_path: &str) -> std::io::Result<Vec<String>> {
        let paths = std::fs::read_dir(base_path)?;
        Ok(paths
            .filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().ok()).flatten())
            .collect())
    }

    // Creates the Tantivy schema.
    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TextOptions::default().set_stored());
        schema_builder.add_text_field("content", TEXT);
        schema_builder.build()
    }

    // Indexes a note into Tantivy.
    pub fn index_note(&self, title: &str, content: &str) -> tantivy::Result<()> {
        let safe_title = string_utils::sanitize_filename(title);
        let clean_content = string_utils::normalize_whitespace(content);

        let mut index_writer = self.index.writer(50_000_000)?;
        let schema = self.index.schema();
        let title_field = schema.get_field("title").unwrap();
        let content_field = schema.get_field("content").unwrap();

        index_writer.add_document(doc! {
            title_field => safe_title,
            content_field => clean_content,
        })?;
        index_writer.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::note::Note;
    use std::fs;

    const TEST_BASE_PATH: &str = "test_vaults";
    const TEST_VAULT: &str = "TestVault"; 
    const TEST_NOTE: &str = "TestNote"; 

    fn setup() {
        let _ = file_operations::create_directory(TEST_BASE_PATH);
    }

    fn cleanup() {
        if Path::new(TEST_BASE_PATH).exists() {
            let _ = fs::remove_dir_all(TEST_BASE_PATH);
        }
    }

    #[test]
    fn test_create_vault() {
        setup();

        let vault = Vault::create_vault(TEST_VAULT, TEST_BASE_PATH).unwrap();
        
        assert_eq!(vault.name, TEST_VAULT);
        assert!(Path::new(&vault.path).exists());

        cleanup();
    }

    #[test]
    fn test_index_note_in_vault() {
        setup();
    
        let vault_path = "test_vaults/TestVault";
        fs::create_dir_all(vault_path).unwrap(); // Ensure vault exists
    
        let note_path = format!("{}/TestNote.md", vault_path);
        let content = "This is a test note.";
        let result = Note::create_note(vault_path, "TestNote", content);
        assert!(result.is_ok(), "❌ Creating note should succeed");
    
        let indexed = Note::index_note_in_vault(&vault_path, "TestNote");
        assert!(indexed.is_ok(), "❌ Indexing note should succeed");
    
        cleanup();
    }    
}