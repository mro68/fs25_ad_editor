//! Build-Script fuer das Flutter-FFI-Crate.
//!
//! Unter dem `flutter`-Feature wuerde hier `flutter_rust_bridge_codegen` aufgerufen
//! um Dart-Bindings aus den Rust-Signaturen in `flutter_api.rs` zu generieren.
//!
//! # TODO(flutter-codegen)
//! Vollstaendige Codegen-Integration erfordert:
//! 1. `flutter_rust_bridge_codegen` als Build-Dependency
//! 2. Dart-SDK im PATH des Build-Systems
//! 3. Ausgabepfad fuer generierte Dart-Dateien (z.B. `../flutter_app/lib/src/rust/`)

fn main() {
    // Feature-Check: Codegen nur wenn flutter-Feature aktiv
    if std::env::var("CARGO_FEATURE_FLUTTER").is_ok() {
        println!(
            "cargo:warning=flutter feature aktiv: frb-Codegen-Stub (Dart-Generierung deaktiviert)"
        );
        // TODO(flutter-codegen): flutter_rust_bridge_codegen aufrufen
        // flutter_rust_bridge_codegen::init_env_logger(log::LevelFilter::Info);
        // flutter_rust_bridge_codegen::run(Config { ... });
    }
}
