# API des editor_app-Moduls

## Ueberblick

Das `editor_app`-Modul ist die duenne Integrationsschale zwischen dem Binary-Start (`main.rs`/`runtime.rs`) und den fachlichen Layern aus `ui`, `app`, `render` sowie `fs25_auto_drive_host_bridge`. Es besitzt keinen eigenen Fach-Use-Case. `EditorApp` haelt genau eine `HostBridgeSession` als einzige Session-Quelle; `AppController` und `AppState` werden nur noch intern von dieser Session gekapselt. Das Laden der Editor-Optionen erfolgt beim Start ueber `app::use_cases::options`, damit Runtime-/Dateisystem-Policy nicht in `shared` lebt.

Bridge-owned Read-Seams (`HostUiSnapshot`, `HostChromeSnapshot`, `HostRouteToolViewportSnapshot`, `ViewportOverlaySnapshot`, gekoppelter `HostRenderFrameSnapshot`) und stabile Write-Seams (`HostSessionAction`, inklusive `RouteTool`-Familie und `SubmitViewportInput`) laufen im `editor_app` ausschliesslich ueber diese Session. Fuer verbleibende nicht serialisierte UI-Local-Zustaende nutzt `editor_app` schmale Session-Seams (`panel_properties_state_mut`, `dialog_ui_state_mut`, `viewport_input_context_mut`, `toggle_floating_menu`) statt direkter `app_state_mut()`-Zugriffe; damit existiert im produktiven egui-Pfad kein direkter mutabler `AppState`-Bypass mehr. Der read-only Escape-Hatch `app_state()` bleibt fuer Exit-/Repaint-Checks bestehen. Fuer bereits kanonisierte Route-Tool-/Chrome-Intents ist der Intent-Fallback im `editor_app` explizit gesperrt.

Die Event-Sammlung ist seit der Entflechtung bewusst modularisiert: `event_collection.rs` orchestriert nur noch den Frame-Aufbau, waehrend `panel_collector.rs`, `dialog_collector.rs` und `viewport_collector.rs` die jeweiligen Teilflaechen ueber klar getrennte Session-Seams bedienen.

Das Modul bleibt als `pub mod` fuer stabile Frontend-Importpfade sichtbar, die konkreten Typen wie `EditorApp` sind jedoch crate-intern. Die kanonische Dokumentation liegt hier, damit `src/app/API.md` ausschliesslich den Application-Layer beschreibt und nicht gleichzeitig die eframe-Integrationsschale als zweite Wahrheitsquelle pflegen muss.

## Modulaufbau

| Submodul | Verantwortung |
|---|---|
| `mod.rs` | `EditorApp`, Konstruktion, `eframe::App::ui()` und Session-basierte Event-Weitergabe |
| `event_collection.rs` | Orchestriert den Frame: baut `HostUiSnapshot` und `HostChromeSnapshot`, verteilt an Panel-/Dialog-/Viewport-Collector und fuehrt den zentralen Viewport samt Overlays zusammen |
| `panel_collector.rs` | Sammelt Menue-, Status-, Defaults-, Marker-, Eigenschaften- und Edit-Panel-Events ueber `HostUiSnapshot`, `HostChromeSnapshot` und `panel_properties_state_mut()` |
| `dialog_collector.rs` | Drainet Datei-/Pfad-Dialoge ueber `HostBridgeSession::take_dialog_requests()`, mappt Ergebnisse auf Intents zurueck und bedient modale egui-Dialoge ueber `dialog_ui_state_mut()` |
| `viewport_collector.rs` | Sammelt rohe Viewport-Gesten, konsumiert `HostRouteToolViewportSnapshot` und kombiniert dies mit `viewport_input_context_mut()` fuer den host-lokalen Input-Zustand |
| `helpers.rs` | Render-Callback, Floating-Menue-Toggle, Background-Upload und Repaint-Steuerung; Render-Reads laufen ueber den gekoppelten `HostBridgeSession::build_render_frame(...)`-Seam, Floating-Menue-Toggle ueber `HostBridgeSession::toggle_floating_menu(...)` |
| `overlays.rs` | Holt `ViewportOverlaySnapshot` ueber `HostBridgeSession::build_viewport_overlay_snapshot(...)`, zeichnet Tool-/Clipboard-/Distanzen-/Gruppen-Overlays und nutzt `HostChromeSnapshot` fuer Tool-/Options-Kontext ohne doppelten Snapshot-Build |

## Integrationsrelevante Typen

### `EditorApp`

Crate-interne Hauptstruktur fuer eine laufende eframe-Editorinstanz.

```rust
pub(crate) struct EditorApp {
    session: HostBridgeSession,
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

- Haelt die laufende `HostBridgeSession` als einzige Session-Quelle des egui-Hosts
- Verwaltet die wgpu-Bruecke zum `render::Renderer`
- Kapselt fensterlokalen Integrationszustand (`ui::InputState`, Cursor-Cache, Icon-Handles)
- Initialisiert Editor-Optionen beim Start ueber `HostSessionAction::ApplyOptions`
- Delegiert fachliche Aenderungen primär ueber `session.apply_action(...)`; `session.apply_intent(...)` bleibt als uebergangsweiser, explizit begrenzter Fallback fuer noch nicht kanonisierte Intents

### `impl eframe::App for EditorApp`

Die `ui()`-Implementierung bildet den Frame-Zyklus der Integrationsschale:

1. Exit-Guard pruefen (`session.app_state().should_exit`)
2. UI-, Dialog-, Viewport- und Overlay-Events sammeln
3. Die gesammelte gemischte Event-Liste by-value durchlaufen und schalenlokale Events behandeln (z. B. `ToggleFloatingMenu`)
4. `HostSessionAction`s direkt auf die Session anwenden
5. `AppIntent`s ueber `dispatch_intent_via_session(...)` erst auf die kanonische Host-Action-Surface mappen; der lokale Fallback bleibt nur fuer explizit erlaubte, noch nicht kanonisierte Intents offen
6. Background-Sync aus den Assets des bereits fuer den Viewport aufgebauten RenderFrames ausfuehren und danach die Repaint-Entscheidung treffen

## Integrationsrelevante Funktionen

| Signatur | Zweck |
|---|---|
| `pub(crate) fn new(render_state: &egui_wgpu::RenderState) -> Self` | Laedt `EditorOptions`, initialisiert `HostBridgeSession`, schreibt die Optionen ueber `HostSessionAction::ApplyOptions` in den Session-State und baut `render::Renderer` plus `ui::InputState` auf |
| `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)` | Zentraler eframe-Frame-Zyklus der Integrationsschale |
| `fn process_events(&mut self, ctx: &egui::Context, events: Vec<CollectedEvent>)` | Behandelt gemischte Events: `HostSessionAction` direkt ueber die Session, `AppIntent` ueber `dispatch_intent_via_session(...)` |
| `fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<CollectedEvent>` | Baut Panels ueber `build_host_ui_snapshot()` + `build_host_chrome_snapshot()`, drainet Dialog-Anforderungen ueber `session.take_dialog_requests()`, liest `HostRouteToolViewportSnapshot` und sammelt Viewport-Gesten als `CollectedEvent::HostAction(SubmitViewportInput { ... })` oder `CollectedEvent::Intent(...)` |
| `fn collect_panel_events(&mut self, ctx: &egui::Context, host_ui_snapshot: &HostUiSnapshot, host_chrome_snapshot: &HostChromeSnapshot, marker_list: &HostMarkerListSnapshot, top_ui: &mut egui::Ui) -> Vec<CollectedEvent>` | Rendert Menues, Status, Defaults-Panel und Edit-Panel; nutzt fuer Properties-/Edit-Local-State ausschliesslich den schmalen Session-Seam `panel_properties_state_mut()` und konsumiert den pro Frame einmal aufgebauten Marker-Snapshot fuer die rechte Sidebar |
| `fn collect_dialog_events(&mut self, ctx: &egui::Context, host_ui_snapshot: &HostUiSnapshot, marker_list: &HostMarkerListSnapshot) -> Vec<AppIntent>` | Fuehrt semantische Host-Dialoge ueber `session.take_dialog_requests()` und `HostSessionAction::SubmitDialogResult`; modale egui-Fenster mutieren host-lokale Dialog-States ueber `dialog_ui_state_mut()` und teilen sich den im Frame-Loop vorgeladenen Marker-Snapshot mit dem Panel-Collector |
| `fn collect_viewport_events(&mut self, ui: &egui::Ui, response: &egui::Response, viewport_size: [f32; 2], command_palette_open: bool) -> Vec<CollectedEvent>` | Liest `HostRouteToolViewportSnapshot`, leitet daraus Drag-/Tangenten-/Lasso-Hinweise fuer `ui::InputState` ab und bezieht verbleibende lokale Viewport-Read/Write-Daten ueber `viewport_input_context_mut()` |
| `fn render_viewport(&mut self, ui: &egui::Ui, rect: egui::Rect, viewport_size: [f32; 2])` | Baut einen gekoppelten RenderFrame ueber `session.build_render_frame(...)`, uebergibt dessen Szene an den egui/wgpu-Render-Callback und cached dessen Assets fuer denselben Frame |
| `fn render_overlays(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response, viewport_size: [f32; 2], host_chrome_snapshot: &HostChromeSnapshot) -> Vec<AppIntent>` | Baut `ViewportOverlaySnapshot` ueber `session.build_viewport_overlay_snapshot(...)`, nutzt den bereits vorhandenen `HostChromeSnapshot` fuer Tool-/Options-Kontext und mappt Overlay-Interaktionen auf `AppIntent`s |
| `fn toggle_floating_menu(&mut self, ctx: &egui::Context, kind: FloatingMenuKind)` | Oeffnet oder schliesst das kontextbezogene Floating-Menue an der aktuellen Mausposition ueber `session.toggle_floating_menu(...)` |
| `fn sync_background_upload(&mut self)` | Synchronisiert Background-Upload/Clear revisionsbasiert aus den Assets des bereits aufgebauten RenderFrames; kein separater spaeter Host-Asset-Read |
| `fn maybe_request_repaint(&self, ctx: &egui::Context, has_meaningful_events: bool)` | Vermeidet unnoetige Idle-Repaints und haelt aktive UI-Zustaende fluessig; prueft dafuer host-lokale Sichtbarkeit ueber `session.chrome_state()` |
| `fn dispatch_intent_via_session(session: &mut HostBridgeSession, intent: AppIntent) -> anyhow::Result<()>` | Nutzt zuerst `map_intent_to_host_action(...)`; fuer kanonisierte Route-Tool-/Chrome-Intents ist ein lokaler Fallback verboten, nur explizit erlaubte Rest-Intents laufen uebergangsweise ueber `session.apply_intent(...)` |

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
        EDITOR --> SESSION[HostBridgeSession]
        UI --> EVENTS[AppIntent / HostSessionAction]
        EVENTS --> EDITOR
        SESSION --> CTRL[AppController]
        CTRL --> STATE[AppState]
        SESSION --> HOSTUI[HostUiSnapshot]
        SESSION --> CHROME[HostChromeSnapshot]
        SESSION --> ROUTEVIEW[HostRouteToolViewportSnapshot]
        SESSION --> OVERLAY[ViewportOverlaySnapshot]
        SESSION --> FRAME[HostRenderFrameSnapshot]
        HOSTUI --> UI
        CHROME --> UI
        ROUTEVIEW --> UI
        OVERLAY --> UI
        FRAME --> SCENE[RenderScene]
        FRAME --> ASSETS[RenderAssetsSnapshot]
        SCENE --> RENDER[render::Renderer]
        ASSETS --> RENDER
```

## Abgrenzung

- `editor_app` ist keine zweite Application-Schicht, sondern reine Anbindung
- Die fachliche API des Controllers, der States und der Use-Cases bleibt in `../app/API.md` dokumentiert
- `runtime.rs` startet die App; `editor_app` kapselt den laufenden Frame-Betrieb auf Basis einer session-owned `HostBridgeSession`