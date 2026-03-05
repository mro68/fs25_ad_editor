//! Feature-Handler fuer AppCommand-Verarbeitung.
//!
//! Jeder Handler gruppiert die Command-Ausfuehrung eines Feature-Bereichs.
//! Der Controller dispatcht an die passende Handler-Funktion.

pub mod dialog;
pub mod editing;
pub mod file_io;
pub mod helpers;
pub mod history;
pub mod route_tool;
pub mod selection;
pub mod view;
