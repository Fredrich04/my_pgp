# `src/domain` — documentation technique

## Contenu actuel

| Fichier | Rôle |
|---|---|
| `mod.rs` | Déclare les modules et réexporte `MyPgpError`, `Result<T>`, `CryptoSystem`, `Mode`, `SymmetricKey`, `RsaCipher` |
| `error.rs` | Type d'erreur du projet (`Display` et `Error` écrits à la main, sans crate) |
| `rsa.rs` | Contrat `RsaCipher` entre la composition PGP (Karyl) et le RSA (Frédrich) — désormais entièrement implémenté |
| `types.rs` | Types partagés : `CryptoSystem`, `Mode`, `SymmetricKey`, `Plaintext`, effacement mémoire `wipe` |

## `mod.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1–3 | Déclare les trois sous-modules `error`, `rsa`, `types`. |
| 5–7 | Réexporte les noms les plus utilisés (`MyPgpError`, `Result`, `RsaCipher`, `CryptoSystem`, `Mode`, `Plaintext`, `SymmetricKey`) pour écrire `crate::domain::Result` au lieu de `crate::domain::error::Result`. |

## `error.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1–3 | Importe `Error`, `fmt` et `io` de la bibliothèque standard (aucune crate). |
| 5 | `Result<T>` (`error.rs:5 Result`) : alias du `Result` standard avec `MyPgpError` ; évite de répéter le type d'erreur partout. |
| 7–8 | `MyPgpError` (`error.rs:8 MyPgpError`) : une variante par cause d'échec (REGLES.md § 12). |
| 9 | `InvalidArguments` : ligne de commande invalide ; porte une raison statique. |
| 10–15 | `InvalidHexLength` (nombre de chiffres impair) et `InvalidHexDigit` (position du caractère fautif, **jamais sa valeur** : un secret ne doit pas fuiter dans un message). |
| 16–18 | `InvalidKeyLength` : clé de taille interdite. |
| 19–22 | `BlockSizeMismatch` : mode bloc, message et clé de tailles différentes (les deux tailles, § 6.3 unités explicites). |
| 23–26 | `CiphertextLengthNotMultiple` : chiffré dont la longueur n'est pas un multiple de la clé. |
| 27–32 | `InputTooLarge` (entrée au-delà de la limite) et `NumberTooLarge` (nombre qui ne tient pas dans le type visé). |
| 33 | `Io` : erreur système enveloppée. |
| 34 | `NotImplemented` : fonction pas encore câblée. |
| 37–71 | `Display` : chaque variante a son message en anglais, sur une ligne, sans couleur (sortie lisible par un humain et par un lecteur d'écran). |
| 73–88 | `Error::source` : seule `Io` a une cause sous-jacente ; toutes les autres variantes sont listées explicitement pour que le compilateur signale toute variante oubliée. |
| 90–94 | `From<io::Error>` : permet d'écrire `?` sur une opération d'E/S. |

## `rsa.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1 | Importe `Result`. |
| 3 | `RsaCipher` (`rsa.rs:3 RsaCipher`) : contrat entre la composition PGP et RSA (voir `explication.md`). |
| 4 | `encrypt` : clé publique (texte `e-n`) et message → chiffré. |
| 5 | `decrypt` : clé privée (texte `d-n`) et chiffré → message. |

## `types.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 3 | Importe `black_box`, utilisé par `wipe`. |
| 5–12 | `CryptoSystem` (`types.rs:6 CryptoSystem`) : les cinq cryptosystèmes du sujet. `Copy` : copie triviale. |
| 14–19 | `Mode` (`types.rs:15 Mode`) : `-c`, `-d` ou `-g`. |
| 21–23 | `SymmetricKey` (`types.rs:21 SymmetricKey`) : clé secrète. Champ privé, donc impossible de la construire sans validation. Pas de `Clone` : pas de copie accidentelle. |
| 26–31 | `new` : refuse une clé vide (`InvalidKeyLength { bytes: 0 }`) ; les tailles propres à chaque algorithme sont contrôlées par l'algorithme. |
| 33–35 | `as_bytes` : lecture seule. |
| 37–43 | `len` et `is_empty` : taille de la clé. |
| 46–50 | `Debug` masqué : n'affiche que la taille. |
| 52–56 | `Drop` : appelle `wipe` pour effacer la clé quand elle disparaît. |
| 58–60 | `Plaintext` (`types.rs:58 Plaintext`) : le message en clair, mêmes protections que la clé. |
| 62–65 | `new` : encapsule des octets (un message vide est autorisé). |
| 67–78 | `as_bytes`, `len`, `is_empty`. |
| 80–90 | `Debug` masqué et `Drop` effaçant. |
| 92 | `wipe` (`types.rs:92 wipe`) : `pub(crate)`, utilisé par tous les modules qui manipulent des secrets. |
| 93 | `fill(0)` : remplit le tampon de zéros. |
| 94 | `black_box` : empêche le compilateur de supprimer l'écriture jugée inutile (sans `unsafe`). |
| 97–111 | Seul test interne : `wipe` n'est pas public, donc inaccessible depuis `tests/`. |

## Historique

### 1.1 — Retrait de l'ancienne architecture (2026-09-24)

Supprimé, car incompatible avec le sujet (`G-CNA-500-my_pgp.pdf`) :

- `types.rs` : types de l'ancien message JSON (`EncryptedMessage`, `Nonce`, `Iv`, `EncryptedKey`, `MessageMetadata`) et clés PEM (`PublicKey`, `PrivateKey`, `KeyPair`, `Signature`) ;
- les modules `src/encryption/` (AES-GCM, RSA-OAEP, CBC via crates) et `src/io/` (fichiers, JSON) ;
- toutes les dépendances sauf `thiserror` : `aes`, `aes-gcm`, `rsa`, `sha2`, `rand`, `pkcs8`, `serde`, `serde_json`, `hex`, `anyhow`, `clap`, `tempfile`, `criterion`.

`thiserror` est conservé : ce n'est pas une crate cryptographique, et REGLES.md § 2.1 recommande son usage.

`src/main.rs` est temporairement réduit à un binaire qui affiche une erreur `NotImplemented` sur stderr et se termine avec le code 84 (exigence du sujet p. 1). Cet état transitoire est remplacé par la CLI à l'étape 5 (REGLES.md § 3.5 : `NotImplemented` ne doit pas devenir permanent).

`error.rs` est inchangé à ce stade ; ses variantes sont réécrites à la sous-étape 1.2.

### 1.2 — Erreurs typées pour le sujet (2026-09-24)

`MyPgpError` ne contient plus que les erreurs que la partie de Karyl peut produire :

| Variante | Charge utile | Déclencheur |
|---|---|---|
| `InvalidArguments` | `&'static str` | Ligne de commande incorrecte (nombre d'arguments, mode inconnu…) |
| `InvalidHexLength` | `digits` | Chaîne hexadécimale de longueur impaire |
| `InvalidHexDigit` | `position` | Caractère non hexadécimal |
| `InvalidKeyLength` | `bytes` | Clé de taille non acceptée par l'algorithme |
| `BlockSizeMismatch` | `message_bytes`, `key_bytes` | Option `-b` avec un message et une clé de tailles différentes |
| `InputTooLarge` | `limit_bytes` | Entrée standard au-delà de la limite |
| `Io` | `std::io::Error` (`#[from]`) | Lecture de stdin / écriture de stdout |
| `NotImplemented` | `&'static str` | Variante obsolète (step 5), plus jamais utilisée dans le code |

Choix de conception :

- **Aucun secret dans les erreurs (REGLES.md § 4.4, § 27.3)** : les variantes ne transportent que des tailles et des positions, jamais d'octets de clé ou de message. Les textes libres sont des `&'static str` : il est donc impossible d'y injecter à l'exécution une valeur issue de l'entrée.
- **Pas de variante fourre-tout ni de chevauchement (§ 3.3, § 3.4)** : chaque variante correspond à une cause unique. Les variantes de l'ancienne architecture (`Encryption`, `Decryption`, `Key`, `Serialization`, `Deserialization`, `Signature`, `InvalidSignature`, `Validation`, `InvalidKeyFormat`) sont supprimées, ainsi que les constructeurs utilitaires qui fabriquaient des `String`.
- **Cause conservée (§ 3.2)** : `Io` garde l'erreur d'origine, accessible par `source()`.
- **Code de sortie** : toutes les variantes aboutissent au code 84 (dérogation D7, REGLES.md § 37), mais restent distinctes dans le code et dans le message.
- Les variantes propres au RSA (`NumberTooLarge`, `CiphertextLengthNotMultiple`) ont été ajoutées par Frédrich ; `NotImplemented` n'est plus construite nulle part (conservée uniquement pour compatibilité du type).

Tests : affichage de chaque famille d'erreur (tailles et positions uniquement) et conservation de la source d'une erreur d'E/S.

### 1.3 — Types partagés (2026-09-24)

| Type | Définition | Rôle |
|---|---|---|
| `CryptoSystem` | `Xor`, `Aes`, `Rsa`, `PgpXor`, `PgpAes` | Premier argument de la CLI (REGLES.md § 2.4 : enum plutôt que `String`) |
| `Mode` | `Cipher`, `Decipher`, `Generate` | Options `-c`, `-d`, `-g` |
| `SymmetricKey` | `Vec<u8>` privé, non vide | Clé XOR ou AES, et partie symétrique d'une clé PGP |

`SymmetricKey` applique les règles de protection des secrets :

- **§ 4.1** : `Debug` écrit uniquement la taille (`SymmetricKey(<4 bytes, redacted>)`) ;
- **§ 4.2, § 4.3** : ni `Clone` ni `Serialize` ;
- **§ 1.3** : une clé vide ne peut pas être construite (`InvalidKeyLength { bytes: 0 }`). Les tailles propres à chaque algorithme (16/24/32 octets pour AES) sont validées par l'algorithme lui-même ;
- **§ 20.2** : les octets sont effacés à la destruction (`Drop`).

#### Effacement mémoire sans crate ni `unsafe`

REGLES.md § 20.2 suggère la crate `zeroize`. Le projet n'a aucune dépendance et, depuis le 2026-09-24, **aucun bloc `unsafe`** : l'effacement est codé à la main dans `wipe`, en Rust entièrement sûr :

- `buffer.fill(0)` remet chaque octet à zéro ;
- `std::hint::black_box(buffer)` présente ensuite le tampon au compilateur comme « lu par un code inconnu » : l'optimiseur ne peut plus considérer la mise à zéro comme inutile et, en pratique, ne la supprime pas.

Limite assumée (REGLES.md § 37.2 bis) : la documentation de Rust décrit `black_box` comme un effort maximal, **pas une garantie formelle**. L'ancienne version (`std::ptr::write_volatile`) offrait cette garantie mais exigeait un bloc `unsafe` ; l'équipe a choisi la sûreté du langage. Comme avant, seul le tampon final est effacé : les copies intermédiaires (chaîne hexadécimale d'origine, réallocations) doivent être évitées ou effacées par le code qui les crée (§ 20.3).

`wipe` est `pub(crate)` pour être réutilisée par les autres modules (état interne d'AES, blocs temporaires).

Tests : refus d'une clé vide, accès aux octets, `Debug` masqué, effacement effectif d'un tampon, distinction des variantes.

### 1.4 — Contrat RSA pour la composition PGP (2026-09-24)

```rust
pub trait RsaCipher {
    fn encrypt(&self, key: &str, message: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, key: &str, ciphertext: &[u8]) -> Result<Vec<u8>>;
}
```

Contrat (défini par Karyl, implémenté par Frédrich) :

- `key` est la partie RSA de l'argument PGP, telle que tapée : `e-n` pour chiffrer, `d-n` pour déchiffrer (sujet p. 7-8). **L'analyse de la clé appartient à B** ; une clé mal formée produit une `MyPgpError` (variantes RSA ajoutées par B).
- `message` et `ciphertext` sont des octets **little-endian** : l'octet d'indice 0 est le poids faible du nombre (sujet p. 4). A convertit ces octets en hexadécimal pour l'affichage.
- Le trait est utilisable en objet (`&dyn RsaCipher`) : la composition PGP ne dépend pas de l'implémentation concrète, ce qui permet de la tester avant que le RSA de B n'existe.

Écart assumé à REGLES.md § 2.4 : la clé reste un `&str` plutôt qu'un type dédié, car son format et sa validation relèvent de Frédrich ; Frédrich pourra introduire un type `RsaKey` et faire évoluer ce contrat d'un commun accord.

Tests : un chiffreur factice (inversion des octets) vérifie l'usage via `&dyn RsaCipher`, l'aller-retour et la propagation des erreurs.

### Remplacement de `thiserror` (2026-09-24)

À la demande de l'équipe, `thiserror` est retirée : le projet n'a plus **aucune** dépendance externe. `error.rs` implémente à la main ce que la macro générait :

- `impl fmt::Display` : un `match` exhaustif produit exactement les mêmes messages qu'avant (les tests d'affichage existants passent sans modification) ;
- `impl std::error::Error` avec `source()` : seule `Io` a une cause ; les autres variantes sont énumérées explicitement (pas de `_`), pour que toute nouvelle variante oblige à décider de sa source ;
- `impl From<io::Error>` : remplace l'attribut `#[from]` et permet toujours l'opérateur `?`.

REGLES.md § 2.1 est amendée en conséquence (et § 37.2 bis).

Tests ajoutés : absence de source pour une erreur non-E/S, affichage de `NotImplemented`.

`lib.rs` et `main.rs` portent `#![forbid(unsafe_code)]` : tout bloc `unsafe` ajouté plus tard fera échouer la compilation.

Depuis le 2026-09-24, les tests publics de ce dossier vivent dans `tests/unit/{error,rsa,types}.rs` (REGLES.md § 38) ; seul le test de `wipe`, fonction non publique, reste dans `types.rs`.

### Audit 2.5 — Unités explicites (2026-09-24, REGLES.md § 6.3)

Les champs des erreurs portent désormais leur unité dans leur nom : `InvalidHexLength { digits }`, `InvalidKeyLength { bytes }`, `BlockSizeMismatch { message_bytes, key_bytes }`, `InputTooLarge { limit_bytes }` (et `NumberTooLarge { max_bytes }`, déjà conforme). Les messages affichés sont inchangés.

### Audit 2.5 — Type `Plaintext` (2026-09-24, REGLES.md § 4.5, § 20.1)

`Plaintext` enveloppe le message lu sur l'entrée standard (et, plus tard, le message déchiffré), avec les mêmes protections que `SymmetricKey` : `Debug` réduit à la taille (`Plaintext(<6 bytes, redacted>)`), ni `Clone` ni `Serialize`, effacement à la destruction par `wipe`. Contrairement à une clé, un message vide est valide (`new` ne peut pas échouer).

En mode déchiffrement, l'entrée standard contient un message chiffré en hexadécimal, qui n'est pas secret ; `read_message` renvoie pourtant toujours un `Plaintext` : traiter toute entrée comme sensible évite une API à deux types pour une même lecture, au prix d'un nom imparfait dans ce cas (décision du 2026-09-24).

### Intégration RSA + PGP (2026-09-30)

Choix de correspondance des erreurs venues du RSA, qui ne peut pas construire de `&'static str` dynamiques :

- `keygen` renvoie `Result<KeyPair, String>` : l'erreur est convertie par `map_err(|_| InvalidArguments("key generation failed"))`. On perd le détail (P et Q égaux, exposant introuvable) au profit d'un message statique, imposé par REGLES.md § 4.4 (aucune donnée d'entrée dans les erreurs) et par le type `&'static str` ;
- `BigUint::from_hex_le` renvoie `HexError` : converti vers `MyPgpError` via `decode_hex` + `from_bytes_le`, pour réutiliser les variantes `InvalidHexLength`/`InvalidHexDigit` (pas de chevauchement, § 3.4) ;
- message ou chiffré `>= n` : `NumberTooLarge { max_bytes }` (taille du module), jamais le contenu ;
- `RsaCipher` est implémenté par `KeyPair`, dont les méthodes délèguent à `encrypt_message`/`decrypt_message` (`src/rsa/cipher.rs`).

Tests : vecteurs sujet (`d3`/`e3`, 256-bit, `WF` ↔ `8f84`), keygen 1024-bit, roundtrip 2048-bit, et cas limites (`m >= n`, `p == q`, aucun Fermat valide).
