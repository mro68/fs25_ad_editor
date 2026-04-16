# fs25_map_overview — API-Dokumentation

Stand: 2026-03-07

## Überblick

Das Crate `fs25_map_overview` generiert aus einem FS25-Map-Mod-ZIP eine detaillierte Übersichtskarte:
- Terrain-Compositing aus Weight-Maps (gewichtete Farbmischung)
- Hillshade aus DEM (digitales Höhenmodell)
- Farmland-Grenzen und ID-Labels
- POI-Marker mit Beschriftung
- Legende und Titelleiste

Farmland-Polygone werden zusätzlich extrahiert und dem Aufrufer bereitgestellt (für `FieldBoundaryTool`).

---

## Modulstruktur

```text
crates/fs25_map_overview/src/
    lib.rs          # Einstiegspunkte und ZIP-Wrapper
    layer_bundle.rs # RGBA-Layer-Bundle und CPU-Komposition
    composite.rs    # Endmontage: Farmland-Grenzen, POIs, Legende
    composite/
        legend.rs
  discovery.rs    # Kartenstruktur-Erkennung aus ZIP (modDesc.xml, Map-Config-XML)
  farmland.rs     # Moore-Neighbor-Boundary-Tracing → FarmlandPolygon
  gdm.rs          # GDM-Dekoder (GIANTS Data Format)
  grle.rs         # GRLE-Dekoder (GIANTS Run-Length Encoded InfoLayer)
  hillshade.rs    # Hillshade-Berechnung aus DEM
  palette.rs      # Farbpalette für Terrain-Layer
  terrain.rs      # Weight-Map-Compositing → RGB-Terrain-Bild
  text.rs         # Textrenderung auf Bildern
```

---

## Öffentliche Funktionen (lib.rs)

### `generate_overview_from_zip`

```rust
pub fn generate_overview_from_zip(zip_path: &str, options: &OverviewOptions) -> Result<RgbImage>
```

Generiert eine RGB-Übersichtskarte aus einem FS25 Map-Mod-ZIP.  
Gibt nur das Bild zurück — für Farmland-Polygone `generate_overview_result_from_zip` verwenden.

---

### `generate_overview`

```rust
pub fn generate_overview(
    files: &HashMap<String, Vec<u8>>,
    map_info: &MapInfo,
    options: &OverviewOptions,
) -> Result<RgbImage>
```

Generiert eine Übersichtskarte aus bereits entpackten Dateien. Nützlich wenn das ZIP bereits vorliegt.

---

### `generate_overview_result_from_zip`

```rust
pub fn generate_overview_result_from_zip(
    zip_path: &str,
    options: &OverviewOptions,
) -> Result<OverviewResult>
```

Generiert Übersichtsbild **und** extrahiert Farmland-Polygone in einem Schritt.  
Gibt ein [`OverviewResult`] zurück. Wird vom Editor verwendet, um Polygone für das `FieldBoundaryTool` bereitzustellen.

Das enthaltene Bild entspricht dem kombinierten RGBA-Ergebnis der aktuell sichtbaren Layer und kann daher Transparenz enthalten.

---

### `generate_overview_layer_bundle_from_zip`

```rust
pub fn generate_overview_layer_bundle_from_zip(
    zip_path: &str,
    options: &OverviewOptions,
) -> Result<OverviewLayerBundle>
```

Generiert ein vollständiges Layer-Bundle aus einem FS25-Map-ZIP.  
Liefert das opake Terrain-Basisbild, separate transparente Overlay-PNGs sowie ein `combined`-Bild, das aus den sichtbaren Layern zusammengesetzt ist.

---

### `generate_overview_layer_bundle`

```rust
pub fn generate_overview_layer_bundle(
    files: &HashMap<String, Vec<u8>>,
    map_info: &MapInfo,
    options: &OverviewOptions,
) -> Result<OverviewLayerBundle>
```

Variante fuer bereits extrahierte ZIP-Dateien.  
Nuetzlich fuer Tests oder Aufrufer, die Discovery und ZIP-IO selbst kontrollieren.

---

### `try_extract_polygons_from_zip_ground_gdm`

```rust
pub fn try_extract_polygons_from_zip_ground_gdm(
    zip_path: &str,
) -> Option<(Vec<FarmlandPolygon>, u32, u32)>
```

Liest `densityMap_ground.gdm` direkt aus einer Map-ZIP, erkennt dafuer zuerst die Kartenstruktur per `discover_map(...)` und extrahiert dann Feldpolygone aus dem Ground-GDM-Raster. Liefert `None`, wenn ZIP, Discovery, Datei-Suche oder Dekodierung fehlschlagen.

---

## Öffentliche Typen

### `OverviewResult`

Ergebnis von `generate_overview_result_from_zip`.

```rust
pub struct OverviewResult {
    /// Generiertes kombiniertes Übersichtsbild (RGBA möglich)
    pub image: DynamicImage,
    /// Extrahierte Farmland-Polygone im GRLE-Pixel-Koordinatenraum
    pub farmland_polygons: Vec<FarmlandPolygon>,
    /// GRLE-Rasterbreite in Pixeln (= Weltgröße in Metern)
    pub grle_width: u32,
    /// GRLE-Rasterhöhe in Pixeln (= Weltgröße in Metern)
    pub grle_height: u32,
    /// Weltgröße in Metern (aus MapInfo)
    pub map_size: f32,
    /// Rohes Farmland-ID-Raster (1 Byte pro Pixel, 0 = kein Feld).
    /// Für Pixel-basierte Analysen im Editor (z.B. Feldweg-Erkennung via `FarmlandGrid`).
    /// `None` wenn kein Farmland-Layer verfügbar war.
    pub farmland_ids: Option<Vec<u8>>,
}
```

**Koordinaten-Umrechnung:**  
`world = pixel * (map_size / grle_width)` — muss vom Aufrufer durchgeführt werden.

---

### `OverviewOptions`

Steuert, welche Layer im kombinierten Ergebnis initial sichtbar sind.

```rust
pub struct OverviewOptions {
    pub terrain: bool,        // Terrain-Basisbild im combined-Ergebnis
    pub hillshade: bool,      // 3D-Reliefschattierung aus DEM
    pub farmlands: bool,      // Farmland-Grenzlinien einzeichnen
    pub farmland_ids: bool,   // Farmland-ID-Nummern einzeichnen
    pub pois: bool,           // POI-Marker mit Beschriftung
    pub legend: bool,         // Legende unten links
}
```

`Default` aktiviert alle Layer.

Die Legacy-Funktionen `generate_overview_from_zip()` und `generate_overview()` liefern weiterhin ein opakes RGB-Bild und ignorieren `terrain = false` bewusst, um abwaertskompatibel zu bleiben.

---

### `OverviewLayerBundle`

Ergebnis der Layer-Bundle-Generierung.

```rust
pub struct OverviewLayerBundle {
    pub terrain: RgbaImage,
    pub hillshade: RgbaImage,
    pub farmland_borders: RgbaImage,
    pub farmland_ids: RgbaImage,
    pub poi_markers: RgbaImage,
    pub legend: RgbaImage,
    pub combined: RgbaImage,
    pub farmland_polygons: Vec<FarmlandPolygon>,
    pub grle_width: u32,
    pub grle_height: u32,
    pub map_size: f32,
    pub farmland_ids_raw: Option<Vec<u8>>,
}
```

`terrain` enthaelt das opake Basisbild inklusive Title-Bar.  
Alle anderen Bildfelder sind transparente RGBA-Overlays und koennen per `compose_layers(...)` wieder zusammengesetzt werden.

---

### `FieldDetectionSource`

Enum fuer die Feldquellen, die der Editor fuer die Polygon-Extraktion auswaehlen kann.

```rust
pub enum FieldDetectionSource {
    FromZip,
    ZipGroundGdm,
    FieldTypeGrle,
    GroundGdm,
    FruitsGdm,
}
```

`Default` zeigt auf `ZipGroundGdm`.

---

### `MapInfo`

Erkannte Kartenstruktur aus einem Map-Mod.

```rust
pub struct MapInfo {
    pub title: String,                    // Kartentitel (aus modDesc.xml)
    pub map_size: u32,                    // Kartengröße in Pixeln (quadratisch)
    pub config_path: String,              // Pfad zur Map-Config-XML rel. zum Mod-Root
    pub data_dir: String,                 // Pfad zum data/-Verzeichnis rel. zum Mod-Root
    pub config_dir: String,               // Pfad zum config/-Verzeichnis rel. zum Mod-Root
    pub placeables_path: Option<String>,  // Pfad zur placeables.xml (optional)
}
```

---

### `FarmlandPolygon`

Geordneter Umriss-Polygon eines einzelnen Farmland-Felds.

```rust
pub struct FarmlandPolygon {
    pub id: u32,                   // Farmland-ID (1–254; 0 und 255 = kein Feld)
    pub vertices: Vec<(f32, f32)>, // Rand-Pixel als (x, y) in GRLE-Pixel-Koordinaten
}
```

---

### `Poi`

Erkannter Point of Interest (aus placeables.xml).

```rust
pub struct Poi {
    pub x: u32,       // Pixel-X-Koordinate
    pub y: u32,       // Pixel-Y-Koordinate
    pub label: String, // Anzeigename
}
```

---

## Öffentliche Funktionen nach Modul

### `farmland`

```rust
pub fn extract_farmland_polygons(grle_data: &[u8]) -> Result<(Vec<FarmlandPolygon>, u32, u32)>
```
Extrahiert Farmland-Polygone aus GRLE-Rohdaten via Moore-Neighbor-Boundary-Tracing.  
Gibt `(polygons, width, height)` zurück.

```rust
pub fn extract_farmland_polygons_from_ids(
    pixels: &[u8],
    width: usize,
    height: usize,
) -> Vec<FarmlandPolygon>
```
Formatunabhängige Variante: nimmt bereits dekodierte Grayscale-Pixeldaten entgegen.  
Wird sowohl für GRLE- als auch PNG-basierte Farmlands verwendet.

---

### `discovery`

```rust
pub fn discover_map(files: &HashMap<String, Vec<u8>>) -> Result<MapInfo>
```
Erkennt die Kartenstruktur aus den Dateien eines Map-Mod-ZIPs.  
Parst `modDesc.xml` und die Map-Config-XML.

```rust
pub fn find_weight_maps(files: &HashMap<String, Vec<u8>>, data_dir: &str) -> Vec<(String, Vec<u8>)>
pub fn find_dem(files: &HashMap<String, Vec<u8>>, data_dir: &str) -> Option<&Vec<u8>>
pub fn find_farmlands(files: &HashMap<String, Vec<u8>>, data_dir: &str) -> Option<(String, &Vec<u8>)>
pub fn find_ground_gdm(files: &HashMap<String, Vec<u8>>, data_dir: &str) -> Option<(String, &Vec<u8>)>
```
Lokalisiert spezifische Dateitypen im Mod-ZIP.

---

### `grle`

```rust
pub fn decode_grle(data: &[u8]) -> Result<GrleImage>
```
Dekodiert GRLE-Rohdaten (GIANTS Run-Length Encoding) zu Grayscale-Pixeln.

```rust
pub struct GrleImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,  // 1 Byte pro Pixel
}
```

---

### `hillshade`

```rust
pub fn compute_hillshade(dem: &GrayImage, params: &HillshadeParams) -> Result<Vec<f32>>
```
Berechnet Hillshade-Werte (0.0–1.0) aus einem DEM-Graybild via Sobel-Gradient.

```rust
pub fn apply_hillshade(image: &mut [u8], hillshade: &[f32], blend_factor: f32)
```
Überlagert das Hillshade-Ergebnis auf ein RGB-Bild (Multiplikations-Blend).

```rust
pub struct HillshadeParams {
    pub azimuth_deg: f32,  // Lichtrichtung (Standard: 315° = NW)
    pub altitude_deg: f32, // Lichthöhe (Standard: 45°)
    pub blend_factor: f32, // Mischstärke (Standard: 0.45)
}
```

---

### `terrain`

```rust
pub fn composite_terrain(layers: &[WeightLayer], target_size: u32) -> Result<RgbImage>
```
Mischt Weight-Map-Layer zu einem RGB-Terrain-Bild (gewichteter Farbdurchschnitt).

```rust
pub fn composite_terrain_from_images(
    weight_images: &[(String, DynamicImage)],
    target_size: u32,
) -> Result<RgbImage>
```
Konvenienz-Variante: nimmt `(Name, Bild)`-Paare direkt entgegen.

```rust
pub struct WeightLayer {
    pub name: String,     // Weight-Map-Name (bestimmt Farbe via palette)
    pub weights: GrayImage, // Gewichtsbild (0–255)
}
```

---

### `composite` (öffentliche Unterstruktur)

```rust
pub fn extract_farmland_boundaries(grle_data: &[u8], target_size: u32) -> Result<FarmlandData>
pub fn draw_farmland_boundaries(image: &mut RgbImage, farmlands: &FarmlandData)
pub fn draw_farmland_ids(image: &mut RgbImage, farmlands: &FarmlandData)
pub fn extract_pois(xml_data: &[u8], map_size: u32) -> Vec<Poi>
pub fn draw_pois_with_labels(image: &mut RgbImage, pois: &[Poi])
pub fn draw_legend(image: &mut RgbImage, options: &OverviewOptions)
pub fn draw_title_bar(image: &mut RgbImage, title: &str)
```

---

## Algorithmen

### Moore-Neighbor-Boundary-Tracing (`farmland.rs`)

Extrahiert für jede Farmland-ID einen geordneten Randpolygon:
1. Ersten Pixel der ID in Scan-Reihenfolge (Top-Left) finden
2. Clockwise-Neighbor-Tracing (W, NW, N, NE, E, SE, S, SW)
3. Jacob's Stopping Criterion: Abbruch beim ersten Rückbesuch von `initial_b`
4. Aufeinanderfolgende Duplikate entfernen

Koordinaten liegen im GRLE-Pixel-Raum. Weltkoordinaten: `pixel * (map_size / grle_width)`.

### Terrain-Compositing (`terrain.rs`)

Pro Pixel: gewichteter Farbdurchschnitt aller Layer:
```
color[pixel] = Σ(weight[i] * layer_color[i]) / Σ weight[i]
```
Pixel ohne Abdeckung erhalten die Hintergrundfarbe `[80, 100, 60]`.

### Hillshade (`hillshade.rs`)

Sobel-artiger Gradient → Lambert-Beleuchtungsmodell:
```
hs = sin(alt) * cos(slope) + cos(alt) * sin(slope) * cos(azimuth - aspect)
```
