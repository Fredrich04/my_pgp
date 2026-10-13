# `src/io` — explication

## À quoi sert ce dossier ?

Le programme ne parle qu'en hexadécimal : les clés passées en argument et les messages chiffrés sont écrits avec les chiffres `0-9` et les lettres `a-f`. Ce dossier traduit ce texte en octets que les algorithmes peuvent manipuler, et inversement.

## Comment fonctionne le décodage ?

Deux caractères hexadécimaux forment un octet : `4` et `1` donnent `0x41`, c'est-à-dire la lettre `A`. Le décodage lit donc le texte deux caractères à la fois.

Il refuse deux sortes d'erreurs :

- un nombre impair de caractères, car le dernier caractère n'aurait pas de partenaire ;
- un caractère qui n'est pas hexadécimal, comme `g` ou un espace.

Dans le second cas, le message indique **où** se trouve l'erreur, mais pas **quel** caractère pose problème : la chaîne décodée peut être une clé secrète, et on ne veut jamais qu'un morceau de clé apparaisse dans un message d'erreur.

**Où (décodage).** `hex.rs:5 decode_hex` fait le découpage par paires (ligne 13) et l'assemblage `(haut << 4) | bas` (ligne 16) ; `hex.rs:30 hex_value` convertit un caractère. **Exemple :** `"4a"` → `4` (haut) et `a`=10 (bas) → `4 × 16 + 10 = 74 = 0x4a`, la lettre `J`. Un texte de 3 chiffres est refusé (ligne 7).

## Et l'encodage ?

C'est l'opération inverse : chaque octet devient deux caractères. On découpe l'octet en deux moitiés de 4 bits, et chaque moitié (une valeur de 0 à 15) choisit un caractère dans la liste `0123456789abcdef`. Le résultat est toujours en minuscules, sans espace ni préfixe, exactement comme dans les exemples du sujet.

**Où (encodage).** `hex.rs:21 encode_hex`, lignes 24 et 25. **Exemple :** l'octet `0x4a` donne `4a` ; l'octet `0x05` donne `05` (le zéro de tête est conservé, il faut toujours deux caractères).

## Pourquoi les nombres sont-ils « à l'envers » ?

Le sujet écrit les nombres en *little-endian* : on commence par le petit bout. Le nombre `0x01234567` est formé des octets `01 23 45 67` ; en little-endian, on les écrit en commençant par le dernier : `67 45 23 01`. C'est l'ordre dans lequel la plupart des processeurs rangent les nombres en mémoire.

Les zéros « de tête » (ceux qui ne changent pas la valeur, comme dans `007`) ne sont pas écrits, sauf pour zéro lui-même, qui s'écrit `00`.

**Où.** `little_endian.rs:5 u128_to_le_bytes` (suppression des zéros de tête lignes 7 à 10) et `little_endian.rs:14 le_bytes_to_u128`. **Exemple :** `0x1234` donne les octets `34 12` ; `0` donne `00`. **Analogie :** écrire une date « jour, mois, année » plutôt que « année, mois, jour » : même information, ordre différent.

## Comment le message est-il lu ?

Quand on tape `echo "All men must die" | ./my_pgp ...`, la commande `echo` ajoute un retour à la ligne à la fin. Ce retour à la ligne ne fait pas partie du message : on le retire. Sous Windows, un retour à la ligne s'écrit avec deux caractères (`\r\n`) ; les deux sont retirés. On n'en retire qu'un seul, pour ne pas abîmer un message qui contiendrait volontairement des lignes vides à la fin.

Le programme refuse aussi les entrées de plus de 64 Mio. Sans cette limite, quelqu'un pourrait envoyer un flux sans fin et remplir toute la mémoire de la machine. La lecture s'arrête dès que la limite est dépassée, sans attendre la fin du flux.

**Où.** `stdin.rs:7 read_message` (limite lignes 8 à 19) et `stdin.rs:24 strip_final_newline` (retrait du retour à la ligne). **Exemple :** `"All men must die\n"` devient `"All men must die"` (16 octets, un bloc AES exact) ; `"a\n\n"` devient `"a\n"` (un seul retour est retiré). **Pourquoi lire un octet de plus que la limite :** pour savoir si le flux dépassait la limite sans avoir à le lire en entier.
