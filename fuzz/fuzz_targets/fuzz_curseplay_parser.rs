#![no_main]

use libfuzzer_sys::fuzz_target;
use fs25_auto_drive_editor::xml::curseplay::parse_curseplay;

/// Fuzz-Target für CursePlay-XML-Parser.
/// 
/// Dieser Target überprüft, ob der CursePlay-XML-Parser mit adversarialem Input
/// robust umgehen kann, ohne zu crashen oder unkontrolliertes Speicherwachstum zu verursachen.
fuzz_target!(|data: &[u8]| {
    // Versuche, den XML-Input als UTF-8 zu dekodieren und als CursePlay-Datei zu parsen.
    if let Ok(xml_str) = std::str::from_utf8(data) {
        let _ = parse_curseplay(xml_str);
    }
});
