//! Architektur-Boundary-Test: fuehrt `scripts/check_layer_boundaries.sh` als
//! `cargo test`-Ziel aus.
//!
//! Zweck: Layer-Grenzenverletzungen ("no forbidden imports") sollen nicht nur
//! ueber `make check-layers`/CI sichtbar werden, sondern auch bei einem lokalen
//! `cargo test --workspace`, ohne die Regel-Logik doppelt in Rust nachzubauen.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn layer_boundaries_script_reports_no_violations() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = workspace_root.join("scripts/check_layer_boundaries.sh");

    assert!(
        script.is_file(),
        "Erwartetes Architektur-Gate-Skript fehlt: {}",
        script.display()
    );

    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&workspace_root)
        .output()
        .expect("scripts/check_layer_boundaries.sh konnte nicht ausgefuehrt werden");

    assert!(
        output.status.success(),
        "scripts/check_layer_boundaries.sh meldete Layer-Grenzenverletzungen:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
