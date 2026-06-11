// Frontend de l'application de notes.
//
// Tout passe par `invoke` : chaque commande Rust (src-tauri/src/lib.rs)
// appelle elle-même l'API REST du module 5 avec reqwest. Le frontend ne
// fait AUCUN appel réseau direct.
//
// Le squelette fournit toute la plomberie DOM (récupération des éléments,
// rendu de la liste, bascule création/édition). Votre travail : les trois
// fonctions marquées TODO, qui parlent au Rust via `invoke`.

import { invoke } from "@tauri-apps/api/core";

// Miroir TypeScript de la struct Rust `Note` (src-tauri/src/api.rs).
// Le pont ne vérifie pas cette correspondance à la compilation : c'est
// à nous de garder les deux définitions synchronisées.
interface Note {
  id: number;
  title: string;
  content: string;
  created_at: string; // ISO 8601, formatée à l'affichage
}

// ---------------------------------------------------------------------------
// Récupération des éléments du DOM (index.html) — fourni
// ---------------------------------------------------------------------------
// `querySelector` renvoie `Element | null` : le `!` affirme que l'élément
// existe (il est écrit en dur dans index.html, donc c'est sûr ici).

const refreshButton = document.querySelector<HTMLButtonElement>("#refresh-button")!;
const errorBanner = document.querySelector<HTMLDivElement>("#error-banner")!;
const errorMessage = document.querySelector<HTMLParagraphElement>("#error-message")!;
const retryButton = document.querySelector<HTMLButtonElement>("#retry-button")!;
const form = document.querySelector<HTMLFormElement>("#note-form")!;
const formTitle = document.querySelector<HTMLHeadingElement>("#form-title")!;
const titleInput = document.querySelector<HTMLInputElement>("#title-input")!;
const contentInput = document.querySelector<HTMLTextAreaElement>("#content-input")!;
const submitButton = document.querySelector<HTMLButtonElement>("#submit-button")!;
const cancelButton = document.querySelector<HTMLButtonElement>("#cancel-button")!;
const notesList = document.querySelector<HTMLUListElement>("#notes-list")!;
const emptyState = document.querySelector<HTMLParagraphElement>("#empty-state")!;

// Identifiant de la note en cours d'édition, ou `null` en mode création.
// Le formulaire est partagé entre les deux modes.
let editingId: number | null = null;

// ---------------------------------------------------------------------------
// Affichage des erreurs — fourni
// ---------------------------------------------------------------------------

// Quand une commande Rust renvoie `Err(ApiError)`, la Promise d'`invoke`
// est rejetée avec la version sérialisée de l'erreur — ici une chaîne
// (voir l'impl Serialize dans src-tauri/src/error.rs).
function showError(err: unknown): void {
  errorMessage.textContent = typeof err === "string" ? err : String(err);
  errorBanner.hidden = false;
}

function hideError(): void {
  errorBanner.hidden = true;
}

// ---------------------------------------------------------------------------
// Rendu de la liste — fourni
// ---------------------------------------------------------------------------

// On construit le DOM avec createElement + textContent plutôt qu'avec
// innerHTML : le contenu des notes est une donnée externe, textContent
// l'échappe automatiquement (pas d'injection HTML possible).
function renderNotes(notes: Note[]): void {
  notesList.replaceChildren();
  emptyState.hidden = notes.length > 0;

  for (const note of notes) {
    const item = document.createElement("li");
    item.className = "note-card";

    const title = document.createElement("h3");
    title.textContent = note.title;

    const date = document.createElement("span");
    date.className = "note-date";
    date.textContent = new Date(note.created_at).toLocaleString("fr-FR");

    const content = document.createElement("p");
    content.textContent = note.content;

    const actions = document.createElement("div");
    actions.className = "note-actions";

    const editButton = document.createElement("button");
    editButton.type = "button";
    editButton.textContent = "Modifier";
    editButton.addEventListener("click", () => startEditing(note));

    const deleteButton = document.createElement("button");
    deleteButton.type = "button";
    deleteButton.textContent = "Supprimer";
    deleteButton.addEventListener("click", () => void removeNote(note.id));

    actions.append(editButton, deleteButton);
    item.append(title, date, content, actions);
    notesList.append(item);
  }
}

// ---------------------------------------------------------------------------
// Mode édition / mode création du formulaire — fourni
// ---------------------------------------------------------------------------

function startEditing(note: Note): void {
  editingId = note.id;
  titleInput.value = note.title;
  contentInput.value = note.content;
  formTitle.textContent = `Modifier la note « ${note.title} »`;
  submitButton.textContent = "Enregistrer";
  cancelButton.hidden = false;
  titleInput.focus();
}

function resetForm(): void {
  editingId = null;
  form.reset();
  formTitle.textContent = "Nouvelle note";
  submitButton.textContent = "Créer";
  cancelButton.hidden = true;
}

// ---------------------------------------------------------------------------
// Appels des commandes Rust — À VOUS DE JOUER
// ---------------------------------------------------------------------------

async function refreshNotes(): Promise<void> {
  // TODO (étape 4) :
  // 1. appeler la commande Rust :  await invoke<Note[]>("list_notes")
  // 2. passer le résultat à renderNotes(...) puis appeler hideError()
  // 3. envelopper le tout dans try/catch et, en cas d'erreur, showError(err)
  showError("TODO : implémenter refreshNotes() — étape 4 du PROJET");
}

async function submitForm(): Promise<void> {
  const title = titleInput.value.trim();
  const content = contentInput.value.trim();
  if (title === "" || content === "") {
    return;
  }

  // TODO (étape 5) : si editingId === null, créer la note :
  //   await invoke<Note>("create_note", { title, content })
  // TODO (étape 6) : sinon, mettre à jour la note en cours d'édition :
  //   await invoke<Note>("update_note", { id: editingId, title, content })
  // Dans les deux cas : resetForm() puis await refreshNotes(),
  // et try/catch → showError(err).
  showError("TODO : implémenter submitForm() — étapes 5 et 6 du PROJET");
}

async function removeNote(id: number): Promise<void> {
  // TODO (étape 6) :
  // 1. await invoke<void>("delete_note", { id })
  // 2. si editingId === id, resetForm() (la note éditée vient d'être supprimée)
  // 3. await refreshNotes(), et try/catch → showError(err)
  showError(`TODO : implémenter removeNote(${id}) — étape 6 du PROJET`);
}

// ---------------------------------------------------------------------------
// Branchement des évènements — fourni
// ---------------------------------------------------------------------------

form.addEventListener("submit", (event) => {
  event.preventDefault(); // pas de rechargement de page : on est en SPA
  void submitForm();
});

cancelButton.addEventListener("click", resetForm);
refreshButton.addEventListener("click", () => void refreshNotes());
retryButton.addEventListener("click", () => void refreshNotes());

// Premier chargement : si l'API du module 5 n'est pas démarrée,
// le bandeau d'erreur devra s'afficher avec la marche à suivre.
void refreshNotes();
