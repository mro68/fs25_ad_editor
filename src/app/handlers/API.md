# Handlers API Documentation

## Überblick

Das `handlers`-Modul gruppiert die Verarbeitung von `AppCommand`s in Feature-bereiche. Jeder Handler ist eine Sammlung von Funktionen, die einen spezifischen Aspekt der App mutieren — Datei-I/O, Selektion, Editing, etc.

**Architektur:**

1. Der `AppController` (in `controller.rs`) dispatcht jeden `AppCommand` ueber `controller/by_feature/*` anhand der internen `events::AppEventFeature`-Schnitte an den passenden Handler
2. Handler rufen Funktionen aus `use_cases/` auf — diese enthalten die Geschäftslogik
3. Handler selbst sind dünn und koordinieren hauptsächlich Undo-Snapshots und State-Updates

**Modulzugriff:**  
Die Handler-Module liegen unter [`src/app/handlers`](.) und werden intern vom Controller genutzt:

```rust
app::handlers::{dialog, editing, file_io, group, helpers, history, route_tool, selection, view}
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
pub fn open_trace_all_fields_dialog(state: &mut AppState)
pub fn close_trace_all_fields_dialog(state: &mut AppState)
pub fn request_curseplay_import_dialog(state: &mut AppState)
pub fn request_curseplay_export_dialog(state: &mut AppState)
```

Oeffnet die Datei- und Feature-Dialoge fuer Heightmap, Background-Map, Uebersichtskarten-ZIP, Batch-Feldnachzeichnen und Curseplay-Import/Export.

```rust
pub fn open_options_dialog(state: &mut AppState)
pub fn close_options_dialog(state: &mut AppState)
pub fn apply_options(state: &mut AppState, options: EditorOptions) -> anyhow::Result<()>
pub fn reset_options(state: &mut AppState) -> anyhow::Result<()>
pub fn toggle_command_palette(state: &mut AppState)
pub fn open_overview_options_dialog(state: &mut AppState, zip_path: String)
```

Verwaltet app-weite Dialoge und Overlay-Zustaende. `apply_options()` validiert und persistiert neue Optionen; `toggle_command_palette()` schaltet die Palette um; `open_overview_options_dialog()` bereitet den ZIP-basierten Overview-Flow vor.

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

Uebertraegt Scroll-basierte Rotation auf das aktive Route-Tool via `RouteToolRotate`. Wird typischerweise nur von ParkingTool verwendet (Alt+Scroll). Node-Count- und Segmentlaengen-Shortcuts laufen separat ueber `RouteToolSegmentAdjustments`.
Shortcut-Aufrufe ohne Route-Modus oder ohne passende Segment-Capability werden defensiv verworfen und nur als Debug-Log erfasst.

```rust
pub fn execute(state: &mut AppState)
```

Fuehrt das aktive Route-Tool aus (Enter-Bestaetigung). Erstellt Nodes + Connections, speichert Undo-Snapshot und registriert Gruppen-Record fuer nachtraegliche Bearbeitung.

**Group-Registry-Integration:**

- Nach Tool-Ausfuehrung werden die `original_positions` aus der RoadMap gesammelt
- Gruppen-Record wird mit allen Tool-Parametern registriert
- Ermoeglicht spaeteres Editieren: `EditGroup { record_id }` laedt das Tool mit gespeicherten Parametern neu

```rust
pub fn cancel(state: &mut AppState)
```

Bricht das aktive Route-Tool ab (Escape).

```rust
pub fn select(state: &mut AppState, tool_id: RouteToolId)
```

Aktiviert ein Route-Tool per stabiler Tool-ID. Initialisiert Tool-Parameter (Richtung, Prioritaet, Snap-Radius) aus EditorToolState, synchronisiert den `ToolHostContext` ins aktive Tool und laedt optional eine vorhandene Selektion als Kette (fuer Chain-basierte Tools wie BypassTool).

```rust
pub fn select_with_anchors(
    state: &mut AppState,
    tool_id: RouteToolId,
    start_node_id: u64,
    end_node_id: u64,
)
```

Aktiviert Tool und setzt Start/End-Anker aus zwei selektierten Nodes. Simuliert zwei `on_click()`-Aufrufe; bei StraightLine => sofortige Ausfuehrung, bei Curves => Phase::Control fuer Steuerpunkt-Platzierung.

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
pub fn edit_group(state: &mut AppState, record_id: u64)
```

Startet den destruktiven Tool-Edit fuer eine persistierte Tool-Gruppe. Fuehrt folgende Schritte aus:

1. **Lookup:** Laedt neutralen `GroupRecord` und passenden `ToolEditRecord` aus Registry bzw. `ToolEditStore`
2. **Marker-Cleanup:** Entfernt Marker der Gruppe anhand von `record.marker_node_ids`
3. **Node-Loeschung:** Loescht nur die inneren Tool-Nodes; ExistingNode-Anker aus dem Payload bleiben erhalten
4. **Tool-Rehydrierung:** Aktiviert das zugehoerige Route-Tool und stellt dessen Zustand via `RouteToolGroupEdit::restore_edit_payload()` wieder her
5. **Session-Backup:** Legt `ActiveToolEditSession` fuer Cancel/Undo an

**Tool-Editing-Integration:**

- Registry bleibt tool-neutral; tool-spezifische Parameter kommen aus `state.tool_edit_store`
- Cancel laeuft spaeter ueber `tool_editing::cancel_active_edit()` und stellt Registry plus Payload-Store wieder her
- Manuelle Gruppen sowie ephemere Tools besitzen keinen Tool-Edit-Snapshot und koennen diesen Flow deshalb nicht nutzen

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
pub fn set_node_flag(state: &mut AppState, node_id: u64, flag: NodeFlag)
```

Setzt das Flag eines vorhandenen Nodes (z.B. `Regular`, `SubPrio`) inklusive Undo-Snapshot ueber den Editing-Use-Case.

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
    stop_at_junction: bool,
    max_angle_deg: f32,
)
```

Selektiert das Segment zwischen den nächsten Segmentgrenzen. Abbruchbedingungen:
- `stop_at_junction = true`: Stopp bei Kreuzungen (Knotengrad != 2)
- `max_angle_deg > 0`: Stopp wenn die Richtungsänderung diesen Winkel überschreitet (0 = deaktiviert)

Beide Werte werden aus `EditorOptions` (via `AppCommand`) übergeben.

```rust
pub fn select_in_rect(state: &mut AppState, min: glam::Vec2, max: glam::Vec2, additive: bool)
pub fn select_in_lasso(state: &mut AppState, polygon: &[glam::Vec2], additive: bool)
```

Selektiert Nodes innerhalb eines Rechtecks (Shift+Drag) oder Lasso-Polygons (Alt+Drag).

```rust
pub fn select_group_nodes(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
)
```

Selektiert alle Nodes der Gruppe, zu der der nächste Node gehört (Doppelklick-Handler).
Findet den nächsten Node innerhalb `max_distance`, sucht den zugehörigen `GroupRecord` via `GroupRegistry::find_first_by_node_id()` und selektiert alle Node-IDs des Records.
Bei `additive = true` werden Gruppen-Nodes zur bestehenden Selektion hinzugefügt.
Tut nichts wenn kein Node gefunden oder der Node keiner Gruppe angehört.

```rust
pub fn select_all(state: &mut AppState)
pub fn clear(state: &mut AppState)
pub fn invert(state: &mut AppState)
```

Bulk-Selektionen: Alle Nodes, Selektion aufheben, Selektion invertieren.

```rust
pub fn begin_move(state: &mut AppState)
pub fn move_selected(state: &mut AppState, delta_world: glam::Vec2)
pub fn begin_rotate(state: &mut AppState)
pub fn rotate_selected(state: &mut AppState, delta_angle: f32)
pub fn end_rotate(state: &mut AppState)
```

Move-Lifecycle: `begin_move()` zeichnet einen Undo-Snapshot auf, `move_selected()` verschiebt die selektierten Nodes um das Delta.

Rotation-Lifecycle: `begin_rotate()` zeichnet einen Undo-Snapshot auf, `rotate_selected()` rotiert die selektierten Nodes um ihr Zentrum (kein Spatial-Rebuild), `end_rotate()` stößt den Spatial-Index-Rebuild ein.

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
pub fn trace_all_fields(state: &mut AppState)
```

Zeichnet alle geladenen Farmland-Polygone als Wegpunkt-Ring nach (Batch-Operation).
Alle Polygone werden in einem einzigen Undo-Schritt zusammengefasst.
Gibt fruehzeitig zurueck wenn keine Polygone geladen oder keine RoadMap vorhanden.

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
pub fn center_on_node(state: &mut AppState, node_id: u64)
```

Kontinuierliche Kamera-Bewegung (wird typischerweise pro Frame aufgerufen). `center_on_node()` springt gezielt auf einen vorhandenen Node, ohne den Zoom zu veraendern.

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

```rust
pub fn zoom_to_selection_bounds(state: &mut AppState)
pub fn zoom_to_fit(state: &mut AppState)
```

Zoom-Hilfsfunktionen: `zoom_to_selection_bounds` zoomt auf die Bounding Box der
aktuellen Selektion (keine Operation wenn Selektion leer). `zoom_to_fit` zoomt auf
die Selektion wenn vorhanden, sonst auf die gesamte RoadMap. Beide Funktionen
delegieren an `use_cases::camera` und haben keine Wirkung wenn keine RoadMap geladen ist.

---

### `route_tool` — Route-Tool-Operationen

Verarbeitet Klicks, Drags und semantische Panel-Aktionen fuer die Route-Tools (Gerade, Kurve, Spline, Bypass, Analyse-Tools).

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
pub fn select(state: &mut AppState, tool_id: RouteToolId)
pub fn select_with_anchors(
    state: &mut AppState,
    tool_id: RouteToolId,
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
pub fn apply_panel_action(state: &mut AppState, action: RouteToolPanelAction)
```

Wendet eine semantische Panel-Aktion aus dem Floating-Panel auf das aktive Tool an. Falls das Tool `RouteToolPanelEffect { needs_recreate: true, .. }` meldet, wird die letzte erzeugte Strecke automatisch neu aufgebaut.

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

### `group` — Gruppen-Lock, Popup- und Segment-Aufloesungs-Flow

Verwaltet den Lock-Zustand von Segmenten und loest Gruppen-Records auf.

**Funktionen:**

```rust
pub fn toggle_lock(state: &mut AppState, segment_id: u64)
```

Schaltet den Lock-Zustand eines Segments um. Gesperrte Segmente bewegen alle zugehoerigen Nodes gemeinsam beim Drag. Unbekannte IDs werden ignoriert.

```rust
pub fn dissolve(state: &mut AppState, segment_id: u64)
pub fn open_dissolve_confirm_dialog(state: &mut AppState, segment_id: u64)
pub fn open_settings_popup(state: &mut AppState, world_pos: glam::Vec2)
pub fn close_settings_popup(state: &mut AppState)
```

`dissolve()`: Entfernt nur den Gruppen-Record aus der Registry. Die zugehoerigen Nodes und Verbindungen in der RoadMap bleiben unveraendert. Wird **nach** Nutzer-Bestaetigung aufgerufen, nachdem `DissolveGroupRequested` zunaechst den Bestaetigungsdialog via `open_dissolve_confirm_dialog()` geoeffnet hat. Unbekannte IDs werden ignoriert.

`open_dissolve_confirm_dialog()`: Setzt den modalen Bestaetigungsdialog fuer die Segment-Aufloesung.

`open_settings_popup()`: Oeffnet bzw. aktualisiert das Gruppen-Einstellungs-Popup an der Weltposition eines Segment-Doppelklicks.

`close_settings_popup()`: Schliesst das Gruppen-Einstellungs-Popup wieder, wenn ein Selection-Command den Fokus aus dem Segment-Popup herausnimmt.
```rust
pub fn remove_selected_from_groups(state: &mut AppState)
```

Entfernt alle selektierten Nodes aus ihren zugehörigen Gruppen. Nodes und Verbindungen in der RoadMap bleiben unveraendert. Gruppen mit weniger als 2 verbleibenden Nodes werden automatisch aufgeloest (`GroupRegistry::remove_nodes_from_record()`). Ist keine Selektion aktiv oder kein Node Mitglied einer Gruppe, wird nichts getan. Erstellt einen Undo-Snapshot vor dem Mutieren.

```rust
pub fn set_boundary_nodes(state: &mut AppState, record_id: u64, entry: Option<u64>, exit: Option<u64>)
```

Setzt Einfahrt- und Ausfahrt-Node-IDs fuer eine Gruppe. Delegiert an `GroupRegistry::set_entry_exit()`: Die Node-IDs werden auf Zugehoerigkeit zum Record validiert; ungueltige IDs werden verworfen. Gibt eine Warnung aus wenn die Validierung fehlschlaegt (unbekannte Record-ID oder IDs ausserhalb des Records). Kein Undo-Snapshot — die Zuweisung ist sofort wirksam und im naechsten Frame sichtbar.

```rust
pub fn start_group_edit(state: &mut AppState, record_id: u64)
```

Startet den nicht-destruktiven Gruppen-Edit-Modus fuer einen Gruppen-Record. Erstellt einen Undo-Snapshot, entsperrt den Record temporaer (falls gesperrt), setzt den Edit-Guard in der GroupRegistry (verhindert automatische Invalidierung), und selektiert alle zugehoerigen Nodes. Gibt eine Warnung aus wenn der Record nicht existiert oder bereits ein Group-Edit aktiv ist. Setzt `AppState::group_editing` auf `Some(GroupEditState { record_id, was_locked })`.

```rust
pub fn apply_group_edit(state: &mut AppState)
```

Schliesst den Gruppen-Edit-Modus ab und uebernimmt alle Aenderungen. Berechnet die neue Node-ID-Menge als Vereinigung von (Original-Nodes, die noch in der RoadMap existieren) und (aktuell selektierten Nodes). **Verbindungsfilter:** Neu hinzugefügte selektierte Nodes werden nur übernommen, wenn sie eine direkte oder indirekte Verbindung zu einem bereits erreichbaren Node im Record haben (iterativer Erreichbarkeits-Algorithmus). Isolierte Nodes ohne Verbindung zur Gruppe werden verworfen. Aktualisiert den Record via `GroupRegistry::update_record()`, stellt den Lock-Zustand wieder her und hebt den Edit-Guard auf. Tut nichts wenn kein Edit aktiv ist.

```rust
pub fn cancel_group_edit(state: &mut AppState)
```

Bricht den Gruppen-Edit-Modus ab und stellt den Zustand ueber Undo wieder her. Der Undo-Snapshot wurde in `start_group_edit` angelegt. Hebt den Edit-Guard auf und setzt `group_editing` auf `None`.

```rust
pub fn begin_tool_edit_from_group(state: &mut AppState, record_id: u64)
```

Wechselt aus dem aktiven Gruppen-Edit-Modus in den destruktiven Tool-Edit-Modus fuer den angegebenen Record. Setzt voraus, dass `group_editing` aktiv ist.
Ablauf: `cleanup_group_edit_state()` (Edit-Guard aufheben, `group_editing` leeren) → Undo-Reset via `edit_group(state, record_id)` (loescht alte Nodes, laedt Tool mit gespeicherten Parametern neu). Gibt eine Warnung aus wenn kein Group-Edit aktiv ist.

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
