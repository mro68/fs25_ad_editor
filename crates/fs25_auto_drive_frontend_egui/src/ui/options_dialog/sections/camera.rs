use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::apply_wheel_step_default;

/// Rendert die Kamera-Einstellungen (Zoom-Grenzen, Scroll-Schritt, Kompensation).
pub fn render_camera(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraZoomMin));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_min)
                .range(0.01..=10.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.camera_zoom_min, 0.01..=10.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraZoomMinHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraZoomMax));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_max)
                .range(1.0..=1000.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.camera_zoom_max, 1.0..=1000.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraZoomMaxHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraZoomStep));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_step)
                .range(1.01..=3.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.camera_zoom_step, 1.01..=3.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraZoomStepHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraScrollZoomStep));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_scroll_zoom_step)
                .range(1.01..=2.0)
                .speed(0.1),
        );
        changed |= r.changed()
            | apply_wheel_step_default(ui, &r, &mut opts.camera_scroll_zoom_step, 1.01..=2.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraScrollZoomStepHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptZoomCompensationMax));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.zoom_compensation_max, 1.0..=8.0)
                    .step_by(0.1)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptZoomCompensationMaxHelp));
        changed |= r.changed()
            | apply_wheel_step_default(ui, &r, &mut opts.zoom_compensation_max, 1.0..=8.0);
    });
    changed
}
