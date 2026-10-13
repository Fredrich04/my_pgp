# `src/symmetric` — documentation technique

## Contenu

| Fichier | Rôle |
|---|---|
| `mod.rs` | Déclare les algorithmes symétriques |
| `xor.rs` | XOR en mode bloc et en mode flux |

## `mod.rs` — ligne par ligne

| Ligne | Code | Explication |
|---|---|---|
| 1 | `pub mod aes;` | Rend publique la sous-arborescence `aes/` (FIPS-197), accessible depuis la CLI. |
| 2 | `pub mod xor;` | Rend public `xor.rs`. |

## `xor.rs` — ligne par ligne

Constantes, aucune. Les deux premières lignes importent les types communs : `MyPgpError`/`Result` (erreurs du projet) et `Plaintext`/`SymmetricKey` (types sécurisés qui s'effacent de la mémoire).

### `xor_block_cipher` (`xor.rs:4 xor_block_cipher`)

| Ligne | Explication |
|---|---|
| 4 | Signature : prend le message (`&Plaintext`) et la clé (`&SymmetricKey`), rend les octets chiffrés (`Vec<u8>`, non secrets). |
| 5 | Vérifie **avant tout calcul** que message et clé ont la même taille (mode `-b`, sujet p. 3) ; `?` propage `BlockSizeMismatch`. |
| 6 | `Ok(` : le résultat est construit comme une expression, sans variable intermédiaire. |
| 7 | `message.as_bytes()` : accès en lecture seule aux octets, sans copie. |
| 8 | `.iter()` : parcourt les octets du message. |
| 9 | `.rev()` : **inverse l'ordre du message** (convention du sujet, voir ci-dessous). |
| 10 | `.zip(key.as_bytes())` : apparie le i-ème octet (inversé) du message au i-ème octet de la clé. |
| 11 | `.map(... message_byte ^ key_byte)` : XOR bit à bit de chaque paire. |
| 12 | `.collect()` : rassemble les octets dans un `Vec<u8>`. |

### `xor_block_decipher` — (`xor.rs:15 xor_block_decipher`)

| Ligne | Explication |
|---|---|
| 15 | Prend les octets chiffrés (`&[u8]`, issus de `decode_hex`) et la clé ; rend un `Plaintext`. |
| 16 | Même contrôle de taille qu'au chiffrement. |
| 17–21 | XOR de chaque octet chiffré avec l'octet de clé correspondant (le XOR est son propre inverse). Le résultat est un `Vec<u8>` modifiable. |
| 22 | `message.reverse()` : remet le message dans son ordre de lecture, inverse de la ligne 9. |
| 23 | Enveloppe le résultat dans un `Plaintext` (déplacé, sans copie) pour qu'il soit effacé à sa destruction. |

### `ensure_same_size` — (`xor.rs:26 ensure_same_size`)

| Ligne | Explication |
|---|---|
| 26 | Fonction privée, partagée par le chiffrement et le déchiffrement bloc. |
| 27 | Compare les deux tailles. |
| 28–31 | Si elles diffèrent, retourne `BlockSizeMismatch` avec les **deux** tailles (message d'erreur utile à l'utilisateur). |
| 33 | Sinon `Ok(())`. |

### `xor_stream_cipher` — (`xor.rs:36 xor_stream_cipher`)

| Ligne | Explication |
|---|---|
| 37 | La taille de bloc est la taille de la clé. |
| 38 | Pré-alloue le tampon de sortie à sa taille finale (nombre de blocs arrondi au-dessus × taille de clé) : pas de réallocation, donc pas de copie de données dans la mémoire. |
| 39 | `chunks(key_bytes)` : découpe le message dans l'ordre de lecture ; le dernier morceau peut être plus court. |
| 40 | Copie le morceau dans un `Vec` modifiable. |
| 41 | `resize(key_bytes, 0)` : complète le dernier morceau avec des **zéros à sa fin**. |
| 42 | Enveloppe le bloc dans un `Plaintext` temporaire, effacé à la fin du tour de boucle. |
| 43 | Chiffre le bloc comme en mode `-b` (donc avec inversion interne) et ajoute le résultat à la sortie. |
| 45 | Rend la concaténation des blocs chiffrés. |

### `xor_stream_decipher` — (`xor.rs:48 xor_stream_decipher`)

| Ligne | Explication |
|---|---|
| 49 | Taille de bloc = taille de la clé. |
| 50–55 | Refuse (`CiphertextLengthNotMultiple`) un chiffré dont la longueur n'est pas un multiple de la clé : il a été coupé ou abîmé. |
| 56 | Pré-alloue le message à sa taille finale. |
| 57 | `chunks_exact` : découpe en blocs complets de la taille de la clé (aucun reste possible grâce au contrôle ligne 50). |
| 58 | Déchiffre chaque bloc et ajoute les octets ; les zéros de complétion sont **conservés** (décision de l'équipe). |
| 60 | Enveloppe l'ensemble dans un `Plaintext`. |

## Historique

### 3.1 — XOR en mode bloc (2026-09-24)

#### Convention d'ordre des octets, déduite du vecteur du sujet (p. 4)

Le sujet précise que « la clé et la sortie chiffrée sont des nombres hexadécimaux », écrits en little-endian (p. 4). Le vecteur officiel le confirme octet par octet :

| Octet de sortie | Valeur | Calcul |
|---|---|---|
| premier | `20` | `'w'` (0x77, **dernier** caractère du message) ⊕ `0x57` (**premier** octet de la clé) |
| dernier | `3c` | `'Y'` (0x59, **premier** caractère du message) ⊕ `0x65` (**dernier** octet de la clé) |

Le message est donc lu comme un nombre écrit naturellement (premier caractère = poids fort), tandis que la clé et la sortie sont des nombres en hexadécimal little-endian. En octets :

```text
chiffrement   : sortie[i]  = message[n-1-i] ⊕ clé[i]
déchiffrement : message[n-1-i] = entrée[i] ⊕ clé[i]
```

`decode_hex`/`encode_hex` restent neutres (ordre du texte) ; l'inversion est faite ici, par l'algorithme, parce qu'elle découle de la définition du XOR sur des nombres.

#### API

| Fonction | Entrées | Sortie |
|---|---|---|
| `xor_block_cipher(message, key)` | `&Plaintext`, `&SymmetricKey` | `Result<Vec<u8>>` : octets à afficher avec `encode_hex` |
| `xor_block_decipher(ciphertext, key)` | `&[u8]` (issu de `decode_hex`), `&SymmetricKey` | `Result<Plaintext>` |

- **Taille (sujet p. 3, option `-b`)** : message et clé doivent avoir la même taille ; sinon `BlockSizeMismatch { message_bytes, key_bytes }`, vérifié **avant** tout calcul (§ 17, § 27.2). Un message vide est donc refusé (une clé n'est jamais vide).
- **Secrets (§ 4, § 20)** : le message est reçu et rendu sous forme de `Plaintext` (effacé à la destruction). Le vecteur intermédiaire du déchiffrement est inversé sur place puis déplacé dans le `Plaintext`, sans copie.
- Le texte chiffré est un `Vec<u8>` ordinaire : il n'est pas secret.
- Aucune dérogation nouvelle : le XOR simple n'offre ni intégrité ni sécurité si la clé est réutilisée, ce que le sujet reconnaît lui-même (p. 4) ; c'est une exigence du sujet couverte par l'esprit de D1 (§ 37).

#### Tests (REGLES.md § 38)

| Type | Couverture |
|---|---|
| Unitaire | Inversion + XOR, déchiffrement, bloc d'un octet, clé nulle, message plus court / plus long / vide, taille incorrecte au déchiffrement |
| Intégration | stdin → XOR `-b` → hex → XOR `-b` |
| Statique | Règles existantes (documentation de ce dossier vérifiée) |
| Régression | Trois vecteurs figés |
| E2E | **Sans objet à ce stade** : la CLI n'existe pas encore (étape 5) ; les vecteurs du sujet seront rejoués sur le binaire |
| Acceptation | Vecteur officiel « You know nothing, Jon Snow », chiffrement et déchiffrement |
| Performance | Aller-retour d'un bloc de 1 Mio en moins de 2 s |
| Sécurité | Erreur de taille sans fuite ; `Debug` du message déchiffré masqué |
| Usabilité | Message d'erreur donnant les deux tailles |
| Accessibilité | Sortie et erreur en ASCII sur une ligne |

### 3.2 — XOR en mode flux (2026-09-24)

Décisions de l'équipe du 2026-09-24 (le sujet p. 4 ne donne pas de vecteur pour le mode flux) :

- **Bloc par bloc** : le message est découpé **dans l'ordre de lecture** en blocs de la taille de la clé ; le dernier bloc est complété par des zéros **à sa fin** ; chaque bloc passe par `xor_block_cipher` (donc est inversé à l'intérieur du bloc) ; les sorties sont concaténées dans l'ordre. Un message d'exactement un bloc donne donc la même sortie qu'avec `-b`, ce que vérifie le test d'acceptation sur le vecteur officiel.
- **Zéros conservés** : au déchiffrement, les zéros de complétion ne sont pas retirés (décision antérieure, § 0 du plan).
- **Longueur invalide** : un texte chiffré dont la longueur n'est pas un multiple de la taille de la clé est refusé avec `CiphertextLengthNotMultiple { ciphertext_bytes, key_bytes }` (nouvelle variante du fichier commun `src/domain/error.rs`, à relire par Frédrich), avant tout calcul (§ 17, § 27.2).
- **Message vide** : aucun bloc, sortie vide (et inversement au déchiffrement).

| Fonction | Entrées | Sortie |
|---|---|---|
| `xor_stream_cipher(message, key)` | `&Plaintext`, `&SymmetricKey` | `Result<Vec<u8>>`, longueur = multiple de la taille de la clé |
| `xor_stream_decipher(ciphertext, key)` | `&[u8]`, `&SymmetricKey` | `Result<Plaintext>` |

Secrets (§ 20.1, § 20.3) : chaque bloc de message complété est enveloppé dans un `Plaintext` temporaire, effacé dès la fin de son tour de boucle ; le tampon du message déchiffré est pré-alloué à sa taille finale, sans réallocation, et chaque bloc intermédiaire est lui aussi un `Plaintext` effacé.

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | Un bloc = mode `-b`, ordre des blocs et complétion, message vide, zéros conservés, entrée vide, longueur non multiple, aller-retour exact |
| Intégration | stdin → flux → hex → flux (zéros visibles) |
| Statique | Règles existantes |
| Régression | Trois vecteurs figés |
| E2E | **Sans objet à ce stade** (CLI à l'étape 5) |
| Acceptation | Vecteur officiel identique en flux ; complétion d'un message de 5 octets en un bloc de 8 |
| Performance | Aller-retour de 1 Mio avec une clé de 16 octets en moins de 2 s |
| Sécurité | Erreur de longueur sans fuite ; toutes les tailles de clé de 1 à n+3 |
| Usabilité | Message donnant les deux tailles |
| Accessibilité | Sortie et erreur en ASCII sur une ligne |
