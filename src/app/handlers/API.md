# Handlers API Documentation

## Überblick

Das `handlers`-Modul gruppiert die Verarbeitung von `AppCommand`s in Feature-bereiche. Jeder Handler ist eine Sammlung von Funktionen, die einen spezifischen Aspekt der App mutieren — Datei-I/O, Selektion, Editing, etc.

**Architektur:**
1. Der `AppController` (in `controller.rs`) dispatcht jeden `AppCommand` an den passenden Handler
2. Handler rufen Funktionen aus `use_cases/` auf — diese enthalten die Geschäftslogik
3. Handler selbst sind dünn und koordinieren hauptsächlich Undo-Snapshots und State-Updates

**Re-Exports:**  
Die Handler-Module werden in [`src/app/mod.rs`](../mod.rs) re-exportiert:
```rust
pub use handlers::{dialog, editing, file_io, helpers, history, route_tool, selection, view};
```

---

## Handler-Module

### `dialog` — Dialog-State und Anwendungssteuerung

Verwaltet die Sichtbarkeit und Logik von Dialogen sowie optionale Benutzer-Interaktionen.

**Funktionen:**

```rust
pub fn request_exit(state: &mut AppState)
```
Markiert die Anwendung zum Beenden im nächsten Frame. Setzt `.should_exit = true`.

```rust
pub fn request_heightmap_dialog(state: &mut AppState)
pub fn request_background_map_dialog(state: &mut AppState)
pub fn request_overview_dialog(state: &mut AppState)
```
Öffnet die Datei-Dialoge für Heightmap, Background-Map bzw. Übersichtskarten-ZIP.

```rust
pub fn dismiss_heightmap_warning(state: &mut AppState)
pub fn close_marker_dialog(state: &mut AppState)
pub fn close_zip_browser(state: &mut AppState)
pub fn dismiss_dedup_dialog(state: &mut AppState)
pub fn dismiss_post_load_dialog(state: &mut AppState)
pub fn close_overview_options_dialog(state: &mut AppState)
pub fn dismiss_save_overview_dialog(state: &mut AppState)
```
Schliesst verschiedene Dialog-Boxen und räumt deren State auf.

---

### `route_tool` — Route-Tool-Operationen (Linie, Kurve, Spline)

Handelt Viewport-Interaktionen und Ausfuehrung von Route-Tools.

**Funktionen:**

```rust
pub fn click(state: &mut AppState, world_pos: glam::Vec2, ctrl: bool)
```
Verarbeitet einen Viewport-Klick im aktiven Route-Tool. Wenn das Tool `ToolAction::ReadyToExecute` zurueckgibt, wird sofort `execute_and_apply()` aufgerufen.

```rust
pub fn rotate(state: &mut AppState, delta: f32)
```
Uebertraegt Scroll-basierte Rotation auf das aktive Route-Tool via `on_scroll_rotate()` callback. Wird typischerweise nur von ParkingTool verwendet (Alt+Scroll).

```rust
pub fn execute(state: &mut AppState)
```
Fuehrt das aktive Route-Tool aus (Enter-Bestaetigung). Erstellt Nodes + Connections, speichert Undo-Snapshot und registriert Segment-Record fuer nachtraegliche Bearbeitung.

**Segment-Registry-Integration:**
- Nach Tool-Ausfuehrung werden die `original_positions` aus der RoadMap gesammelt
- Segment-Record wird mit allen Tool-Parametern registriert
- Ermoeglicht spaeteres Editieren: `EditSegment { record_id }` laedt das Tool mit gespeicherten Parametern neu

```rust
pub fn cancel(state: &mut AppState)
```
Bricht das aktive Route-Tool ab (Escape).

```rust
pub fn select(state: &mut AppState, index: usize)
```
Aktiviert ein Route-Tool per Index. Initialisiert Tool-Parameter (Richtung, Prioritaet, Snap-Radius) aus EditorToolState und laedt optional eine vorhandene Selektion als Kette (fuer Chain-basierte Tools wie BypassTool).

```rust
pub fn select_with_anchors(state: &mut AppState, tool_index: usize, node_id_1: u64, node_id_2: u64)
```
Aktiviert Tool und setzt Start/End-Anker aus zwei selektierten Nodes. Simuliert zwei `on_click()`-Aufrufe; bei StraightLine => sofortige Ausfuehrung, bei Curves => Phase::Control fuer Steuerpunkt-Platzierung.

```rust
pub fn open_options_dialog(state: &mut AppState)
pub fn close_options_dialog(state: &mut AppState)
pub fn apply_options(state: &mut AppState, options: EditorOptions) -> anyhow::Result<()>
pub fn reset_options(state: &mut AppState) -> anyhow::Result<()>
```
Verwaltet den Options-Dialog. `apply_options()` validiert die neuen Optionen, persistiert sie in die Konfigurationsdatei und aktualisiert den State.

```rust
pub fn open_overview_options_dialog(state: &mut AppState, zip_path: String)
```
Öffnet den Optionen-Dialog für die Übersichtskarten-Generierung mit einem bestimmten ZIP-Pfad.

---

### `file_io` — Datei-Operationen

Zentralisiert alle Datei-I/O-Operationen: Laden, Speichern, Heightmap, Background-Map, Overview-Generierung.

**Funktionen:**

```rust
pub fn request_open(state: &mut AppState)
pub fn request_save(state: &mut AppState)
```
Öffnen/Speichern-Dialoge anzeigen. Delegiert an `use_cases::file_io`.

```rust
pub fn load(state: &mut AppState, path: String) -> anyhow::Result<()>
```
Lädt eine AutoDrive-XML-Datei. Parst die XML, erstellt die `RoadMap`, setzt den Dateipfad und führt automatische Post-Load-Erkennung durch (Heightmap, overview.jpg, Map-Mod-ZIP).

```rust
pub fn save(state: &mut AppState, path: Option<String>) -> anyhow::Result<()>
```
Speichert die aktuelle Road Map unter dem angegebenen Pfad (mit Heightmap-Check). `None` speichert unter dem aktuellen Pfad oder öffnet den Dialog.

```rust
pub fn confirm_and_save(state: &mut AppState) -> anyhow::Result<()>
```
Bestätigt die Heightmap-Warnung und speichert sofort.

```rust
pub fn set_heightmap(state: &mut AppState, path: String)
pub fn clear_heightmap(state: &mut AppState)
```
Lädt oder entfernt eine Heightmap.

```rust
pub fn deduplicate(state: &mut AppState)
```
Führt die Duplikat-Bereinigung auf der geladenen Road Map aus.

---

### `editing` — Node- und Connection-Bearbeitung

Handhabt Bearbeitung von Nodes, Verbindungen und Marker. Integriert Segment-Cleanup bei Edits.

**Funktionen:**

```rust
pub fn edit_segment(
    state: &mut AppState,
    record_id: u64,
    kind: SegmentKind,
    node_ids_to_delete: &[u64],
) -> anyhow::Result<()>
```
Bearbeitet ein zuvor erstelltes Segment. Fuehrt folgende Schritte aus:
1. **Marker-Cleanup:** Entfernt `MapMarker` von den zu loeschenden Nodes (aus `record.marker_node_ids`)
2. **Node-Loeschung:** Loescht die alten Segment-Nodes aus der RoadMap
3. **Tool-Reload:** Laedt das passende Route-Tool mit den gespeicherten Parametern (via `load_for_edit()`)
4. **Neu-Ausfuehrung:** Tool wird neu mit den User-Aenderungen ausfuehrt (neue Node-IDs generiert)

**Segment-Registry-Integration:**
- Nutzt `SegmentRecord` (fuer Marker-Cleanup und Tool-Parameter)
- Lokalisiert via `segment_registry.get_record(record_id)`
- Marker-Cleanup ist **kritisch:** Unvollstaendiges Cleanup verwaist BrainCells im SegmentRecord

```rust
pub fn delete_nodes_by_ids(state: &mut AppState, node_ids: &[u64])
```
Loescht Nodes aus der Road Map. Aktualisiert alle Verbindungen automatisch.

```rust
pub fn add_node(state: &mut AppState, pos: glam::Vec2, after_node: Option<u64>) -> u64
```
Fuegt einen neuen Node hinzu. Optional splittet neuer Node eine Verbindung `after_node → next`.

```rust
pub fn set_node_position(state: &mut AppState, node_id: u64, new_pos: glam::Vec2)
```
Verschiebt einen Node (mit Spatial-Index-Update).

```rust
pub fn create_connection(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
)
```
Erzeugt eine neue Verbindung zwischen zwei Nodes.

```rust
pub fn delete_connection(state: &mut AppState, start_id: u64, end_id: u64)
```
Loescht eine Verbindung.

```rust
pub fn set_node_marker(state: &mut AppState, node_id: u64, name: String, group: String)
```
Setzt/Aktualisiert einen Marker auf einem Node.

```rust
pub fn clear_node_marker(state: &mut AppState, node_id: u64)
```
Entfernt einen Marker vom Node.

---

### `selection` — Selektions-Operationen

Verarbeitet Node-Selektionen (Pick, Rechteck, Lasso, Segment) und koordiniert Undo-Snapshots.

**Funktionen:**

```rust
pub fn select_nearest_node(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    extend_path: bool,
)
```
Selektiert den nächstgelegenen Node zum Klickpunkt. 
- `additive = true`: Zur aktuellen Selektion hinzufügen
- `extend_path = true`: Pfad zwischen Anker und neuer Node selektieren
- **Undo:** Wenn die Selektion sich ändert, wird ein Snapshot mit der alten Selektion aufgezeichnet

```rust
pub fn select_segment(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
)
```
Selektiert das Segment zwischen den nächsten Kreuzungen (Intermediate Junctions).

```rust
pub fn select_in_rect(state: &mut AppState, min: glam::Vec2, max: glam::Vec2, additive: bool)
pub fn select_in_lasso(state: &mut AppState, polygon: &[glam::Vec2], additive: bool)
```
Selektiert Nodes innerhalb eines Rechtecks (Shift+Drag) oder Lasso-Polygons (Alt+Drag).

```rust
pub fn select_all(state: &mut AppState)
pub fn clear(state: &mut AppState)
pub fn invert(state: &mut AppState)
```
Bulk-Selektionen: Alle Nodes, Selektion aufheben, Selektion invertieren.

```rust
pub fn begin_move(state: &mut AppState)
pub fn move_selected(state: &mut AppState, delta_world: glam::Vec2)
```
Move-Lifecycle: `begin_move()` zeichnet einen Undo-Snapshot auf, `move_selected()` verschiebt die selektierten Nodes um das Delta.

---

### `editing` — Node/Connection-Editing und Marker

Verarbeitet alle Änderungen an der Road Map: Nodes hinzufügen/löschen, Connections, Marker.

**Funktionen:**

```rust
pub fn set_editor_tool(state: &mut AppState, tool: EditorTool)
```
Wechselt das aktive Editor-Werkzeug und setzt das `connect_source_node` zurück.

```rust
pub fn add_node(state: &mut AppState, world_pos: glam::Vec2)
```
Fügt einen neuen Node an der Position hinzu (oder selektiert einen bestehenden, falls die Position darin fällt).

```rust
pub fn delete_selected(state: &mut AppState)
```
Löscht alle selektierten Nodes.

```rust
pub fn connect_tool_pick(state: &mut AppState, world_pos: glam::Vec2, max_distance: f32)
```
Connect-Tool: Registriert einen Pick. Nach zwei Picks wird die Verbindung erstellt.

```rust
pub fn add_connection(
    state: &mut AppState,
    from_id: u64,
    to_id: u64,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
)
pub fn remove_connection_between(state: &mut AppState, node_a: u64, node_b: u64)
```
Erstellt oder entfernt Verbindungen.

```rust
pub fn set_connection_direction(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    direction: ConnectionDirection,
)
pub fn set_connection_priority(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    priority: ConnectionPriority,
)
```
Ändert Eigenschaften existierender Verbindungen.

```rust
pub fn set_default_direction(state: &mut AppState, direction: ConnectionDirection)
pub fn set_default_priority(state: &mut AppState, priority: ConnectionPriority)
```
Setzt Standard-Werte für neue Verbindungen (auch im aktiven Route-Tool).

```rust
pub fn set_all_directions_between_selected(state: &mut AppState, direction: ConnectionDirection)
pub fn remove_all_between_selected(state: &mut AppState)
pub fn invert_all_between_selected(state: &mut AppState)
pub fn set_all_priorities_between_selected(state: &mut AppState, priority: ConnectionPriority)
pub fn connect_selected(state: &mut AppState)
```
Bulk-Operationen auf Verbindungen zwischen selektierten Nodes. `connect_selected()` verbindet zwei selektierte Nodes bidirektional.

```rust
pub fn create_marker(state: &mut AppState, node_id: u64, name: &str, group: &str)
pub fn update_marker(state: &mut AppState, node_id: u64, name: &str, group: &str)
pub fn remove_marker(state: &mut AppState, node_id: u64)
```
Verwaltet Map-Marker (Labels für Nodes).

```rust
pub fn open_marker_dialog(state: &mut AppState, node_id: u64, is_new: bool)
```
Öffnet den Dialog zum Erstellen/Bearbeiten von Markern.

```rust
pub fn edit_segment(state: &mut AppState, record_id: u64)
```
Lädt ein gespeichertes Segment zur nachträglichen Bearbeitung. Löscht die zugehörigen Nodes, aktiviert das passende Route-Tool und befüllt es mit den gespeicherten Parametern.

```rust
pub fn resample_path(state: &mut AppState)
```
Verteilt die selektierten Nodes gleichmäßig entlang eines Catmull-Rom-Splines.

```rust
pub fn streckenteilung_aktivieren(state: &mut AppState)
```
Aktiviert die Streckenteilungs-Vorschau für die selektierten Nodes (min. 2 erforderlich).

---

### `view` — Kamera, Viewport und Background-Map

Steuert die Ansicht: Kamera-Navigation, Viewport-Größe, Background-Maps.

**Funktionen:**

```rust
pub fn reset_camera(state: &mut AppState)
pub fn zoom_in(state: &mut AppState)
pub fn zoom_out(state: &mut AppState)
```
Kamera-Steuerung (schrittweise Operationen).

```rust
pub fn pan(state: &mut AppState, delta: glam::Vec2)
pub fn zoom_towards(state: &mut AppState, factor: f32, focus_world: Option<glam::Vec2>)
```
Kontinuierliche Kamera-Bewegung (wird typischerweise pro Frame aufgerufen).

```rust
pub fn set_viewport_size(state: &mut AppState, size: [f32; 2])
pub fn set_render_quality(state: &mut AppState, quality: RenderQuality)
```
Viewport-Verwaltung und Render-Qualitäts-Konfiguration.

```rust
pub fn load_background_map(
    state: &mut AppState,
    path: String,
    crop_size: Option<u32>,
) -> anyhow::Result<()>
pub fn toggle_background_visibility(state: &mut AppState)
pub fn scale_background(state: &mut AppState, factor: f32)
```
Background-Map-Handling (Laden, Ein/Aus, Skalierung).

```rust
pub fn browse_zip_background(state: &mut AppState, path: String) -> anyhow::Result<()>
pub fn load_background_from_zip(
    state: &mut AppState,
    zip_path: String,
    entry_name: String,
    crop_size: Option<u32>,
) -> anyhow::Result<()>
pub fn generate_overview_with_options(state: &mut AppState) -> anyhow::Result<()>
pub fn save_background_as_overview(state: &mut AppState, path: String) -> anyhow::Result<()>
```
ZIP-Archiv-Support und Übersichtskarten-Generierung/Speicherung.

---

### `route_tool` — Route-Tool-Operationen

Verarbeitet Klicks, Drags und Konfigurationsänderungen für die Route-Tools (Gerade, Kurve, Spline, Bypass, Constraint).

**Funktionen:**

```rust
pub fn click(state: &mut AppState, world_pos: glam::Vec2, ctrl: bool)
```
Registriert einen Viewport-Klick beim aktiven Tool (mit optionalem `ctrl`-Modifier).

```rust
pub fn execute(state: &mut AppState)
pub fn cancel(state: &mut AppState)
```
`execute`: Erstellt die Strecke (Enter). `cancel`: Bricht das Tool ab (Escape).

```rust
pub fn select(state: &mut AppState, index: usize)
pub fn select_with_anchors(
    state: &mut AppState,
    index: usize,
    start_node_id: u64,
    end_node_id: u64,
)
```
Wechselt das aktive Tool. Mit `select_with_anchors` wird das Tool mit vordefiniertem Start/End aktiviert (simuliert zwei Klicks mit bekannten Node-Positionen). Bei StraightLine aktiviert dies sofort die Erstellung; bei Curves wird der Control-Punkt-Editor aktiviert.

```rust
pub fn recreate(state: &mut AppState)
```
Löscht die letzte erstellte Strecke und erstellt sie mit den aktuellen Tool-Parametern neu. Wird automatisch aufgerufen, wenn sich Konfiguration ändert und `needs_recreate()` true ist.

```rust
pub fn apply_tangent(state: &mut AppState, start: TangentSource, end: TangentSource)
```
Wendet die vom User gewählten Tangenten an und triggert ggf. eine Neuberechnung (für Cubic-Kurven).

```rust
pub fn drag_start(state: &mut AppState, world_pos: glam::Vec2)
pub fn drag_update(state: &mut AppState, world_pos: glam::Vec2)
pub fn drag_end(state: &mut AppState)
```
Drag-Lifecycle für Kontrollpunkt-Anpassung während der Tool-Ausführung.

```rust
pub fn increase_node_count(state: &mut AppState)
pub fn decrease_node_count(state: &mut AppState)
pub fn increase_segment_length(state: &mut AppState)
pub fn decrease_segment_length(state: &mut AppState)
```
Schnelle Konfigurationsanpassungen per Pfeiltasten (Numerische Feinabstimmung). Triggern automatisch `recreate()` wenn nötig.

---

### `history` — Undo/Redo-Verwaltung

Verarbeitet Undo/Redo-Operationen.

**Funktionen:**

```rust
pub fn undo(state: &mut AppState)
pub fn redo(state: &mut AppState)
```
Führt Undo/Redo-Operationen durch, indem Snapshots aus der History hergestellt werden.

---

### `helpers` — Zentrale Hilfsfunktionen für Undo und Selektion

Minimiert redundanten Code beim Aufnehmen von Undo-Snapshots und beim Vergleichen von Selektionszuständen.

**Funktionen:**

```rust
pub fn capture_selection_snapshot(state: &AppState) -> (Arc<IndexSet<u64>>, Option<u64>)
```
Erfasst den aktuellen Selektionszustand als Arc-Clone (O(1)) und Anchor-Node-ID.

```rust
pub fn record_selection_if_changed(
    state: &mut AppState,
    old_selected: Arc<IndexSet<u64>>,
    old_anchor: Option<u64>,
)
```
Vergleicht den übergebenen alten Selektionszustand mit dem aktuellen und legt einen Undo-Snapshot mit der alten Selektion an, falls sich etwas geändert hat. Wird häufig in Selection-Handlern verwendet:

```rust
let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
use_cases::selection::select_nearest_node(state, ...);
helpers::record_selection_if_changed(state, old_selected, old_anchor);
```

---

## Flow-Beispiel

**User klickt auf einen Node zum Selektieren:**

```
UI-Event (Klick)
  → AppIntent::NodePickRequested { world_pos, additive: false, extend_path: false }
  → map_intent_to_commands() → [AppCommand::SelectNearestNode { ... }]
  → controller.handle_intent() wählt handlers::selection::select_nearest_node()
  → snap_to_node() + use_cases::selection::select_nearest_node()
  → record_selection_if_changed() — Undo-Snapshot falls Selektion sich ändern
  → AppState.selection aktualisiert
  → controller.build_render_scene() nutzt die neue Selection
  → Rendering
```

---

## Undo/Redo-Strategie

Handler verwenden `state.history.record_snapshot(snapshot)` zum Capture des Vorher-Zustands:
- **Selections-Handler:** Snapshot mit `old_selection` vor dem Mutation
- **Editing-Handler:** Snapshot mit `old_road_map` vor Adds/Deletes/Modifications
- **File-IO-Handler:** Snapshot mit `old_road_map` und optionaler `old_file_path`

Siehe [`history.rs`](history.rs) für Details zur Edit-History-Verwaltung.

---

## Fehlerbehandlung

Handler geben typischerweise `anyhow::Result<()>` zurück für I/O-Operationen:
- `file_io::*` — Datei-Fehler
- `view::load_background_map()` — Bild-Fehler
- `route_tool::create_route()` — Ungültige Route

Der Controller in [`controller.rs`](../controller.rs) fängt Fehler ab und loggt sie.
