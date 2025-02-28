// Full-text search
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, TEXT, STORED};
use tantivy::{Index, IndexWriter, DocAddress};

pub struct NoteSearch {
    schema: Schema,
    index: Index,
}

impl NoteSearch {
    pub fn new() -> Self {
        todo!("Initialize a new NoteSearch instance");
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        todo!("Search for notes matching the query");
    }
}