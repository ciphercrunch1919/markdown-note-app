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
        // Disable the base path for tests
        file_operations::set_base_path(None);

        // Clean up any existing test directory
        cleanup();

        // Create the test directory
        file_operations::create_directory(TEST_BASE_PATH).unwrap();

        // Create the Vault instance and store it in the static variable
        let vault = Vault::create_vault(TEST_VAULT, TEST_BASE_PATH).unwrap();
        VAULT.lock().unwrap().set(vault).unwrap();
    }

    fn cleanup() {
        if Path::new(TEST_BASE_PATH).exists() {
            println!("🧹 Cleaning up test directory: {}", TEST_BASE_PATH);
            file_operations::delete_directory(TEST_BASE_PATH).unwrap();
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

        cleanup();
    }

    #[test]
    fn test_index_note_in_vault() {
        setup();

        // Get the Vault instance from the static variable
        let binding = VAULT.lock().unwrap();
        let vault = binding.get().unwrap();
        let _vault_path = &vault.path;

        // Create a note
        let content = "This is a test note.";
        let note = Note::new("TestNote", content);
        let result = Note::create_note(&note, vault);
        assert!(result.is_ok(), "❌ Creating note should succeed");

        // Index the note
        let indexed = vault.index_note("TestNote", content);
        assert!(indexed.is_ok(), "❌ Indexing note should succeed");

        // Verify the note is indexed
        let schema = vault.index.schema();
        let title_field = schema.get_field("title").unwrap();
        let reader = vault.index.reader().unwrap();
        let searcher = reader.searcher();

        let term = TermQuery::new(
            tantivy::Term::from_field_text(title_field, "TestNote"),
            tantivy::schema::IndexRecordOption::Basic,
        );

        let top_docs = searcher.search(&term, &TopDocs::with_limit(1)).unwrap();
        assert_eq!(top_docs.len(), 1, "❌ Note should be indexed");

        cleanup();
    }

    #[test]
    fn test_delete_note_index() {
        setup();

        // Get the Vault instance from the static variable
        let binding = VAULT.lock().unwrap();
        let vault = binding.get().unwrap();
        let _vault_path = &vault.path;

        // Create and index a note
        let title = "TestNote";
        let content = "This is a test note.";
        let note = Note::new(title, content);
        Note::create_note(&note, vault).unwrap();
        vault.index_note(title, content).unwrap();

        // Verify the note is indexed
        let schema = vault.index.schema();
        let title_field = schema.get_field("title").unwrap();
        let reader = vault.index.reader().unwrap();
        let searcher = reader.searcher();

        let term = TermQuery::new(
            tantivy::Term::from_field_text(title_field, title),
            tantivy::schema::IndexRecordOption::Basic,
        );

        let top_docs = searcher.search(&term, &TopDocs::with_limit(1)).unwrap();
        assert_eq!(top_docs.len(), 1, "❌ Note should be indexed");

        // Delete the note from the index
        vault.delete_note_index(title).unwrap();

        // Verify the note is no longer indexed
        let top_docs_after_delete = searcher.search(&term, &TopDocs::with_limit(1)).unwrap();
        assert_eq!(top_docs_after_delete.len(), 0, "❌ Note should be deleted from the index");

        cleanup();
    }
}