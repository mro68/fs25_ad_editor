//! Einzelne Einstellungs-Abschnitte fuer den Options-Dialog.
//!
//! Jede `render_*`-Funktion rendert einen thematischen Block und gibt `true`
//! zurueck wenn sich ein Wert geaendert hat.

mod background;
mod camera;
mod connections;
mod copy_paste;
mod lod;
mod markers;
mod node_behavior;
mod nodes;
mod overview_layers;
mod selection;
mod tools;

pub(super) use background::render_background;
pub(super) use camera::render_camera;
pub(super) use connections::render_connections;
pub(super) use copy_paste::render_copy_paste;
pub(super) use lod::render_lod;
pub(super) use markers::render_markers;
pub(super) use node_behavior::render_node_behavior;
pub(super) use nodes::render_nodes;
pub(super) use overview_layers::render_overview_layers;
pub(super) use selection::render_selection;
pub(super) use tools::render_tools;

/// Hilfsfunktion: Farbauswahl-Widget mit Float-Array-Speicherung.
fn color_edit(ui: &mut egui::Ui, label: &str, color: &mut [f32; 4]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let mut c = egui::Color32::from_rgba_unmultiplied(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );
        if ui.color_edit_button_srgba(&mut c).changed() {
            color[0] = c.r() as f32 / 255.0;
            color[1] = c.g() as f32 / 255.0;
            color[2] = c.b() as f32 / 255.0;
            color[3] = c.a() as f32 / 255.0;
            changed = true;
        }
    });
    changed
}
