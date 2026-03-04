# Use-Cases API

Alle Use-Case-Funktionen des `app::use_cases`-Moduls. Use-Cases mutieren `AppState` direkt und enthalten die gesamte Business-Logik. Sie werden ausschließlich von Handler-Modulen (`app/handlers/`) aufgerufen.

**Zurück:** [`../API.md`](../API.md)

---

## `use_cases::camera`

- `reset_camera(state)` — Kamera auf Default zurücksetzen
- `zoom_in(state)` / `zoom_out(state)` — Stufenweise zoomen (Faktor 1.2)
- `pan(state, delta)` — Kamera verschieben (Delta in Welt-Einheiten)
- `zoom_towards(state, factor, focus_world)` — Zoom mit optionalem Fokuspunkt in Weltkoordinaten
- `center_on_road_map(state, road_map)` — Kamera auf Bounding-Box der geladenen RoadMap zentrieren

---

## `use_cases::file_io`

- `request_open_file(state)` — Open-Dialog triggern
- `load_selected_file(state, path)` — XML laden, Kamera zentrieren; anschließend wird automatisch die Post-Load-Detection ausgeführt (Heightmap + ZIP-Suche)
- `request_save_file(state)` — Save-Dialog triggern
- `save_current_file(state)` — Unter aktuellem Pfad speichern
- `save_file_as(state, path)` — Unter neuem Pfad speichern
- `save_with_heightmap_check(state, path)` — Speichern mit Heightmap-Prüfung (zeigt Warnung wenn nötig)
- `confirm_and_save(state)` — Speichern nach Bestätigung der Heightmap-Warnung

---

## `use_cases::heightmap`

- `request_heightmap_dialog(state)` — Heightmap-Dialog öffnen
- `clear_heightmap(state)` — Heightmap entfernen
- `set_heightmap(state, path)` — Heightmap setzen
- `dismiss_heightmap_warning(state)` — Heightmap-Warnung schließen

---

## `use_cases::selection`

- `select_nearest_node(state, world_pos, max_distance, additive, extend_path)` — Node per Klick selektieren; `additive` für Ctrl/Shift-Add, `extend_path` nur für Shift-Pfadselektion zwischen Anker und Ziel
- `select_segment_between_nearest_intersections(state, world_pos, max_distance, additive)` — Doppelklick selektiert den Korridor bis zu den nächsten Segmentgrenzen (Kreuzung oder Sackgassen-Endpunkt)
- `select_nodes_in_rect(state, corner_a, corner_b, additive)` — Rechteckselektion (Shift + Drag)
- `select_nodes_in_lasso(state, polygon, additive)` — Lasso-Selektion (Alt + Drag)
- `move_selected_nodes(state, delta_world)` — Alle selektierten Nodes gemeinsam verschieben
- `clear_selection(state)` — Selektion explizit löschen

---

## `use_cases::auto_detect`

- `detect_post_load(xml_path, map_name) -> PostLoadDetectionResult` — Sucht nach `terrain.heightmap.png` im XML-Verzeichnis und passenden Map-Mod-ZIPs im Mods-Verzeichnis (`../../mods/` relativ zum Savegame). Matching: case-insensitive, Underscores/Spaces als Wildcard, bidirektionale Umlaut-Expansion (ä↔ae, ö↔oe, ü↔ue, ß↔ss).

---

## `use_cases::editing`

- `add_node_at_position(state, world_pos) -> AddNodeResult` — Neuen Node einfügen oder existierenden selektieren

```rust
pub enum AddNodeResult {
    NoMap,
    SelectedExisting(u64), // Snap auf existierenden Node
    Created(u64),          // Neuer Node erstellt
}
```

- `delete_selected_nodes(state)` — Selektierte Nodes + betroffene Connections löschen
- `connect_tool_pick_node(state, world_pos, max_distance)` — Connect-Tool: Source/Target-Node auswählen
- `add_connection(state, from_id, to_id, direction, priority)` — Verbindung erstellen
- `remove_connection_between(state, node_a, node_b)` — Alle Verbindungen zwischen zwei Nodes entfernen
- `set_connection_direction(state, start_id, end_id, direction)` — Richtung ändern
- `set_connection_priority(state, start_id, end_id, priority)` — Priorität ändern
- `set_all_connections_direction_between_selected(state, direction)` — Bulk: Richtung aller Verbindungen zwischen Selektion ändern
- `remove_all_connections_between_selected(state)` — Bulk: Alle Verbindungen zwischen Selektion trennen
- `invert_all_connections_between_selected(state)` — Bulk: Richtung invertieren (start↔end)
- `set_all_connections_priority_between_selected(state, priority)` — Bulk: Priorität ändern
- `apply_tool_result(state, result) -> Vec<u64>` — Wendet ein `ToolResult` auf den AppState an (mit Undo-Snapshot): erstellt Nodes + Connections, setzt Selektion; ruft danach `make_segment_record()` auf dem aktiven Tool auf und speichert den Record in `state.segment_registry`
- `apply_tool_result_no_snapshot(state, result) -> Vec<u64>` — Wie `apply_tool_result`, aber ohne Undo-Snapshot (für Neuberechnung)
- `delete_nodes_by_ids(state, ids)` — Löscht Nodes mit den angegebenen IDs + zugehörige Connections; invalidiert betroffene Einträge in `state.segment_registry`
- `resample_selected_path(state)` — Selektierte Nodes-Kette per Catmull-Rom-Spline gleichmäßig neu verteilen; Konfiguration aus `state.ui.distanzen`

### `use_cases::editing::markers`

- `open_marker_dialog(state, node_id, is_new)` — Marker-Dialog öffnen (neu oder bearbeiten)
- `create_marker(state, node_id, &name, &group)` — Marker erstellen (mit Undo-Snapshot)
- `update_marker(state, node_id, &name, &group)` — Bestehenden Marker aktualisieren (mit Undo-Snapshot)
- `remove_marker(state, node_id)` — Marker eines Nodes entfernen (mit Undo-Snapshot)

---

## `use_cases::viewport`

- `resize(state, size)` — Viewport-Größe setzen
- `set_render_quality(state, quality)` — Kantenglättung steuern

---

## `use_cases::background_map`

- `request_background_map_dialog(state)` — Background-Map-Dialog öffnen
- `load_background_map(state, path, crop_size) -> anyhow::Result<()>` — Background-Map laden (PNG/JPG/DDS), Fehler werden an den Controller propagiert
- `toggle_background_visibility(state)` — Sichtbarkeit umschalten
- `clear_background_map(state)` — Background-Map entfernen
- `generate_overview_with_options(state) -> anyhow::Result<()>` — Übersichtskarte aus Map-Mod-ZIP generieren (Layer-Optionen aus Dialog-State), Einstellungen persistent speichern
- `save_background_as_overview(state, path) -> anyhow::Result<()>` — Aktuelle Background-Map als overview.jpg speichern (JPEG Qualität 90)

---

## `SegmentRegistry`

In-Session-Registry aller erstellten Segmente (für nachträgliche Bearbeitung).

- **Transient:** Wird **nicht** in Undo/Redo-Snapshots aufgenommen; leer nach Datei-Reload.
- **Gespeichert:** Alle Tool-Parameter (CPs, Tangenten, Anker, Richtung, Priorität, max_segment_length).
- **Invalidierung:** Beim manuellen Löschen von Nodes werden betroffene Records automatisch entfernt.

```rust
pub enum SegmentKind {
    Straight     { direction, priority, max_segment_length },
    CurveQuad    { cp1, direction, priority, max_segment_length },
    CurveCubic   { cp1, cp2, tangent_start, tangent_end, direction, priority, max_segment_length },
    Spline       { anchors, tangent_start, tangent_end, direction, priority, max_segment_length },
}

// Tool-Index-Konstanten (stimmen mit ToolManager::new()-Reihenfolge überein,
// abgesichert durch Unit-Test `tool_index_stimmt_mit_tool_manager_reihenfolge_ueberein`):
pub const TOOL_INDEX_STRAIGHT: usize = 0;
pub const TOOL_INDEX_CURVE_QUAD: usize = 1;
pub const TOOL_INDEX_CURVE_CUBIC: usize = 2;
pub const TOOL_INDEX_SPLINE: usize = 3;

pub struct SegmentRecord {
    pub id: u64,
    pub node_ids: Vec<u64>,
    pub start_anchor: ToolAnchor,
    pub end_anchor: ToolAnchor,
    pub kind: SegmentKind,
}
```

**Methoden:**

```rust
registry.register(record) -> u64
registry.get(record_id) -> Option<&SegmentRecord>
registry.remove(record_id)
registry.find_by_node_ids(node_ids: &IndexSet<u64>) -> Vec<&SegmentRecord>
registry.invalidate_by_node_ids(node_ids)  // bei manuellem Node-Löschen
registry.len() / is_empty()
```

### Bearbeitungs-Flow (`EditSegmentRequested`)

```
Properties-Panel (Button "Bearbeiten")
  → AppIntent::EditSegmentRequested { record_id }
  → AppCommand::EditSegment { record_id }
  → handlers::editing::edit_segment(state, record_id)
      1. Record aus Registry holen (Clone)
      2. Undo-Snapshot erstellen
      3. delete_nodes_by_ids() — Segment-Nodes aus RoadMap entfernen
      4. Registry-Record entfernen
      5. route_tool::select() — passendes Tool aktivieren
      6. tool.load_for_edit() — Tool mit gespeicherten Parametern befüllen
```

### `RouteTool`-Trait Erweiterungen (für Registry)

```rust
// Wird nach execute() + apply_tool_result() aufgerufen:
fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord>;

// Wird in edit_segment() aufgerufen um das Tool wiederherzustellen:
fn load_for_edit(&mut self, record: &SegmentRecord, kind: &SegmentKind);
```

Implementierungen: `StraightLineTool`, `CurveTool` (Quad + Cubic), `SplineTool`.
