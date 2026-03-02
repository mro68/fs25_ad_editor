//! Runtime-Bootstrap fuer den eframe-Start.

use crate::editor_app::EditorApp;
use eframe::egui;

/// Startet die Anwendung inkl. Logger und eframe-Window.
pub(crate) fn run() -> Result<(), eframe::Error> {
    init_logger();

    log::info!(
        "FS25 AutoDrive Editor v{} startet...",
        env!("CARGO_PKG_VERSION")
    );

    eframe::run_native(
        "FS25 AutoDrive Editor",
        native_options(),
        Box::new(|cc| {
            // SVG/Bild-Loader fuer egui installieren (benoetigt fuer Toolbar-Icons)
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let render_state = cc.wgpu_render_state.as_ref().ok_or_else(|| {
                anyhow::anyhow!("wgpu nicht verfuegbar: Renderer konnte nicht initialisiert werden")
            })?;
            Ok(Box::new(EditorApp::new(render_state)))
        }),
    )
}

fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("FS25 AutoDrive Editor"),
        renderer: eframe::Renderer::Wgpu,
        multisampling: 4,
        ..Default::default()
    }
}
