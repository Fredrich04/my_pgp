# my_pgp — A Song of Ciphers and Primes

Projet Epitech G-CNA-500 : implémentation from scratch, en Rust, des cryptosystèmes XOR, AES, RSA et PGP décrits dans `G-CNA-500-my_pgp.pdf`.

Zéro dépendance externe, zéro `unsafe` (`#![forbid(unsafe_code)]`).

## État

Les cinq cryptosystèmes du sujet sont implémentés et vérifiés contre tous les vecteurs du PDF :

- **XOR** : modes bloc (`-b`) et flux (padding zéros), `xor -c/-d [-b] KEY`
- **AES-128/192/256** : S-box, expansion de clé, tours FIPS-197, bloc et flux, `aes -c/-d [-b] KEY`
- **RSA** : arithmétique multiprécision maison (`src/bignum`), génération de clés par fonction de Carmichael + plus grand Fermat valide, chiffrement/déchiffrement, `rsa -g P Q` / `rsa -c e-n` / `rsa -d d-n` (testé jusqu'à 2048-bit)
- **PGP (`pgp-xor`, `pgp-aes`)** : hybride réel — message chiffré en symétrique, clé symétrique chiffrée en RSA ; sortie sur deux lignes (clé chiffrée, puis message chiffré)

## Compilation

```sh
make        # build release + copie ./my_pgp
make clean  # cargo clean
make fclean # clean + supprime ./my_pgp
make re     # rebuild complet
```

## Utilisation

```
./my_pgp CRYPTO_SYSTEM MODE [-b] KEY
./my_pgp rsa -g P Q
```

- `CRYPTO_SYSTEM` : `xor`, `aes`, `rsa`, `pgp-xor`, `pgp-aes`
- `MODE` : `-c` (chiffrer), `-d` (déchiffrer), `-g P Q` (génération RSA uniquement)
- `-b` : mode bloc — message et clé symétrique de même taille (incompatible avec `-g`)
- `KEY` : hexadécimal little-endian ; `e-n` / `d-n` pour RSA ; `SYMMETRIC_KEY:RSA_KEY` pour PGP

Le message est lu sur `stdin` (le `\n` final d'`echo` est exclu). En chiffrement, la sortie est en hexadécimal minuscule sur `stdout`. Toute erreur va sur `stderr` avec le code de sortie **84**.

## Exemples (vecteurs du sujet, tous rejoués OK)

```sh
echo "You know nothing, Jon Snow" > message
./my_pgp xor -c -b 576861742069732064656164206d6179206e6576657220646965 < message
# 20070f2700071c6a4449060a490515164e4e12190b190011063c

echo "All men must die" > message
./my_pgp aes -c -b 57696e74657220697320636f6d696e67 < message
# 744ce22c385958348f0df26eceb62eef

./my_pgp rsa -g d3 e3
# public key: 0101-19bb
# private key: 9d5b-19bb
echo "WF" | ./my_pgp rsa -c 0101-19bb
# 8f84

echo "All men must die" | ./my_pgp pgp-aes -c -b 57696e74657220697320636f6d696e67:010001-c9f91a9ff3bd6d84005b9cc8448296330bd23480f8cf8b36fd4edd0a8cd925de139a0076b962f4d57f50d6f9e64e7c41587784488f923dd60136c763fd602fb3
# 97f2af4c1b712008c1e46935f446756443a8a700f20581d138e4e6916afe5c5f9b9d6eaa0a870374b686f1a024f9bbb88c23c766654579339caf55afd149d41d
# 744ce22c385958348f0df26eceb62eef
```

## Tests

```sh
cargo test --release   # 13 suites (unit, acceptance, e2e, security, static…), 0 échec
```

## Documentation

Chaque dossier de `src/` contient `code.md` (technique) et `explication.md` (pédagogique) — exigence REGLES.md § 35, vérifiée par test statique :

- `src/cli/` : parsing, routage, protocole PGP
- `src/rsa/` : keygen (Carmichael, Fermat), chiffrement, Miller-Rabin
- `src/bignum/` : entiers multiprécision, conversions little-endian
- `src/domain/` : erreurs typées, types sensibles (`SymmetricKey`, `Plaintext`), contrat `RsaCipher`
- `src/symmetric/`, `src/io/` : AES/XOR, codec hex, lecture stdin

`REGLES.md` : règles de développement. `RESUME.md` / `TODO.md` : suivi d'avancement.

## Sécurité — limites assumées (pédagogique, pas production)

- **RSA textbook sans padding** : malléable, déterministe — **NON SÉCURISÉ** en production
- **Pas d'authentification** : ni AEAD sur AES, ni signatures (`-s` non implémenté)
- **`powmod` non constant-time** ; effacement mémoire en `black_box` (effort maximal, pas garantie formelle)
- **XOR** : sûr uniquement à usage unique avec clé aussi longue que le message

## Contributeurs

- Karyl : codec hexadécimal, XOR, AES, CLI, Makefile, composition PGP
- Frédrich : arithmétique multiprécision, RSA (keygen, chiffrement, primalité), tests bignum/RSA
