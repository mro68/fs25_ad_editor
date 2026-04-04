# Flutter Bridge API

## Ueberblick

`fs25_auto_drive_frontend_flutter_bridge` definiert die kleine Rust-seitige Andockstelle fuer ein spaeteres Flutter-Frontend. Die Crate haengt nur von `fs25_auto_drive_engine` ab und erzwingt bewusst noch kein FFI- oder Flutter-SDK.

## Oeffentliche Oberflaeche

- `FlutterBridgeSession` — Session-Fassade mit `dispatch()` und `snapshot()`
- `EngineSessionSnapshot` — serialisierbare Zustandszusammenfassung
- `EngineSelectionSnapshot` — serialisierbare Auswahl
- `EngineViewportSnapshot` — serialisierbarer Viewport-Stand

## Scope-Cut

Diese Crate ist absichtlich klein gehalten. Sie stellt nur Rust-seitige Session- und DTO-Seams bereit; Transport, Method-Channel, `flutter_rust_bridge` oder andere SDK-Details folgen spaeter.
