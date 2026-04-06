# API der Host-Bridge-Core-Crate

## Ueberblick

`fs25_auto_drive_host_bridge` ist die kanonische, toolkit-freie Host-Bridge ueber `fs25_auto_drive_engine`. Die Crate kapselt `AppController` und `AppState` in `HostBridgeSession` und buendelt damit die gemeinsame Session-Surface fuer den egui-Host, direkte Flutter-/FFI-Consumer und spaetere Transport-Adapter.

`HostBridgeSession` ist verbindlich die kanonische Session-Surface fuer den egui-Host sowie direkte Flutter-/FFI-Consumer. Host-spezifische Adapter duerfen neue host-neutrale Session-Seams nicht mehr direkt auf `AppController`/`AppState` aufbauen, sondern ausschliesslich ueber diese Bridge-Surface.

Fuer bestehende Flutter-/FFI-Call-Sites stellt die Crate die bisherigen `Engine*`-Typnamen und den Session-Namen `FlutterBridgeSession` direkt als Kompatibilitaets-Aliase bereit. Damit koennen externe Consumer direkt auf `fs25_auto_drive_host_bridge` wechseln, ohne im selben Schritt alle Symbolnamen umzubenennen.

Der aktuell produktive Flutter-Pfad konsumiert diese Kanonik ueber den Linux-first-C-ABI-Adapter `fs25_auto_drive_host_bridge_ffi`. Dieser Transportadapter fuehrt bewusst keine zweite Session- oder DTO-Surface ein, sondern serialisiert nur die bereits hier definierten Host-Vertraege.

Die Bridge exponiert Mutationen ausschliesslich ueber explizite `HostSessionAction`-DTOs. Die Action-Surface deckt stabile Host-Aktionen ab (Datei-/Dialog-Anforderungen, Kamera-/Viewport-Shortcuts, Historie, Optionen, Toolwechsel, Exit) und seit dem ersten schreibenden Viewport-Slice genau eine kleine screen-space-basierte Input-Familie fuer `Resize`, Kamera-`Pan`/`Zoom`, `Select` per Single-Tap, Segment-/Gruppen-Selektion per Double-Tap, `Shift`-Rect, normales `Alt`-Lasso, Move-Drag sowie `AddNode`-/`Connect`-Klicks. Route-Tool-spezifische Drag-/Lasso-Pfade, Rotations- und Touch-Gesten bleiben bewusst ausserhalb dieses Slices. Fuer read-only Hosts liefert die Crate weiterhin kleine Session-Snapshots, host-neutrale Panel-Read-Modelle, Viewport-Overlay-Snapshots, einen minimalen serialisierbaren Viewport-Geometry-Snapshot sowie gekoppelten Render-Output aus `RenderScene` und `RenderAssetsSnapshot`. Dieser gekoppelte RenderFrame ist jetzt sowohl ueber `HostBridgeSession::build_render_frame(...)` als auch ueber den freien Dispatch-Helper `build_render_frame(...)` fuer lokale Rust-Hosts verfuegbar.

Der konsolidierte Host-Dialog-Vertrag deckt neben `open_file` und `save_file` auch `heightmap` und `background_map` ab. Ob ein Host dafuer einen nativen Picker oder einen lokalen Fallback nutzt, bleibt explizit host-local; der Bridge-Vertrag aendert sich dafuer nicht.

Die Crate bleibt absichtlich host-neutral: keine eframe/egui-Runtime, keine Flutter-FFI und keine wgpu-RenderPass-Lifecycle-Logik.

Die konsolidierte Host-Dialog-Seam bildet die interne Engine-Queue `DialogRequest`/`DialogResult` verlustfrei auf die host-stabilen DTOs `HostDialogRequest`/`HostDialogResult` ab. Hosts mit eigener Session nutzen dafuer `HostBridgeSession::take_dialog_requests()` und `submit_dialog_result(...)`; Hosts mit eigenem `AppController`/`AppState` verwenden dieselbe Mapping-Logik ueber `take_host_dialog_requests(...)` plus `HostSessionAction::SubmitDialogResult`.

`take_host_dialog_requests(...)` ist dabei bewusst keine zweite Session-API, sondern ein enger Adapter-Hilfspfad fuer den aktuellen Konsolidierungsslice: Er ueberbrueckt bestehende Host-Integrationen mit lokalem Controller/State, waehrend `HostBridgeSession` die kanonische Session-Surface und Zielrichtung bleibt.

Mit `HostChromeSnapshot` existiert zusaetzlich ein expliziter host-neutraler Read-Seam fuer Menues, Defaults, Status und Route-Tool-Metadaten. Egui konsumiert diesen Snapshot lokal; der FFI-Adapter spiegelt dieselbe Surface additiv ueber `fs25ad_host_bridge_session_chrome_snapshot_json(...)`.

## Session-Grenze (Stand 2026-04-06)

- **bridge-owned:** Explizite Action-/Snapshot-Seams (`HostSessionAction`, `HostSessionSnapshot`, `HostUiSnapshot`, `HostChromeSnapshot`, `ViewportOverlaySnapshot`, Render-Read-Seams inklusive gekoppeltem `build_render_frame(...)`) und die stateful Viewport-Input-Familie (`HostViewportInputBatch`, `HostViewportInputState`) liegen zentral in der Host-Bridge.
- **bridge-gap:** Fuer stabile Host-Aktionen, bridge-owned Read-Seams und den kanonischen Viewport-Gesture-Slice aktuell geschlossen; lokale Host-Glue-Logik bleibt nur fuer bewusst spaetere/out-of-scope Pfade ausserhalb der Bridge.
- **host-local:** eframe-/egui- und Render-Glue bleiben bewusst ausserhalb der Bridge; ebenso out-of-scope bleiben Route-Tool-Viewportflows inklusive Tool-Lasso, Rotations- und Touch-Gesten.

## Bewusste Nicht-Ziele fuer Slice 0

- Kein zweiter Flutter-spezifischer Session- oder DTO-Vertrag neben `HostBridgeSession`.
- Keine zweite, Flutter-spezifische Viewport-Input-Surface neben `HostSessionAction::SubmitViewportInput`.
- Kein Route-Tool-spezifischer Lasso-/Drag-, Rotations- oder Touch-Viewportvertrag in diesem Slice.
- Keine toolkit-spezifische Runtime-, Packaging- oder Loader-Logik in dieser Core-Crate.

## Oeffentliche Module

| Modul | Verantwortung |
|---|---|
| `dispatch` | Wiederverwendbare Rust-Host-Dispatch-Seam (`HostSessionAction` <-> `AppIntent`) und bridge-owned Read-Helper-Seams fuer lokale Controller/State-Hosts |
| `session` | `HostBridgeSession` als kanonische Session-Fassade ueber der Engine |
| `dto` | Serialisierbare Host-Actions, Dialog-DTOs, Session-Snapshots plus `Engine*`-Kompatibilitaets-Aliase |

## Oeffentliche Dispatch-Funktionen

| Signatur | Zweck |
|---|---|
| `pub fn map_intent_to_host_action(intent: &AppIntent) -> Option<HostSessionAction>` | Uebersetzt einen stabilen Engine-Intent in eine explizite Host-Action |
| `pub fn map_host_action_to_intent(action: HostSessionAction) -> Option<AppIntent>` | Uebersetzt eine Host-Action in einen stabilen Engine-Intent |
| `pub fn apply_mapped_intent(controller: &mut AppController, state: &mut AppState, intent: &AppIntent) -> Result<bool>` | Wendet einen stabil gemappten Intent direkt ueber die gemeinsame Host-Seam an |
| `pub fn apply_host_action(controller: &mut AppController, state: &mut AppState, action: HostSessionAction) -> Result<bool>` | Wendet die gemeinsame Dispatch-Seam direkt auf einen bestehenden Rust-Host-State an |
| `pub fn apply_host_action_with_viewport_input_state(controller: &mut AppController, state: &mut AppState, input_state: &mut HostViewportInputState, action: HostSessionAction) -> Result<bool>` | Wendet auch stateful `SubmitViewportInput`-Actions direkt auf einen bestehenden Rust-Host-State an |
| `pub fn apply_viewport_input_batch(controller: &mut AppController, state: &mut AppState, input_state: &mut HostViewportInputState, batch: HostViewportInputBatch) -> Result<bool>` | Interpretiert den kleinen screen-space Viewport-Input-Vertrag bridge-owned auf bestehende Engine-Intents |
| `pub fn take_host_dialog_requests(controller: &AppController, state: &mut AppState) -> Vec<HostDialogRequest>` | Enger Adapter-Hilfspfad fuer Hosts mit lokalem Controller/State; entnimmt ausstehende Dialog-Anforderungen und mappt sie auf den kanonischen Host-Dialog-DTO-Vertrag |
| `pub fn build_host_ui_snapshot(controller: &AppController, state: &AppState) -> HostUiSnapshot` | Baut den host-neutralen Panel-Snapshot fuer Hosts mit lokalem Controller/State |
| `pub fn build_host_chrome_snapshot(state: &AppState) -> HostChromeSnapshot` | Baut den host-neutralen Chrome-Snapshot fuer Menues, Defaults, Status und Route-Tool-Metadaten |
| `pub fn build_viewport_overlay_snapshot(controller: &AppController, state: &mut AppState, cursor_world: Option<Vec2>) -> ViewportOverlaySnapshot` | Baut den host-neutralen Overlay-Snapshot fuer lokale Host-Adapter |
| `pub fn build_render_scene(controller: &AppController, state: &AppState, viewport_size: [f32; 2]) -> RenderScene` | Baut den per-frame Render-Vertrag fuer lokale Host-Adapter |
| `pub fn build_render_frame(controller: &AppController, state: &AppState, viewport_size: [f32; 2]) -> HostRenderFrameSnapshot` | Baut Szene und Assets als gekoppelten read-only RenderFrame fuer lokale Rust-Hosts |
| `pub fn build_viewport_geometry_snapshot(controller: &AppController, state: &AppState, viewport_size: [f32; 2]) -> HostViewportGeometrySnapshot` | Baut einen kleinen, serialisierbaren Geometry-Snapshot fuer FFI-/Polling-Hosts |
| `pub fn build_render_assets(controller: &AppController, state: &AppState) -> RenderAssetsSnapshot` | Baut den langlebigen Render-Asset-Snapshot fuer lokale Host-Adapter |

## Wichtige oeffentliche Typen

| Typ | Zweck |
|---|---|
| `HostBridgeSession` | Toolkit-freie Session-Fassade mit expliziten Mutationen und Read-Snapshots |
| `FlutterBridgeSession` | Kompatibilitaetsalias auf `HostBridgeSession` |
| `HostRenderFrameSnapshot` | Gekoppelter Render-Snapshot (`RenderScene` + `RenderAssetsSnapshot`) |
| `EngineRenderFrameSnapshot` | Kompatibilitaetsalias auf `HostRenderFrameSnapshot` |
| `HostSessionAction` | Kanonische Mutationsoberflaeche fuer Host-seitige Eingriffe |
| `HostViewportInputBatch` / `HostViewportInputEvent` | Kleine screen-space Viewport-Input-Familie fuer Resize, Pointer- und Scroll-Events |
| `HostPointerButton` / `HostTapKind` / `HostInputModifiers` | Stabile Transport-DTOs fuer Pointer-Buttons, Tap-Art und Modifiers |
| `HostViewportInputState` | Kleiner bridge-owned Drag-/Resize-Zustand fuer Session oder lokale Rust-Hosts |
| `EngineSessionAction` | Kompatibilitaetsalias auf `HostSessionAction` |
| `HostSessionSnapshot` | Kleine serialisierbare Session-Zusammenfassung fuer Polling-Hosts |
| `EngineSessionSnapshot` | Kompatibilitaetsalias auf `HostSessionSnapshot` |
| `HostChromeSnapshot` | Host-neutrales Read-Modell fuer Menues, Defaults, Status und Route-Tool-Availability |
| `HostRouteToolEntrySnapshot` / `HostRouteToolSelectionSnapshot` | Serialisierbare Route-Tool-Metadaten fuer Surface, Gruppe, Icon-Key, Availability und Gruppen-Memory |
| `HostDefaultConnectionDirection` / `HostDefaultConnectionPriority` | Stabile Default-Enums fuer Verbindungsrichtung und Prioritaet im Chrome-Snapshot |
| `HostSelectionSnapshot` / `HostViewportSnapshot` | Read-only Detail-Snapshots fuer Auswahl und Kamera |
| `HostViewportGeometrySnapshot` | Minimaler, serialisierbarer Viewport-Geometry-Snapshot fuer Nodes, Connections, Marker und Kamera-/Viewport-Metadaten |
| `EngineSelectionSnapshot` / `EngineViewportSnapshot` | Kompatibilitaets-Aliase auf die kanonischen Host-Snapshots |
| `EngineViewportGeometrySnapshot` | Kompatibilitaetsalias auf den kanonischen Geometry-Snapshot |
| `HostDialogRequestKind` / `HostDialogRequest` / `HostDialogResult` | Semantische Host-Dialoganforderungen und Rueckmeldungen |
| `EngineDialogRequestKind` / `EngineDialogRequest` / `EngineDialogResult` | Kompatibilitaets-Aliase auf den kanonischen Host-Dialog-Vertrag |
| `HostActiveTool` | Stabiler Tool-Identifier fuer Snapshot- und Action-Vertrag |
| `EngineActiveTool` | Kompatibilitaetsalias auf `HostActiveTool` |
| `HostUiSnapshot` / `ViewportOverlaySnapshot` | Host-neutrale Read-Modelle fuer Panels bzw. Viewport-Overlays, am Crate-Root re-exportiert |

## Oeffentliche Methoden

| Signatur | Zweck |
|---|---|
| `pub fn new() -> Self` | Erstellt eine neue Bridge-Session mit leerem Engine-State |
| `pub fn apply_action(&mut self, action: HostSessionAction) -> Result<()>` | Wendet eine explizite Host-Aktion an |
| `pub fn toggle_command_palette(&mut self) -> Result<()>` | Komfort-Action fuer die Command-Palette |
| `pub fn set_editor_tool(&mut self, tool: HostActiveTool) -> Result<()>` | Komfort-Action fuer den Toolwechsel |
| `pub fn set_options_dialog_visible(&mut self, visible: bool) -> Result<()>` | Oeffnet oder schliesst den Optionen-Dialog explizit |
| `pub fn undo(&mut self) -> Result<()>` | Fuehrt Undo ueber die Action-Surface aus |
| `pub fn redo(&mut self) -> Result<()>` | Fuehrt Redo ueber die Action-Surface aus |
| `pub fn take_dialog_requests(&mut self) -> Vec<HostDialogRequest>` | Entnimmt ausstehende semantische Dialoganforderungen aus der Session |
| `pub fn submit_dialog_result(&mut self, result: HostDialogResult) -> Result<()>` | Gibt ein Host-Dialogergebnis semantisch an die Engine zurueck |
| `pub fn snapshot(&mut self) -> &HostSessionSnapshot` | Liefert den gecachten Session-Snapshot fuer allokationsarmes Polling |
| `pub fn snapshot_owned(&mut self) -> HostSessionSnapshot` | Liefert den Snapshot als besitzende Kopie |
| `pub fn build_render_scene(&self, viewport_size: [f32; 2]) -> RenderScene` | Liefert den per-frame Render-Vertrag |
| `pub fn build_render_assets(&self) -> RenderAssetsSnapshot` | Liefert den langlebigen Asset-Snapshot |
| `pub fn build_render_frame(&self, viewport_size: [f32; 2]) -> HostRenderFrameSnapshot` | Liefert Szene und Assets als gekoppelten read-only Render-Output |
| `pub fn build_viewport_geometry_snapshot(&self, viewport_size: [f32; 2]) -> HostViewportGeometrySnapshot` | Liefert einen kleinen, serialisierbaren Geometry-Snapshot fuer Transport-Adapter |
| `pub fn build_host_ui_snapshot(&self) -> HostUiSnapshot` | Liefert host-neutrale Paneldaten |
| `pub fn build_host_chrome_snapshot(&self) -> HostChromeSnapshot` | Liefert host-neutrale Chrome-Daten fuer Menues, Defaults und Status |
| `pub fn build_viewport_overlay_snapshot(&mut self, cursor_world: Option<Vec2>) -> ViewportOverlaySnapshot` | Liefert host-neutrale Viewport-Overlay-Daten; `&mut self` bleibt absichtlich noetig, weil der App-Layer dabei Overlay- und Boundary-Caches aufwaermt |

## Beispiel

```rust
use fs25_auto_drive_host_bridge::{HostBridgeSession, HostSessionAction};

let mut session = HostBridgeSession::new();
session.apply_action(HostSessionAction::ToggleCommandPalette)?;

let snapshot = session.snapshot();
let frame = session.build_render_frame([1280.0, 720.0]);

assert!(snapshot.show_command_palette);
assert_eq!(frame.scene.viewport_size(), [1280.0, 720.0]);
```

## Datenfluss

```mermaid
flowchart LR
	HOST[Host / Adapter] --> ACTION[HostSessionAction]
	ACTION --> SESSION[HostBridgeSession]
	SESSION --> CTRL[AppController]
	CTRL --> STATE[AppState]
	STATE --> SNAP[HostSessionSnapshot]
	CTRL --> HOSTUI[HostUiSnapshot]
	CTRL --> OVERLAY[ViewportOverlaySnapshot]
	CTRL --> SCENE[RenderScene]
	CTRL --> ASSETS[RenderAssetsSnapshot]
	SESSION --> FRAME[HostRenderFrameSnapshot]
	SCENE --> FRAME
	ASSETS --> FRAME
	SESSION --> DIALOGS[HostDialogRequest Queue]
	HOST --> RESULT[HostDialogResult]
	RESULT --> SESSION
	SNAP --> HOST
	HOSTUI --> HOST
	OVERLAY --> HOST
	FRAME --> HOST
	DIALOGS --> HOST
```

## Kompatibilitaets-Aliase

`fs25_auto_drive_host_bridge` bietet die Legacy-Namen fuer direkte Consumer-Migration:

| Aliasname | Kanonischer Typ |
|---|---|
| `FlutterBridgeSession` | `HostBridgeSession` |
| `EngineRenderFrameSnapshot` | `HostRenderFrameSnapshot` |
| `EngineSessionAction` | `HostSessionAction` |
| `EngineSessionSnapshot` | `HostSessionSnapshot` |
| `EngineSelectionSnapshot` | `HostSelectionSnapshot` |
| `EngineViewportSnapshot` | `HostViewportSnapshot` |
| `EngineDialogRequestKind` / `EngineDialogRequest` / `EngineDialogResult` | `HostDialogRequestKind` / `HostDialogRequest` / `HostDialogResult` |
| `EngineActiveTool` | `HostActiveTool` |

## Hinweise

- `snapshot()` arbeitet ueber einen Dirty-Cache und baut `HostSessionSnapshot` nur nach erfolgreichen Mutationen oder entnommenen Dialog-Requests neu auf.
- `HostBridgeSession::apply_action(...)` delegiert intern an dieselbe `dispatch`-Seam, die auch nicht-Session-basierte Rust-Hosts nutzen koennen.
- `HostSessionAction::SubmitViewportInput` bleibt die einzige neue write-heavy Action-Familie auf der bestehenden JSON-Surface. Der Vertrag ist bewusst klein und screen-space-basiert: `Resize`, `Tap` (Single/Double), `DragStart`/`DragUpdate`/`DragEnd` fuer Pan/Move/Rect/Select-Lasso und `Scroll`.
- Stateful Viewport-Input benoetigt `HostViewportInputState`. `HostBridgeSession` besitzt diesen Zustand intern; lokale Rust-Hosts verwenden dafuer `apply_host_action_with_viewport_input_state(...)` oder `apply_viewport_input_batch(...)`.
- `take_dialog_requests()` und `submit_dialog_result(...)` bilden die kanonische Dialog-Seam der Session-API. Fuer Adapter mit eigenem `AppController`/`AppState` steht dieselbe Mapping-Logik zusaetzlich ueber `take_host_dialog_requests(...)` als schmaler Adapter-Hilfspfad bereit.
- Die Mapping-Seam fuer stabile, niederfrequente Host-Aktionen liegt zentral in `dispatch` (`map_intent_to_host_action`, `map_host_action_to_intent`, `apply_mapped_intent`, `apply_host_action`).
- `apply_host_action(...)` bleibt absichtlich stateless. Fuer `SubmitViewportInput` liefert diese Hilfsfunktion daher einen Fehler statt stiller Semantik-Drift; stateful Input laeuft ueber `HostBridgeSession` oder die dedizierte Dispatch-Hilfe mit `HostViewportInputState`.
- Die bridge-owned Read-Seams fuer lokale Controller/State-Hosts sind zentral in `dispatch` verfuegbar (`build_host_ui_snapshot`, `build_viewport_overlay_snapshot`, `build_render_scene`, `build_render_frame`, `build_viewport_geometry_snapshot`, `build_render_assets`).
- Der serialisierbare Geometry-Snapshot bleibt bewusst klein und read-only: Nodes, Connections, Marker sowie Kamera-/Viewport-Metadaten. Der neue Write-Slice ist gezielt auf den stabilen Viewport-Input begrenzt und zieht keine Route-Tool-spezifischen oder Touch-Vertraege mit.
- Host-Adapter mit eigenem `AppController`/`AppState` koennen den Datei-/Pfad-Dialogpfad ueber `take_host_dialog_requests(...)` und `HostSessionAction::SubmitDialogResult` auf denselben Bridge-DTO-/Dispatch-Vertrag konsolidieren.
