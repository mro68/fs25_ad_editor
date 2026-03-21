# Shared API Documentation

## Ueberblick

Das `shared`-Modul enthaelt Layer-uebergreifende Typen, die zwischen `app` (Produzent) und `render` (Konsument) geteilt werden, um direkte Abhaengigkeiten zwischen diesen Schichten zu vermeiden.

## Module

- `render_scene.rs` — `RenderScene` Uebergabevertrag App → Render
- `render_quality.rs` — `RenderQuality` Enum (Low/Medium/High)
- `options/` — Zentrale Konfigurationskonstanten + `EditorOptions` (Laufzeit-Optionen), aufgeteilt in `camera.rs`, `render.rs`, `tools.rs`, `editor.rs`
- `i18n/` — Mehrsprachigkeits-System: `Language`-Enum, `I18nKey`-Enum, `t()`-Funktion (DE + EN, Zero-Alloc)
- `spline_geometry.rs` — Layer-neutrale Catmull-Rom-Geometrie-Funktionen (kein import aus `tools` noetig)

## Haupttypen

### `RenderScene`

Expliziter, read-only Uebergabevertrag zwischen App-Layer und Renderer.

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

`hidden_node_ids` wird genutzt, um Nodes im aktuellen Frame temporaer auszublenden
(z. B. bei Vorschau-Overlays im Properties-Panel), ohne die Domain-Daten zu mutieren.

**Methoden:**

- `has_map() -> bool` — Prueft ob eine RoadMap vorhanden ist

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

### Konfigurationskonstanten (`options.rs`)

Zentral gesammelte Konfigurationswerte, gegliedert nach Bereich:

| Bereich | Konstante | Wert | Beschreibung |
|---------|-----------|------|-------------|
| Kamera | `CAMERA_BASE_WORLD_EXTENT` | 2048.0 | Sichtbare Welt-Halbbreite bei Zoom 1.0 (Referenz-Duplikat, kanonisch in `Camera2D`) |
| Kamera | `CAMERA_ZOOM_MIN` / `MAX` | 0.75 / 200.0 | Zoom-Grenzen (konfigurierbarer Default) |
| Kamera | `CAMERA_ZOOM_STEP` | 1.1 | Zoom-Schritt bei Menue-Buttons / Shortcuts |
| Kamera | `CAMERA_SCROLL_ZOOM_STEP` | 1.025 | Zoom-Schritt bei Mausrad-Scroll |
| Selektion | `SELECTION_SIZE_FACTOR` | 125.0 | Vergroesserung selektierter Nodes in % (100..=200) |
| Nodes | `NODE_SIZE_WORLD` | 0.5 | Feste Node-Groesse in Welt-Einheiten |
| Nodes | `NODE_COLOR_DEFAULT` | `[0.0, 0.298, 1.0, 1.0]` | Blau (Regular) |
| Nodes | `NODE_COLOR_SUBPRIO` | `[1.0, 0.733, 0.0, 1.0]` | Gelborange (SubPrio) |
| Nodes | `NODE_COLOR_SELECTED` | `[0.843, 0.0, 1.0, 1.0]` | Violett (Selektiert) |
| Nodes | `NODE_COLOR_WARNING` | `[1.0, 0.0, 0.0, 1.0]` | Rot (Warning) |
| Nodes | `MIN_NODE_SIZE_PX` | 3.0 | Mindestgroesse in Pixeln beim Herauszoomen (0 = deaktiviert) |
| Connections | `CONNECTION_THICKNESS_WORLD` | 0.1 | Hauptstrassen-Linienbreite |
| Connections | `CONNECTION_THICKNESS_SUBPRIO_WORLD` | 0.05 | Nebenstrassen-Linienbreite |
| Connections | `ARROW_LENGTH_WORLD` / `ARROW_WIDTH_WORLD` | 1.0 / 0.5 | Pfeilgeometrie |
| Connections | `MIN_CONNECTION_WIDTH_PX` | 1.5 | Mindestbreite in Pixeln beim Herauszoomen |
| Connections | `MIN_ARROW_SIZE_PX` | 4.0 | Mindestpfeil-Groesse in Pixeln |
| Connections | `CONNECTION_COLOR_REGULAR` | `[0.0, 0.694, 1.0, 1.0]` | Blau (Einrichtung) |
| Connections | `CONNECTION_COLOR_DUAL` | `[0.890, 1.0, 0.396, 1.0]` | Hellgruen (Bidirektional) |
| Connections | `CONNECTION_COLOR_REVERSE` | `[1.0, 0.5, 0.1, 1.0]` | Orange (Rueckwaerts) |
| Marker | `MARKER_SIZE_WORLD` | 2.6 | Pin-Hoehe in Welt-Einheiten |
| Marker | `MARKER_COLOR` | `[0.0, 0.467, 0.102, 1.0]` | Dunkelgruen |
| Marker | `MARKER_OUTLINE_COLOR` | `[1.0, 0.643, 0.0, 1.0]` | Goldgelb |
| Marker | `MARKER_OUTLINE_WIDTH` | 0.08 | Standard-Umrissstärke (Anteil am Radius, 0.01–0.3) |
| Marker | `MIN_MARKER_SIZE_PX` | 8.0 | Mindestgroesse in Pixeln |
| Decimation | `NODE_DECIMATION_SPACING_PX` | 6.0 | Mindestabstand (px) fuer Grid-Decimation |
| Tools | `SNAP_SCALE_PERCENT` | 100.0 | Snap-Radius in % der Node-Groesse |
| Tools | `HITBOX_SCALE_PERCENT` | 100.0 | Standard-Hitbox-Skalierung in % der Node-Groesse |
| Tools | `MOUSE_WHEEL_DISTANCE_STEP_M` | 0.1 | Schrittweite (m) fuer Distanz-Felder bei Mausrad |
| Terrain | `TERRAIN_HEIGHT_SCALE` | 255.0 | Hoehenskala fuer Heightmap-Export |
| Zoom-Kompensation | `DEFAULT_ZOOM_COMPENSATION_MAX` | 4.0 | Standard-Maximum fuer den Zoom-Kompensationsfaktor (1.0 = deaktiviert) |

### `ValueAdjustInputMode`

Steuert, wie numerische Felder im UI veraendert werden.

```rust
pub enum ValueAdjustInputMode {
    DragHorizontal, // LMT nach links/rechts
    MouseWheel,     // Mausrad hoch/runter
}
```

### `OverviewLayerOptions`

Konfigurierbare Layer-Optionen fuer die Uebersichtskarten-Generierung.
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

Der Default setzt alle Layer ausser `legend` auf `true`.

### `SelectionStyle`

Darstellungsmodus fuer selektierte Nodes.

```rust
pub enum SelectionStyle {
    Ring,     // Farbiger Ring am Rand (Standard)
    Gradient, // Farbverlauf von Mitte nach Rand
}
```

## Design-Prinzipien

1. **Entkopplung:** `shared` verhindert direkte Abhaengigkeiten zwischen `app` und `render`
2. **Single Source of Truth:** Alle Rendering-Konstanten in `options.rs` zentralisiert
3. **Immutable Contract:** `RenderScene` ist read-only (Clone, keine Mutation)

---

### `EditorOptions` (Laufzeit-Optionen)

Alle zur Laufzeit aenderbaren Editor-Optionen. Wird als `fs25_auto_drive_editor.toml` neben der Binary gespeichert.

```rust
pub struct EditorOptions {
    // Nodes
    pub node_size_world: f32,
    pub node_color_default: [f32; 4],
    pub node_color_subprio: [f32; 4],
    pub node_color_selected: [f32; 4],
    pub node_color_warning: [f32; 4],
    // Selektion
    pub selection_size_factor: f32, // Prozentwert 100..=200
    pub selection_style: SelectionStyle,
    /// Doppelklick-Segment: Bei Kreuzung (degree != 2) stoppen.
    pub segment_stop_at_junction: bool,
    /// Doppelklick-Segment: Max. Winkelabweichung in Grad (0 = nicht pruefen).
    pub segment_max_angle_deg: f32,
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
    /// Umrissstärke des Map-Markers als Anteil am Radius (0.01–0.3).
    pub marker_outline_width: f32,
    // Kamera
    pub camera_zoom_step: f32,
    pub camera_scroll_zoom_step: f32,
    // Tools
    pub snap_scale_percent: f32,
    /// Hitbox-Skalierung in Prozent der Node-Groesse (100 = exakte Node-Groesse)
    pub hitbox_scale_percent: f32,
    /// Schrittweite in Metern fuer Distanz-Felder bei Mausrad
    pub mouse_wheel_distance_step_m: f32,
    /// Eingabemodus fuer numerische Feldaenderungen
    pub value_adjust_input_mode: ValueAdjustInputMode,
    /// true = Mittelpunkt zwischen Vorgaenger und Nachfolger beim Loeschen verbinden
    pub reconnect_on_delete: bool,
    /// true = bestehende Verbindung beim Platzieren splitten
    pub split_connection_on_place: bool,
    /// true = Route-Tool-Ergebnisse automatisch als Segment registrieren.
    pub auto_create_segment: bool,
    // Kamera (erweitert)
    /// Minimaler Zoom-Faktor (konfig, ueberschreibt Camera2D::ZOOM_MIN)
    pub camera_zoom_min: f32,
    /// Maximaler Zoom-Faktor (konfig, ueberschreibt Camera2D::ZOOM_MAX)
    pub camera_zoom_max: f32,
    // Terrain
    pub terrain_height_scale: f32,
    // Hintergrund (Fade-Out bei kleinem Zoom)
    pub bg_opacity: f32,
    pub bg_opacity_at_min_zoom: f32,
    pub bg_fade_start_zoom: f32,
    // Copy/Paste
    /// Deckkraft der Paste-Vorschau (0.0 transparent … 1.0 opak)
    pub copy_preview_opacity: f32,
    // Segment-Overlay
    /// Schriftgroesse des Lock-Icons im Segment-Overlay in Pixeln
    pub segment_lock_icon_size_px: f32,
    // Uebersichtskarte
    /// Layer-Optionen fuer Uebersichtskarten-Generierung
    pub overview_layers: OverviewLayerOptions,
    // Zoom-Kompensation
    /// Maximaler Zoom-Kompensationsfaktor (1.0 = deaktiviert, 4.0 = Standard).
    /// Verhindert, dass Nodes und Verbindungen beim Herauszoomen unsichtbar werden.
    pub zoom_compensation_max: f32,
    // LOD / Mindestgroessen (Zoomout-Darstellung)
    pub min_node_size_px: f32,
    pub min_connection_width_px: f32,
    pub min_arrow_size_px: f32,
    pub min_marker_size_px: f32,
    pub node_decimation_spacing_px: f32,
    // Sprache
    /// Aktive UI-Sprache (Standard: `Language::De`). Steuert alle UI-Übersetzungen via `t()`.
    pub language: Language,
}
```

**Methoden:**

- `EditorOptions::load_from_file(path) -> Self` — TOML-Datei laden (bei Fehler: Defaults)
- `EditorOptions::save_to_file(&self, path) -> Result<()>` — Als TOML speichern
- `EditorOptions::config_path() -> PathBuf` — Pfad zur Optionen-Datei neben der Binary
- `hitbox_radius(&self) -> f32` — Berechnet den Hitbox-Radius in Welteinheiten (`node_size_world * hitbox_scale_percent / 100`)
- `snap_radius(&self) -> f32` — Berechnet den Snap-Radius in Welteinheiten
- `selection_size_multiplier(&self) -> f32` — Selektions-Multiplikator aus `selection_size_factor` in Prozent
- `zoom_compensation(&self, zoom: f32) -> f32` — Berechnet den Zoom-Kompensationsfaktor fuer eine gegebene Zoom-Stufe. Formel: `(1/zoom)^0.5`, geclampt auf `[1.0, zoom_compensation_max]`. Bei `zoom >= 1.0` ist der Faktor `1.0`; bei `zoom_compensation_max <= 1.0` ist die Kompensation deaktiviert.

---

## Mehrsprachigkeit (`shared::i18n`)

Compile-Time-sicheres i18n-System mit Enum-Keys. Alle Übersetzungen sind `&'static str` (Zero-Alloc).
Unterstützte Sprachen: Deutsch (`De`, Standard) und Englisch (`En`).
`match` in den Sprachdateien erzwingt Vollständigkeit bei neuen Keys.

### `Language`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    De,
    En,
}
```

**Methoden:**
- `display_name(self) -> &'static str` — Anzeigename in der jeweiligen Sprache (`"Deutsch"` / `"English"`)
- `all() -> &'static [Language]` — Alle verfügbaren Sprachen — geeignet für ComboBox-Iteration

### `I18nKey`

Enum aller übersetzbaren UI-Schlüssel. Gruppen: Allgemein, Dialog-Chrome, Options-Dialog (Abschnitte, Felder, Tooltips), Menüleiste, Status-Bar, Tool-Namen, Sidebar, Zoom, Hintergrund, Route-Gruppen, Floating-Menus, Kontextmenüs, Command-Palette, LongPress-Tooltips.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // Allgemein
    AppTitle, Ok, Cancel, Apply, Close, Reset, Delete, Add, Remove, LanguageLabel,
    // Dialog-Chrome
    DialogClose, DialogDefaults,
    // Options-Dialog: Navigation & Felder
    OptSectionGeneral, OptSectionNodes, OptSectionTools, OptSectionConnections, OptSectionBehavior,
    // ... (Opt* — ~95 Keys, vollständig in keys.rs)
    // Menüleiste (MenuXxx — 29 Keys)
    // Status-Bar (StatusXxx — 13 Keys)
    // Tool-Namen (ToolNameXxx — 4 Keys)
    // === NEU (Branch feature/zoom-shortcuts-i18n, 79 neue Keys) ===
    // Sidebar-Abschnittstitel (SidebarXxx — 7 Keys: Tools, Basics, Edit, Direction, Priority, Zoom, Background)
    SidebarTools, SidebarBasics, SidebarEdit, SidebarDirection, SidebarPriority, SidebarZoom, SidebarBackground,
    // Zoom-Buttons (ZoomXxx — 4 Keys)
    ZoomFullMap, ZoomFullMapHelp, ZoomToSelection, ZoomToSelectionHelp,
    // Hintergrund-Buttons (BackgroundXxx — 5 Keys)
    BackgroundHide, BackgroundShow, BackgroundScaleDown, BackgroundScaleUp, BackgroundScaleReset,
    // Route-Gruppenbezeichnungen (RouteGroupXxx — 3 Keys)
    RouteGroupStraight, RouteGroupCurves, RouteGroupSection,
    // Floating-Menu Tooltips (FloatingXxx — 13 Keys: Tools, Basics, Edit, DirectionPriority, Zoom)
    FloatingToolSelect, FloatingToolConnect, FloatingToolAddNode,
    FloatingBasicStraight, FloatingBasicQuadratic, FloatingBasicCubic, FloatingBasicSpline, FloatingBasicSmoothCurve,
    FloatingEditBypass, FloatingEditParking, FloatingEditRouteOffset,
    FloatingDirectionRegular, FloatingDirectionDual, FloatingDirectionReverse, FloatingPriorityMain, FloatingPrioritySub,
    FloatingZoomFullMap, FloatingZoomSelection,
    // Kontextmenü-Einträge (CtxXxx — 28 Keys)
    CtxToolSubmenu, CtxToolSelect, CtxToolConnect, CtxToolAddNode,
    CtxZoomSubmenu, CtxZoomFullMap, CtxZoomSelection,
    CtxRouteSubmenu, /* ... weitere Ctx* Keys */
    // Command-Palette-Einträge (PaletteXxx — 14 Keys)
    PaletteSearchHint, PaletteNoResults, PaletteOpenFile, /* ... weitere Palette* Keys */
    // LongPress-Tooltips (LpXxx — 13 Keys: Tools, Basics, SectionTools, Direction, Priority)
    LpToolSelect, LpToolConnect, LpToolAddNode,
    LpStraight, LpCurveQuad, LpCurveCubic, LpSpline, LpSmoothCurve,
    LpBypass, LpParking, LpRouteOffset,
    LpDirectionRegular, LpDirectionDual, LpDirectionReverse, LpPriorityMain, LpPrioritySub,
    // Bestätigungs-Dialoge (ConfirmDissolveXxx — 4 Keys)
    ConfirmDissolveTitle,   // "Gruppe auflösen"
    ConfirmDissolveMessage, // "Soll die Gruppe wirklich aufgelöst werden? Die Nodes bleiben erhalten."
    ConfirmDissolveOk,      // "Auflösen"
    ConfirmDissolveCancel,  // "Abbrechen"
}
```

### `t()`

```rust
pub fn t(lang: Language, key: I18nKey) -> &'static str
```

Übersetzt `key` in die gewählte Sprache. Gibt immer `&'static str` zurück — keine Heap-Allokation.

**Beispiel:**
```rust
use crate::shared::{t, I18nKey, Language};

let lang = opts.language;
ui.label(t(lang, I18nKey::OptSectionGeneral)); // → "Allgemein" oder "General"
```

**Re-Exports aus `shared`:**
- `shared::t` — Übersetzungs-Funktion
- `shared::Language` — Sprachen-Enum
- `shared::I18nKey` — Schlüssel-Enum

**Importrichtung:** `UI → shared::i18n`, `App → shared::i18n` (erlaubt, da `shared` Cross-Layer)
