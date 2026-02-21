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
// Node Instance Data
struct NodeInstance {
    position: [f32; 3],      // Welt-Position
    color: [f32; 4],         // RGBA
    node_type: u32,          // Regular, Parking, Warning, etc.
    selected: u32,           // 0 oder 1
}

// Connection Instance Data
struct ConnectionInstance {
    start: [f32; 3],
    end: [f32; 3],
    color: [f32; 4],
    connection_type: u32,    // Regular, Dual, Reverse
}
```

### Uniform-Buffer
```rust
struct CameraUniforms {
    view_proj: Mat4,         // View-Projection Matrix
    viewport_size: [f32; 2], // Für Pixel-genaue Berechnungen
    zoom_level: f32,         // Für LOD-Entscheidungen
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
```rust
// Berechne sichtbaren Bereich in Welt-Koordinaten
let viewport_bounds = camera.viewport_bounds();

// Filtere Nodes
let visible_nodes: Vec<&MapNode> = road_map.nodes
    .values()
    .filter(|node| viewport_bounds.contains(node.position))
    .collect();

// Update Instance-Buffer nur mit visible_nodes
```

## Performance-Budget
- **Node Rendering:** <5ms bei 100k Nodes
- **Connection Rendering:** <3ms bei 100k Connections
- **Culling:** <1ms
- **Gesamt Frame:** <16ms (60 FPS)
