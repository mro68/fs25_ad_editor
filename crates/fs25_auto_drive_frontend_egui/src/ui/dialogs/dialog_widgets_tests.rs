use super::{
    dialog_three_action_row, dialog_three_action_row_enabled, dialog_two_action_row_enabled,
    resolve_three_action, resolve_two_action, DialogThreeAction, DialogTwoAction,
};

fn render_without_input<R>(render_fn: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let ctx = egui::Context::default();
    let mut render_fn = Some(render_fn);
    let mut result = None;
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
        let f = render_fn
            .take()
            .expect("render_without_input darf nur einmal aufgerufen werden");
        result = Some(f(ui));
    });

    result.expect("render_without_input muss ein Ergebnis setzen")
}

#[test]
fn dialog_two_action_row_enabled_without_click_returns_none() {
    let result =
        render_without_input(|ui| dialog_two_action_row_enabled(ui, "OK", "Abbrechen", true, true));

    assert_eq!(result, None);
}

#[test]
fn dialog_three_action_row_without_click_returns_none() {
    let result = render_without_input(|ui| dialog_three_action_row(ui, "A", "B", "C"));

    assert_eq!(result, None);
}

#[test]
fn dialog_three_action_row_enabled_without_click_returns_none() {
    let result = render_without_input(|ui| {
        dialog_three_action_row_enabled(ui, "A", "B", "C", true, true, true)
    });

    assert_eq!(result, None);
}

#[test]
fn resolve_two_action_respects_enabled_and_returns_expected_actions() {
    assert_eq!(
        resolve_two_action(true, true, true, false),
        Some(DialogTwoAction::Confirm)
    );
    assert_eq!(
        resolve_two_action(true, true, false, true),
        Some(DialogTwoAction::Cancel)
    );
    assert_eq!(resolve_two_action(false, true, true, false), None);
    assert_eq!(resolve_two_action(true, false, false, true), None);
}

#[test]
fn resolve_three_action_respects_enabled_and_returns_expected_actions() {
    assert_eq!(
        resolve_three_action(true, true, true, true, false, false),
        Some(DialogThreeAction::Primary)
    );
    assert_eq!(
        resolve_three_action(true, true, true, false, true, false),
        Some(DialogThreeAction::Secondary)
    );
    assert_eq!(
        resolve_three_action(true, true, true, false, false, true),
        Some(DialogThreeAction::Tertiary)
    );
    assert_eq!(
        resolve_three_action(false, true, true, true, false, false),
        None
    );
    assert_eq!(
        resolve_three_action(true, false, true, false, true, false),
        None
    );
    assert_eq!(
        resolve_three_action(true, true, false, false, false, true),
        None
    );
}
