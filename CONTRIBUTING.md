# Contribuer à UltraMusic

Lire le [README](README.md) et l’[architecture](docs/ARCHITECTURE.md) avant de modifier
le lecteur. Utiliser Rust stable et, pour le build de référence, Windows x64 avec
les outils C++ MSVC et le SDK Windows.

La configuration Codex du projet est décrite dans [docs/CODEX.md](docs/CODEX.md).

La CI utilise MSVC et Inno Setup pour publier `UltraMusic-Setup.exe` sur GitHub
Releases lorsqu'un tag `v*` est poussé, après réussite des contrôles et compilation.
Le runtime C est lié statiquement via `.cargo/config.toml` ; le loader WebView2
et l'icône sont également incorporés. Le README fournit le lien direct vers
l'installeur. La version du tag doit correspondre à celle de `Cargo.toml`.

## Vérifications avant publication

```powershell
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
.\scripts\audit.ps1
```

La politique actuelle du projet ne comprend pas de tests unitaires. La CI ne
lance que l’audit des dépendances, les contrôles statiques et la compilation. Le script `scripts/smoke.ps1`
reste disponible pour une validation fonctionnelle explicitement demandée ; il
n’est pas exécuté par la CI. Les essais audio et benchmarks ont leur propre
[protocole](docs/VALIDATION.md) et ne se déduisent pas d’un build réussi.

Avec l’outillage local, utiliser `scripts/dev.ps1 audit` pour préparer
l’environnement Cargo. Le script d’audit vérifie le SHA-256 de son outil RustSec
avant exécution. La [revue de sécurité du 19 octobre 2026](docs/SECURITY-REVIEW-2026-10-19.md)
décrit les contrôles réalisés et leurs limites.

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
