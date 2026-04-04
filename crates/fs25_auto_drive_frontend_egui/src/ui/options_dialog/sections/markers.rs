use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::apply_wheel_step_default;

/// Rendert die Marker-Darstellungseinstellungen (Pin-Groesse, Farben, Umrissstaerke).
pub fn render_markers(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!(
                "../../../../../../assets/icons/icon_map_pin.svg"
            ))
            .fit_to_exact_size(egui::Vec2::new(14.0, 14.0)),
        );
        ui.label(t(lang, I18nKey::OptMarkerSize));
        let r = ui.add(
            egui::DragValue::new(&mut opts.marker_size_world)
                .range(0.5..=10.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.marker_size_world, 0.5..=10.0);
        r.on_hover_text(t(lang, I18nKey::OptMarkerSizeHelp));
    });
    changed |= super::color_edit(ui, t(lang, I18nKey::OptMarkerColor), &mut opts.marker_color);
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptMarkerOutlineWidth));
        let r = ui.add(
            egui::DragValue::new(&mut opts.marker_outline_width)
                .range(0.01..=0.3)
                .speed(0.1)
                .fixed_decimals(3),
        );
        changed |= r.changed()
            | apply_wheel_step_default(ui, &r, &mut opts.marker_outline_width, 0.01..=0.3);
        r.on_hover_text(t(lang, I18nKey::OptMarkerOutlineWidthHelp));
    });
    changed
}
