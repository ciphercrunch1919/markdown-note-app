import { invoke } from "@tauri-apps/api/core";

// UI Elements
const createNoteButton = document.querySelector("#create-note") as HTMLButtonElement;
const deleteNoteButton = document.querySelector("#delete-note") as HTMLButtonElement;
const noteTitleInput = document.querySelector("#note-title") as HTMLInputElement;
const noteContentInput = document.querySelector("#note-content") as HTMLTextAreaElement;
const vaultSelect = document.querySelector("#vault-select") as HTMLSelectElement;
const noteList = document.querySelector("#note-list") as HTMLUListElement;
const previewButton = document.querySelector("#preview-button") as HTMLButtonElement;
const editorTab = document.querySelector("#editor") as HTMLDivElement;
const previewTab = document.querySelector("#preview") as HTMLDivElement;
const addVaultButton = document.querySelector("#add-vault") as HTMLButtonElement;

// Current state
let currentVault: string | null = null;
let currentNote: string | null = null;
let isPreviewMode = false;

// Create a default vault if it doesn't exist
async function initializeDefaultVault() {
    const vaults: string[] = await invoke("list_vaults");
    if (vaults.length === 0) {
        // Create a default vault
        await invoke("create_vault", { vault: "DefaultVault", basePath: "./vaults" });
        console.log("Default vault created: DefaultVault");
    }
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

// Add a new vault
async function addVault() {
    const vaultName = prompt("Enter a name for the new vault:");
    if (!vaultName) return;

    try {
        await invoke("create_vault", { vault: vaultName, basePath: "./vaults" });
        console.log(`Vault created: ${vaultName}`);
        await loadVaults(); // Refresh the vault list
    } catch (error) {
        console.error("Error creating vault:", error);
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

// Create a new note
async function createNote() {
    if (!currentVault || !noteTitleInput || !noteContentInput) return;

    const title = noteTitleInput.value.trim();
    const content = noteContentInput.value;

    if (!content) {
        alert("Note content is required.");
        return;
    }

    try {
        // Create the note
        await invoke("create_note", { vault: currentVault, title, content });

        // Use indexNote instead of direct invoke
        await indexNote(currentVault, title, content);

        await loadNotes();
    } catch (error) {
        console.error("Error creating note:", error);
    }
}

// Delete the current note
async function deleteNote() {
    if (!currentVault || !currentNote) return;

    try {
        // Delete the note file
        await invoke("delete_note", { vault: currentVault, title: currentNote });

        // Delete the note from the index
        await invoke("delete_note_index", { vault: currentVault, title: currentNote });

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
            .map((note) => `<li data-note="${note}">${note}</li>`)
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

    // Extract plain text for search or preview
    if (currentVault) {
        await extractPlainText(currentVault, note);
    }

    // Extract links from the note
    if (currentVault) {
        await getNoteLinks(currentVault, note);
    }
}

// Function to extract plain text from a note
async function extractPlainText(vault: string, title: string) {
    try {
        const content = await invoke("read_note", { vault, title });
        const plainText: string = await invoke("extract_plain_text", { content });
        console.log("Extracted plain text:", plainText);
        // Display the plain text in the preview
        const plainTextPreview = document.getElementById("plain-text-preview");
        if (plainTextPreview) {
            plainTextPreview.innerText = plainText;
        }
    } catch (error) {
        console.error("Error extracting plain text:", error);
    }
}

// Function to extract links from a note
async function getNoteLinks(vault: string, title: string) {
    try {
        const links: string[] = await invoke("extract_links", { vault, title });
        console.log("Extracted links:", links);
        // Optionally process the links (e.g., for graph visualization)
        processLinks(links);
    } catch (error) {
        console.error("Error extracting links:", error);
    }
}

// Example function to process links (e.g., for graph visualization)
function processLinks(links: string[]) {
    console.log("Processing links:", links);
    // Add your logic here (e.g., update a graph, store links in state, etc.)
}

// Function to delete a vault
async function deleteVault(vault: string) {
    try {
        await invoke("delete_vault", { vault });
        console.log("Vault deleted successfully");
        await loadVaults(); // Refresh the vault list
    } catch (error) {
        console.error("Error deleting vault:", error);
    }
}

// Add event listener for the Delete Vault button
const deleteVaultButton = document.querySelector("#delete-vault") as HTMLButtonElement;
deleteVaultButton?.addEventListener("click", async () => {
    if (currentVault) {
        const confirmDelete = confirm(`Are you sure you want to delete the vault "${currentVault}"?`);
        if (confirmDelete) {
            await deleteVault(currentVault);
            await loadVaults(); // Refresh the vault list
        }
    }
});

// Function to index a note
async function indexNote(vault: string, title: string, content: string) {
    try {
        await invoke("index_note", { vault, title, content });
        console.log("Note indexed successfully");
    } catch (error) {
        console.error("Error indexing note:", error);
    }
}

// Function to toggle between Markdown and Plain Text views
async function togglePreview() {
    isPreviewMode = !isPreviewMode;

    if (isPreviewMode) {
        // Switch to Markdown preview
        const content = noteContentInput.value;

        // Parse the Markdown content and render it as HTML
        const htmlContent = await parseMarkdownContent(content);

        // Display the rendered HTML in the preview tab
        previewTab.innerHTML = htmlContent;

        // Hide the editor and show the preview
        editorTab.style.display = "none";
        previewTab.style.display = "block";
        previewButton.textContent = "Edit";
    } else {
        // Switch back to Markdown editor
        editorTab.style.display = "block";
        previewTab.style.display = "none";
        previewButton.textContent = "Preview";
    }
}

// Function to parse Markdown content and return HTML
async function parseMarkdownContent(content: string): Promise<string> {
    try {
        // Use the Tauri command to parse the Markdown content
        const htmlContent: string = await invoke("parse_markdown_content", { content });
        return htmlContent;
    } catch (error) {
        console.error("Error parsing Markdown content:", error);
        return "<p>Error rendering Markdown.</p>";
    }
}

// Add event listener for the Preview button
previewButton?.addEventListener("click", togglePreview);

// Add event listener for the Add Vault button
addVaultButton?.addEventListener("click", addVault);

// Initialize the app
window.addEventListener("DOMContentLoaded", async () => {
    // Initialize the default vault
    await initializeDefaultVault();

    // Load vaults and notes
    await loadVaults();

    // Add event listeners
    createNoteButton?.addEventListener("click", createNote);
    deleteNoteButton?.addEventListener("click", deleteNote);
    vaultSelect?.addEventListener("change", (e) => {
        const target = e.target as HTMLSelectElement;
        switchVault(target.value);
    });

    // Add event listener for note list clicks
    noteList?.addEventListener("click", (event) => {
        const target = event.target as HTMLLIElement;
        if (target.tagName === "LI") {
            const note = target.getAttribute("data-note");
            if (note) {
                selectNote(note);
            }
        }
    });
});