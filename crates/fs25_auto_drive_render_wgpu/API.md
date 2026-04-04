# API der Render-wgpu-Crate

## Ueberblick

`fs25_auto_drive_render_wgpu` enthaelt den host-neutralen wgpu-Renderer-Kern. Die Crate konsumiert den Engine-Vertrag (`RenderScene`) und raw `wgpu`-Typen, kennt aber kein `egui`, `eframe` oder Flutter-SDK.

## Oeffentliche Typen

| Typ | Zweck |
|---|---|
| `Renderer` | Host-neutraler GPU-Renderer fuer `RenderScene` |
| `RendererTargetConfig` | Zielkonfiguration des Render-Targets (`color_format`, `sample_count`) |
| `BackgroundWorldBounds` | Weltkoordinaten des Background-Quads |

## Oeffentliche Methoden

| Signatur | Zweck |
|---|---|
| `Renderer::new(device, queue, target_config)` | Erstellt den Renderer mit raw `wgpu` |
| `Renderer::render_scene(device, queue, render_pass, scene)` | Rendert den aktuellen `RenderScene`-Snapshot |
| `Renderer::set_background(device, queue, image, world_bounds, scale)` | Setzt/aktualisiert das Background-Asset |
| `Renderer::clear_background()` | Entfernt das Background-Asset |

## Scope

- Diese Crate enthaelt nur den GPU-Kern und keine Host-Callback-Logik.
- Host-spezifische Adapter (z. B. egui `CallbackTrait`) bleiben in den Frontend-Crates.
