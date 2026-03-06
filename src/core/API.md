# Core API Documentation

## Ueberblick

Das `core`-Modul enthaelt die zentralen Datenstrukturen fuer AutoDrive-Konfigurationen und Geometrie-Typen. Keine Abhaengigkeiten zu UI, Render oder App.

## Haupttypen

### `Camera2D`

2D-Kamera mit Pan und Zoom. Reiner Geometrie-Typ ohne I/O-Abhaengigkeiten.

```rust
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
}
```

**Konstanten:**
- `BASE_WORLD_EXTENT: f32 = 2048.0` — Basis-Weltgroesse fuer Projektion
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
camera.zoom_by(1.5);  // vergroessern
camera.zoom_by(0.5);  // verkleinern

// Zoom mit benutzerdefinierten Grenzen clamped
camera.zoom_by_clamped(1.5, 0.2, 50.0);  // custom min/max

// Zoom auf gegebene Grenzen clampen
camera.clamp_zoom(0.2, 50.0);

// View-Matrix fuer Shader (nur Translation, kein Scale)
let mat = camera.view_matrix(); // Mat3

// Screen zu Welt-Koordinaten (beruecksichtigt BASE_WORLD_EXTENT + Zoom)
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

// Pick-Radius in Welt-Einheiten (fuer Node-Selektion)
let pick_radius = camera.pick_radius_world(viewport_height, pick_radius_px);

// Pick-Radius skaliert (fester Wert bei ZOOM_MAX, unabhaengig vom aktuellen Zoom)
let scaled = camera.pick_radius_world_scaled(viewport_height, pick_radius_px);
```

**View-Matrix:** Enthaelt nur Translation (`-position`). Zoom wird ausschliesslich ueber die orthographische Projektion im Renderer gesteuert.

**Screen-to-World:** Beruecksichtigt `BASE_WORLD_EXTENT`, Zoom und Aspekt-Ratio:
```
world = NDC * BASE_WORLD_EXTENT * aspect / zoom + position
```

**Pick-Radius:** Konvertiert den uebergebenen Pixel-Radius in Welt-Koordinaten basierend auf Zoom und Viewport-Hoehe. Der Pixel-Wert (`SELECTION_PICK_RADIUS_PX`) lebt in `shared::options`, damit `core` keine Abhaengigkeit auf `shared` hat.

**Pick-Radius (skaliert):** `pick_radius_world_scaled()` gibt einen fixen Weltradius zurueck, berechnet bei `ZOOM_MAX` — damit bleibt der Pick-Bereich unabhaengig vom aktuellen Zoom konsistent.

---

### `BackgroundMap`

Laedt Bilder (PNG, JPG, DDS) als Map-Hintergrund und stellt sie fuer GPU-Rendering bereit.
Unterstuetzt auch das Laden aus ZIP-Archiven.

```rust
pub struct BackgroundMap { /* intern */ }
```

**Methoden:**
- `BackgroundMap::load_from_file(path, crop_size) -> Result<Self>` — Bild laden, optional Center-Crop
- `BackgroundMap::from_image(image, source_label, crop_size) -> Result<Self>` — `pub(crate)`: BackgroundMap aus bereits dekodiertem `DynamicImage` erstellen (fuer Overview-Generator u.a.)
- `image_data() -> &DynamicImage` — Bilddaten
- `world_bounds() -> &WorldBounds` — Weltkoordinaten-Bereich
- `opacity() -> f32` — Aktuelle Opacity
- `set_opacity(opacity)` — Opacity setzen (clamped 0.0–1.0)
- `dimensions() -> (u32, u32)` — Bildgroesse in Pixeln

**Freie Funktionen (ZIP-Support):**
- `list_images_in_zip(zip_path) -> Result<Vec<ZipImageEntry>>` — Bilddateien (png/jpg/jpeg/dds) im ZIP auflisten
- `load_from_zip(zip_path, entry_name, crop_size) -> Result<BackgroundMap>` — Bild aus ZIP in-memory extrahieren und als BackgroundMap laden

### `ZipImageEntry`

Beschreibt eine Bilddatei innerhalb eines ZIP-Archivs.

```rust
pub struct ZipImageEntry {
    pub name: String,  // Dateiname im Archiv (inkl. Pfad)
    pub size: u64,     // Unkomprimierte Dateigroesse in Bytes
}
```

---

### `RoadMap`

Container fuer das gesamte AutoDrive-Strassennetzwerk.

```rust
pub struct RoadMap {
    pub nodes: HashMap<u64, MapNode>,
    connections: HashMap<(u64, u64), Connection>,  // Privat, Zugriff ueber connections_iter()
    pub map_markers: Vec<MapMarker>,
    pub meta: AutoDriveMeta,
    pub version: u32,
    pub map_name: Option<String>,
}
```

**Methoden:**
- `new(version: u32) -> Self` — Erstellt leere RoadMap
- `add_node(&mut self, node: MapNode)` — Fuegt Node hinzu (markiert Spatial-Index als dirty)
- `remove_node(&mut self, node_id: u64) -> Option<MapNode>` — Entfernt Node + betroffene Verbindungen
- `update_node_position(&mut self, node_id: u64, new_position: Vec2) -> bool` — Position aktualisieren (baut Geometrie neu, markiert Spatial als dirty)
- `add_connection(&mut self, connection: Connection)` — Fuegt Verbindung hinzu
- `has_connection(&self, start_id: u64, end_id: u64) -> bool` — Prueft ob Verbindung existiert
- `find_connection(&self, start_id: u64, end_id: u64) -> Option<&Connection>` — Findet exakte Verbindung
- `find_connections_between(&self, node_a: u64, node_b: u64) -> Vec<&Connection>` — Alle Verbindungen zwischen zwei Nodes (beide Richtungen)
- `remove_connection(&mut self, start_id: u64, end_id: u64) -> bool` — Entfernt exakte Verbindung
- `remove_connections_between(&mut self, node_a: u64, node_b: u64) -> usize` — Entfernt alle Verbindungen zwischen zwei Nodes
- `invert_connection(&mut self, start_id: u64, end_id: u64) -> bool` — Invertiert Start/End einer Verbindung
- `set_connection_direction(&mut self, start_id: u64, end_id: u64, direction) -> bool` — Richtung aendern
- `set_connection_priority(&mut self, start_id: u64, end_id: u64, priority) -> bool` — Prioritaet aendern
- `connections_iter(&self) -> impl Iterator<Item = &Connection>` — Iterator ueber alle Verbindungen
- `connections_between_ids(&self, ids: &IndexSet<u64>) -> Box<dyn Iterator<Item = &Connection>>` — Connections zwischen Nodes aus der gegebenen ID-Menge (O(n), fuer Bulk-Operationen)
- `connected_neighbors(&self, node_id: u64) -> Vec<ConnectedNeighbor>` — Alle Nachbarn eines Nodes mit Richtung und Winkel
- `is_resampleable_chain(&self, node_ids: &IndexSet<u64>) -> bool` — Prueft ob die selektierten Nodes eine zusammenhaengende Kette bilden (Kreuzungen nur an Endpunkten erlaubt)
- `next_node_id(&self) -> u64` — Naechste freie Node-ID
- `add_map_marker(&mut self, marker: MapMarker)` — Fuegt Marker hinzu
- `has_marker(&self, node_id: u64) -> bool` — Prueft ob Node einen Marker hat
- `find_marker_by_node_id(&self, node_id: u64) -> Option<&MapMarker>` — Marker eines Nodes finden
- `remove_marker(&mut self, node_id: u64) -> bool` — Marker eines Nodes entfernen
- `rebuild_connection_geometry(&mut self)` — Aktualisiert Connection-Geometrie
- `recalculate_node_flags(&mut self, node_ids: &[u64])` — NodeFlags basierend auf Verbindungsprioriaeten neu berechnen
- `ensure_spatial_index(&mut self)` — Baut Spatial-Index nur auf, wenn dirty-Flag gesetzt ist (lazy rebuild)
- `build_spatial_index(&self) -> SpatialIndex` — Erstellt neuen Spatial-Index aus aktuellen Nodes
- `rebuild_spatial_index(&mut self)` — Baut den internen Spatial-Index sofort neu auf
- `node_count() -> usize` / `connection_count() -> usize` / `marker_count() -> usize`
- `count_duplicates(&self, epsilon: f32) -> (u32, u32)` — Zaehlt Duplikat-Nodes und -Gruppen
- `deduplicate_nodes(&mut self, epsilon: f32) -> DeduplicationResult` — Entfernt Duplikat-Nodes und verbindet Referenzen um

**Spatial Queries (persistenter KD-Tree, lazy rebuild via `ensure_spatial_index`):**
- `nearest_node(&self, query: Vec2) -> Option<SpatialMatch>` — Naechster Node
- `nodes_within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch>` — Nodes im Umkreis
- `nodes_within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64>` — Nodes im Rechteck
- `nodes_within_rect_into(&self, min: Vec2, max: Vec2, out: &mut Vec<u64>)` — Rechteck-Query in einen bereitgestellten Scratch-Buffer (keine Extra-Allocation im Hotpath)

---

### `ConnectedNeighbor`

Beschreibt einen ueber eine Verbindung erreichbaren Nachbar-Node.

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
- `had_duplicates() -> bool` — Gibt `true` zurueck wenn Duplikate gefunden und entfernt wurden

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
- `Regular` (0) — Normaler Wegpunkt (Hauptstrasse)
- `SubPrio` (1) — Nebenstrasse (nur SubPriority-Verbindungen)
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
- `is_empty() -> bool` — Prueft ob Index leer ist

---

### `Heightmap`

Laedt PNG-Heightmaps und berechnet Y-Koordinaten via bikubische Interpolation.

```rust
pub struct Heightmap { /* intern */ }
pub struct WorldBounds { pub min_x, min_z, max_x, max_z: f32 }
```

**Methoden:**
- `Heightmap::load(path) -> Result<Self>` — Laedt Heightmap, erkennt Bit-Tiefe und Map-Groesse automatisch (FS25: pixels = map_size + 1)
- `Heightmap::load_with_bounds(path, world_bounds) -> Result<Self>` — Laedt Heightmap mit expliziten World-Bounds
- `sample_height(x, z, height_scale) -> f32` — Bikubische Interpolation
- `dimensions() -> (u32, u32)`
- `bit_depth() -> u8` — Erkannte Bit-Tiefe (8 oder 16)
- `world_bounds() -> &WorldBounds` — Verwendete Weltkoordinaten-Grenzen
- `WorldBounds::from_map_size(size)` — Bounds aus Map-Groesse (zentriert bei 0,0)

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

---

### `FieldPolygon` und Feldgrenz-Geometrie

In `core::farmland` (re-exportiert aus `core`).

```rust
/// Geordnetes Feldgrenz-Polygon in Weltkoordinaten (x/z-Ebene).
/// Vertices stammen aus dem GRLE-Farmland-Raster.
/// Serialisierbar als JSON fuer Persistierung neben overview.jpg.
#[derive(Clone, Serialize, Deserialize)]
pub struct FieldPolygon {
    pub id: u32,           // Farmland-ID (1–255)
    pub vertices: Vec<Vec2>, // Geordnete Rand-Vertices in Weltkoordinaten
}
```

**Freie Funktionen:**

```rust
// Prueft ob ein Punkt innerhalb eines Polygons liegt (Ray-Casting).
pub fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool

// Findet das erste FieldPolygon, das den gegebenen Weltpunkt enthaelt.
pub fn find_polygon_at<'a>(point: Vec2, polygons: &'a [FieldPolygon]) -> Option<&'a FieldPolygon>

// Douglas-Peucker-Vereinfachung fuer geschlossene Polygone.
// tolerance = 0.0 → kein Effekt; Mindestens 3 Punkte werden immer behalten.
pub fn simplify_polygon(vertices: &[Vec2], tolerance: f32) -> Vec<Vec2>

// Normalenbasiertes Polygon-Offset (negativ = nach innen, positiv = nach aussen).
// Fallback auf Original bei Degeneration (Orientierungswechsel, Miter-Overshoot).
pub fn offset_polygon(vertices: &[Vec2], offset: f32) -> Vec<Vec2>
```

**Koordinaten-Konvention:** Vertices in der x/z-Ebene, umgerechnet per
`world = pixel * (map_size / grle_width) - map_size / 2`.

**Beispiel:**

```rust
use fs25_ad_editor::core::{find_polygon_at, simplify_polygon, offset_polygon};

// Feld an Klickposition finden
if let Some(polygon) = find_polygon_at(click_pos, &farmland_polygons) {
    // Mit 5 m Toleranz vereinfachen + 3 m nach innen versetzen
    let simplified = simplify_polygon(&polygon.vertices, 5.0);
    let inset = offset_polygon(&simplified, -3.0);
}
```

---

## Design-Prinzipien

1. **HashMap statt Array:** Nodes AND Connections sind ueber ID(-Paar) indexiert → O(1)-Zugriff
2. **2D-Koordinaten:** Nur x/z gespeichert (y kommt aus Heightmap beim Export)
3. **Geometrie-Caching:** Midpoint/Angle werden vorberechnet fuer Rendering
4. **Lazy Spatial-Index:** Node-Mutationen setzen ein `spatial_dirty`-Flag; `ensure_spatial_index()` baut den Index erst bei der naechsten Abfrage neu auf
5. **Flag-Neuberechnung:** `recalculate_node_flags()` setzt Flags basierend auf Verbindungsprioriaeten
6. **Keine UI/Render-Abhaengigkeiten:** Reines Datenmodell + Geometrie
7. **Privates `connections`-Feld:** Kapselung gewaehrleistet Invarianten; Iterator-Zugriff via `connections_iter()`
