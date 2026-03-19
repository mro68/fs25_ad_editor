//! Deutsche Übersetzungen.

use super::keys::I18nKey;

/// Deutsche Übersetzung für den gegebenen Schlüssel.
pub(super) fn translate(key: I18nKey) -> &'static str {
    match key {
        I18nKey::AppTitle => "FS25 AutoDrive Editor",
        I18nKey::Ok => "OK",
        I18nKey::Cancel => "Abbrechen",
        I18nKey::Apply => "Übernehmen",
        I18nKey::Close => "Schliessen",
        I18nKey::Reset => "Zurücksetzen",
        I18nKey::Delete => "Löschen",
        I18nKey::Add => "Hinzufügen",
        I18nKey::Remove => "Entfernen",
        I18nKey::LanguageLabel => "Sprache",
        I18nKey::DialogClose => "Schliessen",
        I18nKey::DialogDefaults => "Standardwerte",
        I18nKey::OptSectionGeneral => "Allgemein",
        I18nKey::OptSectionNodes => "Nodes",
        I18nKey::OptSectionTools => "Tools",
        I18nKey::OptSectionConnections => "Verbindungen",
        I18nKey::OptSectionBehavior => "Verhalten",
        I18nKey::OptSubtitleGeneral => "Globale Anzeige- und Karten-Einstellungen.",
        I18nKey::OptSubtitleNodes => "Darstellung und Größe der Wegpunkte.",
        I18nKey::OptSubtitleTools => "Werkzeug-Verhalten und Snap-Einstellungen.",
        I18nKey::OptSubtitleConnections => "Darstellung von Verbindungslinien und Pfeilen.",
        I18nKey::OptSubtitleBehavior => "Editor-Verhalten beim Bearbeiten und Löschen.",
        I18nKey::OptLanguageLabel => "Sprache:",
    }
}
