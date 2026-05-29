//! Wiederverwendbare Widget-Bausteine fuer modale Dialoge.

/// Standard-Abstand oberhalb einer Dialog-Buttonzeile.
pub const DIALOG_BUTTON_ROW_TOP_SPACING: f32 = 8.0;

/// Standard-Abstand zwischen Buttons in einer Dialog-Buttonzeile.
pub const DIALOG_BUTTON_ROW_ITEM_SPACING: f32 = 8.0;

/// Aktionsergebnis einer Dialogzeile mit genau zwei Aktionen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogTwoAction {
    /// Primaere Bestaetigungsaktion.
    Confirm,
    /// Abbruch- oder Verwerfungsaktion.
    Cancel,
}

/// Aktionsergebnis einer Dialogzeile mit genau drei Aktionen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogThreeAction {
    /// Erste (primaere) Aktion.
    Primary,
    /// Zweite (sekundaere) Aktion.
    Secondary,
    /// Dritte Aktion, typischerweise Abbrechen.
    Tertiary,
}

/// Rendert eine Dialog-Buttonzeile mit benanntem Standard-Spacing.
pub fn dialog_button_row_with_spacing(ui: &mut egui::Ui, add_buttons: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(DIALOG_BUTTON_ROW_TOP_SPACING);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = DIALOG_BUTTON_ROW_ITEM_SPACING;
        add_buttons(ui);
    });
}

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

/// Rendert eine Zwei-Aktions-Zeile mit aktivierbaren Buttons.
///
/// Gibt die geklickte Aktion zurueck oder `None`, wenn in diesem Frame
/// keine Aktion ausgeloest wurde.
pub fn dialog_two_action_row_enabled(
    ui: &mut egui::Ui,
    confirm_label: &str,
    cancel_label: &str,
    confirm_enabled: bool,
    cancel_enabled: bool,
) -> Option<DialogTwoAction> {
    let mut result = None;
    dialog_button_row_with_spacing(ui, |ui| {
        let confirm_clicked = ui
            .add_enabled(confirm_enabled, egui::Button::new(confirm_label))
            .clicked();
        if confirm_clicked {
            result = Some(DialogTwoAction::Confirm);
        }

        let cancel_clicked = ui
            .add_enabled(cancel_enabled, egui::Button::new(cancel_label))
            .clicked();
        if cancel_clicked {
            result = Some(DialogTwoAction::Cancel);
        }
    });
    result
}

/// Rendert eine Drei-Aktions-Zeile in der Reihenfolge primary/secondary/tertiary.
///
/// Gibt die geklickte Aktion zurueck oder `None`, wenn in diesem Frame
/// keine Aktion ausgeloest wurde.
pub fn dialog_three_action_row(
    ui: &mut egui::Ui,
    primary_label: &str,
    secondary_label: &str,
    tertiary_label: &str,
) -> Option<DialogThreeAction> {
    let mut result = None;
    dialog_button_row_with_spacing(ui, |ui| {
        if ui.button(primary_label).clicked() {
            result = Some(DialogThreeAction::Primary);
        }
        if ui.button(secondary_label).clicked() {
            result = Some(DialogThreeAction::Secondary);
        }
        if ui.button(tertiary_label).clicked() {
            result = Some(DialogThreeAction::Tertiary);
        }
    });
    result
}
