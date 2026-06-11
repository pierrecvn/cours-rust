//! Filtres d'image opérant sur des buffers de pixels RGBA — corrigé.
//!
//! Chaque pixel occupe 4 octets consécutifs : Rouge, Vert, Bleu, Alpha
//! (le format exact de `ImageData.data` du `<canvas>` HTML). Un buffer
//! de N pixels a donc une longueur de 4 × N octets.
//!
//! Architecture : les fonctions sont de la pure logique sur des tranches
//! d'octets, sans aucune dépendance au DOM ni au navigateur. Conséquences :
//! - elles se testent en natif avec un simple `cargo test` ;
//! - `#[wasm_bindgen]` suffit à les exposer au JavaScript quand la crate
//!   est compilée vers `wasm32-unknown-unknown`.
//!
//! Côté frontière JS ↔ WASM :
//! - `&mut [u8]`   : la glue JS copie le `Uint8Array` dans la mémoire
//!   linéaire du module, exécute la fonction, puis recopie le résultat
//!   dans le tableau JS. Deux copies, mais une API très simple.
//! - `Vec<u8>` en retour : la glue lit la mémoire WASM et construit un
//!   nouveau `Uint8Array` côté JS (une copie).

use wasm_bindgen::prelude::*;

/// Calcule la luminance (niveau de gris perçu) d'un pixel RGB.
///
/// Formule standard (Rec. 601) : 0,299 × R + 0,587 × G + 0,114 × B.
/// L'œil humain est plus sensible au vert qu'au rouge, et au rouge
/// qu'au bleu — d'où les poids.
///
/// On reste en arithmétique entière pour un résultat parfaitement
/// déterministe (les flottants arrondissent différemment selon les
/// plateformes) : poids multipliés par 1000, `+ 500` pour arrondir au
/// plus proche, division entière finale.
#[wasm_bindgen]
pub fn luminance(r: u8, g: u8, b: u8) -> u8 {
    // Conversion en u32 AVANT la multiplication : 299 × 255 = 76 245,
    // bien au-delà de ce qu'un u8 (ou même un u16) peut contenir.
    let weighted = 299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b);
    // Maximum possible : (299 + 587 + 114) × 255 + 500 = 255 500,
    // donc le résultat de la division tient toujours dans un u8.
    ((weighted + 500) / 1000) as u8
}

/// Convertit l'image en niveaux de gris, en place.
#[wasm_bindgen]
pub fn grayscale(pixels: &mut [u8]) {
    // `chunks_exact_mut(4)` découpe la tranche en sous-tranches mutables
    // de 4 octets : un pixel RGBA à la fois. C'est l'équivalent sûr et
    // sans index manuel d'une boucle `for i in (0..len).step_by(4)`.
    for pixel in pixels.chunks_exact_mut(4) {
        let gray = luminance(pixel[0], pixel[1], pixel[2]);
        pixel[0] = gray;
        pixel[1] = gray;
        pixel[2] = gray;
        // pixel[3] (alpha) reste intact.
    }
}

/// Inverse les couleurs (négatif photo), en place.
#[wasm_bindgen]
pub fn invert(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        // On n'inverse que R, G, B : inverser l'alpha rendrait
        // l'image transparente, ce qui n'est pas un « négatif ».
        for channel in &mut pixel[..3] {
            *channel = 255 - *channel;
        }
    }
}

/// Ajuste la luminosité, en place.
///
/// `delta` peut être négatif (assombrir). Le résultat est borné dans
/// [0, 255] : en Rust, un débordement de u8 panique en mode debug —
/// le passage par `i32` + `clamp` rend l'intention explicite.
#[wasm_bindgen]
pub fn brightness(pixels: &mut [u8], delta: i32) {
    for pixel in pixels.chunks_exact_mut(4) {
        for channel in &mut pixel[..3] {
            // i32 est assez large pour 255 + delta quel que soit delta
            // raisonnable ; clamp ramène dans la plage d'un canal.
            *channel = (i32::from(*channel) + delta).clamp(0, 255) as u8;
        }
    }
}

/// Flou « boîte » (box blur) : chaque pixel devient la moyenne de tous
/// les pixels situés dans un carré de côté 2 × radius + 1 centré sur lui.
///
/// Un flou lit les pixels voisins : on ne peut pas écrire dans le buffer
/// qu'on est en train de lire (le résultat dépendrait de l'ordre de
/// parcours !). D'où la signature : source en lecture seule (`&[u8]`),
/// nouveau buffer en sortie (`Vec<u8>`). Le borrow checker nous aurait
/// de toute façon interdit de faire les deux à la fois sur la même tranche.
///
/// Complexité : O(largeur × hauteur × (2r+1)²) — volontairement naïf.
/// Le défi 3 du module explore une version séparable en O(largeur ×
/// hauteur × (2r+1)).
#[wasm_bindgen]
pub fn box_blur(pixels: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    // Coordonnées en i64 : `x + dx` peut être négatif près des bords,
    // et un u32 négatif… n'existe pas. i64 absorbe tous les cas.
    let (w, h, r) = (i64::from(width), i64::from(height), i64::from(radius));
    let mut output = Vec::with_capacity(pixels.len());

    for y in 0..h {
        for x in 0..w {
            // Sommes des 4 canaux du voisinage, et nombre de voisins
            // réellement dans l'image (les bords en ont moins).
            let mut sums = [0u32; 4];
            let mut count = 0u32;

            for dy in -r..=r {
                for dx in -r..=r {
                    let (nx, ny) = (x + dx, y + dy);
                    // Les voisins hors image sont ignorés : on divisera
                    // par `count`, pas par (2r+1)², pour ne pas
                    // assombrir les bords.
                    if (0..w).contains(&nx) && (0..h).contains(&ny) {
                        let idx = ((ny * w + nx) * 4) as usize;
                        for (sum, &value) in sums.iter_mut().zip(&pixels[idx..idx + 4]) {
                            *sum += u32::from(value);
                        }
                        count += 1;
                    }
                }
            }

            // `count` ≥ 1 : le pixel central (dx = dy = 0) est toujours
            // dans l'image, la division est donc sûre.
            for sum in sums {
                output.push((sum / count) as u8);
            }
        }
    }

    output
}
