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

    /// Stellt sicher, dass DE und EN fuer ALLE Keys nicht-leere Strings liefern.
    /// Bricht sofort ab wenn ein Key einen leeren String zurueckgibt.
    #[test]
    fn alle_keys_haben_nicht_leere_uebersetzungen() {
        let all_keys: &[I18nKey] = &[
            I18nKey::AppTitle,
            I18nKey::Ok,
            I18nKey::Cancel,
            I18nKey::Apply,
            I18nKey::Close,
            I18nKey::Reset,
            I18nKey::Delete,
            I18nKey::Add,
            I18nKey::Remove,
            I18nKey::LanguageLabel,
            I18nKey::DialogClose,
            I18nKey::DialogDefaults,
            I18nKey::OptSectionGeneral,
            I18nKey::OptSectionNodes,
            I18nKey::OptSectionTools,
            I18nKey::OptSectionConnections,
            I18nKey::OptSectionBehavior,
            I18nKey::OptSubtitleGeneral,
            I18nKey::OptSubtitleNodes,
            I18nKey::OptSubtitleTools,
            I18nKey::OptSubtitleConnections,
            I18nKey::OptSubtitleBehavior,
            I18nKey::OptLanguageLabel,
            I18nKey::OptLanguageHelp,
            I18nKey::OptDialogTitle,
            I18nKey::OptNavHeader,
            I18nKey::OptSubSectionSelection,
            I18nKey::OptSubSectionMarker,
            I18nKey::OptSubSectionCamera,
            I18nKey::OptSubSectionLod,
            I18nKey::OptSubSectionLodDesc,
            I18nKey::OptSubSectionBackground,
            I18nKey::OptSubSectionCopyPaste,
            I18nKey::OptSubSectionOverview,
            I18nKey::OptNodeSizeWorld,
            I18nKey::OptNodeSizeWorldHelp,
            I18nKey::OptNodeColorDefault,
            I18nKey::OptNodeColorSubprio,
            I18nKey::OptNodeColorSelected,
            I18nKey::OptNodeColorWarning,
            I18nKey::OptHitboxScale,
            I18nKey::OptHitboxScaleHelp,
            I18nKey::OptValueAdjustMode,
            I18nKey::OptValueAdjustDrag,
            I18nKey::OptValueAdjustWheel,
            I18nKey::OptValueAdjustModeHelp,
            I18nKey::OptSnapRadius,
            I18nKey::OptSnapRadiusHelp,
            I18nKey::OptMouseWheelDistStep,
            I18nKey::OptMouseWheelDistStepHelp,
            I18nKey::OptSelectionSizeFactor,
            I18nKey::OptSelectionSizeFactorHelp,
            I18nKey::OptSelectionStyle,
            I18nKey::OptSelectionStyleRing,
            I18nKey::OptSelectionStyleGradient,
            I18nKey::OptSelectionStyleHelp,
            I18nKey::OptDoubleClickSegment,
            I18nKey::OptSegmentStopAtJunction,
            I18nKey::OptSegmentStopAtJunctionHelp,
            I18nKey::OptSegmentMaxAngle,
            I18nKey::OptSegmentMaxAngleHelp,
            I18nKey::OptSegmentDisabled,
            I18nKey::OptConnectionWidthMain,
            I18nKey::OptConnectionWidthMainHelp,
            I18nKey::OptConnectionWidthSubprio,
            I18nKey::OptConnectionWidthSubprioHelp,
            I18nKey::OptArrowLength,
            I18nKey::OptArrowLengthHelp,
            I18nKey::OptArrowWidth,
            I18nKey::OptArrowWidthHelp,
            I18nKey::OptConnectionColorRegular,
            I18nKey::OptConnectionColorDual,
            I18nKey::OptConnectionColorReverse,
            I18nKey::OptMarkerSize,
            I18nKey::OptMarkerSizeHelp,
            I18nKey::OptMarkerColor,
            I18nKey::OptMarkerOutlineWidth,
            I18nKey::OptMarkerOutlineWidthHelp,
            I18nKey::OptCameraZoomMin,
            I18nKey::OptCameraZoomMinHelp,
            I18nKey::OptCameraZoomMax,
            I18nKey::OptCameraZoomMaxHelp,
            I18nKey::OptCameraZoomStep,
            I18nKey::OptCameraZoomStepHelp,
            I18nKey::OptCameraScrollZoomStep,
            I18nKey::OptCameraScrollZoomStepHelp,
            I18nKey::OptZoomCompensationMax,
            I18nKey::OptZoomCompensationMaxHelp,
            I18nKey::OptLodMinSizes,
            I18nKey::OptLodNodes,
            I18nKey::OptLodNodesHelp,
            I18nKey::OptLodConnections,
            I18nKey::OptLodConnectionsHelp,
            I18nKey::OptLodArrows,
            I18nKey::OptLodArrowsHelp,
            I18nKey::OptLodMarkers,
            I18nKey::OptLodMarkersHelp,
            I18nKey::OptLodNodeDecimation,
            I18nKey::OptLodDecimationSpacing,
            I18nKey::OptLodDecimationSpacingHelp,
            I18nKey::OptBgOpacity,
            I18nKey::OptBgOpacityHelp,
            I18nKey::OptBgOpacityAtMinZoom,
            I18nKey::OptBgOpacityAtMinZoomHelp,
            I18nKey::OptBgFadeStartZoom,
            I18nKey::OptBgFadeStartZoomHelp,
            I18nKey::OptOverviewHillshade,
            I18nKey::OptOverviewHillshadeHelp,
            I18nKey::OptOverviewFarmlands,
            I18nKey::OptOverviewFarmlandsHelp,
            I18nKey::OptOverviewFarmlandIds,
            I18nKey::OptOverviewFarmlandIdsHelp,
            I18nKey::OptOverviewPois,
            I18nKey::OptOverviewPoisHelp,
            I18nKey::OptOverviewLegend,
            I18nKey::OptOverviewLegendHelp,
            I18nKey::OptReconnectOnDelete,
            I18nKey::OptReconnectOnDeleteHelp,
            I18nKey::OptSplitConnectionOnPlace,
            I18nKey::OptSplitConnectionOnPlaceHelp,
            I18nKey::OptCopyPastePreviewOpacity,
            I18nKey::OptCopyPastePreviewOpacityHelp,
            I18nKey::MenuFile,
            I18nKey::MenuOpen,
            I18nKey::MenuSave,
            I18nKey::MenuSaveAs,
            I18nKey::MenuSelectHeightmap,
            I18nKey::MenuChangeHeightmap,
            I18nKey::MenuClearHeightmap,
            I18nKey::MenuGenerateOverview,
            I18nKey::MenuExit,
            I18nKey::MenuEdit,
            I18nKey::MenuUndo,
            I18nKey::MenuRedo,
            I18nKey::MenuCopy,
            I18nKey::MenuPaste,
            I18nKey::MenuOptions,
            I18nKey::MenuView,
            I18nKey::MenuResetCamera,
            I18nKey::MenuZoomIn,
            I18nKey::MenuZoomOut,
            I18nKey::MenuLoadBackground,
            I18nKey::MenuChangeBackground,
            I18nKey::MenuRenderQuality,
            I18nKey::MenuQualityLow,
            I18nKey::MenuQualityMedium,
            I18nKey::MenuQualityHigh,
            I18nKey::MenuExtras,
            I18nKey::MenuDetectField,
            I18nKey::MenuTraceAllFields,
            I18nKey::MenuExtrasNeedBackground,
            I18nKey::MenuTraceAllFieldsHelp,
            I18nKey::MenuHelp,
            I18nKey::MenuAbout,
            I18nKey::StatusNoFile,
            I18nKey::StatusNodes,
            I18nKey::StatusConnections,
            I18nKey::StatusMarkers,
            I18nKey::StatusMap,
            I18nKey::StatusZoom,
            I18nKey::StatusPosition,
            I18nKey::StatusHeightmap,
            I18nKey::StatusHeightmapNone,
            I18nKey::StatusSelectedNodes,
            I18nKey::StatusExample,
            I18nKey::StatusTool,
            I18nKey::StatusFps,
            I18nKey::ToolNameSelect,
            I18nKey::ToolNameConnect,
            I18nKey::ToolNameAddNode,
            I18nKey::ToolNameRoute,
        ];

        for &key in all_keys {
            for &lang in Language::all() {
                let text = t(lang, key);
                assert!(
                    !text.is_empty(),
                    "Leere Uebersetzung fuer {:?} in {:?}",
                    key,
                    lang,
                );
            }
        }
    }

    /// Prüft, dass Language korrekt durch TOML serialisiert/deserialisiert wird.
    #[test]
    fn language_toml_roundtrip() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Cfg {
            language: Language,
        }
        let cfg_de = Cfg {
            language: Language::De,
        };
        let toml_str = toml::to_string(&cfg_de).expect("Serialisierung fehlgeschlagen");
        assert!(
            toml_str.contains("language"),
            "TOML muss language-Feld enthalten"
        );
        let parsed: Cfg = toml::from_str(&toml_str).expect("Deserialisierung fehlgeschlagen");
        assert_eq!(parsed.language, Language::De, "Roundtrip DE fehlgeschlagen");

        let cfg_en = Cfg {
            language: Language::En,
        };
        let toml_en = toml::to_string(&cfg_en).expect("Serialisierung EN fehlgeschlagen");
        let parsed_en: Cfg = toml::from_str(&toml_en).expect("Deserialisierung EN fehlgeschlagen");
        assert_eq!(
            parsed_en.language,
            Language::En,
            "Roundtrip EN fehlgeschlagen"
        );
    }

    /// Prüft, dass unbekannte Sprachwerte in TOML einen Fehler erzeugen.
    #[test]
    fn language_toml_unbekannter_wert_fehler() {
        let bad_toml = r#"language = "Fr""#;
        #[derive(serde::Deserialize)]
        struct Cfg {
            #[allow(dead_code)]
            language: Language,
        }
        let result = toml::from_str::<Cfg>(bad_toml);
        assert!(
            result.is_err(),
            "Unbekannter Sprachwert 'Fr' muss Fehler erzeugen"
        );
    }

    /// Prüft, dass fehlende Language in TOML den Default (De) verwendet.
    #[test]
    fn language_toml_fehlend_nutzt_default() {
        #[derive(serde::Deserialize, Debug)]
        struct Cfg {
            #[serde(default)]
            language: Language,
        }
        let empty_toml = "";
        let parsed: Cfg = toml::from_str(empty_toml).expect("Leere TOML muss parsebar sein");
        assert_eq!(
            parsed.language,
            Language::De,
            "Fehlende Language muss Default De sein"
        );
    }
}
