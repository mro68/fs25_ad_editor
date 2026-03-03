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
- `new() → Self` — Erstellt ToolManager mit vorregistrierten Standard-Tools (StraightLine, Bézier Grad 2, Bézier Grad 3, Spline)
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
- `set_last_created(ids, road_map)` / `last_created_ids() → &[u64]` — Erstellte Node-IDs (für Verkettung, road_map dient zur End-Anchor-Ermittlung)
- `last_end_anchor() → Option<ToolAnchor>` — Letzter Endpunkt für Verkettung
- `needs_recreate() → bool` / `clear_recreate_flag()` — Neuberechnung bei Config-Änderung
- `execute_from_anchors(road_map) → Option<ToolResult>` — ToolResult aus gespeicherten Ankern
- `drag_targets() → Vec<Vec2>` — Weltpositionen verschiebbarer Punkte (für Drag-Hit-Test)
- `on_drag_start(pos, road_map, pick_radius) → bool` — Drag auf einen Punkt starten
- `on_drag_update(pos)` — Position des gegriffenen Punkts aktualisieren
- `on_drag_end(road_map)` — Drag beenden (Re-Snap bei Start/Ende)
- `render_context_menu(response) → bool` — Kontextmenü im Viewport rendern (z.B. Tangenten-Auswahl)

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

---

## Gemeinsame Tool-Infrastruktur (`tools/common/`)

Aufgeteilt in vier Submodule (alle privat, Re-Exporte via `common/mod.rs`):

### `geometry.rs`

Hilfsfunktionen: `angle_to_compass`, `node_count_from_length`, `populate_neighbors`, `linear_connections`, `tangent_options`

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
- `save_created_ids(&mut self, ids: &[u64], road_map: &RoadMap)` — Speichert erstellte Node-IDs und ermittelt End-Anker aus der RoadMap
- `has_last_created() → bool` — Prüft ob letzte erstellte IDs vorhanden sind
- `chaining_start_anchor() → Option<ToolAnchor>` — Gibt den End-Anker für die Verkettung zurück, wobei `NewPosition` zu `ExistingNode` hochgestuft wird (verhindert doppelte Nodes am Verkettungspunkt)
- `prepare_for_chaining(&mut lifecycle, &mut seg, &last_anchors)` — Setzt Lifecycle-State und SegmentConfig für die nächste Verkettung zurück (DRY-Hilfsmethode)

**`render_segment_config_3modes(seg, ui, adjusting, ready, length, label, distance_wheel_step_m) → (changed, recreate)`** — Gemeinsame Hilfsfunktion für die 3 SegmentConfig-Darstellungsmodi (Adjusting/Live/Default) inkl. Mausrad-Änderungen für Distanz/Node-Anzahl.

**`impl_lifecycle_delegation!`** — Makro zur Delegation der Standard-RouteTool-Lifecycle-Methoden (`on_deactivate`, `chaining_start_anchor`, `is_adjusting`, `segment_config_mut`) an die gemeinsamen Felder. Eliminiert ~20 Zeilen Boilerplate pro Tool.

### `builder.rs`

**`assemble_tool_result(positions, start, end, direction, priority, road_map) → ToolResult`** — Gemeinsame Logik aller Route-Tools: Nimmt berechnete Positionen, erstellt neue Nodes (überspringt existierende) und baut interne/externe Verbindungen auf.
