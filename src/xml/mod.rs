//! XML Import/Export fuer AutoDrive-Konfigurationen.
//!
//! Dieses Modul implementiert das Parsen und Schreiben von AutoDrive XML-Configs.
//! Das Format nutzt "Structure of Arrays" (parallele Listen in XML-Tags).

/// Curseplay XML-Import/Export fuer Feldumrandungen (`<customField>`-Format).
pub mod curseplay;
/// XML-Parser fuer AutoDrive-Konfigurationen (quick-xml, Structure of Arrays).
pub mod parser;
/// XML-Writer fuer AutoDrive-Konfigurationen mit lueckenloser ID-Neunummerierung.
pub mod writer;

pub use curseplay::{parse_curseplay, write_curseplay};
pub use parser::parse_autodrive_config;
pub use writer::write_autodrive_config;
