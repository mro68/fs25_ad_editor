use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::apply_wheel_step;

/// Rendert die Hintergrundkarten-Einstellungen (Deckung, Fade-out).
pub fn render_background(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptBgOpacity));
        let r = ui.add(
            egui::Slider::new(&mut opts.bg_opacity, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |= r.changed() | apply_wheel_step(ui, &r, &mut opts.bg_opacity, 0.05, 0.0..=1.0);
        r.on_hover_text(t(lang, I18nKey::OptBgOpacityHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptBgOpacityAtMinZoom));
        let r = ui.add(
            egui::Slider::new(&mut opts.bg_opacity_at_min_zoom, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.bg_opacity_at_min_zoom, 0.05, 0.0..=1.0);
        r.on_hover_text(t(lang, I18nKey::OptBgOpacityAtMinZoomHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptBgFadeStartZoom));
        let r = ui.add(
            egui::DragValue::new(&mut opts.bg_fade_start_zoom)
                .range(0.1..=50.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.bg_fade_start_zoom, 0.5, 0.1..=50.0);
        r.on_hover_text(t(lang, I18nKey::OptBgFadeStartZoomHelp));
    });
    changed
}
