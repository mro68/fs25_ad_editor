//! Empty Area Menu: Rechtsklick auf leeren Bereich (kein Node gehovered).
//!
//! Zeigt Tool-Auswahl und ggf. Streckenteilung-Controls, wenn diese aktiviert ist.

use super::{button_intent, render_streckenteilung};
use crate::app::{state::DistanzenState, AppIntent, EditorTool};

pub fn render_empty_area_menu(
    ui: &mut egui::Ui,
    distanzen_state: &mut DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    ui.label("🛠 Werkzeug");
    ui.separator();
    button_intent(
        ui,
        "⭘ Auswahl (1)",
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Select,
        },
        events,
    );
    button_intent(
        ui,
        "⚡ Verbinden (2)",
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Connect,
        },
        events,
    );
    button_intent(
        ui,
        "➕ Node hinzufügen (3)",
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::AddNode,
        },
        events,
    );

    // Streckenteilung-Controls, falls aktiv
    if distanzen_state.active {
        ui.separator();
        render_streckenteilung(ui, distanzen_state, events);
    }
}
