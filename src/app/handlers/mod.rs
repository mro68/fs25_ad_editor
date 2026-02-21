//! Feature-Handler für AppCommand-Verarbeitung.
//!
//! Jeder Handler gruppiert die Command-Ausführung eines Feature-Bereichs.
//! Der Controller dispatcht an die passende Handler-Funktion.

pub mod dialog;
pub mod editing;
pub mod file_io;
pub mod history;
pub mod route_tool;
pub mod selection;
pub mod view;
