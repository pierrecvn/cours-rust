// Implémentations JavaScript « pur » des mêmes filtres que la crate Rust.
// Elles ne servent qu'au benchmark : comparer le temps d'exécution du même
// algorithme en JS et en WASM, à logique strictement identique.

/** Niveaux de gris, en place. `pixels` est un Uint8Array ou Uint8ClampedArray RGBA. */
export function jsGrayscale(pixels) {
  for (let i = 0; i < pixels.length; i += 4) {
    // Même formule entière que la version Rust : (299r + 587g + 114b + 500) / 1000.
    const gray =
      (299 * pixels[i] + 587 * pixels[i + 1] + 114 * pixels[i + 2] + 500) / 1000 | 0;
    pixels[i] = gray;
    pixels[i + 1] = gray;
    pixels[i + 2] = gray;
    // pixels[i + 3] (alpha) inchangé.
  }
}

/**
 * Box blur naïf, identique à l'algorithme Rust : retourne un NOUVEAU buffer.
 * Complexité O(width × height × (2·radius + 1)²), volontairement.
 */
export function jsBoxBlur(pixels, width, height, radius) {
  const output = new Uint8ClampedArray(pixels.length);
  let o = 0;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      let r = 0, g = 0, b = 0, a = 0, count = 0;
      for (let dy = -radius; dy <= radius; dy++) {
        for (let dx = -radius; dx <= radius; dx++) {
          const nx = x + dx, ny = y + dy;
          if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
            const idx = (ny * width + nx) * 4;
            r += pixels[idx];
            g += pixels[idx + 1];
            b += pixels[idx + 2];
            a += pixels[idx + 3];
            count++;
          }
        }
      }
      output[o++] = r / count | 0;
      output[o++] = g / count | 0;
      output[o++] = b / count | 0;
      output[o++] = a / count | 0;
    }
  }
  return output;
}
