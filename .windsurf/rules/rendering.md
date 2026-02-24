# Rendering-Architektur

## Ziel
Flüssiges Rendering von 100k+ Punkten und Verbindungen auf der GPU.

## Strategie
1. **GPU-Instancing:** Ein Draw-Call für alle Nodes, ein Draw-Call für alle Connections
2. **Viewport-Culling:** Nur Elemente im sichtbaren Bereich rendern
3. **LOD (Level of Detail):** Zoom-abhängige Darstellung (Icons vs. einfache Kreise)

## wgpu Pipeline

### Vertex-Buffer-Layout
```rust
// Node-Instanz-Daten (GPU-Instancing pro sichtbarem Node)
struct NodeInstance {
    position: [f32; 2],      // 2D Welt-Position (x, z)
    base_color: [f32; 4],    // RGBA Mitte des Nodes
    rim_color: [f32; 4],     // RGBA Außenring (Selection/Flag)
    size: f32,               // Größe in Welteinheiten
    _padding: [f32; 1],
}

// Connection-Vertex (Linien + Pfeile, pre-transformiert auf CPU)
struct ConnectionVertex {
    position: [f32; 2],      // 2D Position
    color: [f32; 4],         // RGBA der Verbindung
}

// Marker-Instanz (Pin-Symbol für Destinations)
struct MarkerInstance {
    position: [f32; 2],
    color: [f32; 4],
    outline_color: [f32; 4],
    size: f32,
    _padding: [f32; 1],
}
```

### Uniform-Buffer
```rust
struct Uniforms {
    view_proj: [[f32; 4]; 4], // View-Projection Matrix
    aa_params: [f32; 4],      // Anti-Aliasing-Parameter
}
```

## Integration in egui
- Verwende `egui::PaintCallback` für Custom-Rendering
- Render-Zyklus:
  1. egui zeichnet UI-Elemente
  2. PaintCallback ruft wgpu-Renderer auf
  3. wgpu rendert in dieselbe Render-Target

## Shader-Logik (WGSL)

### Node Vertex Shader
- Nimmt instanzierte Quad-Vertices
- Transformiert mit Camera-Matrix
- Leitet Node-Typ weiter für Fragment Shader

### Node Fragment Shader
- SDF (Signed Distance Field) für Kreise
- Färbung basierend auf Node-Typ und Selection
- Optional: Texture-Lookup für Icons

### Connection Vertex Shader
- Generiert Linie mit Thickness (über Geometry Expansion)
- Erzeugt Pfeilspitzen für Richtung

## Culling-Strategie

**Nodes:** KD-Tree-basiert via `SpatialIndex::within_rect()` — nur Viewport-sichtbare Nodes.

**Connections:** Aktuell lineare Iteration über alle Connections mit Segment-Rect-Intersection-Test.
Verbesserungspotential: Spatial-Vorfilter (z.B. Midpoint-KD-Tree) für O(k) statt O(n).

### Scratch-Buffer-Pattern (alle Renderer)
```rust
// Vermeidet Heap-Allokation pro Frame
let mut scratch = std::mem::take(&mut self.instance_scratch);
scratch.clear();
// ... befülle scratch ...
self.instance_scratch = scratch; // Kapazität bleibt erhalten
```

### Viewport-Berechnung
```rust
// compute_visible_rect() in render/types.rs
let (min, max) = compute_visible_rect(&render_context);
// Gibt AABB mit 8px Padding in Weltkoordinaten zurück
```

## Performance-Budget
- **Node Rendering:** <5ms bei 100k Nodes
- **Connection Rendering:** <3ms bei 100k Connections
- **Culling:** <1ms
- **Gesamt Frame:** <16ms (60 FPS)
