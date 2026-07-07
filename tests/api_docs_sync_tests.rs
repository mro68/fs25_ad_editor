//! API-Doku-Sync-Test: fuehrt `scripts/check_api_docs_sync.sh` als
//! `cargo test`-Ziel aus.
//!
//! Zweck: veraltete/fehlende API.md-Vertraege sollen nicht nur ueber
//! `make check-doc-contracts`/CI auffallen, sondern auch bei einem lokalen
//! `cargo test --workspace`, ohne die Contract-Regeln doppelt in Rust
//! nachzubauen.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn api_docs_sync_script_reports_no_violations() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = workspace_root.join("scripts/check_api_docs_sync.sh");

    assert!(
        script.is_file(),
        "Erwartetes API-Doku-Sync-Skript fehlt: {}",
        script.display()
    );

    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&workspace_root)
        .output()
        .expect("scripts/check_api_docs_sync.sh konnte nicht ausgefuehrt werden");

    assert!(
        output.status.success(),
        "scripts/check_api_docs_sync.sh meldete veraltete/fehlende API.md-Vertraege:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
