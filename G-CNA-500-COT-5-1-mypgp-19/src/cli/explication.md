# `src/cli` — explication

## À quoi sert ce dossier ?

Le sujet impose une interface unique : `./my_pgp CRYPTO_SYSTEM MODE [OPTIONS] KEY`.
Ce dossier isole le parsing d'arguments et le routage afin de garder `main.rs`
minimal et de permettre de tester la logique sans spawner le binaire.

## Comment fonctionne le parsing ?

`args.rs` lit le vecteur d'arguments positionnel sans bibliothèque externe.
L'ordre est fixe : CRYPTO_SYSTEM, puis MODE (`-c`, `-d` ou `-g`), puis l'option
facultative `-b`, puis la clé.

Pour `-g`, les deux nombres premiers P et Q suivent directement le `-g`.
Pour les systèmes PGP, la clé est au format `SYMMETRIC_KEY:RSA_KEY`.

**Où.** `args.rs:49 parse` lit le cryptosystème (lignes 51 à 54) puis le mode (55 à 57) ; `args.rs:80 parse_mode_and_options` gère `-g` (87 à 108) et `-b` (114 à 124) ; `args.rs:135 validate_args` rejette les combinaisons interdites. **Exemples :** `aes -c -b 5769…` donne mode chiffrer + bloc ; `rsa -g d3 e3` donne mode générer avec P=`d3`, Q=`e3` ; `xor -c -g …` est refusé (`-g` réservé à RSA). **Analogie :** un guichet qui vérifie le formulaire avant de le transmettre.

## Comment fonctionne le dispatch ?

`dispatch.rs` reçoit un `Args` déjà validé et sépare la question "quoi
faire ?" de la question "comment le faire ?" (REGLES.md § 4) :

- **Décider** (`decide`) : à partir de `args.system`, `args.mode` et
  `args.operation` (bloc unique ou flux de blocs), on choisit quelle
  fonction cryptographique appeler et avec quelle clé. Cette étape ne lit
  ni n'écrit rien — c'est une pure décision, testable sans toucher à
  stdin/stdout. Distinguer bloc/flux avec un enum plutôt qu'un booléen
  évite l'ambiguïté d'un `true` qui ne dit pas, à la lecture d'un appel,
  s'il désigne le mode bloc ou le mode flux.
- **Exécuter** (`execute`) : une fois la décision prise, cette étape (et
  elle seule) lit stdin via `read_message`, appelle la fonction décidée,
  puis écrit le résultat sur stdout (hexadécimal en chiffrement, octets
  bruts en déchiffrement).

Analogie : `decide` est un aiguilleur qui regarde la destination écrite
sur le colis et choisit le quai ; `execute` est le porteur qui prend
effectivement le colis et le pose sur ce quai. Le porteur ne regarde
jamais l'étiquette, l'aiguilleur ne touche jamais au colis.

Les erreurs remontent via `Result` jusqu'à `main.rs` qui les affiche sur
stderr et quitte avec le code 84.

**Où.** `dispatch.rs:58 decide` (et `decide_xor`, `decide_aes`, `decide_rsa`, `decide_pgp` lignes 68, 91, 115, 143) pour la décision ; `dispatch.rs:207 execute` pour l'exécution ; `dispatch.rs:54 run` les enchaîne. P et Q sont décodés en entiers multiprécision, sans limite de taille fixe. **Exemple :** `xor -d -b KEY` donne `Operation::Decipher` avec `xor_block_decipher` ; `execute` lit alors l'hexadécimal sur stdin et écrit le message en clair.

## Systèmes implémentés

- **`xor`** : chiffrement/déchiffrement XOR symétrique, bloc ou flux
- **`aes`** : chiffrement/déchiffrement AES-128/192/256, bloc ou flux
- **`rsa`** : génération de clés (`-g P Q`), chiffrement (`-c e-n`),
  déchiffrement (`-d d-n`) avec des nombres multiprécision
- **`pgp-xor`** et **`pgp-aes`** : chiffrement hybride RSA + symétrique.
  La clé est au format `SYMMETRIC_KEY:RSA_KEY`. En chiffrement, le
  message est chiffré avec la clé symétrique et la clé symétrique est
  chiffrée avec la clé publique RSA. La sortie est deux lignes :
  clé symétrique chiffrée puis message chiffré. En déchiffrement, la
  première ligne est la clé symétrique chiffrée et la deuxième est le
  message chiffré.

**Où (PGP).** `dispatch.rs:143 decide_pgp` et les bras `PgpEncrypt` (ligne 237) / `PgpDecrypt` (ligne 249) de `dispatch.rs:207 execute`. **Exemple :** `pgp-aes -c -b CLE_SYM:010001-n` chiffre le message avec AES, chiffre la clé AES avec RSA, et affiche deux lignes hexadécimales.

## Historique

### Implémentation complète (2026-09-25)

Tous les cryptosystèmes sont désormais câblés et fonctionnels. La CLI gère
le parsing complet de `-g`, la validation d'incompatibilité `-b`/`-g`,
et le routage vers RSA, PGP-XOR et PGP-AES avec leur logique de
chiffrement hybride à deux étapes.
