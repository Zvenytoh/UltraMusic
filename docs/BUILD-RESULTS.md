# Vérification locale — 17 octobre 2026

> Historique du prototype **avant réorganisation des modules**. Les tests
> unitaires décrits ici ont depuis été supprimés à la demande du propriétaire.
> Ces résultats ne valident pas les changements ultérieurs ; aucune nouvelle
> exécution de tests n’a été faite pendant la revue de publication.

Prototype 0.1.0, Windows x64, Rust 1.99.1, cible `x86_64-pc-windows-gnu`.
Le build local utilise GCC MinGW ; le workflow CI fourni cible MSVC et n’a pas
encore été exécuté sur GitHub.

| Vérification | Résultat |
|---|---|
| `cargo check` | Réussi |
| `cargo fmt --all -- --check` | Réussi |
| `cargo clippy --locked --all-targets -- -D warnings` | Réussi |
| `cargo test --locked` | 2 tests réussis : limites de navigation et protocoles externes |
| `cargo build --release --locked` | Réussi |
| `scripts/smoke.ps1` | Réussi |

Le test natif a créé la fenêtre, activé explicitement sa visibilité après un
démarrage masqué demandé par PowerShell, observé un processus WebView2 enfant,
envoyé WM_CLOSE, vérifié le masquage sans sortie, lancé une seconde instance,
vérifié sa sortie et la restauration, puis envoyé WM_ENDSESSION. Le wrapper et
les descendants recensés étaient terminés à la fin du test.

Ce test ne prouve pas le chargement visuel du site, l’authentification, les DRM,
la continuité audio, le double-clic réel ou les commandes du menu de notification.
Il n’a pas mesuré CPU, GPU, puissance ou latence de restauration. Ces vérifications
restent à effectuer selon `VALIDATION.md` avant de déclarer le MVP terminé.
