//! Wiederverwendbare Widget-Bausteine fuer modale Dialoge.

/// Rendert eine standardisierte Schaltflaechen-Zeile fuer modale Dialoge.
///
/// Gibt `Some(true)` zurueck wenn der Bestaetigen-Button geklickt wurde,
/// `Some(false)` wenn der Abbrechen-Button geklickt wurde,
/// `None` wenn kein Button geklickt wurde (kein Event in diesem Frame).
///
/// # Beispiel
/// ```ignore
/// if let Some(confirmed) = dialog_button_row(ui, "Bereinigen", "Abbrechen") {
///     events.push(if confirmed { AppIntent::Confirm } else { AppIntent::Cancel });
/// }
/// ```
pub fn dialog_button_row(
    ui: &mut egui::Ui,
    confirm_label: &str,
    cancel_label: &str,
) -> Option<bool> {
    let mut result = None;
    ui.horizontal(|ui| {
        if ui.button(confirm_label).clicked() {
            result = Some(true);
        }
        if ui.button(cancel_label).clicked() {
            result = Some(false);
        }
    });
    result
}
