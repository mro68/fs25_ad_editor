//! Tangenten-Zustand fuer Curve- und Spline-Tool.

use crate::app::tool_contract::TangentSource;
use crate::core::ConnectedNeighbor;

/// Gemeinsamer Tangenten-Zustand fuer Curve- und Spline-Tool.
///
/// Kapselt die 6 Tangenten-Felder, die beide Tools identisch teilen,
/// und bietet Hilfsmethoden fuer haeufige Operationen.
#[derive(Debug, Clone)]
pub struct TangentState {
    /// Gewaehlte Tangente am Startpunkt
    pub tangent_start: TangentSource,
    /// Gewaehlte Tangente am Endpunkt
    pub tangent_end: TangentSource,
    /// Verfuegbare Nachbarn am Startpunkt (Cache, wird beim Snap befuellt)
    pub start_neighbors: Vec<ConnectedNeighbor>,
    /// Verfuegbare Nachbarn am Endpunkt (Cache, wird beim Snap befuellt)
    pub end_neighbors: Vec<ConnectedNeighbor>,
    /// Tangente Start der letzten Erstellung (fuer Recreation)
    pub last_tangent_start: TangentSource,
    /// Tangente Ende der letzten Erstellung (fuer Recreation)
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

    /// Setzt nur die gewaehlten Tangenten auf `None` zurueck; der Nachbarn-Cache bleibt erhalten.
    ///
    /// **Wann benutzen:** Bei Verkettung (Chaining) — die Nachbarn des neuen Startpunkts
    /// werden erst nach dem naechsten Snap befuellt, daher den Cache nicht unnoetig loeschen.
    /// Wird in `on_click()` aufgerufen, bevor der neue Start-Snap ausgewertet wird.
    pub fn reset_tangents(&mut self) {
        self.tangent_start = TangentSource::None;
        self.tangent_end = TangentSource::None;
    }

    /// Setzt Tangenten **und** Nachbarn-Cache vollstaendig zurueck.
    ///
    /// **Wann benutzen:** Beim vollstaendigen Tool-Reset (Escape, neues Werkzeug waehlen
    /// oder `execute()` ohne Verkettung) — nach dem Reset ist der Cache ungueltig,
    /// da der naechste Snap an einem anderen Node landen kann.
    pub fn reset_all(&mut self) {
        self.tangent_start = TangentSource::None;
        self.tangent_end = TangentSource::None;
        self.start_neighbors.clear();
        self.end_neighbors.clear();
    }

    /// Speichert die aktuellen Tangenten in den `last_*`-Feldern (fuer Recreation).
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
