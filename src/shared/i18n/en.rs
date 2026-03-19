//! Englische Übersetzungen.

use super::keys::I18nKey;

/// Englische Übersetzung für den gegebenen Schlüssel.
pub(super) fn translate(key: I18nKey) -> &'static str {
    match key {
        I18nKey::AppTitle => "FS25 AutoDrive Editor",
        I18nKey::Ok => "OK",
        I18nKey::Cancel => "Cancel",
        I18nKey::Apply => "Apply",
        I18nKey::Close => "Close",
        I18nKey::Reset => "Reset",
        I18nKey::Delete => "Delete",
        I18nKey::Add => "Add",
        I18nKey::Remove => "Remove",
        I18nKey::LanguageLabel => "Language",
        I18nKey::DialogClose => "Close",
        I18nKey::DialogDefaults => "Defaults",
        I18nKey::OptSectionGeneral => "General",
        I18nKey::OptSectionNodes => "Nodes",
        I18nKey::OptSectionTools => "Tools",
        I18nKey::OptSectionConnections => "Connections",
        I18nKey::OptSectionBehavior => "Behavior",
        I18nKey::OptSubtitleGeneral => "Global display and map settings.",
        I18nKey::OptSubtitleNodes => "Appearance and size of waypoints.",
        I18nKey::OptSubtitleTools => "Tool behavior and snap settings.",
        I18nKey::OptSubtitleConnections => "Display of connection lines and arrows.",
        I18nKey::OptSubtitleBehavior => "Editor behavior when editing and deleting.",
        I18nKey::OptLanguageLabel => "Language:",
    }
}
