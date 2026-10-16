# Validation et mesures

## Tests fonctionnels avant de déclarer le MVP terminé

Pour chaque service, consigner la date, Windows, CPU/GPU, pilote, version du
runtime, commit, type d’abonnement et mode de rendu, sans identifiant utilisateur.

| Test | Procédure | Résultat attendu |
|---|---|---|
| Connexion | Se connecter puis quitter/relancer | Session conservée |
| Audio | Lire une playlist pendant 30 minutes | Aucun défaut audible |
| Arrière-plan | Fermer avec la croix pendant la lecture, attendre 30 min | Pistes suivantes et audio continus |
| Restauration | Double-cliquer l’icône | Même page, aucun rechargement |
| Pause | Masquer en pause puis restaurer | Session intacte, reprise possible |
| Multimédia | Play/pause, suivant, précédent visibles et masqués | Action correcte, y compris avec un autre lecteur ouvert |
| Instance unique | Relancer visible/masqué | Un seul wrapper, restauration |
| Quitter | Choisir Quitter dans l’icône | Wrapper et ses descendants terminés après le délai du runtime |
| Reprise | Veille/reprise, réseau coupé/rétabli | Interface récupérable sans perdre le profil |
| Erreur moteur | En profil de test, arrêter un renderer du lecteur | Message puis redémarrage manuel fonctionnel |
| DPI | Déplacer entre deux écrans à échelles différentes | WebView ajustée |
| Liens | Lien externe, protocole non HTTPS, popup de connexion | Confirmation HTTPS, autres protocoles bloqués |

Répéter arrière-plan/restauration avec `--keep-rendering`. Tester un démarrage
sans runtime dans une VM et une fermeture pendant l’initialisation.

## Benchmark reproductible

Comparer UltraMusic standard, UltraMusic compatibilité, Edge PWA, un wrapper
Tauri et un client Electron identifiés par version/commit, sur le même service,
compte, playlist, qualité, volume, réseau, plan d’alimentation et écran. Fermer
les autres lecteurs ; attendre 2 minutes de stabilisation après chargement.
Utiliser les builds release. Ne pas comparer des sites ou contenus différents.

Mesurer les cinq scénarios : visible/pause, visible/lecture, masqué/pause,
masqué/lecture, puis restauration. Pour les quatre états stables, faire trois
répétitions de 5 minutes, dans un ordre alterné entre clients. Pour la restauration,
mesurer au moins dix essais après 5 minutes masqué et relever médiane/p95 du délai
entre l’action et la première image utilisable. Noter chaque coupure audible.

Utiliser Windows Performance Recorder/Analyzer (Windows Performance Toolkit)
avec événements CPU, changements de contexte, GPU/DWM et processus. Conserver
les traces ETL localement : elles peuvent contenir des informations privées.
Identifier l’arbre complet du wrapper, y compris les processus créés/terminés
pendant la capture. Pour Edge PWA, isoler son profil et inclure tous ses processus
pertinents ; ne pas compter uniquement la fenêtre ou le processus principal.

Exporter : temps CPU cumulé (secondes), changements de contexte et réveils,
activité GPU par moteur (sans additionner arbitrairement les pourcentages),
présentations/rendu, mémoire privée cumulée et pic, durée de restauration,
défauts audio. Le working set cumulé double-compte les pages partagées : le
présenter séparément, jamais comme de la mémoire physique unique.
Une somme des compteurs CPU à deux instants manque les processus terminés ;
utiliser la trace ETW pour les résultats publiables.

Mesurer la puissance seulement avec un outil/capteur documenté ; sinon écrire
« non mesuré ». Le coût de l’outil de mesure doit être identique pour tous les
clients. Publier valeurs brutes, médianes, dispersion, versions et conditions.

| Client/version | État | Répétition | Durée | CPU s | Context switches | GPU moteurs | Présentations | Mémoire privée | Audio |
|---|---|---|---|---|---|---|---|---|---|
| À mesurer | | | | | | | | | |

À ce stade aucune mesure comparative n’est disponible. Aucune économie de
ressources et aucune compatibilité audio/DRM ne sont annoncées comme acquises.
