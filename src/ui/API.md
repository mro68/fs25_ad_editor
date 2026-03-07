# UI API Documentation

## Überblick

Das `ui`-Modul enthält egui-UI-Komponenten (Menüs, Statusbar, Input-Handling, Dialoge). Interaktionen emittieren primär `AppIntent`s; direkte Mutation von Fachzustand wird vermieden.

## Module

- `menu.rs` — Top-Menü-Leiste
- `status.rs` — Statusleiste
- `toolbar.rs` — Werkzeugleiste
- `command_palette.rs` — Command Palette Overlay (Suche + Intent-Auswahl)
- `properties.rs` — Properties-Panel (Detailanzeige selektierter Nodes)
- `options_dialog/` — Optionen-Dialog für Laufzeit-Einstellungen (`mod.rs`, `sections.rs`)
- `tool_preview.rs` — Tool-Preview-Overlay (Route-Tool-Vorschau im Viewport)
- `input/` — Viewport-Input-Orchestrator (phasenbasierte Submodule)
  - `clicks.rs` — Klick-Events (Einfach-/Doppel-Klick, Tool-Routing)
  - `drag_primary.rs` — Drag-Start/-Ende (Selektion-Move, Kamera-Pan, Route-Tool-Drag)
  - `pointer_delta.rs` — Pan/Move-Deltas während aktiver Drags
  - `zoom.rs` — Scroll-Zoom auf Mausposition
  - `keyboard.rs` — Tastatur-Shortcuts (Delete, Escape, Ctrl+A) [Peer-Modul]
  - `drag.rs` — Drag-Selektion-Overlay und DragSelection-Typen [Peer-Modul]
  - `context_menu/` — Rechtsklick-Kontextmenü mit validiertem Command-System
    - `commands/mod.rs` — CommandId, Precondition, MenuCatalog, validate_entries()
    - `commands/catalogs.rs` — Katalog-Konstruktoren: `for_empty_area()`, `for_node_focused()`, `for_selection_only()`, `for_route_tool()`
    - `commands/validation.rs` — Precondition-Auswertung und Separator-Cleanup
    - `commands/preconditions.rs` — Precondition-Enum und is_valid()-Logik
    - `commands/tests.rs` — Unit-Tests für Kataloge, Preconditions und Intent-Mapping
    - `mod.rs` — MenuVariant (`EmptyArea`, `NodeFocused`, `SelectionOnly`, `RouteTool`), `determine_menu_variant()`, `render_context_menu()`
    - `icons.rs` — `command_icon()`, Farbhilfs-Funktionen (pub(super))
    - `render.rs` — `render_validated_entries()` und weitere Rendern-Helfer (pub(super))
    - `tangent_ui.rs` — `render_tangent_selection()`, `render_node_info_submenu()` (pub(super))
- `dialogs/` — Datei-Dialoge und modale Fenster
  - `file_dialogs.rs` — Open/Save-Dateidialoge
  - `heightmap_warning.rs` — Heightmap-Warnung vor dem Speichern
  - `marker_dialog.rs` — Marker erstellen/bearbeiten
  - `dedup_dialog.rs` — Duplikat-Bestätigungsdialog
  - `zip_browser.rs` — ZIP-Browser für Background-Map-Auswahl
  - `post_load_dialog.rs` — Post-Load-Dialog (Auto-Erkennung von Heightmap/ZIP/Overview)
  - `save_overview_dialog.rs` — Dialog: Hintergrundbild als overview.jpg speichern
- `segment_overlay.rs` — Segment-Rahmen und Lock-Icons als egui-Overlay (`SegmentOverlayEvent`, `render_segment_overlays()`)

## Funktionen
---

### `SegmentOverlayEvent`

Event, den das Segment-Overlay beim Klick auf ein Lock-Icon ausloest.

```rust
pub enum SegmentOverlayEvent {
  /// Der Lock-Zustand des Segments soll umgeschaltet werden.
  LockToggled { segment_id: u64 },
  /// Das Segment soll aufgeloest werden (nur Segment-Record entfernen).
  Dissolved { segment_id: u64 },
}
```

Wird von `render_segment_overlays()` zurueckgegeben und in den Intent-Flow als
`AppIntent::ToggleSegmentLockRequested { segment_id }` bzw.
`AppIntent::DissolveSegmentRequested { segment_id }` uebersetzt.

---

### `render_segment_overlays`

Zeichnet Segment-Rahmen (AABB) und Lock-Icons als egui-Overlay ueber den Viewport.
```rust
pub fn render_segment_overlays(
  painter: &egui::Painter,
  rect: egui::Rect,
  camera: &Camera2D,
  viewport_size: Vec2,
  registry: &SegmentRegistry,
  road_map: &RoadMap,
  selected_node_ids: &IndexSet<u64>,
  clicked_pos: Option<egui::Pos2>,
  ctrl_held: bool,
  icon_size_px: f32,
) -> Vec<SegmentOverlayEvent>
```

**Verhalten:**

- Iteriert ueber selektierte Nodes und dedupliziert Segment-IDs
- Zeichnet pro Segment ein Lock-Icon ueber dem ersten selektierten Node
- `Ctrl` + Klick auf das Icon erzeugt `SegmentOverlayEvent::Dissolved`
- Die Icon-Größe entspricht `segment_lock_icon_size_px` in `EditorOptions`
- Normaler Klick auf das Icon erzeugt `SegmentOverlayEvent::LockToggled`

**Lock-Zustand:**

- Entsperrt (`locked = false`): grauer Rahmen, offenes Schloss-Icon
- Gesperrt (`locked = true`): gelber Rahmen, 15%-schwarze Fuellung, geschlossenes Schloss-Icon
---

- Select / Connect / AddNode / Route (immer sichtbar)
- Linien-Tool-Dropdown: Zeigt alle Route-Tools ausser FieldBoundaryTool

### `render_properties_panel`

Rendert das Properties-Panel mit Detailanzeige selektierter Nodes (IDs, Positionen, Verbindungen).

Zeigt tool- und selektionsabhängig:

- Distanzen-Panel (wenn ≥ 2 Nodes selektiert): Catmull-Rom-Resample (→ `ResamplePathRequested`)
- Route-Tool-Konfiguration (wenn `active_tool == EditorTool::Route`)
- Standard-Richtung und Straßenart-Selector

**Hinweis:** Node-Verhalten-Einstellungen (reconnect_on_delete, split_connection_on_place) sind jetzt in `render_options_dialog()` integriert.

```rust
pub fn render_properties_panel(
  ctx: &egui::Context,
  road_map: Option<&RoadMap>,
  selected_node_ids: &IndexSet<u64>,
  default_direction: ConnectionDirection,
  default_priority: ConnectionPriority,
  distance_wheel_step_m: f32,
  segment_registry: Option<&SegmentRegistry>,
  distance_state: &mut DistanzenState,
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

### `render_edit_panel`

Rendert das schwebende Edit-Panel für aktive Modi (Streckenteilung, Route-Tool)
und gibt erzeugte Intents zurück.

```rust
pub fn render_edit_panel(
  ctx: &egui::Context,
  road_map: Option<&RoadMap>,
  selected_node_ids: &IndexSet<u64>,
  distanzen_state: &mut DistanzenState,
  default_direction: ConnectionDirection,
  default_priority: ConnectionPriority,
  distance_wheel_step_m: f32,
  active_tool: EditorTool,
  tool_manager: Option<&mut ToolManager>,
  panel_pos: Option<egui::Pos2>,
) -> Vec<AppIntent>
```

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
    active_tool, route_tool_is_drawing,
  &options, command_palette_open, default_direction, default_priority,
  &drag_targets, &mut distanzen_state, tangent_data,
);
```

**Sub-Module:**

- **`keyboard`:** Verarbeitet Tastatur-Shortcuts
  - `Delete` → Node(s) löschen
  - `Escape` → Selektion aufheben
  - `Ctrl+A` → Alle selektieren
  - `Ctrl+C` → Selektion kopieren
  - `Ctrl+V` → Paste-Vorschau starten
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

- **`context_menu`:** Rechtsklick-Kontextmenü mit validiertem Command-System (CommandId + Preconditions → nur gültige Einträge). SVG-Icons werden aus `assets/` gerendert und über `EditorOptions` sowie die aktuell gewählte Standard-Richtung/-Priorität eingefärbt. Streckenteilung-Widget wird nur angezeigt wenn `RoadMap::is_resampleable_chain()` für die aktuelle Selektion `true` liefert (zusammenhängende Kette, Kreuzungen nur an Endpunkten).
  - **Segment-Integration:** `segment_registry` wird zur Validierung herangezogen. Wenn alle selektierten Nodes zu einem einzigen validen Segment gehoeren → `EditSegment` Command verfuegbar.

### `render_context_menu`

Rendert das Kontextmenü für einen Viewport-Rechtsklick. Enthaelt validierte Command-Kataloge und Intent-Generierung.

```rust
pub fn render_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    distanzen_active: bool,
    clipboard_has_data: bool,
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    variant: &MenuVariant,
    segment_registry: Option<&SegmentRegistry>,
    events: &mut Vec<AppIntent>,
) -> bool
```

Alle Commands werden durch ein Precondition-System gefiltert: Nur Commands deren Bedingungen erfuellt sind werden angezeigt.

**Segment-Integration:**

- `segment_registry` wird zur Validierung herangezogen
- Prueft ob alle selektierten Nodes zu einem einzigen validen Segment gehoeren
- Wenn ja → `EditSegment` Command verfuegbar im Katalog
- Segment-Validierung: Nodes existieren und Originalpositionen unveraendert

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

### `render_command_palette`

Rendert die Command Palette als zentriertes Overlay-Fenster mit Substring-Suche.

```rust
pub fn render_command_palette(
  ctx: &egui::Context,
  show: &mut bool,
  tool_manager: Option<&ToolManager>,
) -> Vec<AppIntent>
```

**Verhalten:**

- Suchfeld mit Auto-Focus beim Oeffnen
- Filterung ueber `entry.label.contains(search)` (case-insensitive)
- Tastatur: Pfeil hoch/runter, Enter (ausfuehren), Escape (schliessen)
- Klick ausserhalb schliesst die Palette
- Katalog: statische Befehle + dynamische Route-Tools (`SelectRouteToolRequested { index }`)

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
    ctx: &ToolPreviewContext<'_>
)
```

### `paint_clipboard_preview`

Zeichnet die Copy/Paste-Vorschau (kopierte Nodes + interne Verbindungen)
halbtransparent an der aktuellen Paste-Position.

```rust
pub fn paint_clipboard_preview(
  painter: &egui::Painter,
  rect: egui::Rect,
  camera: &Camera2D,
  viewport_size: Vec2,
  clipboard: &Clipboard,
  paste_pos: Vec2,
  opacity: f32,
)
```

---

### `paint_preview` und `paint_preview_polyline`

Zeichnen ein Preview als Overlay im Viewport.

```rust
pub fn paint_preview(
  painter: &egui::Painter,
  rect: egui::Rect,
  camera: &Camera2D,
  viewport_size: Vec2,
  preview: &ToolPreview,
  options: &EditorOptions,
)

pub fn paint_preview_polyline(
  painter: &egui::Painter,
  rect: egui::Rect,
  camera: &Camera2D,
  viewport_size: Vec2,
  positions: &[Vec2],
)

### `render_route_defaults_panel`

Linkes Panel fuer Standard-Richtung und -Prioritaet (Icon-Only, vertikal gestapelt).

```rust
pub fn render_route_defaults_panel(
  ctx: &egui::Context,
  default_direction: ConnectionDirection,
  default_priority: ConnectionPriority,
) -> Vec<AppIntent>
```

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

---

### `show_dedup_dialog`

Zeigt den Duplikat-Bereinigungsdialog als modales Fenster. Erscheint nach dem Laden einer XML-Datei, wenn duplizierte Nodes erkannt wurden. Der Benutzer kann die Bereinigung bestätigen oder abbrechen.

```rust
pub fn show_dedup_dialog(ctx: &egui::Context, ui_state: &UiState) -> Vec<AppIntent>
```

**Emittierte Intents:**

- `AppIntent::DeduplicateConfirmed` — Benutzer bestätigt Bereinigung
- `AppIntent::DeduplicateCancelled` — Benutzer bricht ab

**Layout:**

```
[Titel: "Duplizierte Wegpunkte erkannt"]
  ⚠ AutoDrive hat Teile des Netzwerks mehrfach erstellt.
  Gefunden: N duplizierte Nodes in M Positions-Gruppen
  [Bereinigen]  [Ignorieren]
```

---

### `show_zip_browser`

Zeigt den ZIP-Browser-Dialog zur Auswahl einer Bilddatei aus einem ZIP-Archiv. Erscheint wenn eine `.zip`-Datei als Background-Map gewählt wurde und mehrere Bilddateien enthält. Bei genau einem Bild im ZIP wird automatisch geladen (kein Dialog).

```rust
pub fn show_zip_browser(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent>
```

**Emittierte Intents:**

- `AppIntent::ZipBackgroundFileSelected { zip_path, entry_name }` — Bild aus ZIP gewählt (Doppelklick oder Übernehmen-Button)
- `AppIntent::ZipBrowserCancelled` — Abbrechen oder X-Button

**Layout:**

```
[Titel: "Bild aus ZIP wählen"]
  N Bilddateien gefunden:
  ┌─────────────────────────┐
  │  maps/overview.dds      │  ← scrollbar, selectable
  │  maps/detail.png        │
  └─────────────────────────┘
  [Übernehmen]  [Abbrechen]
```

---

### `show_post_load_dialog`

Zeigt den Post-Load-Dialog nach dem Laden einer XML-Datei. Informiert über automatisch erkannte Heightmap und bietet die Möglichkeit, eine Übersichtskarte aus einem passenden Map-Mod-ZIP zu generieren.

```rust
pub fn show_post_load_dialog(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent>
```

**Emittierte Intents:**

- `AppIntent::PostLoadGenerateOverview { zip_path }` — Benutzer will Übersichtskarte generieren
- `AppIntent::PostLoadDialogDismissed` — Benutzer schließt den Dialog

**Layout:**

```
[Titel: "Nach dem Laden erkannt"]
  ✓ Heightmap automatisch geladen
     terrain.heightmap.png
  Karte: "Höflingen"
  Passender Map-Mod gefunden:
     📦 FS25_Hoeflingen.zip
  [Übersichtskarte generieren]  [Schließen]
```

Bei mehreren ZIPs werden RadioButtons zur Auswahl angezeigt.

---

### `show_overview_options_dialog`

Zeigt den Layer-Dialog für die Übersichtskarten-Generierung (Hillshade/Farmlands/IDs/POIs/Legende).

```rust
pub fn show_overview_options_dialog(
  ctx: &egui::Context,
  state: &mut OverviewOptionsDialogState,
) -> Vec<AppIntent>
```

**Emittierte Intents:**

- `AppIntent::OverviewOptionsConfirmed`
- `AppIntent::OverviewOptionsCancelled`

---

### `show_save_overview_dialog`

Zeigt den Dialog "Hintergrundbild als overview.jpg speichern?" nach dem Laden eines Hintergrundbildes aus einem ZIP-Archiv oder nach Generierung einer Übersichtskarte. Erscheint nur wenn eine XML-Datei geladen ist und noch keine overview.jpg im selben Verzeichnis existiert.

```rust
pub fn show_save_overview_dialog(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent>
```

**Emittierte Intents:**

- `AppIntent::SaveBackgroundAsOverviewConfirmed` — Benutzer bestätigt Speichern
- `AppIntent::SaveBackgroundAsOverviewDismissed` — Benutzer lehnt ab

**Layout:**

```
[Titel: "Hintergrundbild speichern?"]
  Soll das Hintergrundbild als overview.jpg
  im Savegame-Verzeichnis gespeichert werden?
  /pfad/zur/overview.jpg
  Beim nächsten Laden wird es automatisch als Hintergrund verwendet.
  [Ja, speichern]  [Nein]
```

---

---

### `SegmentOverlayEvent`

Event, den das Segment-Overlay beim Klick auf ein Lock-Icon ausloest.

```rust
pub enum SegmentOverlayEvent {
  /// Der Lock-Zustand des Segments soll umgeschaltet werden.
  LockToggled { segment_id: u64 },
  /// Das Segment soll aufgeloest werden (nur Segment-Record entfernen).
  Dissolved { segment_id: u64 },
}
```

Wird von `render_segment_overlays()` zurueckgegeben und in den Intent-Flow als
`AppIntent::ToggleSegmentLockRequested { segment_id }` bzw.
`AppIntent::DissolveSegmentRequested { segment_id }` uebersetzt.

---

### `render_segment_overlays`

Zeichnet Segment-Rahmen (AABB) und Lock-Icons als egui-Overlay ueber den Viewport.

```rust
pub fn render_segment_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &SegmentRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    clicked_pos: Option<egui::Pos2>,
    ctrl_held: bool,
    icon_size_px: f32,
) -> Vec<SegmentOverlayEvent>
```

**Verhalten:**

- Iteriert ueber selektierte Nodes und dedupliziert Segment-IDs
- Zeichnet pro Segment ein Lock-Icon ueber dem ersten selektierten Node
- `Ctrl` + Klick auf das Icon erzeugt `SegmentOverlayEvent::Dissolved`
- Die Icon-Größe basiert auf `EditorOptions::segment_lock_icon_size_px`
- Normaler Klick auf das Icon erzeugt `SegmentOverlayEvent::LockToggled`

**Lock-Zustand:**

- Entsperrt (`locked = false`): grauer Rahmen, offenes Schloss-Icon
- Gesperrt (`locked = true`): gelber Rahmen, 15%-schwarze Fuellung, geschlossenes Schloss-Icon

---

## Design-Prinzipien

1. **Intent-based:** Interaktions-Funktionen liefern `Vec<AppIntent>`
2. **Read-only:** Statusbar zeigt nur State an
3. **State-Zugriff:** Fachzustand wird nicht direkt mutiert; Dialog-/UI-Lifecycle kann `UiState` lokal aktualisieren
4. **Import-Regel:** UI importiert nur aus `app` und `shared` (nie direkt aus `core`)
5. **Sub-Modul-Delegation:** `input.rs` orchestriert, Logik steckt in `keyboard`, `drag`, `context_menu`
