use crate::shared::{t, EditorOptions, I18nKey, Language, ValueAdjustInputMode};
use crate::ui::common::{apply_wheel_step, apply_wheel_step_default};

/// Rendert die Werkzeug-Einstellungen (Eingabemodus, Snap-Radius, Mausrad-Schritt).
pub fn render_tools(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptValueAdjustMode));
        let current_label = match opts.value_adjust_input_mode {
            ValueAdjustInputMode::DragHorizontal => t(lang, I18nKey::OptValueAdjustDrag),
            ValueAdjustInputMode::MouseWheel => t(lang, I18nKey::OptValueAdjustWheel),
        };
        egui::ComboBox::from_id_salt("value_adjust_input_mode")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut opts.value_adjust_input_mode,
                        ValueAdjustInputMode::DragHorizontal,
                        t(lang, I18nKey::OptValueAdjustDrag),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .selectable_value(
                        &mut opts.value_adjust_input_mode,
                        ValueAdjustInputMode::MouseWheel,
                        t(lang, I18nKey::OptValueAdjustWheel),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
    })
    .response
    .on_hover_text(t(lang, I18nKey::OptValueAdjustModeHelp));
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSnapRadius));
        let r = ui.add(
            egui::DragValue::new(&mut opts.snap_scale_percent)
                .range(50.0..=2000.0)
                .speed(10.0)
                .suffix(" %"),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.snap_scale_percent, 10.0, 50.0..=2000.0);
        r.on_hover_text(t(lang, I18nKey::OptSnapRadiusHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptMouseWheelDistStep));
        let r = ui.add(
            egui::DragValue::new(&mut opts.mouse_wheel_distance_step_m)
                .range(0.01..=5.0)
                .speed(0.1)
                .suffix(" m"),
        );
        changed |= r.changed()
            | apply_wheel_step_default(ui, &r, &mut opts.mouse_wheel_distance_step_m, 0.01..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptMouseWheelDistStepHelp));
    });
    changed
}
