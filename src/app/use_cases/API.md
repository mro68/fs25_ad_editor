# Use-Cases API

Alle Use-Case-Funktionen des `app::use_cases`-Moduls. Use-Cases mutieren `AppState` direkt und enthalten die gesamte Business-Logik. Sie werden ausschliesslich von Handler-Modulen (`app/handlers/`) aufgerufen.

Seit Phase 7 schneiden `controller.rs` und `intent_mapping.rs` die Control-Plane zwar intern in `by_feature/*`-Slices, die Grenze zu `use_cases` bleibt aber unveraendert: Handler bleiben die einzige Schreib-Schale oberhalb der Use-Cases; Intents und Commands dringen nicht in dieses Modul ein.

**Zurueck:** [`../API.md`](../API.md)

---

## `use_cases::camera`

- `reset_camera(state)` — Kamera auf Default zuruecksetzen
- `zoom_in(state)` — Stufenweise hineinzoomen (Faktor 1.2)
- `zoom_out(state)` — Stufenweise herauszoomen (Faktor 1.2)
- `pan(state, delta)` — Kamera verschieben (Delta in Welt-Einheiten)
- `zoom_towards(state, factor, focus_world)` — Zoom mit optionalem Fokuspunkt in Weltkoordinaten
- `center_on_road_map(state, road_map)` — Kamera auf Bounding-Box der geladenen RoadMap zentrieren
- `zoom_to_selection_bounds(state, road_map)` — Kamera auf die Bounding Box der aktuellen Selektion zoomen; Padding-Faktor 80 % (Konstante `SELECTION_ZOOM_PADDING`); keine Operation wenn Selektion leer oder keine selektierten Nodes in der RoadMap vorhanden
- `zoom_to_fit(state, road_map)` — Zoomt auf Selektion wenn vorhanden, sonst auf die gesamte RoadMap (delegiert an `zoom_to_selection_bounds` bzw. `center_on_road_map`)

---

## `use_cases::file_io`

- `request_open_file(state)` — Open-Dialog triggern
- `load_selected_file(state, path)` — XML laden, Duplikate zaehlen, Kamera zentrieren und Dateipfad im State setzen; die Post-Load-Detection laeuft anschliessend im File-I/O-Handler
- `deduplicate_loaded_roadmap(state)` — Fuehrt die Duplikat-Bereinigung auf der geladenen RoadMap aus und aktualisiert Status-/Dialog-State
- `request_save_file(state)` — Save-Dialog triggern
- `save_current_file(state)` — Unter aktuellem Pfad speichern
- `save_file_as(state, path)` — Unter neuem Pfad speichern
- `save_with_heightmap_check(state, path)` — Speichern mit Heightmap-Pruefung (zeigt Warnung wenn noetig)
- `confirm_and_save(state)` — Speichern nach Bestaetigung der Heightmap-Warnung

---

## `use_cases::options`

- `config_path() -> PathBuf` — Standardpfad der Optionen-Datei neben der Binary bestimmen
- `load_editor_options() -> EditorOptions` — Optionen vom Standardpfad laden; bei Fehlern Defaults verwenden
- `load_editor_options_from_file(path) -> EditorOptions` — Optionen aus einer konkreten TOML-Datei laden; Legacy-Prozentwerte normalisieren und validieren
- `save_editor_options(options) -> anyhow::Result<()>` — Optionen am Standardpfad validieren und speichern
- `save_editor_options_to_file(path, options) -> anyhow::Result<()>` — Optionen als TOML an einen konkreten Pfad schreiben

---

## `use_cases::heightmap`

- `request_heightmap_dialog(state)` — Heightmap-Dialog oeffnen
- `clear_heightmap(state)` — Heightmap entfernen
- `set_heightmap(state, path)` — Heightmap setzen
- `dismiss_heightmap_warning(state)` — Heightmap-Warnung schliessen

---

## `use_cases::selection`

- `select_nearest_node(state, world_pos, max_distance, additive, extend_path)` — Node per Klick selektieren; `additive` fuer Ctrl/Shift-Add, `extend_path` nur fuer Shift-Pfadselektion zwischen Anker und Ziel.
- `select_segment_between_nearest_intersections(state, world_pos, max_distance, additive, stop_at_junction, max_angle_deg)` — Doppelklick selektiert den Korridor bis zu den naechsten Segmentgrenzen. `stop_at_junction`: Stopp bei Kreuzungen (Grad != 2); `max_angle_deg`: harter Winkel-Constraint — Kandidaten mit Abweichung > Schwellwert werden verworfen (0.0 = deaktiviert). An Kreuzungen erfolgt score-basierte Auswahl: gleiche Strassenart wie Hit-Node (+40), `Regular`-Prioritaet (+20), gerichtete Verbindung (+10), geringe Winkelabweichung (+0..+10). Bei >2 Pfaden: Sortierung absteigend nach Strassenart-Match-Anzahl, Kuerzung auf 2. Konfiguration wird aus `EditorOptions` uebergeben.
- `select_group_by_nearest_node(state, world_pos, max_distance, additive)` — Selektiert per Doppelklick alle Nodes der Gruppe, zu der der getroffene Node gehoert; `additive = true` fuegt die Gruppenselektion zur bestehenden Selektion hinzu
- `select_nodes_in_rect(state, corner_a, corner_b, additive)` — Rechteckselektion (Shift + Drag)
- `select_nodes_in_lasso(state, polygon, additive)` — Lasso-Selektion (Alt + Drag)
- `move_selected_nodes(state, delta_world)` — Alle selektierten Nodes gemeinsam verschieben; gesperrte Gruppen werden ueber `GroupRegistry::expand_locked_selection()` mitgezogen, `original_positions` betroffener Locks werden aktualisiert, der Spatial-Index wird dabei bewusst noch nicht rebuilt und muss ueber den Move-Lifecycle separat abgeschlossen werden
- `rotate_selected_nodes(state, angle_rad)` — Alle selektierten Nodes um ihr gemeinsames Zentrum rotieren (Spatial-Index **nicht** rebuilt — muss separat per `EndRotateSelectedNodes` angestossen werden)
- `clear_selection(state)` — Selektion explizit loeschen

---

## `use_cases::auto_detect`

- `detect_post_load(xml_path, map_name) -> PostLoadDetectionResult` — Sucht nach `terrain.heightmap.png` im XML-Verzeichnis und passenden Map-Mod-ZIPs im Mods-Verzeichnis (`../../mods/` relativ zum Savegame). Matching: case-insensitive, Underscores/Spaces als Wildcard, bidirektionale Umlaut-Expansion (ae↔ae, oe↔oe, ue↔ue, ss↔ss).

---

## `use_cases::editing`

- `add_node_at_position(state, world_pos) -> AddNodeResult` — Neuen Node einfuegen oder existierenden selektieren; der Rueckgabewert ist Teil des Workflow-Vertrags, damit Handler No-Map-, Snap- und Create-Faelle explizit surfacen koennen

```rust
pub enum AddNodeResult {
    NoMap,
    SelectedExisting(u64), // Snap auf existierenden Node
    Created(u64),          // Neuer Node erstellt
}
```

- `delete_selected_nodes(state)` — Selektierte Nodes + betroffene Connections loeschen
- `connect_tool_pick_node(state, world_pos, max_distance)` — Connect-Tool: Source/Target-Node auswaehlen
- `add_connection(state, from_id, to_id, direction, priority)` — Verbindung erstellen
- `remove_connection_between(state, node_a, node_b)` — Alle Verbindungen zwischen zwei Nodes entfernen
- `set_connection_direction(state, start_id, end_id, direction)` — Richtung aendern
- `set_connection_priority(state, start_id, end_id, priority)` — Prioritaet aendern
- `set_node_flag(state, node_id, flag)` — Node-Flag direkt setzen (mit Undo-Snapshot)
- `set_all_connections_direction_between_selected(state, direction)` — Bulk: Richtung aller Verbindungen zwischen Selektion aendern
- `remove_all_connections_between_selected(state)` — Bulk: Alle Verbindungen zwischen Selektion trennen
- `invert_all_connections_between_selected(state)` — Bulk: Richtung invertieren (start↔end)
- `set_all_connections_priority_between_selected(state, priority)` — Bulk: Prioritaet aendern
- `apply_tool_result(state, result) -> Vec<u64>` — Wendet ein `ToolResult` auf den AppState an (mit Undo-Snapshot): erstellt Nodes + Connections und setzt die Selektion; Persistenz in `GroupRegistry`/`ToolEditStore` passiert anschliessend separat im Route-Tool-Handler ueber `tool_editing::persist_after_apply()`
- `apply_tool_result_no_snapshot(state, result) -> Vec<u64>` — Wie `apply_tool_result`, aber ohne Undo-Snapshot (fuer Neuberechnung)
- `delete_nodes_by_ids(state, ids)` — Loescht Nodes mit den angegebenen IDs + zugehoerige Connections; invalidiert betroffene Eintraege in `state.group_registry` und entfernt die passenden Payloads aus `state.tool_edit_store`
- `resample_selected_path(state)` — Selektierte Nodes-Kette per Catmull-Rom-Spline gleichmaessig neu verteilen; Konfiguration aus `state.ui.distanzen`
- `trace_all_fields(state, spacing, offset, tolerance, corner_angle, corner_rounding_radius, corner_rounding_max_angle_deg)` — Zeichnet alle geladenen Farmland-Polygone als Wegpunkt-Ring nach (Batch-Operation). Nutzt die uebergebenen Feldgrenzen-Parameter fuer Abstand, Versatz, Begradigung, Ecken-Erkennung und optionale Eckenverrundung; alle Polygone werden in einem einzigen Undo-Schritt zusammengefasst, Spatial-Index-Rebuild und Flag-Berechnung erfolgen nur einmal am Ende.
- `copy_selected_to_clipboard(state)` — Kopiert die aktuelle Selektion inklusive interner Verbindungen und Marker in die Zwischenablage und speichert das geometrische Zentrum als Paste-Referenz
- `start_paste_preview(state)` — Aktiviert den Einfuegen-Vorschau-Modus auf Basis des Clipboard-Zentrums
- `update_paste_preview(state, world_pos)` — Aktualisiert die aktuelle Paste-Vorschauposition im Weltkoordinatensystem
- `confirm_paste(state)` — Fuegt die Zwischenablage an der aktuellen Vorschauposition ein, remappt IDs, baut Geometrie/Spatial-Index neu auf und selektiert die neuen Nodes
- `cancel_paste_preview(state)` — Bricht den Paste-Vorschau-Modus ohne Mutation ab
- `import_curseplay(state, path)` — Importiert eine Curseplay-`<customField>`-XML-Datei: Liesst Vertices, erstellt einen MapNode (Regular, Y=0.0) pro Vertex und verbindet aufeinanderfolgende Paare bidirektional als Dual/SubPriority-Ring (letzter→erster schliesst den Ring). Nimmt vor der Mutation einen Undo-Snapshot. Bricht fruehzeitig ab wenn keine RoadMap geladen ist oder die Datei keine Vertices enthaelt.
- `export_curseplay(state, path)` — Exportiert die selektierten Nodes in Selektionsreihenfolge als Curseplay-`<customField>`-XML-Datei. Bricht fruehzeitig ab bei leerer Selektion oder fehlender RoadMap.

### `use_cases::editing::markers`

- `open_marker_dialog(state, node_id, is_new)` — Marker-Dialog oeffnen (neu oder bearbeiten)
- `create_marker(state, node_id, &name, &group)` — Marker erstellen (mit Undo-Snapshot)
- `update_marker(state, node_id, &name, &group)` — Bestehenden Marker aktualisieren (mit Undo-Snapshot)
- `remove_marker(state, node_id)` — Marker eines Nodes entfernen (mit Undo-Snapshot)

---

## `use_cases::viewport`

- `resize(state, size)` — Viewport-Groesse setzen
- `set_render_quality(state, quality)` — Kantenglaettung steuern

---

## `use_cases::background_map`

- `request_background_map_dialog(state)` — Background-Map-Dialog oeffnen
- `load_background_map(state, path, crop_size) -> anyhow::Result<()>` — Background-Map laden (PNG/JPG/DDS), Fehler werden an den Controller propagiert
- `toggle_background_visibility(state)` — Sichtbarkeit umschalten
- `scale_background(state, factor)` — Skalierungsfaktor relativ anpassen (Multiplikation; Bereich 0.125–8.0)
- `clear_background_map(state)` — Background-Map entfernen
- `browse_zip_background(state, path) -> anyhow::Result<()>` — ZIP-Archiv nach Bilddateien durchsuchen; bei einem Treffer wird direkt geladen
- `load_background_from_zip(state, zip_path, entry_name, crop_size) -> anyhow::Result<()>` — Einzelne Bilddatei aus ZIP als Background laden
- `generate_overview_with_options(state) -> anyhow::Result<()>` — Uebersichtskarte aus Map-Mod-ZIP generieren (Layer-Optionen aus Dialog-State), Einstellungen persistent speichern; Persistenzfehler werden per `log::warn!` und `state.ui.status_message` sichtbar gemacht, die Generierung selbst laeuft weiter
- `save_background_as_overview(state, path) -> anyhow::Result<()>` — Aktuelle Background-Map als overview.jpg speichern (JPEG Qualitaet 90), Farmland-Polygone als `.json` daneben
- `load_farmland_json(state, image_path)` — Laedt Farmland-Polygone aus einer `.json`-Datei neben der Bilddatei (z.B. `overview.json` neben `overview.jpg`); lautlos keine-Op wenn Datei fehlt

---

## `GroupRegistry` und `tool_editing`

Die Registry ist seit Phase 4 tool-neutral; tool-spezifische Persistenz liegt separat in `app/tool_editing`.

- **`GroupRegistry`** speichert nur neutrale Gruppendaten (`GroupRecord` mit `id`, `node_ids`, `original_positions`, `marker_node_ids`, `locked`, `entry_node_id`, `exit_node_id`).
- **`ToolEditStore`** haelt `ToolEditRecord { group_id, tool_id, payload }` fuer group-backed editierbare Tools.
- **Undo/Redo-Vertrag:** `Snapshot` in `app/history.rs` sichert `road_map`, `selection`, `group_registry` und `tool_edit_store`; laufende `ActiveToolEditSession`s bleiben bewusst transient und werden bei Cancel-/Restore-Flows separat aus Backups rekonstruiert.
- **Invalidierung:** Beim manuellen Loeschen oder Resampling von Nodes liefert `invalidate_by_node_ids(...)` die entfernten Record-IDs zurueck; die Caller entfernen damit die passenden Tool-Payloads aus `state.tool_edit_store`.

### Bearbeitungs-Flow (`GroupEditToolRequested` / `EditGroup`)

```
Gruppen-Edit-Panel (Button "Tool bearbeiten")
  → AppIntent::GroupEditToolRequested { record_id }
  → AppCommand::BeginToolEditFromGroup { record_id }
  → handlers::group::begin_tool_edit_from_group(state, record_id)
      1. Nicht-destruktiven Gruppen-Edit aufraeumen
      2. Undo auf Snapshot vor Gruppen-Edit
      3. handlers::editing::edit_group(state, record_id)
          a. GroupRecord + ToolEditRecord laden
          b. Marker bereinigen, innere Nodes loeschen, Anker schuetzen
          c. Route-Tool aktivieren und `restore_edit_payload()` aufrufen
          d. `ActiveToolEditSession` fuer Cancel/Undo anlegen
```

### `RouteToolGroupEdit`

```rust
fn build_edit_payload(&self) -> Option<RouteToolEditPayload>;
fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload);
```

Implementierungen: `StraightLineTool`, `CurveTool` (Quad + Cubic), `SplineTool`, `BypassTool`, `SmoothCurveTool`, `ParkingTool`, `RouteOffsetTool`, `FieldBoundaryTool`.
