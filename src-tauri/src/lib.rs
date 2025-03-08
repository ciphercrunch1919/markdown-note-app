use tauri::Manager;
use serde::{Serialize, Deserialize};

mod storage;
mod utils;

use storage::{note::{self, Note}, vault::{self, Vault}};
use utils::markdown;

#[derive(Serialize, Deserialize)]
struct NoteMetadata {
    tags: Vec<String>,
    backlinks: Vec<String>,
    created_at: String,
    updated_at: String,
}

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

#[tauri::command]
fn create_note(vault: Vault, note: Note) -> Result<(), String> {
    note.create_note(&vault).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_note(vault: Vault, title: String) -> Result<Note, String> {
    let content = note::Note::read_note(&vault, &title).map_err(|e| e.to_string())?;
    Ok(Note { title, content })
}

#[tauri::command]
fn delete_note(vault: Vault, note: Note) -> Result<(), String> {
    note.delete_note(&vault).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_notes(vault: Vault) -> Result<Vec<String>, String> {
    Note::list_notes(&vault).map_err(|e| e.to_string())
}

#[tauri::command]
fn render_html(vault: Vault, note: Note) -> Result<String, String> {
    note.render_html(&vault).map_err(|e| e.to_string())
}

#[tauri::command]
fn extract_links(vault_path: String, title: String) -> Result<Vec<String>, String> {
    let vault = Vault::create_vault(&vault_path, "path/to/base").map_err(|e| e.to_string())?;
    let content = note::Note::read_note(&vault, &title).map_err(|e| e.to_string())?;
    Ok(markdown::extract_links(&content))
}

#[tauri::command]
fn extract_plain_text(content: String) -> Result<String, String> {
    Ok(markdown::extract_plain_text(&content))
}

#[tauri::command]
fn delete_vault(vault: String) -> Result<(), String> {
    let vault = Vault::create_vault(&vault, "path/to/base").map_err(|e| e.to_string())?;
    vault.delete_vault().map_err(|e| e.to_string())
}

#[tauri::command]
fn index_note(vault: Vault, title: String, content: String) -> Result<(), String> {
    vault.index_note(&title, &content).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_note_index(vault: Vault, title: String) -> Result<(), String> {
    vault.delete_note_index(&title).map_err(|e| e.to_string())
}

#[tauri::command]
fn parse_markdown_content(content: String) -> Result<String, String> {
    Ok(markdown::render_markdown(&content))
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Use the app handle to manage the application state
            let app_handle = app.handle();

            // Log the app starting
            println!("App started!");

            // You can also manage windows here
            let main_window = app_handle.get_webview_window("main").unwrap();
            main_window.set_title("Markdown Note App").unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_vault,
            list_vaults,
            create_note,
            read_note,
            delete_note,
            list_notes,
            render_html,
            extract_links,
            extract_plain_text,
            delete_vault,
            index_note,
            delete_note_index,
            parse_markdown_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}