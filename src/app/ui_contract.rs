//! App-weite Read-DTOs fuer UI-nahe Route-Tool-Daten.

use crate::app::tool_contract::TangentSource;

/// Eine waehlbare Tangenten-Option mit bereits aufbereitetem UI-Label.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentOptionData {
    /// Semantische Quelle der Tangente.
    pub source: TangentSource,
    /// Fertig formatierter UI-Text fuer Menues und Listen.
    pub label: String,
}

/// Reine Menue-Daten fuer die Tangenten-Auswahl eines Route-Tools.
///
/// Enthalten sind nur read-only DTOs und primitive Werte, damit die UI keine
/// Tool-Interna oder `egui`-nahe Zustandsobjekte kennen muss.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentMenuData {
    /// Aufbereitete Optionen fuer die Start-Tangente.
    pub start_options: Vec<TangentOptionData>,
    /// Aufbereitete Optionen fuer die End-Tangente.
    pub end_options: Vec<TangentOptionData>,
    /// Aktuell gewaehlte Start-Tangente.
    pub current_start: TangentSource,
    /// Aktuell gewaehlte End-Tangente.
    pub current_end: TangentSource,
}