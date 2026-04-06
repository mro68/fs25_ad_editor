# API der C-ABI-Host-Bridge

## Ueberblick

`fs25_auto_drive_host_bridge_ffi` ist der duenne Linux-first-Transportadapter ueber der kanonischen `HostBridgeSession`. Die Crate fuehrt keine zweite fachliche Surface ein: Mutationen laufen weiter ueber `HostSessionAction`, inklusive des ersten schreibenden Viewport-Input-Slices `HostSessionAction::SubmitViewportInput`, Dialoge ueber `HostDialogRequest`/`HostDialogResult`, Session-Polling ueber `HostSessionSnapshot` und der minimale Viewport-Read-Pfad ueber `HostViewportGeometrySnapshot`.

Technikentscheidung fuer Slice 0: JSON ueber eine kleine C-ABI mit `char*`-Payloads. Das ist heute baubar, direkt per `dart:ffi` nutzbar und vermeidet in dieser Runde den Overhead eines zusaetzlichen Codegen-Stacks wie `flutter_rust_bridge`.

## Exportierte Funktionen

| Symbol | Zweck |
|---|---|
| `fs25ad_host_bridge_session_new() -> *mut HostBridgeSession` | Erstellt eine neue kanonische Bridge-Session |
| `fs25ad_host_bridge_session_dispose(session)` | Gibt eine Session frei |
| `fs25ad_host_bridge_session_snapshot_json(session) -> *mut c_char` | Liefert `HostSessionSnapshot` als UTF-8-JSON |
| `fs25ad_host_bridge_session_apply_action_json(session, action_json) -> bool` | Liest `HostSessionAction` aus UTF-8-JSON und mutiert die Session |
| `fs25ad_host_bridge_session_take_dialog_requests_json(session) -> *mut c_char` | Liefert ein JSON-Array aus `HostDialogRequest` und drainet die Queue |
| `fs25ad_host_bridge_session_submit_dialog_result_json(session, result_json) -> bool` | Liest `HostDialogResult` aus UTF-8-JSON und fuehrt ihn in die Session zurueck |
| `fs25ad_host_bridge_session_viewport_geometry_json(session, width, height) -> *mut c_char` | Liefert `HostViewportGeometrySnapshot` als UTF-8-JSON |
| `fs25ad_host_bridge_last_error_message() -> *mut c_char` | Liefert die letzte thread-lokale Fehlernachricht als UTF-8-String |
| `fs25ad_host_bridge_string_free(value)` | Gibt von der Bibliothek allozierten UTF-8-String-Speicher frei |

## Transportvertrag

- Session-Handles sind opaque Pointer auf die kanonische `HostBridgeSession`.
- Alle JSON-Payloads verwenden exakt die bereits in `fs25_auto_drive_host_bridge` definierten DTOs.
- Dialog-Requests fuer `open_file`, `save_file`, `heightmap` und `background_map` bleiben reine Host-DTOs; nativer Picker oder Host-Fallback werden oberhalb dieser C-ABI entschieden.
- Schreibender Viewport-Input (`Resize`, Pointer-Drags/Taps, Scroll-Zoom) wird ohne neues ABI-Symbol als `HostSessionAction::SubmitViewportInput` ueber `fs25ad_host_bridge_session_apply_action_json(...)` transportiert.
- Fehler laufen minimal ueber `bool`/`null` plus `fs25ad_host_bridge_last_error_message()`.
- Der Geometry-Read-Pfad ist bewusst read-only und Slice-0-klein: Nodes, Connections, Marker sowie Kamera-/Viewport-Metadaten.

## Bewusste Nicht-Ziele von Slice 0

- Kein Flutter-only Parallelvertrag neben `HostBridgeSession` und den kanonischen Host-DTOs.
- Keine neue C-ABI-Funktion fuer den ersten Viewport-Input-Slice; der bestehende JSON-Action-Entry-Point bleibt verbindlich.
- Kein Route-Tool-, Lasso-, Doppelklick-, Rotations- oder Touch-Viewportvertrag ueber diese C-ABI in diesem Slice.
- Kein Codegen- oder Binding-Stack als Produktivvoraussetzung fuer den ersten Host-Slice.
- Kein finales Multi-Plattform-Packaging; Linux ist bewusst der erste produktive Transportpfad.

## Build-Artefakt

Auf Linux erzeugt `cargo build -p fs25_auto_drive_host_bridge_ffi` eine ladbare Shared Library `libfs25_auto_drive_host_bridge_ffi.so`.