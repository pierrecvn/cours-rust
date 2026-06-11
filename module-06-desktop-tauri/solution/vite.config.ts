import { defineConfig } from "vite";

// Configuration minimale pour Tauri :
// - port fixe 1420 (celui que `devUrl` attend dans tauri.conf.json) ;
// - `strictPort` : échouer si le port est pris, plutôt que d'en changer
//   silencieusement (la webview pointerait alors dans le vide) ;
// - `clearScreen: false` : ne pas effacer les logs de Tauri/cargo.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
