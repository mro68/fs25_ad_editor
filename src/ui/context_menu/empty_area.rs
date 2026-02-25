//! Empty Area Menu: Rechtsklick auf leeren Bereich (kein Node gehovered).
//!
//! Zeigt Tool-Auswahl und ggf. Streckenteilung-Controls, wenn diese aktiviert ist.

use super::button_intent;
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
        ui.label("Streckenteilung:");

        let prev_distance = distanzen_state.distance;
        ui.horizontal(|ui| {
            ui.label("Abstand:");
            ui.add(
                egui::DragValue::new(&mut distanzen_state.distance)
                    .speed(0.5)
                    .range(1.0..=25.0)
                    .suffix(" m"),
            );
        });
        if (distanzen_state.distance - prev_distance).abs() > f32::EPSILON {
            distanzen_state.by_count = false;
            distanzen_state.sync_from_distance();
        }

        let prev_count = distanzen_state.count;
        ui.horizontal(|ui| {
            ui.label("Nodes:");
            ui.add(
                egui::DragValue::new(&mut distanzen_state.count)
                    .speed(1.0)
                    .range(2..=10000),
            );
        });
        if distanzen_state.count != prev_count {
            distanzen_state.by_count = true;
            distanzen_state.sync_from_count();
            if distanzen_state.distance < 1.0 {
                distanzen_state.distance = 1.0;
                distanzen_state.sync_from_distance();
            }
        }

        ui.add_space(4.0);
        if ui.button("✓ Übernehmen").clicked() {
            events.push(AppIntent::ResamplePathRequested);
            distanzen_state.deactivate();
            ui.close();
        }
        if ui.button("✕ Verwerfen").clicked() {
            distanzen_state.deactivate();
            ui.close();
        }
    }
}
