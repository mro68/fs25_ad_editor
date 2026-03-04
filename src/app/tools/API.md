# Tools API

Dokumentation für `app::tools`: `ToolManager`, `RouteTool`-Trait, registrierte Tools und gemeinsame Infrastruktur.

**Zurück:** [`../API.md`](../API.md)

---

## `ToolManager`

Verwaltet registrierte Route-Tools und den aktiven Tool-Index.

```rust
pub struct ToolManager { /* intern */ }
```

**Methoden:**
- `new() → Self` — Erstellt ToolManager mit vorregistrierten Standard-Tools (StraightLine, Bézier Grad 2, Bézier Grad 3, Spline, Bypass, ConstraintRoute)
- `register(tool)` — Neues Route-Tool registrieren
- `tool_count() → usize` — Anzahl registrierter Tools
- `tool_names() → Vec<(usize, &str)>` — Name + Index aller Tools
- `tool_entries() → Vec<(usize, &str, &str)>` — Index, Name und Icon aller Tools (für Dropdown-Rendering)
- `set_active(index)` — Aktives Tool setzen (Reset des vorherigen)
- `active_index() → Option<usize>` — Index des aktiven Tools
- `active_tool() → Option<&dyn RouteTool>` — Referenz auf aktives Tool
- `active_tool_mut() → Option<&mut dyn RouteTool>` — Mutable Referenz
- `reset()` — Alle Tools zurücksetzen, aktives deaktivieren

---

## `RouteTool` (Trait)

Schnittstelle für alle Route-Tools (Linie, Kurve, …). Tools sind zustandsbehaftet und erzeugen Preview-Geometrie + `ToolResult`.

**Pflicht-Methoden:**
- `name() → &str` — Anzeigename
- `icon() → &str` — Icon-Zeichen für das Dropdown (rechts vom Label); Default: `""`
- `description() → &str` — Tooltip-Text
- `status_text() → &str` — Statustext für Properties-Panel
- `on_click(pos, road_map, ctrl) → ToolAction` — Viewport-Klick verarbeiten
- `preview(cursor_pos, road_map) → ToolPreview` — Preview-Geometrie berechnen
- `render_config(ui, distance_wheel_step_m) → bool` — Tool-Konfiguration im Properties-Panel (inkl. Mausrad-Schrittweite für Distanz-Felder)
- `execute(road_map) → Option<ToolResult>` — Ergebnis erzeugen
- `reset()` — Tool-Zustand zurücksetzen
- `is_ready() → bool` — Bereit zur Ausführung?

**Optionale Methoden (Default-Implementierung):**
- `has_pending_input() → bool` — Hat das Tool angefangene Eingaben? (für stufenweise Escape-Logik)
- `set_direction(dir)` / `set_priority(prio)` — Editor-Defaults übernehmen
- `set_snap_radius(radius)` — Snap-Radius für Node-Snapping setzen
- `set_last_created(ids, road_map)` / `last_created_ids() → &[u64]` — Erstellte Node-IDs (für Verkettung und Nachbearbeitung)
- `current_end_anchor() → Option<ToolAnchor>` — Liefert den End-Anker für den gemeinsamen `set_last_created`-Flow
- `save_anchors_for_recreate(road_map)` — Speichert tool-spezifische Recreate-Daten
- `last_end_anchor() → Option<ToolAnchor>` — Letzter Endpunkt für Verkettung
- `needs_recreate() → bool` / `clear_recreate_flag()` — Neuberechnung bei Config-Änderung
- `execute_from_anchors(road_map) → Option<ToolResult>` — ToolResult aus gespeicherten Ankern
- `drag_targets() → Vec<Vec2>` — Weltpositionen verschiebbarer Punkte (für Drag-Hit-Test)
- `on_drag_start(pos, road_map, pick_radius) → bool` — Drag auf einen Punkt starten
- `on_drag_update(pos)` — Position des gegriffenen Punkts aktualisieren
- `on_drag_end(road_map)` — Drag beenden (Re-Snap bei Start/Ende)
- `tangent_menu_data() → Option<TangentMenuData>` — liefert Tangenten-Menüdaten für das Kontextmenü
- `apply_tangent_selection(start, end)` — wendet die im Kontextmenü gewählten Tangenten an

### Capability-Traits

Der optionale Teil wurde zusätzlich in Capability-Traits gekapselt. `RouteTool` bleibt als kompatibler Obervertrag bestehen und delegiert seine Default-Implementierungen an diese Traits:

- `RouteToolDrag` — `drag_targets`, `on_drag_start`, `on_drag_update`, `on_drag_end`
- `RouteToolTangent` — `tangent_menu_data`, `apply_tangent_selection`
- `RouteToolRegistry` — `make_segment_record`, `load_for_edit`
- `RouteToolChainInput` — `needs_chain_input`, `load_chain`

**Registry-Erweiterungen** (für `SegmentRegistry`, siehe [`../use_cases/API.md`](../use_cases/API.md)):
```rust
// Wird nach execute() + apply_tool_result() aufgerufen:
fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord>;

// Wird in edit_segment() aufgerufen um das Tool wiederherzustellen:
fn load_for_edit(&mut self, record: &SegmentRecord, kind: &SegmentKind);
```

---

## Registrierte Tools

| Idx | Tool | Icon | Konstruktor |
|-----|------|------|-------------|
| 0 | `StraightLineTool` | `━` | `StraightLineTool::new()` |
| 1 | `CurveTool` (Grad 2) | `⌒` | `CurveTool::new()` |
| 2 | `CurveTool` (Grad 3) | `〜` | `CurveTool::new_cubic()` |
| 3 | `SplineTool` | `〰` | `SplineTool::new()` |
| 4 | `BypassTool` | `⤴` | `BypassTool::new()` |
| 5 | `ConstraintRouteTool` | `⊿` | `ConstraintRouteTool::new()` |

### `StraightLineTool`

Gerade Strecke zwischen zwei Punkten mit konfigurierbarem Nodeabstand.

### `CurveTool`

Bézier-Kurve wahlweise Grad 2 (quadratisch, 1 Steuerpunkt) oder Grad 3 (kubisch, 2 Steuerpunkte). `name()` und `description()` sind grad-spezifisch.

Konstruktoren: `CurveTool::new()` (Grad 2), `CurveTool::new_cubic()` (Grad 3).

Modulstruktur: `state.rs` (Enums, Struct, Ctors), `lifecycle.rs` (RouteTool-Impl), `drag.rs` (Drag-Logik), `config_ui.rs` (egui-Panel), `geometry.rs` (Bézier-Mathe), `tests.rs`

**Cubic-Extras (Grad 3):**
- **Auto-Tangente:** Beim Eintreten in Phase::Control wird automatisch der beste Start-Nachbar gewählt (bevorzugt eingehende Verbindungen; max `dot(continuation_dir, chord_dir)`). CP1 und CP2 werden sofort gesetzt.
- **Virtueller Scheitelpunkt (Apex):** `virtual_apex = B(0.5)` wird als fünftes Drag-Handle angeboten. Drag verschiebt B(0.5) und passt CP1/CP2 via inverser Bézier-Formel an:
  - Mit Start-Tangente: CP1 fixiert → nur CP2 = `(8·apex − P0 − 3·CP1 − P3) / 3`
  - Mit End-Tangente: CP2 fixiert → nur CP1 = `(8·apex − P0 − 3·CP2 − P3) / 3`
  - Ohne Tangente: beide CPs symmetrisch aus Apex (`cps_from_apex_symmetric`)

### `SplineTool`

Catmull-Rom-Spline: interpolierende Kurve durch alle geklickten Punkte. Beliebig viele Kontrollpunkte, fortlaufende Vorschau (Cursor als nächster Punkt), Enter bestätigt. Nachbearbeitung (Segment-Länge/Node-Anzahl) und Verkettung unterstützt.

### `ConstraintRouteTool`

Winkelgeglättete Route mit automatischen Tangenten-Übergängen. Solver-Pipeline:
1. **Approach-Steerer:** Auto-Steuerpunkt am Start (Dot-Product-basierte Nachbar-Auswahl, dynamischer Abstand)
2. **User-Kontrollpunkte:** Beliebig viele Zwischen-Kontrollpunkte (Phase::ControlNodes)
3. **Departure-Steerer:** Analog am Ende
4. **Subdivide:** Gleichmäßige Unterteilung aller Segmente auf `max_segment_length`
5. **Chaikin-Corner-Cutting:** Iteratives Glätten der schärfsten Ecke (mit Re-Subdivision nach jedem Cut)
6. **Resampling:** Finale Punkte auf gleichmäßige Abstände

**Steuerpunkte (Steerer):**
- Automatisch berechnet aus Nachbar-Richtungen (`connected_neighbors`)
- Im UI als Steuerpunkte angezeigt (Config-Panel + Viewport)
- Per Drag im Viewport verschiebbar (wird dann als manuell markiert)
- Reset-Button (↺) im Config-Panel setzt auf Auto-Berechnung zurück
- Manuell verschobene Steuerpunkte werden als Kontrollpunkte an den Solver übergeben

**Solver-Typen:**
- `SolverResult` — Positionen + optionale Approach/Departure-Steuerpunkte
- `ConstraintRouteInput` — Eingabeparameter für den Solver
- Öffentliche Helper-Exporte: `constraint_route::solve_route`, `constraint_route::ConstraintRouteInput` (u.a. für Benchmarks)

**Solver-Parameter:**
- `max_angle_deg: f32` (5°..135°) — Maximale Richtungsänderung pro Segment
- `max_segment_length: f32` — via `SegmentConfig`

**Drag-Targets:** Start, End, ApproachSteerer, DepartureSteerer, Control(i)

**Phasen:** `Start` → `End` → `ControlNodes` (Enter bestätigt)

Modulstruktur: `state.rs`, `lifecycle.rs`, `geometry.rs`, `drag.rs`, `config_ui.rs`, `tests.rs`

---

## Gemeinsame Tool-Infrastruktur (`tools/common/`)

Aufgeteilt in vier Submodule (alle privat, Re-Exporte via `common/mod.rs`):

### `geometry.rs`

Hilfsfunktionen: `angle_to_compass`, `node_count_from_length`, `populate_neighbors`, `snap_with_neighbors`, `linear_connections`, `tangent_options`

### `tangent.rs`

**`TangentSource`** — Tangenten-Quelle am Start-/Endpunkt (für Curve + Spline):
- `None` — Kein Tangenten-Vorschlag
- `Connection { neighbor_id, angle }` — Tangente aus bestehender Verbindung

**`render_tangent_combo(ui, id_salt, label, none_label, current, neighbors) → bool`** — Gemeinsamer UI-Baustein für Tangenten-ComboBoxen (verwendet von Curve + Spline config_ui).

### `lifecycle.rs`

**`ToolLifecycleState`**, **`SegmentConfig`**, **`LastEdited`**

**`SegmentConfig`** — Gekapselte Konfiguration für Segment-Länge und Node-Anzahl:
- `max_segment_length: f32` — Maximaler Abstand zwischen Zwischen-Nodes
- `node_count: usize` — Gewünschte Anzahl Nodes (inkl. Start+End)
- `last_edited: LastEdited` — Welcher Wert zuletzt geändert wurde (bestimmt Sync-Richtung)
- `sync_from_length(length)` — Synchronisiert abhängigen Wert aus Streckenlänge

**`ToolLifecycleState`-Methoden:**
- `save_created_ids(&mut self, ids: &[u64])` — Speichert erstellte Node-IDs und setzt das Recreate-Flag zurück
- `has_last_created() → bool` — Prüft ob letzte erstellte IDs vorhanden sind
- `chaining_start_anchor() → Option<ToolAnchor>` — Gibt den End-Anker für die Verkettung zurück, wobei `NewPosition` zu `ExistingNode` hochgestuft wird (verhindert doppelte Nodes am Verkettungspunkt)
- `prepare_for_chaining()` — Setzt den Lifecycle-State für die nächste Verkettung zurück (DRY-Hilfsmethode)

**`render_segment_config_3modes(seg, ui, adjusting, ready, length, label, distance_wheel_step_m) → (changed, recreate)`** — Gemeinsame Hilfsfunktion für die 3 SegmentConfig-Darstellungsmodi (Adjusting/Live/Default) inkl. Mausrad-Änderungen für Distanz/Node-Anzahl.

**`impl_lifecycle_delegation!`** — Makro zur Delegation der Standard-RouteTool-Lifecycle-Methoden (`set_direction`, `set_priority`, `set_snap_radius`, `last_created_ids`, `last_end_anchor`, `needs_recreate`, `clear_recreate_flag`, Inkrement/Decrement-Helfer) an die gemeinsamen Felder. Eliminiert Boilerplate pro Tool.

Der Makro-Flow für `set_last_created` ist vereinheitlicht:
1. `current_end_anchor()` übernehmen
2. `save_anchors_for_recreate(road_map)` aufrufen
3. `lifecycle.save_created_ids(ids)` ausführen

### `bypass::geometry` Export

- Öffentlicher Helper-Export: `bypass::compute_bypass_positions` (u.a. für Preview-Benchmarks)

### `builder.rs`

**`assemble_tool_result(positions, start, end, direction, priority, road_map) → ToolResult`** — Gemeinsame Logik aller Route-Tools: Nimmt berechnete Positionen, erstellt neue Nodes (überspringt existierende) und baut interne/externe Verbindungen auf.

`ToolResult.external_connections` kodiert externe Kanten als
`(new_node_idx, existing_node_id, existing_to_new, direction, priority)`.
Damit bleibt die Richtung (`Regular`/`Dual`/`Reverse`) an Start- und Endrand konsistent,
ohne implizite Richtungs-Spiegelung.
