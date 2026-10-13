# `src/symmetric/aes` — documentation technique

AES implémenté entièrement à la main (décision du 2026-09-24 : aucune crate), d'après FIPS-197.

## Contenu

| Fichier | Rôle |
|---|---|
| `mod.rs` | Déclare les sous-modules |
| `gf256.rs` | Arithmétique du corps GF(2⁸) : `gf_mul`, `gf_inverse` |
| `key_schedule.rs` | Expansion de clé (`expand_key`, `RoundKeys`, `BLOCK_BYTES`) |
| `round.rs` | Transformations d'un tour et leurs inverses |
| `sbox.rs` | S-box et S-box inverse |
| `block.rs` | Chiffrement/déchiffrement d'un bloc de 16 octets : tours FIPS-197 + couche « format du sujet » |
| `stream.rs` | Mode flux : blocs de 16 octets, dernier bloc complété par des zéros (sujet p. 5) |

## `mod.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1–6 | Une ligne `pub mod` par fichier (`block`, `gf256`, `key_schedule`, `round`, `sbox`, `stream`) : chaque sous-module est public pour que la CLI et les tests y accèdent. |

## `gf256.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1 | `const REDUCTION: u8 = 0x1b` : le polynôme d'AES x⁸+x⁴+x³+x+1 privé de son bit x⁸ ; sert à « replier » un octet qui déborde (FIPS-197 § 4.2). |
| 3 | `gf_mul` (`gf256.rs:3 gf_mul`) : multiplication dans GF(2⁸), `const fn` pour pouvoir construire les tables à la compilation. |
| 4–6 | Copies de travail `a`, `b` et accumulateur `product` à 0. |
| 7 | Boucle tant qu'il reste des bits à 1 dans `b` (multiplication « à la russe », bit par bit). |
| 8–10 | Si le bit de poids faible de `b` vaut 1, on ajoute (XOR) `a` au produit : l'addition dans GF(2⁸) est un XOR. |
| 11 | `carry` : le bit de poids fort de `a` (0x80) va déborder au décalage. |
| 12 | `a <<= 1` : multiplie `a` par x. |
| 13–15 | En cas de débordement, XOR avec `REDUCTION` pour rester sur 8 bits. |
| 16 | `b >>= 1` : passe au bit suivant du multiplicateur. |
| 18 | Rend le produit. |
| 21 | `gf_inverse` (`gf256.rs:21 gf_inverse`) : inverse multiplicatif, calculé comme `valeur^254` (petit théorème de Fermat dans GF(2⁸) : v²⁵⁵ = 1). `gf_inverse(0)` vaut 0, convention d'AES. |
| 22–24 | `result = 1`, `base = valeur`, `exponent = 254`. |
| 25–31 | Exponentiation rapide : si le bit courant de l'exposant est 1, `result *= base` ; puis `base` au carré et exposant décalé. |

## `sbox.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 1 | Importe `gf_inverse`. |
| 3 | `AFFINE_CONSTANT = 0x63` : constante de la transformation affine (FIPS-197 § 5.1.1). |
| 5 | `SBOX` (`sbox.rs:5 SBOX`) : tableau de 256 octets calculé **à la compilation** par `build_sbox`. |
| 6 | `INV_SBOX` (`sbox.rs:6 INV_SBOX`) : tableau inverse calculé par `invert`. |
| 8–10 | `sub_byte` : lit la S-box à l'indice de la valeur. |
| 12–14 | `inv_sub_byte` : même chose avec la S-box inverse. |
| 16–23 | `affine` : XOR de la valeur, de ses quatre rotations à gauche (1 à 4 bits) et de `0x63`. |
| 25–33 | `build_sbox` : pour chaque octet 0..255, inverse dans GF(2⁸) puis transformation affine. |
| 35–43 | `invert` : place `index` à l'emplacement `table[index]`, ce qui construit la permutation réciproque. |

## `key_schedule.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 7 | `BLOCK_BYTES = 16` : taille d'un bloc AES. |
| 8 | `WORD_BYTES = 4` : taille d'un mot de la clé. |
| 10–13 | `RoundKeys` : toutes les clés de tour mises bout à bout (`bytes`) et le nombre de tours. Champs privés : invariants protégés. |
| 16–18 | `rounds()` : nombre de tours (10, 12 ou 14). |
| 20–22 | `round_key(n)` : la n-ième clé de 16 octets, `None` si hors limites (pas de panique). |
| 25–29 | `Debug` masqué : n'affiche jamais les octets, seulement le nombre de tours. |
| 31–35 | `Drop` : efface (`wipe`) les clés de tour à la destruction (secrets). |
| 37 | `expand_key` (`key_schedule.rs:37 expand_key`) : produit toutes les clés de tour (FIPS-197 § 5.2). |
| 38–43 | Nombre de mots de la clé `Nk` : 4, 6 ou 8 selon 16, 24 ou 32 octets ; autre taille → `InvalidKeyLength`. |
| 44 | Nombre de tours `Nr = Nk + 6`. |
| 45 | Nombre total de mots à produire : `(Nr+1) × 4`. |
| 46–47 | Tampon pré-alloué, initialisé avec la clé d'origine. |
| 48 | Constante de tour `Rcon`, démarre à 1. |
| 49 | `temp` : mot de travail de 4 octets. |
| 50 | Boucle sur chaque nouveau mot. |
| 51 | `temp` ← mot précédent. |
| 52–56 | Tous les `Nk` mots : rotation d'un octet à gauche (`RotWord`), S-box sur chaque octet (`SubWord`), XOR de `Rcon` sur le premier octet, puis `Rcon ← Rcon × 2` dans GF(2⁸). |
| 57–59 | Cas AES-256 uniquement (`Nk > 6`), 4 mots après le début du cycle : `SubWord` seul. |
| 60–63 | Nouveau mot = mot d'il y a `Nk` positions ⊕ `temp`. |
| 65 | Efface `temp` (secret dérivé de la clé). |
| 66 | Rend `RoundKeys`. |

## `round.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 6 | `State` (`round.rs:6 State`) : le bloc sous forme de 16 octets, rangés **colonne par colonne** (octet `i` → ligne `i%4`, colonne `i/4`, FIPS-197 § 3.4). |
| 8–9 | `ROWS` et `COLUMNS` valent 4. |
| 10 | `Table` : tableau de 256 octets. |
| 12–18 | `TIMES_1`, `TIMES_2`, `TIMES_3`, `TIMES_9`, `TIMES_11`, `TIMES_13`, `TIMES_14` : tables de multiplication par une constante dans GF(2⁸), précalculées à la compilation. |
| 19 | `MIX` : coefficients 2, 3, 1, 1 (matrice de MixColumns). |
| 20 | `INV_MIX` : coefficients 14, 11, 13, 9 (matrice inverse). |
| 22–24 | `sub_bytes` : S-box sur chaque octet. |
| 26–30 | `inv_sub_bytes` : S-box inverse sur chaque octet. |
| 32–34 | `shift_rows` : l'octet de la ligne `r` vient de la colonne `(c + r) mod 4`, donc la ligne `r` glisse de `r` cases vers la gauche. |
| 36–38 | `inv_shift_rows` : glissement inverse. |
| 40–42 | `mix_columns` : multiplication de chaque colonne par la matrice MixColumns. |
| 44–46 | `inv_mix_columns` : par la matrice inverse. |
| 48–53 | `add_round_key` : XOR du bloc avec la clé de tour. |
| 55–63 | `rotate_rows` : recopie les octets selon une règle de colonne source ; la copie `original` est effacée. |
| 65–76 | `multiply_columns` : pour chaque colonne, chaque octet de sortie est le XOR des 4 produits (octet d'entrée × coefficient) ; copie temporaire effacée. |
| 78–86 | `product_table` : remplit une table `valeur → facteur × valeur` avec `gf_mul`. |

## `block.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 9 | `WORD_BYTES = 4` : taille des groupes inversés pour la convention du sujet. |
| 11 | `encrypt_block` (`block.rs:11 encrypt_block`) : chiffre exactement 16 octets (mode `-b`). |
| 12 | Refuse un message qui n'a pas 16 octets. |
| 13 | Clés de tour calculées depuis la clé « format du sujet ». |
| 14–15 | Copie le message dans l'état (le message n'est **pas** un nombre : aucune inversion). |
| 16 | Applique les tours AES. |
| 17 | Inverse chaque mot de 4 octets du résultat (le chiffré est un nombre little-endian). |
| 18 | Rend les 16 octets. |
| 21 | `decrypt_block` (`block.rs:21 decrypt_block`) : opération inverse. |
| 22–25 | Contrôle de taille, clés de tour, copie du chiffré dans l'état. |
| 26 | Défait l'inversion des mots du chiffré. |
| 27 | Applique les tours inverses. |
| 28 | Enveloppe le résultat dans un `Plaintext` (effacé à la destruction). |
| 31 | `subject_round_keys` (`block.rs:31 subject_round_keys`) : clé du sujet → clés de tour. |
| 32–35 | Inverse chaque mot de 4 octets de la clé (la clé est un nombre), la reconstruit en `SymmetricKey` (revalide sa taille), puis `expand_key`. |
| 38 | `cipher_rounds` (`block.rs:38 cipher_rounds`) : les tours de FIPS-197 § 5.1. |
| 39–40 | `AddRoundKey` initial avec la clé de tour 0. |
| 41–46 | Tours 1 à `Nr-1` : `SubBytes`, `ShiftRows`, `MixColumns`, `AddRoundKey`. |
| 47–49 | Dernier tour : sans `MixColumns`. |
| 52 | `inv_cipher_rounds` (`block.rs:52 inv_cipher_rounds`) : tours inverses (FIPS-197 § 5.3). |
| 53–54 | Part de la dernière clé de tour. |
| 55–60 | Tours `Nr-1` à 1 en sens inverse : `InvShiftRows`, `InvSubBytes`, `AddRoundKey`, `InvMixColumns`. |
| 61–63 | Dernier pas : `InvShiftRows`, `InvSubBytes`, clé de tour 0. |
| 66 | `reverse_words` (`block.rs:66 reverse_words`) : inverse chaque groupe de 4 octets. |
| 72 | `ensure_block_size` (`block.rs:72 ensure_block_size`) : refuse tout bloc différent de 16 octets avec `BlockSizeMismatch`. |
| 40, 45, 49… | `unwrap_or_default()` : une clé de tour manquante donnerait une clé vide, jamais une panique (§ 12.8) ; `expand_key` garantit qu'elles existent. |

## `stream.rs` — ligne par ligne

| Ligne | Explication |
|---|---|
| 6 | `aes_stream_cipher` (`stream.rs:6 aes_stream_cipher`) : chiffre un message de longueur quelconque. |
| 7 | Sortie pré-allouée à la taille finale (`padded_len`). |
| 8 | Découpe le message en tranches de 16 octets, dans l'ordre. |
| 9–10 | Copie la tranche et la complète de zéros à sa fin. |
| 11 | Enveloppe le bloc dans un `Plaintext` (effacé en fin de tour). |
| 12 | Chiffre le bloc avec `encrypt_block` et concatène. |
| 17 | `aes_stream_decipher` (`stream.rs:17 aes_stream_decipher`). |
| 18–23 | Refuse un chiffré dont la longueur n'est pas un multiple de 16. |
| 24 | Message pré-alloué. |
| 25–27 | Déchiffre chaque bloc de 16 octets et concatène ; les zéros de complétion sont conservés. |
| 28 | Rend un `Plaintext`. |
| 31 | `padded_len` (`stream.rs:31 padded_len`) : longueur finale = arrondi au multiple de 16 supérieur ; un message vide donne un bloc (16). |

## Historique

### 4.1 — Corps GF(2⁸) et S-box (2026-09-24)

**GF(2⁸)** (FIPS-197 § 4) :

- `gf_mul(a, b)` : multiplication « russe » ; à chaque tour, `a` est doublé (`xtime`) et réduit par le polynôme `x⁸ + x⁴ + x³ + x + 1` (constante `0x1b` après débordement du bit 7). Servira aussi à MixColumns (4.3).
- `gf_inverse(x)` : `x²⁵⁴` par exponentiation rapide, puisque `x²⁵⁵ = 1` pour tout `x ≠ 0` ; `gf_inverse(0) = 0`, comme l'exige FIPS-197 § 5.1.1.

**S-box** (FIPS-197 § 5.1.1) : `S(x) = A(x⁻¹)`, où la transformation affine vaut `b ⊕ rotl(b,1) ⊕ rotl(b,2) ⊕ rotl(b,3) ⊕ rotl(b,4) ⊕ 0x63`. `INV_SBOX` est obtenue en inversant la table.

Choix de conception :

- **Calcul à la compilation** : les tables sont des `const` construites par des `const fn`. Elles ne sont pas recopiées depuis la norme (principe « from scratch ») et ne coûtent rien à l'exécution. Les boucles utilisent `while`, seule forme autorisée dans une `const fn`.
- **Indexation sûre** : `usize::from(u8)` ne dépasse jamais 255, l'indexation d'un tableau de 256 cases ne peut donc pas paniquer.
- **Limite de sécurité connue** : une S-box par table n'est pas à temps constant (fuites possibles par le cache mémoire). Le sujet ne demande pas de protection contre les canaux auxiliaires ; c'est documenté ici plutôt que caché, et relève de l'esprit de la dérogation D1 (§ 37 : AES « pédagogique » imposé par le sujet).

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | Exemples FIPS-197 de `gf_mul`, élément neutre et absorbant, commutativité, inverse, `gf_inverse(0)`, valeurs S-box FIPS, aller-retour S/S⁻¹, taille des tables |
| Intégration | S-box = construction `A(x⁻¹)` pour les 256 valeurs |
| Statique | La documentation est désormais vérifiée dans **tous** les sous-dossiers de `src/`, y compris celui-ci |
| Régression | Première et dernière lignes de la S-box, première ligne de la S-box inverse |
| E2E | **Sans objet** : aucune commande n'utilise encore AES (étape 5) |
| Acceptation | Sujet p. 5 : AES exigé, S-box standard (`S(53) = ed`) |
| Performance | 10⁷ allers-retours S/S⁻¹ en moins de 2 s |
| Sécurité | Permutation, sans point fixe ni point fixe opposé (propriétés FIPS-197) |
| Usabilité | **Sans objet** : fonctions internes sans message ni sortie utilisateur |
| Accessibilité | **Sans objet** : aucune sortie produite à ce stade |

### 4.2 — Expansion de clé (2026-09-24)

`expand_key(key: &SymmetricKey) -> Result<RoundKeys>` suit FIPS-197 § 5.2 :

| Taille de clé | Mots `Nk` | Tours `Nr` | Clés de tour |
|---|---|---|---|
| 16 octets | 4 | 10 | 11 |
| 24 octets | 6 | 12 | 13 |
| 32 octets | 8 | 14 | 15 |

- Toute autre taille → `InvalidKeyLength { bytes }`, avant tout calcul (sujet p. 5, § 17).
- Pour chaque mot `w[i]` (`i ≥ Nk`) : `temp = w[i-1]` ; si `i mod Nk = 0`, `temp = SubWord(RotWord(temp)) ⊕ Rcon` ; sinon, pour AES-256 seulement, si `i mod Nk = 4`, `temp = SubWord(temp)` ; puis `w[i] = w[i-Nk] ⊕ temp`.
- `Rcon` n'est pas une table : il part de `0x01` et est multiplié par `0x02` dans GF(2⁸) (`gf_mul`) à chaque usage.
- Les octets sont dans l'**ordre FIPS brut** : la conversion au format du sujet (mots inversés) sera faite par la couche dédiée (4.4, décision du 2026-09-24).

`RoundKeys` protège les clés de tour (§ 4, § 20) : champ privé, `Debug` réduit au nombre de tours, ni `Clone` ni `Serialize`, effacement à la destruction ; le mot temporaire `temp` est effacé à la fin du calcul ; le tampon est pré-alloué à sa taille exacte (aucune réallocation, § 20.3). `round_key(round)` renvoie `None` au-delà du dernier tour au lieu de paniquer.

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | FIPS-197 annexes A.1, A.2, A.3 (premier, dernier et tour spécifique à AES-256), `None` après le dernier tour, 10 tailles invalides |
| Intégration | Clé hexadécimale du sujet → `SymmetricKey` → 11 clés de tour |
| Statique | Règles existantes |
| Régression | Clé de tour 5 figée pour les trois tailles |
| E2E | **Sans objet** (CLI à l'étape 5) |
| Acceptation | Sujet p. 5 : 128 obligatoire, 192 et 256 acceptés |
| Performance | 10⁵ expansions AES-256 en moins de 2 s |
| Sécurité | `Debug` masqué, diffusion d'un bit de clé sur la dernière clé de tour (≥ 32 bits sur 128), erreur sans fuite |
| Usabilité | Message donnant la taille reçue |
| Accessibilité | Message ASCII sur une ligne |

### 4.3 — Transformations d'un tour (2026-09-24)

`State = [u8; 16]` rangé **colonne par colonne** (FIPS-197 § 3.4) : l'octet de la ligne `r`, colonne `c` est à l'indice `r + 4c`. C'est l'ordre des octets d'entrée, il n'y a donc aucune conversion à faire.

| Fonction | Rôle (FIPS-197) | Inverse |
|---|---|---|
| `sub_bytes` | § 5.1.1 : S-box sur chaque octet | `inv_sub_bytes` (§ 5.3.2) |
| `shift_rows` | § 5.1.2 : la ligne `r` tourne de `r` cases vers la gauche | `inv_shift_rows` (§ 5.3.1) |
| `mix_columns` | § 5.1.3 : chaque colonne multipliée par `{02 03 01 01}` circulant | `inv_mix_columns` (`{0e 0b 0d 09}`, § 5.3.3) |
| `add_round_key` | § 5.1.4 : XOR avec la clé du tour | elle-même |

Choix de conception :

- **Tables de produits à la compilation** : les multiplications par 1, 2, 3, 9, 11, 13 et 14 sont des tables `const` de 256 octets construites avec `gf_mul`. Une première version appelait `gf_mul` à chaque octet : 10⁶ allers-retours prenaient 16,6 s en debug ; avec les tables, la boucle interne n'est plus qu'une indexation.
- **Une seule fonction par famille** : `rotate_rows` et `multiply_columns` servent au sens direct et au sens inverse ; seuls le décalage ou les coefficients changent (§ 1, pas de duplication).
- **Aucune panique possible** : `add_round_key` combine par `zip`, sans indexation ; les autres indices restent dans `0..16` par construction.
- **Copies effacées** : les copies temporaires de l'état (ShiftRows) et des colonnes (MixColumns) sont effacées avec `wipe` (§ 20).

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | États intermédiaires du tour 1 de l'annexe B (SubBytes, ShiftRows, MixColumns, AddRoundKey), mêmes états parcourus à l'envers par les inverses, six colonnes MixColumns de référence, ligne 0 inchangée, aller-retour de chaque transformation, AddRoundKey involutive |
| Intégration | `expand_key` + tour 1 complet depuis le texte clair de l'annexe B → début du tour 2 |
| Statique | Règles existantes |
| Régression | MixColumns de l'état `00..0f` figé (valeur calculée indépendamment) |
| E2E | **Sans objet** (CLI à l'étape 5) |
| Acceptation | Sujet p. 5 : MixColumns standard |
| Performance | 10⁵ allers-retours complets en moins de 2 s en debug. Le volume a été ramené de 10⁶ à 10⁵ : même avec les tables, 10⁶ prend environ 7 s en debug (itérateurs non optimisés), le même ordre que les autres tests de § 34.4 |
| Sécurité | Diffusion : deux tours changent les 16 octets quand un seul octet d'entrée change |
| Usabilité | **Sans objet** : aucune erreur possible, aucun message |
| Accessibilité | **Sans objet** : aucune sortie |

### 4.4 — Chiffrement d'un bloc (2026-09-24)

`encrypt_block(message: &Plaintext, key: &SymmetricKey) -> Result<Vec<u8>>` chiffre un unique bloc de 16 octets (FIPS-197 annexe C : `AddRoundKey` puis `Nr - 1` tours complets puis un dernier tour sans `MixColumns`).

Couche « format du sujet » (sujet p. 5 : « both the key and the ciphered message are numbers ») : la clé et la sortie chiffrée sont des **nombres**, au même sens que `little_endian.rs` (§ 2.3) — le premier octet écrit est le poids faible. La conversion vers/depuis l'ordre naturel d'AES (poids fort en premier) se fait mot par mot (4 octets), chaque mot étant inversé : `reverse_words`. Le message, lui, n'est pas un nombre et se lit tel quel.

Trouvé par élimination sur le vecteur du sujet (clé hexadécimale de « Winter is coming », message « All men must die », sortie `744ce22c385958348f0df26eceb62eef`) : seule la combinaison « clé inversée par mots, message tel quel, sortie inversée par mots » reproduit le vecteur ; les 63 autres combinaisons testées (inversion totale du bloc, inversion de l'ordre des mots sans inverser leurs octets, inversion du message) échouent.

`ensure_block_size` rejette tout message qui n'a pas exactement 16 octets (`BlockSizeMismatch`) avant tout calcul.

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | Vecteur du sujet (« All men must die » → `744ce22c…`), message de taille incorrecte, clé de taille incorrecte |
| Intégration | **Sans objet à ce stade** : couverte par le vecteur du sujet en unitaire |
| Statique | Règles existantes |
| Régression | **À couvrir en 4.6** avec le vecteur figé une fois le mode flux disponible |
| E2E | **Sans objet** (CLI à l'étape 5) |
| Acceptation | Sujet p. 5 : « both the key and the ciphered message are numbers », vecteur `aes -c -b` |
| Performance | **Sans objet à ce stade** : un seul bloc, couvert par les tests de performance de `round.rs`/`key_schedule.rs` |
| Sécurité | Clé et tampon temporaire effacés via `SymmetricKey`/`Drop` ; aucune fuite de secret dans les erreurs |
| Usabilité | Message d'erreur donnant les tailles reçue et attendue |
| Accessibilité | Message ASCII sur une ligne |

### 4.5 — Déchiffrement d'un bloc (2026-09-24)

`decrypt_block(ciphertext: &[u8], key: &SymmetricKey) -> Result<Plaintext>` applique les transformations inverses de 4.4 dans l'ordre contraire (FIPS-197 § 5.3) : `AddRoundKey` (dernier tour), puis pour chaque tour restant `InvShiftRows`, `InvSubBytes`, `AddRoundKey`, `InvMixColumns`, et enfin `InvShiftRows`, `InvSubBytes`, `AddRoundKey` (tour 0).

Couche « format du sujet » : le chiffré reçu est un nombre (comme en 4.4), donc `reverse_words` lui est appliqué avant les tours inverses ; le message obtenu n'est pas un nombre et n'est pas inversé en sortie. La clé suit la même expansion que pour le chiffrement (`subject_round_keys`, factorisée entre `encrypt_block` et `decrypt_block`, § 1 : pas de duplication).

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | Vecteur du sujet en déchiffrement, aller-retour chiffrement/déchiffrement, chiffré de taille incorrecte |
| Intégration | **Sans objet à ce stade** : couverte par le vecteur du sujet en unitaire |
| Statique | Règles existantes |
| Régression | **À couvrir en 4.6** avec le vecteur figé une fois le mode flux disponible |
| E2E | **Sans objet** (CLI à l'étape 5) |
| Acceptation | Sujet p. 5 : round-trip `aes -c -b` / `aes -d -b` sur le vecteur du sujet |
| Performance | **Sans objet à ce stade** : couvert par les tests de performance de `round.rs`/`key_schedule.rs` |
| Sécurité | Mêmes garanties qu'en 4.4 (clé effacée via `Drop`) |
| Usabilité | Message d'erreur donnant les tailles reçue et attendue |
| Accessibilité | Message ASCII sur une ligne |

### 4.6 — Mode flux (2026-09-24)

`aes_stream_cipher(message: &Plaintext, key: &SymmetricKey) -> Result<Vec<u8>>` et `aes_stream_decipher(ciphertext: &[u8], key: &SymmetricKey) -> Result<Plaintext>` découpent le message en blocs de 16 octets (`BLOCK_BYTES`), chacun chiffré/déchiffré indépendamment par `encrypt_block`/`decrypt_block` (sujet p. 5 : « In stream mode, the data will be ciphered block by block »). Le dernier bloc partiel est complété par des zéros avant chiffrement (sujet p. 5) ; comme pour XOR (§ 34, décision du 2026-09-24), ce padding n'est pas retiré au déchiffrement — un roundtrip exact suppose un message multiple de 16 octets.

`aes_stream_decipher` rejette tout chiffré dont la longueur n'est pas un multiple de 16 octets (`CiphertextLengthNotMultiple`), avant tout calcul.

Ce module correspond au mode par défaut de la CLI (absence de `-b`), par opposition au mode bloc (`-b`, un seul bloc exact, `encrypt_block`/`decrypt_block` directement).

Tests (REGLES.md § 38) :

| Type | Couverture |
|---|---|
| Unitaire | Vecteur du sujet en un seul bloc (égal à `encrypt_block`), padding du dernier bloc, message/chiffré vides, chiffré de longueur non multiple de 16, aller-retour sur plusieurs blocs |
| Intégration | **Sans objet à ce stade** : couverte par le vecteur du sujet en unitaire |
| Statique | Règles existantes |
| Régression | **À couvrir** : vecteur du sujet répété figé |
| E2E | Couvert via la CLI (`aes -c`/`aes -d` sans `-b`), voir `tests/e2e/` |
| Acceptation | Sujet p. 5 : mode flux, padding zéro du dernier bloc |
| Performance | **Sans objet à ce stade** : couvert par les tests de performance de `round.rs`/`key_schedule.rs` |
| Sécurité | Mêmes garanties que `block.rs` (clé effacée via `Drop`) |
| Usabilité | Message d'erreur donnant les tailles reçue et attendue |
| Accessibilité | Message ASCII sur une ligne |
