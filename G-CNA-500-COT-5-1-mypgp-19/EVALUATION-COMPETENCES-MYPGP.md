# Evaluation des competences - my_PGP

**Date :** 2026-10-02  
**Base de l'evaluation :** code source, tests du depot, README, TODO et documentation existante.  
**Regle XP :** niveau 0 = 0-99, niveau 1 = 100-199, niveau 2 = 200-299, niveau 3 = 300-399, niveau 4 = 400-499, niveau max = 500 XP ou plus.

> Les fonctionnalites seulement annoncees dans le HTML, le README ou un planning ne sont pas considerees comme totalement acquises sans preuve dans le code et les tests.

## Synthese

| Competence | XP actuel | Niveau | XP manquants pour le max |
|---|---:|---|---:|
| Proposer une alternative ou une solution corrective | 240 | Niveau 2 | 260 |
| Justifier ou challenger un choix technique | 320 | Niveau 3 | 180 |
| Evaluer les impacts legal, ethique, business et UX | 160 | Niveau 1 | 340 |
| Expliquer les algorithmes cryptographiques | 360 | Niveau 3 | 140 |
| Concevoir une architecture cryptographique hybride | 330 | Niveau 3 | 170 |
| Implementer un cryptosysteme public avec multiprecision | 300 | Niveau 3 | 200 |
| Implementer des chiffrements symetriques binaires precis | 430 | Niveau 4 | 70 |
| **Total** | **2 140 / 3 500** | **Progression estimee : 61 %** | **1 360** |

L'interface affiche 0/12 achievements, mais seulement sept competences sont detaillees dans les donnees fournies. Cette evaluation porte donc sur ces sept competences et ne pretend pas noter les cinq achievements non decrits.

## Etat verifie du depot

### Points effectivement presents

- Rust 2021 sans dependance externe dans [Cargo.toml](Cargo.toml).
- `#![forbid(unsafe_code)]` dans [src/lib.rs](src/lib.rs).
- Codec hex et conversions little-endian dans `src/io/`.
- `BigUint` maison avec addition, soustraction, multiplication, division, `powmod`, PGCD et inverse modulaire dans `src/bignum/`.
- XOR bloc et flux dans `src/symmetric/xor.rs`.
- AES-128, AES-192 et AES-256, expansion de cle, tours et flux dans `src/symmetric/aes/`.
- Generation RSA avec fonction de Carmichael et choix decroissant des nombres de Fermat dans [src/rsa/keygen.rs](src/rsa/keygen.rs).
- Chiffrement et dechiffrement RSA dans [src/rsa/cipher.rs](src/rsa/cipher.rs).
- Routage CLI RSA et PGP present dans [src/cli/dispatch.rs](src/cli/dispatch.rs).
- Tests unitaires, acceptance, integration, E2E, regression, security, static et performance.
- `cargo test --release` execute avec succes apres ajout des preuves CLI RSA/PGP du PDF.

## Repartition basee sur le travail realise

Cette repartition reprend les responsabilites indiquees dans le HTML d'analyse,
le README et le contexte du depot. Elle ne suppose pas que les deux membres ont
travaille sur les memes modules.

| Membre | Contributions constatees | Competences principalement demontrees |
|---|---|---|
| **Karyl** | Codec hex, lecture stdin, XOR bloc/flux, AES-128/192/256, tests AES/XOR, parsing CLI, Makefile et orchestration PGP | Chiffrement symetrique precis, UX CLI, architecture modulaire, tests de compatibilite |
| **Fredrich** | `BigUint`, arithmetique multiprecision, conversions numeriques, Carmichael, generation RSA, chiffrement/dechiffrement RSA, Miller-Rabin, erreurs et tests bignum/RSA | Cryptographie a cle publique, mathematiques modulaires, robustesse des erreurs, validation numerique |
| **Binome** | Contrats de types, tests d'integration, acceptance, regression, securite, performance, documentation et revue croisee | Decisions techniques, analyse des compromis, qualite et architecture hybride |

### Regle de repartition pour la suite

- Karyl reste proprietaire des couches `io`, `symmetric` et `cli`.
- Fredrich reste proprietaire des couches `bignum` et `rsa`.
- PGP est une responsabilite partagee : Karyl gere le branchement avec AES/XOR et Fredrich valide l'enveloppe RSA.
- Les tests E2E, les formats de protocole et la documentation sont valides par les deux membres.
- Toute modification d'un contrat partage (`MyPgpError`, `RsaCipher`, types de cle) necessite une revue croisee.

### Cohérence vérifiée

Le depot contenait trois niveaux de verite differents :

- [README.md](README.md) presente RSA et PGP comme implementes et verifies.
- [tests/e2e/main.rs](tests/e2e/main.rs) contient maintenant les vecteurs RSA et PGP-AES du PDF.
- [TODO.md](TODO.md) indique maintenant la fin des étapes obligatoires RSA/PGP.
- [src/cli/dispatch.rs](src/cli/dispatch.rs) contient deja des chemins RSA et PGP.

La contradiction a été supprimée par des tests CLI RSA/PGP réels et la mise à jour des documents de suivi.

## Evaluation detaillee

### 1. Proposer une alternative ou une solution corrective - 240 XP, niveau 2

**Preuves :** le HTML compare C, C++, Python, Java, Rust et Haskell ; le projet explique le choix d'une implementation from scratch et reconnait les limites du RSA textbook.

**Limites :** les alternatives ne sont pas formalisees dans des ADR ; les formats de cle, les bibliotheques externes et les alternatives TLS/SSH ne sont pas compares dans le depot actif.

**Pour atteindre 500 XP :** produire des comparaisons decisionnelles avec contraintes, couts, risques, solution retenue et condition de retour en arriere.

### 2. Justifier ou challenger un choix technique - 320 XP, niveau 3

**Preuves :** architecture Rust sans dependance, typage de `SymmetricKey` et `Plaintext`, zero `unsafe`, erreurs metier structurees, representation multiprecision maison et fonction de Carmichael.

**Limites :** les choix ne sont pas regroupes dans des ADR ; la preuve experimentale des compromis performance/securite reste limitee ; la coherence RSA/PGP dans la CLI doit etre prouvee par des tests E2E.

**Pour atteindre 500 XP :** documenter les alternatives, ajouter des benchmarks et rendre explicites les criteres de decision.

### 3. Evaluer les impacts legal, ethique, business et UX - 160 XP, niveau 1

**Preuves :** le README indique que le projet est pedagogique et que RSA textbook n'est pas securise en production ; les tests verifient des erreurs CLI, stdout/stderr et le code 84.

**Limites :** absence de threat model complet, de licence explicite, d'analyse des usages abusifs, de responsabilite, de maintenance, de compatibilite et de risques business.

**Pour atteindre 500 XP :** ajouter une documentation sur les menaces, les utilisateurs, les donnees sensibles, la responsabilite, la licence, la divulgation de vulnerabilites et l'ergonomie CLI.

### 4. Expliquer les algorithmes cryptographiques - 360 XP, niveau 3

**Preuves :** implementation et tests de S-box, GF(2^8), key schedule AES, tours AES, XOR, bignum, Carmichael, inverse modulaire, Miller-Rabin et endianness.

**Limites :** il manque une verification systematique des proprietes de securite du protocole hybride : authentification, padding, reutilisation de cle, malleabilite et attaque sur RSA textbook.

**Pour atteindre 500 XP :** relier chaque primitive a ses garanties et limites, ajouter les vecteurs de reference externes et documenter les attaques que l'implementation ne cherche pas a empecher.

### 5. Concevoir une architecture cryptographique hybride - 330 XP, niveau 3

**Preuves :** separation `io`, `bignum`, `rsa`, `symmetric`, `domain` et `cli` ; format de cle composite ; orchestration RSA pour la cle symetrique et AES/XOR pour le message dans [src/cli/dispatch.rs](src/cli/dispatch.rs).

**Limites :** PGP n'est pas encore suffisamment demontre par des tests E2E independants ; il n'y a pas d'authentification du message, de version de format, de nonce ou de gestion de cle de session aleatoire documentee.

**Pour atteindre 500 XP :** stabiliser le protocole, ajouter des types de domaine dedies, une authentification, des tests de troncature/modification et un diagramme de flux.

### 6. Implementer un cryptosysteme public avec multiprecision - 300 XP, niveau 3

**Preuves :** `BigUint` maison, operations arithmetiques, `powmod`, PGCD, inverse, Miller-Rabin, generation RSA et tests RSA unitaires.

**Limites :** certains tests E2E considerent encore RSA comme non implemente ; il manque une campagne CLI complete sur les grandes tailles et des garanties plus explicites sur les erreurs de format et les limites de taille.

**Pour atteindre 500 XP :** valider generation, chiffrement et dechiffrement ensemble sur 256, 512, 1024 et 2048 bits, ajouter des benchmarks et separer les types de cle publique/privee.

### 7. Implementer des chiffrements symetriques binaires precis - 430 XP, niveau 4

**Preuves :** XOR bloc/flux, AES 128/192/256, lecture binaire, suppression controlee du LF final, padding zero, tests FIPS-197, tests de regression, securite, performance et round-trip.

**Limites :** le padding zero ne preserve pas toujours la longueur originale lorsque le message finit par des octets nuls ; les modes CBC, CTR, AEAD et l'integrite ne sont pas presents.

**Pour atteindre 500 XP :** ajouter un format de longueur ou padding non ambigu, des tests binaires avec `0x00`, du streaming verifie et une authentification des donnees.

# Roadmap vers le niveau max

## Phase 1 - Coherence et preuves E2E (terminée hors bonus)

**Objectif :** supprimer les contradictions et securiser la base de l'evaluation.

- [x] **Fredrich** : remplacer le test RSA "not yet implemented" dans [tests/e2e/main.rs](tests/e2e/main.rs).
- [x] **Fredrich** : ajouter `rsa -g d3 e3`, `WF -> 8f84 -> WF` et la couverture RSA multiprécision existante.
- [x] **Karyl** : ajouter le test CLI `pgp-aes` sur les deux lignes de sortie.
- [x] **Binome** : valider l'enveloppe RSA et le protocole PGP-AES avec les vecteurs du PDF.
- [x] **Binome** : mettre a jour [TODO.md](TODO.md), [CONTEXT.md](CONTEXT.md) et la documentation des tests.

**Critere de sortie :** atteint; `cargo test --release` passe avec les preuves RSA/PGP réelles.

Les signatures, X25519, OAEP, les modes CBC/CTR, le padding et l'authentification sont des bonus ou des extensions et restent volontairement hors de cette livraison.

## Phase 2 - RSA multiprecision robuste

**Objectif :** passer de 300 XP a environ 500 XP.

- [ ] **Fredrich** : tester `BigUint` sur zero, limites de mots, valeurs multi-mots et invariants `a = q*b + r`.
- [ ] **Fredrich** : tester RSA sur des cles 256, 512, 1024 et 2048 bits.
- [ ] **Fredrich** : ajouter les benchmarks de multiplication, division et `powmod`.
- [ ] **Fredrich** : tester `p == q`, `p <= 1`, modulus nul, cle mal formee et message `m >= n`.
- [ ] **Binome** : separer `RsaPublicKey`, `RsaPrivateKey` et `RsaKeyPair`, puis revoir le contrat partage.
- [ ] **Binome** : documenter clairement RSA textbook et son absence de padding.
- [ ] **Fredrich** : etudier OAEP ; **binome** : decider de son implementation ou de son exclusion du perimetre.

**Critere de sortie :** generation, chiffrement et dechiffrement valides de bout en bout sur plusieurs tailles.

## Phase 3 - Symetrique binaire sans ambiguite

**Objectif :** passer de 430 XP au niveau max.

- [ ] **Karyl** : tester messages vides, messages multi-blocs et messages finissant par `0x00`.
- [ ] **Karyl** : ajouter une longueur originale ou un padding explicite pour les formats qui le permettent.
- [ ] **Karyl** : ajouter une API de streaming qui ne charge pas tout le fichier en memoire.
- [ ] **Binome** : ajouter des property tests : `decrypt(encrypt(message)) == message`.
- [ ] **Karyl** : mesurer les performances sur des fichiers d'au moins 1 MiB et 64 MiB.
- [ ] **Binome** : documenter les differences entre chiffrement, integrite et authentification.

**Critere de sortie :** toute donnee binaire valide est restauree sans ambiguite et sans fuite de memoire.

## Phase 4 - Architecture PGP verifiable et authentifiee

**Objectif :** passer de 330 XP a 500 XP.

- [ ] **Binome** : definir une structure PGP versionnee et documentee.
- [ ] **Karyl** : separer parsing, chiffrement symetrique et serialisation ; **Fredrich** : valider l'enveloppe RSA.
- [ ] **Binome** : utiliser une source aleatoire adaptee pour les cles de session.
- [ ] **Binome** : ajouter HMAC-SHA-256 ou une AEAD complete pour detecter les modifications.
- [ ] **Karyl** : rejeter les sorties tronquees cote CLI ; **Fredrich** : valider les erreurs de dechiffrement RSA.
- [ ] **Binome** : ajouter un diagramme de flux et des tests de compatibilite du format.

**Critere de sortie :** un ciphertext modifie ou tronque est refuse, et un message valide est dechiffre sans ambiguite.

## Phase 5 - Decisions techniques et alternatives

**Objectif :** passer de 320/240 XP a 500 XP en decision technique.

Creer `docs/decisions/` avec au minimum :

- [ ] **Binome** : Rust sans dependance cryptographique.
- [ ] **Fredrich** : `BigUint` maison contre bibliotheque multiprecision.
- [ ] **Fredrich** : RSA textbook contre OAEP.
- [ ] **Karyl** : XOR/AES sans authentification contre chiffrement authentifie.
- [ ] **Karyl** : padding zero contre longueur explicite ou PKCS#7.
- [ ] **Binome** : format de cle texte contre JSON, PEM ou DER.
- [ ] **Binome** : PGP maison contre TLS/SSH pour les objectifs du projet.

Chaque decision doit contenir : contexte, options, contraintes, choix, consequences, risques et condition de reexamen.

## Phase 6 - Impacts legal, ethique, business et UX

**Objectif :** passer de 160 XP a 500 XP.

- [ ] **Fredrich** : creer `docs/security/threat-model.md` sur les actifs, adversaires et garanties cryptographiques.
- [ ] **Binome** : documenter les limites, l'interdiction d'utiliser le projet pour des donnees reelles et la politique de divulgation.
- [ ] **Binome** : ajouter une licence et analyser les risques business de maintenance et de compatibilite.
- [ ] **Karyl** : documenter les messages CLI, les codes 84 et les comportements stdin/stdout/stderr.
- [ ] **Binome** : ajouter les tests d'ergonomie et faire une revue finale croisee.

**Critere de sortie :** un nouvel utilisateur peut comprendre ce qui est protege, ce qui ne l'est pas et comment utiliser le programme sans interpretation dangereuse.

## Ordre de priorite recommande

1. Coherence des tests RSA/PGP et mise a jour du suivi.
2. Preuves E2E RSA et PGP.
3. Robustesse et benchmarks `BigUint`.
4. Format binaire non ambigu et streaming.
5. Authentification et durcissement du protocole PGP.
6. ADR et comparaisons d'alternatives.
7. Threat model, impacts legal/ethique/business et UX.

## Definition de fini niveau max

Le projet peut etre considere au niveau max lorsque :

- toutes les fonctionnalites revendiquees sont testees via la CLI et les APIs internes ;
- RSA fonctionne sur plusieurs tailles de cle avec round-trip complet ;
- PGP detecte les modifications et les troncatures ;
- les donnees binaires sont restaurees sans ambiguite ;
- les choix techniques sont compares dans des ADR ;
- les risques, limites, usages interdits et impacts utilisateur sont documentes ;
- `cargo test --release`, les benchmarks et les tests de regression sont verts ;
- README, TODO, CONTEXT, RESUME et cette evaluation decrivent le meme etat.
