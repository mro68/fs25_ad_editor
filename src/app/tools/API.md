# Tools API

Dokumentation fuer `app::tools`: `ToolManager`, `RouteTool`-Trait, registrierte Tools und gemeinsame Infrastruktur.

**Zurueck:** [`../API.md`](../API.md)

---

## `ToolManager`

Verwaltet registrierte Route-Tools und den aktiven Tool-Index.

```rust
pub struct ToolManager { /* intern */ }
```

**Methoden:**

- `new() → Self` — Erstellt ToolManager mit vorregistrierten Standard-Tools (StraightLine, Bézier Grad 2, Bézier Grad 3, Spline, Bypass, SmoothCurve, Parking, FieldBoundary, RouteOffset)
- `register(tool)` — Neues Route-Tool registrieren
- `tool_count() → usize` — Anzahl registrierter Tools
- `tool_names() → Vec<(usize, &str)>` — Name + Index aller Tools
- `tool_entries() → Vec<(usize, &str, &str)>` — Index, Name und Icon aller Tools (fuer Dropdown-Rendering)
- `set_active(index)` — Aktives Tool setzen (Reset des vorherigen)
- `active_index() → Option<usize>` — Index des aktiven Tools
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

**`ToolPreview` Felder**

- `nodes: Vec<Vec2>` — Vorschau-Node-Positionen
- `connections: Vec<(usize, usize)>` — Index-Paare in `nodes`
- `connection_styles: Vec<(ConnectionDirection, ConnectionPriority)>` — Stil pro Verbindung (gleiche Laenge wie `connections`)

### Capability-Traits

Der optionale Teil wurde zusaetzlich in Capability-Traits gekapselt. `RouteTool` bleibt als kompatibler Obervertrag bestehen und delegiert seine Default-Implementierungen an diese Traits:

- `RouteToolDrag` — `drag_targets`, `on_drag_start`, `on_drag_update`, `on_drag_end`
- `RouteToolTangent` — `tangent_menu_data`, `apply_tangent_selection`
- `RouteToolRegistry` — `make_group_record`, `load_for_edit`
- `RouteToolChainInput` — `needs_chain_input`, `load_chain`

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

| Idx | Tool | Icon | Konstruktor |
|-----|------|------|-------------|
| 0 | `StraightLineTool` | `━` | `StraightLineTool::new()` |
| 1 | `CurveTool` (Grad 2) | `⌒` | `CurveTool::new()` |
| 2 | `CurveTool` (Grad 3) | `〜` | `CurveTool::new_cubic()` |
| 3 | `SplineTool` | `〰` | `SplineTool::new()` |
| 4 | `BypassTool` | `⤴` | `BypassTool::new()` |
| 5 | `SmoothCurveTool` | `⊿` | `SmoothCurveTool::new()` |
| 6 | `ParkingTool` | `🅿` | `ParkingTool::new()` |
| 7 | `FieldBoundaryTool` | `🌾` | `FieldBoundaryTool::new()` |
| 8 | `RouteOffsetTool` | `⇶` | `RouteOffsetTool::new()` |

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

- Implementiert `RouteToolRegistry` Trait (`make_group_record()`, `load_for_edit()`)
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
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
}
```

**Interne Ring-Berechnung:**

1. `offset_polygon()` — Polygon nach innen/aussen verschieben
2. `simplify_polygon()` — Douglas-Peucker vereinfachen
3. `resample_by_distance()` — Gleichmaessiges Resampling mit `node_spacing`
4. Geschlossener Ring: letzte Verbindung (N−1 → 0) schliesst den Ring

**Gruppen-Record:** `GroupKind::FieldBoundary { field_id, node_spacing, offset, straighten_tolerance, base }` (Slot 7 im ToolManager)

Modulstruktur: `mod.rs` (Re-Exporte), `state.rs` (Struct, Phasen-Enum, Default), `lifecycle.rs` (RouteTool-Impl, Ring-Berechnung), `config_ui.rs` (egui-Panel)

### `RouteOffsetTool`

Parallelversatz einer selektierten Kette ohne S-Kurven-Anbindung (Slot 8 im ToolManager). Generiert eine oder zwei Parallel-Versatz-Ketten (links und/oder rechts) mit konfigurierbarem Abstand und optionalem Entfernen der Original-Kette.

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
    pub(crate) cached_preview: Option<(Vec<Vec2>, Vec<(usize, usize)>)>,
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

**Gruppen-Record:** `GroupKind::RouteOffset { chain_positions, chain_start_id, chain_end_id, offset_left, offset_right, keep_original, base_spacing, base }` (Slot 8 im ToolManager)

Modulstruktur: `mod.rs` (Re-Exporte), `state.rs` (Struct + OffsetConfig), `lifecycle.rs` (RouteTool-Impl), `geometry.rs` (compute_offset_positions), `config_ui.rs` (egui-Panel), `tests.rs`

---

### `BypassTool`

Parallele Ausweichstrecke einer selektierten Kette mit S-förmigen An-/Abfahrten. Das Tool benötigt eine Eingabe-Kette (via `load_chain()`), generiert dann automatisch die Bypass-Positionen und erstellt neue Nodes mit entsprechenden Verbindungen.

**Input-Modus:** Chain-basiert (nutzt `RouteToolChainInput` Trait).

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
