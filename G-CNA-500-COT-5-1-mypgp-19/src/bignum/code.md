# `src/bignum` — documentation technique

## Contenu

| Fichier | Rôle |
|---|---|
| `mod.rs` | Définit la structure `BigUint` et exporte `HexError` |
| `base.rs` | Opérations de base : zéro, un, normalisation, comparaison, décalage |
| `arithmetic.rs` | Addition, soustraction et multiplication multiprécision |
| `division.rs` | Division euclidienne, reste, quotient |
| `modular.rs` | Puissance modulaire, PGCD, inverse modulaire |
| `conversion.rs` | Conversion entre hexadécimal little-endian et `BigUint` |

## Structure de `BigUint`

`BigUint` représente un entier non signé à précision arbitraire à l'aide d'un vecteur de `u32` (limbes). Chaque limbe correspond à 32 bits du nombre. Le vecteur est stocké en little-endian : le premier élément est le mot de poids faible.

Par exemple, le nombre `0x0123456789ABCDEF` est représenté par les limbes `[0x89ABCDEF, 0x01234567]`.

## Conversion hexadécimale

Les nombres sont convertis vers et depuis le format hexadécimal little-endian (les deux caractères hexadécimaux les plus à droite représentent l'octet de poids faible). Cette convention suit la spécification du sujet (REGLES.md § 4).

## Historique

### Implémentation multiprécision (2026-09-25)

Le module `bignum` fournit les opérations arithmétiques nécessaires au chiffrement RSA. Toutes les opérations sont implémentées à partir de zéro en Rust sûr, sans aucune dépendance externe ni bloc `unsafe`.

### Documentation (2026-09-25)

Ajout de `code.md` et `explication.md` pour satisfaire la règle REGLES.md § 35.

### Conformité sans panique (2026-09-30)

`to_bytes_le` utilisait `out.last().unwrap()` : remplacé par `out.last().map_or(false, |b| *b == 0)`. Le `len() > 1` garantissait déjà `Some`, mais la règle REGLES.md § 12.8 interdit le motif `unwrap()` même prouvé sûr ; la version `map_or` est équivalente et passe le test statique.