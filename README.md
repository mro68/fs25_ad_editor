# FS25 AutoDrive Editor

Hochperformanter Editor für [AutoDrive](https://github.com/Stephan-S/FS25_AutoDrive)-Kurse in Farming Simulator 25, geschrieben in Rust.

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-blue)
![License](https://img.shields.io/badge/license-GPL--3.0-green)

## Warum?

Der Original-[AutoDrive Course Editor](https://github.com/Jesper-Hustad/AutoDrive_Course_Editor) (JavaScript/Electron) wird bei großen Karten mit 100.000+ Wegpunkten langsam. Dieser Editor nutzt GPU-Rendering für flüssiges Arbeiten auch bei sehr großen Kursnetzwerken.

## Features

- **Laden & Speichern** von AutoDrive-Configs (FS25 XML-Format)
- **GPU-Rendering** via wgpu mit Instancing – 100k+ Nodes flüssig
- **Spatial Index** (KD-Tree) für schnelle Punkt-Abfragen
- **Map-Hintergrund** – DDS/PNG/JPG als Übersichtskarte
- **Übersichtskarten-Generierung** – Erzeugt vollständige Map-Übersichten direkt aus Map-Mod-ZIPs (Terrain, Farmlands, POIs), Layer einzeln konfigurierbar
- **Heightmap-Support** – 8/16-Bit PNG, automatische Höhenrekonstruktion beim Speichern
- **Duplikat-Erkennung** – Findet und bereinigt doppelte Wegpunkte
- **Cross-Platform** – Native Binaries für Linux und Windows

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
# Ubuntu/Debian: benötigte System-Libraries
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                 libxkbcommon-dev libssl-dev
```

### Build

```bash
# Release-Build
cargo build --release

# Starten
cargo run --release
```

### Cross-Compile (Linux → Windows)

Benötigt [cargo-xwin](https://github.com/rust-cross/cargo-xwin):

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

1. **Datei öffnen:** `Datei → Öffnen` oder `Strg+O` – eine AutoDrive XML-Config laden
2. **Navigieren:** Mausrad zum Zoomen, Rechtsklick + Ziehen zum Verschieben
3. **Hintergrundkarte:** `View → Hintergrund laden` – eine overview.dds/png/jpg laden
4. **Bearbeiten:** Punkte auswählen, verschieben, verbinden, löschen
5. **Speichern:** `Strg+S` – Höhen werden automatisch aus der Heightmap rekonstruiert

Ausführliche Anleitung: [docs/How-To-Use.md](docs/How-To-Use.md)

## Architektur

| Layer | Verzeichnis | Aufgabe |
|-------|-------------|---------|
| UI | `src/ui/` | Darstellung + AppIntent-Erzeugung |
| Application | `src/app/` | Controller, Use-Cases, State |
| Domain | `src/core/` | Fachmodell: RoadMap, Node, Connection |
| Persistence | `src/xml/` | AutoDrive XML Parser/Writer |
| Rendering | `src/render/` | wgpu GPU-Pipeline, Culling, Instancing |
| Shared | `src/shared/` | RenderScene, RenderQuality (cross-layer) |

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
| Map-Übersicht | `fs25_map_overview` (integriertes Crate) |
| Mathe | [glam](https://github.com/bitshifter/glam-rs) |

## Danksagungen / Attributionen

- **[AutoDrive](https://github.com/Stephan-S/FS25_AutoDrive)** von Stephan S. – Die Grundlage für das XML-Format und die Wegpunkt-Logik
- **GRLE/GDM-Parsing** basiert auf Erkenntnissen aus dem [FS Map Viewer](https://github.com/example/fs-map-viewer) und der FS-Modding-Community
- Die Übersichtskarten-Generierung (`fs25_map_overview` Crate) dekodiert GIANTS-eigene GDM/GRLE-Formate für Farmland-Grenzen und Terrain-Daten

## Lizenz

GPL-3.0 – siehe [LICENSE](LICENSE).

## Mitmachen / Contributing

Dieses Projekt ist mein erstes öffentliches Repository – ich bin komplett neu im Umgang mit Git und GitHub. Über Tipps, Anregungen und Verbesserungsvorschläge freue ich mich sehr! Erstelle gerne ein [Issue](https://github.com/mro68/fs25_ad_editor/issues) oder einen Pull Request.

**Sprache / Language:**
Meine Muttersprache ist Deutsch. Ich verstehe Englisch, aber eine umfangreiche Unterhaltung auf Englisch fällt mir schwer. Issues und Kommentare auf Deutsch sind willkommen – English is fine too for short messages.

## Credits

Inspiriert vom Original [AutoDrive Course Editor](https://github.com/Jesper-Hustad/AutoDrive_Course_Editor) von Jesper Hustad.
