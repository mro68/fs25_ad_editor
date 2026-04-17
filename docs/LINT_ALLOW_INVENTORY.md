# Governance: Inventar der Lint-Ausnahmen (`allow(...)`)

Stand: 2026-04-17  
Scope: Rust-Quelltexte (`**/*.rs`) im Repository, ohne `.git/` und `target/`.

## Reproduzierbarkeit

Primärer Weg:

```bash
make report-lint-allows
```

Direkter Script-Aufruf:

```bash
bash ./scripts/report_lint_allows.sh
```

Methodik:
- Das Script listet alle Attributzeilen mit `allow(...)` aus Rust-Dateien.
- Erfasst werden `#[allow(...)]`, `#![allow(...)]` und `#[cfg_attr(..., allow(...))]`.
- Das Script ist report-only (kein Fail-Gate).

## Inventar (29 Ausnahmen)

| Nr. | Datei/Ort | Genaue Ausnahme | Begruendung (im Code / TODO) | Geplanter Abbaupfad |
|---:|---|---|---|---|
| 1 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/sampling.rs:215` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Nutzung nachziehen oder ungenutzte Funktion entfernen; falls nur Test-Helfer, unter `#[cfg(test)]` verschieben; danach `allow` entfernen und mit `cargo clippy` absichern. |
| 2 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/sampling.rs:288` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Nutzung nachziehen oder ungenutzte Funktion entfernen; falls nur Test-Helfer, unter `#[cfg(test)]` verschieben; danach `allow` entfernen und mit `cargo clippy` absichern. |
| 3 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/skeleton.rs:49` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Nutzung klarziehen oder tote API streichen; bei test-only Logik in Testmodul verschieben; danach `allow` entfernen. |
| 4 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/skeleton.rs:491` | `#[cfg_attr(not(test), allow(dead_code))]` | TODO: Keine Begruendung im Code | Test-Helfer und Produktivlogik trennen (z. B. `#[cfg(test)]`-Modul); `cfg_attr(...allow...)` entfernen, sobald nur noch notwendiger Pfad verbleibt. |
| 5 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/skeleton.rs:760` | `#[cfg_attr(not(test), allow(dead_code))]` | TODO: Keine Begruendung im Code | Test-Helfer und Produktivlogik trennen (z. B. `#[cfg(test)]`-Modul); `cfg_attr(...allow...)` entfernen, sobald nur noch notwendiger Pfad verbleibt. |
| 6 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/state.rs:210` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Nutzung nachziehen oder Element entfernen; bei geplanter spaeterer Nutzung mit Ticket referenzieren und Termin zur Entfernung setzen. |
| 7 | `crates/fs25_auto_drive_engine/src/app/tools/color_path/state.rs:235` | `#[allow(dead_code)]` | Im Code: "Geplantes Feature: Rect-Begrenzung fuer die Erkennung" | Feature finalisieren oder temporaeren Code bis zur Implementierung entfernen; nach Aktivierung `allow` entfernen und Tests erweitern. |
| 8 | `crates/fs25_auto_drive_engine/src/app/tools/common/mod.rs:18` | `#[allow(unused_imports)]` | TODO: Keine Begruendung im Code | Re-Exports/Imports konsolidieren und nur genutzte Imports behalten; danach `allow` entfernen. |
| 9 | `crates/fs25_auto_drive_engine/src/app/tools/common/mod.rs:20` | `#[allow(unused_imports)]` | TODO: Keine Begruendung im Code | Re-Exports/Imports konsolidieren und nur genutzte Imports behalten; danach `allow` entfernen. |
| 10 | `crates/fs25_auto_drive_engine/src/shared/i18n/mod.rs:319` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Nicht genutzte Hilfsroutine entfernen oder produktiv nutzen; anschliessend `allow` entfernen. |
| 11 | `crates/fs25_auto_drive_frontend_egui/src/ui/context_menu/mod.rs:99` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Funktionssignatur ueber Parameter-Struct/Context-Objekt entkoppeln, Argumentanzahl reduzieren, `allow` entfernen. |
| 12 | `crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel/route_tool_panel.rs:36` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | UI-Parameter in klaren Konfig-/State-Strukturen buendeln; Signatur verschlanken; `allow` entfernen. |
| 13 | `crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel/route_tool_panel.rs:474` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | UI-Parameter in klaren Konfig-/State-Strukturen buendeln; Signatur verschlanken; `allow` entfernen. |
| 14 | `crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel/route_tool_panel.rs:498` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | UI-Parameter in klaren Konfig-/State-Strukturen buendeln; Signatur verschlanken; `allow` entfernen. |
| 15 | `crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel/route_tool_panel.rs:527` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | UI-Parameter in klaren Konfig-/State-Strukturen buendeln; Signatur verschlanken; `allow` entfernen. |
| 16 | `crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel/route_tool_panel.rs:554` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | UI-Parameter in klaren Konfig-/State-Strukturen buendeln; Signatur verschlanken; `allow` entfernen. |
| 17 | `crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel.rs:26` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Parameterobjekt fuer Panel-Renderpfad einfuehren und Signatur entschlacken; danach `allow` entfernen. |
| 18 | `crates/fs25_auto_drive_frontend_egui/src/ui/group_boundary_overlay.rs:134` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Overlay-Input als dedizierte Struct zusammenfassen; Signatur vereinfachen; `allow` entfernen. |
| 19 | `crates/fs25_auto_drive_frontend_egui/src/ui/group_boundary_overlay.rs:173` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Overlay-Input als dedizierte Struct zusammenfassen; Signatur vereinfachen; `allow` entfernen. |
| 20 | `crates/fs25_auto_drive_frontend_egui/src/ui/group_overlay.rs:38` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Overlay-Argumente in Kontextobjekt kapseln; `allow` entfernen. |
| 21 | `crates/fs25_auto_drive_frontend_egui/src/ui/input/viewport_collect.rs:33` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Eingabedaten in Struct kapseln und Funktion splitten; danach `allow` entfernen. |
| 22 | `crates/fs25_auto_drive_frontend_egui/src/ui/properties.rs:94` | `#[allow(clippy::too_many_arguments)]` | TODO: Keine Begruendung im Code | Properties-Renderparameter als Context-Struct konsolidieren; `allow` entfernen. |
| 23 | `crates/fs25_auto_drive_frontend_egui/src/ui/properties/selectors.rs:116` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Unbenutzte Selektor-Hilfe entfernen oder verwenden; danach `allow` entfernen. |
| 24 | `crates/fs25_auto_drive_frontend_egui/src/ui/properties/selectors.rs:135` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Unbenutzte Selektor-Hilfe entfernen oder verwenden; danach `allow` entfernen. |
| 25 | `crates/fs25_auto_drive_host_bridge_ffi/src/flutter_api.rs:47` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | FFI-Symbol nur behalten, wenn extern konsumiert; sonst entfernen oder Sichtbarkeit reduzieren; dann `allow` entfernen. |
| 26 | `crates/fs25_auto_drive_host_bridge_ffi/src/flutter_api.rs:63` | `#[allow(clippy::arc_with_non_send_sync)]` | Im Code: "HostBridgeSession ist !Send, aber FFI-Zugriff ist seriell" | FFI-Sessionzugriff mit klarer Single-Thread-Garantie typisieren (z. B. Wrapper ohne missverstaendliches `Arc`-Signal) und `allow` danach entfernen. |
| 27 | `crates/fs25_auto_drive_host_bridge_ffi/src/flutter_gpu.rs:122` | `#[allow(clippy::arc_with_non_send_sync)]` | Im Code: "HostBridgeSession ist !Send, aber FFI-Zugriff ist seriell" | FFI-Sessionzugriff mit klarer Single-Thread-Garantie typisieren (z. B. Wrapper ohne missverstaendliches `Arc`-Signal) und `allow` danach entfernen. |
| 28 | `crates/fs25_auto_drive_host_bridge_ffi/src/shared_texture_v2.rs:40` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Unbenutzte API entfernen oder aktiv nutzen; anschliessend `allow` entfernen. |
| 29 | `crates/fs25_auto_drive_render_wgpu/src/export_core.rs:186` | `#[allow(dead_code)]` | TODO: Keine Begruendung im Code | Export-Helfer validieren (benoetigt/nicht benoetigt) und bereinigen; danach `allow` entfernen. |

## Hinweis zur Governance

Dieses Dokument ist bewusst ein Inventar mit Abbaupfaden (CP-02) und ersetzt keine direkte Bereinigung. Die tatsaechliche Entfernung der Ausnahmen erfolgt in separaten Commit-Punkten.
