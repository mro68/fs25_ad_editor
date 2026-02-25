# Shared API Documentation

## Überblick

Das `shared`-Modul enthält Layer-übergreifende Typen, die zwischen `app` (Produzent) und `render` (Konsument) geteilt werden, um direkte Abhängigkeiten zwischen diesen Schichten zu vermeiden.

## Module

- `render_scene.rs` — `RenderScene` Übergabevertrag App → Render
- `render_quality.rs` — `RenderQuality` Enum (Low/Medium/High)
- `options.rs` — Zentrale Konfigurationskonstanten + `EditorOptions` (Laufzeit-Optionen)
- `spline_geometry.rs` — Layer-neutrale Catmull-Rom-Geometrie-Funktionen (kein import aus `tools` nötig)

## Haupttypen

### `RenderScene`

Expliziter, read-only Übergabevertrag zwischen App-Layer und Renderer.

```rust
pub struct RenderScene {
    pub road_map: Option<Arc<RoadMap>>,
    pub camera: Camera2D,
    pub viewport_size: [f32; 2],
    pub render_quality: RenderQuality,
    pub selected_node_ids: Arc<HashSet<u64>>,
    pub connect_source_node: Option<u64>,
    pub background_map: Option<Arc<BackgroundMap>>,
    pub background_opacity: f32,
    pub background_visible: bool,
    pub options: EditorOptions,
    pub hidden_node_ids: Arc<HashSet<u64>>,
}
```

`hidden_node_ids` wird genutzt, um Nodes im aktuellen Frame temporär auszublenden
(z. B. bei Vorschau-Overlays im Properties-Panel), ohne die Domain-Daten zu mutieren.

**Methoden:**
- `has_map() -> bool` — Prüft ob eine RoadMap vorhanden ist

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

### Konfigurationskonstanten (`options.rs`)

Zentral gesammelte Konfigurationswerte, gegliedert nach Bereich:

| Bereich | Konstante | Wert | Beschreibung |
|---------|-----------|------|-------------|
| Kamera | `CAMERA_BASE_WORLD_EXTENT` | 2048.0 | Sichtbare Welt-Halbbreite bei Zoom 1.0 (Referenz-Duplikat, kanonisch in `Camera2D`) |
| Kamera | `CAMERA_ZOOM_MIN` / `MAX` | 0.1 / 100.0 | Zoom-Grenzen (Referenz-Duplikat) |
| Kamera | `CAMERA_ZOOM_STEP` | 1.2 | Zoom-Schritt bei Menü-Buttons / Shortcuts |
| Kamera | `CAMERA_SCROLL_ZOOM_STEP` | 1.1 | Zoom-Schritt bei Mausrad-Scroll |
| Selektion | `SELECTION_PICK_RADIUS_PX` | 12.0 | Maus-Fangradius in Pixeln |
| Selektion | `SELECTION_SIZE_FACTOR` | 1.8 | Vergrößerung selektierter Nodes |
| Nodes | `NODE_SIZE_WORLD` | 0.5 | Feste Node-Größe in Welt-Einheiten |
| Nodes | `NODE_COLOR_DEFAULT` | `[0.0, 0.8, 1.0, 1.0]` | Cyan (Regular) |
| Nodes | `NODE_COLOR_SUBPRIO` | `[1.0, 1.0, 0.0, 1.0]` | Gelb (SubPrio) |
| Nodes | `NODE_COLOR_SELECTED` | `[1.0, 0.0, 1.0, 1.0]` | Magenta (Selektiert) |
| Nodes | `NODE_COLOR_WARNING` | `[1.0, 0.0, 0.0, 1.0]` | Rot (Warning) |
| Connections | `CONNECTION_THICKNESS_WORLD` | 0.2 | Hauptstraßen-Linienbreite |
| Connections | `CONNECTION_THICKNESS_SUBPRIO_WORLD` | 0.1 | Nebenstraßen-Linienbreite |
| Connections | `ARROW_LENGTH_WORLD` / `WIDTH` | 1.0 / 0.6 | Pfeilgeometrie |
| Connections | `CONNECTION_COLOR_REGULAR` | `[0.2, 0.9, 0.2, 1.0]` | Grün |
| Connections | `CONNECTION_COLOR_DUAL` | `[0.2, 0.7, 1.0, 1.0]` | Blau |
| Connections | `CONNECTION_COLOR_REVERSE` | `[1.0, 0.5, 0.1, 1.0]` | Orange |
| Marker | `MARKER_SIZE_WORLD` | 2.0 | Pin-Höhe in Welt-Einheiten |
| Marker | `MARKER_COLOR` | `[0.9, 0.1, 0.1, 1.0]` | Rot |
| Marker | `MARKER_OUTLINE_COLOR` | `[0.6, 0.0, 0.0, 1.0]` | Dunkles Rot |
| Tools | `SNAP_RADIUS` | 3.0 | Snap-Radius für Route-Tools (Welteinheiten) |
| Tools | `HITBOX_SCALE_PERCENT` | 100.0 | Standard-Hitbox-Skalierung in % der Node-Größe |
| Terrain | `TERRAIN_HEIGHT_SCALE` | 255.0 | Höhenskala für Heightmap-Export |

### `OverviewLayerOptions`

Konfigurierbare Layer-Optionen für die Übersichtskarten-Generierung.
Wird als Teil der `EditorOptions` persistent in TOML gespeichert.

```rust
pub struct OverviewLayerOptions {
    pub hillshade: bool,
    pub farmlands: bool,
    pub farmland_ids: bool,
    pub pois: bool,
    pub legend: bool,
}
```

Der Default setzt alle Layer außer `legend` auf `true`.

## Design-Prinzipien

1. **Entkopplung:** `shared` verhindert direkte Abhängigkeiten zwischen `app` und `render`
2. **Single Source of Truth:** Alle Rendering-Konstanten in `options.rs` zentralisiert
3. **Immutable Contract:** `RenderScene` ist read-only (Clone, keine Mutation)

---

### `EditorOptions` (Laufzeit-Optionen)

Alle zur Laufzeit änderbaren Editor-Optionen. Wird als `fs25_auto_drive_editor.toml` neben der Binary gespeichert.

```rust
pub struct EditorOptions {
    // Nodes
    pub node_size_world: f32,
    pub node_color_default: [f32; 4],
    pub node_color_subprio: [f32; 4],
    pub node_color_selected: [f32; 4],
    pub node_color_warning: [f32; 4],
    // Selektion
    pub selection_size_factor: f32,
    pub selection_pick_radius_px: f32,
    // Connections
    pub connection_thickness_world: f32,
    pub connection_thickness_subprio_world: f32,
    pub arrow_length_world: f32,
    pub arrow_width_world: f32,
    pub connection_color_regular: [f32; 4],
    pub connection_color_dual: [f32; 4],
    pub connection_color_reverse: [f32; 4],
    // Marker
    pub marker_size_world: f32,
    pub marker_color: [f32; 4],
    pub marker_outline_color: [f32; 4],
    // Kamera
    pub camera_zoom_step: f32,
    pub camera_scroll_zoom_step: f32,
    // Tools
    pub snap_radius: f32,
    /// Hitbox-Skalierung in Prozent der Node-Größe (100 = exakte Node-Größe)
    pub hitbox_scale_percent: f32,
    /// true = Mittelpunkt zwischen Vorgänger und Nachfolger beim Löschen verbinden
    pub reconnect_on_delete: bool,
    /// true = bestehende Verbindung beim Platzieren splitten
    pub split_connection_on_place: bool,
    // Kamera (erweitert)
    /// Minimaler Zoom-Faktor (konfig, überschreibt Camera2D::ZOOM_MIN)
    pub camera_zoom_min: f32,
    /// Maximaler Zoom-Faktor (konfig, überschreibt Camera2D::ZOOM_MAX)
    pub camera_zoom_max: f32,
    // Background-Map
    /// Standard-Deckung für Background-Map (0.0–1.0)
    pub background_opacity_default: f32,
    /// Deckung bei minimalem Zoom (0.0–1.0)
    pub background_opacity_at_min_zoom: f32,
    // Terrain
    pub terrain_height_scale: f32,
    // Übersichtskarte
    /// Layer-Optionen für Übersichtskarten-Generierung
    pub overview_layers: OverviewLayerOptions,
}
```

**Methoden:**
- `EditorOptions::load_from_file(path) -> Self` — TOML-Datei laden (bei Fehler: Defaults)
- `EditorOptions::save_to_file(&self, path) -> Result<()>` — Als TOML speichern
- `EditorOptions::config_path() -> PathBuf` — Pfad zur Optionen-Datei neben der Binary
- `hitbox_radius(&self) -> f32` — Berechnet den Hitbox-Radius in Welteinheiten (`node_size_world * hitbox_scale_percent / 100`)
