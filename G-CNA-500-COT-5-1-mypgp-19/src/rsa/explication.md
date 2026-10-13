# `src/rsa` — explication

## À quoi sert ce dossier ?

Ce dossier implémente l'algorithme de cryptographie asymétrique RSA. Il permet de générer des paires de clés publiques/privées à partir de deux nombres premiers, et de chiffrer/déchiffrer des messages utilisant ces clés.

## Comment fonctionne la génération de clés ?

La fonction `keygen(p, q)` prend deux nombres premiers `p` et `q` en entrée et calcule :

- **`n = p × q`** : le module commun aux deux clés
- **`λ(n) = lcm(p-1, q-1)`** : la fonction de Carmichael, qui est le plus petit entier positif tel que `a^λ(n) ≡ 1 (mod n)` pour tout `a` premier avec `n`
- **`e`** : l'exposant public, choisi comme le plus grand nombre premier de Fermat inférieur à `λ(n)` et premier avec `λ(n)`. Les nombres de Fermat testés sont 65537, 257, 17, 5 et 3, dans cet ordre.
- **`d`** : l'exposant privé, calculé comme l'inverse modulaire de `e` modulo `λ(n)`, c'est-à-dire `d × e ≡ 1 (mod λ(n))`.

Le résultat est une structure `KeyPair` contenant `e`, `d` et `n`.

## Comment fonctionne le chiffrement ?

Le message est lu sous forme d'octets. Ces octets sont interprétés comme un nombre en little-endian (l'octet de poids faible est en premier). Ce nombre `m` est ensuite chiffré par `c = m^e mod n`. Le résultat `c` est reconverti en octets et affiché en hexadécimal.

Le déchiffrement fait l'opération inverse : `m = c^d mod n`.

## Pourquoi des nombres multiprécision ?

Les nombres utilisés en RSA sont trop grands pour tenir dans les types standards de Rust (comme `u64` ou `u128`). Le dossier `src/bignum` fournit une implémentation de nombres entiers non signés à précision arbitraire (`BigUint`) avec des opérations arithmétiques, modulaires et de conversion hexadécimale.

## Pourquoi `/dev/urandom` ?

Pour la génération de nombres premiers aléatoires, le programme lit les octets depuis `/dev/urandom`, un périphérique du système qui fournit des données aléatoires de haute qualité issues de l'entropie du noyau.

## Structure des fichiers

- `mod.rs` : déclare les sous-modules et exporte `KeyPair` et `keygen`
- `keygen.rs` : contient la fonction `keygen` et la structure `KeyPair`
- `cipher.rs` : implémente le chiffrement et le déchiffrement RSA ainsi que le trait `RsaCipher`
- `prime.rs` : test de primalité probabiliste (Miller-Rabin) et génération de nombres premiers
- `random.rs` : génération d'octets aléatoires et de nombres premiers aléatoires
