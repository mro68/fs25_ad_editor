# Vorschlag: Einheitliches Kontextmenü-System

## Zielsetzung
**Alle Befehle über das Rechtsklick-Kontextmenü erreichbar machen** — kontextabhängig, aber mit konsistenter Struktur über alle Tools hinweg.

---

## Aktueller Zustand (Analyse)

### Bestehende Context-Menus

1. **≥2 Nodes selektiert + Verbindungen** ([src/ui/context_menu.rs](../src/ui/context_menu.rs#L15))
   - Nodes verbinden (bei 2 ohne Verbindung)
   - Richtung ändern (Regular/Dual/Reverse/Invertieren)
   - Straßenart (Haupt-/Nebenstraße)
   - Alle trennen
   - Streckenteilung (mit Live-Steuerung wenn aktiv)

2. **Einzelner Node (1 selektiert)** ([src/ui/context_menu.rs](../src/ui/context_menu.rs#L182))
   - Node-ID-Label
   - Marker erstellen/ändern/löschen

3. **Route-Tool Control-Phase** (Tool-intern, `render_context_menu()`)
   - Tangenten-Auswahl (Cubic/Spline)

### Was fehlt?

- **Tool-Wechsel** (Select/Connect/AddNode/Route)
- **Node-Operationen**: Löschen, Duplizieren
- **Quick-Shortcuts**: Connect (C), Disconnect (X), Select All, Clear
- **File-Operationen**: Open, Save, Save As
- **Undo/Redo** (Verlauf)
- **View**: Zoom-Presets, Kamera zurücksetzen
- **Rechtsklick auf leerem Bereich** (keine Nodes in der Nähe)
- **Route-Tool**: Execute, Recreate, Cancel

---

## Vorschlag: 5 kontextabhängige Menü-Varianten

### **1. LEERER BEREICH** (kein Node in der Nähe, keine Selektion)

```
┌────────────────────────────────┐
│ 🗺 Datei                       │
│   ├─ Öffnen...        Ctrl+O  │
│   ├─ Speichern       Ctrl+S   │
│   └─ Speichern als...         │
├────────────────────────────────┤
│ 🛠 Werkzeug                     │
│   ├─ ⭘ Auswahl (1)            │  ← aktuelles Tool ✓
│   ├─ ⚡ Verbinden (2)          │
│   ├─ ➕ Node hinzufügen (3)    │
│   └─ ➤ Strecke erstellen... ▶ │  → Dropdown: Gerade/Quad/Cubic/Spline
├────────────────────────────────┤
│ 📐 Selektion                   │
│   ├─ Alle auswählen   Ctrl+A  │
│   └─ Auswahl löschen  Esc     │
├────────────────────────────────┤
│ 🔍 Ansicht                     │
│   ├─ Zoom anpassen (1:1)      │
│   ├─ Alles einpassen          │
│   └─ Kamera zurücksetzen      │
├────────────────────────────────┤
│ ↶ Rückgängig         Ctrl+Z   │
│ ↷ Wiederholen        Ctrl+Y   │
└────────────────────────────────┘
```

**Kontext:** User will schnell Tool wechseln oder File-Operationen durchführen.

---

### **2. EINZELNER NODE** (nicht selektiert, Pointer auf Node)

```
┌────────────────────────────────┐
│ Node #42                       │
│ Position: (1234.5, 678.9)     │
│ Verbindungen: 5 (↦3 ↤2)       │  ← Info-Header
├────────────────────────────────┤
│ ✓ Selektieren                  │
│ ⬚ Zur Selektion hinzufügen     │
├────────────────────────────────┤
│ 🗺 Marker                      │
│   ├─ Erstellen...              │  (bei noch keinem Marker)
│   ├─ Bearbeiten...   ✏        │  (bei bestehendem)
│   └─ Löschen         ✕        │
├────────────────────────────────┤
│ ✂ Löschen            Del      │
└────────────────────────────────┘
```

**Verhalten:**
- Klick auf "Selektieren" → Node wird selektiert, Menü schließt nicht (damit sofort weitere Optionen erscheinen)
- Optional: Nach Selektion automatisch **Variante 3** anzeigen

---

### **3. EINZELNER NODE** (selektiert)

```
┌────────────────────────────────┐
│ Node #42 ✓                     │
│ Position: (1234.5, 678.9)     │
│ Verbindungen: 5 (↦3 ↤2)       │
├────────────────────────────────┤
│ ⬚ Von Selektion entfernen      │
├────────────────────────────────┤
│ 🗺 Marker                      │
│   ├─ Erstellen...              │
│   ├─ Bearbeiten...   ✏        │
│   └─ Löschen         ✕        │
├────────────────────────────────┤
│ ✂ Löschen            Del      │
│ ⧉ Duplizieren                  │  ← neu: Node+Verbindungen kopieren
├────────────────────────────────┤
│ 🛠 Werkzeug wechseln... ▶      │
│ ↶ Rückgängig         Ctrl+Z   │
└────────────────────────────────┘
```

---

### **4. MEHRERE NODES** (≥2 selektiert)

```
┌────────────────────────────────┐
│ 5 Nodes selektiert             │
│ 3 Verbindungen zwischen ihnen │
├────────────────────────────────┤
│ 🔗  Verbindungen               │   ← Submenu
│   ├─ Nodes verbinden (C)       │   (nur bei 2 Nodes ohne Verbindung)
│   ├─ Richtung: Regular  ▶      │
│   │   ├─ ↦ Regular (Einbahn)  │
│   │   ├─ ⇆ Dual (beidseitig)  │
│   │   ├─ ↤ Reverse             │
│   │   └─ ⇄ Invertieren         │
│   ├─ Straßenart ▶              │
│   │   ├─ 🛣 Hauptstraße        │
│   │   └─ 🛤 Nebenstraße        │
│   └─ ✕ Alle trennen (X)       │
├────────────────────────────────┤
│ ✂ Streckenteilung              │   ← Direkter Eintrag (kein Submenu)
│   ├─ Aktivieren                │   (wenn noch nicht aktiv)
│   └─ [Live-Steuerung]          │   (wenn aktiv: Abstand/Nodes/Übernehmen/Verwerfen)
├────────────────────────────────┤
│ 📐 Selektion                   │
│   ├─ Alle auswählen   Ctrl+A  │
│   ├─ Auswahl löschen  Esc     │
│   └─ Invertieren               │
├────────────────────────────────┤
│ ✂ Löschen            Del      │
│ ⧉ Duplizieren                  │
├────────────────────────────────┤
│ 🛠 Werkzeug wechseln... ▶      │
│ ↶ Rückgängig         Ctrl+Z   │
└────────────────────────────────┘
```

**Hierarchie-Optionen:**
- **Flach** (alle Richtung-Buttons direkt) — aktuell umgesetzt
- **Submenu** (wie oben) — sauberer, aber 1 Klick mehr

**Empfehlung:** **Submenu** für Verbindungen, weil das Menü sonst zu lang wird.

---

### **5. ROUTE-TOOL AKTIV** (mit pending input)

Wenn ein Route-Tool aktiv ist (StraightLine/Curve/Spline) UND bereits Input vorhanden (1+ Anchors):

```
┌────────────────────────────────┐
│ 〜 Kubische Kurve              │  ← aktuelles Tool
│ Phase: Kontrollpunkte setzen  │
├────────────────────────────────┤
│ ✓ Ausführen          Enter    │
│ 🔄 Neu berechnen (Recreate)   │  ← bei needs_recreate
│ ✕ Abbrechen          Esc      │
├────────────────────────────────┤
│ ⚙ Tangenten (CP2)... ▶         │  ← nur bei Cubic/Spline
│   ├─ Manuell                   │
│   ├─ → Node #42 (NO)          │
│   └─ → Node #17 (SW)          │
├────────────────────────────────┤
│ Node-Abstand: [====◯====] 6m  │  ← Slider inline
│ Richtung: [Dual ▾]            │
│ Straßenart: [Haupt ▾]         │
├────────────────────────────────┤
│ 🛠 Werkzeug wechseln... ▶      │
│ ↶ Rückgängig         Ctrl+Z   │
└────────────────────────────────┘
```

**Besonderheit:**
- Tangenten-Selector bleibt (wie aktuell)
- Zusätzliche Shortcuts: Execute, Recreate, Cancel
- Config-Elemente inline (Slider, Dropdowns)

**Alternative:** Config im Properties-Panel belassen, nur Aktionen im Menu.

---

## Implementierungs-Plan

### Phase 1: Context-Menu-System erweitern

**Neue Funktion:** `show_viewport_context_menu()` (Haupt-Router)

```rust
pub(super) fn show_viewport_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    active_tool: EditorTool,
    route_tool_manager: Option<&mut ToolManager>,
    distanzen_state: &mut DistanzenState,
    pointer_pos_world: Option<glam::Vec2>,
    events: &mut Vec<AppIntent>,
) {
    // 1. Bestimme Kontext:
    let hovered_node_id = find_nearest_node_at(pointer_pos_world, road_map);
    
    let menu_variant = match (selected_node_ids.len(), hovered_node_id, active_tool) {
        (0, None, EditorTool::Route) if route_tool_has_input => MenuVariant::RouteToolActive,
        (0, None, _) => MenuVariant::EmptyArea,
        (0, Some(id), _) => MenuVariant::SingleNodeUnselected(id),
        (1, Some(id), _) if selected_node_ids.contains(&id) => MenuVariant::SingleNodeSelected(id),
        (n, _, _) if n >= 2 => MenuVariant::MultipleNodesSelected,
        _ => MenuVariant::EmptyArea,
    };
    
    // 2. Render passendes Menü:
    response.context_menu(|ui| {
        match menu_variant {
            MenuVariant::EmptyArea => render_empty_area_menu(ui, events),
            MenuVariant::SingleNodeUnselected(id) => render_single_node_menu(ui, id, false, road_map, events),
            // ...
        }
    });
}
```

**Aufrufort:** `src/ui/input/mod.rs` → ersetzt bestehende `show_connection_context_menu` + `show_node_marker_context_menu`.

---

### Phase 2: Neue AppIntents hinzufügen

Fehlende Intents:
```rust
// Tools
SetEditorToolRequested { tool: EditorTool },  // existiert bereits
SetRouteSubToolRequested { index: usize },    // neu

// Selektion
SelectNodeRequested { node_id: u64 },         // neu: einzelnen Node auswählen
AddToSelectionRequested { node_id: u64 },     // neu: additiv
RemoveFromSelectionRequested { node_id: u64 }, // neu
InvertSelectionRequested,                      // neu

// Node-Ops
DuplicateSelectedNodesRequested,               // neu

// Route-Tool
RouteToolExecuteRequested,                     // bereits als RouteToolExecuteClicked
RouteToolRecreateRequested,                    // bereits vorhanden
RouteToolCancelRequested,                      // bereits vorhanden

// View
ZoomToFitRequested,                           // neu
ResetCameraRequested,                         // neu: auf (0,0), Zoom 1.0
```

---

### Phase 3: Properties-Panel vs. Context-Menu (Design-Entscheidung)

**Option A: Duplikation erlauben**
- Slider/Dropdowns sowohl im Properties-Panel als auch im Context-Menu
- Vorteil: maximaler Komfort (User kann wählen)
- Nachteil: Code-Duplikation, Sync-Logik

**Option B: Context-Menu nur Aktionen**
- Config bleibt im Properties-Panel
- Context-Menu: nur Shortcuts/Commands
- Vorteil: klare Trennung, kein Duplikat-Code
- Nachteil: User muss manchmal Panel öffnen

**Empfehlung:** **Option B** — Context-Menu für Aktionen, Properties-Panel für Live-Config.

Ausnahme: **Streckenteilung** (bereits umgesetzt) — weil es ein temporärer Modus ist, passt Live-Steuerung im Menu.

---

### Phase 4: Submenu vs. Flat (Design-Entscheidung)

**Aktuell:** Alle Verbindungs-Optionen flach im Menu (12+ Buttons).

**Vorschlag:** Hierarchie:
```
🔗 Verbindungen ▶
   ├─ Richtung ▶
   │   ├─ Regular
   │   └─ ...
   ├─ Straßenart ▶
   │   └─ ...
   └─ Alle trennen
```

**Vorteile:** Übersichtlicher, weniger Scroll.  
**Nachteile:** Mehr Hover/Klicks (UX-Trade-off).

**Empfehlung:** **Submenu** für Verbindungen, aber mit **Icons** und Sprechblasen-Tooltip (egui `on_hover_text()`).

---

## Keyboard-Shortcut-Überblick (Referenz)

| Aktion | Shortcut | Menu-Pfad |
|--------|----------|-----------|
| **Tool-Wechsel** |
| Select | `1` | Werkzeug → Auswahl |
| Connect | `2` | Werkzeug → Verbinden |
| AddNode | `3` | Werkzeug → Node hinzufügen |
| **Selektion** |
| Select All | `Ctrl+A` | Selektion → Alle auswählen |
| Clear | `Esc` | Selektion → Auswahl löschen |
| **Editing** |
| Delete | `Del`, `Backspace` | [Node-Menu] → Löschen |
| Connect (Quick) | `C` (2 sel) | Verbindungen → Nodes verbinden |
| Disconnect (Quick) | `X` (2 sel) | Verbindungen → Alle trennen |
| Undo | `Ctrl+Z` | Rückgängig |
| Redo | `Ctrl+Y` | Wiederholen |
| **Route-Tool** |
| Execute | `Enter` | Ausführen |
| Cancel | `Esc` | Abbrechen |
| **File** |
| Open | `Ctrl+O` | Datei → Öffnen |
| Save | `Ctrl+S` | Datei → Speichern |

Alle Shortcuts sollten auch im Menu angezeigt werden (aktuell nur teilweise der Fall).

---

## Nächste Schritte

1. **Diskussion:** Welche Variante bevorzugst du?
   - Flat vs. Submenu für Verbindungen?
   - Properties im Menu oder nur Aktionen?
   - Alle 5 Varianten umsetzen oder pragmatisch reduzieren?

2. **Prototyp:** Kleine Implementierung für **Variante 1 (Leerer Bereich)** als Proof-of-Concept.

3. **Rollout:** Schrittweise auf alle Kontexte erweitern.

4. **Doku:** Neue `docs/KEYBOARD_SHORTCUTS.md` und `docs/CONTEXT_MENUS.md` für User.

---

## Bekannte Limitierungen

- **egui Submenus:** `egui::menu::menu_button()` unterstützt beliebig tief verschachtelte Menus.
- **Rechtsklick-Priorität:** Aktuell wird nur 1 Context-Menu pro Frame gezeigt (via `response.context_menu()`). Wenn mehrere Bedingungen zutreffen, muss der Router die richtige wählen.
- **Touch-Support:** egui's Context-Menu öffnet nur bei Rechtsklick/Secondary. Touch-Geräte benötigen Long-Press-Emulation (egui hat dafür `PointerButton::Secondary` auf Tap+Hold).

---

## Fazit

Das vorgeschlagene System macht **alle Befehle über das Rechtsklick-Menü erreichbar**, kontextabhängig und hierarchisch strukturiert. Es behält die bestehende Toolbar/Properties/Keyboard-Struktur bei, erweitert sie aber um vollständige Rechtsklick-Navigation.

**Trade-off:** Mehr Menü-Komplexität vs. weniger Maus-Reisen zur Toolbar.

**Vorteil:** Power-User können **alles** per Rechtsklick bedienen, ohne die Maus weit bewegen zu müssen.
