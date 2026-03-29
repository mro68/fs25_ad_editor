use crate::shared::{t, EditorOptions, I18nKey, Language, SelectionStyle};
use crate::ui::common::apply_wheel_step;

/// Rendert die Selektions-Einstellungen (Groessenfaktor, Markierungsstil).
pub fn render_selection(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSelectionSizeFactor));
        let r = ui.add(
            egui::DragValue::new(&mut opts.selection_size_factor)
                .range(100.0..=200.0)
                .speed(1.0),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.selection_size_factor, 5.0, 100.0..=200.0);
        r.on_hover_text(t(lang, I18nKey::OptSelectionSizeFactorHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSelectionStyle));
        let current_label = match opts.selection_style {
            SelectionStyle::Ring => t(lang, I18nKey::OptSelectionStyleRing),
            SelectionStyle::Gradient => t(lang, I18nKey::OptSelectionStyleGradient),
        };
        egui::ComboBox::from_id_salt("selection_style")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (style, label) in [
                    (
                        SelectionStyle::Ring,
                        t(lang, I18nKey::OptSelectionStyleRing),
                    ),
                    (
                        SelectionStyle::Gradient,
                        t(lang, I18nKey::OptSelectionStyleGradient),
                    ),
                ] {
                    if ui
                        .selectable_value(&mut opts.selection_style, style, label)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
    })
    .response
    .on_hover_text(t(lang, I18nKey::OptSelectionStyleHelp));
    ui.separator();
    ui.label(t(lang, I18nKey::OptDoubleClickSegment));
    ui.horizontal(|ui| {
        changed |= ui
            .checkbox(
                &mut opts.segment_stop_at_junction,
                t(lang, I18nKey::OptSegmentStopAtJunction),
            )
            .on_hover_text(t(lang, I18nKey::OptSegmentStopAtJunctionHelp))
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSegmentMaxAngle));
        let r = ui.add(
            egui::DragValue::new(&mut opts.segment_max_angle_deg)
                .range(0.0..=180.0)
                .speed(1.0),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.segment_max_angle_deg, 5.0, 0.0..=180.0);
        r.on_hover_text(t(lang, I18nKey::OptSegmentMaxAngleHelp));
        if opts.segment_max_angle_deg == 0.0 {
            ui.weak(t(lang, I18nKey::OptSegmentDisabled));
        }
    });
    changed
}
