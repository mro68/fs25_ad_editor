//! RouteTool-Trait — Schnittstelle für alle Route-Tools.

use crate::app::segment_registry::{SegmentKind, SegmentRecord};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

use super::{ToolAction, ToolAnchor, ToolPreview, ToolResult};

/// Schnittstelle für alle Route-Tools (Linie, Parkplatz, Kurve, …).
///
/// Tools sind zustandsbehaftet (Klick-Phasen) und erzeugen Preview-Geometrie
/// sowie ein `ToolResult` mit neuen Nodes/Connections.
pub trait RouteTool {
    /// Anzeigename für Toolbar
    fn name(&self) -> &str;

    /// Icon-Zeichen für das Dropdown (rechts vom Label)
    fn icon(&self) -> &str {
        ""
    }

    /// Kurzbeschreibung / Tooltip
    fn description(&self) -> &str;

    /// Statustext für das Properties-Panel (z.B. "Startpunkt wählen")
    fn status_text(&self) -> &str;

    /// Viewport-Klick verarbeiten. Gibt die nächste Aktion zurück.
    /// `ctrl` ist true wenn Ctrl/Cmd gedrückt war.
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, ctrl: bool) -> ToolAction;

    /// Preview-Geometrie für die aktuelle Mausposition berechnen.
    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview;

    /// Tool-spezifische Konfiguration im Properties-Panel rendern.
    /// Gibt `true` zurück wenn sich Einstellungen geändert haben.
    fn render_config(&mut self, ui: &mut egui::Ui) -> bool;

    /// Ergebnis erzeugen (Nodes + Connections als reine Daten).
    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult>;

    /// Tool-Zustand zurücksetzen (Escape / Tool-Wechsel).
    fn reset(&mut self);

    /// Ist das Tool bereit zur Ausführung?
    fn is_ready(&self) -> bool;

    /// Hat das Tool angefangene Eingaben (Punkte gesetzt, aber noch nicht ausgeführt)?
    ///
    /// Wird für die stufenweise Escape-Logik benötigt:
    /// Tool zeichnet → Cancel, Tool idle → Selektion/Tool-Wechsel.
    fn has_pending_input(&self) -> bool {
        false
    }

    /// Verbindungsrichtung vom Editor-Default übernehmen.
    fn set_direction(&mut self, _dir: ConnectionDirection) {}

    /// Straßenart vom Editor-Default übernehmen.
    fn set_priority(&mut self, _prio: ConnectionPriority) {}

    /// Snap-Radius (Welteinheiten) vom Editor übernehmen.
    fn set_snap_radius(&mut self, _radius: f32) {}

    /// Speichert die IDs der zuletzt erstellten Nodes (für nachträgliche Anpassung).
    /// `road_map` erlaubt tools, Nachbar-Informationen für Feintuning zu cachen.
    fn set_last_created(&mut self, _ids: &[u64], _road_map: &RoadMap) {}

    /// Gibt die IDs der zuletzt erstellten Nodes zurück.
    fn last_created_ids(&self) -> &[u64] {
        &[]
    }

    /// Gibt den letzten Endpunkt zurück (für Verkettung).
    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        None
    }

    /// Signalisiert, ob eine Neuberechnung nötig ist (Config geändert nach Erstellung).
    fn needs_recreate(&self) -> bool {
        false
    }

    /// Setzt das Recreate-Flag zurück.
    fn clear_recreate_flag(&mut self) {}

    /// Erzeugt ein ToolResult aus den gespeicherten Ankern (für Nachbearbeitung).
    fn execute_from_anchors(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        None
    }

    /// Gibt die Weltpositionen aller verschiebbaren Punkte zurück (für Drag-Hit-Test).
    ///
    /// Nur nicht-leer wenn alle nötigen Punkte gesetzt sind und das Tool
    /// im Drag-Modus bereitsteht (z.B. Phase::Control mit vollständigen CPs).
    fn drag_targets(&self) -> Vec<Vec2> {
        vec![]
    }

    /// Startet einen Drag auf einem Punkt nahe `pos`.
    ///
    /// Gibt `true` zurück wenn ein Punkt gegriffen wurde, `false` wenn nichts in Reichweite.
    fn on_drag_start(&mut self, _pos: Vec2, _road_map: &RoadMap, _pick_radius: f32) -> bool {
        false
    }

    /// Aktualisiert die Position des gegriffenen Punkts während eines Drags.
    fn on_drag_update(&mut self, _pos: Vec2) {}

    /// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
    fn on_drag_end(&mut self, _road_map: &RoadMap) {}

    /// Rendert ein Kontextmenü (Rechtsklick im Viewport) für das Tool.
    ///
    /// Wird im Viewport aufgerufen solange das Tool aktiv ist.
    /// Das CurveTool nutzt dies für die Tangenten-Auswahl (Rechtsklick → NW/SO-Einträge).
    ///
    /// Gibt `true` zurück wenn sich der Tool-Zustand geändert hat (ggf. Recreate nötig).
    fn render_context_menu(&mut self, _response: &egui::Response) -> bool {
        false
    }

    /// Erstellt einen `SegmentRecord` für die Registry aus dem aktuellen Tool-Zustand.
    ///
    /// Wird nach `execute()` aufgerufen um das Segment in der Registry zu speichern.
    /// Gibt `None` zurück wenn das Tool keine Registry-Einträge unterstützt.
    fn make_segment_record(&self, _id: u64, _node_ids: &[u64]) -> Option<SegmentRecord> {
        None
    }

    /// Lädt einen gespeicherten `SegmentRecord` zur nachträglichen Bearbeitung.
    ///
    /// Stellt Start/End-Anker und alle tool-spezifischen Parameter (CP1, CP2,
    /// Tangenten, Anker-Liste) aus dem Record wieder her. Das Tool befindet
    /// sich anschließend in der Control-Phase (bereit für Drag/Anpassung).
    fn load_for_edit(&mut self, _record: &SegmentRecord, _kind: &SegmentKind) {}
}
