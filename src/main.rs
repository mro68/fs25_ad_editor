//! Binary-Einstiegspunkt fuer den FS25 AutoDrive Editor.

mod editor_app;
mod runtime;

fn main() -> Result<(), eframe::Error> {
    runtime::run()
}
