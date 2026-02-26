# Plan: Garantierte Gültigkeit von Context-Menu-Einträgen

## Problem-Statement

**Ausgangslage**: Das Context-Menu zeigt verschiedene Einträge je nach `MenuVariant`, aber:
- Einige Einträge werden *dynamisch* konditioniert (z.B. "Nodes verbinden" nur wenn 2 Nodes UND keine Connection)
- Einige Vor­bedingungen sind *im UI versteckt* (z.B. in `button_intent` Closures)
- Wenn eine Vorbedingung *nach* Menü-Anzeige verletzt wird (z.B. Node wird gelöscht), kann ein ungültiger Intent ausgelöst werden

**Ziel**: *Garantieren*, dass JEDER sichtbare Menu-Eintrag unter der aktuellen Selektion/Position ausgeführt werden kann.

---

## Lösung: 4-Schichten-Architektur

### Schicht 1: Command-Definition (Immutable)

**Datei**: Neue `src/app/commands/mod.rs` (oder in `src/app/mod.rs` erweitern)

Jeder Command wird als Struct mit Preconditions definiert:

```rust
/// Eindeutige Identifikation eines Befehls (CommandId)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    // Single Node
    SelectNode,
    EditMarker,
    DeleteNode,
    // Multiple Nodes
    ConnectTwoNodes,
    SetDirection(ConnectionDirection),
    // Route Tool
    ExecuteRoute,
    // etc.
}

/// Command beschreibt Intent + Vorbedingungen
#[derive(Debug, Clone)]
pub struct Command {
    pub id: CommandId,
    pub label: String,          // "✓ Selektieren"
    pub intent: AppIntent,      // Was soll passieren?
    pub icon: &'static str,     // Optional: emoji
}

/// Precondition: Prüft, ob ein Command in der aktuellen Situation valid ist
pub trait Precondition {
    fn is_valid(
        &self,
        road_map: &RoadMap,
        selected: &HashSet<u64>,
        hovered: Option<u64>,
        route_tool_active: bool,
    ) -> bool;
}

// Beispiel:
pub struct CanConnectTwoSelectedNodes;
impl Precondition for CanConnectTwoSelectedNodes {
    fn is_valid(
        &self,
        road_map: &RoadMap,
        selected: &HashSet<u64>,
        _hovered: Option<u64>,
        _route_tool: bool,
    ) -> bool {
        if selected.len() != 2 { return false; }
        let ids: Vec<_> = selected.iter().cloned().collect();
        !road_map.has_connection(ids[0], ids[1])
    }
}
```

### Schicht 2: Command-Katalog pro MenuVariant

**Datei**: Neue `src/app/commands/catalog.rs`

**Idee**: Für jede `MenuVariant` einen statischen "Katalog" definieren, der:
- Welche Commands sind überhaupt sichtbar?
- Welche Preconditions müssen erfüllt sein?

```rust
pub struct MenuCatalog {
    /// Commands der Reihe nach (mit optionalen Separatoren)
    pub entries: Vec<MenuEntry>,
}

pub enum MenuEntry {
    Command(CommandId, Vec<Box<dyn Precondition>>),
    Separator,
    Section(String),  // z.B. "🗺 Marker"
}

impl MenuCatalog {
    /// Katalog für "SingleNodeSelected"
    pub fn for_single_node_selected() -> Self {
        MenuCatalog {
            entries: vec![
                MenuEntry::Section("Node-Operationen".to_owned()),
                MenuEntry::Command(
                    CommandId::DeselectNode,
                    vec![]  // Keine Preconditions: sichtbar wenn Node ausgewählt
                ),
                MenuEntry::Separator,
                MenuEntry::Section("🗺 Marker".to_owned()),
                MenuEntry::Command(CommandId::EditMarker, vec![]),
                MenuEntry::Command(CommandId::DeleteMarker, vec![]),
                MenuEntry::Separator,
                MenuEntry::Command(CommandId::DeleteNode, vec![]),
                MenuEntry::Command(CommandId::DuplicateNode, vec![]),
            ],
        }
    }

    /// Katalog für "MultipleNodesSelected"
    pub fn for_multiple_nodes_selected() -> Self {
        MenuCatalog {
            entries: vec![
                MenuEntry::Section("Selektion".to_owned()),
                MenuEntry::Command(
                    CommandId::ConnectTwoNodes,
                    vec![Box::new(CanConnectTwoSelectedNodes)]
                ),
                MenuEntry::Command(
                    CommandId::SetDirectionRegular,
                    vec![Box::new(HasSelectedConnections)]
                ),
                // ...
            ],
        }
    }
}
```

### Schicht 3: Validierung & Filtering

**Datei**: Neue `src/app/commands/validator.rs`

```rust
pub struct CommandValidator;

impl CommandValidator {
    /// Gibt alle **gültigen** Commands für eine Situation zurück
    pub fn valid_commands(
        catalog: &MenuCatalog,
        road_map: &RoadMap,
        selected: &HashSet<u64>,
        hovered: Option<u64>,
        route_tool_active: bool,
    ) -> Vec<Command> {
        catalog
            .entries
            .iter()
            .filter_map(|entry| match entry {
                MenuEntry::Command(id, preconditions) => {
                    let all_valid = preconditions.iter().all(|p| {
                        p.is_valid(road_map, selected, hovered, route_tool_active)
                    });
                    all_valid.then(|| CommandId::to_command(*id))
                }
                _ => None,
            })
            .collect()
    }
}
```

### Schicht 4: Rendering mit garantierter Validität

**Datei**: `src/ui/context_menu/mod.rs` (angepasst)

```rust
/// Neue, validierte Context-Menu-Rendering
pub fn render_context_menu_validated(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    variant: &MenuVariant,
    events: &mut Vec<AppIntent>,
) -> bool {
    let Some(rm) = road_map else { return false };

    response
        .context_menu(|ui| {
            // **SCHRITT 1**: Katalog für diese Variante
            let catalog = match variant {
                MenuVariant::SingleNodeSelected { node_id } => {
                    MenuCatalog::for_single_node_selected()
                }
                MenuVariant::MultipleNodesSelected => {
                    MenuCatalog::for_multiple_nodes_selected()
                }
                // ...
            };

            // **SCHRITT 2**: Nur gültige Commands
            let valid_commands = CommandValidator::valid_commands(
                &catalog,
                rm,
                selected_node_ids,
                /* hovered */ None,
                /* route_tool */ false,
            );

            // **SCHRITT 3**: Rendern
            for item in &catalog.entries {
                match item {
                    MenuEntry::Section(label) => ui.label(label),
                    MenuEntry::Separator => ui.separator(),
                    MenuEntry::Command(id, _) => {
                        if let Some(cmd) = valid_commands.iter().find(|c| c.id == *id) {
                            if ui.button(&cmd.label).clicked() {
                                events.push(cmd.intent.clone());
                                ui.close();
                            }
                        }
                        // Implizit: Ungültige Commands werden NICHT gerendert
                    }
                }
            }
        })
        .is_some()
}
```

---

## Vorteile dieser Architektur

1. **Zentrale Source of Truth**
   - Jeder Command ist einmal definiert (Label, Intent, Preconditions)
   - Keine Duplikation über multiple Functions

2. **Explizite Preconditions**
   - Leicht zu testen: `CanConnectTwoNodes.is_valid(...)` → bool
   - Leicht zu debuggen: Warum ist Command unsichtbar?

3. **Type-safe**
   - `CommandId` enum verhindert Tippfehler
   - Compiler prüft, dass alle Commands definiert sind

4. **Erweiterbar**
   - Neue Commands: Command-Definition + Precondition + Katalog-Eintrag
   - Neue Preconditions: trait impl

5. **Testbar**
   - `CommandValidator::valid_commands()` Unit-testbar
   - Preconditions sind isoliert testbar

6. **Optional: UI-Feedback**
   - Statt hidden → disabled anzeigen ("Verbinden" grau, Tooltip "Keine 2 Nodes")
   - User sieht, *warum* Command nicht verfügbar ist

---

## Implementierungs-Roadmap

### Phase 1: Grundgerüst
- [ ] `Command` + `CommandId` + command enum definieren
- [ ] `Precondition` trait definieren
- [ ] Erste 3 Preconditions implementieren (z.B. `CanConnectTwoNodes`, `HasSelectedConnections`, `NodeExists`)

### Phase 2: Kataloge
- [ ] `MenuCatalog` struct + impl
- [ ] Katalog für `SingleNodeSelected` schreiben
- [ ] Katalog für `MultipleNodesSelected` schreiben
- [ ] Katalog für `EmptyArea` schreiben

### Phase 3: Validator + Rendering
- [ ] `CommandValidator` implementieren
- [ ] `render_context_menu_validated()` schreiben
- [ ] Alte Rendering-Functions entfernen

### Phase 4: Preconditions vervollständigen
- [ ] Alle Commands durchgehen und Preconditions hatten
- [ ] Route-Tool Commands + Preconditions
- [ ] Marker Commands + Preconditions

### Phase 5: Tests + Docs
- [ ] Unit-Tests für Preconditions
- [ ] Integration-Tests: Menu-Kataloge
- [ ] Docstrings aktualisieren

---

## Beispiel: Detaillierte Implementierung "Nodes verbinden"

### Command-Definition

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    ConnectTwoSelectedNodes,
    // ...
}

pub struct ConnectTwoSelectedNodesCmd;

static CONNECT_TWO_NODES: Command = Command {
    id: CommandId::ConnectTwoSelectedNodes,
    label: "🔗 Nodes verbinden",
    intent_template: |node_a, node_b| {
        AppIntent::ConnectSelectedNodesRequested
    },
};
```

### Precondition

```rust
pub struct ExactlyTwoNodesSelected;

impl Precondition for ExactlyTwoNodesSelected {
    fn is_valid(&self, _: &RoadMap, selected: &HashSet<u64>, _: Option<u64>, _: bool) -> bool {
        selected.len() == 2
    }
}

pub struct NoConnectionBetweenSelected;

impl Precondition for NoConnectionBetweenSelected {
    fn is_valid(&self, road_map: &RoadMap, selected: &HashSet<u64>, _: Option<u64>, _: bool) -> bool {
        if selected.len() != 2 { return false; }
        let ids: Vec<_> = selected.iter().cloned().collect();
        !road_map.has_connection(ids[0], ids[1])
            && !road_map.has_connection(ids[1], ids[0])
    }
}
```

### Katalog-Eintrag

```rust
MenuEntry::Command(
    CommandId::ConnectTwoSelectedNodes,
    vec![
        Box::new(ExactlyTwoNodesSelected),
        Box::new(NoConnectionBetweenSelected),
    ]
),
```

### Rendering (automatisch!)

```rust
let catalog = MenuCatalog::for_multiple_nodes_selected();
let valid = CommandValidator::valid_commands(&catalog, road_map, selected, ...);
// → Button wird NICHT gerendert, wenn Preconditions nicht erfüllt
```

---

## Alternativer Ansatz: "Precondition erst im Handler prüfen"

**Nicht empfohlen**, weil:
- User sieht Button, der nicht funktioniert (verwirrrend)
- Handler muss Fehler abfangen (error handling komplex)
- Keine zentrale Regel, was angezeigt wird

**Empfohlen**:
- UI zeigt nur gültige Commands
- Handler kann *assume*, dass Preconditions erfüllt sind
- Fehler im Handler sind echte bugs, nicht normale cases

---

## Fragen zur Klärung

1. **Disabled vs. Hidden?**
   - Hidden: (aktueller Ansatz) Command ist nicht sichtbar wenn invalid
   - Disabled: Command ist sichtbar aber grau, mit Tooltip warum
   - **Empfehlung**: Zunächst Hidden (einfacher). Später Disabled für bessere UX

2. **Precondition-Evaluation Frequenz?**
   - Bei jedem Render (safe, aber slower)
   - Nur bei State-Änderung (fast, aber komplexer)
   - **Empfehlung**: Bei jedem Render (egui ist fast genug)

3. **Vererbung von Preconditions?**
   - z.B. "HasSelectedNodes" → "ExactlyTwoNodes" + "TwoNodesUnconnected"?
   - **Empfehlung**: Flach halten (composition über inheritance)

---

## Risks & Mitigation

| Risk | Mitigation |
|------|-----------|
| Precondition-Bugs (falsch positive) | unit tests für jede Precondition |
| Performance bei vielen Commands | Lazy evaluation, caching |
| Refactoring-Aufwand | schrittweise (Phase 1→5), alte Funktionen parallel halten |
| Handler erwartet ungültige State | assertions in Handler, oder precondition-name in intent |

