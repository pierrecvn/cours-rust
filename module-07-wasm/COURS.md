# Module 7 — Rust dans le navigateur avec WebAssembly

Dans les modules précédents, votre Rust tournait sur votre machine : binaires CLI, serveur Axum, application Tauri. Ce module l'envoie là où on ne l'attend pas : **dans le navigateur**, aux côtés de votre JavaScript, grâce à WebAssembly.

Prérequis : modules 1 à 6 (en particulier les slices du module 1 et l'organisation lib/tests du module 3). La cible `wasm32-unknown-unknown` est déjà installée sur votre machine.

---

## 1. Qu'est-ce que WebAssembly ?

**Définition.** WebAssembly (WASM) est un **format de binaire portable** : un jeu d'instructions de bas niveau, compact, que tous les navigateurs modernes savent exécuter à une vitesse proche du natif. Ce n'est pas un langage qu'on écrit à la main : c'est une **cible de compilation**, comme le bytecode JVM ou le code machine x86. On écrit du Rust (ou du C, du C++, du Go…), on compile vers WASM, et le navigateur exécute le résultat.

**Comparaison Java/TS.**
- Côté **Java**, l'analogie est directe : WASM est au navigateur ce que le bytecode est à la JVM. Vous compilez vers un format intermédiaire portable, et une machine virtuelle (ici, celle du navigateur) l'exécute. Différence notable : WASM est conçu pour être compilé en code machine très vite et tourner sans interpréteur ni warm-up de JIT.
- Côté **TypeScript**, le contraste est instructif : TS est *transpilé* en JS, qui reste du texte que le moteur doit parser, interpréter, puis optimiser à chaud (JIT). Un module WASM arrive déjà sous forme binaire typée et validée : le moteur le compile en code machine en une passe, et les performances sont **prévisibles** — pas de « déoptimisation » surprise comme en JS quand le JIT abandonne une hypothèse.

**La sandbox.** Un module WASM s'exécute dans un bac à sable strict :

- il ne voit qu'une **mémoire linéaire** : un grand tableau d'octets qui lui appartient, alloué par le moteur. Il ne peut ni lire ni écrire en dehors ;
- il n'a **aucun accès** au DOM, au réseau, aux fichiers, à l'horloge… Tout ce qu'il sait faire, c'est du calcul sur sa mémoire, plus appeler les fonctions que le JavaScript hôte a bien voulu lui passer ;
- chaque module est validé avant exécution : pas de saut vers une adresse arbitraire, pas de débordement hors de la mémoire linéaire.

C'est le même esprit que la sandbox des applets ou du `SecurityManager` que Java a tenté (et abandonné), mais conçu dès le départ dans le format lui-même : la sûreté est une propriété du bytecode, pas une couche de contrôle ajoutée après coup.

**Complémentaire de JS, pas un remplaçant.** Point capital pour un dev TS : WASM ne « tue » pas JavaScript.

| Tâche | Bon outil |
|---|---|
| Manipuler le DOM, gérer les événements, orchestrer l'UI | JavaScript/TypeScript |
| Appels réseau, logique métier ordinaire | JavaScript/TypeScript |
| Calcul intensif : traitement d'image/vidéo/audio, compression, crypto, physique, parsing lourd | WASM |
| Réutiliser une bibliothèque existante écrite en Rust/C/C++ (ffmpeg, sqlite, un moteur de jeu…) | WASM |

Le modèle gagnant est : **JS pilote, WASM calcule**. Figma (moteur de rendu C++ en WASM), Google Earth, AutoCAD web, la prévisualisation Photoshop dans le navigateur : dans tous ces produits, l'interface reste du JS/TS, et le cœur de calcul est un module WASM. C'est exactement l'architecture de notre projet : la page, le `<canvas>` et les boutons en JS ; les filtres d'image en Rust.

À noter : WASM sort aussi du navigateur (serverless, plugins, edge computing) via des runtimes comme Wasmtime — même binaire, même sandbox, sans navigateur. Nous restons ici sur le cas navigateur.

---

## 2. Pourquoi Rust est LE langage de WebAssembly

N'importe quel langage compilé peut viser WASM. Pourquoi Rust est-il devenu le choix de référence ?

**Pas de runtime à embarquer.** Un module WASM doit emporter avec lui tout ce dont son langage a besoin pour fonctionner. Java aurait besoin d'une JVM et d'un garbage collector ; Go embarque son GC et son scheduler (binaire WASM minimal : plusieurs mégaoctets). Rust, lui, gère la mémoire par ownership — **à la compilation** (module 1). Résultat : rien à embarquer, à part un petit allocateur. Un module Rust trivial pèse quelques **kilo-octets**.

Pourquoi la taille compte double sur le web : le binaire est téléchargé par chaque visiteur, et chaque kilooctet retarde le premier rendu. Un dev TS qui surveille la taille de ses bundles est en terrain connu.

**Sûreté mémoire sans coût.** La sandbox WASM empêche un module de corrompre la page, mais pas de se corrompre lui-même : un module C avec un buffer overflow écrase joyeusement ses propres données. Les garanties du borrow checker s'appliquent telles quelles dans le module compilé — sans GC ni vérifications à l'exécution.

**Un outillage de premier ordre.** C'est l'argument décisif au quotidien : la cible s'installe en une commande (`rustup target add wasm32-unknown-unknown` — déjà fait sur votre machine), et l'écosystème `wasm-bindgen` + `wasm-pack` (sections suivantes) automatise toute la plomberie d'interopérabilité avec JS, y compris la génération de définitions TypeScript.

Le triplet de cible se lit ainsi : `wasm32` (architecture : WASM, pointeurs 32 bits) - `unknown` (pas de fournisseur) - `unknown` (pas de système d'exploitation — cohérent : la sandbox n'offre pas d'OS).

---

## 3. wasm-bindgen : le pont entre Rust et JavaScript

**Le problème.** WASM, nativement, ne sait échanger avec JS que des nombres (`i32`, `i64`, `f32`, `f64`). Pas de chaînes, pas de tableaux, pas d'objets. Pour passer une `String`, il faudrait écrire à la main : allouer de la mémoire linéaire côté Rust, y copier les octets UTF-8 côté JS, passer le pointeur et la longueur, libérer après usage… pour chaque fonction.

**Définition.** [`wasm-bindgen`](https://github.com/wasm-bindgen/wasm-bindgen) est une crate qui génère toute cette plomberie. On annote une fonction avec `#[wasm_bindgen]`, et l'outillage produit le code Rust de conversion **et** le code JavaScript « de glue » correspondant.

**Comparaison Java/TS.** C'est l'équivalent de JNI côté Java (appeler du natif depuis la JVM), mais avec la pénibilité en moins : là où JNI demande d'écrire les conversions à la main, `#[wasm_bindgen]` les dérive de la signature. Côté TS, pensez aux types générés par un client d'API (OpenAPI, Prisma…) : on décrit une fois, l'outillage produit l'interface des deux côtés. Vous avez déjà rencontré cette idée au module 6 : les commandes Tauri (`#[tauri::command]` + `invoke`) jouent exactement ce rôle entre le frontend TS et le backend Rust — `#[wasm_bindgen]` est le même pont, mais à l'intérieur du navigateur.

**Exemple minimal.**

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Bonjour, {name} !")
}
```

Côté JavaScript, après compilation :

```js
import init, { greet } from "./pkg/my_crate.js";
await init();                  // charge et instancie le .wasm
console.log(greet("Ada"));     // "Bonjour, Ada !"
```

### Les types qui passent la frontière

| Côté Rust | Côté JS | Mécanisme |
|---|---|---|
| `u8`…`u32`, `i8`…`i32`, `f32`, `f64`, `bool` | `number` / `boolean` | passage direct, quasi gratuit |
| `u64` / `i64` | `bigint` | direct |
| `&str`, `String` | `string` | **copie** + conversion UTF-16 ⇄ UTF-8 |
| `&[u8]`, `Vec<u8>` (et autres slices numériques) | `Uint8Array`, etc. | **copie** vers/depuis la mémoire linéaire |
| `&mut [u8]` | `Uint8Array` | copie aller, exécution, **copie retour** |
| `Option<T>` | `T \| undefined` | conversion |
| struct annotée `#[wasm_bindgen]` | classe JS | l'objet **reste côté WASM**, JS reçoit un handle |
| `JsValue` | n'importe quelle valeur JS | référence opaque côté Rust |

**Le coût des copies.** La mémoire linéaire WASM et le tas JavaScript sont deux mondes étanches : JS ne peut pas pointer « dans » un `Vec` Rust, et Rust ne peut pas pointer dans un objet JS. Chaque chaîne ou tableau qui traverse est donc **copié**. Pour notre projet, c'est concret : passer le buffer d'une image 1024 × 768 (3 Mo de RGBA) à un filtre `&mut [u8]` coûte une copie de 3 Mo à l'aller et une au retour. C'est rapide (de l'ordre de la milliseconde), mais pas gratuit : la règle d'or est **peu d'appels gros plutôt que beaucoup de petits** — un appel qui traite l'image entière, jamais un appel par pixel.

### Ce qui ne passe pas

- **Les références empruntées en retour** : impossible de retourner un `&str` ou un `&[u8]` (vers la mémoire de qui pointerait-il, une fois la fonction terminée ? le borrow checker pose la question fatale). On retourne des valeurs possédées : `String`, `Vec<u8>`.
- **Les génériques** : `fn process<T>(…)` n'a pas de sens pour JS, qui a besoin d'une signature concrète. On expose des fonctions monomorphes.
- **Les enums riches** : un `enum` Rust avec données (`Result<T, E>`, `Option` imbriquées complexes…) n'a pas d'équivalent JS direct ; seuls les enums « à la C » (variantes sans données) passent. `Result<T, JsValue>` est toutefois géré : `Err` devient une exception JS levée.
- **Les traits, les lifetimes, les closures Rust ordinaires** : tout cela reste à l'intérieur du module.

En pratique : la frontière doit rester **simple** — nombres, chaînes, buffers. La richesse du système de types Rust vit *derrière* la frontière, pas dessus. C'est une discipline saine, la même qu'une bonne API REST.

---

## 4. wasm-pack : compiler et empaqueter

**Définition.** [`wasm-pack`](https://drager.github.io/wasm-pack/) est l'outil en ligne de commande qui orchestre tout : il compile la crate vers `wasm32-unknown-unknown`, exécute wasm-bindgen pour générer la glue, optimise le binaire, et range le tout dans un dossier `pkg/` prêt à l'emploi.

**Installation** (une seule fois) :

```bash
cargo install wasm-pack
```

**Commande clé** :

```bash
wasm-pack build --target web
```

`--target web` produit un module ES importable **directement** par un navigateur, sans bundler (ni webpack, ni Vite). Les autres cibles existent (`bundler` pour webpack/Vite, `nodejs` pour Node), mais `web` est la plus simple et la plus instructive : on voit exactement ce qui se passe.

**Ce qui est généré** dans `pkg/` (pour une crate `image-filters`) :

```
pkg/
├── image_filters_bg.wasm    ← le binaire WebAssembly (votre Rust compilé)
├── image_filters.js         ← la glue JS : init(), conversions, exports
├── image_filters.d.ts       ← les définitions TypeScript ← !
└── package.json             ← le tout est un paquet npm publiable
```

**Les types TS gratuits.** Ouvrez le `.d.ts` : chaque fonction `#[wasm_bindgen]` y a sa signature TypeScript, dérivée de la signature Rust.

```ts
export function grayscale(pixels: Uint8Array): void;
export function box_blur(pixels: Uint8Array, width: number, height: number, radius: number): Uint8Array;
```

Pour un dev TS, c'est un argument de poids : l'autocomplétion et le contrôle de types fonctionnent immédiatement, et ils sont **garantis exacts** puisque générés depuis la source — pas un fichier `.d.ts` écrit à la main qui dérive avec le temps.

**Servir la page.** Dernier détail pratique : les navigateurs refusent de charger des modules ES et du WASM depuis `file://`. Il faut un serveur HTTP, n'importe lequel :

```bash
python3 -m http.server 8000   # ou : npx serve
```

---

## 5. Manipuler des pixels : le format RGBA du canvas

Notre projet traite des images. Côté navigateur, l'API de référence est le `<canvas>` 2D :

```js
const ctx = canvas.getContext("2d");
const imageData = ctx.getImageData(0, 0, width, height);
imageData.data   // ← Uint8ClampedArray : LES pixels
```

**Définition du format.** `imageData.data` est un tableau d'octets plat où chaque pixel occupe **4 octets consécutifs** : Rouge, Vert, Bleu, Alpha (opacité), chacun de 0 à 255. Les pixels sont rangés ligne par ligne, de haut en bas. Donc :

- une image `width × height` → un buffer de `width × height × 4` octets ;
- le canal `c` (0 = R, 1 = G, 2 = B, 3 = A) du pixel `(x, y)` est à l'index `(y * width + x) * 4 + c`.

**Comparaison Java/TS.** En Java, `BufferedImage.getRGB()` rend des pixels empaquetés en `int` ARGB — même idée, ordre différent. En TS, vous avez peut-être déjà croisé `Uint8ClampedArray` sans creuser : « clamped » signifie que les écritures sont automatiquement bornées dans [0, 255] (`data[i] = 300` écrit 255). Côté Rust, pas de filet équivalent : un `u8` qui déborde panique en mode debug — il faudra borner explicitement (`clamp`), et c'est tout l'objet du filtre de luminosité du projet.

**Et côté Rust ?** Un buffer RGBA est simplement une tranche `&mut [u8]` de longueur multiple de 4. Pas besoin de struct `Pixel`, ni de crate d'imagerie : l'itérateur `chunks_exact_mut(4)` (module 3) découpe la tranche en pixels :

```rust
for pixel in pixels.chunks_exact_mut(4) {
    // pixel : &mut [u8] de longueur 4 → [R, G, B, A]
    pixel[0] = 255 - pixel[0]; // inverse le rouge
}
```

C'est ce qui rend les filtres **testables en natif** : une fonction qui transforme `&mut [u8]` n'a aucune idée qu'un navigateur existe. `cargo test` la valide avec des buffers de 2 ou 3 pixels fabriqués à la main ; le navigateur ne sert qu'à la fin, pour le spectacle.

**Aparté pédagogique : le borrow checker et le flou.** Les filtres « point par point » (gris, inversion, luminosité) modifient chaque pixel indépendamment : `&mut [u8]`, en place, parfait. Un **flou**, lui, lit les pixels *voisins* pour calculer chaque pixel de sortie. Tentons l'écriture en place :

```rust
pub fn box_blur_wrong(pixels: &mut [u8], /* … */) {
    let source = &pixels[..];      // emprunt partagé pour lire les voisins…
    pixels[0] = source[4];         // …et écriture via l'emprunt mutable
}
```

```text
error[E0502]: cannot borrow `*pixels` as mutable because it is also borrowed as immutable
  --> src/lib.rs
   |
   |     let source = &pixels[..];
   |                   ------ immutable borrow occurs here
   |     pixels[0] = source[4];
   |     ^^^^^^^^^   --------- immutable borrow later used here
```

Le compilateur ne fait pas de zèle : écrire dans un buffer **pendant qu'on le lit** rendrait le résultat dépendant de l'ordre de parcours — un pixel déjà flouté contaminerait le calcul de ses voisins. C'est un bug d'algorithmique classique en Java ou en JS, où rien ne vous arrête ; en Rust, c'est une erreur de compilation. La signature correcte sépare lecture et écriture : `fn box_blur(pixels: &[u8], …) -> Vec<u8>`.

---

## 6. Performances : quand le WASM gagne (et quand il perd)

Le module 7 inclut un benchmark JS contre WASM — voici la grille de lecture des résultats.

**Ce qui rend WASM rapide :**
- code machine compilé en avance par rustc/LLVM, avec les optimisations agressives d'un compilateur natif ;
- types fixés à la compilation : pas de JIT qui spécule, pas de déoptimisation ;
- pas de GC : pas de pauses, disposition mémoire compacte et prévisible (un `Vec<u8>` est un bloc contigu, pas un objet avec en-tête).

**Ce qui peut le faire perdre :**
- **le coût de la frontière** : chaque appel copie ses arguments (section 3). Un calcul minuscule appelé très souvent perd : la copie domine ;
- **un adversaire redoutable** : les moteurs JS modernes (V8, SpiderMonkey) JIT-compilent remarquablement bien les boucles numériques chaudes sur `TypedArray`. Sur un filtre trivial — niveaux de gris : trois multiplications par pixel — le JS optimisé fait quasiment jeu égal, et l'écart mesuré tient parfois plus à la copie qu'au calcul.

**La règle empirique** :

> gain WASM ≈ (intensité du calcul par octet transféré) − (coût des copies)

D'où les deux mesures du projet : le **niveau de gris** (calcul léger, écart modeste — parfois nul) et le **box blur** (pour un rayon r, environ (2r + 1)² lectures par pixel : calcul lourd, écart net, qui grandit avec le rayon). Mesurez, ne croyez personne sur parole — pas même ce cours : `performance.now()` est l'arbitre, et les résultats varient d'un navigateur à l'autre.

Dernier point : compilez en **release** pour mesurer (`wasm-pack build` le fait par défaut ; `--dev` est le mode rapide-à-compiler-lent-à-exécuter). Benchmarker un build debug n'a aucun sens — vous l'avez vu au module 4, c'est encore plus vrai ici.

---

## 7. `crate-type = ["cdylib", "rlib"]` expliqué

Dernière pièce du puzzle, dans le `Cargo.toml` :

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

**Définition.** `crate-type` indique à rustc quel(s) artefact(s) produire pour la bibliothèque :

- **`rlib`** (*Rust library*, le défaut) : bibliothèque au format interne de Rust, consommable uniquement par d'autres code Rust. C'est ce que produisaient toutes vos crates `lib` depuis le module 3.
- **`cdylib`** (*C dynamic library*) : bibliothèque dynamique à interface « C », débarrassée de tout le bagage spécifique à Rust, exposant uniquement les symboles publics marqués pour l'export. Sur Linux ça produit un `.so`, sur Windows une `.dll`… et pour la cible `wasm32-unknown-unknown`, un **`.wasm`**. C'est ce format que wasm-bindgen attend.

**Pourquoi les deux ?**

- sans `cdylib` : pas de `.wasm` — `wasm-pack build` échoue ;
- sans `rlib` : la crate n'est plus consommable par du Rust… or les **tests d'intégration** (le dossier `tests/`, module 3) sont précisément un petit programme Rust qui lie votre crate. `cargo test` échouerait à l'édition de liens.

**Comparaison Java/TS.** C'est l'équivalent d'un module publié à la fois pour deux consommateurs : comme un paquet npm qui expédie un build ESM pour les bundlers et un build CJS pour Node, ou un artefact Java utilisé à la fois comme JAR de bibliothèque et comme ressource embarquée. Un même code source, deux emballages.

---

## Récapitulatif avant le projet

1. WASM = cible de compilation binaire, sandboxée, complémentaire de JS : **JS pilote, WASM calcule**.
2. Rust y excelle : pas de GC à embarquer, binaires minuscules, outillage exemplaire.
3. `#[wasm_bindgen]` expose vos fonctions à JS ; la frontière copie chaînes et buffers — peu d'appels gros, jamais beaucoup de petits.
4. `wasm-pack build --target web` génère `.wasm` + glue JS + `.d.ts` (types TS gratuits), importables sans bundler.
5. Les pixels d'un canvas = buffer plat RGBA, soit un honnête `&mut [u8]` côté Rust — donc des filtres testables avec `cargo test`, sans navigateur.
6. Le WASM gagne quand le calcul par octet transféré est lourd ; mesurez avec `performance.now()`.
7. `crate-type = ["cdylib", "rlib"]` : un emballage pour le navigateur, un pour `cargo test`.

Place au projet : `PROJET.md`.
