# Projet 7 — Filtres d'image dans le navigateur

Vous allez écrire une bibliothèque Rust de filtres d'image (niveaux de gris, inversion, luminosité, flou), la compiler en WebAssembly, et l'utiliser depuis une page web avec un `<canvas>` : chargement d'une image, application des filtres, et benchmark JS pur contre WASM.

L'architecture suit la règle du cours théorique : **JS pilote, WASM calcule**. La page HTML et le JavaScript sont fournis dans `projet/web/` — votre travail est côté Rust, dans `projet/src/lib.rs`. Lisez quand même le JS fourni : la frontière JS ↔ WASM y est commentée pas à pas.

## Mise en place

```bash
cd module-07-wasm/projet
cargo test --no-run    # le squelette doit compiler tel quel
```

Outils nécessaires (une seule fois) :

```bash
# wasm-pack : compile + génère la glue JS (voir COURS.md § 4)
cargo install wasm-pack

# La cible wasm32-unknown-unknown est déjà installée sur votre machine.
# Sinon : rustup target add wasm32-unknown-unknown
```

Pour servir la page web (étape 3), n'importe quel serveur statique fait l'affaire :

```bash
python3 -m http.server 8000    # ou : npx serve
```

Rappel important : ouvrir `index.html` en double-cliquant (`file://`) **ne marchera pas** — les modules ES et le WASM exigent HTTP.

---

## Étape 1 — La bibliothèque et le premier filtre, testés en natif

**Objectif.** Implémenter `luminance` et `grayscale` dans `src/lib.rs`, et faire passer leurs tests **sans toucher au navigateur**.

**Pourquoi.** C'est le cœur de la méthode : les filtres sont de la pure logique sur des `&[u8]`/`&mut [u8]`. On les développe et on les valide avec `cargo test`, en natif, avec le confort habituel (messages d'erreur, rapidité, débogueur). Le navigateur n'interviendra que pour l'affichage. Si vous commencez par le navigateur, chaque erreur vous coûte un cycle compilation-WASM + rechargement de page.

**Quoi faire.**

1. Ouvrez `src/lib.rs` : les signatures, la doc et les `todo!()` sont en place. Ouvrez aussi `tests/filters.rs` : les tests fournis sont votre spécification exécutable.
2. Implémentez `luminance(r, g, b)` : la formule entière est donnée dans le commentaire de la fonction. Piège classique : convertissez en `u32` **avant** de multiplier (en Rust, `299u32 * u32::from(r)`, pas `299 * r` sur des `u8` — le débordement panique en debug, contrairement à Java où l'`int` déborde en silence ou à JS où tout est flottant).
3. Implémentez `grayscale(pixels)` : itérez pixel par pixel avec `pixels.chunks_exact_mut(4)` et écrivez la luminance dans R, G et B. Ne touchez pas à l'alpha (4ᵉ octet) : une image opaque doit rester opaque.

**Indices.**

- `chunks_exact_mut(4)` donne des `&mut [u8]` de longueur 4 : `pixel[0]` à `pixel[3]` sont R, G, B, A.
- Les tests vous donnent les valeurs attendues exactes : rouge pur → 76, vert pur → 150, bleu pur → 29.

**Point de contrôle.**

```bash
cargo test luminance
cargo test grayscale
```

Tous verts. Les autres tests paniquent encore sur `todo!()` — c'est normal.

---

## Étape 2 — Premier build WASM

**Objectif.** Compiler la crate en module WebAssembly et inspecter ce que wasm-pack a généré.

**Pourquoi.** Avant d'écrire la page, vérifions que la chaîne de compilation fonctionne et regardons concrètement les artefacts décrits dans le cours — y compris les types TypeScript générés.

**Quoi faire.**

1. À la racine de `projet/`, lancez :

   ```bash
   wasm-pack build --target web --out-dir web/pkg
   ```

   (`--out-dir web/pkg` place le résultat dans le dossier web, pour que la page puisse l'importer en chemin relatif simple.)

2. Explorez `web/pkg/` : repérez le `.wasm` (votre Rust compilé), la glue `image_filters.js`, et surtout `image_filters.d.ts`.

**Indices.**

- Si la compilation échoue avec une erreur mentionnant `cdylib`, relisez le `[lib]` du `Cargo.toml` (il est fourni correct — mais sachez diagnostiquer).
- Notez la taille du `.wasm` : quelques dizaines de kilo-octets. Comparez mentalement avec le moindre bundle JS de vos projets TS.

**Point de contrôle.** `web/pkg/image_filters.d.ts` existe et contient la signature `export function grayscale(pixels: Uint8Array): void;` — vos types Rust, devenus types TS, sans rien écrire.

---

## Étape 3 — La page canvas

**Objectif.** Servir la page fournie et appliquer votre filtre niveaux de gris sur une vraie image.

**Pourquoi.** C'est le moment où les deux mondes se rejoignent : le buffer `Uint8ClampedArray` du canvas part dans votre fonction Rust et en revient transformé. Comprendre ce trajet (et ses copies) est l'objectif pédagogique central du module.

**Quoi faire.**

1. Servez le dossier `web/` :

   ```bash
   cd web && python3 -m http.server 8000
   ```

2. Ouvrez `http://localhost:8000`. La page affiche un dégradé de test.
3. Cliquez sur « Niveaux de gris » : le dégradé devient gris. Chargez une image à vous et recommencez.
4. Lisez `web/main.js`, en particulier la fonction `applyInPlace` et le commentaire sur la frontière : pourquoi crée-t-on un `Uint8Array` sur le buffer du `Uint8ClampedArray` ? Combien de copies du buffer un clic déclenche-t-il ? (Réponse dans le commentaire — mais formulez la vôtre d'abord.)

**Indices.**

- Boutons sans effet ou page blanche → ouvrez la console du navigateur (F12). Une 404 sur `pkg/image_filters.js` signifie que l'étape 2 n'a pas écrit dans `web/pkg`, ou que le serveur n'est pas lancé depuis `web/`.
- Après chaque modification du Rust : relancer `wasm-pack build --target web --out-dir web/pkg`, puis recharger la page (pensez à vider le cache si le navigateur s'accroche à l'ancien .wasm : Ctrl+Maj+R).

**Point de contrôle.** Votre image passe en niveaux de gris dans le navigateur, et le statut affiche la durée du filtre en millisecondes.

---

## Étape 4 — Tous les filtres

**Objectif.** Implémenter `invert`, `brightness` et `box_blur` ; tous les tests natifs passent ; tous les boutons de la page fonctionnent.

**Pourquoi.** `invert` et `brightness` consolident le motif « filtre point par point en place ». `box_blur` introduit la vraie difficulté : un filtre **de voisinage**, où le borrow checker impose une signature différente (relisez l'aparté du cours, § 5 — l'erreur E0502 y est expliquée).

**Quoi faire.**

1. `invert` : chaque canal R, G, B devient `255 - valeur` ; alpha intact. Trois lignes.
2. `brightness(pixels, delta)` : ajoutez `delta` (possiblement négatif) à R, G, B en bornant dans [0, 255]. Le test `brightness_saturates_at_255` est là pour vous faire trébucher : `pixel[0] + 20` sur un `u8` à 250 **panique** en debug. Passez par `i32` puis `clamp(0, 255)`.
3. `box_blur(pixels, width, height, radius)` : pour chaque pixel, moyenne de tous les pixels du carré `(2·radius + 1)²` centré sur lui, **en ignorant les voisins hors image** (divisez par le nombre de voisins effectivement visités). Retournez un nouveau `Vec<u8>`.

**Indices (box_blur).**

- Index du canal `c` du pixel `(x, y)` : `(y * width + x) * 4 + c`.
- `x + dx` peut être négatif : faites l'arithmétique de coordonnées en `i64`, pas en `u32` (un `u32` négatif n'existe pas — le compilateur ou un débordement vous le rappellerait brutalement).
- Accumulez les sommes des 4 canaux dans un `[u32; 4]` et comptez les voisins valides ; le pixel central est toujours valide, donc la division est sûre.
- Le test `box_blur_averages_the_neighborhood` travaille sur une image 3 × 1 calculée à la main : si vous échouez, déroulez-le sur papier.

**Point de contrôle.**

```bash
cargo test          # 15 tests, tous verts
```

Puis rebuild WASM (étape 2) et vérifiez chaque bouton dans la page, y compris le flou avec différents rayons. Supprimez le `#![allow(unused_variables)]` en tête de `lib.rs` : plus aucun avertissement ne doit subsister.

---

## Étape 5 — Benchmark : JS pur contre WASM

**Objectif.** Mesurer, sur la même image et le même algorithme, le temps d'exécution JavaScript et WebAssembly.

**Pourquoi.** Le cours (§ 6) annonce que le gain WASM dépend de l'intensité du calcul par octet transféré. Une affirmation de cours n'est pas une mesure : vérifiez-la sur votre machine.

**Quoi faire.**

1. Lisez `web/js-filters.js` : `jsGrayscale` et `jsBoxBlur` sont les transcriptions JS fidèles de vos fonctions Rust (même formule entière, même gestion des bords). Un benchmark n'a de sens qu'à algorithme identique.
2. Chargez une image raisonnablement grande (la page la redimensionne à 1024 px de large maximum), cliquez « Benchmark JS vs WASM ».
3. Analysez :
   - sur le **niveau de gris**, l'écart est modeste, parfois quasi nul — le JIT de votre navigateur est excellent sur ce genre de boucle, et la copie de frontière pèse dans le temps WASM ;
   - sur le **flou**, l'écart se creuse. Augmentez le rayon (3 → 6 → 10) et observez : le coût de calcul croît en (2r + 1)², le coût de copie reste constant — le ratio penche de plus en plus vers le WASM.
4. Question pour vérifier votre compréhension : le temps « WASM » mesuré inclut-il les copies de frontière ? (Regardez où sont placés les `performance.now()` dans `main.js`.) Est-ce une mesure honnête ? Pourquoi ?

**Point de contrôle.** Le panneau de résultats affiche les quatre temps et les ratios. Vous savez expliquer pourquoi le ratio du flou dépasse celui du niveau de gris, et ce qui se passerait avec un rayon encore plus grand.

---

## Pour aller plus loin

Direction `DEFIS.md` — sépia, transformations géométriques, et profilage d'un flou optimisé. Le corrigé complet et commenté est dans `solution/`.
