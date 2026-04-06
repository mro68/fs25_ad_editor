# Projekt: FS25 AutoDrive Editor (RADE)

## Ueberblick

Neuimplementierung des AutoDrive Course Editors in Rust mit egui + wgpu. Hochperformantes Tool fuer 100k+ Wegpunkte mit Cross-Platform-Faehigkeit.

## Architektur

- **Root-Package (`FS25-AutoDrive-Editor`):** Kompat-Fassade (`src/lib.rs`) + nativer Launcher (`src/main.rs`)
- **Engine (`crates/fs25_auto_drive_engine`):** `app`, `core`, `shared`, `xml`
- **Host-Bridge-Core (`crates/fs25_auto_drive_host_bridge`):** toolkit-freie gemeinsame Session-/Action-/Snapshot-Seam ueber der Engine
- **Render-Core (`crates/fs25_auto_drive_render_wgpu`):** host-neutraler wgpu-Kern (`Renderer`, Sub-Renderer, Shader)
- **Egui-Frontend (`crates/fs25_auto_drive_frontend_egui`):** `ui`, `editor_app`, `runtime`, `render` als Host-Adapter plus duenne `host_bridge_adapter`-Kompat-Surface (Reexports auf die kanonische Host-Bridge-Seam)
- **Overview-Crate (`crates/fs25_map_overview`):** Terrain-, Farmland- und Overview-Generierung

## Event- und Mutationsfluss

- UI erzeugt `AppIntent`
- `AppController` mappt auf `AppCommand`
- Commands werden ueber Use-Cases ausgefuehrt
- Mutationen laufen zentral gegen `AppState`
- Renderer bekommt pro Frame `RenderScene` plus Asset-Sync ueber `RenderAssetsSnapshot`
- `RenderScene` transportiert nur Render-Snapshots; Core-Typen duerfen nicht nach `crates/fs25_auto_drive_render_wgpu/src/*` durchgereicht werden
- Hintergrund-Uploads laufen ueber `background_asset_revision` / `background_transform_revision` statt Dirty-Flag
- `fs25_auto_drive_host_bridge` darf nur von `fs25_auto_drive_engine` abhaengen
- Direkte Flutter-/FFI-Consumer nutzen `fs25_auto_drive_host_bridge` inklusive `Engine*`-/`FlutterBridgeSession`-Kompat-Aliasen; eine separate Flutter-Bridge-Crate existiert nicht mehr

## Verbindlicher Host-Bridge-Session-Vertrag

- `HostBridgeSession` ist die kanonische Session-Surface fuer den egui-Host sowie direkte Flutter-/FFI-Consumer.
- Verbleibende egui-Zugriffe werden in drei Klassen gefuehrt:
	- `bridge-owned`: stabil ueber `HostSessionAction`/`HostSessionSnapshot`/`HostUiSnapshot`/`ViewportOverlaySnapshot`.
	- `bridge-gap`: fuer stabile Host-Aktionen und bridge-owned Read-Seams aktuell geschlossen.
	- `host-local`: dauerhaft host-spezifische Runtime-/Rendering-/Input-Zustaende.
- Datei-/Pfad-Dialog-Lifecycle in egui ist auf die Host-Seam konsolidiert (`take_host_dialog_requests(...)` + `HostSessionAction::SubmitDialogResult`).
- `host_bridge_adapter` enthaelt keine eigene Mapping-Logik mehr; Mapping und Dispatch liegen kanonisch in `fs25_auto_drive_host_bridge::dispatch`.
- Keine neuen Escape-Hatches direkt auf `AppState`/`AppController` fuer neue host-neutrale Fluesse.
- Shared-Texture-v3 bleibt als opaque same-process Runtime-Vertrag bestehen; Texture-Registration-v4 laeuft additiv mit eigener Capability-Negotiation und plattformspezifischen Payload-Familien fuer Windows/Linux/Android.
- Plattform-/Interop-Typen fuer Texture-Registration duerfen nur in `crates/fs25_auto_drive_render_wgpu` und `crates/fs25_auto_drive_host_bridge_ffi` liegen.

## Technologie-Stack

- **UI Framework:** egui
- **Rendering:** wgpu (GPU-basiert, WebGL/Vulkan/Metal)
- **Spatial Index:** kiddo (KD-Tree fuer Nearest-Neighbor)
- **XML:** quick-xml
- **DDS:** image crate (mit DDS-Feature)
- **Assets:** egui_extras (SVG-Support)

## Datenmodell

### MapNode

- ID als Key (`HashMap<u64, MapNode>`)
- `position: Vec2` (x, z — nur 2D, y wird nur beim XML-Export geschrieben)
- `flag: NodeFlag` (Regular, SubPrio, Warning, etc.)
- **Keine** Connections-Liste auf dem Node — Connections leben in `RoadMap.connections`

### Connection

- `start_id: u64` / `end_id: u64`
- `direction: ConnectionDirection` (Regular, Dual, Reverse)
- `priority: ConnectionPriority` (Regular, SubPriority)
- `midpoint: Vec2` / `angle: f32` (berechnete Geometrie)

### RoadMap

- `nodes: HashMap<u64, MapNode>`
- `connections: HashMap<(u64, u64), Connection>` — Key ist `(start_id, end_id)`
- `markers: Vec<MapMarker>`
- `meta: AutoDriveMeta` (Version, Map-Name)
- `spatial: SpatialIndex` (KD-Tree fuer schnelle 2D-Abfragen)

## XML-Format (AutoDrive Config)

**Structure of Arrays:** Parallele CSV-Listen in XML-Tags

- `id`: Comma-separated
- `x, y, z`: Comma-separated
- `out, incoming`: Semicolon-separated (outer), Comma-separated (inner)
- `flags`: Comma-separated

**Versions-Logik:** FS22/FS25 Configs bereinigen Flags 2/4 → 0 beim Laden

## Performance-Ziele

- 100k+ Nodes/Connections fluessig (>60 FPS)
- GPU-Instancing fuer Batch-Rendering
- Viewport-Culling (nur sichtbare Elemente rendern)
- KD-Tree fuer schnelle raeumliche Abfragen (<1ms bei Mausklicks)

## Code-Konventionen

- **Kommunikation:** Deutsch
- **Code/Variablen/Typen:** Englisch
- **Kommentare/Docstrings/README:** Deutsch
- **Fehler-Messages:** Deutsch (User-facing) / Englisch (Debug-Logs)

## Dokumentations-Pflicht

Dokumentation wird **im selben Commit** wie der Code geaendert — nie spaeter:

- **Docstrings (`///`):** Jede oeffentliche Funktion/Struct/Enum braucht einen deutschen Docstring. Bei Signaturnaenderungen sofort anpassen.
- **`crates/*/src/*/API.md`:** Je ein `API.md` pro Layer/Crate-Bereich (`app/`, `core/`, `render/`, `shared/`, `ui/`, `xml/`, `editor_app/`). Aenderungen an der oeffentlichen API → `API.md` sofort nachfuehren.
- **`docs/ROADMAP.md`:** Fertige Items als `[x]` markieren, neue Todos eintragen.
- **`.windsurf/rules/`:** Neue Architektur-Entscheidungen, Layer-Grenzen oder Pattern-Aenderungen hier dokumentieren.
