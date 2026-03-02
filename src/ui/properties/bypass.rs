use indexmap::IndexSet;

use crate::app::state::BypassState;
use crate::app::{AppIntent, RoadMap};

/// Rendert das Ausweichstrecken-Panel im Properties-Bereich.
///
/// Wird angezeigt wenn ≥ 2 Nodes selektiert sind und diese eine lineare Kette bilden.
pub fn render_bypass_panel(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    bypass_state: &mut BypassState,
    events: &mut Vec<AppIntent>,
) {
    // Nur anzeigen wenn die Selektion eine ordentliche Kette bildet
    let is_chain = road_map.is_resampleable_chain(selected_node_ids);
    if !is_chain {
        return;
    }

    ui.separator();
    ui.heading("Ausweichstrecke");

    ui.horizontal(|ui| {
        ui.label("Versatz:");
        ui.add(
            egui::DragValue::new(&mut bypass_state.offset)
                .speed(0.5)
                .range(-200.0..=200.0)
                .suffix(" m"),
        );
    });
    ui.label(if bypass_state.offset >= 0.0 {
        "Richtung: links"
    } else {
        "Richtung: rechts"
    });

    ui.horizontal(|ui| {
        ui.label("Knotenabstand:");
        ui.add(
            egui::DragValue::new(&mut bypass_state.base_spacing)
                .speed(0.5)
                .range(1.0..=50.0)
                .suffix(" m"),
        );
    });

    ui.add_space(4.0);

    if ui.button("⤴ Ausweichstrecke generieren").clicked() {
        events.push(AppIntent::GenerateBypassRequested);
    }
    ui.small("S-Kurven: halber Abstand · Parallel zur Kette");
}
