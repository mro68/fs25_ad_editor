# Render API Documentation

## Überblick

Das `render`-Modul implementiert GPU-beschleunigtes Rendering mit wgpu. Importiert `Camera2D` und `RoadMap` aus `core` (keine Abhängigkeit auf `app`).

## Module

- `node_renderer.rs` — GPU-Instanced Nodes (Wegpunkte)
- `connection_renderer/` — Verbindungslinien + Richtungspfeile (Submodule: `culling.rs` Viewport-Culling-Geometrie, `mesh.rs` Vertex-Generierung)
- `background_renderer.rs` — Background-Map-Quad mit Texture-Sampling
- `marker_renderer.rs` — GPU-Instanced Map-Marker (Pin-Symbole)
- `texture.rs` — Texture-Erstellung aus DynamicImage (PNG/JPG/DDS)
- `callback.rs` — wgpu Render Callback für egui-Integration (`WgpuRenderCallback`, `WgpuRenderData`)
- `types.rs` — GPU-Typen (`Vertex`, `ConnectionVertex`, `NodeInstance`, `MarkerInstance`, `Uniforms`), View-Projection
- `shaders.wgsl` — WGSL Shader (Node, Connection, Background, Marker)

## Haupttypen

### `Renderer`

Haupt-Renderer mit eigenem State-Management.

```rust
pub struct Renderer { /* intern */ }
```

**Methoden:**

```rust
let renderer = Renderer::new(render_state);

renderer.render_scene(device, queue, render_pass, &scene);

renderer.set_background(device, queue, &bg_map);  // Background-Map setzen
renderer.clear_background();                       // Background-Map entfernen
```

**Parameter von `render_scene`:**
- `device: &wgpu::Device`
- `queue: &wgpu::Queue`
- `render_pass: &mut RenderPass<'static>`
- `scene: &RenderScene` — Render-Vertrag

---

### `RenderScene`

Expliziter Übergabevertrag zwischen App-Layer und Renderer. Definiert im `shared`-Modul (`shared::render_scene`).

```rust
pub struct RenderScene {
    pub road_map: Option<Arc<RoadMap>>,
    pub camera: Camera2D,
    pub viewport_size: [f32; 2],
    pub render_quality: RenderQuality,
    pub selected_node_ids: Vec<u64>,
    pub connect_source_node: Option<u64>,
    pub background_map: Option<Arc<BackgroundMap>>,
    pub background_opacity: f32,
    pub background_visible: bool,
    pub options: EditorOptions,
}
```

**Methoden:**
- `has_map() -> bool` — Prüft ob eine RoadMap vorhanden ist

---

### `NodeRenderer`

Interner Renderer für Nodes (Wegpunkte) mit GPU-Instancing.

**Features:**
- GPU-Instancing für 100k+ Nodes
- Shader-basierte kreisförmige Darstellung via Distance Field
- Adaptives Anti-Aliasing (Low/Medium/High via `RenderQuality`)
- Selektierte Nodes werden 1.8× größer dargestellt
- Farb-Coding nach NodeFlag:
  - Cyan `[0.0, 0.8, 1.0]`: Regular
  - Gelb `[1.0, 1.0, 0.0]`: SubPrio
  - Magenta `[1.0, 0.0, 1.0]`: Selected
  - Rot `[1.0, 0.0, 0.0]`: Warning

**Konstanten:**
- `NODE_SIZE_WORLD: 0.5` — Feste Node-Größe in Welt-Einheiten

---

### `ConnectionRenderer`

Interner Renderer für Verbindungslinien und Richtungspfeile.

**Features:**
- Linien als Quad-Geometrie mit konfigurierbarer Breite
- Richtungspfeile an Verbindungs-Mittelpunkten
- Farb-Coding nach Richtung:
  - Grün `[0.2, 0.9, 0.2]`: Regular
  - Blau `[0.2, 0.7, 1.0]`: Dual
  - Orange `[1.0, 0.5, 0.1]`: Reverse
- SubPriority-Verbindungen werden heller dargestellt

**Konstanten:**
- `CONNECTION_THICKNESS_WORLD: 0.2` (Hauptstraße)
- `CONNECTION_THICKNESS_SUBPRIO_WORLD: 0.1` (Nebenstraße)
- `ARROW_LENGTH_WORLD: 1.0`
- `ARROW_WIDTH_WORLD: 0.6`

---

### GPU-Typen (`types.rs`)

```rust
pub struct Vertex { pub position: [f32; 2] }
pub struct ConnectionVertex { pub position: [f32; 2], pub color: [f32; 4] }
pub struct NodeInstance {
    pub position: [f32; 2],
    pub base_color: [f32; 4],
    pub rim_color: [f32; 4],
    pub size: f32,
}
pub struct MarkerInstance { pub position: [f32; 2], pub color: [f32; 4], pub outline_color: [f32; 4], pub size: f32 }
pub struct Uniforms { pub view_proj: [[f32; 4]; 4], pub aa_params: [f32; 4] }
```

**Vertex-Buffer-Layouts:**
- `Vertex::desc()` / `ConnectionVertex::desc()` / `NodeInstance::desc()` / `MarkerInstance::desc()`

**Hilfsfunktion:**
- `build_view_projection(camera: &Camera2D, viewport_size: [f32; 2]) -> Mat4`

---

### `RenderContext` (crate-intern)

Bündelt die gemeinsamen Render-Parameter für alle Sub-Renderer. Vermeidet wiederholte Parameter-Listen.

```rust
pub(crate) struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub camera: &'a Camera2D,
    pub viewport_size: [f32; 2],
    pub options: &'a EditorOptions,
}
```

---

### `WgpuRenderCallback` / `WgpuRenderData`

Egui-Integration für wgpu-Rendering.

```rust
pub struct WgpuRenderData {
    pub scene: RenderScene,
}

pub struct WgpuRenderCallback {
    pub data: Arc<WgpuRenderData>,
}
```

---

### `RenderQuality`

Qualitätsstufe für Anti-Aliasing.

```rust
pub enum RenderQuality { Low, Medium, High }
```

- **Low:** Harte Kanten (`step`)
- **Medium:** Standard-AA (`fwidth * 1.0`)
- **High:** Breiteres AA (`fwidth * 1.8`)

---

### `RenderOptions`

Konfigurations-Optionen (für zukünftige Features).

```rust
pub struct RenderOptions {
    pub node_size: f32,           // Default: 5.0
    pub highlight_subprio: bool,  // Default: true
    pub highlight_warnings: bool, // Default: true
}
```

## Rendering-Pipeline

### View-Projektion-Aufbau

```rust
// View-Matrix: nur Translation (Camera2D.view_matrix())
let view = Mat3::from_translation(-camera.position);

// Orthographische Projektion: Zoom über Extent-Skalierung
let zoom_scale = 1.0 / camera.zoom;
let projection = Mat4::orthographic_rh(
    -BASE_WORLD_EXTENT * aspect * zoom_scale,
     BASE_WORLD_EXTENT * aspect * zoom_scale,
     BASE_WORLD_EXTENT * zoom_scale,         // bottom (z-)
    -BASE_WORLD_EXTENT * zoom_scale,         // top (z+)
    -1.0, 1.0
);

let view_proj = projection * view_as_mat4;
```

**Wichtig:** Zoom wird ausschließlich über die Projektion gesteuert, nicht über die View-Matrix.

### Draw-Calls

- **Nodes:** 1 instanced Draw-Call für alle Nodes (6 Vertices × N Instanzen)
- **Connections:** 1 Draw-Call mit vorgenerierter Triangle-Geometrie

### `BackgroundRenderer`

Interner Renderer für Map-Hintergrund.

**Features:**
- Textured Quad in Weltkoordinaten
- Opacity-Steuerung (0.0–1.0)
- Sichtbarkeits-Toggle
- Unterstützt PNG, JPG, DDS via `image` Crate

---

### `MarkerRenderer`

Interner Renderer für Map-Marker (Pin-Symbole) mit GPU-Instancing.

**Features:**
- GPU-Instancing für beliebig viele Marker
- Pin-Form via SDF (Kreis + Träne) im Fragment-Shader
- Pin-Spitze steht exakt auf dem Node-Zentrum
- Outline-Farbe für bessere Sichtbarkeit

**Konstanten:**
- `MARKER_SIZE_WORLD: 2.0` — Pin-Höhe in Welt-Einheiten
- `MARKER_COLOR: [0.9, 0.1, 0.1]` — Rot
- `MARKER_OUTLINE_COLOR: [0.6, 0.0, 0.0]` — Dunkles Rot

---

## Render-Reihenfolge

1. **Background** — Map-Hintergrund (optional)
2. **Marker** — Pin-Symbole (hinter Nodes/Connections)
3. **Connections** — Verbindungslinien + Pfeile
4. **Nodes** — Wegpunkte (zuoberst)

## Design-Prinzipien

1. **State-Management:** Renderer verwaltet alle GPU-Ressourcen selbst
2. **Render Contract:** Nimmt nur `RenderScene`-Referenz
3. **Layered Rendering:** Background → Marker → Connections → Nodes
4. **Kein App-Import:** Bezieht `Camera2D` und `RoadMap` aus `core`
