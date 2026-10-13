# `src/symmetric` — explication

Ce dossier regroupe les deux chiffrements à clé partagée : XOR (ici) et AES (`aes/`). Le fichier `xor.rs` contient tout le XOR.

## Qu'est-ce que le XOR ?

**Quoi.** Le XOR (« ou exclusif ») compare deux bits : le résultat vaut 1 s'ils sont différents, 0 s'ils sont identiques. Appliqué octet par octet entre un message et une clé, il produit un message chiffré.

**Pourquoi.** C'est le chiffrement le plus simple qui existe, et celui qu'Arya propose dans le sujet (p. 4). Sa particularité : refaire un XOR avec la même clé rend le message d'origine, donc chiffrer et déchiffrer sont presque la même opération.

**Comment.** Exemple d'un octet : la lettre `Y` vaut `0x59` (`0101 1001`), la clé vaut `0x20` (`0010 0000`). Bit par bit : `0101 1001 ⊕ 0010 0000 = 0111 1001` = `0x79`. Refaire `0x79 ⊕ 0x20` donne `0x59`, la lettre `Y`.

**Où.** Le XOR d'une paire d'octets est la ligne 11 de `xor.rs:4 xor_block_cipher` (`message_byte ^ key_byte`) et la ligne 20 de `xor.rs:15 xor_block_decipher`.

**Analogie.** Un calque transparent troué à des endroits précis (la clé) posé sur un texte inverse ce qu'il recouvre. Reposer exactement le même calque sur le résultat annule l'inversion. Le défaut : celui qui connaît un morceau du message clair retrouve la clé à cet endroit (`clé = message ⊕ chiffré`).

## Qu'est-ce que le mode bloc ?

**Quoi.** Avec l'option `-b`, on ne traite qu'un seul bloc, qui doit faire exactement la taille de la clé.

**Pourquoi.** Le sujet (p. 3) le demande « pour faciliter le débogage ». Si le message est plus court ou plus long, le programme refuse et indique les deux tailles, pour que l'utilisateur sache quoi corriger.

**Comment.** Le contrôle est fait **avant** tout calcul, par la fonction privée `xor.rs:26 ensure_same_size`, appelée en première ligne de `xor_block_cipher` et de `xor_block_decipher`.

## Pourquoi le message est-il « retourné » ?

**Quoi.** Avant le XOR, l'ordre des octets du message est inversé (`.rev()`, ligne 9 de `xor.rs:4 xor_block_cipher`) ; après le XOR, `xor.rs:15 xor_block_decipher` le remet à l'endroit (`reverse()`, ligne 22).

**Pourquoi.** Le sujet considère la clé et le résultat comme des **nombres** écrits en little-endian (le petit bout en premier), alors que le message est un texte lu dans l'ordre normal. Pour que les deux se correspondent, on retourne le message.

**Exemple chiffré.** Message `"You know nothing, Jon Snow"` (26 octets, le saut de ligne final n'est pas compté), clé `57 68 61 74 …`. Le premier octet de sortie est le **dernier** caractère du message, `w` (`0x77`), combiné au **premier** octet de la clé (`0x57`) : `0x77 ⊕ 0x57 = 0x20`, exactement le premier octet du vecteur officiel (`20 07 0f …`). Sans le retournement, on aurait combiné `Y` (`0x59`) avec `0x57` et obtenu `0x0e`, un résultat faux ; un test du dossier `tests/` vérifie ce vecteur.

## Et le mode flux ?

**Quoi.** Sans l'option `-b`, le message peut avoir n'importe quelle longueur.

**Pourquoi.** Un vrai message est rarement de la taille exacte de la clé ; le sujet (p. 3) exige que l'algorithme gère « n'importe quelle longueur ».

**Comment.** `xor.rs:36 xor_stream_cipher` coupe le message en morceaux de la taille de la clé (`chunks`, ligne 39), complète le dernier avec des zéros (`resize`, ligne 41), et chiffre chaque morceau comme en mode bloc (ligne 43). Au déchiffrement, `xor.rs:48 xor_stream_decipher` refait l'opération morceau par morceau ; les zéros réapparaissent à la fin du message, le sujet ne demandant pas de les retirer.

**Exemple chiffré.** Clé de 4 octets, message de 10 octets : blocs de 4 + 4 + 2, le dernier complété à 4 par deux zéros → 12 octets chiffrés, soit 24 caractères hexadécimaux.

**Analogie.** Une pile d'enveloppes scellées une par une avec le même cadenas : la dernière enveloppe, trop petite, est remplie de papier blanc (les zéros) pour avoir la même taille que les autres.

Si on reçoit un message chiffré dont la longueur ne tombe pas juste (par exemple 6 octets avec une clé de 4), c'est qu'il a été coupé ou abîmé : le contrôle de la ligne 50 refuse plutôt que de produire un résultat faux.
