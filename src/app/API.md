# App API Documentation

## Überblick

Das `app`-Modul verwaltet den globalen State, verarbeitet `AppIntent`s zentral über den `AppController`, mappt diese auf `AppCommand`s und baut die `RenderScene` für das Rendering.

**Hinweis:** `Camera2D` lebt im `core`-Modul (reiner Geometrie-Typ). `app` re-exportiert `Camera2D`, `ConnectionDirection`, `ConnectionPriority` und `RoadMap` aus `core`.

## Haupttypen

### `AppController`

Zentrale Intent-Verarbeitung, Command-Ausführung und Render-Scene-Aufbau.

```rust
pub struct AppController;
```

**Methoden:**

```rust
let mut controller = AppController::new();
let mut state = AppState::new();

controller.handle_intent(&mut state, AppIntent::ZoomInRequested)?;
let scene = controller.build_render_scene(&state, [width, height]);
```

**Features:**
- Verarbeitet UI- und Input-Intents gegen `AppState`
- Mappt Intents auf Commands
- Führt Commands über Use-Cases aus (z. B. Datei laden, Heightmap setzen)
- Baut den expliziten Render-Vertrag (`RenderScene`)

---

### `AppState`

Zentraler Anwendungszustand. Enthält keine I/O-Logik — alle Dateisystem-Operationen sind in `use_cases::file_io` zentralisiert.

```rust
pub struct AppState {
    pub road_map: Option<Arc<RoadMap>>,
    pub view: ViewState,
    pub ui: UiState,
    pub selection: SelectionState,
    pub editor: EditorToolState,
    pub command_log: CommandLog,
    pub history: EditHistory,
    pub options: EditorOptions,
    pub show_options_dialog: bool,
    pub should_exit: bool,
}

pub struct SelectionState {
    pub selected_node_ids: HashSet<u64>,
    pub selection_anchor_node_id: Option<u64>,
}

pub struct UiState {
    pub show_file_dialog: bool,
    pub show_save_file_dialog: bool,
    pub show_heightmap_dialog: bool,
    pub show_background_map_dialog: bool,
    pub show_heightmap_warning: bool,
    pub heightmap_warning_confirmed: bool,
    pub pending_save_path: Option<String>,
    pub current_file_path: Option<String>,
    pub heightmap_path: Option<String>,
    pub show_marker_dialog: bool,
    pub marker_dialog_node_id: Option<u64>,
    pub marker_dialog_name: String,
    pub marker_dialog_group: String,
    pub marker_dialog_is_new: bool,
}

pub struct ViewState {
    pub camera: Camera2D,
    pub viewport_size: [f32; 2],
    pub render_quality: RenderQuality,
    pub background_map: Option<Arc<BackgroundMap>>,
    pub background_opacity: f32,
    pub background_visible: bool,
    pub background_dirty: bool,  // GPU-Upload-Signal
}

pub struct EditorToolState {
    pub active_tool: EditorTool,
    pub connect_source_node: Option<u64>,
    pub default_direction: ConnectionDirection,
    pub default_priority: ConnectionPriority,
}
```

**Methoden:**

```rust
let state = AppState::new();
let nodes = state.node_count();
let connections = state.connection_count();
let can_undo = state.can_undo();
let can_redo = state.can_redo();
```

---

### `EditorTool`

Aktives Editor-Werkzeug.

```rust
pub enum EditorTool {
    Select,   // Standard: Nodes selektieren und verschieben
    Connect,  // Verbindungen zwischen Nodes erstellen
    AddNode,  // Neue Nodes auf der Karte platzieren
}
```

---

### `AppIntent` und `AppCommand`

`AppIntent` beschreibt Eingaben aus UI/System. `AppCommand` beschreibt mutierende Schritte am State.

```rust
pub enum AppIntent {
    // Datei-Operationen
    OpenFileRequested,
    SaveRequested,
    SaveAsRequested,
    ExitRequested,
    FileSelected { path: String },
    SaveFilePathSelected { path: String },

    // Heightmap
    HeightmapSelectionRequested,
    HeightmapCleared,
    HeightmapSelected { path: String },
    HeightmapWarningConfirmed,
    HeightmapWarningCancelled,

    // Kamera / Viewport
    ResetCameraRequested,
    ZoomInRequested,
    ZoomOutRequested,
    ViewportResized { size: [f32; 2] },
    CameraPan { delta: glam::Vec2 },
    CameraZoom { factor: f32, focus_world: Option<glam::Vec2> },
    RenderQualityChanged { quality: RenderQuality },

    // Selektion
    NodePickRequested { world_pos: glam::Vec2, additive: bool, extend_path: bool },
    NodeSegmentBetweenIntersectionsRequested { world_pos: glam::Vec2, additive: bool },
    SelectNodesInRectRequested { min: glam::Vec2, max: glam::Vec2, additive: bool },
    SelectNodesInLassoRequested { polygon: Vec<glam::Vec2>, additive: bool },

    // Move-Lifecycle (Drag-Verschieben selektierter Nodes)
    BeginMoveSelectedNodesRequested,
    MoveSelectedNodesRequested { delta_world: glam::Vec2 },
    EndMoveSelectedNodesRequested,

    // Undo / Redo
    UndoRequested,
    RedoRequested,

    // Editor-Werkzeug
    SetEditorToolRequested { tool: EditorTool },

    // Editing
    AddNodeRequested { world_pos: glam::Vec2 },
    DeleteSelectedRequested,
    ConnectToolNodeClicked { world_pos: glam::Vec2 },
    AddConnectionRequested { from_id: u64, to_id: u64, direction: ConnectionDirection, priority: ConnectionPriority },
    RemoveConnectionBetweenRequested { node_a: u64, node_b: u64 },
    SetConnectionDirectionRequested { start_id: u64, end_id: u64, direction: ConnectionDirection },
    SetConnectionPriorityRequested { start_id: u64, end_id: u64, priority: ConnectionPriority },
    SetDefaultDirectionRequested { direction: ConnectionDirection },
    SetDefaultPriorityRequested { priority: ConnectionPriority },

    // Bulk-Operationen auf selektierten Verbindungen
    SetAllConnectionsDirectionBetweenSelectedRequested { direction: ConnectionDirection },
    RemoveAllConnectionsBetweenSelectedRequested,
    InvertAllConnectionsBetweenSelectedRequested,
    SetAllConnectionsPriorityBetweenSelectedRequested { priority: ConnectionPriority },
    ConnectSelectedNodesRequested,

    // Background-Map
    BackgroundMapSelectionRequested,
    BackgroundMapSelected { path: String, crop_size: Option<u32> },
    SetBackgroundOpacity { opacity: f32 },
    ToggleBackgroundVisibility,

    // Map-Marker
    CreateMarkerRequested { node_id: u64 },
    RemoveMarkerRequested { node_id: u64 },
    EditMarkerRequested { node_id: u64 },
    MarkerDialogConfirmed { node_id: u64, name: String, group: String, is_new: bool },
    MarkerDialogCancelled,

    // Selektion (Bulk)
    ClearSelectionRequested,
    SelectAllRequested,

    // Optionen
    OpenOptionsDialogRequested,
    CloseOptionsDialogRequested,
    OptionsChanged { options: EditorOptions },
    ResetOptionsRequested,
}

pub enum AppCommand {
    // Datei-Operationen
    LoadFile { path: String },
    SaveFile { path: String },
    ExitApp,

    // Kamera
    ResetCamera,
    Pan { delta: glam::Vec2 },
    ZoomTowards { factor: f32, focus_world: Option<glam::Vec2> },
    CenterOnRoadMap,
    SetRenderQuality { quality: RenderQuality },
    ResizeViewport { size: [f32; 2] },

    // Selektion
    SelectNearest { world_pos: glam::Vec2, additive: bool, extend_path: bool },
    SelectSegment { world_pos: glam::Vec2, additive: bool },
    SelectInRect { min: glam::Vec2, max: glam::Vec2, additive: bool },
    SelectInLasso { polygon: Vec<glam::Vec2>, additive: bool },
    ClearSelection,
    SelectAll,
    BeginMoveSelected,
    MoveSelected { delta_world: glam::Vec2 },
    EndMoveSelected,

    // Editing
    AddNode { world_pos: glam::Vec2 },
    DeleteSelected,
    ConnectToolPick { world_pos: glam::Vec2 },
    AddConnection { from_id: u64, to_id: u64, direction: ConnectionDirection, priority: ConnectionPriority },
    // ... und weitere Varianten für Connection-Bearbeitung, Marker, Background, Heightmap, Optionen
}
```

---

### `Camera2D` (re-exportiert aus `core`)

2D-Kamera mit Pan und Zoom. Siehe `core/API.md` für Details.

## Use-Cases

### `use_cases::camera`
- `reset_camera(state)` — Kamera auf Default zurücksetzen
- `zoom_in(state)` / `zoom_out(state)` — Stufenweise zoomen (Faktor 1.2)
- `pan(state, delta)` — Kamera verschieben (Delta in Welt-Einheiten)
- `zoom_towards(state, factor, focus_world)` — Zoom mit optionalem Fokuspunkt in Weltkoordinaten
- `center_on_road_map(state, road_map)` — Kamera auf Bounding-Box der geladenen RoadMap zentrieren

### `use_cases::file_io`
- `request_open_file(state)` — Open-Dialog triggern
- `load_selected_file(state, path)` — XML laden, Kamera zentrieren
- `request_save_file(state)` — Save-Dialog triggern
- `save_current_file(state)` — Unter aktuellem Pfad speichern
- `save_file_as(state, path)` — Unter neuem Pfad speichern
- `save_with_heightmap_check(state, path)` — Speichern mit Heightmap-Prüfung (zeigt Warnung wenn nötig)
- `confirm_and_save(state)` — Speichern nach Bestätigung der Heightmap-Warnung

### `use_cases::heightmap`
- `request_heightmap_dialog(state)` — Heightmap-Dialog öffnen
- `clear_heightmap(state)` — Heightmap entfernen
- `set_heightmap(state, path)` — Heightmap setzen
- `dismiss_heightmap_warning(state)` — Heightmap-Warnung schließen

### `use_cases::selection`
- `select_nearest_node(state, world_pos, max_distance, additive, extend_path)` — Node per Klick selektieren; `additive` für Ctrl/Shift-Add, `extend_path` nur für Shift-Pfadselektion zwischen Anker und Ziel
- `select_segment_between_nearest_intersections(state, world_pos, max_distance, additive)` — Doppelklick selektiert den Korridor bis zu den nächsten Segmentgrenzen (Kreuzung oder Sackgassen-Endpunkt)
- `select_nodes_in_rect(state, corner_a, corner_b, additive)` — Rechteckselektion (Shift + Drag)
- `select_nodes_in_lasso(state, polygon, additive)` — Lasso-Selektion (Alt + Drag)
- `move_selected_nodes(state, delta_world)` — Alle selektierten Nodes gemeinsam verschieben
- `clear_selection(state)` — Selektion explizit löschen

### `use_cases::editing`
- `add_node_at_position(state, world_pos)` — Neuen Node einfügen
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

### `use_cases::viewport`
- `resize(state, size)` — Viewport-Größe setzen
- `set_render_quality(state, quality)` — Kantenglättung steuern

### `use_cases::background_map`
- `request_background_map_dialog(state)` — Background-Map-Dialog öffnen
- `load_background_map(state, path, crop_size)` — Background-Map laden (PNG/JPG/DDS)
- `set_background_opacity(state, opacity)` — Opacity setzen (0.0–1.0)
- `toggle_background_visibility(state)` — Sichtbarkeit umschalten
- `clear_background_map(state)` — Background-Map entfernen

### `use_cases::editing::markers`
- `open_marker_dialog(state, node_id, is_new)` — Marker-Dialog öffnen (neu oder bearbeiten)
- `create_marker(state, node_id, &name, &group)` — Marker erstellen (mit Undo-Snapshot)
- `update_marker(state, node_id, &name, &group)` — Bestehenden Marker aktualisieren (mit Undo-Snapshot)
- `remove_marker(state, node_id)` — Marker eines Nodes entfernen (mit Undo-Snapshot)

## Interaktions-Pattern

### Typisches Update-Loop (Intent-basiert)

```rust
let mut intents = Vec::new();
intents.push(AppIntent::ZoomInRequested);

for intent in intents {
    controller.handle_intent(&mut state, intent)?;
}

let scene = controller.build_render_scene(&state, [viewport_w, viewport_h]);
```

### Pan-Delta-Umrechnung

Das Maus-Delta (Pixel) wird vor dem Intent in Welt-Einheiten umgerechnet:

```rust
let wpp = camera.world_per_pixel(viewport_height);
AppIntent::CameraPan { delta: Vec2::new(-dx * wpp, -dy * wpp) }
```

## Design-Prinzipien

1. **Single Source of Truth:** `AppState` hält die Laufzeitdaten (kein I/O)
2. **Intent Boundary:** UI mutiert nicht direkt, sondern emittiert `AppIntent`
3. **Command Execution:** `AppController` mappt Intents auf Commands und führt diese aus
4. **Render Contract:** Ausgabe an Renderer erfolgt nur über `RenderScene`
5. **I/O in Use-Cases:** Dateisystem-Operationen sind in `use_cases::file_io` zentralisiert
6. **Re-Exports:** `app` re-exportiert `Camera2D`, `ConnectionDirection`, `ConnectionPriority`, `RoadMap` aus `core`, damit UI nicht direkt auf `core` zugreift

## Weitere Typen

### `CommandLog`

Protokoliert ausgeführte AppCommands für Debugging.

```rust
pub struct CommandLog { /* intern */ }
```

**Methoden:**
- `new() → Self`
- `log(command: &AppCommand)` — Command protokollieren
- `entries() → &[String]` — Alle Einträge abrufen
- `clear()` — Log leeren

---

### `EditHistory` / `Snapshot`

COW-basiertes Undo/Redo-System.

```rust
pub struct EditHistory { /* intern */ }
pub struct Snapshot { /* intern */ }
```

**EditHistory-Methoden:**
- `new() → Self`
- `record_snapshot(snapshot)` — Snapshot auf den Undo-Stack legen
- `undo() → Option<Snapshot>` — Letzten Snapshot wiederherstellen
- `redo() → Option<Snapshot>` — Redo-Snapshot wiederherstellen
- `can_undo() → bool` / `can_redo() → bool`

**AppState Helper:**
- `record_undo_snapshot(&mut self)` — Convenience-Methode, erstellt Snapshot und legt ihn auf den History-Stack

---

### `render_scene::build()`

Baut die `RenderScene` aus dem aktuellen `AppState` und der Viewport-Größe.

```rust
pub fn build(state: &AppState, viewport_size: [f32; 2]) -> RenderScene
```
