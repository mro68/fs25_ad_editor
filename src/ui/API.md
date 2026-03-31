# UI API Documentation

## Überblick

Das `ui`-Modul enthält egui-UI-Komponenten (Menüs, Statusbar, Input-Handling, Dialoge). Interaktionen emittieren primär `AppIntent`s; direkte Mutation von Fachzustand wird vermieden.

## Module

- `common.rs` — Gemeinsame UI-Hilfsfunktionen (Mausrad-Scroll-Helfer)
- `menu.rs` — Top-Menü-Leiste
- `status.rs` — Statusleiste
- `floating_menu.rs` — Schwebende Kontextmenues fuer Werkzeuggruppen (Toggle via `T/G/B/R/Z`)
- `icons.rs` — Gemeinsame Icon-Konstanten/Helfer (`ICON_SIZE`, `svg_icon`, `route_tool_icon`)
- `long_press.rs` — Wiederverwendbares Long-Press-Dropdown-Widget (`LongPressState`, `LongPressGroup`, `render_long_press_button`)
- `defaults_panel.rs` — Linke Sidebar im Gruppen-Layout (Long-Press fuer Werkzeuge/Route-Gruppen/Defaults, Hintergrund)
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
  - `confirm_dissolve_dialog.rs` — Bestätigungsdialog vor dem Auflösen einer Segment-Gruppe
- `group_overlay.rs` — Segment-Rahmen und Lock-Icons als egui-Overlay (`GroupOverlayEvent`, `render_group_overlays()`)
- `group_boundary_overlay.rs` — Boundary-Icons (Eingang/Ausgang/Bidirektional) ueber Nodes mit externen Verbindungen (`GroupBoundaryIcons`, `render_group_boundary_overlays()`)
- `drag.rs` — Drag-Selektion-Overlay und `DragSelection`-Typen

### `DragSelectionMode`

```rust
pub(crate) enum DragSelectionMode {
    Rect,       // Rechteck-Selektion (Standard-Drag)
    Lasso,      // Freihand-Lasso fuer Node-Selektion
    ToolLasso,  // Freihand-Lasso fuer das aktive Route-Tool (z.B. ColorPathTool)
}
```

`ToolLasso` unterscheidet sich von `Lasso` dadurch, dass das abgeschlossene Polygon
nicht zur Node-Selektion verwendet, sondern per `AppIntent::RouteToolLassoCompleted`
an das aktive Route-Tool weitergeleitet wird. Den Modus schaltet
`drag_primary.rs` automatisch ein wenn `ViewportContext.tool_needs_lasso == true`.

### `ViewportContext.tool_needs_lasso`

```rust
pub(crate) struct ViewportContext<'a> {
    // ...
    /// Gibt an, ob das aktive Route-Tool Alt+Drag als Lasso-Eingabe benoetigt.
    pub tool_needs_lasso: bool,
}
```

Wird von `input/mod.rs` befuellt: `tool_needs_lasso = active_tool.needs_lasso_input()`.
Ist `true`, behandelt `drag_primary.rs` einen Alt+Drag als `DragSelectionMode::ToolLasso`
statt als normale Lasso-Selektion.

## Funktionen
---

### `show_confirm_dissolve_dialog`

Zeigt einen modalen Bestätigungsdialog zum Auflösen einer Segment-Gruppe. Wird aufgerufen wenn `UiState::confirm_dissolve_segment_id` gesetzt ist.

```rust
pub fn show_confirm_dissolve_dialog(
    ctx: &egui::Context,
    confirm_dissolve_id: &mut Option<u64>,
    language: Language,
) -> Vec<AppIntent>
```

**Verhalten:**

- Ist `confirm_dissolve_id` `None`, tut die Funktion nichts und gibt einen leeren Vec zurück
- Zeigt ein zentriertes, nicht minimierbares `egui::Window` mit Titel `ConfirmDissolveTitle`
- Klick auf `ConfirmDissolveOk` → emittiert `AppIntent::DissolveSegmentConfirmed { segment_id }` und setzt `confirm_dissolve_id = None`
- Klick auf `ConfirmDissolveCancel` oder Schließen des Fensters → setzt `confirm_dissolve_id = None` ohne Aktion
- Texte werden über `t(language, I18nKey::ConfirmDissolveXxx)` übersetzt

**Intent-Flow:**
```
Ctrl+Lock-Icon-Klick
  → DissolveSegmentRequested
  → OpenDissolveConfirmDialog  (via intent_mapping)
  → UiState::confirm_dissolve_segment_id = Some(id)
  → [nächster Frame] show_confirm_dissolve_dialog() zeigt Dialog
  → DissolveSegmentConfirmed  (bei Bestätigung)
  → DissolveSegment  (via intent_mapping)
  → handlers::group::dissolve()
```

---

### `GroupOverlayEvent`

Event, den das Segment-Overlay beim Klick auf ein Lock-Icon ausloest.

```rust
pub enum GroupOverlayEvent {
  /// Der Lock-Zustand des Segments soll umgeschaltet werden.
  LockToggled { segment_id: u64 },
  /// Das Segment soll aufgeloest werden (nur Gruppen-Record entfernen).
  Dissolved { segment_id: u64 },
}
```

Wird von `render_group_overlays()` zurueckgegeben und in den Intent-Flow als
`AppIntent::ToggleSegmentLockRequested { segment_id }` bzw.
`AppIntent::DissolveSegmentRequested { segment_id }` uebersetzt.

---

### `render_group_overlays`

Zeichnet Segment-Rahmen (AABB) und Lock-Icons als egui-Overlay ueber den Viewport.
```rust
pub fn render_group_overlays(
  painter: &egui::Painter,
  rect: egui::Rect,
  camera: &Camera2D,
  viewport_size: Vec2,
  registry: &GroupRegistry,
  road_map: &RoadMap,
  selected_node_ids: &IndexSet<u64>,
  clicked_pos: Option<egui::Pos2>,
  ctrl_held: bool,
  icon_size_px: f32,
) -> Vec<GroupOverlayEvent>
```

**Verhalten:**

- Iteriert ueber selektierte Nodes und dedupliziert Segment-IDs
- Zeichnet pro Segment ein Lock-Icon ueber dem ersten selektierten Node
- `Ctrl` + Klick auf das Icon erzeugt `GroupOverlayEvent::Dissolved`
- Die Icon-Größe entspricht `segment_lock_icon_size_px` in `EditorOptions`
- Normaler Klick auf das Icon erzeugt `GroupOverlayEvent::LockToggled`

**Lock-Zustand:**

- Entsperrt (`locked = false`): grauer Rahmen, offenes Schloss-Icon
- Gesperrt (`locked = true`): gelber Rahmen, 15%-schwarze Fuellung, geschlossenes Schloss-Icon
---

### `GroupBoundaryIcons`

Gecachte egui-Textur-Handles fuer die drei Boundary-Icon-Typen (Eingang, Ausgang, Bidirektional).
Die Icons werden per SVG (usvg/resvg) als 32×32-RGBA-Texturen in egui geladen.

```rust
pub struct GroupBoundaryIcons {
    pub entry: TextureHandle,        // Icon fuer Eingang-Nodes
    pub exit: TextureHandle,         // Icon fuer Ausgang-Nodes
    pub bidirectional: TextureHandle, // Icon fuer bidirektionale Nodes
}
```

**Methoden:**

```rust
pub fn load(ctx: &egui::Context) -> Self
```

- Laedt und rasterisiert die drei SVG-Assets aus `assets/icons/group_entry.svg`, `group_exit.svg`, `group_bidirectional.svg`.
- Soll einmal pro App-Lifetime aufgerufen werden (Lazy-Init beim ersten `update()`).

---

### `render_group_boundary_overlays`

Zeichnet Boundary-Icons unterhalb der Nodes aller selektierten Gruppen.

Icons werden **ausschliesslich** gerendert wenn `record.entry_node_id` bzw. `record.exit_node_id` explizit gesetzt sind — keine automatische Zuweisung mehr. Bei `show_all=true` werden zusaetzlich alle gecachten `BoundaryInfo`-Eintraege aus dem Connection-Analyse-Cache gerendert (Debug-Ansicht).

Iteriert ueber ALLE Gruppen selektierter Nodes via `find_by_node_ids()` (nicht nur die erste).

```rust
pub fn render_group_boundary_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &GroupRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    icons: &GroupBoundaryIcons,
    icon_size_px: f32,
    show_all: bool,
)
```

**Parameter:**
- `painter` — egui-Painter fuer den Viewport
- `rect` — Viewport-Rechteck in Screen-Koordinaten
- `camera` — Kamera fuer Welt→Screen-Transformation
- `viewport_size` — Viewport-Abmessungen in Pixeln
- `registry` — Gruppen-Registry mit gecachten BoundaryInfos
- `road_map` — RoadMap fuer Node-Positionen
- `selected_node_ids` — Aktuell selektierte Node-IDs
- `icons` — Gecachte Textur-Handles (via `GroupBoundaryIcons::load()`)
- `icon_size_px` — Icon-Groesse in Pixeln (Minimum: 8 px)
- `show_all` — `false` = nur Nodes mit explizit gesetzter Entry/Exit-ID; `true` = zusaetzlich alle BoundaryInfos aus dem Connection-Cache (Debug-Ansicht)

**Icon-Zuordnung:**
- `BoundaryDirection::Bidirectional` → Bidirektional-Icon
- `BoundaryDirection::Entry` → Eingang-Icon
- `BoundaryDirection::Exit` → Ausgang-Icon

**Voraussetzung:** `registry.warm_boundary_cache(road_map)` muss vor dem Aufruf erfolgt sein.

---

### `render_floating_menu`

Rendert ein schwebendes Kontextmenue an `UiState.floating_menu.pos`.
Die Menue-Art wird ueber `UiState.floating_menu.kind` gesteuert.

```rust
pub fn render_floating_menu(
  ctx: &egui::Context,
  state: &AppState,
) -> (Vec<AppIntent>, bool)
```

Unterstuetzte Menues:

- `FloatingMenuKind::Tools` — Select / Connect / AddNode
- `FloatingMenuKind::Basics` — Gerade, Bezier (Q/C), Spline, Constraint
- `FloatingMenuKind::SectionTools` — Ausweichstrecke, Parkplatz, Strecke versetzen
- `FloatingMenuKind::DirectionPriority` — Verbindungsrichtung (Regular/Dual/Reverse) und Strassenart (Haupt-/Nebenstrasse)
- `FloatingMenuKind::Zoom` — Auf komplette Map einpassen, Auf Auswahl einpassen

Verhalten:

- Aktive Auswahl wird mit Akzentfarbe hervorgehoben
- Item-Klick emittiert passende Intents und schliesst das Menue
- Klick ausserhalb schliesst das Menue

---

### `ui::icons`

Gemeinsame UI-Icon-Helfer fuer Tool-Buttons.

```rust
pub const ICON_SIZE: f32;
pub fn svg_icon(source: ImageSource<'_>, size: f32) -> Image<'_>;
pub fn route_tool_icon(idx: usize) -> ImageSource<'static>;
```

---

### `render_properties_panel`

Rendert das Properties-Panel mit Detailanzeige selektierter Nodes (IDs, Positionen, Verbindungen).

Zeigt tool- und selektionsabhängig:

- Distanzen-Panel (wenn ≥ 2 Nodes selektiert): Catmull-Rom-Resample (→ `ResamplePathRequested`)
- Standard-Richtung und Straßenart-Selector
- **Flag-Editor** (Einzelnode-Selektion): ComboBox für `Regular` / `SubPrio` (→ `NodeFlagChangeRequested`)
- **Connection-Listing** (Einzelnode-Selektion): eingehende und ausgehende Verbindungen mit Richtungsanzeige

**Hinweis:** Node-Verhalten-Einstellungen (reconnect_on_delete, split_connection_on_place) sind in `render_options_dialog()` integriert. Route-Tool-Konfiguration wird separat vom `render_edit_panel()` gerendert (DRY-Bereinigung).

```rust
pub fn render_properties_panel(
  ctx: &egui::Context,
  road_map: Option<&RoadMap>,
  selected_node_ids: &IndexSet<u64>,
  default_direction: ConnectionDirection,
  default_priority: ConnectionPriority,
  distance_wheel_step_m: f32,
  group_registry: Option<&GroupRegistry>,
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

Rendert das schwebende Edit-Panel für aktive Modi (Gruppen-Edit, Streckenteilung, Route-Tool)
und gibt erzeugte Intents zurück. Bei aktivem `group_editing` wird ein Gruppen-Edit-Panel
(Übernehmen/Abbrechen + Checkbox + Entry/Exit-ComboBoxen) angezeigt und die anderen Modi unterdrückt.

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
  group_editing: Option<&GroupEditState>,
  group_record: Option<&GroupRecord>,
  options: &mut EditorOptions,
) -> Vec<AppIntent>
```

Im Gruppen-Bearbeitungsmodus enthält das Panel:
- Checkbox für `options.show_all_group_boundaries` (Sichtbarkeit aller Boundary-Icons)
- ComboBox „Einfahrt" und „Ausfahrt" — emittiert `AppIntent::SetGroupBoundaryNodes` bei Änderung
- Button „🔧 Tool bearbeiten" — nur sichtbar wenn `group_record.kind.tool_index().is_some()` (nicht fuer `Manual`-Gruppen); emittiert `AppIntent::GroupEditToolRequested { record_id }` → wechselt atomar in den destruktiven Tool-Edit-Modus

---

### `InputState`

Orchestrator für Viewport-Input. Delegiert die eigentliche Logik an Sub-Module (`keyboard`, `drag`, `context_menu`).

```rust
pub struct InputState {
    /// Zeigt an, ob gerade eine Gruppen-Rotation per Alt+Mausrad läuft.
    /// Steuert korrekte Begin/End-Lifecycle-Intent-Emission in `zoom.rs`.
    pub(crate) rotation_active: bool,
    /* weitere Felder intern */
}
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
  - `T` (ohne Modifier) → Floating-Menu Tools
  - `G` (ohne Modifier) → Floating-Menu Grundbefehle (Basics)
  - `B` (ohne Modifier) → Floating-Menu Abschnittswerkzeuge (SectionTools)
  - `R` (ohne Modifier) → Floating-Menu Richtung & Strassenart (DirectionPriority)
  - `Z` (ohne Modifier) → Floating-Menu Zoom
  - `K` (ohne Modifier) und `Ctrl+K` → Command-Palette toggeln
  - `Ctrl+Z` → Undo
  - `Ctrl+Y` → Redo

- **`drag`:** Verarbeitet Drag-Operationen
  - Links-Drag → Kamera-Pan oder Selektion-Move
  - Links-Drag nahe Route-Tool-Punkt → Steuerpunkt-Drag (`RouteToolDragStarted/Updated/Ended`)
  - Shift+Drag → Rechteck-Selektion
  - Shift+Alt+Drag → Lasso-Selektion
  - Mittel/Rechts-Drag → Kamera-Pan

- **`context_menu`:** Rechtsklick-Kontextmenü mit validiertem Command-System (CommandId + Preconditions → nur gültige Einträge). SVG-Icons werden aus `assets/` gerendert und über `EditorOptions` sowie die aktuell gewählte Standard-Richtung/-Priorität eingefärbt. Streckenteilung-Widget wird nur angezeigt wenn `RoadMap::is_resampleable_chain()` für die aktuelle Selektion `true` liefert (zusammenhängende Kette, Kreuzungen nur an Endpunkten).
  - **Segment-Integration:** `group_registry` wird zur Validierung herangezogen. Wenn alle selektierten Nodes zu einem einzigen validen Segment gehoeren → `EditGroup` Command verfuegbar.

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
    group_registry: Option<&GroupRegistry>,
    events: &mut Vec<AppIntent>,
) -> bool
```

Alle Commands werden durch ein Precondition-System gefiltert: Nur Commands deren Bedingungen erfuellt sind werden angezeigt.

**Segment-Integration:**

- `group_registry` wird zur Validierung herangezogen
- Prueft ob alle selektierten Nodes zu einem einzigen validen Segment gehoeren
- Wenn ja → `EditGroup` Command verfuegbar im Katalog
- Segment-Validierung: Nodes existieren und Originalpositionen unveraendert

**Unterstützte Interaktionen (gesamt):**

- **Linksklick:** Node-Pick (mit Shift: additiv + Pfad-Erweiterung)
- **Doppelklick:** Segment-Selektion zwischen Kreuzungen
- **Links-Drag:** Kamera-Pan, Selektion-Move, oder Route-Tool-Steuerpunkt-Drag
- **Shift+Drag:** Rechteck-Selektion
- **Shift+Alt+Drag:** Lasso-Selektion
- **Mittel/Rechts-Drag:** Kamera-Pan
- **Scroll:** Zoom
- **Alt+Scroll** (Select-Tool + aktive Selektion): Gruppen-Rotation (5° pro Tick, Lifecycle: `BeginRotateSelectedNodesRequested` → `RotateSelectedNodesRequested` → `EndRotateSelectedNodesRequested`)
- **Rechtsklick:** Kontextmenü

---

### `ui::common` — Gemeinsame UI-Hilfsfunktionen

Kleine, wiederverwendbare Helfer fuer egui-Widgets. Werden von mehreren UI-Modulen importiert.

```rust
/// Schwellenwert fuer Scroll-Events – unterdrückt Rauschen bei kleinen Scroll-Bewegungen.
pub(crate) const WHEEL_THRESHOLD: f32 = 0.5;

/// Wendet Mausrad-Scrolling auf einen numerischen Wert an.
///
/// Wenn die Response gehovert ist und ein Scroll-Event vorliegt,
/// wird `value` um `step` in Scroll-Richtung veraendert und auf `range` geclampt.
/// Gibt `true` zurueck wenn sich der Wert geaendert hat.
pub(crate) fn apply_wheel_step(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut f32,
    step: f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool
```

**Verwendung:**

```rust
use crate::ui::common::apply_wheel_step;

let r = ui.add(egui::DragValue::new(&mut opts.node_size_world));
r.changed() | apply_wheel_step(ui, &r, &mut opts.node_size_world, 0.1, 0.1..=5.0);
```

Wird in `options_dialog/sections.rs` fuer alle 25 numerischen Options-Felder verwendet.

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
```

---

### Long-Press-Widget (`long_press.rs`)

Wiederverwendbares Long-Press-Dropdown-Widget fuer Icon-Buttons mit optionalem Auswahl-Popup.

#### Typen

```rust
/// Long-Press-Status einer Button-Gruppe.
pub struct LongPressState {
    pub press_start: Option<f64>,  // Startzeitpunkt (egui-Zeit in Sekunden)
    pub popup_open: bool,          // Ob das Auswahl-Popup offen ist
    pub popup_pos: Option<egui::Pos2>, // Position des Popups im Screen-Space
}

/// Ein auswaehlbares Item innerhalb einer Long-Press-Gruppe.
pub struct LongPressItem<T: Clone> {
    pub icon: egui::ImageSource<'static>,
    pub tooltip: &'static str,
    pub value: T,
}

/// Definiert eine Long-Press-Gruppe mit mehreren auswaehlbaren Items.
pub struct LongPressGroup<T: Clone + PartialEq> {
    pub id: &'static str,       // Eindeutige ID fuer egui
    pub label: &'static str,    // Anzeigename der Gruppe
    pub items: Vec<LongPressItem<T>>,
}
```

#### Funktionen

```rust
/// Rendert einen Long-Press-Button mit optionalem Auswahl-Popup.
/// Kurzer Klick aktiviert das aktuelle Item; Long-Press (>= 1s) oeffnet Popup.
pub fn render_long_press_button<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    group: &LongPressGroup<T>,
    active_value: &T,
    lp_state: &mut LongPressState,
) -> Option<T>

/// Rendert das Long-Press-Popup neben dem Button.
pub fn render_popup<T: Clone + PartialEq>(
    ctx: &egui::Context,
    group: &LongPressGroup<T>,
    active_value: &T,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    lp_state: &mut LongPressState,
) -> Option<T>

/// Zeichnet einen kleinen Dropdown-Pfeil in die untere rechte Ecke des Buttons.
pub fn paint_dropdown_arrow(ui: &egui::Ui, response: &egui::Response)
```

**Verhalten:**
- Return `Some(value)` wenn ein Item ausgewaehlt wurde (kurzer Klick oder Popup-Klick)
- `LongPressState` muss pro Gruppe getrennt im `UiState` gehalten werden (`lp_tools`, `lp_straights`, `lp_curves`, `lp_constraint`, `lp_section_tools`, `lp_direction`, `lp_priority`)
- Popup schliesst sich bei Klick ausserhalb

---

### `render_route_defaults_panel`

Linke Sidebar im kompakten Gruppen-Layout (64px):

- Long-Press-Gruppe `Werkzeuge` (Select, Connect, AddNode)
- Long-Press-Gruppen fuer Route-Tools (Geraden, Kurven, Constraint, Abschnittswerkzeuge; ohne FieldBoundary)
- Long-Press fuer Richtungs- und Prioritaets-Defaults
- `Hintergrund` als `CollapsingHeader`

```rust
pub fn render_route_defaults_panel(
  ctx: &egui::Context,
  state: &mut AppState,
) -> Vec<AppIntent>
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

### `GroupOverlayEvent`

Event, den das Segment-Overlay beim Klick auf ein Lock-Icon ausloest.

```rust
pub enum GroupOverlayEvent {
  /// Der Lock-Zustand des Segments soll umgeschaltet werden.
  LockToggled { segment_id: u64 },
  /// Das Segment soll aufgeloest werden (nur Gruppen-Record entfernen).
  Dissolved { segment_id: u64 },
}
```

Wird von `render_group_overlays()` zurueckgegeben und in den Intent-Flow als
`AppIntent::ToggleSegmentLockRequested { segment_id }` bzw.
`AppIntent::DissolveSegmentRequested { segment_id }` uebersetzt.

---

### `render_group_overlays`

Zeichnet Segment-Rahmen (AABB) und Lock-Icons als egui-Overlay ueber den Viewport.

```rust
pub fn render_group_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &GroupRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    clicked_pos: Option<egui::Pos2>,
    ctrl_held: bool,
    icon_size_px: f32,
) -> Vec<GroupOverlayEvent>
```

**Verhalten:**

- Iteriert ueber selektierte Nodes und dedupliziert Segment-IDs
- Zeichnet pro Segment ein Lock-Icon ueber dem ersten selektierten Node
- `Ctrl` + Klick auf das Icon erzeugt `GroupOverlayEvent::Dissolved`
- Die Icon-Größe basiert auf `EditorOptions::segment_lock_icon_size_px`
- Normaler Klick auf das Icon erzeugt `GroupOverlayEvent::LockToggled`

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
