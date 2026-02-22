//! Minimales Command-Log für Debug-Zwecke.
//!
//! Speichert Commands als Strings (via Debug-Format), um das Klonen
//! großer Enum-Varianten (z.B. Lasso-Polygon) zu vermeiden.

use super::AppCommand;

/// Speichert ausgeführte Commands als Debug-Strings.
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

    /// Fügt einen ausgeführten Command als Debug-String hinzu.
    /// Begrenzt auf MAX_ENTRIES, ältere Einträge werden verworfen.
    pub fn record(&mut self, command: &AppCommand) {
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.drain(..Self::MAX_ENTRIES / 2);
        }
        self.entries.push(format!("{command:?}"));
    }

    /// Gibt die Anzahl der geloggten Commands zurück.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Gibt `true` zurück, wenn keine Commands vorhanden sind.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Liefert eine read-only Sicht auf alle Einträge.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}
