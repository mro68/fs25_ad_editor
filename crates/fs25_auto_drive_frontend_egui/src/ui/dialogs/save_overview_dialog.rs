//! Dialog: Hintergrundbild als overview.png im Savegame-Verzeichnis speichern.

use super::{dialog_two_action_row_enabled, DialogTwoAction};
use crate::app::AppIntent;
use fs25_auto_drive_host_bridge::HostLocalDialogState;

/// Zeigt den Dialog "Als overview.png speichern?" nach ZIP-Extraktion.
pub fn show_save_overview_dialog(
    ctx: &egui::Context,
    ui_state: &mut HostLocalDialogState,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.save_overview_dialog.visible {
        return events;
    }

    egui::Window::new("Hintergrundbild speichern?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                if ui_state.save_overview_dialog.is_overwrite {
                    ui.label("Es existiert bereits eine overview.png.");
                    ui.label("Soll sie mit dem extrahierten Bild ueberschrieben werden?");
                } else {
                    ui.label("Soll das Hintergrundbild als overview.png");
                    ui.label("im Savegame-Verzeichnis gespeichert werden?");
                }
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(&ui_state.save_overview_dialog.target_path)
                        .weak()
                        .small(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "Beim naechsten Laden wird es automatisch als Hintergrund verwendet.",
                    )
                    .weak(),
                );
                let confirm_label = if ui_state.save_overview_dialog.is_overwrite {
                    "Ja, ueberschreiben"
                } else {
                    "Ja, speichern"
                };
                if let Some(action) =
                    dialog_two_action_row_enabled(ui, confirm_label, "Nein", true, true)
                {
                    match action {
                        DialogTwoAction::Confirm => {
                            events.push(AppIntent::SaveBackgroundAsOverviewConfirmed);
                        }
                        DialogTwoAction::Cancel => {
                            events.push(AppIntent::SaveBackgroundAsOverviewDismissed);
                        }
                    }
                }
            });
        });

    events
}
