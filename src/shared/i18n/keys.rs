//! Schlüssel-Enum für das i18n-System.
//!
//! Jeder Variants-Name entspricht einem UI-String.
//! `match` in den Sprachdateien erzwingt Vollständigkeit bei neuen Keys.

/// Alle übersetzbaren UI-Schlüssel des Editors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // === Allgemein ===
    /// Anwendungstitel
    AppTitle,
    /// Bestätigung
    Ok,
    /// Abbruch
    Cancel,
    /// Übernehmen
    Apply,
    /// Schließen
    Close,
    /// Zurücksetzen
    Reset,
    /// Löschen
    Delete,
    /// Hinzufügen
    Add,
    /// Entfernen
    Remove,
    /// Bezeichnung für Sprachauswahl
    LanguageLabel,

    // === Dialog-Chrome ===
    /// Schaltfläche: Dialog schließen
    DialogClose,
    /// Schaltfläche: Standardwerte wiederherstellen
    DialogDefaults,

    // === Options-Dialog: Abschnitts-Navigation ===
    /// Abschnittstitel "Allgemein"
    OptSectionGeneral,
    /// Abschnittstitel "Nodes"
    OptSectionNodes,
    /// Abschnittstitel "Tools"
    OptSectionTools,
    /// Abschnittstitel "Verbindungen"
    OptSectionConnections,
    /// Abschnittstitel "Verhalten"
    OptSectionBehavior,

    // === Options-Dialog: Abschnitts-Untertitel ===
    /// Untertitel für den Allgemein-Abschnitt
    OptSubtitleGeneral,
    /// Untertitel für den Nodes-Abschnitt
    OptSubtitleNodes,
    /// Untertitel für den Tools-Abschnitt
    OptSubtitleTools,
    /// Untertitel für den Verbindungen-Abschnitt
    OptSubtitleConnections,
    /// Untertitel für den Verhalten-Abschnitt
    OptSubtitleBehavior,

    // === Options-Dialog: Sprache ===
    /// Bezeichnung für das Sprach-Auswahlfeld
    OptLanguageLabel,
}
