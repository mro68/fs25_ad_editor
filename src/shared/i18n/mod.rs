//! Mehrsprachigkeits-System (i18n) für den Editor.
//!
//! Stellt Übersetzungen für DE und EN bereit.
//! Zero-Alloc: Alle Übersetzungen sind `&'static str`.

mod de;
mod en;
mod keys;

pub use keys::I18nKey;

use serde::{Deserialize, Serialize};

/// Verfügbare Sprachen im Editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    /// Deutsch (Standard)
    #[default]
    De,
    /// Englisch
    En,
}

impl Language {
    /// Anzeigename der Sprache (immer in der jeweiligen Sprache).
    pub fn display_name(self) -> &'static str {
        match self {
            Language::De => "Deutsch",
            Language::En => "English",
        }
    }

    /// Alle verfügbaren Sprachen — für ComboBox-Iteration.
    pub fn all() -> &'static [Language] {
        &[Language::De, Language::En]
    }
}

/// Übersetzt einen Schlüssel in die gewählte Sprache.
///
/// Zero-Alloc: Gibt immer `&'static str` zurück.
pub fn t(lang: Language, key: I18nKey) -> &'static str {
    match lang {
        Language::De => de::translate(key),
        Language::En => en::translate(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_de_gibt_deutschen_text() {
        assert_eq!(t(Language::De, I18nKey::Cancel), "Abbrechen");
    }

    #[test]
    fn t_en_gibt_englischen_text() {
        assert_eq!(t(Language::En, I18nKey::Cancel), "Cancel");
    }

    #[test]
    fn language_display_name_korrekt() {
        assert_eq!(Language::De.display_name(), "Deutsch");
        assert_eq!(Language::En.display_name(), "English");
    }

    #[test]
    fn language_all_enthaelt_beide_sprachen() {
        assert_eq!(Language::all().len(), 2);
    }

    #[test]
    fn language_default_ist_de() {
        assert_eq!(Language::default(), Language::De);
    }
}
