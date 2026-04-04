use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::{apply_wheel_step, apply_wheel_step_default};

/// Rendert die Node-Darstellungseinstellungen (Groesse, Farben, Hitbox).
pub fn render_nodes(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptNodeSizeWorld));
        let r = ui.add(
            egui::DragValue::new(&mut opts.node_size_world)
                .range(0.1..=5.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step_default(ui, &r, &mut opts.node_size_world, 0.1..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptNodeSizeWorldHelp));
    });
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorDefault),
        &mut opts.node_color_default,
    );
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorSubprio),
        &mut opts.node_color_subprio,
    );
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorSelected),
        &mut opts.node_color_selected,
    );
    changed |= super::color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorWarning),
        &mut opts.node_color_warning,
    );
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptHitboxScale));
        let r = ui.add(
            egui::DragValue::new(&mut opts.hitbox_scale_percent)
                .range(50.0..=500.0)
                .speed(5.0)
                .suffix(" %"),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.hitbox_scale_percent, 10.0, 50.0..=500.0);
        r.on_hover_text(t(lang, I18nKey::OptHitboxScaleHelp));
    });
    changed
}
