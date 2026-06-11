# Défis du module 5

Trois extensions non guidées, par difficulté croissante. Pas d'étapes détaillées cette fois : un objectif, quelques pistes, à vous de jouer. Le corrigé ne les couvre pas — c'est voulu.

> Travaillez dans `projet/` (votre code), à partir de l'API qui fait passer les 10 tests. Gardez ces tests verts : ils protègent le contrat que le module 6 attend.

---

## Défi 1 — Pagination et recherche

**Objectif.** Enrichir `GET /notes` avec des paramètres de *query string* :

- `GET /notes?limit=10&offset=20` : ne renvoyer qu'une tranche de notes (pagination) ;
- `GET /notes?q=pain` : ne renvoyer que les notes dont le titre ou le contenu contient `pain` ;
- les trois doivent se combiner : `GET /notes?q=pain&limit=5`.

`GET /notes` sans paramètre doit continuer à tout renvoyer (les tests existants doivent rester verts).

**Pistes.**

- L'extracteur pour la query string est `Query<T>` (`axum::extract::Query`). Définissez une struct dédiée et désérialisez-la, exactement comme `Json` le fait pour le corps :

  ```rust
  #[derive(Debug, Deserialize)]
  struct ListParams {
      q: Option<String>,
      limit: Option<i64>,
      offset: Option<i64>,
  }
  ```

  Les champs en `Option` rendent chaque paramètre facultatif : absent → `None`. Ajoutez `Query(params): Query<ListParams>` aux paramètres de `list_notes`.
- Côté SQL, deux approches. La plus simple sans construire de chaîne dynamique : exploitez le fait qu'un `LIMIT` négatif signifie « pas de limite » en SQLite. Une requête unique avec un `WHERE` paramétré sur `q` et un `LIMIT ? OFFSET ?` couvre tous les cas :

  ```sql
  SELECT id, title, content, created_at FROM notes
  WHERE (?1 IS NULL OR title LIKE '%' || ?1 || '%' OR content LIKE '%' || ?1 || '%')
  ORDER BY id
  LIMIT ?2 OFFSET ?3
  ```

  Liez `q` (un `Option<String>`, que `.bind` accepte tel quel : `None` devient `NULL`), puis `limit.unwrap_or(-1)` et `offset.unwrap_or(0)`. Réfléchissez au piège : un `LIKE '%pain%'` traite `%` et `_` comme des jokers — pour ce défi, on l'accepte ; un vrai moteur de recherche échapperait ces caractères.
- **Honnêteté intellectuelle.** `LIKE '%terme%'` ne peut pas utiliser d'index : c'est correct pour des dizaines de milliers de notes, pas pour des millions. La vraie solution serait l'extension *full-text search* de SQLite (`FTS5`). Mentionnez-le si vous documentez votre API.
- Écrivez deux ou trois tests à vous (sur le modèle de `tests/api.rs`) : recherche qui filtre, `limit` qui tronque, `q` inconnu qui renvoie `[]`.

---

## Défi 2 — Journalisation des requêtes (middleware)

**Objectif.** Voir dans la console, pour chaque requête, sa méthode, son chemin, son code de statut et sa durée :

```text
2026-06-11T14:30:12Z  GET    /notes        200   1.2ms
2026-06-11T14:30:15Z  POST   /notes        201   3.8ms
2026-06-11T14:30:18Z  GET    /notes/999    404   0.4ms
```

**Pistes.**

- C'est un **middleware**, l'équivalent du `app.use(...)` d'Express ou d'un `Filter` de servlet en Java. En Axum, les middlewares viennent de l'écosystème **tower** : on les ajoute au routeur avec `.layer(...)`.
- La voie idiomatique et recommandée passe par deux crates :

  ```toml
  tower-http = { version = "0.6", features = ["trace"] }
  tracing = "0.1"
  tracing-subscriber = "0.3"
  ```

  Au démarrage (dans `main`), initialisez l'abonné : `tracing_subscriber::fmt::init();`. Puis ajoutez la couche au routeur : `app.layer(tower_http::trace::TraceLayer::new_for_http())`. `TraceLayer` journalise déjà méthode, chemin, statut et durée — réglez le niveau via la variable d'environnement `RUST_LOG=tower_http=debug,notes_api=debug`.
- **Variante « à la main » (plus formatrice).** Écrivez votre propre middleware avec `axum::middleware::from_fn`. La fonction reçoit la `Request` et un `Next`, mesure le temps autour de `next.run(request).await`, et `println!` le résultat :

  ```rust
  async fn log_requests(request: Request, next: Next) -> Response {
      let method = request.method().clone();
      let path = request.uri().path().to_owned();
      let start = std::time::Instant::now();
      let response = next.run(request).await;   // exécute le handler
      let elapsed = start.elapsed();
      println!("{method:<6} {path:<14} {} {elapsed:.1?}", response.status().as_u16());
      response
  }
  ```

  Branchez-la avec `app.layer(axum::middleware::from_fn(log_requests))`. C'est l'occasion de voir qu'un middleware n'a rien de magique : c'est une fonction qui enveloppe l'appel au handler, avant/après.
- **Question à élucider.** Pourquoi mesurer la durée *autour* de `next.run(...)` capture-t-il bien le temps du handler ? Et où placeriez-vous le `.layer(...)` pour qu'il englobe **toutes** les routes ? (Indice : sur le routeur final, après les `.route(...)`.)

---

## Défi 3 — Authentification par token

**Objectif.** Protéger les routes qui modifient des données. Les requêtes `POST`, `PUT` et `DELETE` doivent fournir un en-tête `Authorization: Bearer <token>` valide, sinon l'API répond `401 Unauthorized`. Les lectures (`GET`) restent publiques.

**Pistes.**

- Le token attendu se lit dans une variable d'environnement au démarrage : `let token = std::env::var("API_TOKEN").unwrap_or_else(|_| "secret-dev".to_string());`. **Ne le codez jamais en dur** dans le binaire.
- Ajoutez une variante `Unauthorized` à votre enum `ApiError` et mappez-la vers `StatusCode::UNAUTHORIZED` dans `IntoResponse`. Le `match` exhaustif vous **forcera** à traiter le nouveau cas : c'est exactement le filet de sécurité décrit dans COURS.md §8.2 — appréciez qu'il vous empêche d'oublier.
- Deux façons d'imposer le contrôle :
  - un **middleware** (réutilisez `from_fn` du défi 2) qui inspecte l'en-tête `Authorization` et répond `401` avant même d'atteindre le handler, posé en `.layer(...)` sur un sous-routeur ne contenant que les routes à protéger ;
  - ou un **extracteur maison** : un type `AuthToken` qui implémente `FromRequestParts` et lit/valide l'en-tête. Ajouté en paramètre d'un handler, il garantit au niveau des types que la route est protégée. C'est plus avancé, mais c'est la vraie démonstration de la « magie des extracteurs » du COURS.md §4.4 : vous en écrivez un vous-même.
- Pour lire l'en-tête depuis la requête : `request.headers().get(axum::http::header::AUTHORIZATION)`. Pensez à comparer en retirant le préfixe `"Bearer "`.
- **Sécurité, l'honnêteté de rigueur.** Un token statique partagé est le strict minimum pédagogique. En production : des tokens par utilisateur, expirables (JWT), transmis **uniquement** sur HTTPS, et une comparaison à temps constant pour éviter les attaques temporelles (la crate `subtle`). Documentez ces limites — un junior qui sait *ce qui manque* vaut mieux qu'un junior qui croit avoir « sécurisé » son API.
- Vérifiez par `curl` : un `POST` sans en-tête → `401` ; avec `-H 'Authorization: Bearer secret-dev'` → `201` ; un `GET` sans en-tête → toujours `200`.
