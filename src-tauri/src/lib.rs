// use tauri::Manager;
use serde::{Serialize, Deserialize};

// mod feature;
mod storage;
mod utils;

use storage::{note, vault};
// use feature::{graph, search, metadata};

// Struct for metadata
#[derive(Serialize, Deserialize)]
struct NoteMetadata {
    tags: Vec<String>,
    backlinks: Vec<String>,
    created_at: String,
    updated_at: String,
}

// Vault commands
#[tauri::command]
fn create_vault(vault: String, base_path: String) -> Result<(), String> {
    vault::Vault::create_vault(&vault, &base_path)
        .map(|_vault| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_vaults(base_path: String) -> Result<Vec<String>, String> {
    vault::Vault::list_vaults(&base_path).map_err(|e| e.to_string())
}

// Note commands
#[tauri::command]
fn create_note(vault: String, title: String, content: String) -> Result<(), String> {
    note::Note::create_note(&vault, &title, &content).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_note(vault: String, title: String) -> Result<String, String> {
    note::Note::read_note(&vault, &title).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_note(vault: String, title: String) -> Result<(), String> {
    note::Note::delete_note(&vault, &title).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_notes(vault: String) -> Result<Vec<String>, String> {
    note::Note::list_notes(&vault).map_err(|e| e.to_string())
}

// Search commands (Commented out for now)
/*
#[tauri::command]
fn search_notes(vault: String, query: String) -> Result<Vec<String>, String> {
    search::search_notes(&vault, &query).map_err(|e| e.to_string())
}
*/

// Graph visualization (Commented out for now)
/*
#[tauri::command]
fn render_graph(vault: String) -> Result<String, String> {
    graph::render_graph(&vault).map_err(|e| e.to_string())
}
*/

// Metadata commands (Commented out for now)
/*
#[tauri::command]
fn get_note_metadata(vault: String, title: String) -> Result<NoteMetadata, String> {
    metadata::get_note_metadata(&vault, &title).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_note_metadata(vault: String, title: String, metadata: NoteMetadata) -> Result<(), String> {
    metadata::update_note_metadata(&vault, &title, metadata).map_err(|e| e.to_string())
}
*/

// Tauri entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_vault,
            list_vaults,
            create_note,
            read_note,
            delete_note,
            list_notes,
            // search_notes,  // Commented out for now
            // render_graph,  // Commented out for now
            // get_note_metadata,  // Commented out for now
            // update_note_metadata // Commented out for now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}