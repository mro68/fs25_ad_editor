//! Linkes Panel fuer Standard-Richtung und -Prioritaet.

use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority};
use crate::ui::properties::selectors::{
    render_direction_icon_selector_vertical, render_priority_icon_selector_vertical,
};

/// Rendert das linke Defaults-Panel mit Richtung/Prioritaet.
pub fn render_route_defaults_panel(
    ctx: &egui::Context,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    egui::SidePanel::left("route_defaults_panel")
        .resizable(false)
        .default_width(56.0)
        .show(ctx, |ui| {
            let mut selected_dir = default_direction;
            render_direction_icon_selector_vertical(ui, &mut selected_dir, "defaults_left");
            if selected_dir != default_direction {
                events.push(AppIntent::SetDefaultDirectionRequested {
                    direction: selected_dir,
                });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let mut selected_prio = default_priority;
            render_priority_icon_selector_vertical(ui, &mut selected_prio, "defaults_left");
            if selected_prio != default_priority {
                events.push(AppIntent::SetDefaultPriorityRequested {
                    priority: selected_prio,
                });
            }
        });

    events
}
