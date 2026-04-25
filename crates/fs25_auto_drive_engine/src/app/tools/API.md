# Tools API

Dokumentation fuer `app::tools`: `ToolManager`, `RouteTool`-Trait, registrierte Tools und gemeinsame Infrastruktur.

`RouteToolId`, `ToolAnchor` und `TangentSource` gehoeren zum app-weiten Tool-Vertrag in `app::tool_contract`. `RouteToolId` wird ausserhalb der Tool-Implementierungen explizit von dort importiert; `app::tools` bleibt fuer Katalog, Manager, Traits und gemeinsame Tool-Helfer zustaendig. UI-taugliche Read-DTOs wie `TangentMenuData` und `TangentOptionData` gehoeren nach `app::ui_contract`.

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
- `tool_entries() → Vec<(RouteToolId, &str, &str)>` — ID, Name und Legacy-Icon aller Tools (fuer Dropdown-Rendering)
- `set_active_by_id(tool_id)` — Aktives Tool setzen (Reset des vorherigen)
- `active_id() → Option<RouteToolId>` — ID des aktiven Tools
- `active_descriptor() → Option<&'static RouteToolDescriptor>` — Katalog-Descriptor des aktiven Tools
- `active_tool() → Option<&dyn RouteTool>` — Referenz auf aktives Tool
- `active_tool_mut() → Option<&mut dyn RouteTool>` — Mutable Referenz
- `sync_active_host(context)` — Synchronisiert Editor-Defaults und externe Assets in das aktive Tool
- `active_recreate[_mut]()` / `active_drag[_mut]()` / `active_tangent[_mut]()` — Capability-Discovery fuer Recreate-, Drag- und Tangenten-Flows
- `active_rotate[_mut]()` / `active_segment_adjustments[_mut]()` / `active_chain_input[_mut]()` / `active_lasso_input[_mut]()` — Zugriff auf Rotations-, Segment-Shortcut-, Ketten- und Lasso-Capabilities
- `load_active_chain(chain)` — Laedt eine geordnete Kette ueber die `RouteToolChainInput`-Capability
- `reset()` — Alle Tools zuruecksetzen, aktives deaktivieren

**Katalog-Metadaten für UI-Surfaces:**

- `RouteToolDescriptor.icon_key: RouteToolIconKey` — kanonischer Icon-Schluessel fuer Menue, Palette und Floating-UI
- `RouteToolIconKey` entkoppelt UI-Icon-Aufloesung von direkten `RouteToolId`-Matches in einzelnen UI-Komponenten

---

## Route-Tool-Vertraege

Phase 3 teilt den frueher breiten Allzweck-Vertrag in drei feste Basisvertraege und additive Capabilities auf.

**Feste Basisvertraege:**

- `RouteToolCore` — Kerninteraktion und Preview/Execute-Lifecycle: `on_click`, `preview`, `execute`, `reset`, `is_ready`, `has_pending_input`
- `RouteToolPanelBridge` — egui-freie Panel-Anbindung: `status_text`, `panel_state`, `apply_panel_action`
- `RouteToolHostSync` — Uebernahme des Editor-/Asset-Kontexts via `sync_host(context: &ToolHostContext)`

**Umbrella-Vertrag:**

- `RouteTool: RouteToolCore + RouteToolPanelBridge + RouteToolHostSync`
- enthaelt nur noch Capability-Discovery (`as_recreate()`, `as_drag()`, `as_tangent()`, `as_rotate()`, `as_segment_adjustments()`, `as_chain_input()`, `as_lasso_input()`, `as_group_edit()`)

**Additive Capabilities:**

- `RouteToolRecreate` — `on_applied`, `last_created_ids`, `last_end_anchor`, `needs_recreate`, `clear_recreate_flag`, `execute_from_anchors`
- `RouteToolDrag` — `drag_targets`, `on_drag_start`, `on_drag_update`, `on_drag_end`
- `RouteToolTangent` — `tangent_menu_data`, `apply_tangent_selection`
- `RouteToolRotate` — `on_scroll_rotate`
- `RouteToolSegmentAdjustments` — `increase_node_count`, `decrease_node_count`, `increase_segment_length`, `decrease_segment_length`
- `RouteToolChainInput` — `load_chain(OrderedNodeChain)`
- `RouteToolLassoInput` — `is_lasso_input_active`, `on_lasso_completed`
- `RouteToolGroupEdit` — `build_edit_payload`, `restore_edit_payload`

**Host-Kontext:**

- `ToolHostContext` buendelt `direction`, `priority`, `snap_radius`, `farmland_data`, `farmland_grid` und `background_image`
- Handler und `ToolManager` synchronisieren diesen Kontext zentral, statt viele einzelne Setter auf dem Tool-Vertrag zu fuehren

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

### Gemeinsame Lifecycle-Helfer

`app/tools/common/lifecycle.rs` enthaelt fuer den neuen Vertrag zwei zentrale Shared-Helfer:

- `sync_tool_host(direction, priority, lifecycle, context)` — uebernimmt Editor-Defaults und Snap-Radius in ein Tool
- `record_applied_tool_state(lifecycle, ids, end_anchor)` — speichert letzte Node-IDs und den End-Anker nach erfolgreicher Anwendung

### Direkte Erweiterungspunkte

Optionales Verhalten wird nicht mehr ueber No-Op-Methoden auf `RouteTool` modelliert, sondern ueber Capability-Discovery im `ToolManager`.

- Viewport-Drag und Tangenten gehen ueber `active_drag()` bzw. `active_tangent()`
- Alt+Scroll-Rotation geht ueber `active_rotate_mut()`, Segment-Shortcuts nur bei vorhandener Capability ueber `active_segment_adjustments_mut()`
- Recreate- und Chaining-Flows gehen ueber `active_recreate()` bzw. `active_chain_input()`
- Alt+Lasso fuer Analyse-Tools wird ueber `active_lasso_input()` und `is_lasso_input_active()` geroutet
- Persistierbare Tools exponieren ihren Edit-Snapshot ueber `active_group_edit()` bzw. `active_group_edit_mut()`

**Tool-Edit-Erweiterung** (fuer den separaten Tool-Editing-Layer, siehe [`../API.md#tooleditstore-routetooleditpayload-und-activetooleditsession`](../API.md#tooleditstore-routetooleditpayload-und-activetooleditsession)):

```rust
fn build_edit_payload(&self) -> Option<RouteToolEditPayload>;
fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload);
```

---

## `RouteToolEditPayload` (Tool-Editing)

Die kanonische Beschreibung des separaten Persistenz-/Edit-Vertrags liegt in [`../API.md#tooleditstore-routetooleditpayload-und-activetooleditsession`](../API.md#tooleditstore-routetooleditpayload-und-activetooleditsession). Kurz: group-backed editierbare Tools bauen ihren Snapshot ausserhalb der Registry als `RouteToolEditPayload`; das neutrale `GroupRecord` speichert nur Gruppenmitgliedschaft, Marker-Cleanup und Boundary-Metadaten.

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

Modulstruktur: `state.rs` (Enums, Struct, Ctors), `lifecycle.rs` (RouteTool-Impl), `drag.rs` (Drag-Logik), `config_ui.rs` (semantische Panel-Bruecke), `geometry.rs` (Bézier-Mathe), `tests.rs`

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

**Tool-Edit:**

- Implementiert `RouteToolGroupEdit`
- Liefert/stellt den Layout-Snapshot ueber `build_edit_payload()` und `restore_edit_payload()` wieder her

**Geometrie-/Konvertierungspfad:**

- `generate_parking_layout(config) → ParkingLayout` — Oeffentlicher Layout-Generator fuer Tests und Benchmarks
- `build_parking_result(layout) → ToolResult` — Modulintern: konvertiert das Layout via `ToolResultBuilder`; befuellt `markers` und laesst `external_connections` sowie `nodes_to_remove` kanonisch leer
- `build_preview(layout) → ToolPreview` — Modulintern: Vorschau-Geometrie inkl. Verbindungsstilen und Labels

Modulstruktur: `state.rs` (Struct + Config), `lifecycle.rs` (RouteTool-Impl + Lifecycle-Delegation), `config_ui.rs` (semantische Panel-Bruecke), `geometry/{mod,layout,blueprint,conversion}.rs` (Layout-Mathe), `tests.rs` (7 Unit-Tests)

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

`RingNodeKind` bleibt bewusst im Geometrie-Modul verankert; `app::use_cases` konsumieren die Ring-Ausgabe nur indirekt ueber die schmale App-Bruecke um `compute_ring`.

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

**ToolResult-Aufbau:** `execute()` nutzt `ToolResultBuilder`; der geschlossene Ring befuellt nur `new_nodes` und `internal_connections`, waehrend `external_connections`, `markers` und `nodes_to_remove` kanonisch leer bleiben.

**Geometrie-Funktionen (`tools/field_boundary/geometry.rs`):**

- `detect_corners(vertices, angle_threshold_rad) -> Vec<usize>` — Sortierte Indizes aller Eckpunkte mit Ablenkungswinkel ≥ Schwellwert
- `round_corner(prev, corner, next, radius, spacing) -> Vec<Vec2>` — Kreisbogen zwischen Tangentenpunkten einer konvexen Ecke. Konkave Ecken (Cross-Product ≤ 0) werden unverändert zurückgegeben. Tangentenpunkte begrenzt auf 40% der Kantenlaenge.
- `resample_ring_with_corners(simplified, corner_indices, spacing, rounding_radius) -> Vec<(Vec2, RingNodeKind)>` — Resampled den Ring segmentweise mit Ecken als festen Ankern

**Edit-Payload:** `RouteToolEditPayload::FieldBoundary { field_id, node_spacing, offset, straighten_tolerance, corner_angle_threshold, corner_rounding_radius, corner_rounding_max_angle_deg, base }` fuer `RouteToolId::FieldBoundary`

Modulstruktur: `mod.rs` (Re-Exporte), `state.rs` (Struct, Phasen-Enum, Default), `lifecycle.rs` (RouteTool-Impl, Ring-Berechnung), `config_ui.rs` (semantische Panel-Bruecke), `geometry.rs` (RingNodeKind, detect_corners, round_corner, resample_ring_with_corners)

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

Die finale Ausgabe wird ueber `ToolResultBuilder` aufgebaut: interne Offset-Ketten gehen in `new_nodes`/`internal_connections`, die lateralen Anschluesse in `with_external_connections(...)` und das optionale Entfernen der Original-Kette in `with_nodes_to_remove(...)`.

**Geometrie-Funktionen (`geometry.rs`):**

- `compute_offset_positions(chain, offset, base_spacing) → Option<Vec<Vec2>>` — Nutzt `parallel_offset()` + `resample_by_distance()`

**Public Exports (`mod.rs`):**

- `RouteOffsetTool`
- `compute_offset_positions()` — Hotpath-Helfer fuer Benchmarks/Tests

**Edit-Payload:** `RouteToolEditPayload::RouteOffset { chain_positions, chain_start_id, chain_end_id, offset_left, offset_right, keep_original, base_spacing, base }` fuer `RouteToolId::RouteOffset`

Modulstruktur: `mod.rs` (Re-Exporte), `state.rs` (Struct + OffsetConfig), `lifecycle.rs` (RouteTool-Impl), `geometry.rs` (compute_offset_positions), `config_ui.rs` (semantische Panel-Bruecke), `tests.rs`

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

Modulstruktur: `mod.rs` (Re-Export), `state.rs` (Structs, Enums, Felder), `lifecycle.rs` (RouteTool-Impl, compute_centerline), `config_ui.rs` (semantische Panel-Bruecke)

---

### `ColorPathTool`

Farb-Pfad-Erkennung: Erkennt zusammenhaengende Teilnetze anhand der Farbe im Hintergrundbild, skelettiert sie per Zhang-Suen-Thinning und exportiert daraus ein Waypoint-Netz mit offenen Enden, Kreuzungen und Segmenten (`RouteToolId::ColorPath`).

**Voraussetzung:** Ein Hintergrundbild muss im `ToolHostContext.background_image` vorhanden sein. Das Tool kann `map_size` zusaetzlich aus `ToolHostContext.farmland_grid` ableiten.

**Phasen (`ColorPathPhase` — Wizard-Fluss):**

Seit dem ColorPath-Wizard (CP-03..CP-08) wird der Pipeline-Fluss explizit in vier
schreibende Phasen plus `Idle` aufgebrochen. Die alte Sammel-Phase `Preview`
entfaellt intern vollstaendig; sie existiert nur noch als Legacy-Alias im DTO
(`ColorPathPanelPhase::Preview`, `#[deprecated]`) fuer Host-Migrationen bis
CP-11.

- **`Idle`** — Leerer Grundzustand ohne aktives Sampling, z.B. nach einem expliziten Reset
- **`Sampling`** — User sammelt Farbproben per Klick oder Alt+Lasso; ein frisches Tool startet bereits in dieser Phase
- **`CenterlinePreview`** — Stages C-E sind durchgelaufen (Maske + Netzextraktion); Stage F ist bewusst zurueckgehalten, damit der User die Mittellinien vor jeder Begradigung begutachten kann
- **`JunctionEdit`** — `EditableCenterlines` ist aus dem Stage-E-Netz befuellt; Junctions/OpenEnds sind ueber `RouteToolDrag` beweglich, Stage F bleibt weiterhin zurueckgehalten
- **`Finalize`** — Stage F laeuft auf den (ggf. bewegten) Editable-Junctions; `PreparedSegment`s liegen vor und sind uebernahmefaehig

**Wizard-Transitions:** `ColorPathPanelAction::NextPhase` / `PrevPhase` fuehren
den Nutzer schrittweise durch den Flow, `Accept` uebernimmt das Netz aus
`Finalize`. Die Legacy-Actions `ComputePreview` und `BackToSampling` mappen auf
`NextPhase` / `PrevPhase`, bleiben aber `#[deprecated]` (CP-04/CP-05).

**Editierbares Zwischenartefakt (`EditableCenterlines`, engine-intern):**

- Eingefuehrt in CP-06 als ID-stabile Sicht zwischen Stage E und Stage F; nicht
  Teil der oeffentlichen API des Crates (`pub(super)` innerhalb `color_path`).
- Haelt `EditableJunction`s (mit `origin_world`, `world_pos`, `kind`) und
  `EditableCenterline`s (mit stabiler `CenterlineId`, `start/end_junction`,
  Polyline, `LaneSpec`-Platzhalter fuer spaetere zweispurige Strassen).
- `revision: u64` bumpt bei jeder Mutation (Junction-Drag); `source_core_revision`
  bindet das Artefakt an die erzeugende Stage-E-Revision.
- Stage F (`PreparedSegmentsCacheKey`) nimmt `editable_revision` mit auf, damit
  Drags Stage F sauber invalidieren.

**`RouteToolDrag`-Capability (nur in `JunctionEdit` aktiv):**

- `drag_targets()` liefert ausserhalb von `JunctionEdit` stets `Vec::new()`; in
  `JunctionEdit` die sortierten Junction-Weltpositionen aus `EditableCenterlines`.
- `on_drag_update(id, pos)` mutiert nur `EditableJunction.world_pos` und bumpt
  `editable.revision`; Stage F wird **nicht** pro Drag-Frame neu gerechnet. Erst
  der Uebergang nach `Finalize` triggert den Rebuild.

**ToolLasso-Mechanismus:**

Das Tool exponiert in Phase `Sampling` die Capability `RouteToolLassoInput` und liefert dann
`is_lasso_input_active() = true`. Damit wird jeder Alt+Drag im Viewport als
`DragSelectionMode::ToolLasso` geroutet (statt als normale Node-Selektion). Das abgeschlossene
Polygon wird per `AppIntent::RouteToolLassoCompleted` an `handlers::route_tool::lasso_completed()`
weitergeleitet, das die Lasso-Capability des aktiven Tools aufruft.

**Interaktionsflow (Wizard):**

1. Tool aktivieren → `Sampling` mit sofort aktivem Tool-Lasso
2. Klick oder Alt+Drag sampelt Farben; der erste Sampling-Punkt setzt zugleich den Flood-Fill-Seed
3. Mehrere Klicks und Lasso-Polygone sind moeglich (Sampling kumulativ)
4. Sidebar `Weiter` (`NextPhase`) → Stages C-E laufen → `CenterlinePreview`
5. `Weiter` → `JunctionEdit`: `EditableCenterlines` wird befuellt, Junctions sind draggable
6. `Weiter` → `Finalize`: Stage F fuehrt Junction-Trim + Resampling auf den (ggf. bewegten) Junctions aus
7. `Zurueck` (`PrevPhase`) in jeder Phase fuehrt genau einen Schritt zurueck; `Reset` leert alles nach `Idle`
8. `Uebernehmen` (`Accept`) → nur in `Finalize` und nur wenn `PreparedSegment`s vorliegen → Graph in Road Map einfuegen

**Interne Stage-Pipeline (`compute_pipeline()`):**

1. **Stage A: Sampling-Input** — `SamplingInput` sammelt Lasso-Regionen, Rohfarben, Durchschnittsfarbe und den ersten Lasso-Startpunkt
2. **Stage B: Matching-Spezifikation** — `MatchingSpec` leitet aus Exaktmodus/Toleranz die wirksame Palette + Toleranz ab
3. **Stage C: Pixel-Maske + Sampling-Vorschau** — `flood_fill_color_mask()` erzeugt die Flood-Fill-Maske; `SamplingPreviewData` haelt dieselbe Maske plus Boundary-Segmente fuer die Sampling-Preview
4. **Stage D: Maskenaufbereitung** — `prepare_mask_for_skeleton()` wendet optional Opening + Closing an und erzeugt die vorbereitete Arbeitsmaske
5. **Stage E: Skeleton-/Netzextraktion** — `extract_network_from_mask()` fuehrt Zhang-Suen, Komponentenbildung, Junction-Clustering, Segment-Trace und Medial-Axis-Korrektur auf der vorbereiteten Maske aus
6. **Stage F: Preview-Aufbereitung** — `simplify_polyline()` vereinfacht die Segment-Polylinien; bei gesetztem `junction_radius` trimmt `trim_segment_near_junctions()` danach Junction-nahe Innenpunkte, bevor `resample_by_distance()` die finale `PreparedSegment`-Kette erneut nach `node_spacing` verteilt; `PreviewData` haelt Netz + PreparedSegments als gemeinsame Wahrheit fuer Preview und Execute
7. **Stage G: Execute-Konvertierung** — `execute_result()` baut aus `PreviewData.network` und denselben `PreparedSegment`s das `ToolResult` inklusive optionaler Bestandsanschluesse

**Preview/Export:**

- Preview zeigt Kennzahlen fuer Kreuzungen, offene Enden, Segmente und Preview-Nodes
- Sampling-Preview zeigt nach jedem Klick oder jeder Lasso-Auswahl alle Randsegmente der Flood-Fill-Maske, nicht nur eine Einzelkontur
- Sampling-Preview und Berechnen-Pipeline lesen dieselbe Stage-C-Maskenwahrheit; es gibt keine separaten Preview-Maskenpfade
- Preview und Export teilen dieselben `PreparedSegment`-Artefakte; Execute legt Junction-/End-Knoten genau einmal an und fuegt pro Segment nur die Zwischenpunkte neu ein
- Bestandsanschluss nutzt `ToolLifecycleState::snap_at()` und damit den konfigurierten Snap-Radius

**Konfiguration (`ColorPathConfig`):**

- `exact_color_match: bool` — Exaktmodus; matcht nur auf exakt gelasso-te RGB-Farben und deaktiviert die Toleranz-UI (Standard: `true`)
- `color_tolerance: f32` — Farb-Toleranz im unscharfen Modus (euklidischer RGB-Abstand; Standard: 25.0, Bereich: 1–80)
- `node_spacing: f32` — Abstand zwischen generierten Nodes in Metern (Standard: 5.0, Bereich: 1–50)
- `simplify_tolerance: f32` — Douglas-Peucker-Toleranz in Metern (Standard: 1.0, Bereich: 0–20)
- `junction_radius: f32` — Radius in Metern fuer die Kreuzungsbegradigung an Junction-Segmenten; die finale Punktverteilung bleibt trotz Trim `node_spacing`-getrieben (Standard: 0.0, Bereich: 0–100)
- `noise_filter: bool` — Morphologischen Rauschfilter aktivieren (Standard: true)
- `existing_connection_mode: ExistingConnectionMode` — Bestandsanschluss: `Never`, `OpenEnds`, `OpenEndsAndJunctions` (Standard: `OpenEnds`)
- `detection_bounds: Option<(Vec2, Vec2)>` — Begrenzt Farberkennung auf eine Rect-Region (geplant)

**Felder:**

```rust
pub struct ColorPathTool {
    pub(crate) phase: ColorPathPhase,
    pub(crate) config: ColorPathConfig,
    pub(super) sampling: SamplingInput,
    pub(super) matching: MatchingSpec,
    pub(super) sampling_preview: Option<SamplingPreviewData>,
    pub(super) preview_data: Option<PreviewData>,
    pub(crate) background_image: Option<Arc<image::DynamicImage>>,
    pub(crate) map_size: f32,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
    pub(super) cache: ColorPathCacheState,
}
```

**Interne Stage-Artefakte:**

- `SamplingInput` — `lasso_regions`, `sampled_colors`, `avg_color`, `lasso_start_world`
- `MatchingSpec` — wirksame `palette` + `tolerance`
- `SamplingPreviewData` — `input_mask`, `boundary_segments`, `start_pixel`
- `PreviewData` — `prepared_mask`, `network`, `prepared_segments`

**Sampling-Funktionen (`sampling.rs`):**

- `world_to_pixel(world, map_size, img_w, img_h) → (u32, u32)` — Weltkoords → Bildpixel
- `pixel_to_world(px, py, map_size, img_w, img_h) → Vec2` — Bildpixel → Weltkoords (X und Y je mit korrektem Skalierungsfaktor)
- `pixel_to_world_f32(px, py, map_size, img_w, img_h) → Vec2` — Sub-Pixel-Position → Weltkoords (fuer Medial-Axis)
- `sample_colors_in_polygon(polygon, image, map_size) → Vec<[u8; 3]>` — RGB-Pixel im Lasso-Polygon sammeln
- `compute_average_color(colors) → [u8; 3]` — RGB-Mittelwert aller Samples
- `build_exact_color_set(raw_colors) → Vec<[u8; 3]>` — Eindeutige Rohfarben fuer Exaktmodus ohne Quantisierung
- `build_color_mask(image, palette, tolerance, bounds, map_size) → (Vec<bool>, u32, u32)` — Bool-Maske ueber einen rechteckig begrenzten Bereich erstellen
- `flood_fill_color_mask(image, palette, tolerance, start_pixel) → (Vec<bool>, u32, u32)` — zusammenhaengende Stage-C-Maske ab Startpixel berechnen
- `extract_boundary_segments_from_mask(mask, w, h, map_size) → Vec<(Vec2, Vec2)>` — Alle Grenzen der Flood-Fill-Maske als Preview-Segmente (inkl. Innenkanten/Loecher)
- `prepare_mask_for_skeleton(mask, w, h, noise_filter) → Vec<bool>` — Stage-D-Aufbereitung der Flood-Fill-Maske fuer die Skeleton-Extraktion
- `erode(mask, w, h) → Vec<bool>` — Erosion mit Majority-Bedingung (≥ 3 von 4 Nachbarn, zum Schutz duenner Verbindungen)
- `dilate(mask, w, h) → Vec<bool>` — Dilatation (4-Connectivity)
- `morphological_open(mask, w, h) → Vec<bool>` — Erosion + Dilatation (Rauschen entfernen)
- `morphological_close(mask, w, h) → Vec<bool>` — Dilatation + Erosion (Luecken schliessen)

**Skelett-Funktionen (`skeleton.rs`):**

- `find_connected_components(mask, w, h) → Vec<Vec<(usize, usize)>>` — Gruppen nach Groesse absteigend
- `refine_medial_axis(ordered, original_mask, w, h) → Vec<(f32, f32)>` — Skelett-Pixel auf geometrische Mittelachse korrigieren
- `extract_network_from_mask(mask, w, h, map_size, start_hint) → SkeletonNetwork` — Netzextraktion auf bereits vorbereiteter Stage-D-Maske
- `extract_paths_from_mask(...) → Vec<Vec<Vec2>>` — Legacy-Wrapper fuer lineare Pfad-Konsumenten/Tests

**Public Exports (`mod.rs`):**

- `ColorPathTool`
- `compute_color_path_network_stats()` — Flood-Fill + Netzextraktion fuer Benchmarks/Analyse, ohne interne Skelett-Typen offenzulegen
- `ColorPathBenchmarkHarness` — baut ueber den echten Tool-Flow reproduzierbare Sampling-/Preview-Ausgangszustaende fuer Criterion-Benchmarks auf
- `ColorPathBenchmarkAction` — vorbereitete Einzelaktion fuer `SamplingPreview`, `compute_pipeline()`, Preview-Core- oder PreparedSegments-Rebuild
- `ColorPathBenchmarkStats` — beobachtbare Kennzahlen und Revisionszaehler einer einzelnen Benchmark-Aktion

**Gruppen-Record:** ColorPathTool speichert keinen `GroupRecord` (keine nachträgliche Bearbeitung).

Modulstruktur: `mod.rs` (Re-Export + Benchmark-Fassade), `state.rs` (Stage-Artefakte, Phasen-Enum, Config, Default), `lifecycle.rs` (Phasenwechsel + RouteTool-Adapter inkl. `RouteToolDrag`-Impl), `pipeline/` (Stage-Pipeline in Submodulen: `mod.rs` — Fassade, `matching.rs` — Stage B, `sampling_stage.rs` — Stage C, `preview_core.rs` — Stages D/E, `prepared.rs` — Stage F), `preview.rs` (Preview-/Execute-Aufbereitung mit `PreparedSegment` als gemeinsamer Wahrheit), `config_ui.rs` (semantische Panel-Bruecke + Wizard-Transitions), `editable.rs` (engine-internes `EditableCenterlines`-Zwischenmodell, CP-06), `drag.rs` (Drag-Verhalten fuer Junction-Editing, CP-08), `sampling.rs` (Farb-Sampling + Masken-Erstellung), `skeleton.rs` (Skelett-Extraktion + Graph-Aufbau)

---

### `BypassTool`

Parallele Ausweichstrecke einer selektierten Kette mit S-förmigen An-/Abfahrten. Das Tool benötigt eine Eingabe-Kette (via `load_chain()`), generiert dann automatisch die Bypass-Positionen und erstellt neue Nodes mit entsprechenden Verbindungen.

**Input-Modus:** Chain-basiert ueber die `RouteToolChainInput`-Capability.

- `ToolManager::active_chain_input()` signalisiert dem Handler, dass eine geordnete Kette benoetigt wird
- `load_chain(OrderedNodeChain)` uebernimmt Positionen sowie Start-/End- und innere Node-IDs in einem DTO

**Konfiguration:**

- `offset: f32` — Seitlicher Versatz in Welteinheiten (positiv = links, negativ = rechts)
- `base_spacing: f32` — Abstand zwischen Nodes auf der Hauptstrecke
- `direction: ConnectionDirection` — Richtung fuer die erzeugten Verbindungen
- `priority: ConnectionPriority` — Prioritaet fuer die erzeugten Verbindungen

**Caching:**

- `cached_positions` — Gecachte Bypass-Positionen (wird invalidiert bei Config-Aenderung)
- `cached_connections` — Gecachte Preview-Connections inkl. Start/End-Anker

**Lifecycle-Integration:**

- Enthaelt gemeinsamen `ToolLifecycleState` fuer Snap-Radius, letzte erstellte Node-IDs und Recreate-Flag
- Host-Defaults laufen ueber `sync_tool_host(...)`
- Recreate-Status und letzte IDs laufen ueber die Capability `RouteToolRecreate`

**Public Exports:**

- `compute_bypass_positions(chain, offset, base_spacing) → Option<(Vec<Vec2>, f32)>` — Berechnet Bypass-Positionen und Uebergangslaenge (fuer Benchmarks + Tests)

Modulstruktur: `state.rs` (Struct + Config), `lifecycle.rs` (RouteTool-Impl + Lifecycle-Delegation), `config_ui.rs` (semantische Panel-Bruecke), `geometry.rs` (Bypass-Mathe), `tests.rs` (15 Unit-Tests)

---

## Gemeinsame Tool-Infrastruktur (`tools/common/`)

Aufgeteilt in fuenf Submodule (alle privat, Re-Exporte via `common/mod.rs`):

### `geometry.rs`

Hilfsfunktionen: `angle_to_compass`, `node_count_from_length`, `populate_neighbors`, `snap_with_neighbors`, `linear_connections`, `tangent_options`

**Polyline-Geometrie** (gemeinsam fuer BypassTool + RouteOffsetTool):

- **`parallel_offset(polyline, offset) → Vec<Vec2>`** — Berechnet eine parallel versetzte Polyline. `offset > 0` = links (positive Senkrechte in Fahrtrichtung), `offset < 0` = rechts.
- **`local_perp(i, poly) → Vec2`** — Lokale Senkrechte am Index `i` einer Polyline (Durchschnitt benachbarter Segmente; Randpunkte nutzen nur das angrenzende Segment).

### `tangent.rs`

**`render_tangent_combo(ui, id_salt, label, none_label, current, neighbors) → bool`** — Gemeinsamer UI-Baustein fuer Tangenten-ComboBoxen (verwendet von Curve + Spline config_ui). Arbeitet auf `app::tool_contract::TangentSource`.

Die Menue-DTOs fuer die UI (`TangentMenuData`, `TangentOptionData`) liegen seit F4a bewusst in `app::ui_contract`; `tools/common` liefert dafuer nur noch die Aufbereitung ueber `geometry::tangent_options()`.

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

**`sync_tool_host(...)`** — Shared-Helper fuer den Host-Kontext-Sync (`direction`, `priority`, `snap_radius`) ueber alle Tools hinweg.

**`record_applied_tool_state(...)`** — Shared-Helper fuer `RouteToolRecreate::on_applied()`: uebernimmt letzte IDs und End-Anker in den gemeinsamen Lifecycle-State.

### `bypass::geometry` Export

- Oeffentlicher Helper-Export: `bypass::compute_bypass_positions` (u.a. fuer Preview-Benchmarks)

### `result.rs`

**`ToolResultBuilder`** — Schmaler Shared-Builder fuer `ToolResult`-Faelle mit kanonischen leeren Defaults der optionalen Sammlungen.

- `new(new_nodes, internal_connections) → ToolResultBuilder` — initialisiert `external_connections`, `markers` und `nodes_to_remove` leer
- `with_external_connections(...)` — setzt nur die benoetigten Anbindungen an bestehende Nodes
- `with_markers(...)` — setzt Marker nur fuer Tools mit Marker-Semantik
- `with_nodes_to_remove(...)` — setzt zu entfernende Bestands-Nodes nur fuer Ersetzungsfaelle
- `build() → ToolResult` — gibt das fertige Ergebnis zurueck

Stand F5: `assemble_tool_result()` sowie die spezialisierten Ausgabepfade von `FieldBoundaryTool`, `RouteOffsetTool` und `ParkingTool` nutzen diesen Builder. Dadurch bleiben Default-Felder konsistent, ohne in jedem Tool ein volles `ToolResult` manuell initialisieren zu muessen.

### `builder.rs`

**`assemble_tool_result(positions, start, end, direction, priority, road_map) → ToolResult`** — Gemeinsame Logik aller Route-Tools: Nimmt berechnete Positionen, erstellt neue Nodes (ueberspringt existierende) und baut interne/externe Verbindungen auf.

`ToolResult.external_connections` kodiert externe Kanten als
`(new_node_idx, existing_node_id, existing_to_new, direction, priority)`.
Damit bleibt die Richtung (`Regular`/`Dual`/`Reverse`) an Start- und Endrand konsistent,
ohne implizite Richtungs-Spiegelung.

Die Rueckgabe wird intern ueber `ToolResultBuilder` erzeugt, sodass `markers` und `nodes_to_remove` auch in den einfachen Polyline-Pfaden kanonisch leer initialisiert werden.
