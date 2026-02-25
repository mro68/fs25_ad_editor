//! Route Tool Menu: Route-Tool aktiv mit pending input.

use super::button_intent;
use crate::app::AppIntent;

pub fn render_route_tool_menu(ui: &mut egui::Ui, events: &mut Vec<AppIntent>) {
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

    ui.separator();
    button_intent(ui, "↶ Rückgängig", AppIntent::UndoRequested, events);
}
