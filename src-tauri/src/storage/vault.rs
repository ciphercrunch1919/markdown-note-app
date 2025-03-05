use tantivy::{Index, schema::{Schema, TextOptions, TEXT}, doc, Result as TantivyResult};
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
    pub fn create_vault(name: &str, base_path: &str) -> TantivyResult<Self> {
        let safe_name = string_utils::sanitize_filename(name);
        let vault_path = format!("{}/{}", base_path, safe_name);
    
        // Ensure base directory exists
        if !Path::new(base_path).exists() {
            println!("📂 Creating base directory: {}", base_path);
            file_operations::create_directory(base_path).map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;
        }
    
        // Ensure vault directory exists
        if !Path::new(&vault_path).exists() {
            println!("📂 Creating vault directory: {}", vault_path);
            file_operations::create_directory(&vault_path).map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;
        }
    
        // Verify the directory structure
        println!("🛠️ Debug: Vault directory structure - {}", vault_path);
        let schema = Self::create_schema();
    
        // Create the Tantivy index
        println!("🛠️ Debug: Creating Tantivy index in directory: {}", vault_path);
        let index = Index::create_in_dir(&vault_path, schema.clone())?;
    
        println!("✅ Vault successfully created: {}", vault_path);
        Ok(Self {
            name: safe_name,
            path: vault_path,
            index,
        })
    }

    // Deletes the vault and its contents.
    pub fn delete_vault(&self) -> std::io::Result<()> {
        if Path::new(&self.path).exists() {
            println!("🗑️ Deleting vault directory: {}", self.path);
            file_operations::delete_directory(&self.path)?;
        }
        Ok(())
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
    pub fn index_note(&self, title: &str, content: &str) -> TantivyResult<()> {
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

    const TEST_BASE_PATH: &str = "test_vaults";
    const TEST_VAULT: &str = "TestVault";

    fn setup() {
        // Clean up any existing test directory
        cleanup();
    
        // Create the test directory
        file_operations::create_directory(TEST_BASE_PATH).unwrap();
    }
    
    fn cleanup() {
        if Path::new(TEST_BASE_PATH).exists() {
            file_operations::delete_directory(TEST_BASE_PATH).unwrap();
        }
    }

    #[test]
    fn test_create_vault() {
        setup();

        let vault = Vault::create_vault(TEST_VAULT, TEST_BASE_PATH).unwrap();

        // Verify the vault name and path
        assert_eq!(vault.name, TEST_VAULT);
        let expected_vault_path = format!("{}/{}", TEST_BASE_PATH, TEST_VAULT);
        assert_eq!(vault.path, expected_vault_path, "❌ Vault path should match expected path");

        // Verify the vault directory exists
        assert!(Path::new(&vault.path).exists(), "❌ Vault directory should exist");

        // Verify the Tantivy index exists
        let index_path = format!("{}/{}", vault.path, "tantivy");
        assert!(Path::new(&index_path).exists(), "❌ Tantivy index directory should exist");

        cleanup();
    }

    #[test]
    fn test_delete_vault() {
        setup();

        // Create a vault
        let vault = Vault::create_vault(TEST_VAULT, TEST_BASE_PATH).unwrap();
        let vault_path = vault.path.clone();

        // Ensure the vault directory exists
        assert!(Path::new(&vault_path).exists());

        // Delete the vault
        let delete_result = vault.delete_vault();
        assert!(delete_result.is_ok(), "❌ Deleting vault should succeed");

        // Ensure the vault directory no longer exists
        assert!(!Path::new(&vault_path).exists());

        cleanup();
    }

    #[test]
    fn test_index_note_in_vault() {
        setup();

        // Create the vault
        let vault = Vault::create_vault(TEST_VAULT, TEST_BASE_PATH).unwrap();
        let vault_path = &vault.path;

        // Create a note
        let content = "This is a test note.";
        let result = Note::create_note(vault_path, "TestNote", content);
        assert!(result.is_ok(), "❌ Creating note should succeed");

        // Index the note
        let indexed = Note::index_note_in_vault(vault_path, "TestNote");
        assert!(indexed.is_ok(), "❌ Indexing note should succeed");

        cleanup();
    }
}