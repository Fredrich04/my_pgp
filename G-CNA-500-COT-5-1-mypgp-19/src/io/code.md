# `src/io` — documentation technique

## Contenu

| Fichier | Rôle |
|---|---|
| `mod.rs` | Déclare les sous-modules |
| `little_endian.rs` | Nombres `u128` ↔ octets little-endian |
| `stdin.rs` | Lecture bornée du message (`read_message`, `MAX_INPUT_BYTES`) |
| `hex.rs` | Conversions hexadécimal ↔ octets (`decode_hex`, `encode_hex`) |

## `mod.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1–3 | `pub mod hex`, `pub mod little_endian`, `pub mod stdin` : trois sous-modules publics, un par fichier. |

## `hex.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1 | Importe les erreurs du projet. |
| 3 | `HEX_DIGITS` (`hex.rs:3 HEX_DIGITS`) : les 16 caractères hexadécimaux en minuscules ; sert d'alphabet d'encodage (sortie toujours en minuscules, comme le sujet). |
| 5 | `decode_hex` (`hex.rs:5 decode_hex`) : texte → octets, **sans** inversion d'ordre (l'ordre du texte est conservé). |
| 6 | Travaille sur les octets du texte (un chiffre hexadécimal = 1 octet ASCII). |
| 7–11 | Un nombre impair de chiffres est refusé (`InvalidHexLength`), car il manquerait la moitié d'un octet. |
| 12 | Tampon de sortie pré-alloué (la moitié du nombre de chiffres). |
| 13 | Parcourt le texte par paires de chiffres, avec leur numéro d'ordre. |
| 14–15 | Convertit chaque chiffre en valeur 0–15 ; la position (pour les messages d'erreur) est celle du caractère dans le texte. |
| 16 | `(haut << 4) \| bas` : assemble les deux moitiés de 4 bits en un octet. |
| 18 | Rend les octets. |
| 21 | `encode_hex` (`hex.rs:21 encode_hex`) : octets → texte. |
| 22 | Texte pré-alloué (2 caractères par octet). |
| 23–26 | Pour chaque octet : moitié haute (`>> 4`) puis moitié basse (`& 0x0f`), chacune choisit un caractère dans `HEX_DIGITS`. |
| 27 | Rend le texte. |
| 30 | `hex_value` (`hex.rs:30 hex_value`) : un caractère → sa valeur. |
| 32–34 | `0-9`, `a-f` et `A-F` sont acceptés (la saisie en majuscules passe). |
| 35 | Tout autre caractère → `InvalidHexDigit { position }`. |

## `little_endian.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 3 | `U128_BYTES = 16` : taille maximale d'un `u128`. |
| 5 | `u128_to_le_bytes` (`little_endian.rs:5 u128_to_le_bytes`) : nombre → octets, petit bout en premier. |
| 6 | `to_le_bytes` du langage : 16 octets, poids faible d'abord. |
| 7–10 | Cherche le dernier octet non nul (`rposition`) pour supprimer les zéros de tête ; si tout vaut 0, on garde 1 octet (le nombre zéro s'écrit `00`). |
| 11 | Rend seulement les octets significatifs. |
| 14 | `le_bytes_to_u128` (`little_endian.rs:14 le_bytes_to_u128`) : octets → nombre. |
| 15–19 | Plus de 16 octets : refuse (`NumberTooLarge`), car ça ne tient pas dans un `u128`. |
| 20–21 | Copie dans un tampon de 16 octets rempli de zéros (les octets absents sont les poids forts nuls). |
| 22 | `u128::from_le_bytes` reconstitue le nombre. |

## `stdin.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 5 | `MAX_INPUT_BYTES` (`stdin.rs:5 MAX_INPUT_BYTES`) : 64 Mio, limite d'entrée. |
| 7 | `read_message` (`stdin.rs:7 read_message`) : lit tout le flux, borné par `limit_bytes`. Générique sur `Read` : testable sans toucher à stdin. |
| 8–10 | Convertit la limite en `u64` et ajoute 1 : on lit **un octet de plus** que la limite pour détecter un dépassement. |
| 11 | Tampon vide. |
| 12–15 | `take(allowed)` coupe la lecture au-delà de la limite ; en cas d'erreur d'E/S, le tampon est effacé avant de la renvoyer. |
| 16–19 | Si on a lu plus que la limite : efface et refuse (`InputTooLarge`). |
| 20 | Retire un seul retour à la ligne final. |
| 21 | Rend un `Plaintext` (effacé à la destruction). |
| 24 | `strip_final_newline` (`stdin.rs:24 strip_final_newline`). |
| 25–26 | Termine par `\r\n` : retire les deux caractères. |
| 27–28 | Termine par `\n` : retire un seul caractère (on n'abîme pas les lignes vides voulues). |

## Historique

### 2.1 — Décodage hexadécimal (2026-09-24)

`decode_hex(text: &str) -> Result<Vec<u8>>` convertit chaque paire de chiffres en un octet, **dans l'ordre du texte** : `"67452301"` donne `[0x67, 0x45, 0x23, 0x01]`.

Ce décodage est volontairement neutre vis-à-vis de l'ordre des octets. Dans le sujet, un message ou une clé XOR/AES en hexadécimal est une suite d'octets lue telle quelle ; le little-endian n'intervient que lorsqu'on interprète ces octets comme **un nombre** (RSA, p. 4). Cette interprétation se fait sur les octets décodés, par le code qui en a besoin.

Validation stricte (REGLES.md § 10.2, § 17) :

- longueur impaire → `InvalidHexLength { digits }` ;
- caractère hors `0-9a-fA-F` → `InvalidHexDigit { position }`, position en octets depuis le début du texte. Le caractère fautif n'est jamais recopié (§ 4.4) ;
- un caractère non ASCII (plusieurs octets UTF-8) est rejeté à la position de son premier octet ;
- le texte vide donne un tableau vide : c'est à l'appelant de décider si une entrée vide est acceptable.

Le travail se fait sur `as_bytes()` pour éviter toute allocation intermédiaire (§ 20.3) : la seule allocation est le résultat, dimensionné une fois (`with_capacity`).

Tests : ordre des octets, majuscules/minuscules, entrée vide, longueur impaire, position du chiffre invalide, caractère non ASCII.

### 2.2 — Encodage hexadécimal (2026-09-24)

`encode_hex(bytes: &[u8]) -> String` écrit chaque octet sous forme de deux chiffres **minuscules**, dans l'ordre des octets, sans préfixe `0x` ni séparateur : c'est le format de toutes les sorties du sujet.

- Table `HEX_DIGITS` (`b"0123456789abcdef"`) indexée par demi-octet : pas de formatage `format!("{:02x}")` par octet, donc pas d'allocation par octet.
- Une seule allocation, dimensionnée à `2 × len` (§ 20.3).
- Fonction totale : elle ne peut pas échouer, donc elle renvoie `String` et non `Result`.

`lib.rs` réexporte désormais les types partagés (`SymmetricKey`, `CryptoSystem`, `Mode`, `RsaCipher`) pour que les tests externes y accèdent.

Tests ajoutés (REGLES.md § 38) dans `tests/<type>/` :

| Type | Ce qui est couvert pour le codec |
|---|---|
| Unitaire | Octets limites `00 0f f0 ff`, ordre, minuscules, 2 chiffres par octet, rejet de `0x` |
| Intégration | Hex → `SymmetricKey` → hex ; clé vide refusée ; aller-retour des 256 valeurs |
| Statique | Règles du code source (voir `tests/code.md`) |
| Régression | Encodage figé des 256 valeurs, exemple `67452301` |
| E2E | Binaire actuel : code 84, stderr non vide, stdout vide. **Le codec n'est pas encore branché à la CLI** : ses tests E2E arriveront à l'étape 5 |
| Acceptation | Sujet p. 1 (stderr + 84) ; sorties en hex minuscule nu |
| Performance | Aller-retour de 1 Mio en moins de 2 s en mode debug (mesuré : ~0,2 s) |
| Sécurité | Message d'erreur sans le contenu, aucune panique sur ASCII et Unicode hostile, `Debug` d'une clé décodée masqué |
| Usabilité | Messages explicites : nature du problème, longueur, position |
| Accessibilité | Sorties et erreurs en ASCII pur, une ligne, sans séquence d'échappement |

Depuis le 2026-09-24, tous les tests unitaires du codec vivent dans `tests/unit/hex.rs` (REGLES.md § 38).

### 2.3 — Nombres en little-endian (2026-09-24)

Le sujet (p. 4) affiche les nombres en hexadécimal little-endian : `0x01234567` s'écrit `67452301`. Deux fonctions font le lien entre un nombre et ses octets ; `encode_hex`/`decode_hex` font ensuite le lien avec le texte.

| Fonction | Comportement |
|---|---|
| `u128_to_le_bytes(value) -> Vec<u8>` | Octet de poids faible en premier ; les zéros de poids fort sont retirés, mais au moins un octet est produit (`0` → `[0x00]`) ; les zéros internes sont conservés (`0x0100` → `[0x00, 0x01]`) |
| `le_bytes_to_u128(bytes) -> Result<u128>` | Accepte de 0 à 16 octets (zéros de poids fort autorisés) ; tableau vide → `0` ; plus de 16 octets → `NumberTooLarge { max_bytes: 16 }` |

Périmètre (décision du 2026-09-24) : ces fonctions couvrent les nombres jusqu'à 128 bits. Les grands nombres de RSA relèvent de l'arithmétique multiprécision de Frédrich.

Nouvelle variante partagée : `MyPgpError::NumberTooLarge { max_bytes }` (fichier commun `src/domain/error.rs`, à relire par Frédrich). Elle ne recopie aucun octet de l'entrée (§ 4.4).

Implémentation : `u128::to_le_bytes`/`from_le_bytes` de la bibliothèque standard, sans `unsafe` ni dépendance. La longueur est vérifiée avant `copy_from_slice`, qui ne peut donc pas paniquer.

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | Exemple du sujet, zéro, zéros internes, `u128::MAX`, 0/3/16/17 octets |
| Intégration | Nombre → octets → hex et retour |
| Statique | Règles de code existantes (aucun ajout nécessaire) |
| Régression | Cinq vecteurs figés dont `0x01234567` et `2^120` |
| E2E | **Sans objet à ce stade** : aucune commande du binaire n'affiche encore de nombre ; ajoutés avec `rsa` et la CLI |
| Acceptation | Sujet p. 4 : `0x01234567` ↔ `67452301` |
| Performance | 10⁶ allers-retours en moins de 2 s (mesuré : ~0,3 s en debug) |
| Sécurité | 17 à 64 octets refusés sans panique ni recopie du contenu |
| Usabilité | Message exact indiquant la limite |
| Accessibilité | Sortie hexadécimale ASCII de longueur prévisible |

### 2.4 — Lecture bornée de l'entrée (2026-09-24)

`read_message<R: Read>(reader: R, limit_bytes: usize) -> Result<Vec<u8>>` lit tout le message, puis retire **un seul** saut de ligne final.

| Règle | Décision |
|---|---|
| Taille maximale | `MAX_INPUT_BYTES` = 64 Mio (décision du 2026-09-24) |
| Contrôle | `reader.take(limit + 1)` : au plus un octet de plus que la limite est lu, même sur une entrée infinie ; au-delà de `limit` → `InputTooLarge { limit_bytes }` (§ 17, § 27.4) |
| Ordre | La limite porte sur l'entrée brute, **avant** le retrait du saut de ligne |
| Fin de ligne | `\r\n` puis `\n` : un seul retiré (décision du 2026-09-24) ; un `\r` isolé et les sauts internes sont conservés |
| Contenu | Octets bruts, sans conversion UTF-8 : le mode flux chiffre des octets |

Le lecteur est générique (`R: Read`) : `main.rs` passera `std::io::stdin().lock()`, les tests un `Cursor` ou `std::io::repeat`. La conversion de `limit` en `u64` ne peut pas échouer sur les plateformes 32/64 bits ; si elle échouait, la limite serait inreprésentable et l'erreur `InputTooLarge` est renvoyée plutôt qu'une panique. Les erreurs d'E/S remontent via `From<io::Error>` (`MyPgpError::Io`).

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | 12 tests (dont propagation d'une erreur de lecture en `Io` avec sa source, § 11.5) : `\n`, `\r\n`, un seul retrait, `\r` isolé, sauts internes, vide, exactement la limite, limite + 1, limite avant retrait, octets binaires |
| Intégration | Entrée lue → texte UTF-8 → `decode_hex` |
| Statique | Règles existantes |
| Régression | Six cas figés dont « Winter is coming » avec `\n` et `\r\n` |
| E2E | **Sans objet** : le binaire ne lit pas encore stdin ; ajouté à l'étape 5 |
| Acceptation | « All men must die\n » donne exactement 16 octets, taille d'un bloc AES (exemple du sujet) |
| Performance | 64 Mio lus en moins de 2 s |
| Sécurité | Entrée infinie coupée à la limite ; erreur sans recopie du contenu |
| Usabilité | Message indiquant la limite ; fins de ligne Unix et Windows équivalentes |
| Accessibilité | Message d'erreur ASCII sans séquence d'échappement |

### Audit 2.5 — `read_message` renvoie un `Plaintext` (2026-09-24, REGLES.md § 4.5, § 20.1)

- Le résultat est un `Plaintext`, effacé de la mémoire à sa destruction.
- Sur erreur (lecture impossible ou limite dépassée), le tampon déjà lu est effacé par `wipe` **avant** d'être libéré.
- Limite connue (§ 20.3) : `read_to_end` agrandit le tampon par réallocations successives ; les anciens tampons sont libérés sans être effacés. La taille finale de l'entrée n'étant pas connue à l'avance, ce point ne peut être corrigé qu'en lisant par blocs dans un tampon pré-alloué, ce qui sera réévalué si nécessaire.
