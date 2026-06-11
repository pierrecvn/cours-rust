# Design — Cours complet de Rust (projets guidés)

Date : 2026-06-11
Public : développeur junior connaissant Java et TypeScript.
Objectif : apprendre Rust de zéro jusqu'à la création d'applications réelles (CLI, web, desktop, WASM, jeu).

## Format retenu

Apprentissage par **projets guidés** : 8 modules, un projet concret par module, difficulté croissante. La théorie est introduite au moment où le projet en a besoin, avec des comparaisons systématiques avec Java et TypeScript.

## Structure du dépôt

```
cours-rust/
├── README.md                  ← présentation du cours, parcours, mode d'emploi
├── GLOSSAIRE.md               ← index des concepts → module où c'est expliqué
├── module-01-fondations/
│   ├── COURS.md               ← théorie du module
│   ├── PROJET.md              ← projet guidé, étapes numérotées
│   ├── projet/                ← squelette Cargo dans lequel l'étudiant code
│   ├── solution/              ← corrigé idiomatique commenté
│   └── DEFIS.md               ← 2 à 4 extensions non guidées
├── module-02-structurer/
├── module-03-cli-pro/
├── module-04-concurrence/
├── module-05-backend-web/
├── module-06-desktop-tauri/
├── module-07-wasm/
└── module-08-jeu-bevy/
```

## Anatomie d'un module

- **COURS.md** : théorie nécessaire au projet. Chaque concept nouveau suit le schéma : définition → comparaison Java/TS → exemple minimal compilable → utilisation dans le projet. Les erreurs classiques du compilateur (surtout le borrow checker) sont montrées et expliquées comme du contenu pédagogique.
- **PROJET.md** : le projet découpé en étapes numérotées. Chaque étape dit quoi faire et pourquoi, donne des indices, mais laisse l'étudiant écrire le code. Points de contrôle : `cargo test`, `cargo run` avec comportement attendu.
- **projet/** : projet Cargo pré-initialisé (squelette avec TODOs, tests fournis quand c'est pertinent).
- **solution/** : corrigé complet, idiomatique, commenté.
- **DEFIS.md** : extensions non guidées pour consolider.

## Les 8 modules

| # | Module | Concepts clés | Projet |
|---|--------|--------------|--------|
| 1 | Fondations | rustup, Cargo, variables/mutabilité, types de base, fonctions, contrôle de flux, ownership & borrowing (introduction), `String` vs `&str` | Jeu du nombre mystère + convertisseur d'unités |
| 2 | Structurer son code | structs, enums, `Option`/`Result`, `match`, modules, `Vec`, `HashMap`, dérive de traits (`Debug`, `Clone`) | Gestionnaire de tâches CLI avec persistance fichier (JSON) |
| 3 | CLI professionnel | traits, génériques, itérateurs & closures, crates externes (clap, serde, anyhow/thiserror), tests unitaires et d'intégration, organisation lib/bin | mini-grep avec options, regex et fichier de config |
| 4 | Concurrence & lifetimes | threads, channels (mpsc), `Arc`/`Mutex`, lifetimes en profondeur, smart pointers (`Box`, `Rc`, `RefCell`), `Send`/`Sync` | Scanner de fichiers parallèle (statistiques sur une arborescence) |
| 5 | Backend web | async/await, tokio, Axum, SQLx + SQLite, REST, JSON, extracteurs, gestion d'erreurs HTTP, state partagé | API REST de gestion de notes |
| 6 | App desktop | Tauri 2, commandes (invoke), communication frontend TS ↔ backend Rust, state, packaging | App desktop de notes branchée sur l'API du module 5 |
| 7 | WASM | wasm-bindgen, wasm-pack, compilation vers `wasm32-unknown-unknown`, interop JS/TS, performances | Module de traitement d'image (filtres) appelé depuis une page web |
| 8 | Jeu 2D | Bevy, architecture ECS (entités, composants, systèmes, ressources), game loop, input, assets, états de jeu | Snake |

## Contrat d'API entre les modules 5 et 6

Le module 6 consomme l'API du module 5. Contrat fixe (les deux modules le respectent) :

- Base : `http://localhost:3000`
- `GET /notes` → `200`, liste de notes
- `POST /notes` (body `{"title": string, "content": string}`) → `201`, note créée
- `GET /notes/{id}` → `200` ou `404`
- `PUT /notes/{id}` (body `{"title": string, "content": string}`) → `200` ou `404`
- `DELETE /notes/{id}` → `204` ou `404`
- Forme d'une note : `{"id": i64, "title": string, "content": string, "created_at": string (ISO 8601)}`

## Conventions pédagogiques

- Tout le texte en français (orthographe et accents corrects), code et identifiants en anglais.
- Comparaisons Java/TS systématiques pour chaque concept nouveau.
- Rust stable (1.96, édition 2024), crates récentes et maintenues.
- Le code des `projet/` (squelettes + tests fournis) et des `solution/` doit compiler ; les solutions doivent passer leurs tests.
- Les dossiers `target/` ne sont jamais commités (`.gitignore`).

## Environnement de la machine

- Rust 1.96 installé via rustup, cible `wasm32-unknown-unknown` ajoutée.
- Node 24 + npm 11 disponibles (module 6).
- Dépendances système Bevy présentes (alsa, udev). Manquantes pour Tauri : `libwebkit2gtk-4.1-dev`, `librsvg2-dev` — la commande d'installation est documentée dans le module 6.
- Disque limité (~15 Go libres) : nettoyer les `target/` après vérification (`cargo clean`).

## Suivi

Le cours complet est généré en une fois (demande explicite de l'utilisateur). Le suivi de progression et les retours sur le code de l'étudiant se font ensuite à la demande, module par module.
