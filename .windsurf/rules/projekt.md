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
- **DDS:** dds-rs oder image crate
- **Assets:** egui_extras (SVG-Support)

## Datenmodell
### MapNode
- ID als Key (HashMap<u64, MapNode>)
- Position (x, y, z)
- Flags (Regular, Parking, Warning, etc.)
- Connections (Vec<ConnectionId>)

### Connection
- Source/Target Node IDs
- Type (Regular, Dual, Reverse)
- Flags

### RoadMap
- Nodes: HashMap<u64, MapNode>
- Connections: Vec<Connection>
- Metadata (Version, Map Name)

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
