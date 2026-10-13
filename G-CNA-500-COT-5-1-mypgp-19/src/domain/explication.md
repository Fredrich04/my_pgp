# `src/domain` — explication

## À quoi sert ce dossier ?

Le « domaine », c'est le vocabulaire commun du projet : les erreurs possibles et, bientôt, les types qui représentent une clé, un bloc, un cryptosystème ou un mode. Karyl et Frédrich s'appuient tous deux dessus, c'est pourquoi il fait partie des fichiers communs.

## Pourquoi avoir tout retiré ?

L'ancienne version construisait une application de chiffrement de fichiers : elle lisait un fichier, le chiffrait avec AES-GCM, chiffrait la clé avec RSA-OAEP et écrivait le tout en JSON. C'est une bonne pratique dans la vraie vie, mais ce n'est pas ce que demande le sujet :

- le sujet lit le message sur l'entrée standard et écrit des nombres hexadécimaux, pas du JSON ;
- il impose la syntaxe `./my_pgp CRYPTO_SYSTEM MODE [OPTIONS] [key]` ;
- il demande d'implémenter soi-même les algorithmes, alors que l'ancien code les empruntait à des bibliothèques.

Aucun exemple du sujet ne pouvait donc passer. Plutôt que d'adapter un code bâti sur de mauvaises fondations, on repart d'une base vide et propre. L'ancien code reste consultable dans l'historique git.

**Où.** Le vocabulaire est réparti dans `error.rs` (`error.rs:8 MyPgpError`), `types.rs` (`types.rs:6 CryptoSystem`, `types.rs:15 Mode`) et `rsa.rs` (`rsa.rs:3 RsaCipher`).

## Pourquoi le programme renvoie-t-il 84 ?

Le sujet exige que toute erreur soit écrite sur la sortie d'erreur et que le programme se termine avec le code 84. Tant qu'aucun cryptosystème n'existe, toute exécution est une erreur : le binaire le dit honnêtement au lieu de prétendre fonctionner.

## Comment sont pensées les erreurs ?

Chaque erreur répond à une question précise : « qu'est-ce qui ne va pas ? ». Un argument manque, un caractère n'est pas hexadécimal, la clé n'a pas la bonne taille… Chaque cas a sa propre variante, ce qui permet au programme (et aux tests) de réagir précisément, au lieu de manipuler un simple texte.

Une règle compte plus que les autres : **une erreur ne répète jamais une donnée secrète**. Si la clé contient un caractère invalide, le message indique sa position (« position 3 »), pas le caractère lui-même. Sans cette précaution, une clé pourrait se retrouver affichée dans un terminal, dans un journal ou dans une capture d'écran.

Quelle que soit l'erreur, le programme se termine avec le code 84, comme l'exige le sujet. La précision est conservée dans le message affiché sur la sortie d'erreur.

**Où.** Les variantes sont aux lignes 9 à 34 de `error.rs:8 MyPgpError` ; leur affichage aux lignes 37 à 71. **Exemple :** `"zz"` donne `invalid hexadecimal: bad digit at position 0` ; la lettre `z` n'est pas répétée. **Analogie :** un ticket de caisse qui indique « article n°3 défectueux » sans recopier le code de la carte bancaire.

## Que représentent les types partagés ?

- `CryptoSystem` : le nom du cryptosystème tapé par l'utilisateur (`xor`, `aes`, `rsa`, `pgp-xor`, `pgp-aes`). Utiliser une énumération plutôt que du texte garantit qu'un nom inconnu est refusé dès l'analyse des arguments, et que le compilateur signale tout cas oublié.
- `Mode` : ce que l'on veut faire (chiffrer `-c`, déchiffrer `-d`, générer des clés `-g`).
- `SymmetricKey` : une clé secrète. Elle est traitée comme une donnée sensible :
  - elle ne s'affiche jamais, même dans un message de débogage : seule sa taille apparaît ;
  - elle ne peut pas être copiée par inadvertance ;
  - quand on n'en a plus besoin, sa mémoire est remplie de zéros avant d'être rendue au système. Sans cela, la clé pourrait rester lisible en mémoire longtemps après usage.

Remplir la mémoire de zéros paraît simple, mais un compilateur optimiseur peut supprimer une écriture qu'il juge inutile (« pourquoi écrire dans une mémoire qu'on va libérer ? »). On remplit donc la mémoire de zéros, puis on la passe à `black_box`, une fonction qui fait croire au compilateur que quelqu'un va relire ces zéros : il n'ose alors plus supprimer l'écriture. Cette méthode n'utilise aucun code `unsafe` (le mode de Rust où le compilateur ne vérifie plus la sécurité mémoire), au prix d'une garantie un peu moins forte.

**Où.** `types.rs:21 SymmetricKey` (effacement par `types.rs:92 wipe`, ligne 93 `fill(0)`, ligne 94 `black_box`). **Exemple :** une clé de 16 octets s'affiche `SymmetricKey(<16 bytes, redacted>)` en débogage, jamais ses octets. **Analogie :** écrire un mot de passe au tableau puis l'effacer à l'éponge avant de quitter la salle, et vérifier que l'éponge a bien frotté.

## À quoi sert `RsaCipher` ?

Le mode PGP combine deux algorithmes : un algorithme symétrique (XOR ou AES, écrit par Karyl) chiffre le message, et RSA (écrit par Frédrich) chiffre la clé symétrique. `RsaCipher` est la « prise électrique » entre les deux : Karyl sait pouvoir appeler `encrypt` et `decrypt` avec une clé et des octets, sans connaître les détails de RSA ; Frédrich sait exactement quelles fonctions il doit fournir. Chacun peut ainsi avancer de son côté, et Karyl peut tester son code PGP avec un faux RSA en attendant le vrai.

**Où.** `rsa.rs:3 RsaCipher`, deux méthodes lignes 4 et 5. **Analogie :** une prise électrique normalisée : peu importe qui fabrique la prise ou l'appareil, tout se branche si la forme est la même.

## Pourquoi écrire les erreurs « à la main » ?

`thiserror` est une bibliothèque qui écrit automatiquement le code d'affichage des erreurs. C'est pratique, mais c'est du code extérieur au projet. Écrire ce code nous-mêmes rend le projet totalement autonome : tout ce qui s'exécute a été écrit et relu par l'équipe. Le résultat est identique : chaque erreur sait s'afficher et, pour une erreur d'entrée/sortie, dire quelle erreur système l'a causée.

**Où.** Les protections d'effacement et de `Debug` masqué sont les mêmes que pour la clé : aucune variante de `error.rs:8 MyPgpError` ne contient de message en clair, donc le message ne peut pas fuiter par une erreur.

## Et le message en clair ?

Le message que l'on veut chiffrer est aussi secret que la clé : c'est justement ce qu'on veut protéger. Il reçoit donc les mêmes protections, dans un type `Plaintext` : il ne s'affiche jamais en débogage et sa mémoire est remise à zéro quand on n'en a plus besoin.

**Où.** `types.rs:58 Plaintext`, avec le même `Drop` et le même `Debug` masqué que la clé. **Exemple :** un message de 16 octets s'affiche `Plaintext(<16 bytes, redacted>)`.
