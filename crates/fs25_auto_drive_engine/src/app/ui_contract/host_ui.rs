//! Host-neutrale Vertraege fuer Dialoge und Tool-Fenster.

use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority};
use crate::shared::EditorOptions;

use super::{RouteToolPanelAction, RouteToolPanelState};

/// Stabile Art eines Host-Datei-/Pfad-Dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogRequestKind {
    /// AutoDrive-XML laden.
    OpenFile,
    /// AutoDrive-XML speichern.
    SaveFile,
    /// Heightmap-Bild auswaehlen.
    Heightmap,
    /// Hintergrundbild oder ZIP auswaehlen.
    BackgroundMap,
    /// Map-Mod-ZIP fuer Overview-Generierung auswaehlen.
    OverviewZip,
    /// Curseplay-Datei importieren.
    CurseplayImport,
    /// Curseplay-Datei exportieren.
    CurseplayExport,
}

/// Semantische Host-Anforderung fuer Datei-/Pfad-Dialoge.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogRequest {
    /// Host soll einen Datei-/Pfaddialog oeffnen.
    PickPath {
        /// Semantische Bedeutung der Anfrage.
        kind: DialogRequestKind,
        /// Optionaler Dateiname fuer Save-Dialoge.
        suggested_file_name: Option<String>,
    },
}

impl DialogRequest {
    /// Erstellt eine Dialog-Anfrage ohne vorgeschlagenen Dateinamen.
    pub fn pick_path(kind: DialogRequestKind) -> Self {
        Self::PickPath {
            kind,
            suggested_file_name: None,
        }
    }

    /// Erstellt eine Save-Dialog-Anfrage mit Dateinamenvorschlag.
    pub fn save_path_with_name(file_name: String) -> Self {
        Self::PickPath {
            kind: DialogRequestKind::SaveFile,
            suggested_file_name: Some(file_name),
        }
    }

    /// Liefert die semantische Art der Anfrage.
    pub fn kind(&self) -> DialogRequestKind {
        match self {
            Self::PickPath { kind, .. } => *kind,
        }
    }

    /// Liefert den optionalen Dateinamenvorschlag.
    pub fn suggested_file_name(&self) -> Option<&str> {
        match self {
            Self::PickPath {
                suggested_file_name,
                ..
            } => suggested_file_name.as_deref(),
        }
    }
}

/// Rueckmeldung eines Hosts zu einer `DialogRequest`.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    /// Host-Dialog wurde ohne Auswahl geschlossen.
    Cancelled {
        /// Semantische Art der beantworteten Anfrage.
        kind: DialogRequestKind,
    },
    /// Host hat einen Pfad ausgewaehlt.
    PathSelected {
        /// Semantische Art der beantworteten Anfrage.
        kind: DialogRequestKind,
        /// Gewaehlter Pfad.
        path: String,
    },
}

/// Uebersetzt ein semantisches Dialog-Ergebnis in einen stabilen App-Intent.
///
/// `Cancelled`-Ergebnisse erzeugen absichtlich keinen Intent.
pub fn dialog_result_to_intent(result: DialogResult) -> Option<AppIntent> {
    match result {
        DialogResult::Cancelled { .. } => None,
        DialogResult::PathSelected { kind, path } => match kind {
            DialogRequestKind::OpenFile => Some(AppIntent::FileSelected { path }),
            DialogRequestKind::SaveFile => Some(AppIntent::SaveFilePathSelected { path }),
            DialogRequestKind::Heightmap => Some(AppIntent::HeightmapSelected { path }),
            DialogRequestKind::BackgroundMap => {
                if path.to_lowercase().ends_with(".zip") {
                    Some(AppIntent::ZipBackgroundBrowseRequested { path })
                } else {
                    Some(AppIntent::BackgroundMapSelected {
                        path,
                        crop_size: None,
                    })
                }
            }
            DialogRequestKind::OverviewZip => Some(AppIntent::GenerateOverviewFromZip { path }),
            DialogRequestKind::CurseplayImport => Some(AppIntent::CurseplayFileSelected { path }),
            DialogRequestKind::CurseplayExport => {
                Some(AppIntent::CurseplayExportPathSelected { path })
            }
        },
    }
}

/// Sichtbarer Host-UI-Snapshot fuer Tool-Fenster und Dialog-Requests.
#[derive(Debug, Clone)]
pub struct HostUiSnapshot {
    /// Alle semantischen Fenster/Panels dieses Frames.
    pub panels: Vec<PanelState>,
    /// Offene Host-Dialog-Anforderungen.
    pub dialog_requests: Vec<DialogRequest>,
}

impl HostUiSnapshot {
    /// Liefert den Route-Tool-Panelzustand, falls sichtbar.
    pub fn route_tool_panel_state(&self) -> Option<&RouteToolPanelState> {
        self.panels.iter().find_map(|panel| match panel {
            PanelState::RouteTool(state) => Some(state),
            _ => None,
        })
    }

    /// Liefert den Optionen-Panelzustand, falls vorhanden.
    pub fn options_panel_state(&self) -> Option<&OptionsPanelState> {
        self.panels.iter().find_map(|panel| match panel {
            PanelState::Options(state) => Some(state),
            _ => None,
        })
    }

    /// Liefert den Command-Palette-Zustand, falls vorhanden.
    pub fn command_palette_state(&self) -> Option<CommandPalettePanelState> {
        self.panels.iter().find_map(|panel| match panel {
            PanelState::CommandPalette(state) => Some(*state),
            _ => None,
        })
    }
}

/// Semantischer Zustand eines Tool-Fensters oder Panels.
#[derive(Debug, Clone)]
pub enum PanelState {
    /// Route-Tool-Konfigurationsfenster.
    RouteTool(RouteToolPanelState),
    /// Optionen-Dialog als host-neutrales Panel.
    Options(OptionsPanelState),
    /// Command-Palette-Zustand.
    CommandPalette(CommandPalettePanelState),
}

/// Read-only Zustand des Optionen-Panels.
#[derive(Debug, Clone)]
pub struct OptionsPanelState {
    /// Sichtbarkeit des Panels.
    pub visible: bool,
    /// Aktuelle Editor-Optionen.
    pub options: EditorOptions,
}

/// Read-only Zustand der Command-Palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandPalettePanelState {
    /// Sichtbarkeit der Palette.
    pub visible: bool,
}

/// Semantische Aktion aus Tool-Fenstern/Panel-UI.
#[derive(Debug, Clone)]
pub enum PanelAction {
    /// Standardrichtung fuer neue Verbindungen setzen.
    SetDefaultDirection {
        /// Neue Richtung.
        direction: ConnectionDirection,
    },
    /// Standard-Prioritaet fuer neue Verbindungen setzen.
    SetDefaultPriority {
        /// Neue Prioritaet.
        priority: ConnectionPriority,
    },
    /// Semantische Route-Tool-Panel-Aktion.
    RouteTool(RouteToolPanelAction),
    /// Route-Tool-Ausfuehrung anfordern.
    RouteToolExecute,
    /// Route-Tool abbrechen.
    RouteToolCancel,
    /// Optionen-Panel-Aktion.
    Options(OptionsPanelAction),
    /// Command-Palette umschalten.
    ToggleCommandPalette,
    /// Optionen-Panel oeffnen.
    OpenOptionsDialog,
}

/// Semantische Aktionen aus dem Optionen-Panel.
#[derive(Debug, Clone)]
pub enum OptionsPanelAction {
    /// Geaenderte Optionen uebernehmen.
    Apply(Box<EditorOptions>),
    /// Optionen auf Standardwerte zuruecksetzen.
    ResetToDefaults,
    /// Optionen-Panel schliessen.
    Close,
}

/// Uebersetzt eine `PanelAction` in einen stabilen `AppIntent`.
pub fn panel_action_to_intent(action: PanelAction) -> AppIntent {
    match action {
        PanelAction::SetDefaultDirection { direction } => {
            AppIntent::SetDefaultDirectionRequested { direction }
        }
        PanelAction::SetDefaultPriority { priority } => {
            AppIntent::SetDefaultPriorityRequested { priority }
        }
        PanelAction::RouteTool(action) => AppIntent::RouteToolPanelActionRequested { action },
        PanelAction::RouteToolExecute => AppIntent::RouteToolExecuteRequested,
        PanelAction::RouteToolCancel => AppIntent::RouteToolCancelled,
        PanelAction::Options(OptionsPanelAction::Apply(options)) => {
            AppIntent::OptionsChanged { options }
        }
        PanelAction::Options(OptionsPanelAction::ResetToDefaults) => {
            AppIntent::ResetOptionsRequested
        }
        PanelAction::Options(OptionsPanelAction::Close) => AppIntent::CloseOptionsDialogRequested,
        PanelAction::ToggleCommandPalette => AppIntent::CommandPaletteToggled,
        PanelAction::OpenOptionsDialog => AppIntent::OpenOptionsDialogRequested,
    }
}

#[cfg(test)]
mod tests {
    use crate::app::ui_contract::{
        dialog_result_to_intent, panel_action_to_intent, DialogRequestKind, OptionsPanelAction,
        PanelAction, ParkingPanelAction, RouteOffsetPanelAction, RouteToolPanelAction,
    };
    use crate::app::AppIntent;
    use crate::core::ConnectionDirection;

    #[test]
    fn panel_action_route_tool_maps_to_stable_intent() {
        let intent = panel_action_to_intent(PanelAction::RouteTool(RouteToolPanelAction::Parking(
            ParkingPanelAction::SetNumRows(4),
        )));

        assert!(matches!(
            intent,
            AppIntent::RouteToolPanelActionRequested {
                action: RouteToolPanelAction::Parking(ParkingPanelAction::SetNumRows(4))
            }
        ));
    }

    #[test]
    fn options_close_maps_to_dialog_intent() {
        let intent = panel_action_to_intent(PanelAction::Options(OptionsPanelAction::Close));
        assert!(matches!(intent, AppIntent::CloseOptionsDialogRequested));
    }

    #[test]
    fn background_zip_dialog_result_maps_to_zip_browse_intent() {
        let result = dialog_result_to_intent(crate::app::ui_contract::DialogResult::PathSelected {
            kind: DialogRequestKind::BackgroundMap,
            path: String::from("/tmp/background.zip"),
        });

        assert!(matches!(
            result,
            Some(AppIntent::ZipBackgroundBrowseRequested { path }) if path == "/tmp/background.zip"
        ));
    }

    #[test]
    fn cancelled_dialog_result_maps_to_no_intent() {
        let result = dialog_result_to_intent(crate::app::ui_contract::DialogResult::Cancelled {
            kind: DialogRequestKind::OpenFile,
        });

        assert!(result.is_none());
    }

    #[test]
    fn set_default_direction_action_maps_to_expected_intent() {
        let intent = panel_action_to_intent(PanelAction::SetDefaultDirection {
            direction: ConnectionDirection::Dual,
        });
        assert!(matches!(
            intent,
            AppIntent::SetDefaultDirectionRequested {
                direction: ConnectionDirection::Dual
            }
        ));
    }

    #[test]
    fn route_tool_tangent_action_stays_untouched() {
        let action =
            RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetLeftEnabled(true));
        let intent = panel_action_to_intent(PanelAction::RouteTool(action.clone()));

        assert!(matches!(
            intent,
            AppIntent::RouteToolPanelActionRequested { action: mapped }
            if mapped == action
        ));
    }
}
