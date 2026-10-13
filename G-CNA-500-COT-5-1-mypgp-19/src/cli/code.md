# `src/cli` — documentation technique

## Contenu

| Fichier | Rôle |
|---|---|
| `args.rs` | Parsing des arguments de ligne de commande |
| `dispatch.rs` | Routage vers le cryptosystème demandé |

## Interface

```
./my_pgp CRYPTO_SYSTEM MODE [-b] KEY
./my_pgp rsa -g P Q
```

- `CRYPTO_SYSTEM` : `xor`, `aes`, `rsa`, `pgp-xor`, `pgp-aes`
- `MODE` : `-c` (chiffrer), `-d` (déchiffrer), `-g P Q` (génération RSA)
- `-b` : mode bloc (facultatif, incompatible avec `-g`)
- `KEY` : clé hexadécimale symétrique, `e-n`/`d-n` (RSA), ou `symkey:rsakey` (PGP)

Le message est lu depuis stdin. La sortie est en hexadécimal sur stdout.
Les erreurs vont sur stderr avec le code de sortie 84.

## `mod.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1–2 | `pub mod args` (analyse de la ligne de commande) et `pub mod dispatch` (décision puis exécution). |

## `args.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 5–8 | `OperationMode` (`args.rs:5 OperationMode`) : `Block` (option `-b`) ou `Stream` ; une énumération plutôt qu'un booléen, pour qu'un `match` se lise sans deviner ce que vaut `true`. |
| 10–17 | `Args` : système, mode (`-c`/`-d`/`-g`), mode bloc/flux, clé (texte brut), et `p`/`q` (seulement pour `-g`). |
| 19 | `ParsedMode` (`args.rs:19 ParsedMode`) : alias du quintuplet rendu par l'analyse (mode, bloc/flux, clé, p, q), pour garder les signatures lisibles. |
| 21–39 | `USAGE` (`args.rs:21 USAGE`) : texte d'aide recopié du sujet (p. 3). |
| 41–43 | `usage` (`args.rs:41 usage`) : rend le texte d'aide. |
| 45–47 | `is_help_requested` (`args.rs:45 is_help_requested`) : vrai si `-h` ou `--help` figure dans les arguments. |
| 49 | `parse` (`args.rs:49 parse`) : transforme les arguments en `Args`. |
| 50 | Itérateur sur les arguments, convertis en `&str`. |
| 51–54 | Premier argument : le cryptosystème, absent → `missing CRYPTO_SYSTEM`. |
| 55–57 | Deuxième argument : le mode, absent → `missing MODE`. |
| 58 | Délègue le reste (option `-b`, clé, ou P et Q) à `parse_mode_and_options`. |
| 59–66 | Assemble `Args`. |
| 69 | `parse_system` (`args.rs:69 parse_system`) : nom → `CryptoSystem`. |
| 70–77 | Un cas par nom ; tout autre nom → `unknown CRYPTO_SYSTEM`. |
| 80–83 | `parse_mode_and_options` (`args.rs:80 parse_mode_and_options`) : rend un `ParsedMode`. |
| 85–86 | `-c` et `-d` donnent `Mode::Cipher` et `Mode::Decipher`. |
| 87–108 | `-g` : lit exactement deux arguments P et Q (sinon erreur), pas de clé ; rend `Mode::Generate`. |
| 109 | Tout autre mode → erreur. |
| 111–113 | Argument suivant : la clé ou `-b` ; absent → `missing key`. |
| 114–124 | Si `-b` : la clé est l'argument suivant, aucun argument après ; rend `OperationMode::Block`. |
| 125–132 | Sinon l'argument est la clé ; rend `OperationMode::Stream`. |
| 135 | `validate_args` (`args.rs:135 validate_args`) : cohérence entre options. |
| 137–146 | `-g` : incompatible avec `-b`, et réservé à `rsa`. |
| 147–153 | `-c`/`-d` : P et Q ne sont autorisés qu'avec `-g`. |

## `dispatch.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 15–16 | `CipherFn` et `DecipherFn` (`dispatch.rs:15 CipherFn`, `dispatch.rs:16 DecipherFn`) : signatures communes aux fonctions de chiffrement et de déchiffrement symétriques (XOR comme AES). |
| 23–52 | `Operation` (`dispatch.rs:23 Operation`) : le résultat de la **décision**, sans aucun I/O. Variantes : `Cipher`, `Decipher`, `RsaGenerate`, `RsaCipher`, `PgpEncrypt`, `PgpDecrypt`. |
| 54–56 | `run` (`dispatch.rs:54 run`) : `execute(decide(args)?)` ; une erreur de décision sort avant toute lecture de stdin. |
| 58–66 | `decide` (`dispatch.rs:58 decide`) : aiguille selon le cryptosystème. |
| 68–89 | `decide_xor` (`dispatch.rs:68 decide_xor`) : clé hexadécimale → `SymmetricKey`, puis choisit `xor_block_*` ou `xor_stream_*` selon le mode et bloc/flux ; `-g` refusé. |
| 91–113 | `decide_aes` (`dispatch.rs:91 decide_aes`) : même chose avec `encrypt_block`/`decrypt_block` ou `aes_stream_*`. |
| 115–141 | `decide_rsa` (`dispatch.rs:115 decide_rsa`) : `-g` → P et Q sont décodés avant l'exécution ; `-c` et `-d` → `RsaCipher` après validation de la clé. |
| 143–205 | `decide_pgp` (`dispatch.rs:143 decide_pgp`) : la clé doit être `SYM:RSA` (séparée par `:`) ; en chiffrement la partie symétrique est lue en hexadécimal ; en déchiffrement elle reste le texte de la clé symétrique chiffrée ; choisit la fonction symétrique. |
| 207–266 | `execute` (`dispatch.rs:207 execute`) : **seul endroit qui touche stdin/stdout** ; un bras par `Operation`. |
| 220–224 | Bras `RsaGenerate` : calcule puis affiche la paire de clés ; P et Q sont déjà décodés et contrôlés dans `decide_rsa`. |
| 225–235 | Bras `RsaCipher` : chiffre un message brut ou déchiffre une entrée hexadécimale. |
| 237–248 | Bras `PgpEncrypt` : chiffre la clé symétrique avec RSA, le message avec la clé symétrique, affiche deux lignes. |
| 249–263 | Bras `PgpDecrypt` : lit le message chiffré, déchiffre la clé symétrique avec RSA, puis déchiffre le message. |
| 268–270 | `read_raw_stdin` (`dispatch.rs:268 read_raw_stdin`) : lit le message brut (limite 64 Mio). |
| 272–277 | `read_hex_stdin` (`dispatch.rs:272 read_hex_stdin`) : lit puis décode un texte hexadécimal (espaces de fin retirés). |
| 279–297 | `read_pgp_ciphertext` (`dispatch.rs:279 read_pgp_ciphertext`) : lit le chiffré PGP ; accepte une ligne (message seul) ou deux lignes dont la première doit égaler la clé chiffrée, sinon erreur. |
| 299–302 | `parse_rsa_prime` (`dispatch.rs:299 parse_rsa_prime`) : décode P ou Q en grand entier sans plafond de taille codé en dur. |
| 304–306 | `symmetric_key_from_hex` (`dispatch.rs:304 symmetric_key_from_hex`) : hexadécimal → `SymmetricKey`. |

## Historique

### Implémentation complète RSA et PGP (2026-09-25)

Ajout du module `rsa` avec `cipher.rs` (implémentation `RsaCipher`),
`keygen.rs`, `prime.rs`, `random.rs`. La CLI gère désormais
`-g`, le format `e-n`/`d-n`, et le chiffrement hybride PGP
à deux lignes (clé symétrique chiffrée + message chiffré).

### Validation des arguments (2026-09-25)

Ajout de `validate_args()` dans `args.rs` et son appel dans `main.rs`
pour vérifier l'incompatibilité `-b`/`-g` et la restriction `-g` à RSA.
