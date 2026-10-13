# `src/symmetric/aes` — explication

## Qu'est-ce qu'AES ?

AES est l'algorithme de chiffrement symétrique standard utilisé partout (Wi-Fi, HTTPS…). Il chiffre des blocs de 16 octets en les mélangeant pendant plusieurs « tours ». On le construit ici pièce par pièce.

**Où.** Le point d'entrée est `block.rs:11 encrypt_block` (un bloc) ; `stream.rs:6 aes_stream_cipher` l'appelle pour les messages plus longs. **Analogie :** un cube de Rubik qu'on tourne selon une séquence dérivée de la clé ; pour revenir en arrière, on rejoue les mêmes mouvements à l'envers.

## Qu'est-ce que la S-box ?

La S-box est un tableau de 256 cases : chaque octet est remplacé par un autre. Elle introduit de la « confusion » : un petit changement à l'entrée donne un résultat très différent. Son inverse permet de défaire ce remplacement au déchiffrement.

Plutôt que de recopier ce tableau depuis la norme, on le calcule : chaque octet est inversé dans une arithmétique spéciale sur 8 bits (le « corps GF(2⁸) »), puis mélangé par une petite formule. Le compilateur fait ce calcul une fois pour toutes, au moment de construire le programme. Des tests vérifient que le résultat est exactement le tableau officiel.

**Où.** Le tableau est `sbox.rs:5 SBOX` (calculé par `sbox.rs:25 build_sbox`), son inverse `sbox.rs:6 INV_SBOX` (par `sbox.rs:35 invert`). **Exemple :** `SBOX[0x00] = 0x63` et `SBOX[0x01] = 0x7c` : l'octet `00` devient `63`. **Analogie :** un code secret de lettres où chacune est remplacée par une autre, sans motif simple.

## Le corps GF(2⁸) en deux phrases

On y additionne avec un XOR et on multiplie avec une règle qui « replie » les résultats trop grands pour qu'ils tiennent toujours dans un octet. Cette arithmétique sert à la S-box et, plus tard, au mélange des colonnes (MixColumns).

**Où.** `gf256.rs:3 gf_mul` multiplie ; `gf256.rs:21 gf_inverse` inverse. **Exemple :** `0x57 × 0x02 = 0xae` (simple décalage), mais `0xae × 0x02 = 0x47` : le décalage déborde (`0x15c`), on retire le bit en trop et on fait XOR avec `0x1b` (`0x5c ⊕ 0x1b = 0x47`). **Analogie :** une horloge à 256 positions où, au lieu de revenir à zéro, on « replie » selon une règle fixe.

## À quoi sert l'expansion de clé ?

AES mélange le bloc pendant 10, 12 ou 14 tours selon la taille de la clé. À chaque tour, il utilise une clé différente, fabriquée à partir de la clé d'origine : c'est l'« expansion de clé ». Chaque nouvelle portion est obtenue en décalant, en passant par la S-box et en combinant par XOR la portion précédente, avec une petite constante qui change à chaque tour. Ainsi, modifier un seul bit de la clé d'origine change profondément toutes les clés de tour, ce qu'un test vérifie.

Ces clés de tour sont aussi secrètes que la clé elle-même : elles ne s'affichent jamais et sont effacées de la mémoire après usage.

**Où.** `key_schedule.rs:37 expand_key` : le nombre de tours est calculé ligne 44 (clé de 16 octets → 10 tours, 24 → 12, 32 → 14), les nouveaux mots ligne 60 à 63. Les clés de tour sont ensuite lues par `key_schedule.rs:20 round_key`. **Exemple :** pour une clé de 16 octets, on produit 11 clés de tour de 16 octets chacune, soit 176 octets. **Analogie :** une recette de cuisine qui change légèrement à chaque étape, à partir d'un même ingrédient de départ.

## Que se passe-t-il pendant un tour ?

Le bloc de 16 octets est vu comme un carré de 4 × 4. Un tour lui applique quatre opérations :

1. **SubBytes** : chaque octet passe par la S-box.
2. **ShiftRows** : chaque ligne du carré glisse vers la gauche, d'autant de cases que son numéro (la première ne bouge pas).
3. **MixColumns** : chaque colonne est mélangée, de sorte que chacun de ses 4 octets dépende des 4 autres.
4. **AddRoundKey** : on combine le carré avec la clé du tour par un XOR.

ShiftRows déplace les octets d'une colonne à l'autre et MixColumns les mélange dans une colonne : après deux tours, changer un seul octet d'entrée modifie les 16 octets, ce qu'un test vérifie. Pour déchiffrer, chaque opération a son inverse, appliqué dans l'ordre contraire.

**Où.** Les quatre opérations sont `round.rs:22 sub_bytes`, `round.rs:32 shift_rows`, `round.rs:40 mix_columns` et `round.rs:48 add_round_key` ; leurs inverses sont `round.rs:26 inv_sub_bytes`, `round.rs:36 inv_shift_rows` et `round.rs:44 inv_mix_columns`. **Exemple :** ShiftRows laisse la ligne 0 telle quelle et fait glisser la ligne 1 d'une case, la ligne 2 de deux cases, la ligne 3 de trois. **Analogie :** battre un jeu de cartes : SubBytes remplace les cartes, ShiftRows les décale, MixColumns les mélange par paquets, AddRoundKey les marque avec la clé.

## Comment un bloc entier est-il chiffré ?

On enchaîne un `AddRoundKey` initial, puis un tour complet répété (`SubBytes`, `ShiftRows`, `MixColumns`, `AddRoundKey`) pour chaque tour sauf le dernier, qui saute `MixColumns`.

Le sujet précise que la clé et le résultat chiffré sont des **nombres** : le premier octet écrit dans l'énoncé est le moins significatif, comme pour les grands nombres du projet (RSA, longueurs). Comme AES range ses octets dans l'autre sens (le plus significatif en premier), il faut inverser chaque groupe de 4 octets de la clé avant de l'utiliser, puis inverser chaque groupe de 4 octets du résultat avant de le renvoyer. Le message en clair, lui, n'est pas un nombre : on le lit tel quel, sans inversion.

**Où.** La boucle des tours est `block.rs:38 cipher_rounds` (clé initiale ligne 40, tours 1 à Nr−1 lignes 41 à 46, dernier tour sans MixColumns lignes 47 à 49). L'inversion des groupes de 4 octets est `block.rs:66 reverse_words`, appliquée à la clé par `block.rs:31 subject_round_keys` (ligne 33) et au résultat par `block.rs:11 encrypt_block` (ligne 17). **Exemple :** la clé `57 69 6e 74` ("Wint") devient `74 6e 69 57` avant le calcul. **Pourquoi seulement la clé et le résultat :** le sujet les définit comme des nombres, pas le message.

## Et pour déchiffrer ?

On refait exactement le chemin inverse : on part du chiffré (un nombre, donc on inverse d'abord ses groupes de 4 octets), on annule chaque opération du dernier tour au premier, avec les S-box et mélanges inverses vus plus haut, et on retombe sur le message d'origine, qui n'a besoin d'aucune inversion puisque ce n'est pas un nombre.

**Où.** `block.rs:21 decrypt_block` appelle `block.rs:52 inv_cipher_rounds` (clé finale ligne 54, tours en sens inverse lignes 55 à 60, clé 0 ligne 63). Le déchiffrement se vérifie avec le vecteur du sujet : `744ce22c385958348f0df26eceb62eef` redonne `All men must die`.

## Et pour un message plus long qu'un bloc ?

AES ne sait chiffrer que des blocs de 16 octets à la fois. Pour un message plus long, on le découpe en tranches de 16 octets et on chiffre chaque tranche indépendamment, comme une pile d'enveloppes scellées une par une avec le même cadenas. Si la dernière tranche est trop courte, on la complète avec des zéros avant de la sceller ; ces zéros restent dans le message une fois déchiffré, exactement comme pour XOR.

**Où.** `stream.rs:6 aes_stream_cipher` découpe (ligne 8), complète de zéros (ligne 10) et chiffre chaque bloc (ligne 12) ; `stream.rs:17 aes_stream_decipher` refuse un chiffré dont la longueur n'est pas un multiple de 16 (ligne 18) ; `stream.rs:31 padded_len` donne la taille finale. **Exemple :** un message de 17 octets donne 2 blocs, donc 32 octets chiffrés (64 caractères hexadécimaux) ; un message vide donne un bloc.
