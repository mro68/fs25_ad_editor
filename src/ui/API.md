# UI API Documentation

## Überblick

Das `ui`-Modul enthält egui-UI-Komponenten (Menüs, Statusbar, Input-Handling, Dialoge). Interaktionen emittieren primär `AppIntent`s; direkte Mutation von Fachzustand wird vermieden.

## Module

- `menu.rs` — Top-Menü-Leiste
- `status.rs` — Statusleiste
- `toolbar.rs` — Werkzeugleiste
- `properties.rs` — Properties-Panel (Detailanzeige selektierter Nodes)
- `options_dialog.rs` — Optionen-Dialog für Laufzeit-Einstellungen
- `tool_preview.rs` — Tool-Preview-Overlay (Route-Tool-Vorschau im Viewport)
- `input.rs` — Viewport-Input-Orchestrator (delegiert an Sub-Module)
  - `keyboard.rs` — Tastatur-Shortcuts (Delete, Escape, Ctrl+A)
  - `drag.rs` — Drag-Operationen (Pan, Move, Rechteck-/Lasso-Selektion)
  - `context_menu.rs` — Rechtsklick-Kontextmenü
- `dialogs.rs` — Datei-Dialoge und modale Fenster

## Funktionen

### `render_menu`

Rendert die Top-Menü-Leiste und gibt gesammelte Intents zurück.

```rust
pub fn render_menu(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent>
```

**Menü-Struktur:**
- **File**
  - Open... → `AppIntent::OpenFileRequested`
  - Save (nur wenn Datei geladen) → `AppIntent::SaveRequested`
  - Save As... (nur wenn Datei geladen) → `AppIntent::SaveAsRequested`
  - Select/Change Heightmap... → `AppIntent::HeightmapSelectionRequested`
  - Clear Heightmap (nur wenn gesetzt) → `AppIntent::HeightmapCleared`
  - Exit → `AppIntent::ExitRequested`

- **View**
  - Reset Camera → `AppIntent::ResetCameraRequested`
  - Zoom In → `AppIntent::ZoomInRequested`
  - Zoom Out → `AppIntent::ZoomOutRequested`
  - Hintergrund laden/ändern → `AppIntent::BackgroundMapSelectionRequested`
  - Render Quality → Submenu (Low/Medium/High) → `AppIntent::RenderQualityChanged`
  - Options... → `AppIntent::OpenOptionsDialogRequested`

- **Help**
  - About → Loggt Version

---

### `render_toolbar`

Rendert die Werkzeugleiste (Select, Connect, AddNode) und gibt gesammelte Intents zurück.

```rust
pub fn render_toolbar(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent>
```

---

### `render_properties_panel`

Rendert das Properties-Panel mit Detailanzeige selektierter Nodes (IDs, Positionen, Verbindungen).

```rust
pub fn render_properties_panel(
  ctx: &egui::Context,
  road_map: Option<&RoadMap>,
  selected_node_ids: &[u64],
  default_direction: ConnectionDirection,
  default_priority: ConnectionPriority,
  active_tool: EditorTool,
  tool_manager: Option<&mut ToolManager>,
) -> Vec<AppIntent>
```

---

### `render_status_bar`

Rendert die untere Statusleiste (read-only).

```rust
pub fn render_status_bar(ctx: &egui::Context, state: &AppState)
```

**Angezeigte Informationen:**
- Node-Count, Connection-Count, Marker-Count
- Map-Name (falls vorhanden)
- Zoom und Kamera-Position
- Heightmap-Status (Dateiname oder "None")
- Selektierte Nodes (Anzahl + Beispiel-ID)
- FPS (rechts-aligned)

---

### `InputState`

Orchestrator für Viewport-Input. Delegiert die eigentliche Logik an Sub-Module (`keyboard`, `drag`, `context_menu`).

```rust
pub struct InputState { /* intern */ }
```

**Methoden:**

```rust
let mut input = InputState::new();

// Drag-Targets vom aktiven Route-Tool berechnen
let drag_targets = tool_manager.active_tool()
    .map(|t| t.drag_targets())
    .unwrap_or_default();

// Sammelt Viewport-Events aus egui-Input
let intents = input.collect_viewport_events(
    ui, &response, viewport_size,
    &camera, road_map.as_deref(), &selected_node_ids,
    active_tool, &options, &drag_targets,
);
```

**Sub-Module:**

- **`keyboard`:** Verarbeitet Tastatur-Shortcuts
  - `Delete` → Node(s) löschen
  - `Escape` → Selektion aufheben
  - `Ctrl+A` → Alle selektieren
  - `Ctrl+O` → Datei öffnen
  - `Ctrl+S` → Datei speichern
  - `Ctrl+Z` → Undo
  - `Ctrl+Y` → Redo

- **`drag`:** Verarbeitet Drag-Operationen
  - Links-Drag → Kamera-Pan oder Selektion-Move
  - Links-Drag nahe Route-Tool-Punkt → Steuerpunkt-Drag (`RouteToolDragStarted/Updated/Ended`)
  - Shift+Drag → Rechteck-Selektion
  - Shift+Alt+Drag → Lasso-Selektion
  - Mittel/Rechts-Drag → Kamera-Pan

- **`context_menu`:** Rechtsklick-Kontextmenü auf Nodes/Connections

**Unterstützte Interaktionen (gesamt):**
- **Linksklick:** Node-Pick (mit Shift: additiv + Pfad-Erweiterung)
- **Doppelklick:** Segment-Selektion zwischen Kreuzungen
- **Links-Drag:** Kamera-Pan, Selektion-Move, oder Route-Tool-Steuerpunkt-Drag
- **Shift+Drag:** Rechteck-Selektion
- **Shift+Alt+Drag:** Lasso-Selektion
- **Mittel/Rechts-Drag:** Kamera-Pan
- **Scroll:** Zoom
- **Rechtsklick:** Kontextmenü

---

### `handle_file_dialogs`

Verarbeitet ausstehende Datei-Dialoge (Open, Save, Heightmap).

```rust
pub fn handle_file_dialogs(ui_state: &mut UiState) -> Vec<AppIntent>
```

---

### `show_heightmap_warning`

Zeigt die Heightmap-Warnung als modales Fenster.

```rust
pub fn show_heightmap_warning(ctx: &egui::Context, show: bool) -> Vec<AppIntent>
```

---

### `show_options_dialog`

Zeigt den Optionen-Dialog als modales Fenster (Farben, Größen, Zoom-Schritte).

```rust
pub fn show_options_dialog(
  ctx: &egui::Context,
  show: bool,
  options: &EditorOptions,
) -> Vec<AppIntent>
```

---

### `render_tool_preview`

Zeichnet das Tool-Preview-Overlay in den Viewport (Verbindungen als Linien, Nodes als Kreise/Rauten, halbtransparent).

```rust
pub fn render_tool_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    tool_manager: &ToolManager,
    road_map: &RoadMap,
    cursor_world: Vec2,
)
```

---

### `show_marker_dialog`

Zeigt den Marker-Bearbeiten-Dialog als modales Fenster (Name, Gruppe, bestehende Gruppen).

```rust
pub fn show_marker_dialog(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    road_map: Option<&RoadMap>,
) -> Vec<AppIntent>
```

**Layout:**
```
[Nodes: 42 | Connections: 38 | Markers: 3 | Map: Example | Zoom: 1.50x | Pos: (10.0, 20.0) | Heightmap: terrain.png | Selected Node: 5    FPS: 60]
```

## Design-Prinzipien

1. **Intent-based:** Interaktions-Funktionen liefern `Vec<AppIntent>`
2. **Read-only:** Statusbar zeigt nur State an
3. **State-Zugriff:** Fachzustand wird nicht direkt mutiert; Dialog-/UI-Lifecycle kann `UiState` lokal aktualisieren
4. **Import-Regel:** UI importiert nur aus `app` und `shared` (nie direkt aus `core`)
5. **Sub-Modul-Delegation:** `input.rs` orchestriert, Logik steckt in `keyboard`, `drag`, `context_menu`
