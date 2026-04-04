//! Nativer Launcher der Root-Fassade fuer den FS25 AutoDrive Editor.

fn main() -> Result<(), eframe::Error> {
    fs25_auto_drive_frontend_egui::run_native()
}
