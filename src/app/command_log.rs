//! Minimales Command-Log fuer Debug-Zwecke.
//!
//! Speichert Commands als Strings (via Debug-Format), um das Klonen
//! grosser Enum-Varianten (z.B. Lasso-Polygon) zu vermeiden.

use super::AppCommand;

/// Speichert ausgefuehrte Commands als Debug-Strings.
#[derive(Default)]
pub struct CommandLog {
    entries: Vec<String>,
}

impl CommandLog {
    const MAX_ENTRIES: usize = 1000;
}

impl CommandLog {
    /// Erstellt ein leeres Command-Log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fuegt einen ausgefuehrten Command als Debug-String hinzu.
    /// Begrenzt auf MAX_ENTRIES, aeltere Eintraege werden verworfen.
    pub fn record(&mut self, command: &AppCommand) {
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.drain(..Self::MAX_ENTRIES / 2);
        }
        self.entries.push(format!("{command:?}"));
    }

    /// Gibt die Anzahl der geloggten Commands zurueck.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Gibt `true` zurueck, wenn keine Commands vorhanden sind.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Liefert eine read-only Sicht auf alle Eintraege.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppCommand;

    #[test]
    fn new_log_ist_leer() {
        let log = CommandLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn record_fuegt_eintrag_als_debug_string_hinzu() {
        let mut log = CommandLog::new();
        log.record(&AppCommand::ResetCamera);
        assert_eq!(log.len(), 1);
        assert!(log.entries()[0].contains("ResetCamera"));
    }

    #[test]
    fn log_begrenzt_auf_max_entries() {
        let mut log = CommandLog::new();
        let cmd = AppCommand::ResetCamera;
        // MAX_ENTRIES + 1 Eintraege → Haelfte soll verworfen werden
        for _ in 0..=CommandLog::MAX_ENTRIES {
            log.record(&cmd);
        }
        // Nach dem Trim: MAX_ENTRIES/2 Eintraege + 1 neuer = 501
        assert!(log.len() <= CommandLog::MAX_ENTRIES);
    }

    #[test]
    fn entries_enthaelt_kommando_name() {
        let mut log = CommandLog::new();
        log.record(&AppCommand::ClearSelection);
        log.record(&AppCommand::SelectAllNodes);
        let entries = log.entries();
        assert!(entries[0].contains("ClearSelection"));
        assert!(entries[1].contains("SelectAllNodes"));
    }
}
