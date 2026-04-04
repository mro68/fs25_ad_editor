# API des render-Moduls

## Ueberblick

Das `render`-Modul im egui-Frontend ist seit dem Renderer-Split nur noch ein Host-Adapter.
Die eigentliche GPU-Implementierung liegt in `fs25_auto_drive_render_wgpu`; dieses Modul
kapselt die egui-spezifische Callback-Integration und reicht Aufrufe an den host-neutralen
Renderer-Kern weiter.

## Module

- `mod.rs` — duenne Adapter-Fassade (`Renderer`) ueber `fs25_auto_drive_render_wgpu::Renderer`
- `callback.rs` — egui-spezifischer `CallbackTrait`-Adapter (`WgpuRenderCallback`)

## Haupttypen

### `Renderer`

Egui-Host-Adapter auf den host-neutralen Renderer-Kern.

```rust
pub struct Renderer { /* intern: core renderer */ }
```

**Methoden:**

```rust
let renderer = Renderer::new(render_state);
renderer.render_scene(device, queue, render_pass, &scene);
renderer.set_background(device, queue, image, world_bounds, scale);
renderer.clear_background();
```

- `new(render_state)` liest `target_format` aus dem egui-Host und baut daraus
  `RendererTargetConfig` fuer den Core-Renderer.
- `render_scene(...)` delegiert 1:1 an den Core-Renderer.
- `set_background(...)` und `clear_background()` bleiben als stabile Adapter-API erhalten.

### `WgpuRenderCallback` / `WgpuRenderData`

Egui-Integration fuer den Paint-Lifecycle.

```rust
pub struct WgpuRenderData {
    pub scene: RenderScene,
}

pub struct WgpuRenderCallback {
    pub renderer: Arc<Mutex<Renderer>>,
    pub render_data: WgpuRenderData,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}
```

`WgpuRenderCallback` implementiert `egui_wgpu::CallbackTrait` und ruft im
`paint()`-Pfad den Adapter-Renderer auf.

## Re-Exports

Das Modul re-exportiert weiterhin die zentralen Render-Typen:

- `RenderScene`
- `RenderQuality`
- `BackgroundWorldBounds`
- `RendererTargetConfig`

Damit bleiben bestehende Call-Sites im egui-Frontend und in Root-Reexports stabil,
waehrend die Render-Kernlogik in der neuen Crate lebt.

## Design-Prinzipien

1. Egui-spezifischer Code bleibt lokal (`CallbackTrait`, Paint-Lifecycle)
2. GPU-Logik liegt nur im host-neutralen Core (`fs25_auto_drive_render_wgpu`)
3. Adapter-Schicht bleibt klein und ohne eigene Render-Hotpath-Logik
