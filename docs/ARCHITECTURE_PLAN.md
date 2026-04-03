# Architektur-Plan (Soll-Zustand)

Stand: 2026-04-03  
Status: Weitgehend umgesetzt — Render/shared-Vertrag, Tool-UI-Grenze, Capability-Split, Registry/Katalog/Edit-Flow und Shared-Neutralisierung sind technisch entkoppelt; spaetere Remediationsphasen bleiben offen

## Zielbild

Dieser Plan trennt fachliche Verantwortlichkeiten in fuenf Schichten plus ein geteiltes Shared-Modul. Davor sitzt eine duenne Integrationsschale fuer den Binary-Start und eframe/wgpu:

- Integrationsschale (`src/editor_app/*`, `src/runtime.rs`, `src/main.rs`): Start, Frame-Zyklus und Anbindung zwischen UI, Application und Renderer

- UI (`src/ui/*`): Darstellung + Intent-Erzeugung
- Application (`src/app/*`): Event-Verarbeitung + Orchestrierung + Use-Cases
- Domain (`src/core/*`): Datenmodell + Fachlogik
- Persistence (`src/xml/*`): XML-Mapping und I/O
- Rendering (`src/render/*`): GPU-Darstellung aus vorbereiteten Renderdaten
- Shared (`src/shared/*`): Neutrale, schichtuebergreifende Vertraege und Utilities (u. a. RenderScene, RenderQuality, EditorOptions, i18n) ohne egui- oder Dateisystem-Policy

Kernfluss: **Input -> AppIntent -> AppController -> AppCommand -> AppState/Domain -> RenderScene -> Renderer**.

Die Integrationsschale ist bewusst kein zusaetzlicher Fach-Layer. Sie koordiniert `ui`, `app` und `render`, enthaelt aber keine eigenen Use-Cases oder Domain-Logik.

## Systemübersicht

```mermaid
flowchart LR
  subgraph UI[UI Layer (egui)]
    MENU[menu.rs]
    STATUS[status.rs]
    VIEWPORT[Viewport Input]
  end

  subgraph APP[Application Layer]
    EVENTS[AppIntent]
    CMDS[AppCommand]
    CTRL[AppController]
    STATE[AppState]
    SCENEB[RenderSceneBuilder]
  end

  subgraph CORE[Domain Layer]
    ROAD[RoadMap]
    NODE[MapNode]
    CONN[Connection]
    MARKER[MapMarker]
    META[AutoDriveMeta]
  end

  subgraph INFRA[Infrastructure]
    XML[xml::parser / xml::writer]
    RENDER[render::Renderer]
  end

  MENU --> EVENTS
  VIEWPORT --> EVENTS
  EVENTS --> CTRL
  CTRL --> CMDS
  CMDS --> CTRL
  CTRL --> STATE
  CTRL --> CORE
  CORE --> CTRL
  CTRL --> SCENEB
  SCENEB --> RENDER

  XML --> CORE
  CORE --> XML

  STATUS -. read-only .-> STATE
```

### Layer-Grenzen (erlaubte Import-Richtungen)

```mermaid
graph BT
    CORE["core\n(Domain)"]
    XML["xml\n(Persistence)"]
    APP["app\n(Application)"]
    UI["ui\n(Presentation)"]
    RENDER["render\n(Rendering)"]
    SHARED["shared\n(Cross-Layer)"]

    XML  --> CORE
    APP  --> CORE
    APP  --> XML
    APP  --> SHARED
    UI   --> APP
    RENDER --> SHARED

    CORE  -.->|verboten| UI
    CORE  -.->|verboten| RENDER
    APP   -.->|verboten| UI
    APP   -.->|verboten| RENDER
    RENDER -.->|verboten| CORE
    XML   -.->|verboten| APP
```

> **Regel:** Pfeile zeigen "darf importieren". Gestrichelt = explizit verboten (CI-geprüft via `scripts/check_layer_boundaries.sh`).

### Integrationsschale (`src/editor_app/*`, `src/runtime.rs`, `src/main.rs`)

**Verantwortung**

- Startet `EditorApp` aus `runtime.rs` heraus und bindet eframe/wgpu an die Layer-Architektur an
- Sammelt pro Frame Panel-, Dialog-, Viewport- und Overlay-Events als `Vec<AppIntent>`, verarbeitet schalenlokale Events by-value und reicht nur die verbleibenden Intents an `AppController` weiter
- Registriert den Render-Callback und verwaltet nur fensterlokalen Integrationszustand (`render::Renderer`, `ui::InputState`, Cursor-/Icon-Caches)

**Darf**

- `ui`, `app` und `render` koordinieren
- Read-only auf `AppState` zugreifen, wenn Panels oder Overlays Daten benoetigen

**Darf nicht**

- Eigene Fachlogik oder Duplicate-Use-Cases enthalten
- Domain-Daten am `AppController` vorbei mutieren

**API-Hinweis**

- Kanonische Doku: [`../src/editor_app/API.md`](../src/editor_app/API.md)

### UI Layer (`src/ui/*`)

**Verantwortung**

- Panels, Menüs, Viewport-HUD anzeigen
- Benutzeraktionen in `AppIntent` übersetzen

**Modul-Aufbau:**

- `input.rs` — Orchestrator, delegiert an Sub-Module
  - `keyboard.rs` — Tastatur-Shortcuts (Delete, Escape, Ctrl+A, W/G/B/A/R/Z/K fuer Floating-Menues)
  - `drag.rs` — Drag-Operationen (Pan, Move, Rect-/Lasso-Selektion)
  - `context_menu.rs` — Rechtsklick-Kontextmenü
- `common.rs` — Widgetnahe UI-Helfer fuer numerische Eingaben, Mausrad-Impulse und den zentralen Route-Tool-Availability-Kontext
- `menu.rs` — Top-Menü-Leiste
- `status.rs` — Statusleiste
- `defaults_panel.rs` — Linke Sidebar mit Long-Press-Gruppen (Werkzeuge, RouteTool-Gruppen `Basics/Section/Analysis`, Defaults, Hintergrund; 64px breit)
- `long_press.rs` — Wiederverwendbares Long-Press-Dropdown-Widget (`LongPressState`, `LongPressGroup<T>`, `render_long_press_button`)
- `floating_menu.rs` — Schwebendes Kontextmenue an der Mausposition (T/G/B/A/R/Z; `FloatingMenuKind::Tools` bzw. `FloatingMenuKind::RouteTools(RouteToolGroup)`)
- `command_palette.rs` — Zentrales Overlay fuer statische Befehle plus alle katalogsichtbaren Route-Tools; disabled Eintraege bleiben sichtbar und tragen ihren Grund
- `edit_panel.rs` — Kontextabhaengiges Edit-Panel fuer Gruppen-Edit, Streckenteilung und aktives Route-Tool
- `properties.rs` — Properties-Panel
  - intern modularisiert über `properties/`-Submodule (u. a. Selektoren und Distanzen-Panel)
- `options_dialog/` — Optionen-Dialog (modales Fenster plus thematische `sections/*`)
- `tool_preview.rs` — Tool-Preview-Overlay (Route-Tool-Vorschau im Viewport)
- `dialogs.rs` — Datei-Dialoge und modale Fenster

Numerische Mausrad-Interaktion bleibt bewusst im UI-Layer: `ui::common` kapselt Scroll-Auswertung, Modifier-Semantik und Wertanpassung fuer Numeric-Widgets. Route-Tool- und Analysis-Panels reichen weiterhin nur das boolesche Gate `wheel_enabled` weiter; Komma-Float-Felder im Options-Dialog nutzen ueber `apply_wheel_step_default()` repo-weit den Default-Schritt `0.1`, waehrend Felder mit bewusst groberen Prozent-/Grad- oder ganzzahligen Pixel-Schritten ihre expliziten Schrittweiten behalten. Gehoverte Numeric-Widgets konsumieren Raw-/Smooth-Scroll-Impulse bereits in `wheel_dir()`, damit umgebende ScrollAreas kein Scroll-Through zeigen.

**Darf**

- Read-only auf UI/View-Teile von `AppState`
- Events an Application Layer senden
- Aus `app` und seinen expliziten Submodulen importieren; nur die stabile Leseoberflaeche (`Camera2D`, `RoadMap`, `ConnectionDirection`, `ConnectionPriority`, `RenderQuality`, `ZipImageEntry` usw.) bleibt direkt aus `app` re-exportiert, Tool-Vertraege laufen explizit ueber `app::tool_contract`, `app::ui_contract` und `app::tools::*`

**Darf nicht**

- `core` direkt importieren
- `RoadMap` direkt mutieren
- XML lesen/schreiben
- Renderer direkt steuern

### Application Layer (`src/app/*`)

**Verantwortung**

- Zentrale Event-Verarbeitung (`AppController`)
- Use-Cases (Load/Save, Kamera, Selektion, Heightmap, Tools)
- Aufbau von `RenderScene` aus Domain + ViewState
- Schmale Read-only-Fassade fuer UI und Integrationsschale: app-eigene Typen plus bewusst ausgewaehlte Core-/Shared-Typen wie `ConnectionDirection`, `ConnectionPriority`, `RoadMap`, `Camera2D`, `RenderQuality`, `ZipImageEntry`
- Kanonischer RouteTool-Katalog (`tools/catalog.rs`) als Single Source of Truth fuer `RouteToolId`, `RouteToolGroup`, `RouteToolBackingMode`, `RouteToolIconKey`, Surface-Sichtbarkeit und Aktivierungs-Voraussetzungen
- Konsolidierte Asset-Leseflaeche im `AppState`: `farmland_polygons_arc()`, `farmland_grid_arc()` und `background_image_arc()` kapseln die kanonischen Tool-/Host-Zugriffe; `view.background_map` bleibt Primaerquelle fuer Hintergrundbilder, `background_image` nur Kompatibilitaets-Fallback
- Separater Tool-Editing-Layer (`tool_editing/*`) fuer persistente Tool-Snapshots, Rehydrierung sowie Cancel/Undo im destruktiven Tool-Edit-Flow

**Abgrenzung**

- `src/editor_app/*` gehoert nicht zum Application-Layer. Die Integrationsschale kapselt nur eframe-Frame-Zyklus, Event-Sammlung und Overlay-Anbindung und delegiert fachliche Mutationen an `AppController`.

**Use-Case-Module:**

- `use_cases/file_io.rs` — Laden, Speichern, Heightmap-Warnung
- `use_cases/camera.rs` — Kamera-Operationen
- `use_cases/viewport.rs` — Viewport-Logik
- `use_cases/heightmap.rs` — Heightmap-Laden und Konfiguration
- `use_cases/selection/` — Selektions-Use-Cases
- `use_cases/editing/` — Editing-Use-Cases
- `state.rs` — Fassade; Typen in `state/{app_state,dialogs,editor,selection,view}.rs`

### Overview-Crate (`crates/fs25_map_overview/src/*`)

**Verantwortung**

- Generierung der Übersichtsmap (Terrain-Komposition + optionale Overlays)
- Extraktion von Farmland-Polygonen fuer das `FieldBoundaryTool`

**Modulnotiz**

- `composite.rs` wird schrittweise in `composite/`-Submodule zerlegt (Start mit `legend.rs`).
- Detaillierte API: siehe [`crates/fs25_map_overview/API.md`](../crates/fs25_map_overview/API.md)

**Darf**

- Domain-API aufrufen
- XML-Use-Cases ausführen
- Renderer nur über `RenderScene` beliefern

### Domain/Core Layer (`src/core/*`)

**Verantwortung**

- Fachmodell (`RoadMap`, Knoten/Kanten, Marker, Meta)
- Invarianten und Regeln (IDs, Flags, Verbindungslogik)
- Deterministische Queries

**Darf nicht**

- UI-/egui-Typen kennen
- wgpu/Renderer-Typen kennen
- Dateidialoge oder direkte I/O enthalten

### Persistence Layer (`src/xml/*`)

**Verantwortung**

- XML <-> Domain Mapping (SoA, Delimiter-Regeln, Flag-Bereinigung)
- Datei-Ein-/Ausgabe für AD-Konfigurationen

**Darf nicht**

- UI- oder Kamera-Entscheidungen treffen
- Renderlogik enthalten

### Rendering Layer (`src/render/*`)

**Verantwortung**

- GPU-Ressourcen, Batching, Draw-Calls, Culling
- Darstellung auf Basis von `RenderScene`
- **Shader-Deduplication:** `shaders.wgsl` wird einmal in `Renderer::new()` geladen und an alle Sub-Renderer weitergegeben

**Darf nicht**

- Domain mutieren
- XML/Datei-I/O durchführen

## API-Verträge (Ist)

### Intent/Command-Vertrag

```mermaid
classDiagram
  class AppIntent {
    +OpenFileRequested
    +SaveRequested
    +SaveAsRequested
    +HeightmapSelectionRequested
    +HeightmapCleared
    +HeightmapWarningConfirmed
    +HeightmapWarningCancelled
    +ResetCameraRequested
    +ZoomInRequested
    +ZoomOutRequested
    +ViewportResized(size)
    +CameraPan(delta)
    +CameraZoom(factor)
    +NodePickRequested(world_pos)
    +RenderQualityChanged(quality)
    +FileSelected(path)
    +SaveFilePathSelected(path)
    +HeightmapSelected(path)
  }

  class AppCommand {
    +RequestOpenFileDialog
    +RequestSaveFileDialog
    +RequestHeightmapDialog
    +ClearHeightmap
    +ConfirmAndSaveFile
    +DismissHeightmapWarning
    +ResetCamera
    +ZoomIn
    +ZoomOut
    +SetViewportSize(size)
    +PanCamera(delta)
    +ZoomCamera(factor)
    +SelectNearestNode(world_pos, max_distance)
    +SetRenderQuality(quality)
    +LoadFile(path)
    +SaveFile(path)
    +SetHeightmap(path)
  }
```

### Application API

```text
pub struct AppController;

impl AppController {
  pub fn handle_intent(&mut self, state: &mut AppState, intent: AppIntent) -> anyhow::Result<()>;
  pub fn handle_command(&mut self, state: &mut AppState, command: AppCommand) -> anyhow::Result<()>;
  pub fn build_render_scene(&self, state: &AppState, viewport_size: [f32; 2]) -> RenderScene;
}
```

```text
pub struct AppState {
  pub road_map: Option<Arc<RoadMap>>,
  pub view: ViewState,
  pub ui: UiState,
  pub selection: SelectionState,
  pub editor: EditorToolState,
  pub clipboard: Clipboard,
  pub command_log: CommandLog,
  pub history: EditHistory,
  pub options: EditorOptions,
  pub show_options_dialog: bool,
  pub group_registry: GroupRegistry,
  pub farmland_polygons: Option<Arc<Vec<FieldPolygon>>>,
  pub farmland_grid: Option<Arc<FarmlandGrid>>,
  pub background_image: Option<Arc<DynamicImage>>,
  pub tool_edit_store: ToolEditStore,
  pub should_exit: bool,
}
```

```text
pub struct ViewState {
  pub camera: Camera2D,
  pub viewport_size: [f32; 2],
  pub render_quality: RenderQuality,
  pub background_map: Option<Arc<BackgroundMap>>,
  pub background_visible: bool,
  pub background_scale: f32,
  pub background_dirty: bool,  // Signalisiert GPU-Upload erforderlich
}
```

```text
pub struct SelectionState {
  pub selected_node_ids: Arc<IndexSet<u64>>,
  pub selection_anchor_node_id: Option<u64>,
}
```

### Render-Vertrag

```text
pub struct RenderScene {
  // private Felder
}

// enthaelt intern:
// - RenderMap-Snapshot (Nodes, Connections, Marker, KD-Index)
// - RenderCamera-Snapshot
// - Selection-/Hidden-/Dimmed-Mengen
// - RenderQuality, Optionen und Hintergrundstatus

impl Renderer {
  pub fn render_scene(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    render_pass: &mut wgpu::RenderPass<'static>,
    scene: &RenderScene,
  );
}
```

`render_scene::build()` baut den render-seitigen `RenderMap`-Snapshot nur bei geaenderter `RoadMap::render_cache_key()` neu auf und legt ihn in `AppState::render_map_cache` ab. Jeder Rebuild protokolliert `nodes`, `connections`, `markers` und `approx_bytes`, damit Performance-Reports neben Laufzeiten auch die Snapshot-Groesse desselben Datensatzes dokumentieren koennen.

## Event- und Render-Fluss

```mermaid
sequenceDiagram
  participant User
  participant UI as ui::menu / viewport
  participant Ctrl as AppController
  participant State as AppState
  participant Core as core::RoadMap
  participant Scene as RenderSceneBuilder
  participant Render as render::Renderer

  User->>UI: Click / Drag / Scroll
  UI->>Ctrl: AppIntent
  Ctrl->>Ctrl: map_intent_to_commands
  Ctrl->>Ctrl: handle_command
  Ctrl->>State: UI/View/State aktualisieren
  Ctrl->>Core: Fachoperation (optional)
  Core-->>Ctrl: aktualisierte Daten
  Ctrl->>Scene: RenderScene bauen
  Scene-->>Render: RenderScene
  Render-->>User: Frame
```

## Abhängigkeitsregeln

```mermaid
flowchart LR
  UI --> APP
  APP --> CORE
  APP --> XML
  APP --> SHARED
  RENDER --> SHARED
  XML --> CORE

  CORE -.x.-> UI
  CORE -.x.-> RENDER
  RENDER -.x.-> CORE
  XML -.x.-> APP
  RENDER -.x.-> UI
  UI -.x.-> XML
```

Verbindliche Regeln:

1. UI spricht nur mit `app` (`AppIntent` + read-only State) und `shared` (z.B. `EditorOptions`). **Kein direkter `core`-Import.**
2. Domain (`core`) kennt keine Infrastruktur (UI/Render/XML-Details).
3. Renderer konsumiert nur `RenderScene` plus render-eigene Upload-Vertraege und importiert keine Core-Typen.
4. XML bleibt technisch; fachliche Entscheidungen liegen in `core`/`app`.
5. `AppState` enthält keine I/O-Logik; Dateisystem- und Options-Persistenz liegen in Use-Cases bzw. der Integrationsschale, nicht im State oder in `shared`.
6. Renderer darf keine UI-Typen importieren. **Ausnahme:** `render/callback.rs` implementiert `egui_wgpu::CallbackTrait` — das ist die wgpu-Brücke zwischen egui und dem Rendering-System, kein semantischer UI-Import.
7. `app/mod.rs` bleibt eine schmale Lese-Fassade: app-eigene Typen plus bewusst ausgewaehlte Core-/Shared-Typen fuer UI und Integrationsschale; Tool-Vertraege und tool-spezifische Helfer werden nicht mehr ueber `app` root-reexportiert.
8. `shared`-Modul enthält nur neutrale Typen und Utilities, die von mehreren Layern genutzt werden (`RenderScene`, `RenderQuality`, `EditorOptions`, `i18n`, Geometrie-Helfer). Egui-Eingabe-Helfer leben in `ui`, Runtime-/Pfad-Policy in `app` bzw. `editor_app`. Importrichtung: `UI → shared`, `App → shared`, `Render → shared` (alle erlaubt).

## Aktuelle Modulstruktur

```text
src/
  main.rs             # Binary-Einstieg; ruft runtime::run()
  runtime.rs          # eframe-Bootstrap und EditorApp-Erzeugung
  editor_app/
    mod.rs            # EditorApp + eframe::App::update(); duenne Integrationsschale
    event_collection.rs # Panels, Dialoge und Viewport-Input zu AppIntents buendeln
    helpers.rs        # Render-Callback, Floating-Menue, Background-Upload, Repaint-Gating
    overlays.rs       # Tool-, Clipboard-, Distanz- und Gruppen-Overlays
  app/
    mod.rs              # Schmale Re-Exports fuer AppState/Controller + stabile UI-Lesetypen
    controller.rs       # AppController: duenne Fassade fuer Intent-Loop + Command-Logging
    controller/
      by_feature/       # Feature-Slices fuer Command-Dispatch (file_io, view, selection, editing, ...)
    events/
      mod.rs            # Re-Exports fuer AppIntent & AppCommand
      feature.rs        # Interne AppEventFeature-Klassifikation fuer Intent/Command-Schnitt
      intent.rs         # AppIntent + interne Feature-Zuordnung
      command.rs        # AppCommand + interne Feature-Zuordnung
    state.rs            # AppState, ViewState, SelectionState, UiState, MarkerDialogState, DedupDialogState
    render_scene.rs     # RenderScene-Builder
    command_log.rs      # Command-Log für Debugging
    history.rs          # Undo/Redo-History
    intent_mapping.rs   # duenne Fassade fuer Intent → Command Mapping
    intent_mapping/
      by_feature/       # Feature-Slices fuer reines Intent-Mapping
    handlers/           # Feature-Handler für Command-Verarbeitung
      mod.rs
      file_io.rs        # Datei-Operationen (Öffnen, Speichern, Heightmap)
      view.rs           # Kamera, Viewport, Background-Map
      selection.rs       # Selektions-Operationen
      editing.rs        # Node/Connection-Editing, Marker
      route_tool.rs     # Route-Tool-Operationen
      dialog.rs         # Dialog-State und Anwendungssteuerung
      history.rs        # Undo/Redo
    tools/
      mod.rs            # Re-Exporte fuer Tool-Vertraege, Capabilities, Katalog und snap_to_node()
      manager.rs        # ToolManager + Capability-Discovery
      contracts/        # RouteToolCore, RouteToolPanelBridge, RouteToolHostSync
      capabilities/     # Recreate, Drag, Tangent, Adjustments, ChainInput, LassoInput
      common/           # Gemeinsame Tool-Infrastruktur (alle Submodule privat)
        mod.rs          # Re-Exporte
        geometry.rs     # angle_to_compass, populate_neighbors

      Die Root-Dateien `controller.rs` und `intent_mapping.rs` bleiben damit stabil fuer Call-Sites und Dokumentation, waehrend die eigentlichen Match-Bloecke featureweise in `by_feature/*` liegen. `AppIntent` und `AppCommand` teilen sich dafuer intern die nicht-oeffentliche Klassifikation `AppEventFeature`.
        tangent.rs      # TangentSource, TangentState, render_tangent_combo
        lifecycle.rs    # ToolLifecycleState, SegmentConfig, LastEdited
        builder.rs      # assemble_tool_result
      curve/
        mod.rs          # Bézier-Kurven-Tool (Grad 2 + 3) — Einstiegspunkt
        state.rs        # CurveTool Struct, Enums, Konstruktor
        lifecycle.rs    # RouteTool-Implementierung
        drag.rs         # Drag-Logik (Steuerpunkt-Verschiebung)
        config_ui.rs    # egui-Konfigurationspanel
        geometry.rs     # Bézier-Mathe (Interpolation, Tangenten)
        tests.rs        # Unit-Tests
      spline/
        mod.rs          # Catmull-Rom-Spline-Tool — Einstiegspunkt
        state.rs        # SplineTool Struct, Konstruktor
        lifecycle.rs    # RouteTool-Implementierung
        config_ui.rs    # egui-Konfigurationspanel
        geometry.rs     # Spline-Geometrie (Arc-Length-Resampling)
        tests.rs        # Unit-Tests
      straight_line/
        mod.rs          # Gerade-Linie-Tool — Einstiegspunkt
        state.rs        # StraightLineTool Struct, Konstruktor
        lifecycle.rs    # RouteTool-Implementierung
        config_ui.rs    # egui-Konfigurationspanel
        geometry.rs     # Linien-Geometrie
        tests.rs        # Unit-Tests
    use_cases/
      mod.rs
      file_io.rs        # Load, Save, Heightmap-Warnung, Dateipfad-Handling
      heightmap.rs      # Heightmap-Laden und Konfiguration
      viewport.rs       # Viewport-Resize
      camera.rs         # Kamera-Reset, Zoom, Pan
      background_map.rs # Background-Map laden/konfigurieren
      selection/        # Selektions-Use-Cases (Pick, Rect, Lasso, Segment, Move)
      editing/          # Editing-Use-Cases (Connect, Delete, Direction, Priority, Marker, Route-Tool)
  core/
    mod.rs
    camera.rs           # Camera2D + pick_radius_world()
    centerline.rs       # Oeffentliche Fassade fuer Mittellinien-Berechnung
    centerline/
      polygon.rs        # Polygon-basierte Centerline ohne Raster-Grid
      segment.rs        # Segment-basierte Centerline ohne Raster-Grid
      voronoi.rs        # VoronoiGrid + Multi-Source-BFS auf Farmland-Rastern
      extract.rs        # Korridor- und Boundary-Extraktion aus Rasterdaten
      helpers.rs        # Gemeinsame Raster- und Geometrie-Helfer
      tests.rs          # Unit-Tests fuer die interne Aufteilung
    node.rs             # MapNode, NodeFlag
    connection.rs       # Connection, ConnectionDirection, ConnectionPriority
    road_map.rs         # RoadMap (HashMap-basiert, lazy Spatial-Index-Rebuild)
    road_map/
      dedup.rs          # Duplikat-Erkennung und -Bereinigung
      tests.rs          # Unit-Tests für RoadMap
    spatial.rs          # SpatialIndex (kiddo KD-Tree)
    heightmap.rs        # Heightmap + bikubische Interpolation
    map_marker.rs       # MapMarker
    meta.rs             # AutoDriveMeta
  xml/
    mod.rs
    parser.rs           # parse_autodrive_config()
    parser/
      tests.rs          # Unit-Tests für Parser
    writer.rs           # write_autodrive_config()
  render/
    mod.rs
    node_renderer.rs    # GPU-Instanced Nodes
    connection_renderer.rs  # Verbindungslinien + Pfeile
    background_renderer.rs  # Background-Map-Quad mit Texture
    marker_renderer.rs  # GPU-Instanced Map-Marker (Pin-Symbole)
    texture.rs          # Texture-Erstellung aus DynamicImage
    callback.rs         # wgpu Render-Callback für egui
    types.rs            # GPU-Typen, RenderContext, Konstanten
    shaders.wgsl        # WGSL Shader (Node, Connection, Background, Marker)
  shared/
    mod.rs
    render_scene.rs     # RenderScene (Übergabevertrag App → Render)
    render_quality.rs   # RenderQuality Enum
    options.rs          # Zentrale Konfigurationskonstanten + EditorOptions
    i18n/               # Mehrsprachigkeits-System (Language, I18nKey, t())
  ui/
    mod.rs
    menu.rs             # Top-Menü
    status.rs           # Statusleiste
    toolbar.rs          # Werkzeugleiste
    properties.rs       # Properties-Panel
    input.rs            # Input-Orchestrator
    keyboard.rs         # Tastatur-Shortcuts
    drag.rs             # Drag-Operationen
    context_menu.rs     # Rechtsklick-Kontextmenü
    dialogs.rs          # Datei-Dialoge
    options_dialog/     # Optionen-Dialog
    tool_preview.rs     # Tool-Preview-Overlay
```

**Hinweis:** `Camera2D` lebt in `core/camera.rs` (reiner Geometrie-Typ). `app` re-exportiert ihn als Teil der stabilen UI-Leseflaeche; Tool-Vertraege bleiben in expliziten `app::*`-Submodulen.

## Umsetzungsphasen

### Phase 1: Intent-Grenze ✅

- `AppIntent` als einziges UI->App Übergabeformat etabliert
- UI von direkten Domain-Mutationen entkoppelt
- Input-Sammeln in `collect_app_events(...)` gekapselt

### Phase 2: Controller-Zentrum ✅

- `AppController::handle_intent(...)` als zentraler Einstieg
- Intent->Command Mapping und `handle_command(...)` eingezogen
- Load/Save, Kamera und Viewport in dedizierte Use-Cases verschoben
- Heightmap-Logik in eigenständigen Use-Case extrahiert

### Phase 3: RenderScene-Vertrag ✅

- `RenderScene` als stabile Schnittstelle app->render eingeführt (in `shared`)
- Renderpfad auf read-only Render-Snapshots ohne `RoadMap`-/`Camera2D`-Vertrag umgestellt
- Orphaned `render/scene.rs` entfernt

### Phase 4: Modularisierung ✅

- UI-Input aufgeteilt in `keyboard`, `drag`, `context_menu` Sub-Module
- Pick-Distanz-Berechnung zentralisiert (`Camera2D::pick_radius_world()`)
- UI importiert aus `app` und dessen expliziten Submodulen; nur die stabile Leseoberflaeche bleibt direkt aus `app` re-exportiert
- API-Dokumentation auf aktuellen Stand gebracht

### Phase 5: Edit-Workflow ✅

- Selection/Move/Connect über Intents + Use-Cases implementiert
- Undo/Redo auf Snapshot-Basis (CommandLog/History vorhanden)
- Background-Map-Rendering (DDS/PNG/JPG) mit Use-Cases
- Marker-Rendering + erstellen/löschen via Use-Cases

### Phase 6: Handler-Split + Architektur-Guardrails ✅

- `handle_command()` in Feature-Handler aufgeteilt (`handlers/`)
- Controller ist jetzt schlanker Dispatcher, Logik in Handlern
- UI→Core-Layerverletzung behoben (properties.rs)
- CI-Check-Script für Schichtgrenzen (`scripts/check_layer_boundaries.sh`)
- Alle unwrap()-Aufrufe in Produktionscode durch graceful handling ersetzt
- Route-Tool-Intents: `RouteToolClicked`, `RouteToolExecuteRequested`, `RouteToolCancelled`, `SelectRouteToolRequested`, `RouteToolConfigChanged`, `RouteToolWithAnchorsRequested`, `RouteToolTangentSelected`, `RouteToolLassoCompleted`, `RouteToolRecreateRequested`
- Duplikat-Erkennung: `DeduplicateConfirmed`, `DeduplicateCancelled`
- Optionen-Dialog: `OpenOptionsDialogRequested`, `CloseOptionsDialogRequested`, `OptionsChanged`, `ResetOptionsRequested`

## Definition of Done

- Keine Domain-Mutationslogik in `ui`.
- `main.rs` und `runtime.rs` enthalten nur Bootstrap/Wiring; `src/editor_app/*` bleibt die duenne eframe-Integrationsschale.
- Alle User-Interaktionen laufen über `AppIntent`.
- Renderer arbeitet ohne direkten Domain-Zugriff.
- XML-Funktionalität ist unabhängig von UI/Render testbar.

## Nicht-Ziele

- Kein vollständiger Renderer-Rewrite in einem Schritt
- Kein Big-Bang-Umbau aller Tools
- Kein Wechsel des Dateiformats

---

## Detaillierte Diagramme

### Tool-Lifecycle (RouteTool-Pattern)

- Der feste Tool-Kern besteht aus `RouteToolCore`, `RouteToolPanelBridge` und `RouteToolHostSync`.
- Optionale Verhaltensweisen werden ueber additive Capability-Traits modelliert und vom `ToolManager` per Discovery angesprochen.
- `handlers/route_tool.rs`, `state/editor.rs` und die UI lesen keine No-Op-Hooks mehr direkt vom Umbrella-Trait, sondern fragen gezielt Drag-, Tangent-, Recreate-, Chain- oder Lasso-Capabilities ab.

```mermaid
stateDiagram-v2
  [*] --> Inactive
  Inactive --> Start: on_click(pos)
  Start --> End: on_click(pos)
  End --> Control: on_click(pos)
  Control --> Control: preview(cursor_pos)\npanel_state()/apply_panel_action()
  Control --> Executing: Enter/execute()
  Executing --> [*]: Done
  Control --> [*]: Escape/reset()
```

#### Analyse-Tool-Stages

- `ColorPathTool` trennt intern sieben Stages: A Sampling-Input, B Matching-Spezifikation, C Pixel-Maske + Sampling-Vorschau, D Maskenaufbereitung, E Skeleton-/Netzextraktion, F Preview-Aufbereitung und G Execute-Konvertierung.
- `lifecycle.rs` bleibt dabei auf Orchestrierung, Phase-Wechsel und die Implementierung der jeweiligen Basisvertraege/Capabilities beschraenkt; die Stage-Logik lebt in `pipeline.rs` und `preview.rs`.
- Preview und Execute teilen `PreparedSegment` als gemeinsame Wahrheit; es gibt bewusst keine Dirty-Bits, keine Cache-Ketten und keine gestaffelte Invalidation.

### Command-Intent-Flow

```mermaid
sequenceDiagram
participant UI as UI Layer<br/>(context_menu.rs)
participant CTRL as Controller<br/>(execute_intent)
participant HAND as Handler<br/>(apply)
participant STATE as AppState<br/>(Mutation)
participant SCENE as RenderScene<br/>(Build)

UI->>CTRL: AppIntent::NodePickRequested { world_pos, additive, extend_path }
activate CTRL
CTRL->>HAND: AppCommand::SelectNearestNode { world_pos, max_distance, additive, extend_path }
activate HAND
HAND->>STATE: use_cases::selection::select_nearest_node(...)
deactivate HAND
CTRL->>SCENE: build_render_scene()
CTRL->>UI: Observer notified
deactivate CTRL
```

### Layer-Isolation (Compile-Time Guardrails)

```mermaid
graph TB
    UI["🎨 UI Layer<br/>context_menu, keyboard, dialogs<br/>→ emits AppIntent"]
    APP["⚙️ App Layer<br/>controller, handlers, use_cases<br/>→ mutates AppState"]
    CORE["💾 Core Layer<br/>road_map, node, connection<br/>→ pure data structures"]
    SHARED["🔗 Shared<br/>RenderScene, EditorOptions<br/>→ Schnittstelle"]
    RENDER["🖥️ Render Layer<br/>Vertex/Fragment Shader, GPU Batching<br/>→ consumes RenderScene"]
    XML["📄 XML<br/>parser, writer<br/>→ I/O, unabhängig"]
    
    UI -->|uses| APP
    APP -->|uses| CORE
    APP -->|writes to| SHARED
    RENDER -->|reads from| SHARED
    APP -->|uses| XML
    CORE -->|never| RENDER
    CORE -->|never| UI
    UI -->|never| CORE
    
    style UI fill:#FFC
    style APP fill:#FCF
    style CORE fill:#CFF
    style SHARED fill:#FEE
    style RENDER fill:#FCC
    style XML fill:#EEE
```

---

## Performance-Patterns (aktiv, Stand 2026-03-07)

### Zoom-Kompensation (`EditorOptions.zoom_compensation_max`)

Nodes und Verbindungen erhalten eine garantierte Mindestgröße im Viewport, unabhängig vom Zoomlevel:

```
display_size = base_size * zoom_compensation(zoom)
zoom_compensation(zoom) = clamp(1 / zoom * ref_zoom, 1.0, max)
```

- `zoom_compensation_max` ist konfigurierbar (Standard: 4.0)
- Verhindert, dass Nodes bei 100k+ Waypoints auf < 1px schrumpfen
- Implementiert in `shared::options` (`zoom_compensation()`) und im Render-Hotpath angewendet

### Node-Decimation (Grid-Decimation, `EditorOptions.node_decimation_spacing_px`)

Nodes, die im aktuellen Zoom enger als N Pixel liegen, werden für das Rendering zusammengefasst:

```
Für jeden Node: world_to_screen(pos) → in NxN Pixel-Grid einordnen → nur ersten pro Zelle zeichnen
```

- Reduktion der Instanz-Daten im GPU-Hotpath bei weit herausgezoomter Ansicht
- `node_decimation_spacing_px = 0.0` deaktiviert Decimation vollständig
- Selektierte Nodes werden nie dezimiert (immer gezeichnet)

### Arc-basierter Clone-Schutz (`Arc<EditorOptions>`, `Arc<IndexSet<u64>>`)

Häufig geklonte State-Teile werden in `Arc` verpackt:
- `RenderScene.options: Arc<EditorOptions>` — O(1)-Clone statt Deep-Copy pro Frame
- `SelectionState.selected_node_ids: Arc<IndexSet<u64>>` — Copy-on-Write (CoW) via `Arc::make_mut`

### Lazy Spatial Index (`ensure_spatial_index()`)

`SpatialIndex` intern: `kiddo::ImmutableKdTree<f64, 2>`.  
Nach node-mutierenden Operationen wird dirty-Flag gesetzt; Rebuild erfolgt lazy beim nächsten Query — nicht sofort nach jeder Einzeloperation in Bulk-Loops.

---

## Tool-Encapsulation-Regeln (Stand 2026-03-07)

### Verbotene Abhaengigkeiten

- Tools (`src/app/tools/`) duerfen **niemals** auf `src/render/`, `wgpu` oder `RenderScene` zugreifen
- Tools erhalten ausschliesslich `&RoadMap` (read-only) als Domain-Kontext
- Keine GPU-spezifischen Typen (Vertex-Buffer, Shader, Pipelines) in Tool-Code
- Kein Zugriff auf `AppState` — der Handler (`src/app/handlers/route_tool.rs`) vermittelt

### Preview-Vertrag

- `preview()` liefert **reine Geometrie** (`Vec<Vec2>` + Index-basierte Verbindungen)
- Keine Farben, Texturen oder Render-Hints im `ToolPreview`-Output
- Die Konvertierung zu visuellen Elementen erfolgt im UI-Layer (`src/ui/tool_preview.rs`)
- Tools kennen weder `egui::Color32`, `egui::Painter` noch `egui::Ui`; das Floating-Panel liest `RouteToolPanelState` und sendet `RouteToolPanelAction` ueber den App-Intent-Flow zurueck

### Gruppen-Editierbarkeit

- Nur group-backed editierbare Tools implementieren `RouteToolGroupEdit` und liefern damit ein `RouteToolEditPayload`
- `FieldPath` und `ColorPath` bleiben `Ephemeral`; fuer sie gibt es keinen `ToolEditStore`-Eintrag und keinen destruktiven Tool-Edit
- `GroupRegistry` bleibt tool-neutral; Tool-Edit wird ueber `ToolEditStore` + `RouteToolId` freigeschaltet, nicht ueber Felder im `GroupRecord`
- Alle Pflicht-Surfaces lesen Route-Tools ueber `resolve_route_tool_entries()`; deaktivierte Tools bleiben sichtbar und tragen ihren Disabled-Grund
- `GroupRecord.locked` verhindert versehentliche Mutation
- Undo-Snapshot wird vor jeder Mutation automatisch erstellt (`apply_tool_result`)

### ToolResult-Aufbau

- Lineare Ketten-Tools nutzen `assemble_tool_result()` (`common/builder.rs`)
- Spezial-Topologien (geschlossene Ringe, Multi-Seiten-Offsets) bauen `ToolResult` manuell
- Alle `ToolResult`-Instanzen verwenden den **gleichen Struct** — einheitliches Interface

```
Tool.preview() → ToolPreview (pure Geometrie)
     ↓
UI: paint_preview() → egui::Painter (2D-Overlay)

Tool.execute() → ToolResult (Nodes + Connections)
     ↓
Handler: apply_tool_result() → RoadMap-Mutation
     ↓
Naechster Frame: RenderScene aktualisiert den gecachten Render-Snapshot bei geaenderter RoadMap-Revision
```
