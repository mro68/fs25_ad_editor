#![no_main]

use libfuzzer_sys::fuzz_target;
use fs25_auto_drive_editor::xml::load_config_from_bytes;

/// Fuzz-Target für AutoDrive-Config XML-Parser.
/// 
/// Dieser Target überprüft, ob der XML-Parser mit adversarialem Input
/// robust umgehen kann, ohne zu crashen oder unkontrolliertes Speicherwachstum zu verursachen.
fuzz_target!(|data: &[u8]| {
    // Versuche, den XML-Input als AutoDrive-Konfiguration zu parsen.
    // Der Parser sollte nie panics oder Out-of-Memory auslösen.
    let _ = load_config_from_bytes(data);
});
