# Core API Documentation

## Überblick

Das `core`-Modul enthält die zentralen Datenstrukturen für AutoDrive-Konfigurationen und Geometrie-Typen. Keine Abhängigkeiten zu UI, Render oder App.

## Haupttypen

### `Camera2D`

2D-Kamera mit Pan und Zoom. Reiner Geometrie-Typ ohne I/O-Abhängigkeiten.

```rust
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
}
```

**Konstanten:**
- `BASE_WORLD_EXTENT: f32 = 2048.0` — Basis-Weltgröße für Projektion
- `ZOOM_MIN: f32 = 0.1` — Minimaler Zoom
- `ZOOM_MAX: f32 = 100.0` — Maximaler Zoom

**Methoden:**

```rust
let mut camera = Camera2D::new();

// Position setzen
camera.look_at(Vec2::new(100.0, 200.0));

// Pan (Verschieben in Welt-Einheiten)
camera.pan(Vec2::new(10.0, 5.0));

// Zoom (clamped: 0.1 bis 100.0)
camera.zoom_by(1.5);  // vergrößern
camera.zoom_by(0.5);  // verkleinern

// Zoom mit benutzerdefinierten Grenzen clamped
camera.zoom_by_clamped(1.5, 0.2, 50.0);  // custom min/max

// Zoom auf gegebene Grenzen clampen
camera.clamp_zoom(0.2, 50.0);

// View-Matrix für Shader (nur Translation, kein Scale)
let mat = camera.view_matrix(); // Mat3

// Screen zu Welt-Koordinaten (berücksichtigt BASE_WORLD_EXTENT + Zoom)
let world_pos = camera.screen_to_world(
    Vec2::new(mouse_x, mouse_y),
    Vec2::new(screen_w, screen_h)
);

// Pixel-zu-Welt-Umrechnungsfaktor
let wpp = camera.world_per_pixel(viewport_height);

// Welt zu Screen-Koordinaten (Inverse von screen_to_world)
let screen_pos = camera.world_to_screen(
    Vec2::new(world_x, world_y),
    Vec2::new(screen_w, screen_h)
);

// Pick-Radius in Welt-Einheiten (für Node-Selektion)
let pick_radius = camera.pick_radius_world(viewport_height, pick_radius_px);

// Pick-Radius skaliert (fester Wert bei ZOOM_MAX, unabhängig vom aktuellen Zoom)
let scaled = camera.pick_radius_world_scaled(viewport_height, pick_radius_px);
```

**View-Matrix:** Enthält nur Translation (`-position`). Zoom wird ausschließlich über die orthographische Projektion im Renderer gesteuert.

**Screen-to-World:** Berücksichtigt `BASE_WORLD_EXTENT`, Zoom und Aspekt-Ratio:
```
world = NDC * BASE_WORLD_EXTENT * aspect / zoom + position
```

**Pick-Radius:** Konvertiert den übergebenen Pixel-Radius in Welt-Koordinaten basierend auf Zoom und Viewport-Höhe. Der Pixel-Wert (`SELECTION_PICK_RADIUS_PX`) lebt in `shared::options`, damit `core` keine Abhängigkeit auf `shared` hat.

**Pick-Radius (skaliert):** `pick_radius_world_scaled()` gibt einen fixen Weltradius zurück, berechnet bei `ZOOM_MAX` — damit bleibt der Pick-Bereich unabhängig vom aktuellen Zoom konsistent.

---

### `BackgroundMap`

Lädt Bilder (PNG, JPG, DDS) als Map-Hintergrund und stellt sie für GPU-Rendering bereit.
Unterstützt auch das Laden aus ZIP-Archiven.

```rust
pub struct BackgroundMap { /* intern */ }
```

**Methoden:**
- `BackgroundMap::load_from_file(path, crop_size) -> Result<Self>` — Bild laden, optional Center-Crop
- `BackgroundMap::from_image(image, source_label, crop_size) -> Result<Self>` — `pub(crate)`: BackgroundMap aus bereits dekodiertem `DynamicImage` erstellen (für Overview-Generator u.a.)
- `image_data() -> &DynamicImage` — Bilddaten
- `world_bounds() -> &WorldBounds` — Weltkoordinaten-Bereich
- `opacity() -> f32` — Aktuelle Opacity
- `set_opacity(opacity)` — Opacity setzen (clamped 0.0–1.0)
- `dimensions() -> (u32, u32)` — Bildgröße in Pixeln

**Freie Funktionen (ZIP-Support):**
- `list_images_in_zip(zip_path) -> Result<Vec<ZipImageEntry>>` — Bilddateien (png/jpg/jpeg/dds) im ZIP auflisten
- `load_from_zip(zip_path, entry_name, crop_size) -> Result<BackgroundMap>` — Bild aus ZIP in-memory extrahieren und als BackgroundMap laden

### `ZipImageEntry`

Beschreibt eine Bilddatei innerhalb eines ZIP-Archivs.

```rust
pub struct ZipImageEntry {
    pub name: String,  // Dateiname im Archiv (inkl. Pfad)
    pub size: u64,     // Unkomprimierte Dateigröße in Bytes
}
```

---

### `RoadMap`

Container für das gesamte AutoDrive-Straßennetzwerk.

```rust
pub struct RoadMap {
    pub nodes: HashMap<u64, MapNode>,
    connections: HashMap<(u64, u64), Connection>,  // Privat, Zugriff über connections_iter()
    pub map_markers: Vec<MapMarker>,
    pub meta: AutoDriveMeta,
    pub version: u32,
    pub map_name: Option<String>,
}
```

**Methoden:**
- `new(version: u32) -> Self` — Erstellt leere RoadMap
- `add_node(&mut self, node: MapNode)` — Fügt Node hinzu (markiert Spatial-Index als dirty)
- `remove_node(&mut self, node_id: u64) -> Option<MapNode>` — Entfernt Node + betroffene Verbindungen
- `update_node_position(&mut self, node_id: u64, new_position: Vec2) -> bool` — Position aktualisieren (baut Geometrie neu, markiert Spatial als dirty)
- `add_connection(&mut self, connection: Connection)` — Fügt Verbindung hinzu
- `has_connection(&self, start_id: u64, end_id: u64) -> bool` — Prüft ob Verbindung existiert
- `find_connection(&self, start_id: u64, end_id: u64) -> Option<&Connection>` — Findet exakte Verbindung
- `find_connections_between(&self, node_a: u64, node_b: u64) -> Vec<&Connection>` — Alle Verbindungen zwischen zwei Nodes (beide Richtungen)
- `remove_connection(&mut self, start_id: u64, end_id: u64) -> bool` — Entfernt exakte Verbindung
- `remove_connections_between(&mut self, node_a: u64, node_b: u64) -> usize` — Entfernt alle Verbindungen zwischen zwei Nodes
- `invert_connection(&mut self, start_id: u64, end_id: u64) -> bool` — Invertiert Start/End einer Verbindung
- `set_connection_direction(&mut self, start_id: u64, end_id: u64, direction) -> bool` — Richtung ändern
- `set_connection_priority(&mut self, start_id: u64, end_id: u64, priority) -> bool` — Priorität ändern
- `connections_iter(&self) -> impl Iterator<Item = &Connection>` — Iterator über alle Verbindungen
- `connected_neighbors(&self, node_id: u64) -> Vec<ConnectedNeighbor>` — Alle Nachbarn eines Nodes mit Richtung und Winkel
- `next_node_id(&self) -> u64` — Nächste freie Node-ID
- `add_map_marker(&mut self, marker: MapMarker)` — Fügt Marker hinzu
- `has_marker(&self, node_id: u64) -> bool` — Prüft ob Node einen Marker hat
- `find_marker_by_node_id(&self, node_id: u64) -> Option<&MapMarker>` — Marker eines Nodes finden
- `remove_marker(&mut self, node_id: u64) -> bool` — Marker eines Nodes entfernen
- `rebuild_connection_geometry(&mut self)` — Aktualisiert Connection-Geometrie
- `recalculate_node_flags(&mut self, node_ids: &[u64])` — NodeFlags basierend auf Verbindungsprioriäten neu berechnen
- `ensure_spatial_index(&mut self)` — Baut Spatial-Index nur auf, wenn dirty-Flag gesetzt ist (lazy rebuild)
- `build_spatial_index(&self) -> SpatialIndex` — Erstellt neuen Spatial-Index aus aktuellen Nodes
- `rebuild_spatial_index(&mut self)` — Baut den internen Spatial-Index sofort neu auf
- `node_count() -> usize` / `connection_count() -> usize` / `marker_count() -> usize`
- `count_duplicates(&self, epsilon: f32) -> (u32, u32)` — Zählt Duplikat-Nodes und -Gruppen
- `deduplicate_nodes(&mut self, epsilon: f32) -> DeduplicationResult` — Entfernt Duplikat-Nodes und verbindet Referenzen um

**Spatial Queries (persistenter KD-Tree, lazy rebuild via `ensure_spatial_index`):**
- `nearest_node(&self, query: Vec2) -> Option<SpatialMatch>` — Nächster Node
- `nodes_within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch>` — Nodes im Umkreis
- `nodes_within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64>` — Nodes im Rechteck

---

### `ConnectedNeighbor`

Beschreibt einen über eine Verbindung erreichbaren Nachbar-Node.

```rust
pub struct ConnectedNeighbor {
    pub neighbor_id: u64,
    pub angle: f32,       // Winkel der Verbindung (Radiant, atan2)
    pub is_outgoing: bool, // true = Verbindung geht vom Quell-Node zum Nachbar
}
```

---

### `DeduplicationResult`

Ergebnis einer Duplikat-Bereinigung.

```rust
pub struct DeduplicationResult {
    pub removed_nodes: u32,
    pub remapped_connections: u32,
    pub removed_self_connections: u32,
    pub remapped_markers: u32,
    pub duplicate_groups: u32,
}
```

**Methoden:**
- `had_duplicates() -> bool` — Gibt `true` zurück wenn Duplikate gefunden und entfernt wurden

---

### `MapNode`

Einzelner Wegpunkt im Netzwerk.

```rust
pub struct MapNode {
    pub id: u64,
    pub position: Vec2,  // 2D-Position (x, z)
    pub flag: NodeFlag,
}
```

**Methoden:**
- `MapNode::new(id, position, flag) -> Self` — Erstellt neuen Node

**NodeFlag-Varianten:**
- `Regular` (0) — Normaler Wegpunkt (Hauptstraße)
- `SubPrio` (1) — Nebenstraße (nur SubPriority-Verbindungen)
- `AutoGenerated` (2) — Wird beim Import zu Regular konvertiert
- `Reserved` (3) — Reserviert
- `SplineGenerated` (4) — Wird beim Import zu Regular konvertiert
- `Warning` (5) — Warnung

**NodeFlag-Konvertierung:**
- `NodeFlag::from_u32(value) -> Self` — Zahl zu Flag (2/4 werden zu Regular konvertiert)
- `NodeFlag::to_u32(self) -> u32` — Flag zu Zahl

---

### `Connection`

Verbindung zwischen zwei Nodes.

```rust
pub struct Connection {
    pub start_id: u64,
    pub end_id: u64,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub midpoint: Vec2,
    pub angle: f32,
}
```

**ConnectionDirection:** `Regular`, `Dual`, `Reverse`
**ConnectionPriority:** `Regular`, `SubPriority`

---

### `SpatialIndex` / `SpatialMatch`

KD-Tree-basierter Spatial Index (kiddo).

```rust
pub struct SpatialIndex { /* intern */ }

pub struct SpatialMatch {
    pub node_id: u64,
    pub distance: f32,
}
```

**Methoden:**
- `SpatialIndex::from_nodes(nodes: &HashMap<u64, MapNode>) -> Self`
- `SpatialIndex::empty() -> Self`
- `nearest(&self, query: Vec2) -> Option<SpatialMatch>`
- `within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch>`
- `within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64>`
- `len() -> usize` — Anzahl indexierter Nodes
- `is_empty() -> bool` — Prüft ob Index leer ist

---

### `Heightmap`

Lädt PNG-Heightmaps und berechnet Y-Koordinaten via bikubische Interpolation.

```rust
pub struct Heightmap { /* intern */ }
pub struct WorldBounds { pub min_x, min_z, max_x, max_z: f32 }
```

**Methoden:**
- `Heightmap::load(path) -> Result<Self>` — Lädt Heightmap, erkennt Bit-Tiefe und Map-Größe automatisch (FS25: pixels = map_size + 1)
- `Heightmap::load_with_bounds(path, world_bounds) -> Result<Self>` — Lädt Heightmap mit expliziten World-Bounds
- `sample_height(x, z, height_scale) -> f32` — Bikubische Interpolation
- `dimensions() -> (u32, u32)`
- `bit_depth() -> u8` — Erkannte Bit-Tiefe (8 oder 16)
- `world_bounds() -> &WorldBounds` — Verwendete Weltkoordinaten-Grenzen
- `WorldBounds::from_map_size(size)` — Bounds aus Map-Größe (zentriert bei 0,0)

---

### `MapMarker` / `AutoDriveMeta`

```rust
pub struct MapMarker {
    pub id: u64,
    pub name: String,
    pub group: String,
    pub marker_index: u32,
    pub is_debug: bool,
}
```

**Methoden:**
- `MapMarker::new(id, name, group, marker_index, is_debug) -> Self` — Erstellt neuen Marker

pub struct AutoDriveMeta {
    pub config_version: Option<String>,
    pub route_version: Option<String>,
    pub route_author: Option<String>,
    pub options: Vec<(String, String)>,  // In Original-Reihenfolge
}
```

## Design-Prinzipien

1. **HashMap statt Array:** Nodes AND Connections sind über ID(-Paar) indexiert → O(1)-Zugriff
2. **2D-Koordinaten:** Nur x/z gespeichert (y kommt aus Heightmap beim Export)
3. **Geometrie-Caching:** Midpoint/Angle werden vorberechnet für Rendering
4. **Lazy Spatial-Index:** Node-Mutationen setzen ein `spatial_dirty`-Flag; `ensure_spatial_index()` baut den Index erst bei der nächsten Abfrage neu auf
5. **Flag-Neuberechnung:** `recalculate_node_flags()` setzt Flags basierend auf Verbindungsprioriäten
6. **Keine UI/Render-Abhängigkeiten:** Reines Datenmodell + Geometrie
7. **Privates `connections`-Feld:** Kapselung gewährleistet Invarianten; Iterator-Zugriff via `connections_iter()`
