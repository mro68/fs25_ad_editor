# Projekt: FS25 AutoDrive Editor (RADE)

## Überblick
Neuimplementierung des AutoDrive Course Editors in Rust mit egui + wgpu. Hochperformantes Tool für 100k+ Wegpunkte mit Cross-Platform-Fähigkeit.

## Architektur
- **Core:** Datenmodelle (RoadMap, MapNode), XML-IO, Algorithmen
- **Render:** wgpu-Pipeline mit Instancing, Culling, Viewport-Management
- **App:** Controller + `AppIntent`/`AppCommand`-Flow, Use-Cases, State-Management
- **UI:** egui-Interface, emittiert nur `AppIntent` (keine direkten Domain-Mutationen)

## Event- und Mutationsfluss
- UI erzeugt `AppIntent`
- `AppController` mappt auf `AppCommand`
- Commands werden über Use-Cases ausgeführt
- Mutationen laufen zentral gegen `AppState`
- Renderer bekommt ausschließlich `RenderScene`

## Technologie-Stack
- **UI Framework:** egui
- **Rendering:** wgpu (GPU-basiert, WebGL/Vulkan/Metal)
- **Spatial Index:** kiddo (KD-Tree für Nearest-Neighbor)
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
- `spatial: SpatialIndex` (KD-Tree für schnelle 2D-Abfragen)

## XML-Format (AutoDrive Config)
**Structure of Arrays:** Parallele CSV-Listen in XML-Tags
- `id`: Comma-separated
- `x, y, z`: Comma-separated
- `out, incoming`: Semicolon-separated (outer), Comma-separated (inner)
- `flags`: Comma-separated

**Versions-Logik:** FS22/FS25 Configs bereinigen Flags 2/4 → 0 beim Laden

## Performance-Ziele
- 100k+ Nodes/Connections flüssig (>60 FPS)
- GPU-Instancing für Batch-Rendering
- Viewport-Culling (nur sichtbare Elemente rendern)
- KD-Tree für schnelle räumliche Abfragen (<1ms bei Mausklicks)

## Code-Konventionen
- **Kommunikation:** Deutsch
- **Code/Variablen/Typen:** Englisch
- **Kommentare/Docstrings/README:** Deutsch
- **Fehler-Messages:** Deutsch (User-facing) / Englisch (Debug-Logs)

## Dokumentations-Pflicht
Dokumentation wird **im selben Commit** wie der Code geändert — nie später:
- **Docstrings (`///`):** Jede öffentliche Funktion/Struct/Enum braucht einen deutschen Docstring. Bei Signaturnänderungen sofort anpassen.
- **`src/*/API.md`:** Je ein `API.md` pro Layer (`app/`, `core/`, `render/`, `shared/`, `xml/`). Änderungen an der öffentlichen API → `API.md` sofort nachführen.
- **`docs/ROADMAP.md`:** Fertige Items als `[x]` markieren, neue Todos eintragen.
- **`.windsurf/rules/`:** Neue Architektur-Entscheidungen, Layer-Grenzen oder Pattern-Änderungen hier dokumentieren.
