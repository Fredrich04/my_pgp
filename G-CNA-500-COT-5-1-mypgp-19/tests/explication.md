# `tests` — explication

Chaque dossier regarde le programme sous un angle différent : une fonction seule (unitaire), plusieurs morceaux ensemble (intégration), le code sans l'exécuter (statique), des résultats qui ne doivent jamais changer (régression), le programme comme un utilisateur le lance (end-to-end), les exigences du sujet (acceptation), la vitesse (performance), les attaques et fuites (sécurité), la clarté des messages (usabilité) et la lisibilité par les outils d'assistance comme les lecteurs d'écran (accessibilité).

Avoir un dossier par angle permet de voir d'un coup d'œil ce qui est couvert, et de lancer un seul type : `cargo test --test security`.
