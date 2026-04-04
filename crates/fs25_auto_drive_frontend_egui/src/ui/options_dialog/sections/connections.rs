use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::apply_wheel_step_default;

/// Rendert die Verbindungs-Darstellungseinstellungen (Breite, Pfeilgroessen, Farben).
pub fn render_connections(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptConnectionWidthMain));
        let r = ui.add(
            egui::DragValue::new(&mut opts.connection_thickness_world)
                .range(0.01..=2.0)
                .speed(0.1),
        );
        changed |= r.changed()
            | apply_wheel_step_default(ui, &r, &mut opts.connection_thickness_world, 0.01..=2.0);
        r.on_hover_text(t(lang, I18nKey::OptConnectionWidthMainHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptConnectionWidthSubprio));
        let r = ui.add(
            egui::DragValue::new(&mut opts.connection_thickness_subprio_world)
                .range(0.01..=2.0)
                .speed(0.1),
        );
        changed |= r.changed()
            | apply_wheel_step_default(
                ui,
                &r,
                &mut opts.connection_thickness_subprio_world,
                0.01..=2.0,
            );
        r.on_hover_text(t(lang, I18nKey::OptConnectionWidthSubprioHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptArrowLength));
        let r = ui.add(
            egui::DragValue::new(&mut opts.arrow_length_world)
                .range(0.1..=5.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.arrow_length_world, 0.1..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptArrowLengthHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptArrowWidth));
        let r = ui.add(
            egui::DragValue::new(&mut opts.arrow_width_world)
                .range(0.1..=5.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.arrow_width_world, 0.1..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptArrowWidthHelp));
    });
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptConnectionColorRegular),
        &mut opts.connection_color_regular,
    );
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptConnectionColorDual),
        &mut opts.connection_color_dual,
    );
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptConnectionColorReverse),
        &mut opts.connection_color_reverse,
    );
    changed
}
