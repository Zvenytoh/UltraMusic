# UltraMusic — revue complète du 22 octobre 2026

**Historique 0.4.2 :** les corrections de 0.4.3 sont suivies dans [REVIEW-CORRECTIONS-2026-10-22.md](REVIEW-CORRECTIONS-2026-10-22.md). Les constats ci-dessous décrivent la version initialement revue.

**Version et périmètre :** 0.4.2, commit `6c7facce1979142a7116c31a19e8d7a2eeb2b703`. Revue du code du lecteur, de la navigation, des contrôles audio, du bloqueur, de la mise à jour, de l’installeur et de la chaîne de publication. Charge attendue : une application personnelle, un utilisateur Windows et un seul lecteur WebView2 actif.

**Verdict : base technique saine, mais niveau de qualité élevé non encore démontré. Corriger R1 à R4 en priorité.** Les contrôles de compilation et de dépendances passent ; ils ne couvrent pas les défauts de comportement décrits ci-dessous. Cette revue ne constitue ni une certification de sécurité ni une validation visuelle de l’application.

## Méthode et preuves

Compétence utilisée : Ponytail Review, avec lecture des implémentations et de leurs appelants, et confrontation des points sensibles à la documentation Microsoft. Les compétences locales ont été recherchées : aucun autre skill spécialisé Rust/sécurité/accessibilité n’était nécessaire à installer pour cette revue. Aucun graphe Graphify existant n’est présent. Aucun sous-agent utilisé, conformément aux instructions du projet.

Contrôles réellement exécutés sur cette version :

| Contrôle | Résultat et portée |
| --- | --- |
| `scripts/dev.ps1 fmt` | Réussi : formatage Rust |
| `scripts/dev.ps1 clippy` | Réussi, `--locked --all-targets -- -D warnings` ; compilation des cibles, sans exécution de tests |
| `scripts/dev.ps1 audit` | Réussi : 78 dépendances Cargo, base RustSec contenant 1 349 avis, aucune vulnérabilité signalée |
| `scripts/dev.ps1 build` | Réussi : release Windows GNU locale ; archive uBO Lite vérifiée par SHA-256 |
| Gitleaks `git --all --redact` | 21 commits, environ 1,54 Mo analysés ; aucun secret signalé |
| Analyse syntaxique PowerShell | Les cinq scripts sont valides ; `smoke.ps1` a été analysé, pas exécuté |
| GitHub Actions `39819599012` | Statut final `success` : audit, formatage, Clippy, compilation MSVC, packaging et publication |

Pas de tests unitaires/fonctionnels, de lancement du lecteur ou de l’installeur. Pas de mesures CPU/RAM/latence, de lecteur d’écran ni de manipulation du DPI. L’absence de secret ou de vulnérabilité signalée par les outils ne prouve pas leur absence générale. RustSec ne couvre pas le moteur WebView2, Inno Setup ou le code de l’extension.

## À corriger en priorité

### R1 — P1 : une désactivation du bloqueur peut échouer sans empêcher la navigation

**Code :** `src/native/blocker.rs:164-193` et `src/native/blocker.rs:197-216` ; profil réutilisé dans `src/native/mod.rs` et fermeture dans `src/native/webview.rs`.

**Fonctionnement :** uBO Lite appartient au profil WebView2 partagé. Avant navigation, le lecteur appelle `Enable(false)` pour Apple Music ou lorsque le bloqueur est désactivé.

**Problème :** si cet appel échoue alors que l’extension était activée, le code appelle `finish_blocker(Unavailable, ...)`, qui navigue ou recharge quand même. L’état activé est persistant dans le profil et la fermeture du contrôleur ne le désactive pas. On ne peut donc plus garantir la promesse « désactivé sur Apple Music » ni le respect d’une désactivation explicite. Le paquet possède `<all_urls>` ; cela ne démontre aucune collecte de données, mais rend cette frontière importante. La persistance et la portée par profil sont confirmées par [Microsoft, ICoreWebView2BrowserExtension](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2browserextension?view=webview2-1.0.4022.49).

**Correction minimale :** distinguer l’échec d’un chargement/une activation de l’échec d’une désactivation. Quand la désactivation requise n’est pas confirmée, ne pas naviguer/recharger comme si le choix était appliqué ; afficher une erreur exploitable et conserver un état explicite. La navigation sans bloqueur reste possible après vérification effective de son état. Ne pas créer silencieusement un nouveau profil qui ferait perdre les sessions.

**Si on laisse :** une préférence de désactivation peut ne pas être respectée, notamment au passage vers Apple Music. Scénario déduit du code et du contrat de l’API ; non reproduit à l’exécution.

### R2 — P2 : le déplacement du volume effectue du travail disque et audio sur le thread UI

**Code :** `src/native/settings.rs:355-361`, `src/native/media.rs:168-203`, `src/native/media.rs:219-253`.

**Fonctionnement :** chaque notification du curseur appelle `set_volume`, qui mémorise le réglage et recherche les sessions WASAPI appartenant au navigateur.

**Problème :** les notifications de déplacement, fin de déplacement et relâchement sont traitées de la même façon. Chaque événement écrit puis renomme le fichier de préférence, prend un instantané des processus et énumère les périphériques et sessions audio. Tout cela s’effectue synchroniquement dans la procédure de fenêtre ; même un événement qui ne change pas la valeur refait le travail. [Microsoft documente ces notifications distinctes](https://learn.microsoft.com/en-us/windows/win32/controls/wm-hscroll--trackbar-).

**Correction minimale :** ignorer les valeurs identiques, séparer l’application du volume de sa sauvegarde, et sauvegarder à la fin du geste. Réutiliser les sessions audio valides pendant un geste et les rechercher à nouveau lorsqu’elles deviennent indisponibles, sans ajouter de polling permanent.

**Si on laisse :** l’interface dépend de la vitesse du disque, de l’antivirus et de l’énumération audio pendant un geste fréquent. Le risque de saccades est étayé par les appels ; leur durée n’a pas été mesurée.

### R3 — P2 : les paramètres ne s’adaptent pas à une petite zone de travail

**Code :** `src/native/settings.rs:157-185`, `src/native/settings.rs:255-288`, tableau `ROWS`.

**Fonctionnement :** la fenêtre a un contenu fixe de 456 × 472 pixels logiques, centré sur le lecteur. Ses contrôles gardent des positions fixes lors des changements de DPI.

**Problème :** ni la zone de travail du moniteur ni la place requise par les textes ne pilotent la disposition. À 200 % sur un écran de 768 pixels de haut, le contenu seul mesure 944 pixels physiques, avant la légende et la barre des tâches. Le bouton de mise à jour situé à y=412 devient inaccessible. La fenêtre n’a ni défilement ni possibilité de redimensionnement. L’agrandissement des textes est un risque supplémentaire : certaines zones de texte ont seulement 24 ou 32 pixels logiques de hauteur.

**Correction minimale :** borner la fenêtre à la zone de travail du moniteur et proposer un contenu défilant lorsque la hauteur nécessaire dépasse cette zone. Mesurer les explications avec la police réellement appliquée et recalculer les positions au changement de DPI/police.

**Si on laisse :** des réglages deviennent hors écran dans une configuration Windows prise en charge. Ce calcul est vérifiable dans la géométrie du code ; aucun écran n’a été manipulé pendant la revue.

### R4 — P2 : certains échecs précoces du helper ferment le lecteur sans le relancer

**Code :** `src/native/updates.rs:94-108`, `src/update/install.rs:44-48`, `src/update/install.rs:96-104`.

**Fonctionnement :** une fois le helper créé, la fenêtre du lecteur est détruite. Le helper revérifie ensuite l’archive, attend le parent et prépare la sauvegarde avant de lancer Setup.

**Problème :** ces étapes précoces retournent directement une erreur. Exemple concret : il reste assez de place pour le téléchargement et la copie du helper, mais la copie de sauvegarde échoue faute d’espace. Le lecteur est déjà fermé ; la branche de restauration/relance située plus bas n’est jamais exécutée. Le fichier installé reste intact, mais l’utilisateur doit le rouvrir manuellement.

**Correction minimale :** préparer la sauvegarde et les prérequis avant la fermeture, ou ajouter un accusé de préparation entre helper et lecteur. Après fermeture du parent, centraliser les sorties d’erreur pour relancer l’exécutable intact lorsque cela est sûr.

**Si on laisse :** une mise à jour annulée avant même Setup coupe la lecture et ne rend pas automatiquement la main au lecteur.

## Corrections à programmer ensuite

### R5 — P2 : aucune barrière CI ne compare le tag et la version du binaire

**Code :** `.github/workflows/windows.yml:41-79`, `src/update/mod.rs:62-88`.

**Fonctionnement :** la version embarquée et celle de l’installeur viennent de Cargo.toml ; la release est nommée à partir du tag. L’updater se fie à la version du tag.

**Problème :** un tag accidentel `v0.5.0` sur un code resté en `0.4.2` produit une release éligible contenant le même ancien programme. Après installation, le lecteur reste en 0.4.2 et propose de nouveau cette mise à jour. La documentation demande une correspondance, mais le workflow ne la contrôle pas. Le tag actuel v0.4.2 correspond correctement au code : il s’agit d’un défaut de protection pour les prochaines publications.

**Correction minimale :** sur un événement de tag, comparer strictement `GITHUB_REF_NAME` à `v` suivi de la version du package avant de construire/publier. Refuser toute discordance.

**Si on laisse :** une simple erreur de publication peut créer une boucle de propositions de mise à jour.

### R6 — P2 : les fichiers des mises à jour s’accumulent sans politique de rétention

**Code :** `src/update/mod.rs:102-109`, `src/update/install.rs:98-104`, `src/update/install.rs:154`.

**Fonctionnement :** chaque tentative utilise un nouveau sous-dossier Updates. Le succès supprime le Setup téléchargé, mais conserve helper, sauvegarde et journal.

**Problème :** aucune routine ne supprime les anciens dossiers. Une fermeture de l’application pendant le téléchargement laisse également le fichier partiel, puisque la boucle ne peut pas exécuter son nettoyage normal après arrêt du processus. Les tentatives répétées peuvent ainsi conserver des téléchargements approchant la limite de 128 Mio par tentative.

**Correction minimale :** nettoyer au démarrage d’une nouvelle opération les dossiers anciens dont aucun helper n’est actif ; conserver par exemple la dernière sauvegarde et les derniers journaux utiles. Identifier explicitement l’opération active avant toute suppression.

**Si on laisse :** le stockage local croît au fil des mises à jour réussies et interrompues. Aucun volume réel occupé n’a été attribué à l’application pendant cette revue.

### R7 — P2 : le statut de téléchargement reste présenté comme une vérification

**Code :** `src/native/updates.rs:31-36`, `src/native/updates.rs:76-80`, `src/native/settings.rs:325-332`, `src/native/tray.rs:85-90`.

**Fonctionnement :** `update_busy` représente à la fois la recherche d’une version et le téléchargement d’un installeur. Le même texte propose d’afficher un résultat de vérification.

**Problème :** après confirmation, un téléchargement lent n’a pas de statut distinct dans les paramètres. Recliquer ne montre ni son étape ni sa progression ; cela ne fait que positionner `update_manual`. L’installation possède une progression visible, mais seulement après la fin du téléchargement.

**Correction minimale :** distinguer les états Idle, Checking et Downloading ; afficher au minimum « Téléchargement de la mise à jour… ». Une progression et une annulation seraient ensuite utiles, mais ne sont pas nécessaires au premier correctif du libellé trompeur.

**Si on laisse :** l’utilisateur ne sait pas si sa confirmation a été prise en compte ou si le bouton répond, particulièrement sur une connexion lente.

### R8 — P2 : les erreurs de sauvegarde des réglages audio sont silencieuses

**Code :** `src/native/media.rs:53-60`, `src/native/media.rs:158-172`, `src/native/media.rs:207-213`.

**Fonctionnement :** les commandes changent le volume/mute en mémoire puis tentent d’écrire la préférence.

**Problème :** un échec de création, d’écriture ou de renommage est ignoré. En cas de disque plein ou de fichier en lecture seule, l’interface donne l’impression que le choix est mémorisé ; le prochain lancement restaure pourtant l’ancien choix. La sélection de service et le bloqueur signalent déjà les erreurs de persistance, ce qui rend le comportement audio incohérent.

**Correction minimale :** faire remonter le résultat de sauvegarde et signaler une seule fois, à la fin de l’interaction, que le réglage actuel n’a pas pu être mémorisé. Conserver le réglage audio en mémoire ; ne pas multiplier les boîtes de dialogue pendant le déplacement du curseur.

**Si on laisse :** des préférences promises comme persistantes sont perdues sans explication. À traiter avec R2 pour partager la bonne frontière de sauvegarde.

## Évaluation du niveau de qualité

| Domaine | Appréciation fondée sur cette revue |
| --- | --- |
| Organisation Rust | Responsabilités correctement séparées, visibilité limitée, Cell/RefCell cohérents avec le thread STA. Pas de framework ni de dépendance ajoutée pour la dernière UI. Pas de refonte générale justifiée. |
| Durée de vie native | Propriétaires RAII pour COM/mutex/polices/images, restauration des sélections GDI, callbacks vérifiant la génération. Une compilation n’établit cependant pas à elle seule la sûreté de tout le code unsafe. |
| Navigation et permissions | Liste d’hôtes HTTPS explicite, protocole/port/identifiants contrôlés, pas d’objet hôte ou de WebMessage distant, téléchargements de page annulés. Les ressources et iframes sont déléguées à WebView2 : ce n’est pas un pare-feu réseau global. |
| Mises à jour | HTTPS vérifié, hôtes de redirection limités, réponses bornées, SHA-256 et taille vérifiés. Préparation/retour après erreur et rétention à renforcer. |
| Performances | Boucle événementielle sans polling permanent côté hôte. R2 est la première optimisation justifiée par un chemin d’exécution fréquent ; aucune conclusion CPU/RAM globale sans mesure. |
| UI/UX | Contrôles natifs, raccourcis et noms textuels constituent une bonne base. R3 et R7 empêchent de conclure à une UX robuste. Apparence, clavier réel et lecteurs d’écran non validés. |
| Livraison | Cargo.lock versionné, actions épinglées par SHA, permissions CI limitées, installeur par utilisateur. Ajouter R5 ; audit RustSec et scan de secrets sont des contrôles utiles, pas une attestation de sécurité complète. |

Points explicitement non certifiés : accessibilité UI Automation/Narrator, contraste dans chaque thème, agrandissement du texte Windows, parcours de connexion Apple/Google, audio/DRM, efficacité publicitaire et performance réelle. La [checklist Microsoft d’accessibilité](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-checklist) requiert notamment des vérifications au clavier, au lecteur d’écran et en contraste élevé ; cette revue statique ne les remplace pas. Voir aussi les [exigences Microsoft sur les textes accessibles](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessible-text-requirements).

## Limites connues, à ne pas confondre avec une nouvelle faille démontrée

- Le rollback sauvegarde l’exécutable, pas l’ensemble des ressources/du registre installé. Cette limite est déjà documentée ; ce n’est pas une transaction complète de mise à jour.
- Le digest provenant de GitHub assure la cohérence avec l’asset publié par le dépôt. Il ne constitue pas une signature indépendante du mainteneur et ne protège pas contre une compromission de son compte de publication.
- L’extension officielle est distribuée avec ses notices et son empreinte ; son code complet et toutes ses règles n’ont pas fait l’objet d’un audit exhaustif ici.
- L’absence de tests exécutés respecte l’instruction actuelle du projet. Elle laisse les régressions comportementales et visuelles non couvertes : on ne peut donc pas annoncer une conformité élevée ou un résultat « parfait ».

## Ordre de correction proposé

1. R1 : respecter la désactivation même en cas d’échec du module.
2. R4 et R5 : rendre la préparation et la publication des mises à jour cohérentes.
3. R2 et R8 : alléger le chemin du volume et rendre sa sauvegarde explicite.
4. R3 et R7 : rendre les paramètres accessibles à petite hauteur et les opérations compréhensibles.
5. R6 : ajouter une rétention bornée des fichiers de mise à jour.

La revue ne modifie aucun code de production et ne publie aucune nouvelle release. Les correctifs et leur validation constituent une intervention distincte.
