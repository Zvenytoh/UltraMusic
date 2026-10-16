# Contribuer à UltraMusic

Lire le [README](README.md) et l’[architecture](docs/ARCHITECTURE.md) avant de modifier
le lecteur. Utiliser Rust stable et, pour le build de référence, Windows x64 avec
les outils C++ MSVC et le SDK Windows.

Le script `scripts/package.ps1` prépare le ZIP portable avec `UltraMusic.exe` à
la racine. Utiliser `-IncludeLoader` pour un build GNU local. La CI utilise MSVC
et publie l'archive sur GitHub Releases lorsqu'un tag `v*` est poussé, après
réussite des contrôles et de la compilation. Les versions actuelles sont
publiées comme préversions ; le README fournit un lien direct au ZIP public.

## Vérifications avant publication

```powershell
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
```

La politique actuelle du projet ne comprend pas de tests unitaires. La CI ne
lance que les contrôles statiques et la compilation. Le script `scripts/smoke.ps1`
reste disponible pour une validation fonctionnelle explicitement demandée ; il
n’est pas exécuté par la CI. Les essais audio et benchmarks ont leur propre
[protocole](docs/VALIDATION.md) et ne se déduisent pas d’un build réussi.

## Règles de modification

Conserver les responsabilités des modules et la visibilité la plus étroite
possible. Ne pas ajouter de polling, script permanent, télémétrie ou framework
sans besoin démontré. Documenter les garanties de durée de vie et de thread pour
les appels FFI. Ne pas conserver d’emprunt `RefCell` pendant un appel natif.

Inclure `Cargo.lock` dans les changements de dépendances. Ne jamais versionner
les outils locaux, binaires, profils WebView2, cookies, traces ETL ou identifiants.
Les branches de travail utilisent le préfixe `codex/` ; `main` est la branche de
publication initiale. Décrire le problème corrigé, le comportement obtenu et les
vérifications réellement effectuées, sans annoncer de performances non mesurées.
