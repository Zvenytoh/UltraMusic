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
thread UI initialisé en apartment COM STA, sans thread de travail ni verrou.
Les emprunts de `RefCell` sont terminés avant les appels Win32/COM, qui peuvent
réentrer dans la boucle de messages. Le contrôleur est cloné avant utilisation.

La garde COM équilibre `CoInitializeEx` avec `CoUninitialize`. La garde d’instance
possède le handle du mutex et le ferme exactement une fois. À la sortie, le
contrôleur est fermé, l’icône supprimée et les références COM libérées avant la
désinitialisation du STA. Aucun processus tiers n’est tué.

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
- Les appels natifs `unsafe` restent dans `native/`. Les préconditions sont
  documentées près des frontières FFI ; `unsafe_op_in_unsafe_fn` est interdit.
- Les modules documentent leur rôle ; les commentaires expliquent les invariants
  et contraintes du moteur plutôt que de reformuler chaque instruction.
- Aucun `unwrap` ou `expect` dans le code de production. Les erreurs de démarrage
  remontent au point d’entrée et les erreurs asynchrones passent par Win32.

Le statut de prototype et les limites audio/authentification restent décrits
dans le README et le protocole de validation.
