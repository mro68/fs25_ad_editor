# API der Flutter-Bridge-Crate

## Ueberblick

`fs25_auto_drive_frontend_flutter_bridge` ist die duenne Flutter-seitige Adapter-/Kompat-Crate ueber `fs25_auto_drive_host_bridge`.

Die kanonische Session-, Action- und Snapshot-Logik lebt seit der Unified-Bridge-Einfuehrung in `fs25_auto_drive_host_bridge`. Diese Crate behaelt nur stabile Alias-Namen (`Engine*`, `FlutterBridgeSession`) fuer bestehende Rust-Call-Sites und spaetere Flutter-Transportanbindungen.

## Oeffentliche Module

| Modul | Verantwortung |
|---|---|
| `session` | Kompat-Alias fuer `HostBridgeSession` und `HostRenderFrameSnapshot` |
| `dto` | Kompat-Aliase fuer `Host*`-DTOs unter den bisherigen `Engine*`-Namen |

## Wichtige oeffentliche Typen

| Typ | Zweck |
|---|---|
| `FlutterBridgeSession` | Alias auf `fs25_auto_drive_host_bridge::HostBridgeSession` |
| `EngineRenderFrameSnapshot` | Alias auf `HostRenderFrameSnapshot` |
| `EngineSessionAction` | Alias auf `HostSessionAction` |
| `EngineSessionSnapshot` | Alias auf `HostSessionSnapshot` |
| `EngineDialogRequest` / `EngineDialogResult` | Alias auf `HostDialogRequest` / `HostDialogResult` |
| `EngineActiveTool` | Alias auf `HostActiveTool` |

## Kompatibilitaetsgarantie

- Bestehende Rust-Importpfade ueber `Engine*`-Namen bleiben erhalten.
- Verhalten und Semantik kommen aus der kanonischen Host-Bridge-Core-Crate.
- Diese Crate enthaelt bewusst keine eigene Controller-/State-Logik mehr.

## Beispiel

```rust
use fs25_auto_drive_frontend_flutter_bridge::{
    EngineSessionAction, FlutterBridgeSession,
};

let mut session = FlutterBridgeSession::new();
session.apply_action(EngineSessionAction::ToggleCommandPalette)?;

assert!(session.snapshot().show_command_palette);
```
