//! XML Import/Export für AutoDrive-Konfigurationen.
//!
//! Dieses Modul implementiert das Parsen und Schreiben von AutoDrive XML-Configs.
//! Das Format nutzt "Structure of Arrays" (parallele Listen in XML-Tags).

///
/// Dieses Modul implementiert das Parsen und Schreiben von AutoDrive XML-Configs.
/// Das Format nutzt "Structure of Arrays" (parallele Listen in XML-Tags).
pub mod parser;
pub mod writer;

pub use parser::parse_autodrive_config;
pub use writer::write_autodrive_config;
