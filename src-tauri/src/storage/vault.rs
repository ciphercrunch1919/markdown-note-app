use serde::{Serialize, Deserialize};
use tantivy::{doc, schema::{Schema, STORED, TEXT}, Index, Result as TantivyResult};
use std::path::Path;
use std::sync::Arc;

use crate::utils::{file_operations, string_utils};

#[derive(Debug, Serialize)]
pub struct Vault {
    pub name: String,
    pub path: String,
    #[serde(skip)]
    index: Index, // Exclude `index` from serialization
}

impl<'de> Deserialize<'de> for Vault {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct VaultData {
            name: String,
            path: String,
        }

        let data = VaultData::deserialize(deserializer)?;
        let index = Index::open_in_dir(&data.path).map_err(serde::de::Error::custom)?;

        Ok(Vault {
            name: data.name,
            path: data.path,
            index,
        })
    }
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

        // Clean up any existing index
        let index_path = format!("{}/{}", vault_path, "tantivy");
        if Path::new(&index_path).exists() {
            println!("🧹 Cleaning up existing index directory: {}", index_path);
            file_operations::delete_directory(&index_path).map_err(|e| tantivy::TantivyError::IoError(Arc::new(e)))?;
        }

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
        schema_builder.add_text_field("title", TEXT | STORED); // Ensure "title" is indexed and stored
        schema_builder.add_text_field("content", TEXT); // "content" is indexed by default
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

    // Deletes indexed note from Tantivy.
    pub fn delete_note_index(&self, title: &str) -> TantivyResult<()> {
        let safe_title = string_utils::sanitize_filename(title);
        let mut index_writer: tantivy::IndexWriter = self.index.writer(50_000_000)?;
        let term = tantivy::Term::from_field_text(self.index.schema().get_field("title").unwrap(), &safe_title);
        index_writer.delete_term(term);
        index_writer.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tantivy::{collector::TopDocs, query::TermQuery};

    use crate::storage::note::Note;

    use super::*;
    use std::sync::{Mutex, OnceLock};

    const TEST_BASE_PATH: &str = "test_vaults";
    const TEST_VAULT: &str = "TestVault";

    static VAULT: Mutex<OnceLock<Vault>> = Mutex::new(OnceLock::new());

    fn setup() {
        cleanup();
        let base_path = Path::new(TEST_BASE_PATH);
        let _ = file_operations::create_directory(base_path.to_str().unwrap());
    }

    fn cleanup() {
        if Path::new(TEST_BASE_PATH).exists() {
            fs::read_dir(TEST_BASE_PATH).unwrap()
                .for_each(|entry| {
                    let path = entry.unwrap().path();
                    fs::remove_dir_all(path).ok();
                });
            fs::remove_dir_all(TEST_BASE_PATH).ok();
        }
    }

    #[test]
    fn test_create_vault() {
        setup();

        // Get the Vault instance from the static variable
        let binding = VAULT.lock().unwrap();
        let vault = binding.get().unwrap();

        // Verify the vault name and path
        assert_eq!(vault.name, TEST_VAULT);
        let expected_vault_path = format!("{}/{}", TEST_BASE_PATH, TEST_VAULT);
        assert_eq!(vault.path, expected_vault_path, "❌ Vault path should match expected path");

        // Verify the vault directory exists
        assert!(Path::new(&vault.path).exists(), "❌ Vault directory should exist");

        // Verify the Tantivy index exists
        let index_path = format!("{}/{}", vault.path, "tantivy");
        assert!(Path::new(&index_path).exists(), "❌ Tantivy index directory should exist");

        cleanup(); // Ensure cleanup is called at the end of the test
    }

    #[test]
    fn test_delete_vault() {
        setup();

        let binding = VAULT.lock().unwrap();
        let vault = binding.get().unwrap();
        let vault_path = &vault.path;

        // Ensure the vault directory exists
        assert!(Path::new(&vault_path).exists());

        // Delete the vault
        let delete_result = vault.delete_vault();
        assert!(delete_result.is_ok(), "❌ Deleting vault should succeed");

        // Ensure the vault directory no longer exists
        assert!(!Path::new(&vault_path).exists());

    }

    #[test]
    fn test_index_note_in_vault() {
        setup();
    
        let vault_path = "test_vaults/TestVault";
    
        let note_path = format!("{}/TestNote.md", vault_path);
        let content = "This is a test note.";
        let note = Note::new("TestNote", content);
        let result = Note::create_note(&note, vault);
        assert!(result.is_ok(), "❌ Creating note should succeed");

        // Index the note
        let indexed = vault.index_note("TestNote", content);
        assert!(indexed.is_ok(), "❌ Indexing note should succeed");
    }    
}