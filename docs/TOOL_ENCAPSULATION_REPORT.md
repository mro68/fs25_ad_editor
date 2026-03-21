# Tool Encapsulation & API Unification — Audit-Report

> **Stand:** 2026-03-07  
> **Scope:** `src/app/tools/`, `src/app/group_registry/`, `src/app/handlers/route_tool.rs`  
> **Methodik:** Vollständige Code-Analyse aller 9 Tool-Module + Common-Infrastruktur

---

## Inhaltsverzeichnis

1. [Zusammenfassung](#1-zusammenfassung)
2. [Encapsulation Audit](#2-encapsulation-audit)
3. [API Unification](#3-api-unification)
4. [Editierbarkeits-Analyse](#4-editierbarkeits-analyse)
5. [DRY & Generalisierung](#5-dry--generalisierung)
6. [Performance & Renderer-Trennung](#6-performance--renderer-trennung)
7. [Dokumentation & Konsistenz](#7-dokumentation--konsistenz)
8. [Findings-Tabelle](#8-findings-tabelle)
9. [Code-Vorschläge](#9-code-vorschläge)
10. [Umsetzungsplan](#10-umsetzungsplan)

---

## 1. Zusammenfassung

### Gesamtbewertung: ✅ Gut — mit gezieltem Optimierungspotenzial

Das Tools-System ist **architektonisch solide aufgebaut**:
- **Keine Renderer-Kopplung** — Tools kennen weder `wgpu` noch `RenderScene`
- **Klare Layer-Trennung** — Tools interagieren nur über `&RoadMap` mit dem Domain-Layer
- **Zentralisierte Builder-Logik** — `assemble_tool_result()` wird von 6/9 Tools genutzt
- **Trait-basierte Capabilities** — 4 optionale Traits für Drag/Tangent/Registry/Chain

**Hauptbefunde:**
- 3 Tools (FieldBoundary, Parking, RouteOffset) bauen `ToolResult` manuell statt über `assemble_tool_result()`
- Preview-Allokationen: 2 Tools klonen Caches pro Frame (ConstraintRoute, Bypass)
- Gruppen-Editierbarkeit ist funktional, aber nicht per Trait standardisiert
- Config-UI zeigt hohe Diversität (domain-bedingt, nicht kritisch)

---

## 2. Encapsulation Audit

### 2.1 Layer-Boundary-Verification ✅

| Prüfung | Ergebnis | Evidenz |
|---------|----------|---------|
| `wgpu`-Imports in `tools/` | ✅ Keine | `grep -r "wgpu" src/app/tools/` → 0 |
| `render/`-Imports in `tools/` | ✅ Keine | `grep -r "use.*render" src/app/tools/` → 0 |
| `RenderScene`-Referenzen | ✅ Keine | 0 Treffer |
| GPU-spezifische Typen | ✅ Keine | Keine Vertex-Buffer, Shader, Pipelines |
| `AppState`-Zugriff aus Tools | ✅ Keine | Tools erhalten nur `&RoadMap` |

**Architektur-Konformität:** Die Layer-Grenzen werden vollständig eingehalten.  
Import-Graph: `Tools → core::{RoadMap, MapNode, ConnectionDirection, …}` — korrekt.

### 2.2 Renderer-Agnostische Logik ✅

`ToolPreview` enthält **ausschließlich geometrische Daten**:

```rust
pub struct ToolPreview {
    pub nodes: Vec<Vec2>,           // Reine Positionen
    pub connections: Vec<(usize, usize)>,  // Index-Paare
    pub connection_styles: Vec<(ConnectionDirection, ConnectionPriority)>,  // Domain-Metadaten
}
```

**Keine** Farben, Texturen oder Shader-Referenzen. Die Konvertierung zu visuellen Elementen erfolgt
ausschließlich in `src/ui/tool_preview.rs` (egui-Painter).

### 2.3 Internal State Isolation ✅

- Tool-States (Phasen, Anker, Cache) sind in jedem Tool **privat** (`pub(crate)`)
- Previews erzeugen frische `ToolPreview`-Instanzen (immutable Output)
- Kein geteilter mutabler State zwischen Tools und Renderer

**⚠️ Einzige Einschränkung:** `ConstraintRoute` und `Bypass` klonen interne Caches in `preview()` —
dies ist kein Encapsulation-Problem, sondern ein Performance-Thema (→ Abschnitt 6).

---

## 3. API Unification

### 3.1 Trait-Konsistenz

Das `RouteTool`-Trait definiert **11 Pflichtmethoden + ~20 optionale mit Default-Impls**:

| Methode | Typ | Konsistenz | Anmerkung |
|---------|-----|------------|-----------|
| `name()` | Pflicht | ✅ Einheitlich | Alle liefern `&str` |
| `icon()` | Pflicht | ✅ Einheitlich | Unicode-Symbole |
| `description()` | Pflicht | ✅ Deutsch | Lokalisierte Beschreibungen |
| `status_text()` | Pflicht | ✅ Phasenabhängig | Kontextsensitive Statusmeldungen |
| `on_click()` | Pflicht | ✅ Einheitlich | `→ ToolAction` |
| `preview()` | Pflicht | ✅ Einheitlich | `→ ToolPreview` |
| `execute()` | Pflicht | ⚠️ Divergent | 3 Tools bauen ToolResult manuell |
| `render_config()` | Pflicht | ✅ Domain-bedingt | UI-Diversität ist akzeptabel |
| `reset()` | Pflicht | ✅ Einheitlich | Alle setzen State zurück |
| `is_ready()` | Pflicht | ✅ Einheitlich | Boolean-Check |

### 3.2 ToolResult vs. ToolSegment-Konzept

**Ist-Zustand:** `ToolResult` ist die zentrale Ausgabestruktur:

```rust
pub struct ToolResult {
    pub new_nodes: Vec<(Vec2, NodeFlag)>,
    pub internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)>,
    pub external_connections: Vec<(usize, u64, bool, ConnectionDirection, ConnectionPriority)>,
    pub markers: Vec<(usize, String, String)>,
    pub nodes_to_remove: Vec<u64>,
}
```

**Vergleich mit vorgeschlagenem `ToolSegment`:**

| Aspekt | ToolResult (Ist) | ToolSegment (Vorschlag) |
|--------|-----------------|------------------------|
| Node-Format | `Vec<(Vec2, NodeFlag)>` | `HashMap<u64, MapNode>` |
| Verbindungen | Index-basiert | ID-basiert mit `ConnectionProps` |
| Metadaten | `markers`, `nodes_to_remove` | `SegmentMetadata` |
| ID-Vergabe | Nachgelagert (in `apply_tool_result`) | Im Struct selbst |

**Bewertung:** Die Migration zu `ToolSegment` als HashMap-basiertes Format wäre ein signifikanter
Breaking Change mit **geringem Mehrwert**, da:
- `ToolResult` absichtlich **Index-basiert** ist (IDs werden erst beim Einfügen in `RoadMap` vergeben)
- Die nachgelagerte ID-Vergabe über `apply_tool_result()` ist korrekt und atomisch
- HashMap-Lookup vor ID-Vergabe wäre sinnlos

**→ Empfehlung:** `ToolResult` beibehalten, aber die 3 manuellen Builder vereinheitlichen (→ Abschnitt 5).

### 3.3 Lifecycle-Standardisierung ✅

Der Lifecycle ist bereits gut standardisiert:

```
on_click(pos) → ToolAction::Continue     → weitere Eingabe
             → ToolAction::ReadyToExecute → sofortige Ausführung
             → ToolAction::UpdatePreview  → Vorschau aktualisieren

preview(cursor) → ToolPreview             → reine Geometrie-Daten
execute(road_map) → Option<ToolResult>    → Nodes + Verbindungen
reset()                                   → State zurücksetzen
```

Alle 9 Tools folgen diesem Pattern. Die 4 Capability-Traits (Drag, Tangent, Registry, ChainInput)
erweitern den Lifecycle sauber über optionale Methoden mit Default-Implementierungen.

### 3.4 Capability-Trait-Nutzung

| Capability | Tools | Korrekt genutzt |
|------------|-------|-----------------|
| `RouteToolDrag` | Curve, ConstraintRoute | ✅ |
| `RouteToolTangent` | Curve, Spline, ConstraintRoute | ✅ |
| `RouteToolRegistry` | Alle 9 Tools | ✅ |
| `RouteToolChainInput` | Bypass, RouteOffset | ✅ |

**Keine Re-Implementierung** von Drag/Tangent ohne Trait gefunden.

---

## 4. Editierbarkeits-Analyse

### 4.1 Post-Creation Mutablility

Gruppen werden über `GroupRegistry` verwaltet:

```rust
pub struct GroupRecord {
    pub id: u64,
    pub node_ids: Vec<u64>,
    pub kind: GroupKind,
    pub original_positions: Vec<Vec2>,
    pub marker_node_ids: Vec<u64>,
    pub locked: bool,
}
```

**Editier-Flow:**
1. Tool erzeugt `ToolResult` → `apply_tool_result()` schreibt in `RoadMap`
2. `make_group_record()` speichert Gruppentyp + Parameter in `GroupRegistry`
3. `load_for_edit()` rekonstituiert Tool-State aus gespeichertem `GroupRecord`
4. Erneutes `execute()` überschreibt die Nodes (Undo-Snapshot vorher)

**Status pro Tool:**

| Tool | make_group_record | load_for_edit | Editierbar |
|------|--------------------:|:-------------|:-----------|
| StraightLine | ✅ | ✅ | ✅ Vollständig |
| Curve (Quad/Cubic) | ✅ | ✅ | ✅ Vollständig |
| Spline | ✅ | ✅ | ✅ Vollständig |
| ConstraintRoute | ✅ | ✅ | ✅ Vollständig |
| Bypass | ✅ | ✅ | ✅ Vollständig |
| Parking | ✅ | ✅ | ✅ Vollständig |
| FieldBoundary | ✅ | ✅ | ✅ Vollständig |
| RouteOffset | ✅ | ✅ | ✅ Vollständig |

### 4.2 Mutation Safeguards

- **Gruppen-Lock:** `GroupRecord.locked: bool` verhindert versehentliche Bearbeitung
- **Validierung:** `is_segment_valid()` prüft Node-Existenz und Positionsgleichheit (Toleranz 0.01)
- **Undo-Integration:** `apply_tool_result()` ruft `record_undo_snapshot()` vor jeder Mutation auf
- **Position-Tracking:** `update_original_positions()` synchronisiert nach Move-Operationen

### 4.3 Verbesserungspotenzial

**F-01 — Kein `SegmentEditor`-Trait:**  
Aktuell gibt es keine abstrakte Schnittstelle für Segment-Bearbeitung. Jedes Tool implementiert
`load_for_edit()` individuell. Ein `SegmentEditor`-Trait könnte die Konsistenz erhöhen, ist aber
bei nur 9 Tools kein dringendes Refactoring.

---

## 5. DRY & Generalisierung

### 5.1 Duplikations-Analyse

#### ✅ Bereits gut dedupliziert

| Pattern | Shared Module | Genutzt von |
|---------|--------------|-------------|
| `assemble_tool_result()` | `common/builder.rs` | 6 Tools |
| `ToolLifecycleState` | `common/lifecycle.rs` | 8 Tools |
| `SegmentConfig` | `common/lifecycle.rs` | 4 Tools |
| `TangentState` | `common/tangent.rs` | Curve, Spline |
| `impl_lifecycle_delegation!()` | `common/lifecycle.rs` | 8 Tools |
| `snap_with_neighbors()` | `common/geometry.rs` | 5 Tools |

#### ⚠️ Duplikations-Kandidaten

**D-01 — Manueller ToolResult-Aufbau (3 Tools)**

`FieldBoundaryTool`, `ParkingTool` und `RouteOffsetTool` bauen `ToolResult` manuell,
weil sie spezielle Topologien haben:

| Tool | Grund für manuellen Build | Aufwand Vereinheitlichung |
|------|--------------------------|--------------------------|
| FieldBoundary | Geschlossener Ring (letzter Node → erster) | Mittel |
| Parking | Turnaround-Reihen mit Marker-Integration | Hoch |
| RouteOffset | Links/Rechts-Varianten + `nodes_to_remove` | Mittel |

**Empfehlung:** Anstelle einer Zwangs-Vereinheitlichung eine erweiterte Builder-Variante:

```rust
/// Erweiterte Builder-Optionen für assemble_tool_result
pub struct AssembleOptions {
    /// Schließe die Verbindung (letzter → erster Node)
    pub close_loop: bool,
    /// Nodes, die nach der Erstellung entfernt werden sollen
    pub nodes_to_remove: Vec<u64>,
    /// Marker-Definitionen (Index → Name, Gruppe)
    pub markers: Vec<(usize, String, String)>,
}
```

Damit könnten FieldBoundary und RouteOffset auf `assemble_tool_result_ext()` migriert werden.
Parking bleibt aufgrund der Multi-Reihen-Topologie ein Sonderfall.

**D-02 — Preview-Pattern für Cache-basierte Tools**

`ConstraintRoute`, `Bypass` und `RouteOffset` implementieren ein identisches Cache-Pattern:

```rust
// Pseudo-Pattern (in 3 Tools ähnlich)
fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
    let positions = self.cached_positions.clone();  // ← Frame-Allokation
    let connections = self.cached_connections.clone();
    ToolPreview { nodes: positions, connections, ... }
}
```

**Empfehlung:** Ein `CachedPreview`-Helper, der Clone vermeidet:

```rust
/// Wiederverwendbarer Preview-Cache für Tools mit vorberechneter Geometrie.
pub struct CachedPreview {
    preview: ToolPreview,
    dirty: bool,
}

impl CachedPreview {
    /// Gibt eine Referenz auf den Preview zurück (kein Clone).
    pub fn get_or_rebuild(&mut self, rebuild: impl FnOnce() -> ToolPreview) -> &ToolPreview {
        if self.dirty {
            self.preview = rebuild();
            self.dirty = false;
        }
        &self.preview
    }
}
```

**Hinweis:** Dies erfordert eine Signaturänderung von `preview() → ToolPreview` zu
`preview() → &ToolPreview` oder `Cow<ToolPreview>`, was ein größerer Refactor wäre.
Alternativ: Internes Framing über `Arc<ToolPreview>` mit Clone-on-Write.

**D-03 — Connection-Building Pattern**

`linear_connections()` in `common/geometry.rs` erzeugt `(0,1), (1,2), ..., (n-2, n-1)`,
wird aber nicht von allen Tools genutzt. Einige Tools bauen dieses Pattern manuell nach.

### 5.2 Generische Helpers

**Bereits vorhanden und korrekt genutzt:**
- `angle_to_compass()` — Winkel → Himmelsrichtung (einmalig, kein Duplikat)
- `node_count_from_length()` / `segment_length_from_count()` — Bidirektional
- `populate_neighbors()` — Nachbar-Sammlung aus ToolAnchor
- `parallel_offset()` / `local_perp()` — Geometrie-Operationen

**Kein weiterer Generalisierungsbedarf** bei den bestehenden Hilfsfunktionen.

---

## 6. Performance & Renderer-Trennung

### 6.1 Preview-Allokationen

| Tool | Allokation pro Frame | Schwere |
|------|---------------------|---------|
| StraightLine | 1× `Vec<Vec2>` (klein, N Nodes) | 🟢 Akzeptabel |
| Curve | 1× `Vec<Vec2>` (64-128 Punkte) | 🟢 Akzeptabel |
| Spline | 1× `Vec<Vec2>` (N×density) | 🟢 Akzeptabel |
| ConstraintRoute | `.clone()` auf `preview_positions` + `preview_connections` | 🟡 Vermeidbar |
| Bypass | `Cow<[Vec2]>` + gelegentlicher Clone | 🟡 Vermeidbar |
| Parking | `layout.nodes.clone()` | 🟡 Vermeidbar |
| FieldBoundary | `compute_ring()` + `collect()` | 🟡 Vermeidbar |
| RouteOffset | 3× `Vec` + `extend` | 🟡 Vermeidbar |

**Empfehlung:** Für Tools mit statischem Preview (Cache ändert sich nur bei Parameter-Änderung)
den Preview einmalig berechnen und per `&ToolPreview` zurückgeben. Dies erfordert eine Trait-Anpassung
(s.o. D-02), Priorität: **niedrig** (egui läuft bei 60 FPS, ~16ms Budget ist ausreichend).

### 6.2 Decoupling-Strategien — Ist-Zustand ✅

Die Trennung Tool → App → Render ist bereits **korrekt implementiert**:

```
Tool.preview() → ToolPreview (pure Geometrie)
     ↓
UI: paint_preview() → egui::Painter (2D-Overlay)
     ↓
Renderer: RenderScene (unabhängig, GPU-Pipeline)
```

Tools geben **niemals** Render-spezifische Daten aus. Die Konvertierung zu visuellen Elementen
erfolgt ausschließlich im UI-Layer (`src/ui/tool_preview.rs`).

### 6.3 Benchmark-Empfehlung

Bestehende Benchmarks in `benches/`:
- `render_hotpath_bench.rs` — GPU-Rendering
- `tool_preview_hotpath_bench.rs` — Tool-Preview-Performance

**Empfehlung:** `tool_preview_hotpath_bench.rs` um Cache-basierte Tools erweitern
(ConstraintRoute, Bypass), um Allokations-Overhead zu quantifizieren.

---

## 7. Dokumentation & Konsistenz

### 7.1 Docstring-Audit

| Modul | Pub-Items | Dokumentiert | Status |
|-------|-----------|-------------|--------|
| `tools/mod.rs` | 6 | 6/6 | ✅ Vollständig |
| `tools/route_tool.rs` | 34 | 30/34 | ⚠️ 4 Capability-Methoden ohne Docstring |
| `tools/common/builder.rs` | 1 | 1/1 | ✅ |
| `tools/common/geometry.rs` | 8 | 6/8 | ⚠️ `parallel_offset`, `local_perp` kurz |
| `tools/common/tangent.rs` | 5 | 5/5 | ✅ |
| `tools/common/lifecycle.rs` | 12 | 10/12 | ⚠️ Macro-Varianten |

### 7.2 API.md Status

- `src/app/tools/API.md` — ✅ Vollständig und aktuell (alle 9 Tools, Capabilities, Common-Module)
- `src/app/API.md` — ✅ Aktuell (GroupKind, GroupRecord, AppIntent/AppCommand)

### 7.3 Architektur-Dokumentation

`docs/ARCHITECTURE_PLAN.md` enthält **keine expliziten Regeln** für:
- Tool-Encapsulation (Renderer-Unabhängigkeit)
- Gruppen-Editierbarkeit
- Preview-Performance-Richtlinien

**→ Empfehlung:** Abschnitt "Tool-Encapsulation-Regeln" ergänzen.

---

## 8. Findings-Tabelle

| ID | Schwere | Datei | Bereich | Befund | Empfehlung |
|----|---------|-------|---------|--------|------------|
| **F-01** | 🟡 Mittel | `tools/route_tool.rs` | API | 4 Capability-Trait-Methoden ohne Docstrings | Docstrings ergänzen |
| **F-02** | 🟡 Mittel | `field_boundary/lifecycle.rs` | DRY | Manueller ToolResult-Aufbau statt `assemble_tool_result()` | `assemble_tool_result_ext()` mit `close_loop` |
| **F-03** | 🟡 Mittel | `route_offset/lifecycle.rs` | DRY | Manueller ToolResult-Aufbau | Dito, mit `nodes_to_remove` |
| **F-04** | 🟢 Niedrig | `parking/lifecycle.rs` | DRY | Manueller ToolResult-Aufbau | Sonderfall (Multi-Reihen), belassen |
| **F-05** | 🟢 Niedrig | `constraint_route/state.rs` | Perf | `.clone()` auf Preview-Cache pro Frame | `CachedPreview`-Helper oder `Arc` |
| **F-06** | 🟢 Niedrig | `bypass/lifecycle.rs` | Perf | Clone auf Cached Positions | Dito |
| **F-07** | 🟢 Niedrig | `docs/ARCHITECTURE_PLAN.md` | Doku | Keine Tool-Encapsulation-Regeln | Abschnitt ergänzen |
| **F-08** | 🟢 Niedrig | `common/geometry.rs` | Doku | `parallel_offset()` und `local_perp()` mit Kurz-Docstrings | Erweitern |
| **F-09** | ✅ Info | Alle Tools | API | `execute()` Signatur einheitlich | Bereits korrekt |
| **F-10** | ✅ Info | Alle Tools | Encaps. | Keine Renderer-Kopplung | Kein Handlungsbedarf |

---

## 9. Code-Vorschläge

### 9.1 Erweiterter Builder (F-02, F-03)

```rust
// src/app/tools/common/builder.rs — Neue Funktion

/// Erweiterte Optionen für assemble_tool_result.
#[derive(Default)]
pub struct AssembleOptions {
    /// Schließe die Verbindung (letzter → erster Node).
    pub close_loop: bool,
    /// Nodes, die nach der Erstellung entfernt werden sollen.
    pub nodes_to_remove: Vec<u64>,
    /// Marker-Definitionen (Index → Name, Gruppe).
    pub markers: Vec<(usize, String, String)>,
}

/// Wie `assemble_tool_result`, aber mit erweiterten Optionen.
pub fn assemble_tool_result_ext(
    positions: &[Vec2],
    start: &ToolAnchor,
    end: &ToolAnchor,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
    options: AssembleOptions,
) -> ToolResult {
    let mut result = assemble_tool_result(positions, start, end, direction, priority, road_map);
    
    if options.close_loop && positions.len() >= 2 {
        // Verbindung vom letzten zum ersten Node hinzufügen
        let last_idx = positions.len() - 1;
        result.internal_connections.push((last_idx, 0, direction, priority));
    }
    
    result.nodes_to_remove = options.nodes_to_remove;
    result.markers = options.markers;
    result
}
```

### 9.2 Architektur-Regeln für ARCHITECTURE_PLAN.md (F-07)

```markdown
## Tool-Encapsulation-Regeln

### Verbotene Abhängigkeiten
- Tools (`src/app/tools/`) dürfen **niemals** auf `src/render/`, `wgpu` oder `RenderScene` zugreifen
- Tools erhalten ausschließlich `&RoadMap` (read-only) als Domain-Kontext
- Keine GPU-spezifischen Typen (Vertex-Buffer, Shader, Pipelines) in Tool-Code

### Preview-Vertrag
- `preview()` liefert **reine Geometrie** (`Vec<Vec2>` + Index-basierte Verbindungen)
- Keine Farben, Texturen oder Render-Hints im Preview-Output
- Die Konvertierung zu visuellen Elementen erfolgt im UI-Layer (`src/ui/tool_preview.rs`)

### Gruppen-Editierbarkeit
- Jedes Tool implementiert `make_group_record()` → speichert Konfiguration in `GroupRegistry`
- `load_for_edit()` rekonstituiert den Tool-State für erneute Bearbeitung
- `GroupRecord.locked` verhindert versehentliche Mutation
- Undo-Snapshot wird vor jeder Mutation automatisch erstellt (`apply_tool_result`)
```

### 9.3 Sequenzdiagramm — Tool-Execution-Flow

```
UI (Klick)
  │
  ▼
Handler: route_tool::click()
  │ tool.on_click(pos, &road_map, ctrl)
  │   → ToolAction::ReadyToExecute
  │
  ▼
Handler: execute_and_apply()
  │ 1. tool.execute(&road_map) → ToolResult
  │ 2. tool.make_group_record(id, node_ids) → GroupRecord
  │ 3. apply_tool_result(state, result) → Vec<u64>
  │    ├── state.record_undo_snapshot()
  │    ├── road_map.add_node() × N
  │    ├── road_map.add_connection() × M
  │    └── road_map.ensure_spatial_index()
  │ 4. group_registry.register(record)
  │ 5. tool.set_last_created(new_ids, &road_map)
  │ 6. tool.reset()
  │
  ▼
RoadMap mutiert ← Core-Layer
  │
  ▼
Nächster Frame: RenderScene wird aus RoadMap neu gebaut ← Render-Layer
```

---

## 10. Umsetzungsplan

### Phase 1: Dokumentation (niedrige Disruption) — Commit 1

- [ ] `docs/ARCHITECTURE_PLAN.md` — Abschnitt "Tool-Encapsulation-Regeln" ergänzen
- [ ] `src/app/tools/route_tool.rs` — 4 fehlende Docstrings für Capability-Methoden
- [ ] `src/app/tools/common/geometry.rs` — Docstrings für `parallel_offset()`, `local_perp()`

### Phase 2: Builder-Erweiterung (mittel) — Commit 2

- [ ] `assemble_tool_result_ext()` in `common/builder.rs` implementieren
- [ ] `FieldBoundaryTool::execute()` auf `assemble_tool_result_ext(close_loop: true)` migrieren
- [ ] `RouteOffsetTool::execute()` auf `assemble_tool_result_ext(nodes_to_remove: ...)` migrieren
- [ ] Tests für erweiterten Builder schreiben

### Phase 3: Preview-Optimierung (optional, niedrig) — Commit 3

- [ ] Benchmark für Cache-basierte Tools erstellen bzw. erweitern
- [ ] Entscheidung: `Arc<ToolPreview>` vs. Signaturänderung basierend auf Messdaten
- [ ] Bei Bedarf: CachedPreview-Helper implementieren

### Nicht im Scope

- **ToolResult → ToolSegment Migration:** Kein Mehrwert (s. Abschnitt 3.2)
- **Config-UI-Vereinheitlichung:** Domain-bedingte Diversität ist akzeptabel
- **ParkingTool auf Builder migrieren:** Zu komplex für den Nutzen

---

## Anhang: Architektur-Diagramm

```
┌────────────────────────────────────────────────────────────────┐
│                        UI Layer                                 │
│  ┌──────────────┐  ┌──────────────────┐  ┌──────────────────┐ │
│  │ tool_panel.rs │  │ tool_preview.rs  │  │ context_menu.rs  │ │
│  │  render_config│  │  paint_preview() │  │                  │ │
│  └──────┬───────┘  └────────┬─────────┘  └──────────────────┘ │
│         │ AppIntent          │ ToolPreview                      │
├─────────┼───────────────────┼──────────────────────────────────┤
│         ▼                   │         App Layer                 │
│  ┌──────────────┐           │                                   │
│  │  Controller  │           │                                   │
│  │  ┌────────┐  │           │                                   │
│  │  │Handler │──┼───────────┤                                   │
│  │  └────────┘  │           │                                   │
│  └──────┬───────┘           │                                   │
│         │                   │                                   │
│  ┌──────▼───────────────────┴──────────────────────────────┐   │
│  │                    Tools System                          │   │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐           │   │
│  │  │Straight│ │ Curve  │ │ Spline │ │Parking │ ... (9)   │   │
│  │  │  Line  │ │Q/C     │ │        │ │        │           │   │
│  │  └────┬───┘ └────┬───┘ └────┬───┘ └────┬───┘           │   │
│  │       └──────────┼──────────┼──────────┘                │   │
│  │                  ▼                                       │   │
│  │  ┌──────────────────────────────────────────────┐       │   │
│  │  │              common/                          │       │   │
│  │  │  builder.rs │ lifecycle.rs │ tangent.rs │ …  │       │   │
│  │  └──────────────────────────────────────────────┘       │   │
│  └─────────────────────┬───────────────────────────────────┘   │
│                        │ ToolResult                             │
│  ┌─────────────────────▼───────────────────────────────────┐   │
│  │  apply_tool_result() → use_cases/editing                │   │
│  │  GroupRegistry → group_registry/                    │   │
│  └─────────────────────┬───────────────────────────────────┘   │
├─────────────────────────┼──────────────────────────────────────┤
│                        ▼         Core Layer                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  RoadMap │ MapNode │ Connection │ SpatialIndex (kiddo)   │  │
│  └──────────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────┤
│                        Render Layer (unabhängig)                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  RenderScene ← App baut aus RoadMap │ wgpu-Pipeline      │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```
