# `src/bignum` — explication

## À quoi sert ce dossier ?

Ce dossier implémente une bibliothèque de nombres entiers non signés à précision arbitraire (`BigUint`). Elle est nécessaire car les nombres utilisés en RSA dépassent la capacité des types standards de Rust comme `u64` ou `u128`.

## Comment fonctionne `BigUint` ?

Un `BigUint` est représenté par un vecteur de `u32` appelé **limbes**. Chaque limbe contient 32 bits du nombre. Le premier élément du vecteur est le mot de poids faible (little-endian). Par exemple, le nombre `0x0123456789ABCDEF` est stocké comme `[0x89ABCDEF, 0x01234567]`.

### Normalisation

La fonction `normalize` supprime les zéros inutiles au sommet du vecteur. Un `BigUint` dont le vecteur est vide représente zéro.

### Comparaison

Deux nombres sont comparés d'abord par la longueur du vecteur (plus long = plus grand), puis par les limbes en partant du sommet.

## Opérations arithmétiques

### Addition et soustraction

L'addition se fait mot par mot avec une retenue. La soustraction se fait mot par mot avec un emprunt. Les deux opérations parcourent les limbes de gauche à droite.

### Multiplication

La multiplication utilise l'algorithme scolaire : pour chaque limbe du premier facteur, on multiplie toutes les limbes du second facteur et on accumule les résultats avec les décalages appropriés.

### Division et reste

La division euclidienne utilise l'algorithme de la division posée bit par bit (en partant du bit de poids fort). Le quotient et le reste sont calculés simultanément.

## Opérations modulaires

### Puissance modulaire (`powmod`)

L'exponentiation modulaire utilise l'algorithme d'exponentiation rapide (square-and-multiply). Elle est essentielle pour le chiffrement RSA (`c = m^e mod n`) et le déchiffrement (`m = c^d mod n`).

### PGCD (`gcd`)

Le plus grand commun diviseur est calculé par l'algorithme d'Euclide itératif. Il est utilisé pour vérifier que l'exposant public est premier avec la fonction de Carmichael.

### Inverse modulaire (`mod_inverse`)

L'inverse modulaire est calculé par l'algorithme d'Euclide étendu. Il permet de trouver l'exposant privé `d` tel que `d × e ≡ 1 (mod λ(n))`.

## Conversion hexadécimale

La conversion depuis et vers le format hexadécimal little-endian permet d'interfacer `BigUint` avec les arguments de la ligne de commande et les messages chiffrés. Les hexadécimaux sont en little-endian selon la convention du sujet.

## Pourquoi un module séparé ?

L'arithmétique multiprécision est un prérequis pour le RSA. En la séparant dans son propre module, on respecte le principe de responsabilité unique et on permet à d'autres parties du programme (comme la génération de nombres premiers) de l'utiliser sans dépendre de la logique RSA.
