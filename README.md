# FS25 AutoDrive Editor (RADE)

Rust-basierter Editor für AutoDrive-Kurse in Farming Simulator 25.

## Motivation

Dieser Editor ist eine Neuimplementierung des [AutoDrive Course Editor](https://github.com/Jesper-Hustad/AutoDrive_Course_Editor) in Rust. Ziele:

- **Performance:** Flüssiges Arbeiten mit 100.000+ Wegpunkten durch GPU-Rendering
- **Cross-Platform:** Native Desktop-Apps (Windows/Linux/macOS) und Web-Version möglich
- **Modern:** Aktuelle Technologien (Rust, wgpu, egui) für wartbaren Code

## Features (geplant)

- ✅ Laden und Speichern von AutoDrive-Configs (FS25-Format)
- ✅ Hochperformantes 2D-Rendering (wgpu + GPU-Instancing)
- ✅ DDS-Texturen für Map-Hintergründe
- ✅ Spatial Index (KD-Tree) für schnelle Abfragen
- ✅ Heightmap-Support (PNG) mit bikubischer Interpolation für präzise Y-Koordinaten
- 🚧 Interaktive Tools (Select, Move, Connect, Delete)
- 🚧 Kurven-Werkzeuge (Bezier, Arc-Spline)
- 🚧 Marker-Management
- 🚧 Undo/Redo-System

## Technologie-Stack

- **Sprache:** Rust
- **UI:** [egui](https://github.com/emilk/egui) (Immediate Mode GUI)
- **Rendering:** [wgpu](https://wgpu.rs/) (plattformübergreifende GPU-API)
- **XML:** [quick-xml](https://github.com/tafia/quick-xml)
- **Spatial Index:** [kiddo](https://github.com/sdd/kiddo) (KD-Tree)
- **DDS:** [dds-rs](https://crates.io/crates/dds) oder image crate

## Architektur

```
src/
├── app/            # Intent/Command-Flow, Controller, Use-Cases, AppState
├── core/           # Datenmodelle und Domain-Logik (RoadMap, MapNode, Connection)
├── xml/            # AutoDrive XML Parser/Writer
├── render/         # wgpu Rendering-Pipeline
└── ui/             # egui Interface (emittiert AppIntent)
```

Kernfluss: `Input -> AppIntent -> AppController -> AppCommand -> Use-Cases -> AppState -> RenderScene -> Renderer`

## Installation

```bash
# Dependencies (Ubuntu/Debian)
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                 libxkbcommon-dev libssl-dev

# Build
cargo build --release

# Run
cargo run --release
```

## Entwicklung

### Projekt bauen
```bash
cargo build
```

### Tests ausführen
```bash
cargo test
```

### Code formatieren
```bash
cargo fmt
```

### Linter ausführen
```bash
cargo clippy
```

## Lizenz

Noch zu klären (wahrscheinlich GPL-3.0 wie das Original).

## Credits

Basierend auf dem Original [AutoDrive Course Editor](https://github.com/Jesper-Hustad/AutoDrive_Course_Editor).
