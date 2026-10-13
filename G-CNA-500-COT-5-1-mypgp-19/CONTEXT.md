# Contexte du projet my_pgp

Ce projet est une implémentation de cryptosystèmes pédagogiques "from scratch" en Rust (XOR, AES, RSA, PGP), séparée entre deux contributeurs (Karyl et Frédrich).

## Périmètre de Karyl
Karyl est responsable de :
- L'I/O (gestion du codec hexadécimal, lecture stdin)
- La cryptographie symétrique (XOR, AES 128/192/256 en modes bloc et stream)
- La CLI (`my_pgp`) et son parsing
- L'intégration de la logique composite PGP (`pgp-xor`, `pgp-aes`)

## État d'avancement de Karyl
- **XOR** : Implémenté et testé (block et stream).
- **AES** : Implémenté et testé (key schedule, transformations de rounds, block et stream).
- **CLI** : Routage XOR, AES, RSA, PGP-XOR et PGP-AES implémenté, avec erreur 84 sur les entrées invalides.
- **Étape courante** : Partie obligatoire terminée et validée par les vecteurs CLI du PDF; les travaux restants sont uniquement des bonus.
- **Règles Strictes** : Pas de `unwrap`/`expect`/`panic`, aucun code `unsafe`, aucune dépendance cryptographique externe.

## Prochaines étapes
- Maintenir la compatibilité des vecteurs obligatoires du PDF.
- Les modes CBC/CTR, le padding, l'authentification, les signatures et X25519 sont hors périmètre obligatoire et ne sont pas implémentés.
