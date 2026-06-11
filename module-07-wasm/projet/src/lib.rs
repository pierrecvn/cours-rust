//! Filtres d'image opérant sur des buffers de pixels RGBA.
//!
//! Chaque pixel occupe 4 octets consécutifs : Rouge, Vert, Bleu, Alpha
//! (exactement le format de `ImageData.data` du `<canvas>` HTML).
//! Un buffer de N pixels a donc une longueur de 4 × N octets.
//!
//! Les fonctions sont de la pure logique sur des tranches d'octets :
//! elles se testent en natif (`cargo test`) sans navigateur, et
//! `#[wasm_bindgen]` les expose au JavaScript une fois compilées en WASM.

// TODO (fin du projet) : une fois toutes les fonctions implémentées,
// supprimez cette ligne — elle ne sert qu'à faire taire les avertissements
// « unused variable » tant que les corps sont des `todo!()`.
#![allow(unused_variables)]

use wasm_bindgen::prelude::*;

/// Calcule la luminance (niveau de gris perçu) d'un pixel RGB.
///
/// Formule standard (Rec. 601) : 0,299 × R + 0,587 × G + 0,114 × B.
/// Pour rester 100 % déterministe, on travaille en arithmétique entière :
///     (299 × r + 587 × g + 114 × b + 500) / 1000
/// Le `+ 500` arrondit au plus proche au lieu de tronquer.
///
/// Attention aux débordements : `r`, `g`, `b` sont des `u8` (max 255),
/// mais `299 × 255` dépasse largement la capacité d'un `u8` — il faut
/// convertir en `u32` AVANT de multiplier.
#[wasm_bindgen]
pub fn luminance(r: u8, g: u8, b: u8) -> u8 {
    // TODO étape 1 : implémenter la formule ci-dessus en u32, puis
    // reconvertir le résultat en u8 (il tient forcément : max = 255).
    todo!("étape 1 : luminance")
}

/// Convertit l'image en niveaux de gris, en place.
///
/// Pour chaque pixel (groupe de 4 octets), remplace R, G et B par la
/// luminance du pixel. L'alpha (4e octet) reste inchangé.
///
/// Indice : `pixels.chunks_exact_mut(4)` itère sur des sous-tranches
/// mutables de 4 octets — exactement un pixel à la fois.
#[wasm_bindgen]
pub fn grayscale(pixels: &mut [u8]) {
    // TODO étape 1 : pour chaque pixel, calculer la luminance et
    // l'écrire dans les canaux R, G et B.
    todo!("étape 1 : grayscale")
}

/// Inverse les couleurs (négatif photo), en place.
///
/// Chaque canal R, G, B devient `255 - valeur`. L'alpha reste inchangé.
#[wasm_bindgen]
pub fn invert(pixels: &mut [u8]) {
    // TODO étape 4 : inverser R, G, B de chaque pixel, laisser A intact.
    todo!("étape 4 : invert")
}

/// Ajuste la luminosité, en place.
///
/// Ajoute `delta` (qui peut être négatif) à chaque canal R, G, B, en
/// bornant le résultat dans [0, 255] (pas de débordement !). L'alpha
/// reste inchangé.
///
/// Indice : convertissez le canal en `i32`, ajoutez `delta`, puis
/// `valeur.clamp(0, 255) as u8`.
#[wasm_bindgen]
pub fn brightness(pixels: &mut [u8], delta: i32) {
    // TODO étape 4 : ajouter delta à R, G, B avec saturation [0, 255].
    todo!("étape 4 : brightness")
}

/// Flou « boîte » (box blur) : chaque pixel devient la moyenne de tous
/// les pixels situés dans un carré de côté 2 × radius + 1 centré sur lui.
///
/// Contrairement aux filtres précédents, un flou lit les pixels VOISINS :
/// impossible de modifier le buffer pendant qu'on le lit. On prend donc
/// le buffer source en lecture seule et on retourne un NOUVEAU buffer.
///
/// Gestion des bords : les voisins hors image sont simplement ignorés
/// (on divise par le nombre de voisins réellement visités, pas par
/// (2r+1)², sinon les bords s'assombrissent).
///
/// Indices :
/// - index du canal c du pixel (x, y) : `(y * width + x) * 4 + c`
/// - travaillez en `i64` pour les coordonnées (x + dx peut être négatif)
/// - accumulez les sommes des 4 canaux en `u32`, comptez les voisins,
///   puis poussez `(somme / compte) as u8` dans le buffer de sortie.
#[wasm_bindgen]
pub fn box_blur(pixels: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    // TODO étape 4 : double boucle sur (y, x), double boucle sur
    // (dy, dx) dans [-radius, +radius], moyenne des voisins valides.
    todo!("étape 4 : box_blur")
}
