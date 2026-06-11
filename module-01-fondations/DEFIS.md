# Défis du module 1

Trois extensions **non guidées** : pas d'étapes, pas de squelette — juste un objectif, des critères de réussite et quelques pistes. C'est volontairement plus difficile que le projet : tu vas devoir chercher, te tromper, relire le cours et la documentation. C'est exactement le but.

> Conseil : commence chaque défi par « comment je le testerais ? ». Une fonction pure + un test, comme à l'étape 8, c'est la bonne hygiène.

---

## Défi 1 — Essais limités et rejouabilité

**Objectif.** Transformer le jeu du nombre mystère en vrai jeu d'arcade :

- le joueur n'a droit qu'à **7 essais** (déclare une `const MAX_ATTEMPTS: u32 = 7;`) ;
- à chaque tour, afficher le nombre d'essais restants ;
- à la défaite, révéler le nombre secret ;
- victoire ou défaite, proposer **« Rejouer ? (o/n) »** et relancer une partie complète si le joueur accepte (nouveau secret, compteur remis à zéro).

**Critères de réussite.**
- Perdre au 7ᵉ essai raté affiche bien le secret, sans 8ᵉ proposition possible.
- Une entrée invalide ne consomme toujours pas d'essai.
- Répondre `o` relance une partie avec un **nouveau** secret ; toute autre réponse quitte poliment.

**Pistes.** Deux boucles imbriquées (les parties, les tours d'une partie) — ou mieux : extrais une fonction `fn play_one_game()` et appelle-la dans une boucle depuis `main`. Si tu veux qu'elle signale le résultat, fais-lui retourner un `bool` (gagné/perdu).

---

## Défi 2 — Historique des parties

**Objectif.** À la fin de chaque partie (dans la version « rejouable » du défi 1), enregistrer le score, puis afficher un récapitulatif quand le joueur quitte :

```text
=== Récapitulatif ===
Partie 1 : gagnée en 5 essais
Partie 2 : perdue (le nombre était 73)
Partie 3 : gagnée en 7 essais
Meilleur score : 5 essais
```

**Critères de réussite.**
- L'historique survit d'une partie à l'autre et s'affiche dans l'ordre.
- Le meilleur score (minimum d'essais parmi les parties gagnées) est correct ; s'il n'y a aucune victoire, l'afficher honnêtement (« aucune partie gagnée »).

**Pistes.** Il te faut une collection qui grandit : `Vec<T>` — le `ArrayList`/`Array` de Rust, vu en détail au module 2 mais tout à fait abordable dès maintenant : `let mut history: Vec<u32> = Vec::new();`, `history.push(attempts);`, puis `for score in &history { ... }` (pourquoi ce `&` ? Relis le cours § 8.3 : sans lui, la boucle *déménagerait* le Vec). Pour le meilleur score : `history.iter().min()` retourne… un `Option` — débrouille-toi avec un `match`, ou cherche `if let` dans la documentation. Défi dans le défi : stocker aussi les parties perdues (un `Vec<String>` de lignes toutes faites est une solution honnête à ce stade).

---

## Défi 3 — Le convertisseur prend du galon

**Objectif.** Enrichir le convertisseur avec **deux nouvelles familles d'unités**, *tests d'abord* :

- **Poids** : kilogrammes ↔ livres (1 lb = 0,45359237 kg exactement) ;
- **Température bonus** : Celsius ↔ Kelvin (K = C + 273,15 ; bonus : refuser les températures sous le zéro absolu, −273,15 °C).

**Critères de réussite.**
- Chaque nouvelle fonction de conversion est **pure**, documentée (`///`), et couverte par au moins **deux tests** écrits *avant* l'implémentation (pense aux allers-retours : `kg → lb → kg` doit rendre la valeur initiale).
- Le menu propose les nouvelles conversions et reste robuste aux entrées invalides.
- `cargo test --bin converter` : tout passe ; `cargo clippy` : aucun avertissement.

**Pistes.** Copie le motif existant : une `const` pour le facteur, des fonctions miroir, des tests `assert_close`. Pour le zéro absolu, la version simple retourne la valeur en l'affublant d'un avertissement ; la version élégante retourne `Option<f64>` (`None` si la température est physiquement impossible) — et là, tu toucheras du doigt le sujet du module 2.

---

Quand les trois défis passent `cargo test` et `cargo clippy` sans broncher, tu es prêt pour le **module 2 — Structurer son code**.
