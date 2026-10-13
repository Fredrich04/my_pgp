# TODO "Au-delà du sujet" — Karyl

> **Référence** : `G-CNA-500-my_pgp.pdf` fait foi (voir § 0 du plan HTML et § 37 de REGLES.md).
>
> **Contraintes** :
>
> - Démarrer uniquement lorsque la partie obligatoire (étapes 0 à 8 du plan de Karyl) est terminée et que tous les vecteurs du sujet passent.
> - Tout bonus vit dans le dossier `bonus/`, avec son propre `Makefile` (exigence du sujet p. 1).
> - Aucun bonus ne doit casser la partie obligatoire : les exemples du sujet doivent toujours passer (sujet p. 10, rétrocompatibilité).
> - Cryptographie entièrement from scratch : aucune crate cryptographique.
> - Respecter REGLES.md pour chaque ligne de code ; toute dérogation est annoncée puis ajoutée au § 37.
> - Une sous-tâche atomique = documentation (`code.md` + `explication.md` du sous-dossier) + un commit `type(scope): subject` sans co-auteur + push, puis validation avant la suivante.

---

## ♻️ Historique : éléments abandonnés le 2026-09-24

L'ancienne version de ce fichier reposait sur une application de chiffrement de fichiers (JSON, AES-GCM, RSA-OAEP, crates `aes`/`aes-gcm`/`rsa`, options `--key-size`, `--mode`, `--streaming`, `--password`, `--authenticated`). Cette architecture est incompatible avec le sujet et est remplacée. Sont notamment abandonnés :

- la phase 7.1 « AES-192/256 », cochée à tort : elle utilisait `Aes256Gcm` pour AES-192 et ne contenait ni key schedule ni vecteur NIST. AES-192/256 fait désormais partie de la **partie obligatoire** (étape 4 du plan : le sujet dit « should also implement ») ;
- la phase 7.2 « AES-CBC » en cours (code non compilable, padding oracle) ;
- tout champ ajouté à `EncryptedMessage` et toute option CLI de type `--xxx`.

---

## 📦 Phase 9 : Modes d'opération AES (bonus)

**Branche** : `bonus/aes-modes` — **Dossier** : `bonus/`

> ✅ Décidé le 2026-09-24 : chaque mode est un nouveau `CRYPTO_SYSTEM` (`./my_pgp aes-cbc -c KEY`). Syntaxe du mode CTR à confirmer avant la phase 9.2.

### 9.1 Mode CBC

- [ ] **9.1.1** : Créer le squelette `bonus/` (Makefile `all`/`re`/`clean`/`fclean`, crate dépendant de la bibliothèque principale)
- [ ] **9.1.2** : Type `Iv` (16 octets, validé à la construction, masqué dans `Debug`)
- [ ] **9.1.3** : Chiffrement CBC réutilisant le chiffrement de bloc AES maison
- [ ] **9.1.4** : Déchiffrement CBC
- [ ] **9.1.5** : Test : vecteurs NIST SP 800-38A F.2 (AES-128/192/256)
- [ ] **9.1.6** : Branchement CLI dans le binaire bonus : `CRYPTO_SYSTEM` `aes-cbc`

### 9.2 Mode CTR

- [ ] **9.2.1** : Type `Counter` (bloc de 128 bits, incrément big-endian sur les 32 bits de poids faible)
- [ ] **9.2.2** : Chiffrement/déchiffrement CTR (opération identique)
- [ ] **9.2.3** : Test : vecteurs NIST SP 800-38A F.5
- [ ] **9.2.4** : Branchement CLI dans le binaire bonus

---

## 📦 Phase 10 : Schémas de padding (bonus cité par le sujet p. 10)

**Branche** : `bonus/padding` — **Dossier** : `bonus/`

- [ ] **10.1** : `pkcs7_pad`
- [ ] **10.2** : `pkcs7_unpad`, avec une erreur unique quelle que soit la cause (pas d'oracle, REGLES.md § 27.3)
- [ ] **10.3** : Tests : toutes les longueurs de 0 à 16 octets manquants, padding corrompu
- [ ] **10.4** : Intégration avec CBC (phase 9)

---

## 📦 Phase 11 : Authentification (bonus)

**Branche** : `bonus/authentication` — **Dossier** : `bonus/`

> ⏸️ Reporté le 2026-09-24 : le partage de SHA-256 avec Frédrich sera décidé plus tard.

- [ ] **11.1** : SHA-256 from scratch (ou réutilisation de celui de Frédrich), test FIPS 180-4
- [ ] **11.2** : HMAC-SHA256, test RFC 4231
- [ ] **11.3** : Comparaison de tags en temps constant
- [ ] **11.4** : Encrypt-then-MAC sur CBC/CTR, rejet de tout message altéré
- [ ] **11.5** : HKDF-SHA256 (extract + expand), test RFC 5869 cas 1

---

## 📦 Phase 12 : Tests renforcés (bonus)

**Branche** : `bonus/property-testing`

> ✅ Décidé le 2026-09-24 : pas de `proptest`. Les entrées aléatoires sont produites par un générateur pseudo-aléatoire déterministe écrit à la main, à graine fixe, pour des tests reproductibles.

- [ ] **12.0** : Générateur pseudo-aléatoire déterministe maison, à graine fixe (tests uniquement)
- [ ] **12.1** : Propriété aller-retour `d(c(m)) == m` pour XOR et AES, toutes tailles de clé
- [ ] **12.2** : Propriété de déterminisme (même clé, même message → même sortie)
- [ ] **12.3** : Mesure de l'effet avalanche d'AES (1 bit modifié → environ 50 % de bits différents)

---

## 📦 Phase 13 : Documentation des bonus

**Branche** : `docs/person-a-encryption`

- [ ] **13.1** : `bonus/README.md` : liste des bonus, syntaxe, exemples
- [ ] **13.2** : Mettre à jour le README principal (renvoi vers `bonus/`)
- [ ] **13.3** : Vérification finale : `make re` (racine et `bonus/`), `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, vecteurs du sujet toujours verts

---

## 🎯 Critères de succès

- [ ] Tous les exemples du sujet passent toujours avec le binaire principal
- [ ] `make -C bonus re` produit un binaire bonus fonctionnel
- [ ] CBC et CTR passent les vecteurs NIST SP 800-38A
- [ ] PKCS#7 sans oracle de padding
- [ ] HMAC et HKDF passent leurs vecteurs RFC
- [ ] Aucune crate cryptographique, 0 warning `clippy`
- [ ] Toutes les dérogations éventuelles listées dans REGLES.md § 37

---

**Création** : 2026-09-21 — **Réécriture** : 2026-09-24 (alignement sur le sujet)
