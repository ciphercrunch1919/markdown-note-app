import { invoke } from "@tauri-apps/api/core";

// UI Elements
const createNoteButton = document.querySelector("#create-note") as HTMLButtonElement;
const deleteNoteButton = document.querySelector("#delete-note") as HTMLButtonElement;
const noteTitleInput = document.querySelector("#note-title") as HTMLInputElement;
const noteContentInput = document.querySelector("#note-content") as HTMLTextAreaElement;
const vaultSelect = document.querySelector("#vault-select") as HTMLSelectElement;
const noteList = document.querySelector("#note-list") as HTMLUListElement;

// Current state
let currentVault: string | null = null;
let currentNote: string | null = null;

// Create a new note
async function createNote() {
    if (!currentVault || !noteTitleInput || !noteContentInput) return;

    const title = noteTitleInput.value;
    const content = noteContentInput.value;

    if (!title || !content) {
        alert("Note title and content are required.");
        return;
    }

    try {
        await invoke("create_note", { vault: currentVault, title, content });
        await loadNotes(); // Refresh the note list
    } catch (error) {
        console.error("Error creating note:", error);
    }
    }

    // Delete the current note
    async function deleteNote() {
    if (!currentVault || !currentNote) return;

    try {
        await invoke("delete_note", { vault: currentVault, title: currentNote });
        noteTitleInput.value = "";
        noteContentInput.value = "";
        await loadNotes(); // Refresh the note list
    } catch (error) {
        console.error("Error deleting note:", error);
    }
    }

    // Load notes for the selected vault
    async function loadNotes() {
    if (!currentVault) return;

    try {
        const notes: string[] = await invoke("list_notes", { vault: currentVault });
        noteList.innerHTML = notes
        .map((note) => `<li onclick="selectNote('${note}')">${note}</li>`)
        .join("");
    } catch (error) {
        console.error("Error loading notes:", error);
    }
    }

    // Select a note to view/edit
    async function selectNote(note: string) {
    currentNote = note;
    noteTitleInput.value = note;
    const content: string = await invoke("read_note", { vault: currentVault, title: note });
    noteContentInput.value = content;
    }

    // Load available vaults
    async function loadVaults() {
    try {
        const vaults: string[] = await invoke("list_vaults");
        vaultSelect.innerHTML = vaults
        .map((vault) => `<option value="${vault}">${vault}</option>`)
        .join("");
        if (vaults.length > 0) {
        currentVault = vaults[0];
        await loadNotes();
        }
    } catch (error) {
        console.error("Error loading vaults:", error);
    }
    }

    // Switch to a different vault
    async function switchVault(vault: string) {
    currentVault = vault;
    currentNote = null;
    noteTitleInput.value = "";
    noteContentInput.value = "";
    await loadNotes();
    }

    // Initialize the app
    window.addEventListener("DOMContentLoaded", () => {
    createNoteButton?.addEventListener("click", createNote);
    deleteNoteButton?.addEventListener("click", deleteNote);
    vaultSelect?.addEventListener("change", (e) => {
        const target = e.target as HTMLSelectElement;
        switchVault(target.value);
    });

    // Load initial data
    loadVaults();
});