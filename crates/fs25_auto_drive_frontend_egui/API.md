# API der egui-Frontend-Crate

## Ueberblick

`fs25_auto_drive_frontend_egui` kapselt den nativen Desktop-Host des Editors. Die Crate enthaelt die komplette egui-Oberflaeche, die eframe-Integrationsschale, den nativen Launcher und einen duennen render-seitigen Host-Adapter ueber `fs25_auto_drive_render_wgpu`.

Sie konsumiert die host-neutrale Engine, re-exportiert deren `app`-, `core`-, `shared`- und `xml`-Module fuer bestehende Frontend-Pfade und stellt mit `run_native()` den nativen Einstieg bereit.

Die Integrationsschale liest Panels ueber `HostUiSnapshot`, konsumiert Datei-/Pfaddialog-Requests ueber `take_host_dialog_requests(...)` als `HostDialogRequest` und verarbeitet Viewport-Overlays ueber `ViewportOverlaySnapshot`. Dialog-Ergebnisse werden ueber `HostSessionAction::SubmitDialogResult` in dieselbe Host-Bridge-Dispatch-Seam zurueckgefuehrt, waehrend `PanelAction` und Overlay-Klicks zentral in `AppIntent` uebersetzt werden. Die egui-Crate fuehrt dafuer keine zweite Dialog-DTO-Familie ein, sondern nutzt den kanonischen Host-Bridge-Vertrag direkt.

Die gemeinsame Host-Bridge ist in dieser Crate die kanonische Dispatch- und Read-Seam fuer stabile, niederfrequente Host-Aktionen und bridge-owned Read-Modelle. `editor_app` bleibt die produktive eframe-Integrationsschale: lokale Spezialfaelle bleiben lokal, bridge-faehige Intents laufen ueber `fs25_auto_drive_host_bridge::apply_mapped_intent(...)` (optional via `host_bridge_adapter`-Kompat-Reexport), hochfrequente Viewport-/Tool-Intents bleiben im Legacy-Fallback ueber `AppController`. Das Onscreen-Rendering liest Szene und Assets dabei ueber denselben gekoppelten `build_render_frame(...)`-Seam wie der bestehende Native-Canvas-Kern; egui nutzt davon nur die Szene fuer den Paint-Callback und wiederverwendet die Assets im selben Frame fuer den revisionsbasierten Background-Sync.

`HostBridgeSession` bleibt dabei verbindlich die kanonische Session-Surface fuer den egui-Host sowie direkte Flutter-/FFI-Consumer. Der freie Dialogpfad ueber `take_host_dialog_requests(...)` ist bewusst nur ein enger Adapter-Hilfspfad fuer den aktuellen Konsolidierungsslice des bestehenden `editor_app`-Hosts, keine zweite vollwertige Session-API.

## Kompatibilitaet (Stand: 2026-04-05)

- Rust-Edition: `2024`
- UI-Stack: `eframe/egui/egui-wgpu 0.34.1`
- Render-Seam: kompatibel zum Render-Core auf `wgpu 29.0.*`
- Scroll-Input: rohe Wheel-Impulse werden aus `MouseWheel`-Events aggregiert (statt des entfernten Feldes `raw_scroll_delta`).

## Oeffentliche Module

| Modul | Verantwortung |
|---|---|
| `editor_app` | eframe-Integrationsschale; sammelt Panels ueber `HostUiSnapshot`, drainet Dialoge ueber `take_host_dialog_requests(...)` als `HostDialogRequest`, liest gekoppelte RenderFrames ueber `build_render_frame(...)` und rendert Overlays aus `ViewportOverlaySnapshot` |
| `host_bridge_adapter` | Duenne Kompat-Surface mit Reexports auf die kanonische Host-Bridge-Mapping-Seam (`map_intent_to_host_action`, `apply_mapped_intent`) |
| `render` | egui-Host-Adapter, revisionsbasierte Background-Upload-Bruecke und egui-Render-Callback ueber die von `editor_app` gelieferten RenderFrame-Daten |
| `ui` | Menues, Panels, Dialoge, Viewport-Input und egui-spezifisches Painting der host-neutralen Overlay-Snapshots |
| `app`, `core`, `shared`, `xml` | Re-Exports aus `fs25_auto_drive_engine` fuer stabile Importpfade |

## Session-Grenze (Stand 2026-04-05)

- **bridge-owned:** Mapping/Dispatch fuer stabile Host-Aktionen (`map_intent_to_host_action`, `apply_mapped_intent`), host-neutrale Read-Modelle (`HostUiSnapshot`, `ViewportOverlaySnapshot`, gekoppelter RenderFrame-Seam) und der Datei-/Pfad-Dialog-Lifecycle ueber `HostDialogRequest`/`HostDialogResult` laufen ueber `fs25_auto_drive_host_bridge`.
- **bridge-gap:** Fuer stabile Host-Aktionen und bridge-owned Reads aktuell geschlossen; verbleibende direkte Controller-Aufrufe sind bewusst host-local/high-frequency.
- **host-local:** eframe-Lifecycle, egui-Widget-State, Input-Orchestrierung, Render-Callback und Upload-Glue.
- **Leitplanke:** Keine neuen host-neutralen Fluesse direkt auf `AppController`/`AppState` aufbauen.

## Wichtige oeffentliche Typen

| Typ | Zweck |
|---|---|
| `render::Renderer` | Egui-Host-Adapter fuer den host-neutralen GPU-Renderer-Kern |
| `render::RendererTargetConfig` | Re-exportierte Target-Konfiguration fuer Farbformat und MSAA des Render-Core |
| `render::BackgroundWorldBounds` | Weltkoordinatenvertrag fuer Background-Uploads |
| `render::WgpuRenderCallback` | egui/wgpu-Bruecke fuer den benutzerdefinierten Render-Pass |
| `render::WgpuRenderData` | Traeger des `RenderScene`-Teils eines gekoppelten RenderFrames pro Frame |
| `ui::InputState` | Persistenter Viewport-Inputzustand pro Fenster |
| `ui::GroupOverlayEvent` | Rueckkanal fuer Gruppen-Overlay-Interaktionen |
| `app::ui_contract::HostUiSnapshot` | Host-neutraler Panel-Snapshot, den `editor_app` pro Frame konsumiert |
| `app::ui_contract::ViewportOverlaySnapshot` | Host-neutraler Overlay-Snapshot fuer Tool-, Clipboard-, Distanzen- und Gruppen-Overlays |

## Oeffentliche Funktionen und Re-Exports

| Signatur | Zweck |
|---|---|
| `pub fn run_native() -> Result<(), eframe::Error>` | Startet Logger, eframe-Fenster und `EditorApp` |
| `pub fn host_bridge_adapter::map_intent_to_host_action(intent: &AppIntent) -> Option<HostSessionAction>` | Kompat-Reexport auf die kanonische Host-Bridge-Mapping-Seam |
| `pub fn host_bridge_adapter::apply_mapped_intent(controller: &mut AppController, state: &mut AppState, intent: &AppIntent) -> Result<bool>` | Kompat-Reexport auf den kanonischen Bridge-Dispatch fuer stabile Host-Aktionen |
| `pub use fs25_auto_drive_engine::{app, core, shared, xml};` | Re-exportiert die host-neutrale Engine-Surface |

## Beispiel

```rust
fn main() -> Result<(), eframe::Error> {
		fs25_auto_drive_frontend_egui::run_native()
}
```

## Integrationsfluss

```mermaid
flowchart LR
	MAIN[src/main.rs] --> ENTRY[fs25_auto_drive_frontend_egui::run_native]
	ENTRY --> RUNTIME[runtime::run_native]
	RUNTIME --> EDITOR[editor_app::EditorApp]
	EDITOR --> UI[ui::*]
	EDITOR --> CTRL[app::AppController]
	CTRL --> HOSTUI[app::ui_contract::HostUiSnapshot]
	CTRL --> OVERLAY[app::ui_contract::ViewportOverlaySnapshot]
	CTRL --> FRAME[HostRenderFrameSnapshot]
	FRAME --> SCENE[shared::RenderScene]
	FRAME --> ASSETS[shared::RenderAssetsSnapshot]
	HOSTUI --> UI
	OVERLAY --> UI
	SCENE --> RENDER[render::Renderer Adapter]
	ASSETS --> RENDER
	RENDER --> CORE[fs25_auto_drive_render_wgpu::Renderer]
```

## Kompatibilitaet

- Das Root-Package re-exportiert `render` und `ui` weiterhin.
- `editor_app` bleibt der produktive Desktop-Flow; `host_bridge_adapter` ist nur noch eine Kompat-Surface ueber der kanonischen Host-Bridge und fuehrt keine lokale Mapping-Logik mehr.
- Der Datei-/Pfad-Dialogpfad in egui laeuft ueber die kanonische Host-Dialog-Seam (`take_host_dialog_requests(...)` + `HostSessionAction::SubmitDialogResult`). `take_host_dialog_requests(...)` bleibt dabei ein schmaler Adapter-Hilfspfad fuer den bestehenden egui-Host mit lokalem Controller/State.
- Das egui-Onscreen-Rendering laeuft bewusst nicht ueber RGBA-Readback oder `CanvasRuntime`; es konsumiert denselben logischen `HostRenderFrameSnapshot`-Seam wie Native-Canvas-Adapter, verwendet lokal aber nur den `RenderScene`-Teil fuer den Paint-Callback.
- Die kanonischen Moduldetails stehen in `src/editor_app/API.md`, `src/render/API.md` und `src/ui/API.md`.
