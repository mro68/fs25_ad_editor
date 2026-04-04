# API der Host-Bridge-Core-Crate

## Ueberblick

`fs25_auto_drive_host_bridge` ist die kanonische, toolkit-freie Bruecken-Crate ueber `fs25_auto_drive_engine`. Sie kapselt `AppController` und `AppState` in einer gemeinsamen Session-Fassade und stellt eine explizite Action- plus Snapshot-Surface fuer mehrere Hosts bereit (egui, Flutter, weitere Adapter).

Die Crate bleibt absichtlich host-neutral: keine eframe/egui-Runtime, keine Flutter-FFI und keine wgpu-RenderPass-Lifecycle-Logik.

## Oeffentliche Module

| Modul | Verantwortung |
|---|---|
| `session` | `HostBridgeSession` als kanonische Session-Fassade ueber der Engine |
| `dto` | Serialisierbare Host-Actions, Dialog-DTOs und Session-Snapshots |

## Wichtige oeffentliche Typen

| Typ | Zweck |
|---|---|
| `HostBridgeSession` | Toolkit-freie Session-Fassade mit expliziten Mutationen und Read-Snapshots |
| `HostRenderFrameSnapshot` | Gekoppelter Render-Snapshot (`RenderScene` + `RenderAssetsSnapshot`) |
| `HostSessionAction` | Explizite Action-Surface fuer Host-seitige Mutationen |
| `HostSessionSnapshot` | Kleine serialisierbare Session-Zusammenfassung fuer Polling-Hosts |
| `HostDialogRequest` / `HostDialogResult` | Semantische Host-Dialoganforderung und Rueckmeldung |
| `HostActiveTool` | Stabiler Tool-Identifier fuer Snapshot/API-Vertrag |

## Oeffentliche Methoden

| Signatur | Zweck |
|---|---|
| `pub fn new() -> Self` | Erstellt eine neue Bridge-Session mit leerem Engine-State |
| `pub fn apply_action(&mut self, action: HostSessionAction) -> Result<()>` | Wendet eine explizite Host-Aktion an |
| `pub fn toggle_command_palette(&mut self) -> Result<()>` | Komfort-Action fuer Command-Palette |
| `pub fn set_editor_tool(&mut self, tool: HostActiveTool) -> Result<()>` | Komfort-Action fuer Toolwechsel |
| `pub fn set_options_dialog_visible(&mut self, visible: bool) -> Result<()>` | Oeffnet/schliesst den Optionen-Dialog |
| `pub fn undo(&mut self) -> Result<()>` | Fuehrt Undo ueber die Action-Surface aus |
| `pub fn redo(&mut self) -> Result<()>` | Fuehrt Redo ueber die Action-Surface aus |
| `pub fn take_dialog_requests(&mut self) -> Vec<HostDialogRequest>` | Entnimmt ausstehende semantische Dialoganforderungen |
| `pub fn submit_dialog_result(&mut self, result: HostDialogResult) -> Result<()>` | Gibt Host-Dialogergebnis semantisch an die Engine zurueck |
| `pub fn snapshot(&mut self) -> &HostSessionSnapshot` | Liefert gecachten Session-Snapshot ohne Allokationsdrift |
| `pub fn snapshot_owned(&mut self) -> HostSessionSnapshot` | Liefert Snapshot als besitzende Kopie |
| `pub fn build_render_scene(&self, viewport_size: [f32; 2]) -> RenderScene` | Liefert den per-frame Render-Vertrag |
| `pub fn build_render_assets(&self) -> RenderAssetsSnapshot` | Liefert den langlebigen Asset-Snapshot |
| `pub fn build_render_frame(&self, viewport_size: [f32; 2]) -> HostRenderFrameSnapshot` | Liefert Szene und Assets als gekoppelten read-only Render-Output |
| `pub fn build_host_ui_snapshot(&self) -> HostUiSnapshot` | Liefert host-neutrale Panel-/Dialogdaten |
| `pub fn build_viewport_overlay_snapshot(&mut self, cursor_world: Option<Vec2>) -> ViewportOverlaySnapshot` | Liefert host-neutrale Viewport-Overlay-Daten |

## Beispiel

```rust
use fs25_auto_drive_host_bridge::{
	HostBridgeSession, HostSessionAction,
};

let mut session = HostBridgeSession::new();
session.apply_action(HostSessionAction::ToggleCommandPalette)?;

let snapshot = session.snapshot();
let frame = session.build_render_frame([1280.0, 720.0]);

assert!(snapshot.show_command_palette);
assert_eq!(frame.scene.viewport_size(), [1280.0, 720.0]);
```
