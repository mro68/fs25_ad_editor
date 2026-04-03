use crate::shared::{t, EditorOptions, I18nKey, Language};
use crate::ui::common::apply_wheel_step_default;

/// Rendert die Copy/Paste-Einstellungen (Vorschau-Deckung).
pub fn render_copy_paste(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCopyPastePreviewOpacity));
        let r = ui.add(
            egui::Slider::new(&mut opts.copy_preview_opacity, 0.0..=1.0)
                .step_by(0.1)
                .fixed_decimals(2),
        );
        changed |= r.changed()
            | apply_wheel_step_default(ui, &r, &mut opts.copy_preview_opacity, 0.0..=1.0);
        r.on_hover_text(t(lang, I18nKey::OptCopyPastePreviewOpacityHelp));
    });
    changed
}
