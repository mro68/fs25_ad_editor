# API des render-Moduls

## Ueberblick

Das `render`-Modul ist der egui-spezifische Host-Adapter ueber dem host-neutralen Kern aus `fs25_auto_drive_render_wgpu`. Seine oeffentliche Surface bleibt bewusst klein: ein duenner `Renderer`, die re-exportierten Kern-Vertraege (`RendererTargetConfig`, `BackgroundWorldBounds`, `RenderScene`, `RenderQuality`) sowie der egui-Callback (`WgpuRenderCallback`, `WgpuRenderData`).

Das Modul baut keine Render-Snapshots selbst. `EditorApp` liest pro Frame einen gekoppelten `HostRenderFrameSnapshot` ueber `fs25_auto_drive_host_bridge::build_render_frame(...)`, uebergibt dessen `RenderScene` an den egui-Callback und synchronisiert dessen langlebige Assets revisionsbasiert im selben Frame in den Host-Adapter hinein.

## Module

| Modul | Verantwortung |
|---|---|
| `mod.rs` | `Renderer` als egui-Host-Adapter plus Re-Exports der Kern-Vertraege |
| `callback.rs` | `WgpuRenderCallback` und `WgpuRenderData` fuer `egui_wgpu::CallbackTrait` |

## Oeffentliche Typen

| Typ | Zweck |
|---|---|
| `Renderer` | Duenner egui-Host-Adapter ueber `fs25_auto_drive_render_wgpu::Renderer` |
| `RendererTargetConfig` | Re-exportierte Target-Konfiguration fuer Farbformat und MSAA des Render-Core |
| `BackgroundWorldBounds` | Re-exportierter Upload-Vertrag fuer das Hintergrund-Quad im Render-Core |
| `RenderScene` | Re-exportierter per-frame Render-Vertrag aus der Engine |
| `RenderQuality` | Re-exportierte Qualitaetsstufe fuer Anti-Aliasing |
| `WgpuRenderData` | Per-Frame-Traeger fuer den `RenderScene`-Teil eines gekoppelten RenderFrames |
| `WgpuRenderCallback` | egui/wgpu-Glue, der den Host-Adapter in den Paint-Callback einhaengt |

## Oeffentliche Methoden

| Signatur | Zweck |
|---|---|
| `Renderer::new(render_state)` | Erstellt den Host-Adapter aus `egui_wgpu::RenderState` und leitet `target_format` an `RendererTargetConfig` weiter |
| `Renderer::render_scene(device, queue, render_pass, scene)` | Delegiert das eigentliche Zeichnen an den host-neutralen Kern |
| `Renderer::set_background(device, queue, image, world_bounds, scale)` | Laedt oder aktualisiert das Background-Asset im Render-Core |
| `Renderer::clear_background()` | Entfernt das aktuell hochgeladene Background-Asset |

## Beispiel

```rust
use std::sync::{Arc, Mutex};

let renderer = Arc::new(Mutex::new(render::Renderer::new(render_state)));
let frame = fs25_auto_drive_host_bridge::build_render_frame(
    &controller,
    &state,
    [viewport_w, viewport_h],
);
let render_data = render::WgpuRenderData { scene: frame.scene };

let callback = egui_wgpu::Callback::new_paint_callback(
    rect,
    render::WgpuRenderCallback {
        renderer: renderer.clone(),
        render_data,
        device: render_state.device.clone(),
        queue: render_state.queue.clone(),
    },
);

ui.painter().add(callback);
```

## Datenfluss

```mermaid
flowchart LR
    EDITOR[editor_app::EditorApp] --> CTRL[AppController]
    CTRL --> FRAME[HostRenderFrameSnapshot]
    FRAME --> SCENE[RenderScene]
    FRAME --> ASSETS[RenderAssetsSnapshot]
    SCENE --> DATA[WgpuRenderData]
  DATA --> CALLBACK[WgpuRenderCallback]
  CALLBACK --> HOST[render::Renderer]
  ASSETS --> SYNC[sync_background_upload()]
  SYNC --> HOST
  HOST --> CORE[fs25_auto_drive_render_wgpu::Renderer]
```

## Integrationsnotizen

- `render::Renderer` enthaelt keine eigene Fachlogik; der GPU-Kern bleibt in `fs25_auto_drive_render_wgpu`.
- `sync_background_upload()` lebt bewusst in `editor_app`, weil dort `Device`, `Queue`, die letzten Host-Revisionen und die Assets des bereits aufgebauten RenderFrames bereits vorliegen.
- Die Engine beschreibt Background-Bounds im Domain-System als `RenderBackgroundWorldBounds { min_x, max_x, min_z, max_z }`. Der egui-Host-Adapter mappt diese beim Upload auf `BackgroundWorldBounds { min_x, max_x, min_y, max_y }`, weil der Render-Core auf einer 2D-X/Y-Ebene arbeitet.
- Das egui-Onscreen-Rendering nutzt bewusst weder RGBA-Readback noch `CanvasRuntime`; es bleibt ein direkter `RenderScene`-Paint-Callback ueber `egui_wgpu`.
