# 四次元 Yonjigen

Moteur de jeu + éditeur **4D** (x, y, z, w) en Rust — l'équivalent d'Unity mais avec
une quatrième dimension spatiale.

## État actuel (étape 1 : MVP)

- Maths 4D complètes : `Vec4`, `Mat5` (transformations homogènes), rotations sur
  les 6 plans (XY, XZ, XW, YZ, YW, ZW), `Transform4`.
- Primitive **tesseract** avec sommets/arêtes/faces.
- Pipeline de rendu : projection perspective 4D→3D→2D, et coupe (slice) par
  l'hyperplan `w = c`, réglable en temps réel.
- Éditeur (`eframe`/`egui`) : viewport avec orbite souris + zoom molette,
  slider de coupe `w`, bascule Projection / Coupe / Les deux, play/pause avec
  vitesse de rotation par plan, hiérarchie de scène, inspecteur de composants.
- Thème sombre néon (violet/bleu), coins carrés, police monospace, logo
  pixel-art "四次元" (police CJK système si disponible, sinon repli "YONJIGEN").
- Tests unitaires sur les maths 4D (rotations préservent les longueurs, coupe
  d'un tesseract à `w = 0` = cube, etc.).

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
  seulement en mode Projection. Le mode Coupe sera étendu à ces formes à
  l'étape 2.
- Sauvegarde de scène : JSON (`serde_json`), extension `.yonjigen`.
- Scripting Lua (`mlua`, feature `vendored`) : prévu à l'étape 3, pas encore
  intégré.

## Roadmap

Voir les étapes 2 à 7 du plan : formes supplémentaires + sauvegarde, scripting
Lua + API moteur, modélisateur 4D, éditeurs pixel art / images, physique 4D +
export standalone, packaging multi-OS + CI + release.
