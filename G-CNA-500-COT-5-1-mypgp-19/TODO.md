# TODO - Karyl (périmètre : codec hex, XOR, AES, CLI, Makefile, PGP)

Détail complet des sous-étapes livrées : `RESUME.md`. Ce fichier ne liste que ce qui reste.

## Étape 4 — AES (terminée)
- [x] 4.1 : GF(2⁸) et S-box calculées à la compilation
- [x] 4.2 : expansion de clé 128/192/256
- [x] 4.3 : transformations d'un tour et leurs inverses
- [x] 4.4 : chiffrement d'un bloc + couche « format du sujet »
- [x] 4.5 : déchiffrement d'un bloc
- [x] 4.6 : mode flux (padding zéro du dernier bloc)

## Étape 5 — CLI (terminée)
- [x] Parsing des arguments (`src/cli/args.rs`)
- [x] Routage `xor`/`aes`, modes bloc et flux (`src/cli/dispatch.rs`)
- [x] Routage `rsa`, `pgp-xor`, `pgp-aes` et code 84 sur erreur
- [x] Tests E2E sur les vecteurs exacts du PDF (XOR p. 4, AES p. 5)
- [x] Revue des types partagés (`NumberTooLarge`, `CiphertextLengthNotMultiple`, `RsaCipher`)

## Étape 6 — PGP (`pgp-xor`, `pgp-aes`) (terminée)
- [x] Format de clé composite `SYM_KEY:RSA_KEY`
- [x] Clé symétrique fournie par l'utilisateur selon le protocole du sujet
- [x] Chiffrement de la clé symétrique par RSA (interface `RsaCipher`)
- [x] Sortie sur deux lignes (clé chiffrée puis message chiffré)
- [x] Vecteur du sujet `pgp-aes -c -b` + round-trip CLI

## Bonus (après la partie obligatoire, cf. `TODO_BEYOND_SUBJECT_PERSON_A.md`)
- [ ] AES-CBC, AES-CTR
- [ ] Padding PKCS#7, HMAC-SHA256
- [ ] HKDF, dérivation de clé par mot de passe
