//! Tangenten-Zustand und UI-Baustein für Curve- und Spline-Tool.

use super::geometry::angle_to_compass;
use crate::core::ConnectedNeighbor;

/// Quelle einer Tangente am Start- oder Endpunkt eines Route-Tools.
///
/// Wird von Curve- und Spline-Tool verwendet, um den Kontrollpunkt
/// bzw. Phantom-Punkt tangential an einer bestehenden Verbindung auszurichten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TangentSource {
    /// Kein Tangenten-Vorschlag — Punkt wird manuell gesetzt
    None,
    /// Tangente aus bestehender Verbindung
    Connection { neighbor_id: u64, angle: f32 },
}

/// Rendert eine Tangenten-ComboBox und gibt `true` zurück wenn die Auswahl geändert wurde.
///
/// Gemeinsamer UI-Baustein für Curve- und Spline-Tool.
pub fn render_tangent_combo(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    none_label: &str,
    current: &mut TangentSource,
    neighbors: &[ConnectedNeighbor],
) -> bool {
    let old = *current;
    let selected_text = match *current {
        TangentSource::None => none_label.to_string(),
        TangentSource::Connection { neighbor_id, angle } => {
            format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
        }
    };
    ui.label(label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            ui.selectable_value(current, TangentSource::None, none_label);
            for neighbor in neighbors {
                let text = format!(
                    "→ Node #{} ({})",
                    neighbor.neighbor_id,
                    angle_to_compass(neighbor.angle)
                );
                ui.selectable_value(
                    current,
                    TangentSource::Connection {
                        neighbor_id: neighbor.neighbor_id,
                        angle: neighbor.angle,
                    },
                    text,
                );
            }
        });
    *current != old
}

/// Gemeinsamer Tangenten-Zustand für Curve- und Spline-Tool.
///
/// Kapselt die 6 Tangenten-Felder, die beide Tools identisch teilen,
/// und bietet Hilfsmethoden für häufige Operationen.
#[derive(Debug, Clone)]
pub struct TangentState {
    /// Gewählte Tangente am Startpunkt
    pub tangent_start: TangentSource,
    /// Gewählte Tangente am Endpunkt
    pub tangent_end: TangentSource,
    /// Verfügbare Nachbarn am Startpunkt (Cache, wird beim Snap befüllt)
    pub start_neighbors: Vec<ConnectedNeighbor>,
    /// Verfügbare Nachbarn am Endpunkt (Cache, wird beim Snap befüllt)
    pub end_neighbors: Vec<ConnectedNeighbor>,
    /// Tangente Start der letzten Erstellung (für Recreation)
    pub last_tangent_start: TangentSource,
    /// Tangente Ende der letzten Erstellung (für Recreation)
    pub last_tangent_end: TangentSource,
}

impl TangentState {
    /// Erstellt einen neuen TangentState mit Standardwerten (alle `None`).
    pub fn new() -> Self {
        Self {
            tangent_start: TangentSource::None,
            tangent_end: TangentSource::None,
            start_neighbors: Vec::new(),
            end_neighbors: Vec::new(),
            last_tangent_start: TangentSource::None,
            last_tangent_end: TangentSource::None,
        }
    }

    /// Setzt nur die gewählten Tangenten auf `None` zurück; der Nachbarn-Cache bleibt erhalten.
    ///
    /// **Wann benutzen:** Bei Verkettung (Chaining) — die Nachbarn des neuen Startpunkts
    /// werden erst nach dem nächsten Snap befüllt, daher den Cache nicht unnötig löschen.
    /// Wird in `on_click()` aufgerufen, bevor der neue Start-Snap ausgewertet wird.
    pub fn reset_tangents(&mut self) {
        self.tangent_start = TangentSource::None;
        self.tangent_end = TangentSource::None;
    }

    /// Setzt Tangenten **und** Nachbarn-Cache vollständig zurück.
    ///
    /// **Wann benutzen:** Beim vollständigen Tool-Reset (Escape, neues Werkzeug wählen
    /// oder `execute()` ohne Verkettung) — nach dem Reset ist der Cache ungültig,
    /// da der nächste Snap an einem anderen Node landen kann.
    pub fn reset_all(&mut self) {
        self.tangent_start = TangentSource::None;
        self.tangent_end = TangentSource::None;
        self.start_neighbors.clear();
        self.end_neighbors.clear();
    }

    /// Speichert die aktuellen Tangenten in den `last_*`-Feldern (für Recreation).
    pub fn save_for_recreate(&mut self) {
        self.last_tangent_start = self.tangent_start;
        self.last_tangent_end = self.tangent_end;
    }
}

impl Default for TangentState {
    fn default() -> Self {
        Self::new()
    }
}
