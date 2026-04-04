# Engine Crate API

## Ueberblick

`fs25_auto_drive_engine` kapselt die host-neutrale Fachlogik des Editors. In diesem Zwischenstand enthaelt die Crate bereits `core`, `shared` und `xml`; der `app`-Layer folgt im naechsten Commit. Die Crate kennt kein `egui`, `eframe` oder anderes Frontend-Toolkit.

## Oeffentliche Oberflaeche

- `core` — RoadMap, Nodes, Connections, Kamera, Spatial-Index, Heightmap, BackgroundMap
- `shared` — EditorOptions, RenderScene und weitere layer-uebergreifende DTOs
- `xml` — AutoDrive- und Curseplay-Import/Export

## Root-Kompatibilitaet

Das Root-Package `fs25_auto_drive_editor` re-exportiert diese Crate weiter, damit bestehende Tests, Benches und Rust-Konsumenten stabil bleiben.
