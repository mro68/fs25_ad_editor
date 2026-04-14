# Developer Log

## Aufgabe

Commit 1 fuer Flutter-Bridge-Parity: vorhandene Bridge-Seams additiv ueber die FRB-Surface in `api.rs` und `flutter_api.rs` verfuegbar machen.

## Geaenderte Dateien

- `crates/fs25_auto_drive_host_bridge_ffi/src/flutter_api.rs`
  - Neue FRB-Delegates fuer `take_dialog_requests()`, `submit_dialog_result()`, `build_route_tool_viewport_snapshot()` und `connection_pair(a, b)` hinzugefuegt.
  - Bestehende FRB-Seams fuer `build_host_ui_snapshot()`, `build_host_chrome_snapshot()` und `build_viewport_overlay_snapshot()` unveraendert verifiziert.
  - Roundtrip-Tests fuer Dialog-Request-Drain, Dialog-Submit, Route-Tool-Viewport und Connection-Pair ergaenzt.
- `crates/fs25_auto_drive_host_bridge_ffi/src/api.rs`
  - Duenne FRB-Delegate-Funktionen fuer die neuen `flutter_api.rs`-Entry-Points ergaenzt.
- `crates/fs25_auto_drive_host_bridge_ffi/API.md`
  - Flutter-Control-Plane-API auf die tatsaechlichen Signaturen synchronisiert.
  - Neue FRB-Funktionen fuer Dialog-Drain, Dialog-Submit, Route-Tool-Viewport und Connection-Pair dokumentiert.
- `docs/ROADMAP.md`
  - Flutter-Backend Phase 1 um den abgeschlossenen FRB-Paritaets-Slice fuer bestehende Bridge-Seams ergaenzt.
- `crates/fs25_auto_drive_render_wgpu/src/external_texture/vulkan_linux.rs`
  - Minimalen Let-Chain-Clippy-Fix fuer eine bestehende `collapsible_if`-Warnung im Linux-DMA-BUF-Pfad eingezogen, damit der geforderte `clippy`-Lauf mit `flutter-linux` sauber durchlaeuft.

## Verifikation

- `nocorrect cargo check -p fs25_auto_drive_host_bridge_ffi --features flutter-linux`
  - Erfolgreich.
  - Hinweis zum aktiven Flutter-Feature-Codegen-Stub ausgegeben, aber kein Fehler.
- `nocorrect cargo test -p fs25_auto_drive_host_bridge`
  - Erfolgreich.
  - 76 Tests bestanden.
- `nocorrect cargo clippy -p fs25_auto_drive_host_bridge_ffi --features flutter-linux -- -D warnings`
  - Erfolgreich nach dem minimalen Clippy-Fix in `crates/fs25_auto_drive_render_wgpu`.
  - Nur der bekannte Flutter-Feature-Hinweis blieb als Build-Hinweis sichtbar.

## Zusatznotizen

- `HostDialogRequest` und `HostDialogResult` waren bereits serde-kompatibel; es wurden keine neuen DTOs eingefuehrt.
- `docs/howto/` ist fuer diesen Commit nicht betroffen.
- Ein separater `cargo test --lib`-Lauf wurde nicht zusaetzlich ausgefuehrt, weil der Nutzer explizit den staerkeren Paketlauf `cargo test -p fs25_auto_drive_host_bridge` vorgegeben hat.

---

## Aufgabe

Commit 2 fuer Flutter-Bridge-Parity: fehlende `HostSessionAction`-Varianten und das bidirektionale Dispatch-Mapping fuer alle im egui-Frontend direkt ausgeloesten, niederfrequenten AppIntents schliessen.

## Geaenderte Dateien

- `crates/fs25_auto_drive_host_bridge/src/dto/actions.rs`
  - `HostSessionAction` um die fehlenden View-, Background-, Heightmap-, Overview-, Marker-Dialog-, Group-Edit-, Resample-, Dedup-, Dissolve- und Trace-All-Fields-Aktionen erweitert.
  - JSON-Roundtrip-Tests fuer die neuen Payload-Formen ergaenzt.
- `crates/fs25_auto_drive_host_bridge/src/dispatch/mappings.rs`
  - `map_intent_to_host_action(...)` und `map_host_action_to_intent(...)` fuer die neuen Varianten vollstaendig erweitert.
- `crates/fs25_auto_drive_host_bridge/src/dispatch.rs`
  - Mapping-Tests fuer die neuen Commit-2-Aktionen ergaenzt.
- `crates/fs25_auto_drive_frontend_egui/src/editor_app/mod.rs`
  - `intent_requires_canonical_host_action(...)` um die neuen kanonischen Actions erweitert.
  - Guard-Tests fuer neue View-/Group-/Extras-Intents ergaenzt.
- `crates/fs25_auto_drive_host_bridge/API.md`
  - Oeffentliche Host-Bridge-API auf die erweiterten `HostSessionAction`-Familien synchronisiert.

## Hinzugefuegte HostSessionAction-Varianten

- Datei-/Dialog-Follow-ups:
  - `ClearHeightmap`
  - `ConfirmHeightmapWarning`
  - `CancelHeightmapWarning`
  - `BrowseOverviewZip`
  - `GenerateOverviewFromZip { path }`
  - `SelectZipBackgroundFile { zip_path, entry_name }`
  - `CancelZipBrowser`
  - `ConfirmOverviewOptions`
  - `CancelOverviewOptions`
  - `DismissPostLoadDialog`
  - `ConfirmSaveBackgroundAsOverview`
  - `DismissSaveBackgroundAsOverview`
  - `ConfirmDeduplication`
  - `CancelDeduplication`
- View-/Background:
  - `ZoomIn`
  - `ZoomOut`
  - `CenterOnNode { node_id }`
  - `SetRenderQuality { quality }`
  - `ToggleBackgroundVisibility`
  - `ScaleBackground { factor }`
- Marker-Dialog-Lifecycle:
  - `OpenCreateMarkerDialog { node_id }`
  - `OpenEditMarkerDialog { node_id }`
  - `CancelMarkerDialog`
- Selection-/Group-/Resample-Flow:
  - `InvertSelection`
  - `StartResampleSelection`
  - `ApplyCurrentResample`
  - `StartGroupEdit { record_id }`
  - `ApplyGroupEdit`
  - `CancelGroupEdit`
  - `OpenGroupEditTool { record_id }`
  - `GroupSelectionAsGroup`
  - `RemoveSelectedNodesFromGroup`
  - `SetGroupBoundaryNodes { record_id, entry_node_id, exit_node_id }`
  - `RecomputeNodeSegmentSelection { world_pos, additive }`
  - `ToggleGroupLock { segment_id }`
  - `DissolveGroup { segment_id }`
  - `ConfirmDissolveGroup { segment_id }`
- Extras:
  - `OpenTraceAllFieldsDialog`
  - `ConfirmTraceAllFields { spacing, offset, tolerance, corner_angle, corner_rounding_radius, corner_rounding_max_angle_deg }`
  - `CancelTraceAllFields`

## Bereits vorhanden und bewusst nicht dupliziert

- Bereits vorhanden waren die Datei-Grundaktionen `OpenFile`, `Save`, `SaveAs`, `RequestHeightmapSelection`, `RequestBackgroundMapSelection`, `GenerateOverview`, `CurseplayImport`, `CurseplayExport`.
- Bereits vorhanden waren die View-/Chrome-Basisaktionen `ResetCamera`, `ZoomToFit`, `ZoomToSelectionBounds`, `ToggleCommandPalette`, `SetEditorTool`, `ApplyOptions`, `ResetOptions`, `OpenOptionsDialog`, `CloseOptionsDialog`, `Undo`, `Redo`.
- Bereits vorhanden waren die Node-/Marker-Abschlussaktionen `SetNodeFlag`, `CreateMarker`, `UpdateMarker`, `RemoveMarker`.
- Bereits vorhanden waren die Selection-/Clipboard-Basisaktionen `DeleteSelected`, `SelectAll`, `ClearSelection`, `CopySelection`, `PasteStart`, `PasteConfirm`, `PasteCancel`.
- Bereits vorhanden waren die Connection- und Route-Tool-Familien einschliesslich `RouteTool { ... }`, `AddConnection`, `RemoveConnectionBetween`, `SetConnectionDirection`, `SetConnectionPriority`, `ConnectSelectedNodes`, `SetAllConnectionsDirectionBetweenSelected`, `InvertAllConnectionsBetweenSelected`, `SetAllConnectionsPriorityBetweenSelected`, `RemoveAllConnectionsBetweenSelected`.
- Kein neues `MoveSelectedNodes`-HostAction-DTO hinzugefuegt: egui emittiert diese Mutation nicht direkt als niederfrequenten Intent; Verschieben laeuft bereits ueber `HostSessionAction::SubmitViewportInput`.

## Verifikation

- `nocorrect cargo fmt --all`
  - Erfolgreich.
- `nocorrect cargo check -p fs25_auto_drive_host_bridge`
  - Erfolgreich.
- `nocorrect cargo test -p fs25_auto_drive_host_bridge`
  - Erfolgreich.
  - 79 Tests bestanden, 0 fehlgeschlagen.
- `nocorrect cargo clippy -p fs25_auto_drive_host_bridge -- -D warnings`
  - Erfolgreich.
  - Keine Warnungen.
- `nocorrect cargo check -p fs25_auto_drive_frontend_egui`
  - Erfolgreich.
  - Zusaetzlicher Guard-Check, weil `editor_app/mod.rs` mitgeaendert wurde.

## Zusatznotizen

- `docs/howto/` ist fuer diesen Commit nicht betroffen.
- Unverwandte Arbeitsbaum-Aenderungen in `crates/fs25_auto_drive_host_bridge_ffi/src/flutter_api.rs` und `crates/fs25_auto_drive_render_wgpu/src/external_texture/vulkan_linux.rs` wurden fuer diesen Commit bewusst nicht angefasst.
- Commit erstellt: `f756a5c` (`feat(host-bridge): add missing HostSessionAction variants for full egui parity`)

---

## Aufgabe

Commit 3 fuer Flutter-Bridge-Parity: serialisierbaren `HostDialogSnapshot` und die zugehoerigen Dialog-DTOs fuer alle im egui-Frontend gerenderten Dialoge/Popups einfuehren und ueber die Flutter-Control-Plane als JSON delegieren.

## Geaenderte Dateien

- `crates/fs25_auto_drive_host_bridge/src/dto/dialogs.rs`
  - Bestehendes Dialog-DTO-Modul um `HostDialogSnapshot` sowie die einzelnen Dialog-Snapshot-DTOs erweitert.
  - Host-neutrale Enum-/Struct-Mappings fuer `FieldDetectionSource`, `OverviewSourceContext` und die Overview-Layer eingefuehrt.
  - Serde-Roundtrip-Test fuer den kompletten Dialog-Snapshot ergaenzt.
- `crates/fs25_auto_drive_host_bridge/src/dto/mod.rs`
  - Neue Dialog-Snapshot-Typen re-exportiert.
  - `Engine*`-Kompatibilitaets-Aliase fuer die neue Dialog-Snapshot-Familie ergaenzt.
- `crates/fs25_auto_drive_host_bridge/src/lib.rs`
  - Crate-Root-Re-Exports fuer `HostDialogSnapshot` und die neuen Dialog-DTOs/Aliase erweitert.
- `crates/fs25_auto_drive_host_bridge/src/session/mod.rs`
  - Builder fuer `HostDialogSnapshot` aus `HostLocalDialogState` plus relevanten `EditorOptions` ergaenzt.
  - Neue Session-Methode `dialog_snapshot()` hinzugefuegt.
  - Session-Test fuer die Abbildung lokaler Dialog-Drafts auf den Snapshot ergaenzt.
- `crates/fs25_auto_drive_host_bridge/Cargo.toml`
  - Direkte Abhaengigkeit auf `fs25_map_overview` fuer das stabile Mapping von `FieldDetectionSource` ergaenzt.
- `Cargo.lock`
  - Lockfile-Sync fuer die neue direkte `fs25_map_overview`-Abhaengigkeit von `fs25_auto_drive_host_bridge` nachgezogen.
- `crates/fs25_auto_drive_host_bridge_ffi/src/flutter_api.rs`
  - Neuer FRB-Delegate `flutter_session_dialog_snapshot_json()` hinzugefuegt.
  - Roundtrip-Test fuer den JSON-Dialog-Snapshot ergaenzt.
- `crates/fs25_auto_drive_host_bridge_ffi/src/api.rs`
  - Duenner FRB-Re-Export fuer `flutter_session_dialog_snapshot_json()` hinzugefuegt.
- `crates/fs25_auto_drive_host_bridge/API.md`
  - Oeffentliche Host-Bridge-API fuer `HostDialogSnapshot` und `HostBridgeSession::dialog_snapshot()` dokumentiert.
- `crates/fs25_auto_drive_host_bridge_ffi/API.md`
  - Flutter-Control-Plane-Doku um `flutter_session_dialog_snapshot_json()` erweitert.
- `docs/ROADMAP.md`
  - Abgeschlossenen Flutter-Control-Plane-Slice fuer den Dialog-Snapshot als erledigt markiert.
- `docs/ARCHITECTURE_PLAN.md`
  - Control-Plane-Beschreibung um den neuen Dialog-Snapshot-Delegate ergaenzt.

## Verifikation

- `nocorrect cargo check -p fs25_auto_drive_host_bridge`
  - Erfolgreich.
- `nocorrect cargo test -p fs25_auto_drive_host_bridge`
  - Erfolgreich.
  - 81 Tests bestanden, 0 fehlgeschlagen.
- `nocorrect cargo clippy -p fs25_auto_drive_host_bridge -- -D warnings`
  - Erfolgreich.
  - Keine Warnungen.
- `nocorrect cargo check -p fs25_auto_drive_host_bridge_ffi --features flutter-linux`
  - Erfolgreich.
  - Nur der bekannte Build-Hinweis des FRB-Codegen-Stubs unter aktivem `flutter`-Feature blieb sichtbar.

## Zusatznotizen

- `crates/fs25_auto_drive_host_bridge/src/session/chrome_state.rs` blieb bewusst unveraendert; der Snapshot liest nur daraus.
- `docs/howto/` ist fuer diesen Commit nicht betroffen.
- Nach dem Hauptcommit blieb durch die neue Direktabhaengigkeit ein kleiner Lockfile-Sync sowie rustfmt-only Line-Wrapping in einigen betroffenen Dateien offen. Diese werden absichtlich per Folgecommit nachgezogen, statt den gerade erzeugten Hauptcommit zu amendieren.