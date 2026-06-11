// Point d'entrée de la page : charge le module WASM, branche les boutons.
//
// Ce fichier est fourni : votre travail est côté Rust. Lisez-le quand même,
// chaque appel à la frontière JS ↔ WASM est commenté.
//
// Prérequis : avoir lancé, à la racine du projet Cargo,
//     wasm-pack build --target web --out-dir web/pkg
// puis servir le dossier web/ avec un serveur statique (voir PROJET.md).

import init, { grayscale, invert, brightness, box_blur } from "./pkg/image_filters.js";
import { jsGrayscale, jsBoxBlur } from "./js-filters.js";

// `init()` télécharge le fichier .wasm et instancie le module.
// C'est asynchrone : tant que ce n'est pas terminé, aucune fonction
// exportée n'est utilisable. Le top-level await d'un module ES suffit.
await init();

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d", { willReadFrequently: true });
const status = document.getElementById("status");
const benchResults = document.getElementById("bench-results");

// Image de départ : un dégradé généré, pour pouvoir jouer sans rien charger.
let sourceImage = null;

function drawPlaceholder() {
  const gradient = ctx.createLinearGradient(0, 0, canvas.width, canvas.height);
  gradient.addColorStop(0, "#e63946");
  gradient.addColorStop(0.5, "#f4a261");
  gradient.addColorStop(1, "#2a9d8f");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#fff";
  ctx.font = "24px system-ui";
  ctx.fillText("Chargez une image, ou filtrez ce dégradé", 40, canvas.height / 2);
}

function resetCanvas() {
  if (sourceImage) {
    // On limite la largeur pour garder des temps de flou raisonnables.
    const scale = Math.min(1, 1024 / sourceImage.width);
    canvas.width = Math.round(sourceImage.width * scale);
    canvas.height = Math.round(sourceImage.height * scale);
    ctx.drawImage(sourceImage, 0, 0, canvas.width, canvas.height);
  } else {
    drawPlaceholder();
  }
}

document.getElementById("file-input").addEventListener("change", async (event) => {
  const file = event.target.files[0];
  if (!file) return;
  sourceImage = await createImageBitmap(file);
  resetCanvas();
  status.textContent = `Image chargée : ${canvas.width} × ${canvas.height} px.`;
});

document.getElementById("reset").addEventListener("click", resetCanvas);

// ---------------------------------------------------------------------------
// La frontière JS ↔ WASM, concrètement.
//
// `imageData.data` est un Uint8ClampedArray : le buffer RGBA du canvas.
// Les fonctions Rust prenant `&mut [u8]` attendent un Uint8Array : on crée
// une VUE Uint8Array sur LE MÊME buffer mémoire (aucune copie ici).
// La glue wasm-bindgen, elle, copie cette vue dans la mémoire linéaire du
// module, exécute la fonction Rust, puis recopie le résultat dans la vue —
// donc dans `imageData.data`. C'est le prix (deux copies) d'une API simple.
// ---------------------------------------------------------------------------

/** Applique un filtre WASM « en place » sur le canvas et affiche la durée. */
function applyInPlace(name, filterFn, ...args) {
  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
  const bytes = new Uint8Array(imageData.data.buffer);
  const start = performance.now();
  filterFn(bytes, ...args);
  const elapsed = performance.now() - start;
  ctx.putImageData(imageData, 0, 0);
  status.textContent = `${name} (WASM) : ${elapsed.toFixed(1)} ms.`;
}

document.getElementById("grayscale").addEventListener("click", () => {
  applyInPlace("Niveaux de gris", grayscale);
});

document.getElementById("invert").addEventListener("click", () => {
  applyInPlace("Inversion", invert);
});

document.getElementById("brighten").addEventListener("click", () => {
  applyInPlace("Luminosité +20", brightness, 20);
});

document.getElementById("darken").addEventListener("click", () => {
  applyInPlace("Luminosité −20", brightness, -20);
});

document.getElementById("blur").addEventListener("click", () => {
  const radius = Number(document.getElementById("radius").value);
  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
  const bytes = new Uint8Array(imageData.data.buffer);
  const start = performance.now();
  // `box_blur` retourne un Vec<u8> côté Rust → un nouveau Uint8Array côté JS.
  const blurred = box_blur(bytes, canvas.width, canvas.height, radius);
  const elapsed = performance.now() - start;
  imageData.data.set(blurred);
  ctx.putImageData(imageData, 0, 0);
  status.textContent = `Flou rayon ${radius} (WASM) : ${elapsed.toFixed(1)} ms.`;
});

// ---------------------------------------------------------------------------
// Benchmark : même algorithme, mêmes données, JS pur contre WASM.
// On garde le MEILLEUR temps de plusieurs exécutions : c'est la mesure la
// moins bruitée (le pire et la moyenne incluent le JIT, le GC, etc.).
// ---------------------------------------------------------------------------

function bestOf(runs, fn) {
  let best = Infinity;
  for (let i = 0; i < runs; i++) {
    const start = performance.now();
    fn();
    best = Math.min(best, performance.now() - start);
  }
  return best;
}

document.getElementById("bench").addEventListener("click", () => {
  const { width, height } = canvas;
  const original = ctx.getImageData(0, 0, width, height);
  const radius = Number(document.getElementById("radius").value);
  const runs = 5;

  status.textContent = "Benchmark en cours…";
  benchResults.hidden = false;

  // setTimeout pour laisser le navigateur afficher le statut avant de bloquer.
  setTimeout(() => {
    // Chaque mesure travaille sur une copie fraîche des pixels.
    const jsGray = bestOf(runs, () => {
      jsGrayscale(new Uint8ClampedArray(original.data));
    });
    const wasmGray = bestOf(runs, () => {
      grayscale(new Uint8Array(original.data));
    });
    const jsBlur = bestOf(runs, () => {
      jsBoxBlur(original.data, width, height, radius);
    });
    const wasmBlur = bestOf(runs, () => {
      box_blur(new Uint8Array(original.data.buffer), width, height, radius);
    });

    const ratio = (js, wasm) => (js / wasm).toFixed(1);
    benchResults.textContent =
      `Image ${width} × ${height} — meilleur temps sur ${runs} exécutions\n\n` +
      `Niveaux de gris   JS : ${jsGray.toFixed(2)} ms   WASM : ${wasmGray.toFixed(2)} ms` +
      `   (×${ratio(jsGray, wasmGray)})\n` +
      `Flou (rayon ${radius})    JS : ${jsBlur.toFixed(2)} ms   WASM : ${wasmBlur.toFixed(2)} ms` +
      `   (×${ratio(jsBlur, wasmBlur)})\n\n` +
      `Rappel : chaque appel WASM paie une copie aller-retour du buffer.\n` +
      `Plus le calcul par octet est lourd (flou), plus le WASM est rentable.`;
    status.textContent = "Benchmark terminé.";
  }, 50);
});

// Affichage initial.
drawPlaceholder();
status.textContent = "Module WASM chargé. À vous de jouer !";
