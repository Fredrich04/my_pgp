# `src/rsa` — documentation technique

## Contenu

| Fichier | Rôle |
|---|---|
| `mod.rs` | Déclare les modules et réexporte `KeyPair`, `keygen` |
| `keygen.rs` | Fonction `keygen(p, q)` et `KeyPair` (clé publique/privée) |
| `cipher.rs` | Implémente le chiffrement/déchiffrement RSA et le trait `RsaCipher` |
| `prime.rs` | Test de primalité de Miller-Rabin et génération de nombres premiers |
| `random.rs` | Génération d'octets aléatoires via `/dev/urandom` |

## Algorithme de génération de clés

1. Calcul de `n = p * q`
2. Calcul de la fonction de Carmichael `λ(n) = lcm(p-1, q-1)`
3. Choix de l'exposant public `e` comme le plus grand nombre premier de Fermat tel que `1 < e < λ(n)` et `gcd(e, λ(n)) = 1`
4. Calcul de l'exposant privé `d = e^(-1) mod λ(n)`

Les nombres premiers de Fermat testés sont : 65537, 257, 17, 5, 3 (dans cet ordre décroissant).

## Format des clés

Les clés RSA sont représentées sous forme de chaînes hexadécimales en little-endian au format `e-n` (clé publique) ou `d-n` (clé privée), où `n` est le module.

Par exemple, `0101-19bb` signifie `e = 0x0101 = 257` et `n = 0x19bb = 47897`.

## Chiffrement et déchiffrement

Le message est converti en nombre little-endian `m`, puis :
- Chiffrement : `c = m^e mod n`
- Déchiffrement : `m = c^d mod n`

Le résultat est reconverti en octets little-endian puis affiché en hexadécimal.

## Historique

### Implémentation RSA (2026-09-25)

Ajout du module `cipher.rs` implémentant le trait `RsaCipher` pour le chiffrement et le déchiffrement RSA à partir de nombres multiprécision. La génération de clés utilise la fonction `keygen` déjà existante.

### Documentation (2026-09-25)

Ajout de `code.md` et `explication.md` pour satisfaire la règle REGLES.md § 35 (chaque dossier source doit avoir une documentation).
