# `tests` — documentation technique

Organisation imposée par REGLES.md § 38 : un dossier par type de test, chacun étant une crate de test Cargo autonome (`tests/<type>/main.rs`), découverte automatiquement par `cargo test`. Aucun wrapper `include!`.

| Dossier | Contenu actuel |
|---|---|
| `unit/` | Un module par fichier source : `aes_block.rs`, `aes_key_schedule.rs`, `aes_round.rs`, `aes_sbox.rs`, `aes_stream.rs`, `error.rs`, `hex.rs`, `little_endian.rs`, `rsa.rs`, `stdin.rs`, `types.rs`, `version.rs`, `xor.rs` |
| `integration/` | Codec hex + `SymmetricKey` |
| `static/` | Analyse du code source : dans `src/`, pas de `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` avant `#[cfg(test)]` ; aucun commentaire `//` ou `/*` dans `src/` **et** `tests/` ; aucun `.unwrap()`/`.expect(` dans `tests/` ; chaque `#[test]` nommé `test_…` ; aucun `unsafe` et `#![forbid(unsafe_code)]` en tête de `lib.rs`/`main.rs` ; aucune dépendance externe ; `code.md` + `explication.md` dans chaque sous-dossier de `src/` |
| `regression/` | Valeurs figées |
| `e2e/` | Binaire `my_pgp` lancé via `CARGO_BIN_EXE_my_pgp` : vecteurs exacts du PDF (XOR p. 4, AES p. 5, RSA et PGP-AES), round-trips, codes d'erreur 84 |
| `acceptance/` | Exigences du sujet, avec leur page |
| `performance/` | Seuils de temps chiffrés |
| `security/` | Fuites, entrées hostiles |
| `usability/` | Clarté des messages |
| `accessibility/` | Sorties ASCII brutes, sans couleur |

Les tests internes (`#[cfg(test)]` dans `src/`) sont réservés aux fonctions non publiques, inaccessibles depuis `tests/`. Il n'en reste qu'un : `wipe` (`pub(crate)`) dans `src/domain/types.rs`. Tous les autres tests internes ont été déplacés dans `tests/unit/` le 2026-09-24.

Conformément à § 12.8, aucun test n'utilise `unwrap()` ou `expect()` : ils renvoient `Result`.

## Conventions (audit 2.5 du 2026-09-24)

- **Nommage (REGLES.md § 34.1)** : chaque test suit `test_<action>_<résultat_attendu>`, vérifié par `tests/static/`.
- **Messages d'assertion (§ 34.2)** : chaque `assert!`, `assert_eq!` et `assert_ne!` porte un message qui explique la règle vérifiée, cite REGLES.md ou le sujet quand c'est pertinent et affiche la valeur obtenue (`got {:?}`) lorsque la macro ne le fait pas déjà. Les assertions répétées passent par de petites fonctions d'aide (`assert_display`, `assert_reads`, `assert_no_leak`, `assert_screen_reader_friendly`, `assert_fast`…).
- **Pas de test qui passe à vide** : un test qui attend une erreur vérifie d'abord `is_err()` avant d'inspecter le message ; les helpers `error_text` renvoient `<unexpected success: …>` si l'opération réussit, ce qui fait échouer l'assertion suivante.
