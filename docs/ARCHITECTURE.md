# Architecture

UltraMusic est un exécutable Windows, sans bibliothèque publique ni serveur local.

```text
src/
├── main.rs              Point d’entrée et erreurs fatales
├── config.rs            Arguments, services et règles de navigation HTTPS
└── native/
    ├── mod.rs           État partagé, initialisation STA et boucle GetMessageW
    ├── instance.rs      Garde RAII du mutex et activation de l’instance existante
    ├── webview.rs       Initialisation asynchrone, contrôleur et callbacks
    ├── window.rs        Messages Win32, taille, visibilité et dialogues différés
    └── tray.rs          Icône de notification et menu natif
```

## Propriété et durée de vie

`App` est privé au module natif. Ses modules enfants partagent son état ; leurs
méthodes sont limitées à `pub(super)` lorsqu’un autre module doit les appeler.
`Rc`, `Cell` et `RefCell` sont intentionnels : tous les accès ont lieu sur le
thread UI initialisé en apartment COM STA. Les opérations de mise à jour utilisent
des workers temporaires qui ne reçoivent aucune référence COM ou à `App`.
Les emprunts de `RefCell` sont terminés avant les appels Win32/COM, qui peuvent
réentrer dans la boucle de messages. Le contrôleur est cloné avant utilisation.

La garde COM équilibre `CoInitializeEx` avec `CoUninitialize`. La garde d’instance
possède le handle du mutex et le ferme exactement une fois. À la sortie, le
contrôleur est fermé, l’icône supprimée et les références COM libérées avant la
désinitialisation du STA. Aucun processus tiers n’est tué.

`build.rs` incorpore l'icône multi-tailles et les métadonnées de l'exécutable via
une dépendance de compilation `winresource`. La fenêtre et la notification
réutilisent la ressource partagée `ULTRAMUSIC_ICON`, possédée par le module Windows ;
elle ne doit pas être libérée avec `DestroyIcon`. Aucun fichier image externe
n'est nécessaire à l'exécution. Les formats sont décrits dans [BRANDING.md](BRANDING.md).

## Asynchronisme et erreurs

Chaque création/redémarrage invalide la génération précédente. Les callbacks
vérifient cette génération avant de modifier l’état. Les messages différés
d’erreur et d’ouverture de lien transportent aussi la génération dans `WPARAM` :
une ancienne erreur ne peut donc pas restaurer la nouvelle fenêtre ni ouvrir
un dialogue périmé. Les liens en attente sont effacés à la fermeture du contrôleur.

Les dialogues sont traités par la procédure de fenêtre après le retour des
callbacks COM. Un échec du lecteur ne provoque jamais une boucle automatique de
redémarrage ; la commande de notification laisse ce choix à l’utilisateur.
Les erreurs de nettoyage et de présentation non critiques sont ignorées lorsque
leur échec ne peut pas être récupéré utilement pendant la fermeture.

Le second processus attend au maximum cinq secondes, par intervalles de 50 ms,
que la première fenêtre soit disponible. Cette attente appartient exclusivement
au démarrage du second processus. Le lecteur principal bloque dans `GetMessageW`
et n’a aucun timer ni polling périodique.

## Frontière de confiance

Les pages distantes n’ont ni objets hôtes Rust ni canal de messages natifs.
La navigation principale utilise une liste explicite d’hôtes HTTPS, sans suffixe
générique. Les liens externes HTTPS initiés par l’utilisateur sont confirmés avant
leur ouverture par Windows ; les autres protocoles sont refusés. Les ressources
et iframes des services restent gérées par WebView2. Aucun contournement de DRM,
d’authentification ou des politiques du service n’est effectué.

## Conventions

- Noms Rust en `snake_case`, types en `PascalCase`, constantes en `SCREAMING_SNAKE_CASE`.
- Formatage standard `rustfmt`, sans règles stylistiques personnalisées.
- Les appels natifs `unsafe` restent dans `native/` et le transport/installateur
  de `update/`. Les préconditions sont
  documentées près des frontières FFI ; `unsafe_op_in_unsafe_fn` est interdit.
- Les modules documentent leur rôle ; les commentaires expliquent les invariants
  et contraintes du moteur plutôt que de reformuler chaque instruction.
- Aucun `unwrap` ou `expect` dans le code de production. Les erreurs de démarrage
  remontent au point d’entrée et les erreurs asynchrones passent par Win32.

Les contraintes des services musicaux restent décrites dans le README.

## Version 0.2 : audio, mises à jour et installation

`native/service.rs` traite le choix Apple Music / YouTube Music du menu de
notification. Le service actif appartient à une `Cell<Service>` sur le thread UI.
La préférence `service.txt` est remplacée après une écriture complète ; un échec
d'écriture laisse le lecteur actuel intact. Au démarrage, une lecture bornée
charge ce choix ; une préférence absente ou invalide utilise YouTube Music et les
arguments explicites restent prioritaires pour ce lancement. Le changement ferme
le contrôleur précédent et invalide sa génération avant de recréer le lecteur.
Le profil, les réglages audio et l'option de rendu sont conservés ; la politique
de navigation et la relance après mise à jour utilisent le service actif.

`native/theme.rs` applique les attributs DWM de la barre noire. `native/media.rs`
gère les commandes ponctuelles du menu de notification : mute natif WebView2,
volume WASAPI limité aux sessions du processus navigateur et de ses descendants,
et lecture/pause via une évaluation ponctuelle des éléments média accessibles.
Les promesses de lecture sont attendues avec un geste utilisateur ; aucun script
de contrôle permanent n'est installé dans les pages.

`native/updates.rs` reçoit des résultats par canal et message Win32. Le transport
WinHTTP et le hachage SHA-256 restent dans des workers temporaires de `update/`.
Si l’utilisateur demande un contrôle manuel pendant celui lancé au démarrage,
cette demande transforme le résultat en retour visible et évite un second worker.
L’entrée reste activable et indique qu’elle affichera le résultat pendant ce délai.
Les boîtes de dialogue gardent l'opération occupée pour empêcher les vérifications
réentrantes. Une vérification a lieu au démarrage ; les échecs réseau de ce contrôle
initial restent silencieux. La commande manuelle affiche les erreurs éventuelles.

Seules les releases publiques non brouillon et non préversion sont éligibles.
Le tag est comparé avec SemVer, l'asset doit s'appeler `UltraMusic-Setup.exe` et
provenir de ce dépôt. HTTPS, les hôtes de redirection, la taille et le digest
GitHub sont contrôlés. Les réponses JSON et binaires sont bornées à 2 et 128 Mio.

Après confirmation, un helper temporaire attend la fin du lecteur pendant au
plus 30 secondes, sans polling. Il vérifie à nouveau le digest, sauvegarde le
lecteur, affiche la progression de l'installeur et conserve son journal dans le
dossier de la mise à jour, puis relance le service choisi. En cas
d'échec détecté de l'installation ou du lancement, il tente de restaurer l'ancien
exécutable. Cela ne constitue pas une détection des défaillances ultérieures de
l'application ni un rollback complet du registre de l'installeur.

`installer/UltraMusic.iss` installe dans le dossier Programs de l'utilisateur,
enregistre la désinstallation et ajoute les raccourcis. Son AppId reste constant
entre versions. Il installe WebView2 si aucune installation existante n'est détectée.
Le script de build vérifie la signature Microsoft du bootstrapper embarqué.
La désinstallation conserve volontairement les cookies et données de session.
