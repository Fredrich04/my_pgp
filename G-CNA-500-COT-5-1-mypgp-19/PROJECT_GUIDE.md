# MyPGP - explication complète du projet

Ce document explique le projet de bout en bout : son objectif, son architecture
Rust, son interface en ligne de commande, ses représentations de données et
les algorithmes XOR, AES, RSA et PGP hybride.

Le projet est pédagogique : les primitives cryptographiques sont implémentées
from scratch, sans dépendance externe.

> **Attention au vocabulaire :** `-b` signifie **mode bloc cryptographique**.
> Ce n'est pas un « mode bloquant » au sens d'une attente réseau ou d'une
> lecture d'entrée.

---

## 1. Objectif général

Le binaire produit est `my_pgp`. Il accepte un message sur l'entrée standard
(`stdin`), applique l'algorithme demandé, puis écrit le résultat sur la sortie
standard (`stdout`).

Les systèmes disponibles sont :

```text
xor       XOR symétrique
aes       AES-128, AES-192 ou AES-256
rsa       RSA
pgp-xor   chiffrement hybride RSA + XOR
pgp-aes   chiffrement hybride RSA + AES
```

Le programme est lancé ainsi :

```text
./my_pgp CRYPTO_SYSTEM MODE [OPTIONS] KEY
```

Exemples :

```bash
echo -n "Bonjour" | ./my_pgp xor -c 01020304
echo -n "All men must die" | ./my_pgp aes -c -b 57696e74657220697320636f6d696e67
echo -n "WF" | ./my_pgp rsa -c 0101-19bb
```

Les erreurs sont écrites sur `stderr` et le programme quitte avec le code `84`.

---

## 2. Parcours complet d'une commande

### 2.1 `main`

Le fichier `src/main.rs` :

1. récupère les arguments après le nom du programme ;
2. détecte `-h` et `--help` ;
3. appelle `parse` ;
4. appelle `validate_args` ;
5. appelle `run` ;
6. affiche une erreur et quitte avec `84` si une étape échoue.

Le programme n'utilise pas de `panic` pour signaler une erreur utilisateur :
les erreurs normales passent par `Result<T, MyPgpError>`.

### 2.2 Parsing et validation

Le module `src/cli/args.rs` convertit les chaînes en types :

```rust
CryptoSystem::Xor
CryptoSystem::Aes
CryptoSystem::Rsa
CryptoSystem::PgpXor
CryptoSystem::PgpAes
```

Les modes sont :

```rust
Mode::Cipher    // -c
Mode::Decipher  // -d
Mode::Generate  // -g
```

L'option `-b` sélectionne :

```rust
OperationMode::Block
```

Son absence sélectionne :

```rust
OperationMode::Stream
```

`-g` est réservé à RSA et prend exactement deux nombres premiers :

```bash
./my_pgp rsa -g P Q
```

### 2.3 Dispatch

`src/cli/dispatch.rs` associe les arguments à une fonction :

| Commande | Chiffrement | Déchiffrement |
|---|---|---|
| `xor -b` | `xor_block_cipher` | `xor_block_decipher` |
| `xor` | `xor_stream_cipher` | `xor_stream_decipher` |
| `aes -b` | `encrypt_block` | `decrypt_block` |
| `aes` | `aes_stream_cipher` | `aes_stream_decipher` |
| `rsa` | `encrypt_message` | `decrypt_message` |
| `pgp-xor` | RSA de la clé puis XOR du message | RSA de la clé puis XOR inverse |
| `pgp-aes` | RSA de la clé puis AES du message | RSA de la clé puis AES inverse |

Les fonctions sont stockées dans `Operation` sous forme de pointeurs typés :

```rust
type CipherFn = fn(&Plaintext, &SymmetricKey) -> Result<Vec<u8>>;
type DecipherFn = fn(&[u8], &SymmetricKey) -> Result<Plaintext>;
```

### 2.4 Lecture de `stdin`

`src/io/stdin.rs` lit au maximum 64 MiB. Un retour à la ligne final est retiré :

```text
echo "abc"  -> le message cryptographique est "abc"
echo -n "abc" -> le message cryptographique est aussi "abc"
```

Les séquences finales `\n` et `\r\n` sont supprimées. Les octets internes,
les zéros et les autres caractères sont conservés.

Les types `Plaintext` et `SymmetricKey` encapsulent les octets. Leur
implémentation de `Drop` efface le contenu avec `wipe`, et leur affichage
`Debug` masque les secrets.

---

## 3. Représentation hexadécimale et endianess

### 3.1 Hexadécimal

Les clés et les sorties sont affichées en hexadécimal :

```text
41 42 43 -> 414243
```

`decode_hex` accepte les chiffres `0-9`, `a-f` et `A-F`. Le nombre de chiffres
doit être pair.

```text
"4142" -> [0x41, 0x42]
"414"  -> erreur de longueur impaire
"41xz" -> erreur au caractère x
```

`encode_hex` produit toujours des lettres minuscules.

### 3.2 Little-endian pour les nombres

Les nombres RSA sont interprétés en little-endian :

```text
hex "d3"       -> 211
hex "0101"     -> 0x0101 = 257
hex "19bb"     -> octets [0x19, 0xbb], nombre 0xbb19
```

La convention exacte est définie par le sujet et doit être respectée lors des
tests RSA.

AES applique en plus l'adaptation de format du sujet : les mots de quatre
octets de la clé et du bloc chiffré sont inversés à l'entrée ou à la sortie.
Cette adaptation est réalisée par `reverse_words` dans `aes/block.rs`.

---

## 4. XOR

### 4.1 Principe mathématique

XOR compare deux bits :

```text
0 XOR 0 = 0
0 XOR 1 = 1
1 XOR 0 = 1
1 XOR 1 = 0
```

La même opération sert à chiffrer et à déchiffrer :

```text
(message XOR clé) XOR clé = message
```

Exemple sur un octet :

```text
message = 0x41 = 0100 0001
clé     = 0x03 = 0000 0011
résultat= 0x42 = 0100 0010
```

### 4.2 XOR en mode bloc

Avec `-b`, le message et la clé doivent avoir exactement la même taille.
Le projet inverse les octets du message avant le XOR conformément au format du
sujet, puis inverse cette transformation au déchiffrement.

Exemple :

```bash
echo -n "ABCD" | ./my_pgp xor -c -b 01020304
```

Le message fait 4 octets et la clé fait 4 octets : la commande est valide.

Cette commande échoue :

```bash
echo -n "ABC" | ./my_pgp xor -c -b 01020304
```

Erreur attendue : la taille du message n'est pas celle de la clé.

### 4.3 XOR en mode stream

Sans `-b`, la clé définit la taille d'un morceau. Le message est découpé en
blocs, puis le dernier morceau est complété par des zéros.

Avec une clé de 4 octets :

```text
message de 10 octets : [AAAA] [BBBB] [CC]
après remplissage     : [AAAA] [BBBB] [CC00]
```

Test :

```bash
echo -n "ABCDEFGHIJ" | ./my_pgp xor -c 01020304
```

Le déchiffrement se fait avec :

```bash
ciphertext=$(echo -n "ABCDEFGHIJ" | ./my_pgp xor -c 01020304)
echo "$ciphertext" | ./my_pgp xor -d 01020304
```

Le résultat peut contenir des octets nuls après `J`, car le format ne mémorise
pas la longueur exacte avant remplissage.

---

## 5. AES

Le projet implémente AES-128, AES-192 et AES-256 :

| Clé | Mots de 32 bits | Tours |
|---:|---:|---:|
| 16 octets | 4 | 10 |
| 24 octets | 6 | 12 |
| 32 octets | 8 | 14 |

AES travaille toujours sur un bloc de 16 octets.

### 5.1 État AES

Les 16 octets sont vus comme une matrice de 4 lignes par 4 colonnes :

```text
| s0  s4  s8  s12 |
| s1  s5  s9  s13 |
| s2  s6  s10 s14 |
| s3  s7  s11 s15 |
```

Le code utilise le type :

```rust
type State = [u8; 16];
```

### 5.2 Expansion de clé

`expand_key` transforme la clé initiale en clés de tour de 16 octets.
L'algorithme utilise :

1. `RotWord` : rotation circulaire d'un mot de 4 octets ;
2. `SubWord` : remplacement de chaque octet par la S-box ;
3. `Rcon` : constante de tour multipliée par 2 dans `GF(2^8)` ;
4. XOR avec le mot situé `Nk` positions avant.

Pour AES-256, une étape `SubWord` supplémentaire est appliquée lorsque
`word % Nk == 4`.

### 5.3 S-box

La S-box n'est pas écrite comme une constante manuelle. `sbox.rs` la construit
à la compilation :

1. calcul de l'inverse multiplicatif dans `GF(2^8)` ;
2. transformation affine ;
3. ajout de la constante `0x63`.

Pour un octet `x` :

```text
S(x) = affine(inverse_GF256(x))
```

La transformation affine du code est :

```text
x XOR rotl(x, 1) XOR rotl(x, 2) XOR rotl(x, 3) XOR rotl(x, 4) XOR 0x63
```

`INV_SBOX` est construite en inversant la table, ce qui permet le déchiffrement.

### 5.4 Arithmétique dans `GF(2^8)`

Les octets AES sont des polynômes binaires modulo :

```text
x^8 + x^4 + x^3 + x + 1
```

Le code utilise la représentation hexadécimale `0x11b`. Dans `gf_mul` :

- un bit à 1 dans le multiplicateur déclenche un XOR ;
- le décalage à gauche représente une multiplication par `x` ;
- si le bit de poids fort déborde, le résultat est réduit par `0x1b`.

L'inverse de `a` est calculé comme `a^254`, car dans `GF(256)`,
`a^255 = 1` pour `a != 0`.

### 5.5 Les transformations d'un tour

Un tour AES normal applique :

1. `SubBytes` : S-box sur chaque octet ;
2. `ShiftRows` : rotations des lignes ;
3. `MixColumns` : multiplication de chaque colonne ;
4. `AddRoundKey` : XOR avec la clé du tour.

Le dernier tour omet `MixColumns`.

La matrice de `MixColumns` est :

```text
| 02 03 01 01 |
| 01 02 03 01 |
| 01 01 02 03 |
| 03 01 01 02 |
```

Le déchiffrement applique les transformations inverses dans l'ordre inverse :

```text
AddRoundKey
InvShiftRows
InvSubBytes
InvMixColumns
```

### 5.6 AES en mode bloc `-b`

Le message doit faire exactement 16 octets.

Test officiel du projet :

```bash
echo -n "All men must die" | \
./my_pgp aes -c -b 57696e74657220697320636f6d696e67
```

Sortie attendue :

```text
744ce22c385958348f0df26eceb62eef
```

Déchiffrement :

```bash
echo -n "744ce22c385958348f0df26eceb62eef" | \
./my_pgp aes -d -b 57696e74657220697320636f6d696e67
```

### 5.7 AES en mode stream

Le nom `stream` signifie ici que le programme accepte un message de taille
quelconque en le découpant en blocs AES. Ce n'est pas un vrai chiffrement par
flot et ce n'est pas un mode de fonctionnement sécurisé comme CTR.

Pour chaque morceau :

```text
bloc court -> ajout de zéros jusqu'à 16 octets -> AES
```

Test :

```bash
echo -n "Bonjour" | \
./my_pgp aes -c 00112233445566778899aabbccddeeff
```

La sortie représente 16 octets, soit 32 chiffres hexadécimaux.

Déchiffrement avec inspection des octets :

```bash
ciphertext=$(echo -n "Bonjour" | \
    ./my_pgp aes -c 00112233445566778899aabbccddeeff)
echo -n "$ciphertext" | \
    ./my_pgp aes -d 00112233445566778899aabbccddeeff | \
    od -An -t x1
```

Les octets finaux peuvent être `00`. Le projet ne peut pas distinguer un zéro
original d'un zéro de remplissage.

---

## 6. BigUint : grands entiers pour RSA

Rust ne fournit pas ici de bibliothèque externe de grands entiers. Le projet
utilise `BigUint`, composé d'une liste de limbs de 32 bits :

```rust
pub(crate) limbs: Vec<u32>
```

Le limb 0 contient les bits de poids faible.

Exemple conceptuel :

```text
nombre = limb[0] + limb[1] * 2^32 + limb[2] * 2^64 + ...
```

Les opérations implémentées sont :

- normalisation : suppression des limbs hauts nuls ;
- comparaison ;
- test de zéro et de parité ;
- longueur en bits ;
- lecture et écriture d'un bit ;
- décalage gauche et droit ;
- addition ;
- soustraction, uniquement lorsque le résultat reste positif ;
- multiplication « schoolbook » ;
- division et reste par division binaire ;
- PGCD d'Euclide ;
- exponentiation modulaire ;
- inverse modulaire ;
- conversions octets little-endian et hexadécimal.

### 6.1 Exponentiation modulaire

`powmod` utilise l'exponentiation rapide par carrés successifs :

```text
result = 1
base = base mod m
pour chaque bit de l'exposant, du moins significatif au plus significatif :
    si le bit vaut 1 : result = result * base mod m
    base = base * base mod m
```

Cela évite de calculer directement `base^exposant`.

Exemple mathématique :

```text
3^13 mod 7
13 = 1101b
```

On effectue seulement quelques multiplications et réductions modulo 7.

### 6.2 Inverse modulaire

`mod_inverse(a, m)` utilise une variante de l'algorithme d'Euclide étendu.
L'inverse existe si et seulement si :

```text
gcd(a, m) = 1
```

Par exemple :

```text
3^-1 mod 7 = 5
car 3 * 5 = 15 = 1 mod 7
```

---

## 7. RSA

### 7.1 Génération des clés

Avec deux nombres premiers `p` et `q` :

```text
n = p * q
lambda = lcm(p - 1, q - 1)
```

Le code calcule le PPCM avec :

```text
lcm(a, b) = a * b / gcd(a, b)
```

Il essaie ensuite les exposants premiers de Fermat, dans cet ordre :

```text
65537, 257, 17, 5, 3
```

Le premier `e` tel que :

```text
e < lambda
gcd(e, lambda) = 1
```

est retenu. Puis `d` est calculé :

```text
d * e = 1 mod lambda
```

La clé publique est `(e, n)` et la clé privée `(d, n)`.

Exemple du README :

```bash
./my_pgp rsa -g d3 e3
```

Résultat :

```text
public key: 0101-19bb
private key: 9d5b-19bb
```

Les valeurs sont affichées en hexadécimal little-endian.

### 7.2 Test de primalité

Le module `prime.rs` propose Miller-Rabin probabiliste :

1. rejet des nombres inférieurs à 2 ;
2. test de petits diviseurs ;
3. écriture `n - 1 = d * 2^s` avec `d` impair ;
4. tirage de témoins aléatoires ;
5. calcul de `a^d mod n` ;
6. carrés successifs jusqu'à obtenir `n - 1` ou conclure que `n` est composé.

La génération aléatoire vient de `/dev/urandom`. Le générateur force :

- le bit de poids fort demandé ;
- le bit de poids faible à 1 ;

afin d'obtenir un candidat impair de la bonne taille.

La clé `keygen` du sujet reçoit toutefois directement `P` et `Q` ; la
fonction de génération de candidats premiers sert aux tests et aux usages
internes prévus par le projet.

### 7.3 Chiffrement RSA

Pour un message `m` et une clé publique `(e, n)` :

```text
c = m^e mod n
```

Déchiffrement avec `(d, n)` :

```text
m = c^d mod n
```

Le message est interprété comme un grand entier little-endian. Le projet
refuse un message `m >= n`, car RSA nécessite ici `0 <= m < n`.

Test :

```bash
echo -n "WF" | ./my_pgp rsa -c 0101-19bb
```

Sortie attendue :

```text
8f84
```

Puis :

```bash
echo -n "8f84" | ./my_pgp rsa -d 9d5b-19bb
```

Résultat :

```text
WF
```

### 7.4 Limites RSA

Il s'agit de RSA textbook :

- pas de OAEP ;
- pas de padding ;
- pas d'authentification ;
- chiffrement déterministe ;
- message limité par `n` ;
- `powmod` n'est pas constant-time.

Il est adapté au sujet pédagogique, mais pas à la protection de vrais secrets.

---

## 8. PGP hybride

PGP ne chiffre pas directement un long message avec RSA. Le projet utilise
une architecture hybride :

1. une clé symétrique chiffre le message ;
2. RSA chiffre cette clé symétrique ;
3. les deux résultats sont envoyés.

### 8.1 Format de la clé en entrée

Pour `pgp-aes` ou `pgp-xor`, la clé est :

```text
SYMMETRIC_KEY:RSA_KEY
```

Exemple :

```text
57696e74657220697320636f6d696e67:010001-c9f91a...
```

À gauche : clé AES/XOR.  
À droite : clé RSA publique pour chiffrer ou clé RSA privée pour déchiffrer.

### 8.2 PGP encryption

Commande conceptuelle :

```bash
echo -n "message" | ./my_pgp pgp-aes -c SYM_KEY:PUBLIC_RSA_KEY
```

Le programme produit deux lignes :

```text
ligne 1 : RSA(SYM_KEY)
ligne 2 : AES(message, SYM_KEY)
```

La ligne 1 est la clé symétrique chiffrée. La ligne 2 est le message chiffré.

Exemple complet avec les paramètres du README :

```bash
echo -n "All men must die" | \
./my_pgp pgp-aes -c -b \
57696e74657220697320636f6d696e67:010001-c9f91a9ff3bd6d84005b9cc8448296330bd23480f8cf8b36fd4edd0a8cd925de139a0076b962f4d57f50d6f9e64e7c41587784488f923dd60136c763fd602fb3
```

### 8.3 PGP decryption

Le déchiffrement est l'opération inverse :

1. RSA déchiffre la clé symétrique ;
2. la ligne de ciphertext est lue en hexadécimal ;
3. AES ou XOR déchiffre le message.

Le dispatcher attend le ciphertext sur `stdin` lors d'un déchiffrement PGP.

---

## 9. Différence entre mode bloc et mode stream

Cette distinction existe pour XOR, AES et PGP.

### Mode bloc : `-b`

```text
un seul bloc
taille exacte obligatoire
pas de padding automatique
```

Pour AES, le message fait exactement 16 octets.

```bash
echo -n "All men must die" | \
./my_pgp aes -c -b 57696e74657220697320636f6d696e67
```

### Mode stream : absence de `-b`

```text
message de taille quelconque
découpage en blocs
dernier bloc complété par des zéros
```

Ce projet appelle cette logique « stream », mais elle correspond à une
succession de blocs indépendants avec padding zéro. Ce n'est pas un mode
cryptographique moderne comme CTR ou GCM.

---

## 10. Gestion des erreurs

`MyPgpError` couvre notamment :

```text
InvalidArguments
InvalidHexLength
InvalidHexDigit
InvalidKeyLength
BlockSizeMismatch
CiphertextLengthNotMultiple
InputTooLarge
NumberTooLarge
Io
```

Exemples de commandes invalides :

```bash
# clé hexadécimale de longueur impaire
echo -n "abc" | ./my_pgp aes -c 123

# message trop court en mode bloc AES
echo -n "abc" | ./my_pgp aes -c -b 00112233445566778899aabbccddeeff

# système inconnu
echo -n "abc" | ./my_pgp unknown -c 00
```

Afficher le code de sortie :

```bash
echo -n "abc" | ./my_pgp aes -c -b 00112233445566778899aabbccddeeff
echo $?
```

Le code doit être `84` en cas d'erreur.

---

## 11. Organisation des fichiers

```text
src/main.rs                 point d'entrée
src/lib.rs                  modules publics et réexports
src/cli/args.rs             parsing et validation
src/cli/dispatch.rs         routage et orchestration
src/io/stdin.rs             lecture contrôlée de stdin
src/io/hex.rs               encodage/décodage hexadécimal
src/io/little_endian.rs     conversions u128 little-endian
src/domain/types.rs         types de données et effacement mémoire
src/domain/error.rs         erreurs du projet
src/domain/rsa.rs           abstraction RsaCipher
src/symmetric/xor.rs        XOR bloc et stream
src/symmetric/aes/block.rs  AES bloc
src/symmetric/aes/stream.rs AES sur plusieurs blocs
src/symmetric/aes/sbox.rs   S-box et S-box inverse
src/symmetric/aes/gf256.rs  multiplication dans GF(2^8)
src/symmetric/aes/round.rs  transformations AES
src/symmetric/aes/key_schedule.rs expansion de clé
src/bignum/*.rs             grand entier et arithmétique modulaire
src/rsa/cipher.rs           chiffrement RSA
src/rsa/keygen.rs           génération de clés
src/rsa/prime.rs            Miller-Rabin
src/rsa/random.rs           aléatoire et candidats premiers
tests/                      tests unitaires, intégration et sécurité
```

`Cargo.toml` ne déclare aucune dépendance externe. Le `Makefile` appelle :

```make
cargo build --release
cp target/release/my_pgp ./my_pgp
```

---

## 12. Tests recommandés

Compiler :

```bash
make
```

Lancer toute la suite :

```bash
cargo test --release
```

Vérifier le test AES bloc :

```bash
result=$(echo -n "All men must die" | \
    ./my_pgp aes -c -b 57696e74657220697320636f6d696e67)
test "$result" = "744ce22c385958348f0df26eceb62eef"
```

Vérifier un aller-retour XOR :

```bash
message="message de test"
ciphertext=$(printf '%s' "$message" | ./my_pgp xor -c 01020304)
decoded=$(printf '%s\n' "$ciphertext" | ./my_pgp xor -d 01020304)
printf '%s\n' "$decoded" | od -An -t x1
```

Vérifier RSA :

```bash
./my_pgp rsa -g d3 e3
echo -n "WF" | ./my_pgp rsa -c 0101-19bb
echo -n "8f84" | ./my_pgp rsa -d 9d5b-19bb
```

---

## 13. Résumé en une phrase par algorithme

- **XOR** : chaque octet du message est combiné avec un octet de clé par XOR.
- **AES** : un bloc de 16 octets subit SubBytes, ShiftRows, MixColumns et
  AddRoundKey pendant plusieurs tours.
- **RSA** : un grand entier est élevé à une puissance modulo `n`.
- **BigUint** : fournit les additions, multiplications, divisions et calculs
  modulaires nécessaires à RSA.
- **PGP** : RSA protège une clé symétrique, puis XOR ou AES protège le message.
- **Mode bloc** : un seul bloc, taille exacte.
- **Mode stream du projet** : plusieurs blocs indépendants, dernier bloc
  complété par des zéros.

Le point essentiel est donc le suivant :

```text
entrée stdin
    -> parsing CLI
    -> lecture et validation
    -> conversion hexadécimale / little-endian
    -> algorithme choisi
    -> encodage hexadécimal
    -> stdout
```

