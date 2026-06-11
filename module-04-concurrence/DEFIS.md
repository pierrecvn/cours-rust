# Défis du module 4

Trois extensions non guidées, par difficulté croissante. Pas d'étapes détaillées cette fois : un objectif, quelques pistes, à vous de jouer. Le corrigé ne les couvre pas — c'est voulu.

---

## Défi 1 — Nombre de workers configurable : trouvez l'optimum

**Objectif.** Ajouter une option `--jobs N` (ou `-j N`) au programme pour choisir le nombre de workers, puis mesurer et tracer le temps d'exécution pour N = 1, 2, 4, 8, 16, 32 sur une grosse arborescence. Trouvez l'optimum sur votre machine, et expliquez-le.

**Pistes.**

- Le parsing d'arguments à la main suffit (`args.iter().position(…)` + `parse::<usize>()`), ou ressortez `clap` du module 3 si vous préférez.
- Mesure honnête : faites d'abord une passe « d'échauffement » non mesurée (pour remplir le cache disque), puis 3 mesures par valeur de N et gardez la médiane.
- Questions à élucider : pourquoi N = 1 en parallèle est-il *plus lent* que la version séquentielle ? (Indice : le coût du channel et des verrous, sans aucun gain.) Pourquoi le gain plafonne-t-il — voire régresse — au-delà d'un certain N ? (Indices : nombre de cœurs, contention sur les deux `Mutex`, et la nature du travail : `metadata` est borné par les entrées/sorties, pas par le CPU.)
- Pour aller plus loin : réduisez la contention en donnant à chaque worker son **propre** `Stats` local, fusionné à la fin avec `Stats::merge` (elle attendait son heure depuis l'étape 3 !). Le worker peut renvoyer son `Stats` comme valeur de retour du thread, récupérée via `join()` — plus aucun `Mutex` autour des stats. Mesurez la différence.

---

## Défi 2 — Barre de progression

**Objectif.** Pendant le scan parallèle, afficher en continu sur une seule ligne le nombre de fichiers traités, mise à jour en temps réel :

```text
Scan en cours… 48 213 fichiers traités
```

**Pistes.**

- Il faut un compteur partagé entre tous les workers, incrémenté à chaque fichier. Un `Arc<Mutex<u64>>` fonctionne, mais c'est l'occasion de découvrir mieux : `std::sync::atomic::AtomicU64`, un entier modifiable **sans verrou** (`fetch_add(1, Ordering::Relaxed)`) et partageable derrière un simple `Arc`. C'est l'équivalent de l'`AtomicLong` de Java.
- Qui affiche ? Un thread dédié qui boucle : lire le compteur, afficher, dormir 100 ms (`thread::sleep`). Pour réécrire sur la même ligne : `print!("\r…")` suivi de `io::stdout().flush().unwrap()` (sans `\n`, la sortie est mise en tampon — sans `flush`, rien ne s'affiche).
- Comment arrêter proprement le thread d'affichage à la fin du scan ? Un `AtomicBool` « terminé » consulté à chaque tour de boucle est la solution la plus simple.
- Effet garanti sur un scan de `~/`.

---

## Défi 3 — Détecteur de doublons

**Objectif.** Ajouter un mode `--duplicates` qui détecte les fichiers au contenu identique et affiche chaque groupe de doublons avec l'espace gaspillé :

```text
=== Doublons ===
2 copies (1.2 Mio gaspillé) :
  ./photos/vacances/IMG_0042.jpg
  ./backup/IMG_0042.jpg
```

**Pistes.**

- Comparer chaque fichier à tous les autres serait quadratique. La stratégie classique en deux temps : (1) grouper les fichiers **par taille** (deux fichiers de tailles différentes ne peuvent pas être identiques — et vous collectez déjà les tailles !) ; (2) pour les groupes d'au moins deux fichiers, calculer une **empreinte du contenu** et comparer les empreintes.
- Pour l'empreinte sans sortir de la bibliothèque standard : lisez le contenu (`fs::read`) et passez-le à un `DefaultHasher` (`std::hash::{DefaultHasher, Hasher}` — `hasher.write(&bytes)` puis `finish()`). Honnêteté intellectuelle : ce hash 64 bits n'est pas cryptographique, des collisions sont théoriquement possibles ; en vrai outil on prendrait la crate `blake3` ou `sha2` (vous savez ajouter une dépendance depuis le module 3), ou on confirmerait par comparaison octet à octet.
- Le hachage est un travail lourd (lecture complète des fichiers !) : parallélisez-le avec la machinerie que vous venez de construire — un channel qui distribue les chemins à hacher, un `Arc<Mutex<HashMap<u64, Vec<PathBuf>>>>` pour collecter les groupes. Vous avez déjà tous les outils.
- Attention aux gros fichiers : `fs::read` charge tout en mémoire. Pour faire mieux, lisez par blocs avec un `BufReader` et nourrissez le hasher bloc par bloc.
