//! Tests natifs des filtres — aucun navigateur nécessaire.
//!
//! Les filtres sont de la pure logique sur des octets : on les teste avec
//! un simple `cargo test`, exactement comme n'importe quelle crate Rust.
//! C'est tout l'intérêt d'avoir gardé les fonctions indépendantes du DOM.

use image_filters::{box_blur, brightness, grayscale, invert, luminance};

// ---------------------------------------------------------------- luminance

#[test]
fn luminance_of_black_is_zero() {
    assert_eq!(luminance(0, 0, 0), 0);
}

#[test]
fn luminance_of_white_is_max() {
    assert_eq!(luminance(255, 255, 255), 255);
}

#[test]
fn luminance_weights_channels_correctly() {
    // Avec la formule entière (299·r + 587·g + 114·b + 500) / 1000 :
    assert_eq!(luminance(255, 0, 0), 76); // rouge pur
    assert_eq!(luminance(0, 255, 0), 150); // vert pur (le plus lumineux)
    assert_eq!(luminance(0, 0, 255), 29); // bleu pur (le plus sombre)
}

// ---------------------------------------------------------------- grayscale

#[test]
fn grayscale_makes_channels_equal_and_keeps_alpha() {
    // Deux pixels : un rouge pur opaque, un vert pur semi-transparent.
    let mut pixels = vec![255, 0, 0, 255, 0, 255, 0, 128];
    grayscale(&mut pixels);
    assert_eq!(pixels, vec![76, 76, 76, 255, 150, 150, 150, 128]);
}

#[test]
fn grayscale_leaves_gray_pixels_unchanged() {
    let mut pixels = vec![100, 100, 100, 200];
    grayscale(&mut pixels);
    assert_eq!(pixels, vec![100, 100, 100, 200]);
}

// ------------------------------------------------------------------- invert

#[test]
fn invert_flips_rgb_and_keeps_alpha() {
    let mut pixels = vec![10, 20, 30, 255, 0, 255, 128, 64];
    invert(&mut pixels);
    assert_eq!(pixels, vec![245, 235, 225, 255, 255, 0, 127, 64]);
}

#[test]
fn invert_twice_is_identity() {
    let original = vec![12, 34, 56, 78, 90, 123, 213, 255];
    let mut pixels = original.clone();
    invert(&mut pixels);
    invert(&mut pixels);
    assert_eq!(pixels, original);
}

// --------------------------------------------------------------- brightness

#[test]
fn brightness_adds_delta_to_rgb_only() {
    let mut pixels = vec![100, 110, 120, 200];
    brightness(&mut pixels, 10);
    assert_eq!(pixels, vec![110, 120, 130, 200]);
}

#[test]
fn brightness_saturates_at_255() {
    let mut pixels = vec![250, 255, 0, 255];
    brightness(&mut pixels, 20);
    assert_eq!(pixels, vec![255, 255, 20, 255]);
}

#[test]
fn brightness_saturates_at_zero_with_negative_delta() {
    let mut pixels = vec![5, 0, 200, 255];
    brightness(&mut pixels, -20);
    assert_eq!(pixels, vec![0, 0, 180, 255]);
}

// ----------------------------------------------------------------- box_blur

/// Construit une image w × h dont tous les pixels valent `rgba`.
fn uniform_image(w: usize, h: usize, rgba: [u8; 4]) -> Vec<u8> {
    rgba.iter().copied().cycle().take(w * h * 4).collect()
}

#[test]
fn box_blur_keeps_output_length() {
    let pixels = uniform_image(5, 3, [10, 20, 30, 255]);
    let blurred = box_blur(&pixels, 5, 3, 1);
    assert_eq!(blurred.len(), pixels.len());
}

#[test]
fn box_blur_of_uniform_image_is_unchanged() {
    // La moyenne de pixels identiques est ce même pixel, peu importe le rayon.
    let pixels = uniform_image(4, 4, [50, 100, 150, 255]);
    let blurred = box_blur(&pixels, 4, 4, 2);
    assert_eq!(blurred, pixels);
}

#[test]
fn box_blur_with_radius_zero_is_identity() {
    let pixels = vec![1, 2, 3, 4, 5, 6, 7, 8]; // image 2 × 1
    let blurred = box_blur(&pixels, 2, 1, 0);
    assert_eq!(blurred, pixels);
}

#[test]
fn box_blur_averages_the_neighborhood() {
    // Image 3 × 1 : rouge 90, rouge 0, rouge 30 (autres canaux à 0, alpha 255).
    let pixels = vec![90, 0, 0, 255, 0, 0, 0, 255, 30, 0, 0, 255];
    let blurred = box_blur(&pixels, 3, 1, 1);
    // Pixel 0 : voisins {90, 0}        → moyenne 45
    // Pixel 1 : voisins {90, 0, 30}    → moyenne 40
    // Pixel 2 : voisins {0, 30}        → moyenne 15
    assert_eq!(blurred[0], 45);
    assert_eq!(blurred[4], 40);
    assert_eq!(blurred[8], 15);
    // L'alpha (uniforme) reste à 255 partout.
    assert_eq!(blurred[3], 255);
    assert_eq!(blurred[7], 255);
    assert_eq!(blurred[11], 255);
}

#[test]
fn box_blur_ignores_pixels_outside_the_image() {
    // Image 2 × 2 dont seul le pixel (0, 0) est blanc.
    // Avec un rayon de 1, chaque pixel voit les 4 pixels de l'image
    // (les voisins hors limites sont ignorés) : moyenne = 255 / 4 = 63.
    let mut pixels = uniform_image(2, 2, [0, 0, 0, 0]);
    pixels[0] = 255;
    pixels[1] = 255;
    pixels[2] = 255;
    let blurred = box_blur(&pixels, 2, 2, 1);
    for pixel in blurred.chunks_exact(4) {
        assert_eq!(&pixel[..3], &[63, 63, 63]);
    }
}
