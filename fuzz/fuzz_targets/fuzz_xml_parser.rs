#![no_main]

use fs25_auto_drive_editor::xml::parser::parse_autodrive_config;
use libfuzzer_sys::fuzz_target;

/// Fuzz-Target für AutoDrive-Config XML-Parser.
///
/// Dieser Target überprüft, ob der XML-Parser mit adversarialem Input
/// robust umgehen kann, ohne zu crashen oder unkontrolliertes Speicherwachstum zu verursachen.
fuzz_target!(|data: &[u8]| {
    // Versuche, den XML-Input als UTF-8 zu dekodieren und als AutoDrive-Konfiguration zu parsen.
    if let Ok(xml_str) = std::str::from_utf8(data) {
        let _ = parse_autodrive_config(xml_str);
    }
});
