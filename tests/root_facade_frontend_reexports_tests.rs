//! Integrationstest fuer Root-Reexports der Frontend-Module.

use fs25_auto_drive_editor::{render, ui};

#[test]
fn root_facade_reexports_render_and_ui_types() {
    let bounds = render::BackgroundWorldBounds {
        min_x: 0.0,
        max_x: 10.0,
        min_y: -5.0,
        max_y: 5.0,
    };

    assert!(bounds.max_x > bounds.min_x);
    assert!(bounds.max_y > bounds.min_y);

    let _input_state = ui::InputState::new();

    // Compile-time Absicherung, dass die Root-Fassade Frontend-Render-Typen weiterreicht.
    let _ = std::any::type_name::<render::RenderScene>();
    let _ = std::any::type_name::<render::RenderQuality>();
}
