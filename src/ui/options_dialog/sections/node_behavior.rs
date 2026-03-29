use crate::shared::{t, EditorOptions, I18nKey, Language};

/// Rendert die Node-Verhalten-Einstellungen (Reconnect beim Loeschen, Verbindung teilen).
pub fn render_node_behavior(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    if ui
        .checkbox(
            &mut opts.reconnect_on_delete,
            t(lang, I18nKey::OptReconnectOnDelete),
        )
        .on_hover_text(t(lang, I18nKey::OptReconnectOnDeleteHelp))
        .changed()
    {
        changed = true;
    }
    if ui
        .checkbox(
            &mut opts.split_connection_on_place,
            t(lang, I18nKey::OptSplitConnectionOnPlace),
        )
        .on_hover_text(t(lang, I18nKey::OptSplitConnectionOnPlaceHelp))
        .changed()
    {
        changed = true;
    }
    changed
}
