# Tools API

Dokumentation fuer `app::tools`: `ToolManager`, `RouteTool`-Trait, registrierte Tools und gemeinsame Infrastruktur.

`RouteToolId` und `ToolAnchor` gehoeren zum app-weiten Tool-Vertrag in `app::tool_contract` und werden hier fuer die Tool-Implementierungen weiterverwendet.

**Zurueck:** [`../API.md`](../API.md)

---

## `ToolManager`

Verwaltet registrierte Route-Tools ueber stabile `RouteToolId`s und den kanonischen Tool-Katalog.

```rust
pub struct ToolManager { /* intern */ }
```

**Methoden:**

- `new() → Self` — Erstellt ToolManager aus `route_tool_catalog()` in kanonischer Reihenfolge
- `register(tool_id, tool)` — Neues Route-Tool unter stabiler ID registrieren
- `tool_count() → usize` — Anzahl registrierter Tools
- `tool_names() → Vec<(RouteToolId, &str)>` — Name + stabile ID aller Tools
- `tool_entries() → Vec<(RouteToolId, &str, &str)>` — ID, Name und Icon aller Tools (fuer Dropdown-Rendering)
- `set_active_by_id(tool_id)` — Aktives Tool setzen (Reset des vorherigen)
- `active_id() → Option<RouteToolId>` — ID des aktiven Tools
- `active_descriptor() → Option<&'static RouteToolDescriptor>` — Katalog-Descriptor des aktiven Tools
- `active_tool() → Option<&dyn RouteTool>` — Referenz auf aktives Tool
- `active_tool_mut() → Option<&mut dyn RouteTool>` — Mutable Referenz
- `reset()` — Alle Tools zuruecksetzen, aktives deaktivieren

---

## `RouteTool` (Trait)

Schnittstelle fuer alle Route-Tools (Linie, Kurve, …). Tools sind zustandsbehaftet und erzeugen Preview-Geometrie + `ToolResult`.

**Pflicht-Methoden:**

- `name() → &str` — Anzeigename
- `icon() → &str` — Icon-Zeichen fuer das Dropdown (rechts vom Label); Default: `""`
- `description() → &str` — Tooltip-Text
- `status_text() → &str` — Statustext fuer Properties-Panel
- `on_click(pos, road_map, ctrl) → ToolAction` — Viewport-Klick verarbeiten
- `preview(cursor_pos, road_map) → ToolPreview` — Preview-Geometrie berechnen
- `render_config(ui, distance_wheel_step_m) → bool` — Tool-Konfiguration im Properties-Panel (inkl. Mausrad-Schrittweite fuer Distanz-Felder)
- `execute(road_map) → Option<ToolResult>` — Ergebnis erzeugen
- `reset()` — Tool-Zustand zuruecksetzen
- `is_ready() → bool` — Bereit zur Ausfuehrung?

**Optionale Methoden (Default-Implementierung):**

- `has_pending_input() → bool` — Hat das Tool angefangene Eingaben? (fuer stufenweise Escape-Logik)
- `on_scroll_rotate(&mut self, delta: f32)` — Scroll-basierte Rotation verarbeiten (z.B. ParkingTool-Winkel-Steuerung)
- `set_direction(dir)` / `set_priority(prio)` — Editor-Defaults uebernehmen
- `set_snap_radius(radius)` — Snap-Radius fuer Node-Snapping setzen
- `set_last_created(ids, road_map)` / `last_created_ids() → &[u64]` — Erstellte Node-IDs (fuer Verkettung und Nachbearbeitung)
- `current_end_anchor() → Option<ToolAnchor>` — Liefert den End-Anker fuer den gemeinsamen `set_last_created`-Flow
- `save_anchors_for_recreate(road_map)` — Speichert tool-spezifische Recreate-Daten
- `last_end_anchor() → Option<ToolAnchor>` — Letzter Endpunkt fuer Verkettung
- `needs_recreate() → bool` / `clear_recreate_flag()` — Neuberechnung bei Config-Aenderung
- `execute_from_anchors(road_map) → Option<ToolResult>` — ToolResult aus gespeicherten Ankern
- `drag_targets() → Vec<Vec2>` — Weltpositionen verschiebbarer Punkte (fuer Drag-Hit-Test)
- `on_drag_start(pos, road_map, pick_radius) → bool` — Drag auf einen Punkt starten
- `on_drag_update(pos)` — Position des gegriffenen Punkts aktualisieren
- `on_drag_end(road_map)` — Drag beenden (Re-Snap bei Start/Ende)
- `tangent_menu_data() → Option<TangentMenuData>` — liefert Tangenten-Menuedaten fuer das Kontextmenue
- `apply_tangent_selection(start, end)` — wendet die im Kontextmenue gewaehlten Tangenten an
- `set_chain_inner_ids(ids: Vec<u64>)` — Setzt die inneren Node-IDs der geladenen Kette (ohne Start/Ende); wird nach `load_chain()` vom Handler aufgerufen um korrekte IDs fuer das "Original entfernen"-Feature bereitzustellen (Standard-Impl.: no-op)
- `set_farmland_grid(grid: Option<Arc<FarmlandGrid>>)` — Setzt das Farmland-Raster fuer Pixel-basierte Analysen (z.B. Feldweg-Erkennung). Standard-Impl.: no-op
- `set_background_map_image(image: Option<Arc<DynamicImage>>)` — Setzt das Hintergrundbild fuer farbbasierte Analysen. Standard-Impl.: no-op
- `needs_lasso_input() → bool` — Gibt `true` zurueck wenn das Tool Alt+Drag als Lasso-Eingabe benoetigt (z.B. `ColorPathTool` in Phase `Sampling`). Ist `true`, wird ein Alt+Drag-Lasso als `ToolLasso` geroutet und per `on_lasso_completed` geliefert statt die normale Node-Selektion auszuloesen. Standard-Impl.: `false`
- `on_lasso_completed(polygon: Vec<Vec2>) → ToolAction` — Verarbeitet ein abgeschlossenes Lasso-Polygon in Weltkoordinaten. Wird aufgerufen sobald der User einen Alt+Drag-Lasso abgeschlossen hat und `needs_lasso_input()` gilt. Standard-Impl.: gibt `ToolAction::Continue` zurueck (no-op)

**`ToolPreview` Felder**

- `nodes: Vec<Vec2>` — Vorschau-Node-Positionen
- `connections: Vec<(usize, usize)>` — Index-Paare in `nodes`
- `connection_styles: Vec<(ConnectionDirection, ConnectionPriority)>` — Stil pro Verbindung (gleiche Laenge wie `connections`)
- `labels: Vec<(usize, String)>` — Optionale Beschriftungen pro Node (Index in `nodes` + Labeltext)

**`ToolPreview::from_polyline()` — gemeinsamer Konstruktor**

```rust
pub fn from_polyline(
    positions: Vec<Vec2>,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
) -> Self
```

Erzeugt eine Vorschau aus einer Polyline mit einheitlicher Richtung und Prioritaet.
Verbindet `positions` linear (`[(0,1), (1,2), ...]`) und weist jeder Verbindung
denselben `direction`/`priority`-Stil zu. Gemeinsames Konstruktor-Pattern aller
Route-Tool-`preview()`-Methoden (`StraightLineTool`, `CurveTool`, `SmoothCurveTool`, …).

---

### `impl_lifecycle_delegation_no_seg!` (Makro)

Liegt in `app/tools/common/lifecycle.rs`. Reduziert Boilerplate fuer Tools
**ohne** `self.seg: SegmentConfig` (konkret: `BypassTool`, `ParkingTool`,
`FieldBoundaryTool`, `RouteOffsetTool`).

Das Makro implementiert innerhalb eines `impl RouteTool for X { ... }`-Blocks:

- `set_direction(dir)` / `set_priority(prio)` / `set_snap_radius(radius)`
- `last_created_ids() -> &[u64]`
- `last_end_anchor() -> Option<ToolAnchor>`
- `needs_recreate() -> bool` / `clear_recreate_flag()`
- `set_last_created(ids, road_map)`

**Voraussetzung:** Der Ziel-Typ muss `self.lifecycle: ToolLifecycleState`,
`self.direction: ConnectionDirection` und `self.priority: ConnectionPriority` besitzen.

```rust
impl RouteTool for BypassTool {
    crate::impl_lifecycle_delegation_no_seg!();
    // ... weitere Methoden
}
```

Analoges Pendant fuer Tools MIT `SegmentConfig` ist `impl_lifecycle_delegation!`
(ohne `_no_seg`-Suffix) — dokumentiert in `common/lifecycle.rs`.

### Direkte Erweiterungspunkte

Der optionale Teil des Vertrags wird direkt ueber Default-Methoden auf `RouteTool` abgebildet. Es gibt keine separaten Capability-Traits mehr.

- Drag/Nachbearbeitung: `drag_targets`, `on_drag_start`, `on_drag_update`, `on_drag_end`
- Tangenten: `tangent_menu_data`, `apply_tangent_selection`
- GroupRegistry: `make_group_record`, `load_for_edit`
- Ketten-Input: `needs_chain_input`, `load_chain`, `set_chain_inner_ids`
- Analyse-Input: `set_farmland_data`, `set_farmland_grid`, `set_background_map_image`, `needs_lasso_input`, `on_lasso_completed`

**Registry-Erweiterungen** (fuer `GroupRegistry`, siehe [`../use_cases/API.md`](../use_cases/API.md)):

```rust
// Wird nach execute() + apply_tool_result() aufgerufen:
fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord>;

// Wird in edit_segment() aufgerufen um das Tool wiederherzustellen:
fn load_for_edit(&mut self, record: &GroupRecord, kind: &GroupKind);
```

---

## `GroupBase` und `GroupKind` (Group-Registry)

Documentation moved to [`../API.md#groupbase--groupkind`](../API.md#groupbase--groupkind). Kurz: Alle Gruppen speichern ihre grundlegenden Parameter (Richtung, Priorität, Max-Abstand) in `GroupBase` ab, was Tool-Typ und Editing-Flow vereinheitlicht.

---

## Registrierte Tools

Der kanonische Katalog beschreibt jedes Tool ueber `RouteToolId`, Anzeigegruppe, Anforderungen und Persistenzvertrag:

| Tool-ID | Gruppe | Voraussetzungen | Backing | Konstruktor |
|---------|--------|-----------------|---------|-------------|
| `RouteToolId::Straight` | `Basics` | keine | `GroupBackedEditable` | `StraightLineTool::new()` |
| `RouteToolId::CurveQuad` | `Basics` | keine | `GroupBackedEditable` | `CurveTool::new()` |
| `RouteToolId::CurveCubic` | `Basics` | keine | `GroupBackedEditable` | `CurveTool::new_cubic()` |
| `RouteToolId::Spline` | `Basics` | keine | `GroupBackedEditable` | `SplineTool::new()` |
| `RouteToolId::Bypass` | `Section` | geordnete Kette | `GroupBackedEditable` | `BypassTool::new()` |
| `RouteToolId::SmoothCurve` | `Basics` | keine | `GroupBackedEditable` | `SmoothCurveTool::new()` |
| `RouteToolId::Parking` | `Section` | keine | `GroupBackedEditable` | `ParkingTool::new()` |
| `RouteToolId::FieldBoundary` | `Analysis` | Farmland geladen | `GroupBackedEditable` | `FieldBoundaryTool::new()` |
| `RouteToolId::FieldPath` | `Analysis` | Farmland geladen | `Ephemeral` | `FieldPathTool::new()` |
| `RouteToolId::RouteOffset` | `Section` | geordnete Kette | `GroupBackedEditable` | `RouteOffsetTool::new()` |
| `RouteToolId::ColorPath` | `Analysis` | Hintergrundbild geladen | `Ephemeral` | `ColorPathTool::new()` |

### `StraightLineTool`

Gerade Strecke zwischen zwei Punkten mit konfigurierbarem Nodeabstand.

### `CurveTool`

Bézier-Kurve wahlweise Grad 2 (quadratisch, 1 Steuerpunkt) oder Grad 3 (kubisch, 2 Steuerpunkte). `name()` und `description()` sind grad-spezifisch.

Konstruktoren: `CurveTool::new()` (Grad 2), `CurveTool::new_cubic()` (Grad 3).

Modulstruktur: `state.rs` (Enums, Struct, Ctors), `lifecycle.rs` (RouteTool-Impl), `drag.rs` (Drag-Logik), `config_ui.rs` (egui-Panel), `geometry.rs` (Bézier-Mathe), `tests.rs`

**Cubic-Extras (Grad 3):**

- **Auto-Tangente:** Beim Eintreten in Phase::Control wird automatisch der beste Start-Nachbar gewaehlt (bevorzugt eingehende Verbindungen; max `dot(continuation_dir, chord_dir)`). CP1 und CP2 werden sofort gesetzt.
- **Virtueller Scheitelpunkt (Apex):** `virtual_apex = B(0.5)` wird als fuenftes Drag-Handle angeboten. Drag verschiebt B(0.5) und passt CP1/CP2 via inverser Bézier-Formel an:
  - Mit Start-Tangente: CP1 fixiert → nur CP2 = `(8·apex − P0 − 3·CP1 − P3) / 3`
  - Mit End-Tangente: CP2 fixiert → nur CP1 = `(8·apex − P0 − 3·CP2 − P3) / 3`
  - Ohne Tangente: beide CPs symmetrisch aus Apex (`cps_from_apex_symmetric`)

### `SplineTool`

Catmull-Rom-Spline: interpolierende Kurve durch alle geklickten Punkte. Beliebig viele Kontrollpunkte, fortlaufende Vorschau (Cursor als naechster Punkt), Enter bestaetigt. Nachbearbeitung (Segment-Laenge/Node-Anzahl) und Verkettung unterstuetzt.

### `SmoothCurveTool`

Winkelgeglaettete Route mit automatischen Tangenten-Uebergaengen. Solver-Pipeline:

1. **Approach-Steerer:** Auto-Steuerpunkt am Start (Dot-Product-basierte Nachbar-Auswahl, dynamischer Abstand)
2. **User-Kontrollpunkte:** Beliebig viele Zwischen-Kontrollpunkte (Phase::ControlNodes)
3. **Departure-Steerer:** Analog am Ende
4. **Subdivide:** Gleichmaessige Unterteilung aller Segmente auf `max_segment_length`
5. **Chaikin-Corner-Cutting:** Iteratives Glaetten der schaerfsten Ecke (mit Re-Subdivision nach jedem Cut)
6. **Resampling:** Finale Punkte auf gleichmaessige Abstaende

**Steuerpunkte (Steerer):**

- Automatisch berechnet aus Nachbar-Richtungen (`connected_neighbors`)
- Im UI als Steuerpunkte angezeigt (Config-Panel + Viewport)
- Per Drag im Viewport verschiebbar (wird dann als manuell markiert)
- Reset-Button (↺) im Config-Panel setzt auf Auto-Berechnung zurueck
- Manuell verschobene Steuerpunkte werden als Kontrollpunkte an den Solver uebergeben

**Solver-Typen:**

- `SolverResult` — Positionen + optionale Approach/Departure-Steuerpunkte
- `SmoothCurveInput` — Eingabeparameter fuer den Solver
- Oeffentliche Helper-Exporte: `smooth_curve::solve_route`, `smooth_curve::SmoothCurveInput` (u.a. fuer Benchmarks)

**Solver-Parameter:**

- `max_angle_deg: f32` (5°..135°) — Maximale Richtungsaenderung pro Segment
- `max_segment_length: f32` — via `SegmentConfig`

**Drag-Targets:** Start, End, ApproachSteerer, DepartureSteerer, Control(i)

**Phasen:** `Start` → `End` → `ControlNodes` (Enter bestaetigt)

**Lifecycle-Verbesserung (2026-03-05):** Vereinfachte `current_end_anchor()` Logik nach Lifecycle-State-Refactoring — `last_end_anchor` wird nur noch aus dem gemeinsamen `ToolLifecycleState` bezogen (nicht mehr redundant auf dem Tool-Struct).

Modulstruktur: `state.rs`, `lifecycle.rs`, `geometry.rs`, `drag.rs`, `config_ui.rs`, `tests.rs`

### `ParkingTool`

Parkplatz-Layout-Generator: Erstellt einen Wendekreis mit Parkreihen in einem konfigurierbaren Raster-Layout.

**Neuer Interaktionsflow (2026-03-05):**

- **Phase::Idle** — Wartet auf Eingabe
- **Rotation (Alt+Scroll):** `on_scroll_rotate()` callback — aendert `self.angle` kontinuierlich (Viewport-Steuerung)
- **Phase::Placing (Klick):** Setzt Origin + fixiert Winkel → wandelt zu `Phase::Configuring` um
- **Phase::Configuring:** Config-Panel aktiv; Buttons zum Anpassen der Layout-Parameter (Reihen, Abstände, etc.)
- **Phase::Adjusting (Viewport-Klick):** Repositionierung des Origins — Klick setzt neue Position, zurueck zu `Phase::Configuring`
- **Execute (Confirm-Button):** Nur in `Phase::Configuring` ausfuehrbar → erstellt Nodes + Connections

**Phasen-Enum:** `ParkingPhase { Idle, Placing, Configuring, Adjusting }`

**Felder:**

- `phase: ParkingPhase` — Aktueller Interaktions-Status
- `origin: Option<Vec2>` — Position des Parkplatz-Zentrums (gesetzt in Phase::Placing)
- `angle: f32` — Rotationswinkel (Radiant; wird via `on_scroll_rotate()` angepasst)
- `config: ParkingConfig` — Parkplatz-Konfiguration (Reihen, Abstände, Wendkreis-Radius)
  - `entrance_side: EntranceSide` — Einfahrts-Seite (Left/Right)
  - `row_count: usize` — Anzahl Parkreihen
  - `spacing_length: f32` — Laengsabstand zwischen Parktaschen
  - `spacing_width: f32` — Querabstand zwischen Reihen
  - `turnaround_radius: f32` — Wendekreis-Radius
  - `connection_spacing: f32` — Verbindungs-Node-Abstände
- `direction: ConnectionDirection` — Richtung fuer die erzeugten Verbindungen
- `priority: ConnectionPriority` — Prioritaet fuer die erzeugten Verbindungen

**Lifecycle-Integration:**

- Enthaelt gemeinsamen `ToolLifecycleState` fuer Snap-Radius, letzte erstellte Node-IDs, Recreate-Flag
- Methoden: `set_snap_radius()`, `last_created_ids()`, `last_end_anchor()`, `needs_recreate()`, `clear_recreate_flag()`, `set_last_created()`

**Group-Registry:**

- Ueberschreibt die `RouteTool`-Hooks `make_group_record()` und `load_for_edit()`
- Speichert Layout-Parameter fuer nachtraegliche Bearbeitung

**Public Exports:**

- `generate_parking_layout(config) → ParkingLayout` — Generiert das Layout (fuer Tests)
- `build_parking_result(layout, origin, angle, ...) → Vec<Vec2>` — Konvertiert zu Positionen
- `build_preview(layout, origin, angle, ...) → (Vec<Vec2>, Vec<(usize, usize)>)` — Vorschau-Geometrie

Modulstruktur: `state.rs` (Struct + Config), `lifecycle.rs` (RouteTool-Impl + Lifecycle-Delegation), `config_ui.rs` (egui-Panel), `geometry/{mod,layout,blueprint,conversion}.rs` (Layout-Mathe), `tests.rs` (7 Unit-Tests)

### `FieldBoundaryTool`

Felderkennung: Erkennt das GRLE-Farmland-Polygon an der Klickposition und erzeugt einen geschlossenen Waypoint-Ring entlang des Feldumrisses.

**Voraussetzung:** `farmland_polygons` muss im `AppState` geladen sein (wird bei Overview-Generierung aus dem Map-ZIP befuellt). Der Toolbar-Button und der Kontextmenue-Eintrag sind deaktiviert wenn keine Farmland-Daten vorliegen.

**Phasen:**

- **`FieldBoundaryPhase::Idle`** — Wartet auf Klick in ein Feld
- **`FieldBoundaryPhase::Configuring`** — Feldgrenze erkannt, Vorschau aktiv

**Interaktionsflow:**

1. Klick im Idle-Zustand → `find_polygon_at()` → selektiertes Feld gespeichert → Phase::Configuring
2. Config-Panel: Node-Abstand, Versatz, Begradigen (Douglas-Peucker), Richtung, Prioritaet
3. Erneuter Klick im Configuring-Zustand → Phase::Idle (Auswahl zuruecksetzen)
4. Confirm-Button → `execute()` → geschlossener Ring mit N Nodes und N Verbindungen (0→1, 1→2, …, N−1→0)

**Konfiguration:**

- `node_spacing: f32` — Abstand zwischen Nodes (1–50 m; Standard 10 m)
- `offset: f32` — Versatz nach innen (<0) oder aussen (>0) in Metern (−20..+20)
- `straighten_tolerance: f32` — Douglas-Peucker-Toleranz (0..10 m; 0 = keine Vereinfachung)
- `corner_rounding_enabled: bool` — Aktiviert Kreisbogen-Verrundung erkannter Ecken
- `corner_rounding_radius: f32` — Verrundungsradius in Metern (0.5–20 m; Standard 5 m)
- `direction: ConnectionDirection` — Verbindungsrichtung (Standard: Dual)
- `priority: ConnectionPriority` — Verbindungsprioriaet (Standard: Regular)

**Felder:**

```rust
pub struct FieldBoundaryTool {
    pub(crate) phase: FieldBoundaryPhase,
    pub(crate) selected_polygon: Option<FieldPolygon>,
    pub(crate) farmland_data: Option<Arc<Vec<FieldPolygon>>>,
    pub(crate) node_spacing: f32,
    pub(crate) offset: f32,
    pub(crate) straighten_tolerance: f32,
    pub(crate) corner_rounding_enabled: bool,
    pub(crate) corner_rounding_radius: f32,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
}
```

**Interne Ring-Berechnung (`compute_ring`):**

Signatur:
```rust
pub(super) fn compute_ring(
    vertices: &[Vec2],
    offset: f32,
    tolerance: f32,
    spacing: f32,
    corner_angle: Option<f32>,
    rounding_radius: Option<f32>,
) -> Vec<(Vec2, RingNodeKind)>
```
1. `offset_polygon()` — Polygon nach innen/aussen verschieben
2. `simplify_polygon()` — Douglas-Peucker vereinfachen
3. `detect_corners()` — Eckpunkte anhand Ablenkungswinkel finden
4. `resample_ring_with_corners()` — Ring segmentweise neu samplen; Ecken als Anker, dazwischen gleichmaessig, optionaler Kreisbogen mit `rounding_radius`
5. Rückgabe: `Vec<(Vec2, RingNodeKind)>` — jeder Punkt mit geometrischer Klassifikation

**`RingNodeKind`-Enum (aus `geometry.rs`):**

```rust
pub enum RingNodeKind {
    Regular,        // Normaler Punkt zwischen Ecken
    Corner,         // Erkannter Eckpunkt (Anker des Bogens)
    RoundedCorner,  // Punkt auf einem Kreisbogen (wird als NodeFlag::RoundedCorner gespeichert)
}
```

Das Mapping `RingNodeKind` → `NodeFlag`:
- `RingNodeKind::RoundedCorner` → `NodeFlag::RoundedCorner` (6, intern; XML-Export: 0)
- `RingNodeKind::Regular | Corner` → `NodeFlag::Regular` (0)

**Geometrie-Funktionen (`tools/field_boundary/geometry.rs`):**

- `detect_corners(vertices, angle_threshold_rad) -> Vec<usize>` — Sortierte Indizes aller Eckpunkte mit Ablenkungswinkel ≥ Schwellwert
- `round_corner(prev, corner, next, radius, spacing) -> Vec<Vec2>` — Kreisbogen zwischen Tangentenpunkten einer konvexen Ecke. Konkave Ecken (Cross-Product ≤ 0) werden unverändert zurückgegeben. Tangentenpunkte begrenzt auf 40% der Kantenlaenge.
- `resample_ring_with_corners(simplified, corner_indices, spacing, rounding_radius) -> Vec<(Vec2, RingNodeKind)>` — Resampled den Ring segmentweise mit Ecken als festen Ankern

**Gruppen-Record:** `GroupKind::FieldBoundary { field_id, node_spacing, offset, straighten_tolerance, corner_angle_threshold, corner_rounding_radius, base }` unter `RouteToolId::FieldBoundary`

Modulstruktur: `mod.rs` (Re-Exporte), `state.rs` (Struct, Phasen-Enum, Default), `lifecycle.rs` (RouteTool-Impl, Ring-Berechnung), `config_ui.rs` (egui-Panel), `geometry.rs` (RingNodeKind, detect_corners, round_corner, resample_ring_with_corners)

### `RouteOffsetTool`

Parallelversatz einer selektierten Kette ohne S-Kurven-Anbindung (`RouteToolId::RouteOffset`). Generiert eine oder zwei Parallel-Versatz-Ketten (links und/oder rechts) mit konfigurierbarem Abstand und optionalem Entfernen der Original-Kette.

**Voraussetzung:** Eine selektierte Kette muss beim Aktivieren des Tools vorhanden sein (`load_chain()` wird automatisch in `init_chain_if_needed()` aufgerufen).

**Phasen:** Kein Phasenmodell — Tool ist direkt nach Ketten-Laden bereit (Enter = Ausführen).

**Konfiguration (`OffsetConfig`):**

- `left_enabled: bool` — Links-Versatz aktiviert
- `right_enabled: bool` — Rechts-Versatz aktiviert
- `left_distance: f32` — Versatz-Distanz links in Metern (Standard: 8 m)
- `right_distance: f32` — Versatz-Distanz rechts in Metern (Standard: 8 m)
- `keep_original: bool` — Original-Kette beibehalten (false = Original-Nodes entfernen)
- `base_spacing: f32` — Maximaler Abstand zwischen Nodes auf der Offset-Kette (Standard: 6 m)

**Felder:**

```rust
pub struct RouteOffsetTool {
    pub(crate) chain_positions: Vec<Vec2>,   // Geordnete Positionen der Quell-Kette
    pub(crate) chain_start_id: u64,          // ID des ersten Ketten-Nodes
    pub(crate) chain_end_id: u64,            // ID des letzten Ketten-Nodes
    pub(crate) chain_inner_ids: Vec<u64>,    // IDs innerer Nodes (fuer "Original entfernen")
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub config: OffsetConfig,
    pub(crate) lifecycle: ToolLifecycleState,
}
```

**Execute-Logik:**

1. `compute_offset_positions(chain, ±distance, base_spacing)` — verschobene Punkte berechnen
2. `resample_by_distance()` — gleichmäßiges Resampling
3. Wenn `!keep_original`: `nodes_to_remove = chain_inner_ids` → Original-Nodes werden im selben Undo-Schritt entfernt
4. `ToolResult.nodes_to_remove` wird von `apply_tool_result()` vor Erstellung neuer Nodes verarbeitet

**Geometrie-Funktionen (`geometry.rs`):**

- `compute_offset_positions(chain, offset, base_spacing) → Option<Vec<Vec2>>` — Nutzt `parallel_offset()` + `resample_by_distance()`

**Public Exports (`mod.rs`):**

- `RouteOffsetTool`
- `compute_offset_positions()` — Hotpath-Helfer fuer Benchmarks/Tests

**Gruppen-Record:** `GroupKind::RouteOffset { chain_positions, chain_start_id, chain_end_id, offset_left, offset_right, keep_original, base_spacing, base }` unter `RouteToolId::RouteOffset`

Modulstruktur: `mod.rs` (Re-Exporte), `state.rs` (Struct + OffsetConfig), `lifecycle.rs` (RouteTool-Impl), `geometry.rs` (compute_offset_positions), `config_ui.rs` (egui-Panel), `tests.rs`

---

### `FieldPathTool`

Feldweg-Erkennung: Berechnet eine Mittellinie zwischen zwei Farmland-Seiten und erzeugt daraus eine gleichmäßig abgetastete Waypoint-Route (`RouteToolId::FieldPath`).

**Voraussetzung:** `farmland_grid` und `farmland_polygons` müssen im `AppState` geladen sein (werden beim Laden der Overview aus dem Map-ZIP befüllt).

**Modi:**

- **`FieldPathMode::Fields`** — ganze Farmland-Polygone pro Seite auswählen (Klick ins Feld)
- **`FieldPathMode::Boundaries`** — einzelne Feldgrenz-Segmente pro Seite auswählen (Klick auf Grenzlinie)

**Phasen:**

- **`FieldPathPhase::Idle`** — Warten auf Nutzerinteraktion
- **`FieldPathPhase::SelectingSide1`** — Felder oder Grenzsegmente für Seite 1 sammeln
- **`FieldPathPhase::SelectingSide2`** — Felder oder Grenzsegmente für Seite 2 sammeln
- **`FieldPathPhase::Preview`** — Berechnung abgeschlossen, Vorschau der Mittellinie aktiv

**Konfiguration (`FieldPathConfig`):**

- `node_spacing: f32` — Abstand zwischen generierten Nodes in Metern (Standard: 5 m)
- `simplify_tolerance: f32` — Toleranz für Douglas-Peucker-Vereinfachung in Metern (Standard: 1 m)
- `connect_to_existing: bool` — An nächste bestehende Nodes anschließen (Standard: true)

**Felder:**

```rust
pub struct FieldPathTool {
    pub(crate) mode: FieldPathMode,
    pub(crate) phase: FieldPathPhase,
    pub(crate) config: FieldPathConfig,

    pub(crate) side1_field_ids: Vec<u32>,     // Ausgewählte Farmland-IDs für Seite 1
    pub(crate) side2_field_ids: Vec<u32>,     // Ausgewählte Farmland-IDs für Seite 2
    pub(crate) side1_segments: Vec<Vec<Vec2>>, // Ausgewählte Grenzsegmente für Seite 1
    pub(crate) side2_segments: Vec<Vec<Vec2>>, // Ausgewählte Grenzsegmente für Seite 2

    pub(crate) centerline: Vec<Vec2>,          // Vereinfachte Mittellinie (Welt-Koordinaten)
    pub(crate) resampled_nodes: Vec<Vec2>,     // Gleichmäßig abgetastete Nodes der Mittellinie
    pub(crate) voronoi_cache: Option<Arc<VoronoiGrid>>, // Voronoi-BFS-Cache (invalidiert bei Grid-Änderung)

    pub(crate) farmland_grid: Option<Arc<FarmlandGrid>>,
    pub(crate) farmland_polygons: Option<Arc<Vec<FieldPolygon>>>,
    pub(crate) background_image: Option<Arc<DynamicImage>>, // Reserviert für spätere Analysen

    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
}
```

**Interne Berechnungs-Pipeline (`compute_centerline`):**

1. Voronoi-BFS auf dem Farmland-Grid berechnen (oder aus Cache verwenden)
2. Im `Fields`-Modus: `extract_corridor_centerline()` zwischen side1/side2 Farmland-IDs
3. Im `Boundaries`-Modus: `extract_boundary_centerline()` aus rasterisierten Grenzsegmenten
4. `simplify_polyline()` mit Douglas-Peucker
5. `resample_by_distance()` für gleichmäßigen Nodeabstand
6. Ergebnis in `centerline` und `resampled_nodes` speichern; Phase → Preview

**Boundary-Snap:**

Im `Boundaries`-Modus sucht `find_nearest_boundary_segment()` das nächste Polygon-Kanten-Segment innerhalb von `BOUNDARY_SNAP_THRESHOLD = 20 m` und gibt es als Zwei-Punkt-Polyline zurück.

Modulstruktur: `mod.rs` (Re-Export), `state.rs` (Structs, Enums, Felder), `lifecycle.rs` (RouteTool-Impl, compute_centerline), `config_ui.rs` (egui-Panel)

---

### `ColorPathTool`

Farb-Pfad-Erkennung: Erkennt zusammenhaengende Teilnetze anhand der Farbe im Hintergrundbild, skelettiert sie per Zhang-Suen-Thinning und exportiert daraus ein Waypoint-Netz mit offenen Enden, Kreuzungen und Segmenten (`RouteToolId::ColorPath`).

**Voraussetzung:** Ein Hintergrundbild muss geladen sein (`set_background_map_image()`). Das Tool bezieht die `map_size` automatisch aus `set_farmland_grid()`.

**Phasen (`ColorPathPhase`):**

- **`Idle`** — Warten auf Nutzerinteraktion (Klick oder Alt+Lasso startet Sampling)
- **`Sampling`** — User sammelt Farbproben per Alt+Lasso; Berechnen-Button startet Pipeline
- **`Preview`** — Teilnetz berechnet und als Vorschau angezeigt; Enter fuegt Nodes ein

**ToolLasso-Mechanismus:**

Das Tool setzt `needs_lasso_input() = true` sobald `phase == Sampling`. Damit wird
jeder Alt+Drag im Viewport als `DragSelectionMode::ToolLasso` geroutet (statt als
normale Node-Selektion). Das abgeschlossene Polygon wird per `AppIntent::RouteToolLassoCompleted`
an `handlers::route_tool::lasso_completed()` weitergeleitet, das `on_lasso_completed()` aufruft.

**Interaktionsflow:**

1. Tool aktivieren → Phase::Idle
2. Alt+Drag zeichnet ein Lasso-Polygon → `on_lasso_completed()` sampelt Farben im Polygon
3. Mehrere Lasso-Polygone moeglich (Sampling kumulativ)
4. Sidebar: Berechnen-Button → `compute_pipeline()` → Phase::Preview
5. Sidebar: Netz pruefen (Kreuzungen, offene Enden, Segmente) + Anschlussmodus waehlen
6. Enter / Uebernehmen-Button → `execute()` → Graph in Road Map einfuegen

**Erkennungs-Pipeline (`compute_pipeline()`):**

1. `flood_fill_color_mask()` — Bool-Maske des zusammenhaengenden Farb-Bereichs ab Lasso-Startpunkt
2. Morphologisches Opening + Closing (wenn `noise_filter == true`) — Rauschen entfernen
3. Original-Maske sichern (vor Zhang-Suen, fuer Medial-Axis-Korrektur)
4. `zhang_suen_thinning()` — Maske auf 1-Pixel-breites Skelett reduzieren
5. `find_connected_components()` — Zusammenhaengende Skelett-Gruppen finden (8-Connectivity)
6. Pixelgrad klassifizieren: offene Enden (`degree == 1`), Kettenpixel (`degree == 2`), Junctions (`degree >= 3`)
7. Benachbarte Junction-Pixel zu Clustern zusammenfassen und pro Cluster einen zentralen Knoten berechnen
8. Zwischen den Graph-Knoten alle Grad-2-Ketten als Segmente verfolgen und per `refine_medial_axis()` in Weltkoordinaten umrechnen
9. Pro Segment `simplify_polyline()` + `resample_by_distance()` anwenden
10. Preview-Netz aufbauen und fuer Export bereithalten

**Preview/Export:**

- Preview zeigt Kennzahlen fuer Kreuzungen, offene Enden, Segmente und Preview-Nodes
- Sampling-Preview zeigt nach jeder Lasso-Auswahl alle Randsegmente der Flood-Fill-Maske, nicht nur eine Einzelkontur
- Export legt Junction-/End-Knoten genau einmal an und fuegt pro Segment nur die Zwischenpunkte neu ein
- Bestandsanschluss nutzt `ToolLifecycleState::snap_at()` und damit den konfigurierten Snap-Radius

**Konfiguration (`ColorPathConfig`):**

- `exact_color_match: bool` — Exaktmodus; matcht nur auf exakt gelasso-te RGB-Farben und deaktiviert die Toleranz-UI (Standard: `true`)
- `color_tolerance: f32` — Farb-Toleranz im unscharfen Modus (euklidischer RGB-Abstand; Standard: 25.0, Bereich: 1–80)
- `node_spacing: f32` — Abstand zwischen generierten Nodes in Metern (Standard: 5.0, Bereich: 1–50)
- `simplify_tolerance: f32` — Douglas-Peucker-Toleranz in Metern (Standard: 1.0, Bereich: 0–20)
- `noise_filter: bool` — Morphologischen Rauschfilter aktivieren (Standard: true)
- `existing_connection_mode: ExistingConnectionMode` — Bestandsanschluss: `Never`, `OpenEnds`, `OpenEndsAndJunctions` (Standard: `OpenEnds`)
- `detection_bounds: Option<(Vec2, Vec2)>` — Begrenzt Farberkennung auf eine Rect-Region (geplant)

**Felder:**

```rust
pub struct ColorPathTool {
    pub(crate) phase: ColorPathPhase,
    pub(crate) config: ColorPathConfig,
    pub(crate) lasso_regions: Vec<Vec<Vec2>>,   // Alle gezeichneten Lasso-Polygone (Weltkoords)
    pub(crate) sampled_colors: Vec<[u8; 3]>,    // Gesammelte RGB-Werte aus Lasso-Regionen
    pub(crate) avg_color: Option<[u8; 3]>,      // Berechneter RGB-Mittelwert
    pub(crate) lasso_start_world: Option<Vec2>, // Erster Klickpunkt des ersten Lassos (Weltkoords)
    pub(crate) mask: Vec<bool>,                  // Bool-Maske (true = Pfadpixel), zeilenweise
    pub(crate) mask_width: u32,
    pub(crate) mask_height: u32,
    pub(crate) skeleton_network: Option<SkeletonNetwork>, // Junctions + offene Enden + Segmente
    pub(crate) prepared_segments: Vec<PreparedSegment>,   // Vereinfachte/abgetastete Preview-Segmente
    pub(crate) background_image: Option<Arc<image::DynamicImage>>,
    pub(crate) map_size: f32,                   // Kartengroesse in Metern (aus FarmlandGrid)
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
}
```

**Sampling-Funktionen (`sampling.rs`):**

- `world_to_pixel(world, map_size, img_w, img_h) → (u32, u32)` — Weltkoords → Bildpixel
- `pixel_to_world(px, py, map_size, img_w, img_h) → Vec2` — Bildpixel → Weltkoords (X und Y je mit korrektem Skalierungsfaktor)
- `pixel_to_world_f32(px, py, map_size, img_w, img_h) → Vec2` — Sub-Pixel-Position → Weltkoords (fuer Medial-Axis)
- `sample_colors_in_polygon(polygon, image, map_size) → Vec<[u8; 3]>` — RGB-Pixel im Lasso-Polygon sammeln
- `compute_average_color(colors) → [u8; 3]` — RGB-Mittelwert aller Samples
- `build_exact_color_set(raw_colors) → Vec<[u8; 3]>` — Eindeutige Rohfarben fuer Exaktmodus ohne Quantisierung
- `build_color_mask(image, avg_color, tolerance, bounds, map_size) → (Vec<bool>, u32, u32)` — Bool-Maske erstellen
- `extract_boundary_segments_from_mask(mask, w, h, map_size) → Vec<(Vec2, Vec2)>` — Alle Grenzen der Flood-Fill-Maske als Preview-Segmente (inkl. Innenkanten/Loecher)
- `erode(mask, w, h) → Vec<bool>` — Erosion mit Majority-Bedingung (≥ 3 von 4 Nachbarn, zum Schutz duenner Verbindungen)
- `dilate(mask, w, h) → Vec<bool>` — Dilatation (4-Connectivity)
- `morphological_open(mask, w, h) → Vec<bool>` — Erosion + Dilatation (Rauschen entfernen)
- `morphological_close(mask, w, h) → Vec<bool>` — Dilatation + Erosion (Luecken schliessen)

**Skelett-Funktionen (`skeleton.rs`):**

- `find_connected_components(mask, w, h) → Vec<Vec<(usize, usize)>>` — Gruppen nach Groesse absteigend
- `refine_medial_axis(ordered, original_mask, w, h) → Vec<(f32, f32)>` — Skelett-Pixel auf geometrische Mittelachse korrigieren
- `extract_network_from_mask(mask, w, h, noise_filter, map_size, start_hint) → SkeletonNetwork` — Haupt-Pipeline fuer Netz-Knoten und Segmente
- `extract_paths_from_mask(...) → Vec<Vec<Vec2>>` — Legacy-Wrapper fuer lineare Pfad-Konsumenten/Tests

**Public Exports (`mod.rs`):**

- `ColorPathTool`
- `compute_color_path_network_stats()` — Flood-Fill + Netzextraktion fuer Benchmarks/Analyse, ohne interne Skelett-Typen offenzulegen

**Gruppen-Record:** ColorPathTool speichert keinen `GroupRecord` (keine nachträgliche Bearbeitung).

Modulstruktur: `mod.rs` (Re-Export), `state.rs` (Struct, Phasen-Enum, Config, Default), `lifecycle.rs` (RouteTool-Impl, Netz-Pipeline, Export), `config_ui.rs` (egui-Panel), `sampling.rs` (Farb-Sampling + Masken-Erstellung), `skeleton.rs` (Skelett-Extraktion + Graph-Aufbau)

---

### `BypassTool`

Parallele Ausweichstrecke einer selektierten Kette mit S-förmigen An-/Abfahrten. Das Tool benötigt eine Eingabe-Kette (via `load_chain()`), generiert dann automatisch die Bypass-Positionen und erstellt neue Nodes mit entsprechenden Verbindungen.

**Input-Modus:** Chain-basiert (nutzt die `RouteTool`-Hooks `needs_chain_input()` und `load_chain()`).

- `needs_chain_input() → true`
- `load_chain(positions, start_id, end_id)` — Laedt die Kette aus der User-Selektion

**Konfiguration:**

- `offset: f32` — Seitlicher Versatz in Welteinheiten (positiv = links, negativ = rechts)
- `base_spacing: f32` — Abstand zwischen Nodes auf der Hauptstrecke
- `direction: ConnectionDirection` — Richtung fuer die erzeugten Verbindungen
- `priority: ConnectionPriority` — Prioritaet fuer die erzeugten Verbindungen

**Caching:**

- `cached_positions` — Gecachte Bypass-Positionen (wird invalidiert bei Config-Aenderung)
- `cached_connections` — Gecachte Preview-Connections inkl. Start/End-Anker

**Lifecycle-Integration:**

- Enthaelt gemeinsamen `ToolLifecycleState` fuer Snap-Radius, letzte erstellte Node-IDs, Recreate-Flag
- Methoden: `set_snap_radius()`, `last_created_ids()`, `last_end_anchor()`, `needs_recreate()`, `clear_recreate_flag()`, `set_last_created()`
- Nutzt `lifecycle.save_created_ids()` zur Verwaltung erstellter IDs

**Public Exports:**

- `compute_bypass_positions(chain, offset, base_spacing) → Option<(Vec<Vec2>, f32)>` — Berechnet Bypass-Positionen und Uebergangslaenge (fuer Benchmarks + Tests)

Modulstruktur: `state.rs` (Struct + Config), `lifecycle.rs` (RouteTool-Impl + Lifecycle-Delegation), `config_ui.rs` (egui-Panel), `geometry.rs` (Bypass-Mathe), `tests.rs` (15 Unit-Tests)

---

## Gemeinsame Tool-Infrastruktur (`tools/common/`)

Aufgeteilt in vier Submodule (alle privat, Re-Exporte via `common/mod.rs`):

### `geometry.rs`

Hilfsfunktionen: `angle_to_compass`, `node_count_from_length`, `populate_neighbors`, `snap_with_neighbors`, `linear_connections`, `tangent_options`

**Polyline-Geometrie** (gemeinsam fuer BypassTool + RouteOffsetTool):

- **`parallel_offset(polyline, offset) → Vec<Vec2>`** — Berechnet eine parallel versetzte Polyline. `offset > 0` = links (positive Senkrechte in Fahrtrichtung), `offset < 0` = rechts.
- **`local_perp(i, poly) → Vec2`** — Lokale Senkrechte am Index `i` einer Polyline (Durchschnitt benachbarter Segmente; Randpunkte nutzen nur das angrenzende Segment).

### `tangent.rs`

**`TangentSource`** — Tangenten-Quelle am Start-/Endpunkt (fuer Curve + Spline):

- `None` — Kein Tangenten-Vorschlag
- `Connection { neighbor_id, angle }` — Tangente aus bestehender Verbindung

**`render_tangent_combo(ui, id_salt, label, none_label, current, neighbors) → bool`** — Gemeinsamer UI-Baustein fuer Tangenten-ComboBoxen (verwendet von Curve + Spline config_ui).

### `lifecycle.rs`

**`ToolLifecycleState`**, **`SegmentConfig`**, **`LastEdited`**

**`SegmentConfig`** — Gekapselte Konfiguration fuer Segment-Laenge und Node-Anzahl:

- `max_segment_length: f32` — Maximaler Abstand zwischen Zwischen-Nodes
- `node_count: usize` — Gewuenschte Anzahl Nodes (inkl. Start+End)
- `last_edited: LastEdited` — Welcher Wert zuletzt geaendert wurde (bestimmt Sync-Richtung)
- `sync_from_length(length)` — Synchronisiert abhaengigen Wert aus Streckenlaenge

**`ToolLifecycleState`-Methoden:**

- `save_created_ids(&mut self, ids: &[u64])` — Speichert erstellte Node-IDs und setzt das Recreate-Flag zurueck
- `has_last_created() → bool` — Prueft ob letzte erstellte IDs vorhanden sind
- `chaining_start_anchor() → Option<ToolAnchor>` — Gibt den End-Anker fuer die Verkettung zurueck, wobei `NewPosition` zu `ExistingNode` hochgestuft wird (verhindert doppelte Nodes am Verkettungspunkt)
- `prepare_for_chaining()` — Setzt den Lifecycle-State fuer die naechste Verkettung zurueck (DRY-Hilfsmethode)

**`render_segment_config_3modes(seg, ui, adjusting, ready, length, label, distance_wheel_step_m) → (changed, recreate)`** — Gemeinsame Hilfsfunktion fuer die 3 SegmentConfig-Darstellungsmodi (Adjusting/Live/Default) inkl. Mausrad-Aenderungen fuer Distanz/Node-Anzahl.

**`impl_lifecycle_delegation!`** — Makro zur Delegation der Standard-RouteTool-Lifecycle-Methoden (`set_direction`, `set_priority`, `set_snap_radius`, `last_created_ids`, `last_end_anchor`, `needs_recreate`, `clear_recreate_flag`, Inkrement/Decrement-Helfer) an die gemeinsamen Felder. Eliminiert Boilerplate pro Tool.

Der Makro-Flow fuer `set_last_created` ist vereinheitlicht:

1. `current_end_anchor()` uebernehmen
2. `save_anchors_for_recreate(road_map)` aufrufen
3. `lifecycle.save_created_ids(ids)` ausfuehren

### `bypass::geometry` Export

- Oeffentlicher Helper-Export: `bypass::compute_bypass_positions` (u.a. fuer Preview-Benchmarks)

### `builder.rs`

**`assemble_tool_result(positions, start, end, direction, priority, road_map) → ToolResult`** — Gemeinsame Logik aller Route-Tools: Nimmt berechnete Positionen, erstellt neue Nodes (ueberspringt existierende) und baut interne/externe Verbindungen auf.

`ToolResult.external_connections` kodiert externe Kanten als
`(new_node_idx, existing_node_id, existing_to_new, direction, priority)`.
Damit bleibt die Richtung (`Regular`/`Dual`/`Reverse`) an Start- und Endrand konsistent,
ohne implizite Richtungs-Spiegelung.
