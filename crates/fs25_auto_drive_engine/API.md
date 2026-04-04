# Engine Crate API

## Ueberblick

`fs25_auto_drive_engine` kapselt die host-neutrale Fachlogik des Editors. Die Crate enthaelt `app`, `core`, `shared` und `xml`, kennt aber kein `egui`, `eframe` oder anderes Frontend-Toolkit.

## Oeffentliche Oberflaeche

- `app` — Controller, State, Events, Use-Cases und Tool-/UI-Vertraege
- `core` — RoadMap, Nodes, Connections, Kamera, Spatial-Index, Heightmap, BackgroundMap
- `shared` — EditorOptions, RenderScene und weitere layer-uebergreifende DTOs
- `xml` — AutoDrive- und Curseplay-Import/Export

## Root-Kompatibilitaet

Das Root-Package `fs25_auto_drive_editor` re-exportiert diese Crate weiter, damit bestehende Tests, Benches und Rust-Konsumenten stabil bleiben.
