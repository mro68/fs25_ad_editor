# Render API Documentation

## Ueberblick

Das `render`-Modul implementiert GPU-beschleunigtes Rendering mit wgpu. Importiert `Camera2D` und `RoadMap` aus `core` (keine Abhaengigkeit auf `app`).

## Module

- `node_renderer.rs` — GPU-Instanced Nodes (Wegpunkte)
- `connection_renderer/` — Verbindungslinien + Richtungspfeile (Submodule: `culling.rs` Viewport-Culling-Geometrie, `mesh.rs` Vertex-Generierung)
- `background_renderer.rs` — Background-Map-Quad mit Texture-Sampling
- `marker_renderer.rs` — GPU-Instanced Map-Marker (Pin-Symbole)
- `texture.rs` — Texture-Erstellung aus DynamicImage (PNG/JPG/DDS)
- `callback.rs` — wgpu Render Callback fuer egui-Integration (`WgpuRenderCallback`, `WgpuRenderData`)
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

renderer.set_background(device, queue, &bg_map, scale);  // Background-Map setzen (scale: f32)
renderer.clear_background();                       // Background-Map entfernen
```

**Parameter von `render_scene`:**
- `device: &wgpu::Device`
- `queue: &wgpu::Queue`
- `render_pass: &mut RenderPass<'static>`
- `scene: &RenderScene` — Render-Vertrag

---

### `RenderScene`

Expliziter Uebergabevertrag zwischen App-Layer und Renderer. Definiert im `shared`-Modul (`shared::render_scene`).

```rust
pub struct RenderScene {
    pub road_map: Option<Arc<RoadMap>>,
    pub camera: Camera2D,
    pub viewport_size: [f32; 2],
    pub render_quality: RenderQuality,
    pub selected_node_ids: Arc<IndexSet<u64>>,
    pub connect_source_node: Option<u64>,
    pub background_map: Option<Arc<BackgroundMap>>,
    pub background_visible: bool,
    pub options: Arc<EditorOptions>,
    pub hidden_node_ids: Arc<IndexSet<u64>>,
}
```

`hidden_node_ids` erlaubt dem App-Layer, Nodes pro Frame selektiv auszublenden
(z. B. Original-Node-Pfad waehrend einer Distanzen-Vorschau), ohne Aenderungen an `RoadMap`.

**Methoden:**
- `has_map() -> bool` — Prueft ob eine RoadMap vorhanden ist

---

### `NodeRenderer`

Interner Renderer fuer Nodes (Wegpunkte) mit GPU-Instancing.

**Features:**
- GPU-Instancing fuer 100k+ Nodes
- Shader-basierte kreisfoermige Darstellung via Distance Field
- Adaptives Anti-Aliasing (Low/Medium/High via `RenderQuality`)
- Selektierte Nodes werden 1.8× groesser dargestellt- **Zoom-Kompensation:** Node-Größe wird via `EditorOptions::zoom_compensation(zoom)` skaliert — verhindert, dass Nodes beim Herauszoomen unsichtbar werden- Farb-Coding nach NodeFlag:
  - Cyan `[0.0, 0.8, 1.0]`: Regular
  - Gelb `[1.0, 1.0, 0.0]`: SubPrio
  - Magenta `[1.0, 0.0, 1.0]`: Selected
  - Rot `[1.0, 0.0, 0.0]`: Warning

**Konstanten:**
- `NODE_SIZE_WORLD: 0.5` — Feste Node-Groesse in Welt-Einheiten

---

### `ConnectionRenderer`

Interner Renderer fuer Verbindungslinien und Richtungspfeile.

**Features:**
- Linien als Quad-Geometrie mit konfigurierbarer Breite
- Richtungspfeile an Verbindungs-Mittelpunkten
- **Zoom-Kompensation:** Linienbreite und Pfeilgrößen werden via `EditorOptions::zoom_compensation(zoom)` skaliert — konsistent mit Node-Skalierung
- Farb-Coding nach Richtung:
  - Gruen `[0.2, 0.9, 0.2]`: Regular
  - Blau `[0.2, 0.7, 1.0]`: Dual
  - Orange `[1.0, 0.5, 0.1]`: Reverse
- SubPriority-Verbindungen werden heller dargestellt

**Konstanten:**
- `CONNECTION_THICKNESS_WORLD: 0.1` (Hauptstrasse)
- `CONNECTION_THICKNESS_SUBPRIO_WORLD: 0.05` (Nebenstrasse)
- `ARROW_LENGTH_WORLD: 1.0`
- `ARROW_WIDTH_WORLD: 0.5`

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

**Hilfsfunktionen (`pub(crate)`):**
- `build_view_projection(camera: &Camera2D, viewport_size: [f32; 2]) -> Mat4` — Orthographische View-Projection-Matrix (Zoom ausschliesslich ueber Extent-Skalierung, nicht View-Matrix)
- `compute_visible_rect(ctx: &RenderContext) -> (Vec2, Vec2)` — Berechnet die sichtbare Welt-AABB mit 8-Pixel-Padding fuer Viewport-Culling. Gibt `(min, max)` in Weltkoordinaten zurueck.

---

### `RenderContext` (crate-intern)

Buendelt die gemeinsamen Render-Parameter fuer alle Sub-Renderer. Vermeidet wiederholte Parameter-Listen.

```rust
pub(crate) struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub camera: &'a Camera2D,
    pub viewport_size: [f32; 2],
    pub options: &'a EditorOptions,
    /// Node-IDs, die in diesem Frame ausgeblendet werden sollen
    pub hidden_node_ids: &'a IndexSet<u64>,
    /// Node-IDs, die mit 50% Opacity gerendert werden sollen (gedimmte Segment-Nodes)
    pub dimmed_node_ids: &'a IndexSet<u64>,
}
```

---

### `WgpuRenderCallback` / `WgpuRenderData`

Egui-Integration fuer wgpu-Rendering.

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

---

### `RenderQuality`

Qualitaetsstufe fuer Anti-Aliasing.

```rust
pub enum RenderQuality { Low, Medium, High }
```

- **Low:** Harte Kanten (`step`)
- **Medium:** Standard-AA (`fwidth * 1.0`)
- **High:** Breiteres AA (`fwidth * 1.8`)

---

## Rendering-Pipeline

### View-Projektion-Aufbau

```rust
// View-Matrix: nur Translation (Camera2D.view_matrix())
let view = Mat3::from_translation(-camera.position);

// Orthographische Projektion: Zoom ueber Extent-Skalierung
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

**Wichtig:** Zoom wird ausschliesslich ueber die Projektion gesteuert, nicht ueber die View-Matrix.

### Draw-Calls

- **Nodes:** 1 instanced Draw-Call fuer alle Nodes (6 Vertices × N Instanzen)
- **Connections:** 1 Draw-Call mit vorgenerierter Triangle-Geometrie

### `BackgroundRenderer`

Interner Renderer fuer Map-Hintergrund.

**Features:**
- Textured Quad in Weltkoordinaten
- Opacity-Steuerung (0.0–1.0)
- Sichtbarkeits-Toggle
- Unterstuetzt PNG, JPG, DDS via `image` Crate

---

### `MarkerRenderer`

Interner Renderer fuer Map-Marker (Pin-Symbole) mit GPU-Instancing und texturbasiertem Rendering.

**Features:**
- GPU-Instancing fuer beliebig viele Marker
- Pin-Symbol als SVG-Textur: `icon_map_pin.svg` wird per `resvg` zur Laufzeit in 64×64 RGBA gerastert
- Strichdicke (`stroke-width`) wird aus `EditorOptions::marker_outline_width` skaliert und direkt im SVG gepatcht
- Neu-Rasterisierung nur bei Aenderung von `outline_width` (Change-Detection via `last_outline_width`)
- Fragment-Shader `fs_marker` nutzt `textureSample` — Textur-Alpha definiert die Pin-Form
- Instanz-Tinting: `instance_color` faerbt den Pin, Textur liefert nur Alpha-Maske
- Pin-Spitze steht exakt auf dem Node-Zentrum (Y-Offset im Vertex-Shader: `−0.8 × size`)
- BindGroup: Binding 0 = Uniform-Buffer, Binding 1 = `marker_texture`, Binding 2 = `marker_sampler`
- Groesse und Farbe kommen aus `EditorOptions` (`marker_size_world`, `marker_color`, `marker_outline_color`)
- Zoom-kompensierte Skalierung und konfigurierbare Mindestgroesse (`min_marker_size_px`)

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
