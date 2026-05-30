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
    let mut confirm_clicked = false;
    let mut cancel_clicked = false;
    dialog_button_row_with_spacing(ui, |ui| {
        confirm_clicked = ui
            .add_enabled(confirm_enabled, egui::Button::new(confirm_label))
            .clicked();

        cancel_clicked = ui
            .add_enabled(cancel_enabled, egui::Button::new(cancel_label))
            .clicked();
    });

    resolve_two_action(
        confirm_enabled,
        cancel_enabled,
        confirm_clicked,
        cancel_clicked,
    )
}

fn resolve_two_action(
    confirm_enabled: bool,
    cancel_enabled: bool,
    confirm_clicked: bool,
    cancel_clicked: bool,
) -> Option<DialogTwoAction> {
    let mut result = None;
    if confirm_enabled && confirm_clicked {
        result = Some(DialogTwoAction::Confirm);
    }
    if cancel_enabled && cancel_clicked {
        result = Some(DialogTwoAction::Cancel);
    }
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
    dialog_three_action_row_enabled(
        ui,
        primary_label,
        secondary_label,
        tertiary_label,
        true,
        true,
        true,
    )
}

/// Rendert eine Drei-Aktions-Zeile mit aktivierbaren Buttons in der Reihenfolge
/// primary/secondary/tertiary.
///
/// Gibt die geklickte Aktion zurueck oder `None`, wenn in diesem Frame
/// keine Aktion ausgeloest wurde.
pub fn dialog_three_action_row_enabled(
    ui: &mut egui::Ui,
    primary_label: &str,
    secondary_label: &str,
    tertiary_label: &str,
    primary_enabled: bool,
    secondary_enabled: bool,
    tertiary_enabled: bool,
) -> Option<DialogThreeAction> {
    let mut primary_clicked = false;
    let mut secondary_clicked = false;
    let mut tertiary_clicked = false;

    dialog_button_row_with_spacing(ui, |ui| {
        primary_clicked = ui
            .add_enabled(primary_enabled, egui::Button::new(primary_label))
            .clicked();

        secondary_clicked = ui
            .add_enabled(secondary_enabled, egui::Button::new(secondary_label))
            .clicked();

        tertiary_clicked = ui
            .add_enabled(tertiary_enabled, egui::Button::new(tertiary_label))
            .clicked();
    });

    resolve_three_action(
        primary_enabled,
        secondary_enabled,
        tertiary_enabled,
        primary_clicked,
        secondary_clicked,
        tertiary_clicked,
    )
}

fn resolve_three_action(
    primary_enabled: bool,
    secondary_enabled: bool,
    tertiary_enabled: bool,
    primary_clicked: bool,
    secondary_clicked: bool,
    tertiary_clicked: bool,
) -> Option<DialogThreeAction> {
    let mut result = None;
    if primary_enabled && primary_clicked {
        result = Some(DialogThreeAction::Primary);
    }
    if secondary_enabled && secondary_clicked {
        result = Some(DialogThreeAction::Secondary);
    }
    if tertiary_enabled && tertiary_clicked {
        result = Some(DialogThreeAction::Tertiary);
    }
    result
}

#[cfg(test)]
#[path = "dialog_widgets_tests.rs"]
mod dialog_widgets_tests;
