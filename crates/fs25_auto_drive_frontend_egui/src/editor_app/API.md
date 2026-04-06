# API des editor_app-Moduls

## Ueberblick

Das `editor_app`-Modul ist die duenne Integrationsschale zwischen dem Binary-Start (`main.rs`/`runtime.rs`) und den fachlichen Layern aus `ui`, `app` und `render`. Es besitzt keinen eigenen Fach-Use-Case: Domain-Mutationen laufen weiterhin ausschliesslich ueber `AppController`, waehrend `EditorApp` nur den eframe-Frame-Zyklus, die Event-Sammlung, das Viewport-Rendering und die Overlay-Anbindung koordiniert. Das Laden der Editor-Optionen erfolgt beim Start ueber `app::use_cases::options`, damit die Runtime-/Dateisystem-Policy nicht in `shared` lebt.

`HostBridgeSession` ist die kanonische Session-Surface fuer den egui-Host sowie direkte Flutter-/FFI-Consumer. Der Datei-/Pfad-Dialogpfad im `editor_app` konsumiert Requests ueber `take_host_dialog_requests(...)` als `HostDialogRequest` und fuehrt Ergebnisse ueber `HostSessionAction::SubmitDialogResult` in den gemeinsamen Dispatch-Pfad zurueck. Bridge-owned Read-Seams (`HostUiSnapshot`, `ViewportOverlaySnapshot`, gekoppelter `HostRenderFrameSnapshot`) und stabile Action-Mappings laufen fuer den lokalen egui-Host ebenfalls ueber die kanonischen Host-Bridge-Helfer in `fs25_auto_drive_host_bridge`.

Die API ist bewusst `pub(crate)` und nur fuer das Binary relevant. Die kanonische Dokumentation liegt hier, damit `src/app/API.md` ausschliesslich den Application-Layer beschreibt und nicht gleichzeitig die eframe-Integrationsschale als zweite Wahrheitsquelle pflegen muss.

## Modulaufbau

| Submodul | Verantwortung |
|---|---|
| `mod.rs` | `EditorApp`, Konstruktion, `eframe::App::update()` und Intent-Weitergabe |
| `event_collection.rs` | Panels ueber `build_host_ui_snapshot(...)` lesen, Datei-/Pfad-Dialoge ueber `take_host_dialog_requests()` als `HostDialogRequest` drainen, `HostDialogResult` ueber `HostSessionAction::SubmitDialogResult` auf denselben Bridge-Dispatch-Pfad fuehren und Viewport-Input buendeln |
| `helpers.rs` | Render-Callback, Floating-Menue-Toggle, Background-Upload und Repaint-Steuerung; bridge-owned Render-Reads laufen ueber den gekoppelten `build_render_frame(...)`-Seam |
| `overlays.rs` | Holt `ViewportOverlaySnapshot` ueber `build_viewport_overlay_snapshot(...)`, zeichnet Tool-/Clipboard-/Distanzen-/Gruppen-Overlays und mappt Overlay-Klicks auf `AppIntent` |

## Integrationsrelevante Typen

### `EditorApp`

Crate-interne Hauptstruktur fuer eine laufende eframe-Editorinstanz.

```rust
pub(crate) struct EditorApp {
    state: AppState,
    controller: AppController,
    renderer: Arc<Mutex<render::Renderer>>,
    device: eframe::wgpu::Device,
    queue: eframe::wgpu::Queue,
    input: ui::InputState,
    last_cursor_world: Option<glam::Vec2>,
    last_background_asset_revision: u64,
    last_background_transform_revision: u64,
    pending_render_assets: Option<RenderAssetsSnapshot>,
    group_boundary_icons: Option<ui::GroupBoundaryIcons>,
}
```

**Verantwortung:**

- Haelt den laufenden `AppState` und den zugehoerigen `AppController`
- Verwaltet die wgpu-Bruecke zum `render::Renderer`
- Kapselt fensterlokalen Integrationszustand (`ui::InputState`, Cursor-Cache, Icon-Handles)
- Delegiert alle fachlichen Aenderungen ueber `AppController::handle_intent(...)`

### `impl eframe::App for EditorApp`

Die `update()`-Implementierung bildet den Frame-Zyklus der Integrationsschale:

1. Exit-Guard pruefen (`state.should_exit`)
2. UI-, Dialog-, Viewport- und Overlay-Events sammeln
3. Die gesammelte `Vec<AppIntent>` by-value durchlaufen und schalenlokale Events behandeln (z.B. `ToggleFloatingMenu`)
4. Stabile, bridge-faehige Intents zuerst ueber `fs25_auto_drive_host_bridge::apply_mapped_intent(...)` auf die gemeinsame Host-Dispatch-Seam geben
5. Nur nicht-gemappte Intents ohne zusaetzliches `event.clone()` an `AppController` delegieren
6. Background-Sync aus den Assets des bereits fuer den Viewport aufgebauten RenderFrames ausfuehren und danach die Repaint-Entscheidung treffen

## Integrationsrelevante Funktionen

| Signatur | Zweck |
|---|---|
| `pub(crate) fn new(render_state: &egui_wgpu::RenderState) -> Self` | Laedt `EditorOptions` ueber `app::use_cases::options::load_editor_options()`, initialisiert `AppState`, `AppController`, `render::Renderer` und `ui::InputState` |
| `fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)` | Zentraler eframe-Frame-Zyklus der Integrationsschale |
| `fn process_events(&mut self, ctx: &egui::Context, events: Vec<AppIntent>)` | Behandelt Intents by-value in drei Stufen: lokal (z. B. Floating-Menue), kanonische Host-Bridge-Seam fuer stabile Host-Aktionen, danach Legacy-Fallback ueber `AppController` |
| `fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent>` | Baut Panels ueber `build_host_ui_snapshot(...)`, drainet Dialog-Anforderungen ueber `take_host_dialog_requests(&controller, &mut state)` und mappt Dialog-Ergebnisse ueber `HostSessionAction::SubmitDialogResult` zurueck in den Intent-Flow |
| `fn render_viewport(&mut self, ui: &egui::Ui, rect: egui::Rect, viewport_size: [f32; 2])` | Baut einen gekoppelten RenderFrame ueber `build_render_frame(...)`, uebergibt dessen Szene an den egui/wgpu-Render-Callback und cached dessen Assets fuer denselben Frame |
| `fn render_overlays(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response, viewport_size: [f32; 2]) -> Vec<AppIntent>` | Baut `ViewportOverlaySnapshot` ueber `build_viewport_overlay_snapshot(...)`, zeichnet daraus Overlays und mappt Overlay-Interaktionen auf `AppIntent`s |
| `fn toggle_floating_menu(&mut self, ctx: &egui::Context, kind: FloatingMenuKind)` | Oeffnet oder schliesst das kontextbezogene Floating-Menue an der aktuellen Mausposition |
| `fn sync_background_upload(&mut self)` | Synchronisiert Background-Upload/Clear revisionsbasiert aus den Assets des bereits aufgebauten RenderFrames; kein separater spaeter Host-Asset-Read |
| `fn maybe_request_repaint(&self, ctx: &egui::Context, has_meaningful_events: bool)` | Vermeidet unnoetige Idle-Repaints und haelt aktive UI-Zustaende fluessig |

## Beispiel

Der Binary-Start erzeugt `EditorApp` nach erfolgreicher eframe/wgpu-Initialisierung in `runtime.rs`:

```rust
eframe::run_native(
    "FS25 AutoDrive Editor",
    native_options(),
    Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("wgpu nicht verfuegbar"))?;

        Ok(Box::new(EditorApp::new(render_state)))
    }),
)?;
```

## Integrationsfluss

```mermaid
flowchart LR
    MAIN[src/main.rs] --> ENTRY[fs25_auto_drive_frontend_egui::run_native()]
    ENTRY --> RUNTIME[runtime::run_native()]
  RUNTIME --> EDITOR[editor_app::EditorApp]
  EDITOR --> UI[ui::*]
  UI --> INTENTS[AppIntent]
  INTENTS --> CTRL[AppController]
  CTRL --> STATE[AppState]
    CTRL --> FRAME[HostRenderFrameSnapshot]
    FRAME --> SCENE[RenderScene]
    FRAME --> ASSETS[RenderAssetsSnapshot]
    SCENE --> RENDER[render::Renderer]
    ASSETS --> RENDER
```

## Abgrenzung

- `editor_app` ist keine zweite Application-Schicht, sondern reine Anbindung
- Die fachliche API des Controllers, der States und der Use-Cases bleibt in `../app/API.md` dokumentiert
- `runtime.rs` startet die App; `editor_app` kapselt den laufenden Frame-Betrieb