# Egui Frontend API

## Ueberblick

`fs25_auto_drive_frontend_egui` kapselt das Desktop-Frontend des Editors. Die Crate enthaelt den render-spezifischen wgpu-/egui-Stack, die komplette egui-Oberflaeche, die eframe-Integrationsschale und den nativen Desktop-Launcher.

## Oeffentliche Oberflaeche

- `editor_app` — eframe-Integrationsschale und Frame-Zyklus
- `render` — wgpu-Renderer, Background-Upload und egui-Callback
- `ui` — Menues, Panels, Dialoge und Viewport-Input
- `run_native()` — nativer Desktop-Einstiegspunkt fuer das egui-Frontend
- Re-Exports von `app`, `core`, `shared`, `xml` aus `fs25_auto_drive_engine` fuer bestehende Frontend-Pfade

## Root-Kompatibilitaet

Das Root-Package re-exportiert `render` und `ui` weiterhin und ruft als duenner Launcher nur noch `fs25_auto_drive_frontend_egui::run_native()` auf.
