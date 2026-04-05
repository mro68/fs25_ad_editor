# API des editor_app-Moduls

## Ueberblick

Das `editor_app`-Modul ist die duenne Integrationsschale zwischen dem Binary-Start (`main.rs`/`runtime.rs`) und den fachlichen Layern aus `ui`, `app` und `render`. Es besitzt keinen eigenen Fach-Use-Case: Domain-Mutationen laufen weiterhin ausschliesslich ueber `AppController`, waehrend `EditorApp` nur den eframe-Frame-Zyklus, die Event-Sammlung, das Viewport-Rendering und die Overlay-Anbindung koordiniert. Das Laden der Editor-Optionen erfolgt beim Start ueber `app::use_cases::options`, damit die Runtime-/Dateisystem-Policy nicht in `shared` lebt.

Die API ist bewusst `pub(crate)` und nur fuer das Binary relevant. Die kanonische Dokumentation liegt hier, damit `src/app/API.md` ausschliesslich den Application-Layer beschreibt und nicht gleichzeitig die eframe-Integrationsschale als zweite Wahrheitsquelle pflegen muss.

## Modulaufbau

| Submodul | Verantwortung |
|---|---|
| `mod.rs` | `EditorApp`, Konstruktion, `eframe::App::update()` und Intent-Weitergabe |
| `event_collection.rs` | Panels ueber `HostUiSnapshot` lesen, Datei-/Pfad-Dialoge ueber `AppController::take_dialog_requests()` drainen, `PanelAction`/`DialogResult` in `AppIntent` mappen und Viewport-Input buendeln |
| `helpers.rs` | Render-Callback, Floating-Menue-Toggle, Background-Upload und Repaint-Steuerung |
| `overlays.rs` | Holt `ViewportOverlaySnapshot` ueber den Controller, zeichnet Tool-/Clipboard-/Distanzen-/Gruppen-Overlays und mappt Overlay-Klicks auf `AppIntent` |

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
4. Uebrige `AppIntent`s ohne zusaetzliches `event.clone()` an `AppController` delegieren
5. Hintergrund-Uploads und Repaint-Entscheidung ausfuehren

## Integrationsrelevante Funktionen

| Signatur | Zweck |
|---|---|
| `pub(crate) fn new(render_state: &egui_wgpu::RenderState) -> Self` | Laedt `EditorOptions` ueber `app::use_cases::options::load_editor_options()`, initialisiert `AppState`, `AppController`, `render::Renderer` und `ui::InputState` |
| `fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)` | Zentraler eframe-Frame-Zyklus der Integrationsschale |
| `fn process_events(&mut self, ctx: &egui::Context, events: Vec<AppIntent>)` | Behandelt die gesammelten Intents by-value, verarbeitet schalenlokale Events in-place und delegiert den Rest ohne zusaetzliche Clone-Kaskade an den Controller |
| `fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent>` | Baut zuerst `AppController::build_host_ui_snapshot()` fuer Panels und drainet Dialog-Anforderungen kanonisch ueber `AppController::take_dialog_requests(&mut state)` |
| `fn render_viewport(&mut self, ui: &egui::Ui, rect: egui::Rect, viewport_size: [f32; 2])` | Baut `RenderScene` und registriert den egui/wgpu-Render-Callback |
| `fn render_overlays(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response, viewport_size: [f32; 2]) -> Vec<AppIntent>` | Baut `AppController::build_viewport_overlay_snapshot(...)`, zeichnet daraus Overlays und mappt Overlay-Interaktionen auf `AppIntent`s |
| `fn toggle_floating_menu(&mut self, ctx: &egui::Context, kind: FloatingMenuKind)` | Oeffnet oder schliesst das kontextbezogene Floating-Menue an der aktuellen Mausposition |
| `fn sync_background_upload(&mut self)` | Synchronisiert Background-Upload/Clear revisionsbasiert ueber `AppController::build_render_assets()` |
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
    CTRL --> SCENE[RenderScene]
    CTRL --> ASSETS[RenderAssetsSnapshot]
    SCENE --> RENDER[render::Renderer]
    ASSETS --> RENDER
```

## Abgrenzung

- `editor_app` ist keine zweite Application-Schicht, sondern reine Anbindung
- Die fachliche API des Controllers, der States und der Use-Cases bleibt in `../app/API.md` dokumentiert
- `runtime.rs` startet die App; `editor_app` kapselt den laufenden Frame-Betrieb