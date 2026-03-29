use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::apply_wheel_step;

/// Rendert die LOD/Mindestgroessen-Einstellungen (Pixel-Untergrenzen + Node-Decimation).
pub fn render_lod(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.label(t(lang, I18nKey::OptLodMinSizes));
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodNodes));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_node_size_px, 0.0..=20.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptLodNodesHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_node_size_px, 1.0, 0.0..=20.0);
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodConnections));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_connection_width_px, 0.0..=10.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptLodConnectionsHelp));
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.min_connection_width_px, 0.5, 0.0..=10.0);
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodArrows));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_arrow_size_px, 0.0..=20.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptLodArrowsHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_arrow_size_px, 1.0, 0.0..=20.0);
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodMarkers));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_marker_size_px, 0.0..=30.0)
                    .step_by(1.0)
                    .fixed_decimals(0),
            )
            .on_hover_text(t(lang, I18nKey::OptLodMarkersHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_marker_size_px, 1.0, 0.0..=30.0);
    });
    ui.separator();
    ui.label(t(lang, I18nKey::OptLodNodeDecimation));
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodDecimationSpacing));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.node_decimation_spacing_px, 0.0..=50.0)
                    .step_by(1.0)
                    .fixed_decimals(0),
            )
            .on_hover_text(t(lang, I18nKey::OptLodDecimationSpacingHelp));
        changed |= r.changed()
            | apply_wheel_step(
                ui,
                &r,
                &mut opts.node_decimation_spacing_px,
                1.0,
                0.0..=50.0,
            );
    });
    changed
}
