# Revue de publication — 17 octobre 2026

Cette revue porte sur les sources réorganisées avant le premier push. Elle
remplace les résultats du prototype pour l’état courant du code ; les résultats
précédents sont conservés séparément dans `BUILD-RESULTS.md` à titre historique.

## Changements examinés

- Séparation du code natif en modules d’instance, de WebView2, de fenêtre et de
  notification ; état partagé privé et visibilité limitée au module parent.
- Génération du lecteur propagée aux messages différés d’erreur et de lien.
  Les messages obsolètes sont ignorés, y compris après un dialogue externe.
- Attente bornée du second processus lors de la création de la première fenêtre.
- Suppression des tests unitaires, de leur commande de développement et de leur
  étape CI. Le script fonctionnel existant est conservé, sans exécution.
- Documentation des invariants COM/Win32, absence de panique explicite dans le
  code de production, et retour d’un code d’échec pour une erreur de démarrage.
- Restauration de l’environnement PowerShell après les commandes de développement.

## Contrôles effectués

Lecture des sources Rust, de la politique de navigation, des scripts et du
workflow. Contrôles locaux : `cargo fmt --all -- --check`, `cargo check --locked`,
`cargo clippy --locked --all-targets -- -D warnings` et
`cargo build --release --locked` réussis sur Windows x64 avec l’outillage GNU.
La syntaxe du script de développement a aussi été analysée sans l’exécuter en
mode fonctionnel. Les fichiers destinés au commit ont été contrôlés pour exclure
outils, binaires, profils, identifiants et chemins personnels.

**Aucun test unitaire ou fonctionnel n’a été exécuté pendant cette intervention.**
Clippy et le compilateur vérifient le code ; ils ne prouvent pas le comportement
du lecteur à l’exécution. Le résultat de la CI MSVC est consulté séparément sur
GitHub après le push ; il ne doit pas être déduit du build GNU local.

## Limites conservées

Authentification et popups, DRM, audio prolongé en arrière-plan, touches
multimédias, performances CPU/GPU et comparaison PWA restent à valider selon
`VALIDATION.md`. Aucun chiffre de performance ni garantie d’absence de défaut
n’est annoncé sur la base de cette revue.
