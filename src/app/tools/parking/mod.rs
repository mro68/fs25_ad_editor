//! Parkplatz-Layout-Tool: erzeugt N bidirektionale Parkreihen mit Wendekreis,
//! unidirektionaler Ein-/Ausfahrt und Map-Markern an den Parkpositionen.

mod config_ui;
mod geometry;
mod lifecycle;
mod state;

pub use state::ParkingConfig;
pub use state::ParkingTool;

#[cfg(test)]
mod tests;
