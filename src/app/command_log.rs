//! Minimales Command-Log für spätere Undo/Redo-Erweiterung.

use super::AppCommand;

/// Speichert ausgeführte Commands in Reihenfolge.
#[derive(Default)]
pub struct CommandLog {
    entries: Vec<AppCommand>,
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

    /// Fügt einen ausgeführten Command hinzu.
    /// Begrenzt auf MAX_ENTRIES, ältere Einträge werden verworfen.
    pub fn record(&mut self, command: AppCommand) {
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.drain(..Self::MAX_ENTRIES / 2);
        }
        self.entries.push(command);
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
    pub fn entries(&self) -> &[AppCommand] {
        &self.entries
    }
}
