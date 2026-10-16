# UltraMusic

*Keep the audio. Stop the unnecessary work.*

Prototype Windows x64 en Rust, Win32 et WebView2. Une seule fenêtre, une seule
WebView et un profil persistant. Aucun Electron, frontend personnalisé, script
injecté, timer applicatif, polling musical ou télémétrie du wrapper.

## Utilisation

Le runtime [Microsoft Edge WebView2 Evergreen](https://developer.microsoft.com/microsoft-edge/webview2/)
doit être installé. Le moteur n’est pas embarqué dans la distribution.

```powershell
.\ultramusic.exe                   # YouTube Music
.\ultramusic.exe --apple           # Apple Music, compatibilité à valider
.\ultramusic.exe --keep-rendering  # repli si le site interrompt l’audio masqué
```

La croix et la minimisation masquent la fenêtre dans la zone de notification.
Double-cliquer sur l’icône pour restaurer la même WebView. Clic droit : ouvrir,
redémarrer le lecteur, basculer la compatibilité, quitter. Un second lancement
restaure la première instance ; il ne change pas son service. Pour changer de
service, quitter puis relancer avec l’option voulue.

La session est enregistrée dans `%LOCALAPPDATA%\UltraMusic\WebView2`. Pour
effacer les cookies et données, quitter UltraMusic, attendre la fermeture de
ses processus WebView2, puis supprimer **ce dossier uniquement**. Aucun mot de
passe n’est traité par Rust ; les sites et le runtime gèrent l’authentification.
Aucun fichier journal n’est produit. Les erreurs sont affichées localement.

## Compiler et vérifier

Configuration recommandée : Windows 11 x64, Rust stable MSVC et Visual Studio
Build Tools avec « Développement Desktop en C++ » et le SDK Windows.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
.\target\release\ultramusic.exe
```

Le workflow GitHub Actions construit un artefact Windows MSVC à chaque push/PR.
Il n’exécute pas de tests unitaires. Les sources sont destinées au dépôt
[Zvenytoh/UltraMusic](https://github.com/Zvenytoh/UltraMusic).
Pour l’outillage GNU local de ce dossier : `./scripts/dev.ps1 build`. Garder
`WebView2Loader.dll` à côté du binaire GNU. `.tools/` est exclu de Git ; le script
utilise l’installation Rust du système si cet outillage local est absent.

## Architecture et limites

`src/native/mod.rs` possède la fenêtre et les objets COM sur le thread STA, avec une
boucle bloquante `GetMessageW`. L’initialisation WebView2 est asynchrone. Un compteur
de génération ignore les callbacks obsolètes après fermeture/redémarrage. La
fermeture appelle `Close` puis libère les références ; aucun processus Edge tiers
n’est tué. L’icône est recréée après un redémarrage d’Explorer. Si elle ne peut pas
être créée, la fenêtre reste accessible.

Le code natif est compartimenté en modules `instance`, `webview`, `window` et
`tray`. Voir l’[architecture et ses invariants](docs/ARCHITECTURE.md) et le
[guide de contribution](CONTRIBUTING.md). Les messages différés portent aussi la
génération du lecteur. Un second lancement attend au plus cinq secondes si la
première fenêtre est encore en cours de création.

En arrière-plan, le contrôleur reçoit `IsVisible = false`, sauf en mode
compatibilité. Selon [Microsoft](https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2controller.isvisible),
cela peut réduire le travail de rendu et de la page. Ce n’est pas une garantie de
GPU nul. Le mode compatibilité garde `IsVisible = true`, mais le parent reste
masqué et le moteur peut encore détecter l’occlusion. Aucune suspension n’est
appelée, même en pause. L’accélération matérielle reste au réglage du runtime.

Les callbacks ne présentent pas de boîte de dialogue modale : ils postent un
message Win32, conformément au [modèle de thread WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/threading-model).
Les navigations principales sont limitées aux hôtes HTTPS explicites du service
et de son authentification. Les liens HTTPS externes déclenchés par l’utilisateur
demandent confirmation avant ouverture dans le navigateur. Les autres protocoles
sont bloqués. Le filtrage ne bloque pas les ressources/CDN et iframes nécessaires
aux services. Aucune API native ni messagerie web n’est exposée.

**Ce prototype n’est pas encore un MVP validé.** À vérifier sur des comptes réels :

- Authentification Google/Apple, abonnements et DRM. Le service peut refuser les
  navigateurs intégrés. Aucun contournement n’est prévu.
- Les fenêtres d’authentification sont redirigées dans la WebView unique ; les
  parcours exigeant un popup et `window.opener` peuvent ne pas fonctionner.
- Continuité audio, changements de piste, veille/reprise, écoute prolongée.
- Touches multimédias et intégration système fournies éventuellement par le
  runtime/site : aucun pont SMTC personnalisé n’est implémenté ou garanti.
- Les gains CPU/GPU et la comparaison PWA/Tauri/Electron restent à mesurer.

Le redémarrage après erreur est manuel pour éviter une boucle de crash et une
interruption automatique. Une erreur de sous-processus non critique peut aussi
afficher l’alerte. Les préférences de compatibilité du menu ne sont pas persistées.

Voir [le protocole de validation](docs/VALIDATION.md). Projet indépendant d’Apple,
Google et Microsoft, distribué sous licence MIT.

`./scripts/smoke.ps1` teste le cycle natif sur le binaire release, sans se connecter
ni lire de musique. Fermer toute instance avant de le lancer. Il démarre le
lecteur, vérifie le processus WebView2, masque la fenêtre, lance une seconde
instance pour la restaurer, puis simule la fin de session Windows et vérifie la
fermeture des descendants observés. Il ne vérifie pas le contenu visuel du site,
le double-clic réel sur l’icône, les menus ou l’authentification.
