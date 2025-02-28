// Metadata handling
use sled::Db;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NoteMetadata {
    pub tags: Vec<String>,
    pub backlinks: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct MetadataStore {
    db: Db,
}

impl MetadataStore {
    pub fn new(path: &str) -> Self {
        todo!("Initialize a new MetadataStore");
    }

    pub fn get_metadata(&self, note_id: &str) -> Option<NoteMetadata> {
        todo!("Retrieve metadata for a note");
    }

    pub fn update_metadata(&self, note_id: &str, metadata: NoteMetadata) {
        todo!("Update metadata for a note");
    }
}