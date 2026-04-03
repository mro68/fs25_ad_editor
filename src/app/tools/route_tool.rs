//! RouteTool-Trait — Schnittstelle fuer alle Route-Tools.

use crate::app::group_registry::{GroupKind, GroupRecord};
use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::{
    RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect, TangentMenuData,
};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

use super::{ToolAction, ToolAnchor, ToolPreview, ToolResult};

/// Schnittstelle fuer alle Route-Tools (Linie, Parkplatz, Kurve, ...).
///
/// Tools sind zustandsbehaftet (Klick-Phasen) und erzeugen Preview-Geometrie
/// sowie ein `ToolResult` mit neuen Nodes/Connections.
pub trait RouteTool {
    /// Anzeigename fuer Toolbar.
    fn name(&self) -> &str;

    /// Icon-Zeichen fuer Dropdowns und Legacy-Toollisten.
    fn icon(&self) -> &str {
        ""
    }

    /// Kurzbeschreibung / Tooltip.
    fn description(&self) -> &str;

    /// Statustext fuer das Properties-Panel (z.B. "Startpunkt waehlen").
    fn status_text(&self) -> &str;

    /// Viewport-Klick verarbeiten. Gibt die naechste Aktion zurueck.
    /// `ctrl` ist true wenn Ctrl/Cmd gedrueckt war.
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, ctrl: bool) -> ToolAction;

    /// Scroll-Rotation verarbeiten (z.B. Alt+Mausrad).
    /// `delta` ist positiv fuer Aufwaertsscroll, negativ fuer Abwaerts.
    fn on_scroll_rotate(&mut self, _delta: f32) {}

    /// Preview-Geometrie fuer die aktuelle Mausposition berechnen.
    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview;

    /// Liefert den egui-freien Panelzustand des Tools.
    fn panel_state(&self) -> RouteToolConfigState;

    /// Wendet eine semantische Panel-Aktion auf das Tool an.
    fn apply_panel_action(&mut self, _action: RouteToolPanelAction) -> RouteToolPanelEffect {
        RouteToolPanelEffect::default()
    }

    /// Ergebnis erzeugen (Nodes + Connections als reine Daten).
    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult>;

    /// Tool-Zustand zuruecksetzen (Escape / Tool-Wechsel).
    fn reset(&mut self);

    /// Ist das Tool bereit zur Ausfuehrung?
    fn is_ready(&self) -> bool;

    /// Hat das Tool angefangene Eingaben (Punkte gesetzt, aber noch nicht ausgefuehrt)?
    ///
    /// Wird fuer die stufenweise Escape-Logik benoetigt:
    /// Tool zeichnet -> Cancel, Tool idle -> Selektion/Tool-Wechsel.
    fn has_pending_input(&self) -> bool {
        false
    }

    /// Verbindungsrichtung vom Editor-Default uebernehmen.
    fn set_direction(&mut self, _dir: ConnectionDirection) {}

    /// Strassenart vom Editor-Default uebernehmen.
    fn set_priority(&mut self, _prio: ConnectionPriority) {}

    /// Snap-Radius (Welteinheiten) vom Editor uebernehmen.
    fn set_snap_radius(&mut self, _radius: f32) {}

    /// Farmland-Polygone fuer Tools setzen, die auf Feldgrenzen reagieren.
    ///
    /// Standard-Implementierung ist ein No-Op. Nur `FieldBoundaryTool`
    /// ueberschreibt diese Methode.
    fn set_farmland_data(&mut self, _data: Option<std::sync::Arc<Vec<crate::core::FieldPolygon>>>) {
    }

    /// Setzt das Farmland-Raster fuer Pixel-basierte Analysen (z.B. Feldweg-Erkennung).
    /// Default: ignoriert den Input.
    fn set_farmland_grid(&mut self, _grid: Option<std::sync::Arc<crate::core::FarmlandGrid>>) {}

    /// Setzt die Hintergrundkarte fuer farbbasierte Analysen.
    /// Default: ignoriert den Input.
    fn set_background_map_image(&mut self, _image: Option<std::sync::Arc<image::DynamicImage>>) {}

    /// Speichert die IDs der zuletzt erstellten Nodes (fuer nachtraegliche Anpassung).
    /// `road_map` erlaubt Tools, Nachbar-Informationen fuer Feintuning zu cachen.
    fn set_last_created(&mut self, _ids: &[u64], _road_map: &RoadMap) {}

    /// Gibt den aktuellen End-Anker fuer Verkettung/Recreate zurueck.
    ///
    /// Standardmaessig `None`; Tools mit End-Anker ueberschreiben diese Methode.
    fn current_end_anchor(&self) -> Option<ToolAnchor> {
        None
    }

    /// Speichert tool-spezifische Anker-/Kontrollpunktdaten fuer Recreate.
    ///
    /// Wird vom gemeinsamen `set_last_created`-Flow aufgerufen.
    fn save_anchors_for_recreate(&mut self, _road_map: &RoadMap) {}

    /// Gibt die IDs der zuletzt erstellten Nodes zurueck.
    fn last_created_ids(&self) -> &[u64] {
        &[]
    }

    /// Gibt den letzten Endpunkt zurueck (fuer Verkettung).
    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        None
    }

    /// Signalisiert, ob eine Neuberechnung noetig ist (Config geaendert nach Erstellung).
    fn needs_recreate(&self) -> bool {
        false
    }

    /// Setzt das Recreate-Flag zurueck.
    fn clear_recreate_flag(&mut self) {}

    /// Erzeugt ein ToolResult aus den gespeicherten Ankern (fuer Nachbearbeitung).
    fn execute_from_anchors(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        None
    }

    /// Gibt die Weltpositionen aller verschiebbaren Punkte zurueck (fuer Drag-Hit-Test).
    ///
    /// Nur nicht-leer wenn alle noetigen Punkte gesetzt sind und das Tool
    /// im Drag-Modus bereitsteht (z.B. Phase::Control mit vollstaendigen CPs).
    fn drag_targets(&self) -> Vec<Vec2> {
        vec![]
    }

    /// Startet einen Drag auf einem Punkt nahe `pos`.
    ///
    /// Gibt `true` zurueck wenn ein Punkt gegriffen wurde, `false` wenn nichts in Reichweite.
    fn on_drag_start(&mut self, _pos: Vec2, _road_map: &RoadMap, _pick_radius: f32) -> bool {
        false
    }

    /// Aktualisiert die Position des gegriffenen Punkts waehrend eines Drags.
    fn on_drag_update(&mut self, _pos: Vec2) {}

    /// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
    fn on_drag_end(&mut self, _road_map: &RoadMap) {}

    /// Liefert Tangenten-Menuedaten fuer das Kontextmenue (nur Daten, kein UI).
    ///
    /// Gibt `Some(TangentMenuData)` zurueck wenn das Tool Tangenten-Optionen
    /// anbieten kann (z.B. kubische Kurve in Control-Phase mit Nachbarn).
    /// Gibt `None` zurueck wenn keine Tangenten-Auswahl verfuegbar ist.
    fn tangent_menu_data(&self) -> Option<TangentMenuData> {
        None
    }

    /// Wendet die vom User gewaehlten Tangenten an.
    ///
    /// Wird vom Context-Menu-Router aufgerufen nachdem der User eine
    /// Tangente im Menue ausgewaehlt hat. Das Tool aktualisiert seine
    /// Kontrollpunkte und setzt ggf. das Recreate-Flag.
    fn apply_tangent_selection(&mut self, _start: TangentSource, _end: TangentSource) {}

    /// Erstellt einen `GroupRecord` fuer die Registry aus dem aktuellen Tool-Zustand.
    ///
    /// Wird nach `execute()` aufgerufen um das Segment in der Registry zu speichern.
    /// Gibt `None` zurueck wenn das Tool keine Registry-Eintraege unterstuetzt.
    fn make_group_record(&self, _id: u64, _node_ids: &[u64]) -> Option<GroupRecord> {
        None
    }

    /// Laedt einen gespeicherten `GroupRecord` zur nachtraeglichen Bearbeitung.
    ///
    /// Stellt Start/End-Anker und alle tool-spezifischen Parameter (CP1, CP2,
    /// Tangenten, Anker-Liste) aus dem Record wieder her. Das Tool befindet
    /// sich anschliessend in der Control-Phase (bereit fuer Drag/Anpassung).
    fn load_for_edit(&mut self, _record: &GroupRecord, _kind: &GroupKind) {}

    /// Erhoeht die Anzahl der Nodes um 1.
    fn increase_node_count(&mut self) {}

    /// Verringert die Anzahl der Nodes um 1 (min. 2).
    fn decrease_node_count(&mut self) {}

    /// Erhoeht den minimalen Segment-Abstand um 0.25m.
    fn increase_segment_length(&mut self) {}

    /// Verringert den minimalen Segment-Abstand um 0.25m (min. 0.1m).
    fn decrease_segment_length(&mut self) {}

    /// Gibt `true` zurueck wenn dieses Tool eine geordnete Kette als Eingabe benoetigt.
    ///
    /// Solche Tools (z.B. `BypassTool`) erhalten ihre Eingabe nicht durch Klicks,
    /// sondern durch `load_chain()`, das vom Handler bei Tool-Aktivierung aufgerufen wird.
    fn needs_chain_input(&self) -> bool {
        false
    }

    /// Laedt eine geordnete Kette von Positionen als Tool-Eingabe.
    ///
    /// Wird vom Route-Tool-Handler aufgerufen wenn `needs_chain_input() == true` und
    /// die aktuelle Selektion eine gueltige Kette bildet.
    /// Standard-Implementierung: no-op.
    fn load_chain(&mut self, _positions: Vec<Vec2>, _start_id: u64, _end_id: u64) {}

    /// Setzt die inneren Node-IDs der geladenen Kette (ohne Start/Ende).
    ///
    /// Wird nach `load_chain` vom Handler aufgerufen um produktionskorrekte IDs
    /// fuer das "Original entfernen"-Feature bereitzustellen.
    /// Standard-Implementierung: no-op (die meisten Tools benoetigen keine inneren IDs).
    fn set_chain_inner_ids(&mut self, _ids: Vec<u64>) {}

    /// Gibt `true` zurueck wenn das Tool Alt+Drag als Lasso-Eingabe benoetigt
    /// (z.B. `ColorPathTool`).
    ///
    /// Ist `true`, wird ein Alt+Drag-Lasso als `ToolLasso` geroutet und der
    /// abgeschlossene Polygon per `on_lasso_completed` geliefert, statt die
    /// normale Node-Selektion auszuloesen.
    fn needs_lasso_input(&self) -> bool {
        false
    }

    /// Verarbeitet ein abgeschlossenes Lasso-Polygon in Weltkoordinaten.
    ///
    /// Wird aufgerufen sobald der User einen Alt+Drag-Lasso abgeschlossen hat
    /// und das Tool `needs_lasso_input()` zurueckgibt. Das Polygon enthaelt die
    /// Eckpunkte in Weltkoordinaten (gleiche Einheit wie `MapNode.position`).
    fn on_lasso_completed(&mut self, _polygon: Vec<Vec2>) -> ToolAction {
        ToolAction::Continue
    }
}
