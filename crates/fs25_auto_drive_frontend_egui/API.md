# Egui Frontend API

## Ueberblick

`fs25_auto_drive_frontend_egui` kapselt das Desktop-Frontend des Editors. In diesem Zwischenstand enthaelt die Crate zunaechst den render-spezifischen wgpu-/egui-Stack; `ui`, `editor_app` und `runtime` folgen im naechsten Commit.

## Oeffentliche Oberflaeche

- `render` — wgpu-Renderer, Background-Upload und egui-Callback
- Re-Exports von `app`, `core`, `shared`, `xml` aus `fs25_auto_drive_engine` fuer bestehende Frontend-Pfade

## Root-Kompatibilitaet

Das Root-Package re-exportiert `render` weiterhin, damit der bisherige Desktop-Launcher in diesem Commit unveraendert gegen `fs25_auto_drive_editor::render` kompilieren kann.
