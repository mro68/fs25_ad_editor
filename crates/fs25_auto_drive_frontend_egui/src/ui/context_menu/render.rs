//! Rendering-Helfer für validierte Kontextmenü-Einträge.

use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority};
use crate::shared::EditorOptions;

use super::commands::ValidatedEntry;
use super::icons::{
    command_icon, direction_or_priority_tooltip, is_direction_or_priority, CM_CHOICE_ICON_SIZE,
};

/// Rendert die validierten Einträge als egui-Elemente.
///
/// Submenüs werden als einklappbare `menu_button` gerendert,
/// die erst bei Hover aufklappen (natives egui-Submenu-Verhalten).
pub(super) fn render_validated_entries(
    ui: &mut egui::Ui,
    entries: &[ValidatedEntry],
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    events: &mut Vec<AppIntent>,
) {
    for entry in entries {
        match entry {
            ValidatedEntry::Label(text) => {
                ui.label(text);
            }
            ValidatedEntry::Separator => {
                ui.separator();
            }
            ValidatedEntry::Command {
                id, label, intent, ..
            } => {
                let clicked = if let Some(icon) =
                    command_icon(*id, options, default_direction, default_priority)
                {
                    if is_direction_or_priority(*id) {
                        let response = ui.add(egui::Button::image(
                            icon.fit_to_exact_size(CM_CHOICE_ICON_SIZE),
                        ));
                        let response = response.on_hover_text(direction_or_priority_tooltip(*id));
                        response.clicked()
                    } else {
                        let response = ui.add(egui::Button::image_and_text(icon, label));
                        response.clicked()
                    }
                } else {
                    let response = ui.button(label);
                    response.clicked()
                };
                if clicked {
                    events.push(*intent.clone());
                    ui.close();
                }
            }
            ValidatedEntry::Submenu {
                label,
                entries: children,
            } => {
                ui.menu_button(label, |ui| {
                    render_validated_entries(
                        ui,
                        children,
                        options,
                        default_direction,
                        default_priority,
                        events,
                    );
                });
            }
        }
    }
}
