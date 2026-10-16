# UltraMusic — Lightweight Background Music Player

**Type :** application desktop Windows, open source

**Langage principal :** Rust

**Moteur :** Microsoft Edge WebView2

**Plateforme initiale :** Windows 11 x64

**Priorité absolue :** consommation CPU et GPU minimale pendant la lecture musicale en arrière-plan.

## Présentation du projet

UltraMusic est une application desktop minimaliste permettant d'utiliser Apple Music ou YouTube Music sans navigateur classique ni framework lourd.

L'application encapsule directement l'interface web officielle du service musical dans une WebView native Windows.

Sa particularité est son mode de fonctionnement en arrière-plan : lorsque l'utilisateur ferme la fenêtre, l'application reste active dans la zone de notification Windows. Le lecteur audio continue de fonctionner tandis que le rendu graphique est désactivé autant que la compatibilité du service le permet.

Le projet vise à éliminer les calculs, animations, rafraîchissements graphiques et traitements applicatifs inutiles, notamment lorsqu'un utilisateur écoute de la musique en jouant à un jeu vidéo.

## 1. Objectifs principaux

- Minimiser l'activité CPU en arrière-plan.
- Minimiser l'activité GPU pendant la lecture audio.
- Éviter les opérations de rendu lorsque la fenêtre est masquée.
- Maintenir une lecture audio continue, sans coupure.
- Réutiliser WebView2 au lieu d'embarquer Chromium.
- Éviter Electron et les frameworks frontend supplémentaires.
- Démarrer rapidement et conserver la connexion utilisateur.
- Fournir une application simple, fiable et maintenable.

L'application ne cherche pas à réduire artificiellement la qualité audio ni à modifier les flux protégés des plateformes.

## 2. Fonctionnement utilisateur

**Mode fenêtre ouverte**

Au lancement, UltraMusic affiche directement le site musical choisi. L'utilisateur retrouve ses playlists, recommandations, albums et commandes habituelles.

**Mode arrière-plan**

Un clic sur la croix ferme visuellement la fenêtre mais ne termine pas le processus.

L'application disparaît de la barre des tâches, reste disponible depuis la zone de notification et poursuit la lecture audio.

La WebView est conservée en mémoire, mais son rendu est désactivé si cela n'interrompt pas la lecture.

**Réouverture**

Un double-clic sur l'icône de notification restaure immédiatement la fenêtre et l'interface actuelle, sans recharger le site.

**Arrêt complet**

Une commande « Quitter » ferme la lecture, libère la WebView et termine les processus associés.

## 3. États de fonctionnement

| État | Comportement |
|---|---|
| Visible | Interface et audio fonctionnels |
| Masqué + lecture | Audio conservé, rendu visuel désactivé si compatible |
| Masqué + pause | Réduction du travail en arrière-plan, sans interrompre la session |
| Inactif prolongé | Suspension facultative, uniquement lorsque la lecture est arrêtée et que cela reste sûr |
| Quitter | Destruction de la WebView et fermeture complète |

La suspension de la page n'est jamais déclenchée automatiquement pendant une lecture active.

## 4. Architecture technique

**Rust + Win32 + WebView2 natif**

Rust assure la gestion de l'application et des événements système.

Win32 gère la fenêtre, la boucle de messages, la zone de notification et les commandes utilisateur.

WebView2 est responsable du chargement des services musicaux, de leur JavaScript et de la lecture multimédia.

Aucune interface React, Svelte ou HTML personnalisée n'est nécessaire pour le lecteur principal.

L'application repose sur un unique profil WebView2 persistant et une seule WebView active.

La boucle Win32 est événementielle et ne doit pas effectuer de rafraîchissements périodiques lorsqu'aucun travail n'est requis.

## 5. Optimisations de performance

### Rendu et GPU

- Masquer la fenêtre native lors du passage en arrière-plan.
- Tester `IsVisible = FALSE` pour supprimer le rendu de la WebView.
- Éviter les overlays, visualiseurs et effets graphiques supplémentaires.
- Désactiver certaines animations CSS non essentielles uniquement si les mesures montrent un bénéfice.
- Conserver l'accélération matérielle par défaut, sauf preuve expérimentale qu'un autre réglage est préférable.

### CPU

- Aucune boucle active côté Rust.
- Aucun polling continu pour récupérer le morceau en cours.
- Privilégier les événements système et multimédias.
- Aucun chargement réseau supplémentaire par le wrapper.
- Ne pas injecter de scripts JavaScript permanents sans nécessité.
- Ne pas imposer de fréquence de rafraîchissement ou de réveils CPU inutiles.

### Lecture audio

- Conserver le mécanisme de lecture officiel du service.
- Préserver les commandes lecture, pause, suivant et précédent.
- Maintenir le fonctionnement lorsque la fenêtre est masquée.
- Éviter les interruptions au changement de morceau.
- Ne pas contourner les mécanismes DRM ou d'authentification.

## 6. Fonctionnalités essentielles (MVP)

- Fenêtre native contenant le service musical.
- Fermeture vers la zone de notification.
- Réouverture sans rechargement.
- Menu Quitter.
- Session utilisateur persistante.
- Instance unique de l'application.
- Lecture continue en arrière-plan.
- Gestion des erreurs et redémarrage propre de WebView2.
- Journalisation minimale, désactivable et sans informations sensibles.

## 7. Fonctionnalités secondaires

Elles seront ajoutées uniquement si leur coût CPU/GPU est négligeable :

- Touches multimédias Windows.
- Intégration Windows SMTC.
- Commandes de lecture dans le menu de notification.
- Démarrage facultatif avec Windows.
- Restauration de la dernière position de la fenêtre.
- Paramètre « Fermer vers la zone de notification ».
- Sélection entre Apple Music et YouTube Music au lancement, dans une version ultérieure.

Pas de système de plugins, de Discord Rich Presence permanent ou de lecteur miniature animé dans la première version.

## 8. Mesure et validation des performances

L'efficacité du projet doit être démontrée par des benchmarks reproductibles.

**Scénarios de test :**

1. Application visible, musique en pause.
2. Application visible, musique en lecture.
3. Application masquée, musique en pause.
4. Application masquée, musique en lecture.
5. Retour à la fenêtre après une période en arrière-plan.

Chaque scénario sera mesuré pendant plusieurs minutes, avec plusieurs répétitions, dans des conditions comparables.

**Indicateurs :**

- Temps CPU cumulé de tous les processus.
- Réveils CPU et changements de contexte.
- Activité des moteurs GPU.
- Opérations de rendu et de présentation.
- Consommation électrique lorsque mesurable.
- Mémoire totale des processus.
- Coupures ou anomalies audio.
- Temps nécessaire pour restaurer l'interface.

**Comparaison de référence :**

UltraMusic devra être comparé à Edge PWA, à un wrapper Tauri et, lorsque pertinent, à un client Electron utilisant le même service.

Le succès sera une baisse mesurable de l'activité CPU/GPU sans perte de fonctionnalité essentielle ni dégradation de l'audio.

Aucun chiffre de performance ne sera annoncé avant les mesures.

## 9. Sécurité et confidentialité

- Authentification directement auprès des plateformes officielles.
- Aucun mot de passe collecté ou stocké par le code applicatif.
- Cookies conservés dans le profil WebView2 local.
- Aucune télémétrie personnalisée.
- Aucune exposition inutile d'API Rust à une page distante.
- Ouverture contrôlée des liens externes.
- Mises à jour de sécurité du moteur WebView2 via son runtime.
- Suppression des données de session possible.

Le projet reste indépendant d'Apple, Google et YouTube.

## 10. Contraintes techniques

La principale contrainte est que le site musical reste une application web complète.

Même masqué, il peut continuer à effectuer des traitements JavaScript, du décodage audio, du réseau et de la gestion des médias.

Un mode totalement inactif avec lecture active est impossible : le traitement audio nécessite toujours certaines ressources.

Apple Music peut également imposer des contraintes de DRM et de compatibilité WebView2.

Enfin, les modifications CSS et les intégrations avec le DOM peuvent cesser de fonctionner lorsque les plateformes mettent à jour leurs sites.

## 11. Plan de développement

**Phase 1 — Faisabilité**

Créer la fenêtre native Rust, initialiser WebView2, afficher le service et vérifier l'authentification ainsi que la lecture.

**Phase 2 — Arrière-plan**

Implémenter la fermeture vers la zone de notification, la restauration et la lecture continue.

**Phase 3 — Optimisation**

Tester la visibilité WebView2, supprimer les traitements non essentiels et identifier les sources de réveils CPU et d'activité GPU.

**Phase 4 — Benchmark**

Comparer objectivement le wrapper natif, Wry/Tauri et Edge PWA.

**Phase 5 — Publication**

Créer un exécutable Windows portable, documenter la compilation, publier les mesures et automatiser les builds sur GitHub.

## 12. Critères de réussite

Le MVP sera considéré comme terminé lorsque :

- Le service musical est utilisable normalement.
- La croix masque la fenêtre sans interrompre la musique.
- Le rendu visuel peut être évité en arrière-plan lorsque le service le permet.
- Les commandes multimédias fonctionnent.
- La restauration ne nécessite pas de reconnexion.
- L'application ne réalise pas de polling ou de travail périodique inutile côté Rust.
- Les performances sont documentées et comparées à une PWA.
- Aucun arrêt audio ni crash n'est observé pendant les tests prolongés.

**Philosophie du projet :**

*Keep the audio. Stop the unnecessary work.*

UltraMusic doit faire uniquement ce qui est nécessaire pour écouter de la musique, et rien de plus.
