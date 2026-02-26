//! Route Tool Menu: Route-Tool aktiv mit pending input.
//! Optional mit Tangenten-Auswahl für kubische Kurven.

use super::button_intent;
use crate::app::tools::common::TangentMenuData;
use crate::app::AppIntent;

pub fn render_route_tool_menu(
    ui: &mut egui::Ui,
    tangent_data: Option<&TangentMenuData>,
    events: &mut Vec<AppIntent>,
) {
    ui.label("➤ Route-Tool aktiv");
    ui.separator();

    button_intent(
        ui,
        "✓ Ausführen",
        AppIntent::RouteToolExecuteRequested,
        events,
    );
    button_intent(
        ui,
        "🔄 Neu berechnen",
        AppIntent::RouteToolRecreateRequested,
        events,
    );
    button_intent(ui, "✕ Abbrechen", AppIntent::RouteToolCancelled, events);

    // Tangenten-Auswahl (nur bei kubischer Kurve mit Nachbarn)
    if let Some(data) = tangent_data {
        let has_start = !data.start_options.is_empty();
        let has_end = !data.end_options.is_empty();

        if has_start || has_end {
            ui.separator();
            ui.label("🎯 Tangenten");

            if has_start {
                ui.label("Start:");
                for (source, label) in &data.start_options {
                    let is_sel = *source == data.current_start;
                    if ui.selectable_label(is_sel, label).clicked() {
                        events.push(AppIntent::RouteToolTangentSelected {
                            start: *source,
                            end: data.current_end,
                        });
                        ui.close();
                    }
                }
            }

            if has_start && has_end {
                ui.separator();
            }

            if has_end {
                ui.label("Ende:");
                for (source, label) in &data.end_options {
                    let is_sel = *source == data.current_end;
                    if ui.selectable_label(is_sel, label).clicked() {
                        events.push(AppIntent::RouteToolTangentSelected {
                            start: data.current_start,
                            end: *source,
                        });
                        ui.close();
                    }
                }
            }
        }
    }
}
