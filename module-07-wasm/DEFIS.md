# Défis du module 7

Trois extensions non guidées. Aucun corrigé : les tests natifs que vous écrirez sont votre filet de sécurité — la méthode reste la même que dans le projet : logique pure testée avec `cargo test`, navigateur en dernier.

## Défi 1 — Filtre sépia

Ajoutez un filtre `sepia(pixels: &mut [u8])` donnant à l'image une teinte vieille photo. La transformation standard, pour chaque pixel :

```text
R' = 0,393·R + 0,769·G + 0,189·B
G' = 0,349·R + 0,686·G + 0,168·B
B' = 0,272·R + 0,534·G + 0,131·B
```

Pièges à anticiper :

- contrairement à la luminance, ces sommes **peuvent dépasser 255** (vérifiez : 0,393 + 0,769 + 0,189 > 1) — il faut saturer ;
- les trois nouveaux canaux se calculent à partir des trois **anciens** : ne modifiez pas R avant de l'avoir lu pour G' et B'.

Écrivez d'abord les tests (pixel blanc → reste-t-il blanc ? pixel noir ?), implémentez, exposez avec `#[wasm_bindgen]`, ajoutez un bouton dans la page. Version entière (poids × 1000) recommandée, comme pour la luminance.

## Défi 2 — Rotation et miroir

Ajoutez deux transformations géométriques :

- `flip_horizontal(pixels: &mut [u8], width: u32, height: u32)` — miroir gauche-droite, **en place** ;
- `rotate90(pixels: &[u8], width: u32, height: u32) -> Vec<u8>` — rotation de 90° horaire, qui retourne un nouveau buffer.

Questions auxquelles vous devrez répondre en chemin :

- pourquoi le miroir peut-il se faire en place alors que la rotation, sur une image non carrée, ne le peut raisonnablement pas ? (Indice : dimensions du résultat.)
- pour le miroir en place, `slice::swap` échange deux octets, mais il faut échanger des **pixels** de 4 octets : regardez du côté de `chunks_exact_mut` ligne par ligne, ou d'une double boucle d'indices ;
- côté page web : après une rotation, `canvas.width` et `canvas.height` doivent être échangés avant le `putImageData`. C'est un (petit) morceau de JS à écrire vous-même cette fois.

Testez la rotation sur une image 2 × 3 construite à la main : la position attendue de chaque pixel se calcule sur papier. Quatre rotations successives doivent redonner l'image d'origine — excellent test de propriété.

## Défi 3 — Flou à rayon variable : profiler et optimiser

Le `box_blur` du projet est en O(largeur × hauteur × (2r + 1)²) : le coût explose avec le rayon.

1. **Mesurez d'abord.** Dans la page, relevez le temps WASM du flou pour r = 2, 4, 8, 16 sur la même image, et tracez (même grossièrement) le temps en fonction de r. Retrouvez-vous la croissance quadratique ?
2. **Optimisez.** Un box blur est *séparable* : flouter horizontalement (moyenne sur 2r + 1 pixels de la même ligne) puis verticalement donne le même résultat qu'un carré complet, pour un coût en O(largeur × hauteur × (2r + 1)) — linéaire en r. Implémentez `box_blur_fast` avec deux passes.
3. **Prouvez l'équivalence… ou pas.** Écrivez un test natif comparant `box_blur` et `box_blur_fast` sur une petite image aléatoire fixe. Surprise probable : les résultats diffèrent de ±1 sur certains octets — deux divisions entières successives n'arrondissent pas comme une seule. Décidez en ingénieur : tolérance de ±1 dans le test (documentez pourquoi), ou accumulation exacte (sommes intermédiaires non divisées) pour une égalité stricte.
4. **Re-mesurez.** Refaites la courbe de l'étape 1 avec la version rapide, et comparez aussi `box_blur_fast` en WASM contre le `jsBoxBlur` naïf : que devient le ratio ? Que faudrait-il faire pour que la comparaison redevienne équitable ?

Bonus pour les curieux : `wasm-pack build --profiling` produit un build optimisé avec les noms de fonctions conservés — l'onglet Performance des outils de développement du navigateur vous montrera alors où le temps part réellement.
