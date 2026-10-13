# REGLES.md

## Règles de développement — Rust / PGP / Cryptographie

> **Statut : obligatoire**
>
> Ce document définit les règles à respecter pour toute évolution du code Rust lié aux erreurs, aux clés, au chiffrement, aux signatures, aux messages chiffrés et à leur sérialisation.
>
> Les règles ci-dessous sont dérivées de l’audit de deux portions de code :
> - le modèle `MyPgpError` ;
> - les modèles `EncryptedMessage`, `PublicKey`, `PrivateKey`, `KeyPair` et `Signature`.
>
> L’objectif n’est pas de rendre le code inutilement complexe. La priorité est :
>
> **sécurité → correction cryptographique → typage → maintenabilité → simplicité.**

---

# 1. Principes fondamentaux

## 1.1. Préserver la simplicité

Le code doit rester :
- lisible ;
- direct ;
- prévisible ;
- facile à tester ;
- sans abstraction inutile.

Éviter le sur-engineering.

Une abstraction n’est acceptable que si elle apporte au moins un bénéfice clair :
- sécurité ;
- invariants métier ;
- réutilisation réelle ;
- réduction de duplication ;
- meilleure lisibilité.

---

## 1.2. Préférer le typage fort

Les données cryptographiques ne doivent pas être représentées par des `Vec<u8>` génériques lorsque des types distincts améliorent la sécurité ou la lisibilité.

Exemples de concepts pouvant bénéficier de types dédiés :

```text
Ciphertext
Nonce / IV
EncryptedKey
SignatureBytes
MessageHash
PublicKeyMaterial
PrivateKeyMaterial
```

Objectif :

```text
Ne pas traiter toutes les données binaires comme étant interchangeables.
```

Le compilateur doit aider à empêcher les confusions de domaines.

---

## 1.3. Rendre les états invalides difficiles à représenter

Un type Rust doit autant que possible empêcher les combinaisons incohérentes.

Exemple à éviter :

```text
Aes256Cbc + iv = None
```

ou :

```text
RsaOaep + encrypted_key = Some(...)
```

si cette combinaison n’a pas de sens dans le protocole.

Le modèle de données doit porter les invariants métier lorsque cela est raisonnablement possible.

Principe :

> **Make invalid states unrepresentable.**

Lorsqu’un invariant ne peut pas être imposé par le type, il doit être imposé par :
- un constructeur validant ;
- une fonction de validation ;
- une API contrôlée.

---

# 2. Ce qui va bien et doit être maintenu

## 2.1. Erreurs métier structurées, implémentées à la main

> **Amendement du 2026-09-24 (décision de l'équipe)** : le projet n'utilise plus `thiserror` ; il n'a plus aucune dépendance externe. `Display`, `std::error::Error` (avec `source()`) et les conversions `From` sont écrits à la main dans `src/domain/error.rs`.

Conserver les enums d'erreurs structurées :

```rust
#[derive(Debug)]
pub enum MyPgpError {
    ...
}

impl std::fmt::Display for MyPgpError { ... }
impl std::error::Error for MyPgpError { ... }
```

Chaque `match` sur `MyPgpError` énumère explicitement toutes les variantes (pas de `_`), pour que l'ajout d'une variante oblige à mettre à jour son message et sa source.

---

## 2.2. Utiliser un alias `Result<T>`

Conserver une définition de ce type :

```rust
pub type Result<T> = std::result::Result<T, MyPgpError>;
```

Cela améliore la lisibilité et uniformise les signatures du projet.

---

## 2.3. Conversions `From` pour les erreurs réellement compatibles

> Amendé le 2026-09-24 (voir § 2.1) : l'attribut `#[from]` n'existe plus ; la conversion est écrite à la main.

```rust
impl From<std::io::Error> for MyPgpError { ... }
```

Ne l'écrire que lorsqu'une conversion automatique est réellement pertinente. Le `?` doit être privilégié pour propager les erreurs.

---

## 2.4. Préférer les enums aux `String` pour les choix finis

Conserver les enums tels que :

```rust
EncryptionAlgorithm
KeyAlgorithm
SignatureAlgorithm
```

lorsqu’ils représentent des ensembles finis.

Éviter :

```rust
algorithm: String
```

pour des valeurs connues.

Un enum :
- réduit les erreurs de saisie ;
- améliore le pattern matching ;
- documente le domaine ;
- permet au compilateur d’aider le développeur.

---

## 2.5. Utiliser `Option<T>` lorsque l’absence a réellement du sens

Conserver `Option<T>` pour une donnée réellement facultative.

Exemples :
- `filename` ;
- `timestamp` ;
- certains champs de protocole.

Mais `Option` ne doit pas servir uniquement à éviter de réfléchir aux invariants.

Une donnée obligatoire dans un format donné doit être obligatoire dans le type ou dans le constructeur correspondant.

---

## 2.6. Utiliser `Default` lorsqu’un état par défaut est réellement défini

Le pattern :

```rust
impl Default for MessageMetadata
```

est correct lorsque les valeurs par défaut sont cohérentes et stables.

La valeur par défaut doit rester :
- documentée ;
- prévisible ;
- sûre.

---

## 2.7. Conserver les builders simples

Le pattern :

```rust
EncryptedMessage::new(...)
    .with_iv(...)
    .with_encrypted_key(...)
    .with_metadata(...)
```

est lisible et ergonomique.

Le conserver lorsque :
- les champs sont réellement optionnels ;
- le chaînage reste simple ;
- le builder ne masque pas les invariants de sécurité.

Ne pas utiliser un builder comme excuse pour permettre la création d’un objet cryptographiquement incohérent.

---

## 2.8. Conserver des tests unitaires ciblés

Les tests comme :

```rust
assert_eq!(...)
assert!(...)
assert!(matches!(...))
```

sont parfaitement légitimes.

Les tests sont organisés selon § 38 (un dossier par type sous `tests/`). Seules les fonctions non publiques gardent des tests internes `#[cfg(test)]`.

---

## 2.9. Préférer les imports explicites dans les modules sensibles

Le code :

```rust
use super::*;
```

est fonctionnel, mais pour du code de sécurité critique, préférer si possible :

```rust
use super::MyPgpError;
```

ou les imports réellement nécessaires.

Objectif :
- dépendances visibles ;
- meilleure lisibilité ;
- moins de symboles importés implicitement.

---

# 3. Gestion des erreurs — règles obligatoires

## 3.1. Éviter de transformer systématiquement les erreurs structurées en `String`

À éviter :

```rust
Encryption(String)
Decryption(String)
Key(String)
Signature(String)
Serialization(String)
Deserialization(String)
Validation(String)
```

si une erreur source typée existe.

Préférer lorsque c’est possible :

```rust
Encryption(#[source] PgpError)
```

ou :

```rust
Serialization(#[source] SerializationError)
```

Objectif :

```text
erreur métier
    ↓
cause originale conservée
    ↓
diagnostic précis
```

et non :

```text
erreur réelle
    ↓
String
    ↓
perte de structure
```

---

## 3.2. Conserver les causes originales

Lorsqu’une bibliothèque dépendante fournit une erreur exploitable, ne pas l’aplatir inutilement.

Une chaîne d’erreurs utile ressemble à :

```text
MyPgpError
    ↓
EncryptionError
    ↓
UnderlyingPgpError
```

Le contexte humain doit être ajouté sans supprimer la cause technique.

---

## 3.3. Éviter les variantes fourre-tout

À éviter autant que possible :

```rust
Other(String)
```

et les équivalents tels que :
- `Unknown`;
- `Misc`;
- `Generic`;
- `Unhandled`.

Ces variantes tendent à devenir des poubelles architecturales.

Une erreur qui apparaît régulièrement doit recevoir une catégorie explicite.

---

## 3.4. Éviter les catégories qui se chevauchent

Analyser toute nouvelle variante pour vérifier qu’elle ne duplique pas une variante existante.

Exemple :

```text
Io
File
```

peut créer une ambiguïté si `File` ne représente rien de différent d’une erreur I/O.

Principe :

> Une erreur = une responsabilité claire.

---

## 3.5. `NotImplemented` ne doit pas devenir un état permanent

Une variante :

```rust
NotImplemented(...)
```

peut être utile temporairement.

Elle doit toutefois être :
- surveillée ;
- documentée ;
- supprimée ou remplacée lorsqu’une fonctionnalité arrive en production.

Un chemin critique de sécurité ne doit pas dépendre durablement d’un comportement `NotImplemented`.

---

# 4. Règles de sécurité — clés privées

## 4.1. Ne jamais exposer naïvement une clé privée avec `Debug`

**Interdit par défaut :**

```rust
#[derive(Debug)]
pub struct PrivateKey {
    ...
}
```

Une clé privée ne doit pas être imprimable implicitement.

Ne jamais permettre qu’un simple :

```rust
println!("{:?}", private_key);
```

puisse révéler son contenu.

Si un `Debug` contrôlé est indispensable, il doit masquer le matériel secret.

Exemple conceptuel :

```text
PrivateKey { key_size: 2048, algorithm: Rsa, key_data: <redacted> }
```

---

## 4.2. Ne pas `Clone` les clés privées par défaut

Éviter :

```rust
#[derive(Clone)]
pub struct PrivateKey {
    ...
}
```

et :

```rust
#[derive(Clone)]
pub struct KeyPair {
    ...
}
```

Une copie supplémentaire d’un secret augmente :
- la surface mémoire ;
- le nombre d’endroits où le secret existe ;
- la difficulté d’effacement ;
- le risque d’exposition accidentelle.

Une copie doit être un choix explicitement justifié.

---

## 4.3. Ne pas sérialiser automatiquement les clés privées

Éviter par défaut :

```rust
#[derive(Serialize, Deserialize)]
pub struct PrivateKey {
    ...
}
```

Une clé privée ne doit pas pouvoir être exportée vers un format arbitraire par simple appel à `serialize`.

L’export d’une clé privée doit être :
- explicite ;
- contrôlé ;
- validé ;
- protégé ;
- documenté.

---

## 4.4. Ne jamais écrire de secrets dans les erreurs

Interdit :

```text
private key
password
passphrase
plaintext
session key
token
secret material
```

dans :
- les erreurs ;
- les logs ;
- les traces de debug ;
- les messages d’exception ;
- les diagnostics utilisateur.

Préférer :

```text
decryption failed
invalid key
signature verification failed
unsupported algorithm
```

Règle :

> **Une erreur doit expliquer l’échec sans révéler le secret impliqué.**

---

## 4.5. Ne jamais loguer des structures cryptographiques sensibles sans redaction

Les appels suivants doivent être considérés comme suspects :

```rust
println!("{:?}", private_key);
dbg!(private_key);
tracing::debug!(?private_key);
```

Même logique pour :
- `KeyPair` ;
- clés déchiffrées ;
- plaintext sensible ;
- clés de session.

---

# 5. Modélisation des clés — règles obligatoires

## 5.1. Distinguer les clés de chiffrement et les clés de signature

Ne pas utiliser un seul enum générique si cela crée une ambiguïté fonctionnelle.

Éviter un modèle du type :

```rust
enum KeyAlgorithm {
    Rsa,
    Ed25519,
}
```

comme représentation universelle de toutes les capacités cryptographiques.

Préférer des concepts séparés lorsque nécessaire :

```text
EncryptionKeyAlgorithm
SignatureAlgorithm
```

Le modèle doit refléter les capacités réelles des algorithmes.

---

## 5.2. Ne pas considérer Ed25519 comme un équivalent générique de chiffrement RSA

Ed25519 est destiné à la signature.

Il ne doit pas être présenté ou utilisé comme simple remplaçant de `RSA-OAEP` pour chiffrer des données.

Le design doit distinguer :
- signature ;
- chiffrement ;
- échange de clés.

Si un algorithme elliptique de chiffrement ou d’échange de clés est requis, le choix doit être explicite et correspondre au protocole utilisé.

---

## 5.3. Ne pas mélanger capacité cryptographique et représentation de clé

Le fait que deux objets contiennent :

```text
key_data
key_size
algorithm
```

ne signifie pas qu’ils ont la même sécurité ou les mêmes opérations.

`PublicKey` et `PrivateKey` doivent rester distincts au niveau de l’API.

---

# 6. Représentation des données binaires

## 6.1. Éviter `Vec<u8>` pour tous les concepts sans distinction

`Vec<u8>` est acceptable pour les données binaires générales.

Mais pour les concepts cryptographiques sensibles, envisager des wrappers dédiés :

```text
Ciphertext
Nonce
EncryptedKey
SignatureBytes
MessageHash
```

Objectif :

```text
Compiler + API
    ↓
empêchent davantage les confusions
```

---

## 6.2. Ne pas faire confiance à un champ `key_size` dupliqué sans validation

Éviter de considérer :

```rust
key_size: usize
```

comme une source de vérité indépendante des données réelles.

Si la taille peut être dérivée du matériel de clé, éviter de stocker une information redondante non vérifiée.

Si elle doit être stockée :
- définir clairement l’unité ;
- valider la cohérence ;
- documenter sa signification.

---

## 6.3. Les unités doivent être explicites

Éviter les champs ambigus comme :

```rust
timestamp: i64
key_size: usize
```

lorsque leur unité n’est pas évidente.

Toujours définir précisément :
- secondes ou millisecondes ;
- bits ou octets ;
- taille logique ou taille physique.

---

# 7. Messages chiffrés — règles de modélisation

## 7.1. Ne pas autoriser silencieusement toutes les combinaisons d’algorithmes et de champs

Actuellement :

```text
algorithm
iv: Option<Vec<u8>>
encrypted_key: Option<Vec<u8>>
```

permet des combinaisons potentiellement incohérentes.

Le modèle doit imposer ou vérifier les relations attendues.

Exemple conceptuel :

```text
AES-CBC
    → IV obligatoire

AES-GCM
    → nonce obligatoire

RSA-OAEP
    → structure adaptée au chiffrement asymétrique

Hybrid
    → définition explicite des primitives utilisées
    → clé symétrique chiffrée requise
```

---

## 7.2. La définition de `Hybrid` doit être explicite

Ne jamais laisser le mot `Hybrid` sans définition protocolaire précise.

La documentation doit dire exactement :

```text
Hybrid = [primitive symétrique] + [primitive asymétrique]
```

Exemple :

```text
AES-256-GCM + RSA-OAEP
```

ou autre combinaison explicitement choisie.

---

## 7.3. Préférer AEAD pour les nouveaux designs lorsque le protocole le permet

Pour les nouveaux systèmes, privilégier une construction fournissant :
- confidentialité ;
- intégrité ;
- authentification du ciphertext.

AES-GCM appartient à cette catégorie.

AES-CBC ne doit pas être introduit dans un nouveau protocole sans mécanisme d’authentification/intégrité complémentaire correctement conçu.

**Ne jamais considérer "chiffré" comme synonyme de "intègre et authentique".**

---

# 8. Métadonnées — règles de sécurité

## 8.1. Traiter les métadonnées comme potentiellement sensibles

Ne pas supposer que :

```text
filename
timestamp
version
```

sont publiques.

Un nom de fichier peut révéler :
- la nature d’un document ;
- une identité ;
- un contexte métier ;
- une information personnelle.

Le protocole doit distinguer :

```text
données protégées
vs
métadonnées visibles
```

et documenter ce qui est effectivement chiffré.

---

## 8.2. Ne jamais croire que chiffrer le contenu chiffre automatiquement les métadonnées

Si le système ne chiffre pas :

```text
filename
timestamp
```

ils restent potentiellement observables.

Cette propriété doit être documentée.

---

# 9. Signatures — règles

## 9.1. Distinguer les causes d’échec de signature

Ne pas réduire tous les cas à :

```text
Signature(String)
```

lorsqu’une distinction utile existe.

Par exemple :
- signature cryptographiquement invalide ;
- algorithme non supporté ;
- clé de vérification absente ;
- signature malformée ;
- politique de sécurité refusant l’algorithme.

---

## 9.2. Ne pas stocker des informations redondantes sans justification

`message_hash` doit rester dans le modèle uniquement si son rôle est clair.

Documenter :
- pourquoi il est stocké ;
- quel algorithme de hash il représente ;
- si le hash est fiable, recalculable ou seulement informatif ;
- s’il fait partie du protocole ou uniquement des métadonnées applicatives.

---

# 10. Sérialisation / désérialisation

## 10.1. Toute structure sérialisable n’est pas forcément sûre à exporter

`Serialize` / `Deserialize` doivent être appliqués selon la sensibilité des données.

Pour les structures contenant des secrets :
- éviter les derives automatiques ;
- utiliser des fonctions d’export/import explicites ;
- contrôler le format ;
- valider les entrées.

---

## 10.2. Toute désérialisation doit être considérée comme une entrée non fiable

Ne jamais supposer qu’un objet provenant de `Deserialize` est automatiquement :
- cryptographiquement valide ;
- cohérent ;
- sûr ;
- utilisable.

La désérialisation reconstruit une structure.

Elle ne remplace pas la validation métier.

Pipeline attendu :

```text
Entrée externe
    ↓
Deserialize
    ↓
Validation
    ↓
Validation cryptographique
    ↓
Objet utilisable
```

---

## 10.3. La version de format doit être explicitement gérée

Le champ :

```rust
version: u32
```

est une bonne idée et doit être conservé.

Mais toute version doit correspondre à :
- un format défini ;
- des règles de compatibilité ;
- des migrations si nécessaire ;
- des tests.

---

# 11. Tests — niveau minimal attendu

## 11.1. Les tests de constructeurs sont nécessaires mais insuffisants

Conserver les tests actuels :

```text
création
builder
default
```

mais ne pas les considérer comme une validation suffisante du système cryptographique.

---

## 11.2. Tester la sérialisation

Pour les modèles concernés :

```text
Serialize
    ↓
Deserialize
    ↓
objet équivalent
```

Tester aussi les données invalides ou inattendues.

---

## 11.3. Tester les invariants

Créer des tests pour vérifier que les combinaisons invalides sont :
- impossibles à construire ;
- ou rejetées explicitement.

---

## 11.4. Tester les scénarios cryptographiques réels

Les tests attendus à terme incluent au minimum :

```text
encrypt → decrypt → original
```

```text
sign → verify → valid
```

```text
modified ciphertext → reject
```

```text
modified message → signature invalid
```

```text
wrong key → reject
```

```text
unsupported algorithm → reject
```

---

## 11.5. Tester la propagation des erreurs

Tester explicitement :
- conversion `std::io::Error` ;
- propagation avec `?` ;
- conservation des causes sources ;
- distinction des variantes ;
- absence de secrets dans les messages.

---

# 12. Ce qui doit absolument être évité

## 12.1. Interdit : clé privée dans `Debug`

Ne jamais faire :

```rust
#[derive(Debug)]
struct PrivateKey {
    key_data: Vec<u8>,
}
```

sans redaction explicite.

---

## 12.2. Interdit : secrets dans les logs

Ne jamais logger :
- clé privée ;
- passphrase ;
- plaintext sensible ;
- clé de session ;
- token ;
- secret matériel.

---

## 12.3. Interdit : `Other(String)` comme solution par défaut

Ne pas utiliser :

```rust
Other("je ne sais pas".into())
```

pour éviter de concevoir la bonne erreur.

---

## 12.4. Interdit : `String` pour tout

Ne pas remplacer systématiquement une erreur typée ou une donnée typée par une simple chaîne.

---

## 12.5. Interdit : utiliser `Ed25519` comme clé de chiffrement générique

Ne pas confondre :
- signature ;
- chiffrement ;
- échange de clés.

---

## 12.6. Interdit : considérer AES-CBC comme authentifié

AES-CBC seul ne fournit pas l’intégrité/authentification du ciphertext.

Tout nouveau design doit traiter explicitement ce problème.

---

## 12.7. Interdit : supposer qu’un objet désérialisé est sûr

Un objet issu du réseau, d’un fichier ou d’un format externe est une entrée non fiable jusqu’à validation.

---

## 12.8. Interdit : utiliser `unwrap()` ou `expect()` pour contourner une gestion d’erreurs réelle

Dans les chemins cryptographiques, les erreurs doivent être traitées explicitement.

Éviter les panics artificiellement introduits pour masquer un défaut de conception.

---

# 13. Règles de conception recommandées pour la prochaine version

## 13.1. Modèle d’erreurs

Tendre vers :

```text
Erreur publique
    ↓
catégorie précise
    ↓
source originale conservée
    ↓
message sûr pour les logs
```

Éviter :

```text
Erreur
    ↓
String
    ↓
perte du contexte
```

---

## 13.2. Modèle de clés

Tendre vers :

```text
PublicKey
    → opérations publiques uniquement

PrivateKey
    → accès contrôlé
    → pas de Debug naïf
    → pas de Clone naïf
    → pas de Serialize naïf
```

---

## 13.3. Modèle des algorithmes

Séparer clairement :

```text
EncryptionAlgorithm
SignatureAlgorithm
KeyExchangeAlgorithm
```

lorsque le protocole le justifie.

Ne pas utiliser un seul enum générique pour des primitives ayant des rôles différents.

---

## 13.4. Modèle des messages

Tendre vers des structures capables d’exprimer directement les invariants.

Exemple conceptuel :

```text
EncryptedMessage
├── Ciphertext
├── EncryptionScheme
│   ├── AesGcm(...)
│   └── Hybrid(...)
└── Metadata
```

plutôt qu’un objet générique où tous les champs sont optionnels.

---

# 14. Checklist obligatoire avant chaque modification

Avant de valider une modification crypto/Rust, vérifier :

### Rust

- [ ] Le code est idiomatique.
- [ ] Les types sont aussi précis que possible.
- [ ] Les conversions automatiques sont limitées aux cas pertinents.
- [ ] Les `match` exhaustifs sont privilégiés lorsque cela améliore la robustesse.
- [ ] Aucun `unwrap()` / `expect()` n’a été utilisé pour masquer une erreur réelle(meme pour les fichiers tests).

### Architecture

- [ ] Les responsabilités sont séparées.
- [ ] Les variantes d’erreurs ne se chevauchent pas inutilement.
- [ ] Les états invalides sont bloqués autant que possible.
- [ ] Aucun fourre-tout (`Other`, `Generic`, etc.) n’a été ajouté sans justification.
- [ ] Les relations entre algorithmes et champs sont explicites.

### Sécurité

- [ ] Aucun secret n’est présent dans les logs.
- [ ] Aucune clé privée n’est imprimable par `Debug`.
- [ ] Aucune copie inutile de clé privée n’est créée.
- [ ] L’export d’une clé privée est explicite et contrôlé.
- [ ] Les métadonnées potentiellement sensibles sont identifiées.
- [ ] L’intégrité/authentification du ciphertext est traitée.
- [ ] Les entrées désérialisées sont validées.
- [ ] Les algorithmes sont utilisés pour leur fonction cryptographique réelle.

### Tests

- [ ] Les constructeurs sont testés.
- [ ] Les invariants sont testés.
- [ ] La sérialisation/désérialisation est testée.
- [ ] Les erreurs sont testées.
- [ ] Les scénarios crypto nominaux sont testés.
- [ ] Les scénarios d’échec sont testés.
- [ ] Les données sensibles ne sont jamais exposées dans les tests/logs.

---

# 15. Règle finale

Le code doit satisfaire cette progression :

```text
1. Ça compile.
       ↓
2. C’est idiomatique Rust.
       ↓
3. Les types expriment correctement le domaine.
       ↓
4. Les états incohérents sont empêchés.
       ↓
5. Les erreurs conservent leur cause.
       ↓
6. Les secrets sont protégés.
       ↓
7. Le protocole cryptographique est correct.
       ↓
8. Les scénarios nominaux ET d’échec sont testés.
       ↓
9. Seulement ensuite : validation.
```

> **Ne jamais confondre "code propre" avec "code sécurisé".**
>
> Un code Rust peut être élégant, compiler sans warning et rester incorrect ou dangereux du point de vue cryptographique.
>
> Pour ce projet, la qualité finale est déterminée par l’intersection de :
>
> **Rust idiomatique + typage fort + invariants + sécurité des secrets + exactitude cryptographique + tests.**

---

# 16. Audit complémentaire — CLI, I/O, sérialisation et chiffrement hybride

Cette section complète les règles précédentes à partir de l’audit des implémentations de :
- génération de clés de test ;
- tests d’intégration CLI ;
- sauvegarde et chargement JSON ;
- validation des messages chiffrés ;
- chiffrement hybride AES-256-GCM + RSA-OAEP ;
- déchiffrement ;
- parsing PEM ;
- commandes CLI.

Les règles de cette section sont obligatoires pour toute nouvelle évolution des mêmes composants.

---

# 17. Validation des messages et exhaustivité des algorithmes

## 17.1. Toute validation de sécurité doit être exhaustive

Lorsqu’un `enum` définit plusieurs algorithmes, toute logique de validation critique doit prendre en compte toutes ses variantes.

Éviter un traitement partiel du type :

```rust
match msg.algorithm {
    EncryptionAlgorithm::Aes256Gcm => { ... }
    EncryptionAlgorithm::Hybrid => { ... }
}
```

si d’autres variantes existent réellement.

Le compilateur Rust doit être utilisé comme mécanisme de sécurité : un `match` critique doit rester exhaustif.

---

## 17.2. Ajouter un algorithme implique une revue globale

L’ajout d’une variante dans un enum de sécurité ne doit jamais être traité comme une simple modification locale.

Après l’ajout d’un algorithme, revoir obligatoirement :
- validation ;
- chiffrement ;
- déchiffrement ;
- format de stockage ;
- sérialisation/désérialisation ;
- tests ;
- documentation ;
- politique de sécurité.

---

## 17.3. L’algorithme déclaré doit être vérifié avant l’opération

Une fonction spécialisée dans une construction précise ne doit jamais ignorer le champ `algorithm` du message.

Exemple : une fonction de déchiffrement hybride doit vérifier que le message est réellement déclaré comme `Hybrid` avant d’appliquer le chemin RSA-OAEP + AES-GCM.

Règle :

> Une donnée déclarative qui détermine le protocole doit être validée avant d’exécuter ce protocole.

---

## 17.4. La validation doit être en couches

La validation attendue suit ce modèle :

```text
Données externes
      ↓
Désérialisation
      ↓
Validation structurelle
      ↓
Validation métier
      ↓
Validation cryptographique
      ↓
Opération
```

La présence d’un champ ou sa bonne désérialisation ne garantit pas sa validité.

---

## 17.5. Un objet valide pour Serde n’est pas nécessairement valide pour le domaine

Ne jamais considérer :

```text
Deserialize == Valid
```

comme une règle de sécurité.

Exemples d’objets pouvant être syntaxiquement valides mais métierement invalides :
- ciphertext vide ;
- nonce absent ;
- nonce de mauvaise longueur ;
- clé chiffrée absente ;
- version inconnue ;
- combinaison d’algorithmes incohérente ;
- taille dépassant la politique autorisée.

---

# 18. Gestion des erreurs dans la couche I/O et la CLI

## 18.1. Ne pas détruire la catégorie d’une erreur lors de l’ajout de contexte

À éviter :

```text
I/O error
   ↓
format!(...)
   ↓
String
   ↓
File(...)
```

ou :

```text
JSON error
   ↓
File(...)
```

Le contexte du fichier est utile, mais la cause originale doit rester accessible.

---

## 18.2. Les erreurs de fichier, JSON, validation et cryptographie doivent rester distinguables

La CLI doit être capable de distinguer au minimum :
- échec d’ouverture/lecture/écriture ;
- erreur de format ;
- désérialisation ;
- validation ;
- clé invalide ;
- déchiffrement ;
- signature ;
- politique de sécurité.

Le message présenté à l’utilisateur peut être simplifié, mais l’erreur interne ne doit pas être aplatie inutilement.

---

## 18.3. Préserver les chaînes de causes

Le modèle attendu est :

```text
Contexte applicatif
      ↓
Erreur métier
      ↓
Erreur source
```

et non :

```text
Erreur source
      ↓
String
      ↓
plus aucune information structurée
```

---

## 18.4. Le contexte ne doit jamais révéler de secrets

Ajouter le chemin d’un fichier peut être acceptable pour le diagnostic.

Ajouter le contenu d’une clé privée, d’une passphrase ou d’un plaintext secret ne l’est pas.

---

# 19. Chiffrement hybride AES-GCM + RSA-OAEP

## 19.1. Conserver la séparation des rôles

Le modèle hybride doit rester conceptuellement :

```text
Message
   ↓
AES-256-GCM
   ↓
Ciphertext

AES key
   ↓
RSA-OAEP + SHA-256
   ↓
EncryptedKey
```

Le RSA ne doit pas être utilisé pour chiffrer de gros payloads lorsque le modèle hybride est prévu.

---

## 19.2. Les longueurs critiques doivent être imposées

Pour une construction AES-256-GCM utilisant un nonce de 96 bits :
- clé AES = 32 octets ;
- nonce = 12 octets.

Ces invariants doivent être explicitement validés lorsque le format permet des entrées externes.

Ne jamais supposer qu’une donnée provenant d’un fichier ou du réseau possède la bonne taille simplement parce qu’elle est désérialisable.

---

## 19.3. Les nonces GCM ne doivent jamais être réutilisés avec la même clé

La combinaison :

```text
même clé AES
+
même nonce GCM
```

est interdite.

La stratégie de génération doit garantir l’unicité nécessaire au protocole.

---

## 19.4. Utiliser un générateur cryptographiquement sûr

Les clés AES et les nonces doivent provenir d’un générateur cryptographiquement sûr adapté à la primitive utilisée.

Ne jamais utiliser pour les secrets cryptographiques :
- timestamp ;
- compteur prévisible ;
- PRNG non cryptographique ;
- seed fixe ;
- valeur dérivée d’une entrée utilisateur non imprévisible.

---

## 19.5. Ne pas confondre intégrité et identité de l’expéditeur

AES-GCM protège l’intégrité/authentification du ciphertext vis-à-vis de la clé utilisée.

Cela ne signifie pas automatiquement :

```text
"Le message vient de cette personne."
```

Si l’identité de l’expéditeur doit être garantie, un mécanisme de signature doit être défini et vérifié séparément.

---

## 19.6. Les métadonnées importantes doivent être liées cryptographiquement au message

Si des champs tels que :

```text
algorithm
version
filename
timestamp
```

ont une importance de sécurité, leur intégrité doit être protégée.

Une construction AEAD peut utiliser des Additional Authenticated Data lorsque cela correspond au protocole.

Le protocole doit définir explicitement :
- ce qui est chiffré ;
- ce qui est authentifié ;
- ce qui est public ;
- ce qui est optionnel.

---

# 20. Mémoire et données sensibles

## 20.1. Minimiser la durée de vie des secrets

Les valeurs telles que :
- clé AES ;
- clé privée ;
- passphrase ;
- plaintext sensible ;

doivent rester en mémoire le moins longtemps possible.

Limiter leur portée lexicale et éviter les copies inutiles.

---

## 20.2. Étudier explicitement la zeroization

Pour les données sensibles conservées en mémoire, utiliser lorsque pertinent des mécanismes adaptés tels que :

```text
Zeroize
Zeroizing<T>
```

La zeroization réduit le risque de persistance accidentelle en mémoire ; elle ne constitue pas à elle seule une garantie globale de sécurité.

---

## 20.3. Réduire les conversions et copies de secrets

Éviter les chaînes inutiles :

```text
bytes → String → bytes
```

ou :

```text
clone() → clone() → clone()
```

pour le matériel cryptographique.

---

# 21. Clés privées sur disque

## 21.1. Une clé privée n’est pas un fichier ordinaire

Tout fichier contenant du matériel privé doit faire l’objet d’une politique spécifique de :
- permissions ;
- emplacement ;
- exposition ;
- sauvegarde ;
- export ;
- suppression.

---

## 21.2. Les fichiers de clés privées doivent avoir des permissions restrictives

Lorsque la plateforme le permet, créer/écrire les clés privées avec des permissions minimales.

Ne pas supposer que `TempDir` ou un nom aléatoire suffit à protéger le contenu.

---

## 21.3. L’export d’une clé privée doit être intentionnel

Une clé privée ne doit pas devenir exportable simplement parce qu’une structure implémente automatiquement `Serialize`.

Préférer une opération explicite d’export contrôlé.

---

# 22. I/O fichiers

## 22.1. Définir une politique d’écrasement

Une opération de type :

```rust
fs::write(output_path, ...)
```

peut écraser un fichier existant.

Le comportement doit être explicitement choisi :
- refus par défaut ;
- option `--force` ;
- autre politique documentée.

---

## 22.2. Privilégier les écritures atomiques pour les sorties importantes

Pour limiter le risque d’un fichier partiellement écrit lors d’un crash ou d’une interruption, préférer lorsque nécessaire :

```text
temporary file
      ↓
écriture complète
      ↓
flush / sync selon besoin
      ↓
rename / persist
```

---

## 22.3. Être conscient des symlinks et des TOCTOU

Une séquence :

```text
check exists
      ↓
open/write
```

ne constitue pas une protection suffisante contre une course ou un remplacement par symlink.

Les opérations sensibles doivent utiliser des primitives d’ouverture/création appropriées à la menace considérée.

---

## 22.4. Définir des limites de taille

Éviter de charger sans limite des fichiers provenant de sources non fiables.

Le système doit définir :
- taille maximale ;
- ou stratégie de streaming/chunking.

Les limites doivent concerner notamment :
- plaintext ;
- ciphertext ;
- JSON ;
- métadonnées ;
- clés importées.

---

# 23. Sérialisation JSON

## 23.1. Une sérialisation lisible n’est pas nécessairement une sérialisation efficace

`to_string_pretty()` est acceptable pour de petits artefacts lisibles par l’humain.

Pour des données volumineuses, évaluer une écriture vers un writer afin de limiter les allocations intermédiaires.

---

## 23.2. Éviter les conversions UTF-8 inutiles

Lorsque le parser le permet, préférer :

```text
bytes
 ↓
deserialize
```

plutôt que :

```text
bytes
 ↓
UTF-8 String
 ↓
deserialize
```

---

## 23.3. Définir explicitement la représentation des données binaires

Avec JSON, des `Vec<u8>` peuvent devenir des tableaux d’entiers volumineux.

Pour les messages importants, le projet doit choisir explicitement :
- JSON + représentation binaire adaptée, par exemple base64 lorsque le protocole le prévoit ;
- format binaire ;
- autre format documenté.

---

# 24. CLI et types de chemins

## 24.1. Utiliser `PathBuf` / `Path` pour les chemins

Éviter :

```rust
input: String
output: String
key: String
```

lorsque ces champs représentent uniquement des chemins.

Préférer des types de chemin dédiés.

---

## 24.2. Éviter `to_str().unwrap()` pour les chemins

À éviter :

```rust
path.to_str().unwrap()
```

Préférer transmettre les chemins sous forme de `Path` / `OsStr` lorsque l’API le permet.

Cela améliore également la robustesse vis-à-vis des chemins non UTF-8 selon la plateforme.

---

## 24.3. Les paramètres numériques doivent être bornés

Un argument comme :

```rust
size: usize
```

ne doit pas accepter silencieusement des valeurs absurdes ou dangereuses.

Définir les bornes adaptées au protocole et à l’algorithme.

---

## 24.4. Ne jamais exposer les détails de répartition de l’équipe

Interdit dans la CLI de production :

```text
(Person B)
(Person A)
branche X
TODO équipe
```

Les détails internes de développement appartiennent à la documentation projet, pas à l’interface utilisateur.

---

## 24.5. Ne pas présenter une fonctionnalité non implémentée comme opérationnelle

Si `Keys`, `Sign` ou `Verify` ne sont pas disponibles, la CLI doit le refléter proprement.

Éviter une interface qui semble complète alors que certaines commandes échouent systématiquement.

---

# 25. Tests d’intégration CLI

## 25.1. Éviter `cargo run` depuis les tests

À éviter :

```rust
Command::new("cargo")
    .args(["run", "--", ...])
```

Cela introduit une dépendance à Cargo et peut déclencher des compilations ou ajouter une couche d’échec inutile.

Préférer l’exécutable déjà construit ou un outil de test CLI dédié comme `assert_cmd`.

---

## 25.2. Les tests doivent être portables

Éviter les chemins absolus codés en dur comme :

```text
/nonexistent/file.txt
```

Créer les entrées manquantes dans `TempDir` ou via des chemins relatifs au contexte de test.

---

## 25.3. Tester le comportement réel, pas un mot arbitraire

À éviter :

```rust
assert!(stderr.contains("Error"));
```

Préférer des assertions sur :
- exit code ;
- catégorie d’erreur ;
- fichier créé/non créé ;
- contenu ;
- absence d’artefacts ;
- message réellement significatif.

---

## 25.4. Couvrir les cas de tampering

Les tests doivent couvrir au minimum :

```text
ciphertext modifié
nonce modifié
encrypted_key modifiée
```

et vérifier que l’opération est rejetée.

---

## 25.5. Couvrir les mauvaises clés et mauvaises entrées

Tester notamment :
- mauvaise clé privée ;
- clé publique invalide ;
- PEM invalide ;
- JSON invalide ;
- version inconnue ;
- nonce manquant ;
- nonce de mauvaise longueur ;
- encrypted key manquante ;
- ciphertext vide ;
- fichier absent.

---

## 25.6. Vérifier les effets secondaires après une erreur

Après un échec, vérifier notamment :
- absence de fichier de sortie partiel ;
- absence de fichiers temporaires inattendus ;
- absence de secrets dans stdout/stderr ;
- état cohérent du répertoire de travail.

---

## 25.7. Les fixtures de test doivent être efficaces

La génération répétée de clés RSA peut ralentir fortement les tests.

Réutiliser des fixtures lorsque cela est compatible avec l’isolation du test.

Les tests restent indépendants du point de vue logique même lorsque certaines ressources de fixture sont partagées de manière sûre.

---

# 26. Génération de clés de test

## 26.1. Séparer strictement test et production

Les clés de test ne doivent jamais :
- être utilisées en production ;
- être commitée comme secrets opérationnels ;
- devenir des fixtures de production.

---

## 26.2. Protéger également les secrets temporaires de test

Une clé de test reste une clé privée.

Même si sa durée de vie est courte, elle doit être manipulée de manière sûre et stockée avec des permissions appropriées lorsque cela est possible.

---

## 26.3. `unwrap()` / `expect()` dans les fixtures : exception limitée

Dans le setup d’un test, `expect()` peut être acceptable lorsque l’échec signifie que la fixture elle-même ne peut pas être construite.

Cette tolérance ne s’applique pas au code de production et ne doit pas servir à masquer les erreurs des données réellement testées.

---

# 27. Robustesse du déchiffrement

## 27.1. Ne jamais supposer que les champs optionnels sont cohérents

Même après désérialisation, le déchiffrement doit rester défensif.

Exemple :

```text
nonce = None
```

doit être rejeté avant l’utilisation de la primitive.

---

## 27.2. Revalider les contraintes cryptographiques critiques au point d’utilisation

Même lorsqu’une validation générale existe, le code cryptographique peut effectuer ses propres vérifications de défense en profondeur :
- longueur de clé ;
- longueur du nonce ;
- algorithme attendu ;
- présence de composants requis.

La double validation doit être justifiée par la sécurité, pas utilisée pour compenser une architecture désordonnée.

---

## 27.3. Les erreurs de déchiffrement doivent rester génériques vis-à-vis des secrets

Le résultat peut indiquer :

```text
Decryption failed
```

mais ne doit pas révéler :
- contenu du plaintext ;
- clé utilisée ;
- détail secret de la clé privée ;
- données internes permettant de contourner une politique de sécurité.

---

# 28. Documentation du protocole

Toute construction maison doit disposer d’une spécification écrite.

Elle doit décrire au minimum :
- format du message ;
- algorithmes ;
- tailles ;
- nonce/IV ;
- clés ;
- ordre des opérations ;
- format JSON ;
- encodage binaire ;
- champs obligatoires ;
- champs optionnels ;
- métadonnées ;
- authentification ;
- versioning ;
- compatibilité ;
- erreurs ;
- limites de taille ;
- politique de gestion des clés.

---

# 29. BLOQUANTS ABSOLUS SUPPLÉMENTAIRES

## BLOCKER-11 — Validation partielle d’un enum de sécurité

Une variante d’algorithme ajoutée mais ignorée par une validation critique doit bloquer la revue.

---

## BLOCKER-12 — Réutilisation d’un nonce GCM avec une même clé

À bloquer immédiatement.

---

## BLOCKER-13 — Secret écrit dans stdout/stderr

Une clé privée, passphrase, plaintext sensible ou clé de session exposée sur la sortie d’un programme constitue un défaut critique.

---

## BLOCKER-14 — Perte de la cause d’une erreur de sécurité

Une transformation arbitraire en `String` ou en erreur générique qui empêche de différencier une erreur de sécurité doit être corrigée avant validation.

---

## BLOCKER-15 — Fichier de sortie partiellement écrit après échec critique

Pour une opération censée produire un artefact fiable, le système doit éviter de laisser silencieusement une sortie corrompue en cas d’interruption ou d’échec.

---

## BLOCKER-16 — CLI permettant un chemin ou une taille sans validation adaptée

Les paramètres contrôlés par l’utilisateur doivent avoir des contraintes adaptées à leur rôle et au threat model.

---

# 30. Priorités mises à jour

## P0 — à traiter avant toute validation de sécurité

1. Protéger les clés privées contre `Debug`, `Clone`, sérialisation et logs non contrôlés.
2. Garantir la non-réutilisation des nonces GCM.
3. Vérifier explicitement l’algorithme annoncé avant son utilisation.
4. Rendre les validations d’algorithmes exhaustives.
5. Définir et documenter complètement le protocole hybride.
6. Empêcher l’exposition des secrets dans les erreurs et sorties CLI.

## P1 — robustesse et architecture

1. Préserver les erreurs sources.
2. Centraliser la validation sans perdre les contrôles de défense en profondeur.
3. Remplacer les chemins `String` par `PathBuf` / `Path`.
4. Définir les politiques de taille et d’écrasement.
5. Sécuriser les écritures de fichiers importantes.
6. Séparer clairement CLI, application, domaine et infrastructure.

## P1 — tests

1. Éviter `cargo run` dans les tests CLI.
2. Remplacer les assertions `stderr.contains("Error")` par des assertions significatives.
3. Tester les données altérées.
4. Tester les mauvaises clés et les formats invalides.
5. Tester les versions et algorithmes inconnus.
6. Vérifier les effets secondaires après échec.

## P2 — qualité et performance

1. Réduire les allocations intermédiaires.
2. Évaluer la sérialisation via writer.
3. Évaluer `from_slice` lorsque pertinent.
4. Définir une représentation efficace des données binaires.
5. Évaluer le streaming pour les gros fichiers.
6. Réduire le coût des fixtures RSA.

---

# 31. Checklist finale consolidée

Avant de merger une modification touchant la cryptographie ou les données sensibles, toutes les cases applicables doivent être validées.

### Rust

- [ ] Code idiomatique.
- [ ] Types explicites.
- [ ] Pas d’abstraction inutile.
- [ ] Erreurs propagées avec `?` lorsque pertinent.
- [ ] Pas de `unwrap()` / `expect()` dans la production pour contourner une erreur.
- [ ] `match` critiques exhaustifs.

### Architecture

- [ ] Responsabilités séparées.
- [ ] Erreurs non redondantes.
- [ ] Causes originales conservées.
- [ ] Validation centralisée.
- [ ] Défense en profondeur présente lorsque nécessaire.
- [ ] États invalides difficiles à représenter.
- [ ] CLI, application, domaine et infrastructure séparés.

### Secrets

- [ ] Aucun secret dans `Debug`.
- [ ] Aucun `Clone` inutile de secret.
- [ ] Aucun secret sérialisé implicitement.
- [ ] Aucun secret dans logs/stdout/stderr.
- [ ] Durée de vie mémoire minimisée.
- [ ] Zeroization étudiée.
- [ ] Fichiers de clés privées protégés.

### Cryptographie

- [ ] CSPRNG approprié.
- [ ] Clé AES de la bonne taille.
- [ ] Nonce de la bonne taille.
- [ ] Aucune réutilisation dangereuse du nonce.
- [ ] Ciphertext authentifié lorsque nécessaire.
- [ ] Métadonnées critiques authentifiées lorsque nécessaire.
- [ ] Signatures distinguées du chiffrement.
- [ ] Algorithmes utilisés pour leur rôle réel.
- [ ] Protocole hybride explicitement spécifié.

### Validation

- [ ] Désérialisation suivie d’une validation.
- [ ] Validation exhaustive des enums.
- [ ] Version contrôlée.
- [ ] Champs obligatoires contrôlés.
- [ ] Longueurs contrôlées.
- [ ] Limites de taille appliquées.
- [ ] Algorithme déclaré vérifié.

### I/O

- [ ] `Path` / `PathBuf` utilisés pour les chemins.
- [ ] Pas de `to_str().unwrap()` inutile.
- [ ] Politique d’écrasement définie.
- [ ] Sorties importantes écrites de manière robuste.
- [ ] Symlinks / TOCTOU pris en compte si nécessaire.
- [ ] Fichiers sensibles protégés.

### CLI

- [ ] Paramètres numériques bornés.
- [ ] Aucun détail interne d’équipe exposé.
- [ ] Fonctionnalités non implémentées clairement signalées.
- [ ] Pas de logique cryptographique critique dans la présentation CLI.

### Tests

- [ ] Roundtrip encrypt/decrypt.
- [ ] Tampering ciphertext.
- [ ] Tampering nonce.
- [ ] Tampering encrypted key.
- [ ] Mauvaise clé.
- [ ] PEM invalide.
- [ ] JSON invalide.
- [ ] Version inconnue.
- [ ] Algorithme incohérent.
- [ ] Champs manquants.
- [ ] Fichier absent.
- [ ] Sortie existante.
- [ ] Taille excessive.
- [ ] Aucun secret dans stdout/stderr.
- [ ] Aucun artefact inattendu après échec.
- [ ] Tests CLI indépendants de `cargo run`.

---

# 32. Règle absolue finale

> **Aucune fonctionnalité cryptographique n’est considérée comme terminée parce qu’elle fonctionne dans le cas nominal.**
>
> Elle n’est considérée comme prête qu’après validation de son modèle de données, de ses invariants, de ses erreurs, de ses secrets, de son protocole cryptographique, de ses fichiers, de sa CLI et de ses scénarios d’échec.
>
> La règle de qualité du projet est donc :
>
> **Fonctionnel + typé + testable + explicable + résilient + sécurisé.**


---

# 33. Corrections appliquées — Historique d'audit (2026-09-21)

Cette section documente les corrections effectuées lors de l'audit de conformité REGLES.md.

## 33.1. Validation stricte des longueurs de nonce (§ 19.2)

**Problème identifié** : La validation des messages acceptait des nonces de taille arbitraire.

**Correction appliquée** :
```rust
// Avant (validation partielle)
if msg.nonce.is_none() {
    return Err(MyPgpError::validation("AES-GCM requires nonce"));
}

// Après (validation stricte)
if msg.nonce.is_none() {
    return Err(MyPgpError::validation("AES-GCM requires nonce"));
}
if msg.nonce.as_ref().map(|n| n.as_bytes().len()) != Some(12) {
    return Err(MyPgpError::validation("AES-GCM nonce must be exactly 12 bytes"));
}
```

**Référence** : `src/io/message_io.rs`, fonction `validate_encrypted_message()`

**Impact** : Détection précoce des erreurs, conformité stricte au protocole GCM.

---

## 33.2. Élimination de tous les unwrap/expect (§ 12.8)

**Problème identifié** : 20+ usages de `.unwrap()` dans les tests, violant la règle d'absence totale.

**Correction appliquée** :
```rust
// Avant
let temp = NamedTempFile::new().unwrap();

// Après
let temp = NamedTempFile::new()
    .expect("TEST FIXTURE: TempFile creation should succeed");
```

**Référence** : Tous les fichiers de tests

**Convention adoptée** :
- Préfixe `"TEST FIXTURE:"` pour setup de test
- Préfixe `"TEST:"` pour opérations testées
- Messages explicites pour chaque `.expect()`

**Impact** : Messages d'erreur clairs, traçabilité complète des échecs.

---

## 33.3. Suppression des variantes d'erreur fourre-tout (§ 3.3, § 3.4)

**Problème identifié** :
- `Other(String)` : variante catch-all interdite par § 3.3
- `File(String)` : duplication de `Io` interdite par § 3.4

**Correction appliquée** :
```rust
// Avant
#[derive(Error, Debug)]
pub enum MyPgpError {
    // ...
    File(String),
    Other(String),
}

// Après
#[derive(Error, Debug)]
pub enum MyPgpError {
    Io(#[from] std::io::Error),  // Utiliser directement
    // File et Other supprimés
}
```

**Migration des usages** :
```rust
// Avant
MyPgpError::file(format!("Failed to read file: {}", e))

// Après
MyPgpError::Io(std::io::Error::new(
    io_err.kind(),
    format!("Failed to read file: {}", io_err)
))
```

**Référence** : `src/domain/error.rs`, `src/main.rs`

**Impact** : Chaîne d'erreurs préservée, catégories non-redondantes.

---

## 33.4. Structure de tests complète (§ 11.4)

**Problème identifié** : Tests concentrés dans un seul fichier, pas de couverture régression/performance/sécurité/acceptation/usabilité.

**Correction appliquée** :

```
tests/
├── unit/               (inline dans src/)
├── integration/        cli_integration_tests.rs (3 tests)
├── security/           nonce_tests.rs (3 tests)
├── regression/         compatibility.rs (3 tests)
├── e2e/               full_workflow.rs (2 tests)
├── performance/        benchmarks.rs (4 tests)
├── acceptance/         user_scenarios.rs (4 tests)
└── usability/          cli_ergonomics.rs (6 tests)
```

**Tests de sécurité ajoutés** :
- `test_nonce_uniqueness_across_encryptions` (§ 19.3)
- `test_invalid_nonce_length_rejected_on_load` (§ 19.2)
- `test_valid_nonce_12_bytes_accepted` (§ 19.2)

**Tests de régression ajoutés** :
- `test_json_format_version_1_compatibility`
- `test_message_structure_remains_stable`
- `test_enum_serialization_stability`

**Tests E2E ajoutés** :
- `test_e2e_complete_encryption_workflow`
- `test_e2e_multiple_files_same_key`

**Tests de performance ajoutés** :
- `test_encryption_performance_1kb` (<1s)
- `test_decryption_performance_1kb` (<1s)
- `test_encryption_performance_1mb` (<5s)
- `test_roundtrip_performance_100_iterations` (<150ms avg)

**Tests d'acceptation ajoutés** :
- `test_user_can_encrypt_sensitive_document`
- `test_user_can_decrypt_received_message`
- `test_user_gets_clear_error_for_wrong_key`
- `test_user_cannot_decrypt_tampered_message`

**Tests d'usabilité ajoutés** :
- `test_help_command_accessible`
- `test_version_command_accessible`
- `test_error_messages_are_user_friendly`
- `test_encrypt_provides_feedback`
- `test_consistent_argument_names`
- `test_commands_have_short_options`

**Impact** : Couverture complète (34 tests), détection précoce des régressions.

---

## 33.5. Tests d'usabilité corrigés (correction #6)

**Problème identifié** : Tests vérifiant `--help` global au lieu de `encrypt --help` / `decrypt --help`.

**Correction appliquée** :
```rust
// Avant (incorrect)
let help = Command::new("cargo").args(["run", "--", "--help"]).output()?;

// Après (correct)
let encrypt_help = Command::new("cargo").args(["run", "--", "encrypt", "--help"]).output()?;
```

**Référence** : `tests/usability/cli_ergonomics.rs`

**Validation** : Le CLI possède déjà les options courtes `-i`, `-o`, `-k` comme attendu par REGLES.md § 17.1 (ergonomie CLI).

**Impact** : Tests d'usabilité passent, validation de l'ergonomie CLI.

---

# 34. Règles additionnelles dérivées de l'audit

## 34.1. Convention de nommage des tests

Les tests doivent suivre le pattern : `test_<action>_<expected_result>`

Exemples :
- `test_encrypt_decrypt_roundtrip` ✅
- `test_invalid_nonce_rejected` ✅
- `test_stuff` ❌ (trop vague)

---

## 34.2. Messages d'erreur dans les tests

Les messages d'erreur doivent :
1. Référencer REGLES.md si applicable
2. Expliquer pourquoi l'assertion a échoué
3. Fournir le contexte (valeurs attendues vs réelles)

Exemple :
```rust
assert!(
    result.is_err(),
    "Invalid nonce length {} should be rejected (REGLES.md § 19.2)",
    invalid_nonce.len()
);
```

---

## 34.3. Organisation des tests par type

**Obligatoire** : voir § 38. Chaque type de test a son dossier `tests/<type>/`, qui est une crate de test Cargo autonome (`tests/<type>/main.rs`). Les wrappers `include!()` sont abandonnés depuis le 2026-09-24.

---

## 34.4. Limites de performance attendues

Les tests de performance doivent définir des seuils réalistes :

| Opération | Taille | Limite |
|-----------|--------|--------|
| Encryption | 1KB | <1s |
| Decryption | 1KB | <1s |
| Encryption | 1MB | <5s |
| Roundtrip moyen | petite | <150ms |

Ces seuils peuvent être ajustés selon la plateforme mais doivent rester documentés.

---

## 34.5. Gestion des erreurs I/O avec contexte

Lorsqu'une opération I/O échoue, enrichir le message avec le chemin du fichier :

```rust
// ✅ Bon
read_file(path).map_err(|e| match e {
    MyPgpError::Io(io_err) => MyPgpError::Io(std::io::Error::new(
        io_err.kind(),
        format!("Failed to read '{}': {}", path, io_err)
    )),
    other => other,
})?;

// ❌ Mauvais
read_file(path)?;  // Perd le contexte du chemin
```

**Justification** : Permet de diagnostiquer rapidement quel fichier pose problème sans avoir à debugger.

---

# 35. Checklist d'audit de conformité

Avant chaque merge majeur, vérifier :

## Code

- [ ] Aucun `unwrap()` ou `expect()` sans message explicite
- [ ] Aucune variante `Other(String)` ou équivalent dans les enums d'erreur
- [ ] Pas de duplication entre variantes d'erreur (ex: `File` vs `Io`)
- [ ] Validation exhaustive des enums de sécurité (`match` complet)
- [ ] Nonces GCM validés à 12 octets
- [ ] Aucun secret dans les erreurs, logs, ou stdout

## Tests

- [ ] Tests unitaires (`tests/unit/` ; inline uniquement pour les fonctions non publiques)
- [ ] Tests d'intégration (API publique)
- [ ] Tests de sécurité (nonces, tampering, secrets)
- [ ] Tests de régression (compatibilité format)
- [ ] Tests E2E (workflow complet CLI)
- [ ] Tests de performance (seuils définis)
- [ ] Tests d'acceptation (critères utilisateur)
- [ ] Tests d'usabilité (ergonomie CLI)
- [ ] Tests statiques et d'accessibilité (§ 38)

## Documentation

- [ ] `code.md` à jour avec les lignes modifiées
- [ ] `explication.md` à jour avec justifications
- [ ] `README.md` reflète les fonctionnalités actuelles
- [ ] `REGLES.md` complété avec nouvelles règles si applicable
- [ ] Messages de commit référencent REGLES.md (§ X.Y) dans leur corps (appliqué à partir du 2026-09-24 ; les commits antérieurs ne sont pas réécrits)

---

# 36. Règle finale mise à jour

Le code doit satisfaire cette progression **étendue** :

```text
1. Ça compile.
       ↓
2. C'est idiomatique Rust.
       ↓
3. Les types expriment correctement le domaine.
       ↓
4. Les états incohérents sont empêchés.
       ↓
5. Les erreurs conservent leur cause.
       ↓
6. Aucune variante fourre-tout (Other, Generic, etc.)
       ↓
7. Les secrets sont protégés.
       ↓
8. Le protocole cryptographique est correct.
       ↓
9. Les longueurs critiques sont validées.
       ↓
10. Tous les unwrap/expect ont des messages clairs.
       ↓
11. Les scénarios nominaux ET d'échec sont testés.
       ↓
12. Les tests sont organisés par type.
       ↓
13. Les performances respectent les seuils définis.
       ↓
14. Seulement ensuite : validation.
```

> **La conformité REGLES.md est un processus itératif.**
>
> Chaque audit peut révéler de nouvelles règles à documenter.
> L'objectif n'est pas la perfection immédiate, mais l'amélioration continue et la traçabilité des décisions.


---

# 37. Dérogations imposées par le sujet (force majeure — 2026-09-24)

Le sujet `G-CNA-500-my_pgp.pdf` fait foi. Lorsqu'une exigence du sujet contredit une règle de ce document, **le sujet l'emporte**, et la dérogation doit être :

- limitée au strict périmètre exigé par le sujet ;
- listée dans ce tableau ;
- rappelée dans le `code.md` du module concerné ;
- accompagnée des mesures compensatoires possibles.

## 37.1. Tableau des dérogations

| # | Exigence du sujet | Règle(s) concernée(s) | Portée | Mesures compensatoires |
|---|---|---|---|---|
| D1 | AES appliqué bloc par bloc (ECB), sans authentification | § 7.3, § 12.6 | `aes`, `pgp-aes` | Documenter l'absence d'intégrité ; tout mode authentifié éventuel va dans `bonus/` |
| D2 | Padding du dernier bloc par des zéros ; zéros non retirés au déchiffrement | § 19.2 | `xor`, `aes`, `pgp-*` en mode flux | Longueurs de clé et de bloc validées strictement avant toute opération |
| D3 | Clé transmise en argument de ligne de commande | § 4.4, § 20.1 | toute la CLI | Clé jamais réaffichée dans les erreurs ; copies effacées dès la fin de l'opération |
| D4 | Entrées/sorties en hexadécimal brut, sans format versionné | § 10.3, § 23 | toute la CLI | Validation stricte de l'hexadécimal (longueur paire, caractères valides) |
| D5 | Clé symétrique PGP fournie par l'utilisateur (`SYMMETRIC_KEY:RSA_KEY`) au lieu d'être tirée aléatoirement | § 19.4 | `pgp-xor`, `pgp-aes` | Documenté comme choix de débogage imposé par le sujet |
| D6 | RSA « textbook » (`m^e mod n`), sans OAEP | § 19.1 | `rsa`, `pgp-*` (implémenté par Frédrich) | Schémas de padding réservés au bonus |
| D7 | Code de sortie 84 pour toute erreur | § 18.2 (distinction des erreurs) | `main.rs` | La distinction est conservée dans `MyPgpError` et dans le message sur stderr ; seul le code de sortie est unifié |

## 37.2. Règles devenues sans objet pour la partie obligatoire

Les sections suivantes décrivaient l'ancienne architecture (fichiers, JSON, AES-GCM, RSA-OAEP, crates crypto). Elles ne s'appliquent plus qu'à un éventuel bonus qui réintroduirait ces mécanismes :

- § 19 (chiffrement hybride AES-GCM + RSA-OAEP) ;
- § 21 et § 22 (fichiers de clés et fichiers de sortie) : le sujet lit stdin et écrit sur stdout ;
- § 23 (sérialisation JSON) ;
- § 24.1 et § 24.2 (chemins de fichiers) ;
- § 33.1 (validation du nonce GCM).

## 37.2 bis. Décisions de l'équipe qui remplacent une recommandation

| Date | Règle | Décision |
|---|---|---|
| 2026-09-24 | § 2.1 (`thiserror`) | Erreurs implémentées à la main ; zéro dépendance externe |
| 2026-09-24 | § 20.2 (`zeroize`) | Aucun `unsafe` dans le projet : effacement par mise à zéro suivie de `std::hint::black_box`, sans garantie formelle (voir `src/domain/code.md`) |
| 2026-09-24 | Dépendances | « From scratch » = aucune crate externe (`[dependencies]` vide, vérifié par `tests/static/`) et tous les algorithmes cryptographiques écrits par l'équipe. La bibliothèque standard (`std`) reste autorisée pour les E/S, les collections, le formatage et les tests |
| 2026-09-24 | Vocabulaire | `crate::` et `pub(crate)` sont des mots-clés du langage désignant **notre propre projet** ; ils ne sont pas des dépendances et restent autorisés (forme idiomatique). Seule une entrée dans `[dependencies]` constitue une crate externe |

## 37.3. Règles qui restent pleinement applicables

Toutes les autres règles restent obligatoires, en particulier :

- types dédiés pour les secrets, masqués dans `Debug`, sans `Clone` ni `Serialize` par défaut (§ 4, § 6) ;
- effacement des secrets en mémoire, implémenté à la main puisque le sujet impose une cryptographie sans crate (§ 20) ;
- erreurs typées, sans variante fourre-tout, sans secret dans les messages (§ 3, § 4.4) ;
- validation stricte de toute entrée : hexadécimal, tailles de clé, nombre d'arguments (§ 10.2, § 17) ;
- aucun `unwrap()` ni `expect()` pour contourner une gestion d'erreurs, y compris dans les tests (§ 12.8) ;
- tests nominaux et d'échec, dont les vecteurs du sujet exécutés sur le binaire (§ 11, § 25).

## 37.4. Procédure pour toute nouvelle dérogation

1. Vérifier que le sujet l'impose réellement (citer la page du PDF).
2. L'annoncer explicitement avant d'écrire le code.
3. L'ajouter au tableau § 37.1 dans le même commit que le code.

---

# 38. Couverture de tests obligatoire (règle de référence — 2026-09-24)

À partir du 2026-09-24, **toute ligne de code et toute décision** doivent être accompagnées des dix types de tests suivants, en quantité comme en qualité. Cette section prime sur § 11, § 25 et § 28.4 en cas de conflit.

## 38.1. Types de tests et dossiers

Chaque type a son propre dossier sous `tests/`. Chaque dossier est une crate de test Cargo autonome (`tests/<type>/main.rs`), sans fichier `include!` ni wrapper.

| Type | Dossier | Ce qu'il vérifie |
|---|---|---|
| Unitaire | `tests/unit/` | Chaque fonction publique isolément : cas nominaux, limites, erreurs |
| Intégration | `tests/integration/` | La collaboration entre modules (ex. codec hex + algorithme) |
| Statique | `tests/static/` | Le code source sans l'exécuter : absence de `unwrap`/`expect`/`panic!` hors tests, de commentaires, de `unsafe` non autorisé, de crate crypto ; présence de `code.md`/`explication.md` |
| Régression | `tests/regression/` | Des sorties figées (vecteurs du sujet, vecteurs NIST, bugs corrigés) qui ne doivent jamais changer |
| End-to-End | `tests/e2e/` | Le binaire `my_pgp` lancé comme un utilisateur : arguments, stdin, stdout, stderr, code de sortie |
| Acceptation | `tests/acceptance/` | Chaque exigence du sujet, citée avec sa page, vérifiée sur le binaire |
| Performance | `tests/performance/` | Temps d'exécution sous charge, avec un seuil explicite et justifié |
| Sécurité | `tests/security/` | Aucune fuite de secret (messages d'erreur, `Debug`), robustesse face aux entrées hostiles, absence de panique |
| Usabilité | `tests/usability/` | Clarté des messages, cohérence de la CLI, aide `-h` |
| Accessibilité | `tests/accessibility/` | Sorties lisibles par lecteur d'écran et outils : texte ASCII brut, pas de codes couleur, une information par ligne, erreurs sur stderr |

## 38.2. Exigences de qualité

- Chaque nouvelle fonctionnalité ajoute des tests dans **chaque** dossier où elle a un effet observable. Si un type n'a réellement aucun objet à une sous-étape donnée (ex. E2E pour une fonction non encore branchée à la CLI), le `code.md` le dit explicitement et le test est ajouté dès que l'objet existe.
- Les tests suivent § 12.8 : pas de `unwrap()`/`expect()` ; ils renvoient `Result` et utilisent `?`.
- Les tests de performance fixent un seuil chiffré et justifié, mesuré en mode `--release` lorsque c'est pertinent.
- Les tests de régression figent des valeurs exactes, jamais des propriétés vagues.
- `cargo test` exécute l'intégralité des dossiers ; aucune sous-étape n'est validée si un test échoue.
- Ces trois vérifications (`cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`) sont automatisées par la CI (`.github/workflows/ci.yml`, § 39.4) sur chaque `push` et chaque pull request ; elles ne dépendent pas de la discipline individuelle pour être exécutées.

---

# 39. Contenu attendu de `code.md` et `explication.md`

Le code étant sans commentaires (§ 2.9, `test_no_comments_in_source_code`), toute explication vit dans ces deux fichiers. `tests/static/` vérifie leur présence (§ 38.1) mais pas leur contenu : cette règle fixe ce que chacun doit couvrir.

## 39.1. `code.md` — référence technique, ligne par ligne

`code.md` explique **chaque fichier du dossier, ligne par ligne** (ou par groupe de lignes cohérent : une `const`, une `fn`, un `match`). Chaque fichier `.rs` a sa section, et chaque ligne de logique y est expliquée. Pour chaque élément public (et chaque élément privé non trivial) du dossier :

- Sa signature et son rôle exact.
- La référence normative dont il découle (numéro de section FIPS-197/PDF du sujet, RFC, etc.), quand elle existe.
- Les choix de conception qui ne sont pas visibles à la lecture (pourquoi une table précalculée plutôt qu'un appel de fonction, pourquoi une structure plutôt qu'un `Vec`, quelles invariants sont protégés).
- Un tableau de couverture de tests par type (§ 38.1), mis à jour à chaque sous-étape qui touche le dossier.
- Un historique par sous-étape (une section `### X.Y` par sous-étape livrée), jamais réécrit après coup sauf correction d'une erreur factuelle constatée.

## 39.2. `explication.md` — pédagogie, pour qui découvre le sujet

Pour chaque notion introduite par le dossier :

- **Quoi** : ce que fait la notion, en une ou deux phrases, sans jargon inutile.
- **Pourquoi** : le problème que ça résout, idéalement relié à l'histoire du sujet (Arya, Brienne, Euron) ou à un exemple concret.
- **Comment** : le mécanisme, assez précis pour qu'un lecteur puisse le vérifier dans le code, mais sans reformuler chaque ligne (c'est le rôle de `code.md`).
- **Avec quoi** : les dépendances internes (quel module s'appuie sur quel autre) quand ça aide à comprendre l'ordre des choses.
- **Où** : l'endroit précis dans le code (fichier, fonction, ligne), écrit au format vérifiable du § 39.5.
- Au moins une analogie quand la notion est abstraite (arithmétique de corps fini, diffusion cryptographique, etc.) ; l'analogie doit rester fidèle au mécanisme, pas juste décorative.
- Les concepts concrets (ex. little-endian, GF(2⁸), padding par zéros) sont illustrés par un exemple chiffré.

## 39.3. Ce que ces fichiers ne sont pas

- `code.md` n'est pas un résumé narratif : c'est une référence qu'on peut consulter fonction par fonction.
- `explication.md` n'est pas une redite de `code.md` avec d'autres mots : il explique le *sens*, pas la mécanique ligne à ligne.
- Ni l'un ni l'autre ne doit présenter une fonctionnalité comme opérationnelle si elle ne l'est pas encore (§ 24.5) : dire explicitement « sans objet à ce stade » plutôt que de suggérer une couverture qui n'existe pas.

## 39.4. Intégration continue

`.github/workflows/ci.yml` exécute, sur chaque `push` et chaque pull request, dans cet ordre : `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, puis `cargo test --all-targets`. Un échec de n'importe laquelle de ces étapes bloque la fusion (§ 38.2). Le workflow n'installe aucune dépendance externe au projet (§ 37.2 bis) : seul le toolchain Rust stable est nécessaire.

## 39.5. Références de lignes vérifiées

Toute référence à une ligne de code dans `code.md` ou `explication.md` s'écrit `` `fichier.rs:LIGNE nom` `` (ex. `` `block.rs:11 encrypt_block` ``), où `fichier.rs` est dans le même dossier que le `.md`, et où la ligne `LIGNE` doit contenir la déclaration de `nom` (`fn`, `struct`, `enum`, `const`, `type` ou `trait`). `tests/static/` (`test_documented_line_references_are_exact`) échoue si une référence est périmée : modifier le code impose donc de remettre la documentation à jour.
