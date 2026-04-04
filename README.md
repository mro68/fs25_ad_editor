# FS25 AutoDrive Editor

Hochperformanter Editor fuer [AutoDrive](https://github.com/Stephan-S/FS25_AutoDrive)-Kurse in Farming Simulator 25, geschrieben in Rust.

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-blue)
![License](https://img.shields.io/badge/license-GPL--3.0-green)

## Warum?

Der originale [AutoDrive Course Editor](https://github.com/KillBait/AutoDrive_Course_Editor) (JavaScript/Electron) von KillBait wird bei grossen Karten mit 10'000+ Wegpunkten bei mir langsam. Dieser Editor nutzt GPU-Rendering fuer fluessiges Arbeiten auch bei sehr grossen Kursnetzwerken.

## Features

- **Laden & Speichern** von AutoDrive-Configs (FS25 XML-Format)
- **GPU-Rendering** via wgpu mit Instancing – 100k+ Nodes fluessig
- **Spatial Index** (KD-Tree) fuer schnelle Punkt-Abfragen
- **Map-Hintergrund** – DDS/PNG/JPG als Uebersichtskarte
- **Uebersichtskarten-Generierung** – Erzeugt vollstaendige Map-Uebersichten direkt aus Map-Mod-ZIPs (Terrain, Farmlands, POIs), Layer einzeln konfigurierbar
- **Auto-Detection** – Erkennt nach dem Laden automatisch Heightmap und passende Map-Mod-ZIPs im Mods-Verzeichnis (Umlaut-tolerantes Fuzzy-Matching)
- **Heightmap-Support** – 8/16-Bit PNG, automatische Hoehenrekonstruktion beim Speichern
- **Duplikat-Erkennung** – Findet und bereinigt doppelte Wegpunkte
- **Distanzen-Neuverteilung** – Catmull-Rom-Spline-basierte gleichmaessige Neuverteilung von Wegpunkten (nach Abstand oder Anzahl)
- **Route-Tools** – Kurven (Bézier), Splines (Catmull-Rom) und Geraden mit Tangenten-Ausrichtung und Verkettung
- **Cross-Platform** – Native Binaries fuer Linux und Windows

## Download

Fertige Binaries findest du unter [Releases](https://github.com/mro68/fs25_ad_editor/releases).

| Plattform | Datei |
|-----------|-------|
| Linux x64 | `FS25-AutoDrive-Editor_x64_linux` |
| Windows x64 | `FS25-AutoDrive-Editor_x64_windows.exe` |

## Aus Quellcode bauen

### Voraussetzungen

- [Rust](https://rustup.rs/) (Edition 2021)
- Linux: GPU-Treiber mit Vulkan-Support

```bash
# Ubuntu/Debian: benoetigte System-Libraries
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                 libxkbcommon-dev libssl-dev
```

### Build

```bash
# Release-Build
cargo build --release -p FS25-AutoDrive-Editor --bin FS25-AutoDrive-Editor

# Starten
cargo run --release -p FS25-AutoDrive-Editor --bin FS25-AutoDrive-Editor

# Gesamten Workspace pruefen
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Cross-Compile (Linux → Windows)

Benoetigt [cargo-xwin](https://github.com/rust-cross/cargo-xwin):

```bash
cargo install cargo-xwin
rustup target add x86_64-pc-windows-msvc
cargo xwin build --release --target x86_64-pc-windows-msvc
```

Oder via Makefile:

```bash
make                  # Release: Linux + Windows
make linux-release    # Nur Linux
make windows-release  # Nur Windows
```

## Verwendung

1. **Datei oeffnen:** `Datei → Oeffnen` oder `Strg+O` – eine AutoDrive XML-Config laden
2. **Navigieren:** Mausrad zum Zoomen, Rechtsklick + Ziehen zum Verschieben
3. **Hintergrundkarte:** `View → Hintergrund laden` – eine overview.dds/png/jpg laden
4. **Bearbeiten:** Punkte auswaehlen, verschieben, verbinden, loeschen
5. **Speichern:** `Strg+S` – Hoehen werden automatisch aus der Heightmap rekonstruiert

Ausfuehrliche Anleitung: [docs/howto/index.md](docs/howto/index.md)

## Architektur

| Crate | Aufgabe |
|-------|---------|
| `FS25-AutoDrive-Editor` | Root-Fassade und nativer Launcher |
| `fs25_auto_drive_engine` | Host-neutrale Engine (`app`, `core`, `shared`, `xml`) |
| `fs25_auto_drive_host_bridge` | Toolkit-freie Host-Bridge-Core-Crate ueber der Engine |
| `fs25_auto_drive_render_wgpu` | Host-neutraler wgpu-Renderer-Kern |
| `fs25_auto_drive_frontend_egui` | Desktop-Frontend (`ui`, `editor_app`, `runtime`, `render` als Host-Adapter) |
| `fs25_auto_drive_frontend_flutter_bridge` | Flutter-seitige Adapter-/Kompat-Schicht fuer die gemeinsame Host-Bridge |
| `fs25_map_overview` | Overview-, Terrain- und Farmland-Generierung |

Die Root-Crate `fs25_auto_drive_editor` bleibt als Kompat-Fassade erhalten und re-exportiert die kanonischen Engine-Module weiterhin fuer Tests, Benches und bestehende Rust-Imports.

Detaillierte Beschreibung: [docs/ARCHITECTURE_PLAN.md](docs/ARCHITECTURE_PLAN.md)

## Entwicklung

```bash
cargo test            # Tests
cargo clippy          # Linter
cargo fmt             # Formatierung
cargo bench           # Benchmarks
```

## Tech-Stack

| Zweck | Crate |
|-------|-------|
| UI | [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) |
| Rendering | [wgpu](https://wgpu.rs/) |
| XML | [quick-xml](https://github.com/tafia/quick-xml) |
| Spatial Index | [kiddo](https://github.com/sdd/kiddo) (KD-Tree) |
| Bildverarbeitung | [image](https://github.com/image-rs/image) |
| Map-Uebersicht | `fs25_map_overview` (integriertes Crate) |
| Mathe | [glam](https://github.com/bitshifter/glam-rs) |
| Pattern-Matching | [regex](https://github.com/rust-lang/regex) |

## Danksagungen / Attributionen

- **[AutoDrive](https://github.com/Stephan-S/FS25_AutoDrive)** von Stephan S. – Die Grundlage fuer das XML-Format und die Wegpunkt-Logik
- **[grleconvert](https://github.com/Paint-a-Farm/grleconvert)** von Paint-a-Farm – Konvertierung von GIANTS-Engine `.grle`-Dateien
- **GRLE/GDM-Parsing** basiert auf `grleconvert` von Kim Brandwijk (MIT-Lizenz) und Erkenntnissen der FS-Modding-Community
- Die Uebersichtskarten-Generierung (`fs25_map_overview` Crate) dekodiert GIANTS-eigene GDM/GRLE-Formate fuer Farmland-Grenzen und Terrain-Daten

## Lizenz

GPL-3.0 – siehe [LICENSE](LICENSE).

## Mitmachen / Contributing

Dein Feedback, Issues und Pull Requests sind willkommen! Ich freue mich über Verbesserungen und Anregungen. Erstelle gerne ein [Issue](https://github.com/mro68/fs25_ad_editor/issues) oder einen Pull Request.

**Sprache / Language:**
Meine Muttersprache ist Deutsch. Ich verstehe Englisch, aber eine umfangreiche Unterhaltung auf Englisch faellt mir schwer. Issues und Kommentare auf Deutsch sind willkommen – English is fine too for short messages.
