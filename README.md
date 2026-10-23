# 四次元 Yonjigen

Moteur de jeu + éditeur **4D** (x, y, z, w) en Rust — l'équivalent d'Unity mais avec
une quatrième dimension spatiale.

## État actuel (étapes 1 à 3)

- Maths 4D complètes : `Vec4`, `Mat5` (transformations homogènes), rotations sur
  les 6 plans (XY, XZ, XW, YZ, YW, ZW), `Transform4`.
- Primitives : tesseract (avec faces, pour la coupe), 5-cell, 16-cell, 24-cell,
  hypersphère, duoprisme.
- Pipeline de rendu : projection perspective 4D→3D→2D, et coupe (slice) par
  l'hyperplan `w = c`, réglable en temps réel.
- Scène/ECS léger avec sauvegarde/chargement JSON (`.yonjigen`).
- Scripting Lua (`mlua`, Lua 5.4 vendored) : composant Script par entité,
  exécuté chaque frame pendant Play via une API minimale (`get_position`,
  `set_position`, `get_rotation`, `set_rotation`, `rotate`, `log`, `dt`,
  constantes `PLANE_XY`..`PLANE_ZW`).
- Éditeur (`eframe`/`egui`) façon **Unity** : barre de menu + toolbar séparées,
  Hiérarchie (recherche, création, renommage par double-clic, actions au
  clic-droit), Inspecteur à sections repliables, onglets Scene / Jeu / Code,
  Project / Console en bas à gauche.
- Éditeur de code Lua intégré : coloration syntaxique, numéros de ligne,
  onglets de scripts ouverts, menu d'autocomplétion de l'API, bouton
  Exécuter, console de logs/erreurs reliée à l'exécution en direct pendant
  Play.
- Thème sombre gris anthracite, accent bleu discret, contour de sélection
  orange, coins arrondis, espacement généreux ; police Inter embarquée dans
  le binaire (cohérente sur tous les OS), repli CJK système pour "四次元".
- Tests unitaires : maths 4D (rotations préservent les longueurs, coupe d'un
  tesseract à `w = 0` = cube...), scène (ajout/duplication/JSON), scripting
  Lua (lecture/écriture de transform, `dt`, logs, erreurs de syntaxe).

## Structure du workspace

- `yonjigen-core` : le moteur (maths, mesh 4D, rendu, scène/ECS léger).
- `yonjigen-editor` : l'éditeur graphique (eframe/egui).

## Build

Nécessite Rust (édition 2024, via [rustup](https://rustup.rs)).

```bash
cargo build --release
cargo run -p yonjigen-editor --release
```

### Linux (Ubuntu / Mint / Fedora / Arch)

Dépendances de build natives typiques (fenêtrage, GL) :

- Ubuntu/Mint (apt) : `libxkbcommon-x11-0 libxcb1 libgl1 pkg-config libfontconfig1-dev`
- Fedora (dnf) : `libxkbcommon-x11 libxcb pkgconf fontconfig-devel`
- Arch (pacman) : `libxkbcommon libxcb fontconfig pkgconf`

Un script `install.sh` qui automatise la détection du gestionnaire de paquets
et le build release sera ajouté à l'étape 7 (packaging multi-OS + CI).

### Windows / macOS

Installez Rust via [rustup](https://rustup.rs), puis `cargo build --release`.
Les scripts dédiés (`install.ps1`, instructions Homebrew) arrivent à l'étape 7.

## Tests

```bash
cargo test
cargo clippy --all-targets --all-features
cargo fmt --all --check
```

## Décisions prises (ambiguïtés du cahier des charges)

- Édition Rust 2024, résolveur de workspace v2.
- `eframe`/`egui` 0.37 (dernière stable au moment du développement).
- Mesh 4D : les faces (`Vec<Vec<usize>>`) ne sont peuplées que pour le
  tesseract pour l'instant (nécessaire à l'algorithme de coupe générique par
  faces) ; les autres primitives (5-cell, 16-cell, 24-cell, hypersphère,
  duoprisme) n'ont pour l'instant que sommets/arêtes et s'affichent donc
  seulement en mode Projection. Le mode Coupe sera étendu à ces formes plus
  tard si besoin.
- Sauvegarde de scène : JSON (`serde_json`), extension `.yonjigen`.
- Scripting Lua : la VM Lua est recréée à chaque frame (pas de VM persistante)
  — plus simple à implémenter en toute sécurité vis-à-vis de l'emprunt mutable
  de la scène, et ça permet l'édition à chaud du script sans redémarrage. Les
  variables locales Lua ne survivent donc pas d'une frame à l'autre ; seul
  l'état du `Transform4` (exposé via l'API) est persistant.
- Éditeur de code : autocomplétion simplifiée (menu listant les signatures de
  l'API, insertion en fin de script) plutôt qu'un popup contextuel au niveau
  du curseur ; coloration syntaxique par un tokenizer ligne par ligne (pas de
  gestion des chaînes multi-lignes `[[...]]`).
- Police embarquée : Inter (SIL OFL 1.1, voir
  `yonjigen-editor/assets/fonts/Inter-OFL-LICENSE.txt`), choisie pour un rendu
  identique sur tous les OS sans dépendre des polices système.

## Roadmap

Voir les étapes 4 à 7 du plan : modélisateur 4D, éditeurs pixel art / images,
physique 4D + export standalone, packaging multi-OS + CI + release.
