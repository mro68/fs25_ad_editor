# Doc Sync Changes (Commit 12)

Datum: 2026-04-17
Commit-Message: docs(sync): docstrings, api.md, roadmap, architecture sync

## Geaenderte Dateien
- crates/fs25_auto_drive_host_bridge/src/session/mod.rs
- crates/fs25_auto_drive_host_bridge/API.md
- docs/ROADMAP.md
- docs/ARCHITECTURE_PLAN.md
- .windsurf/rules/projekt.md
- memories/session/20260417_003625-best-practices-findings/doc_sync_changes.md
- memories/session/20260417_003625-best-practices-findings/developer_log.md

## Docstring-Sync
- `HostBridgeSession::mark_snapshot_dirty()` in `crates/fs25_auto_drive_host_bridge/src/session/mod.rs` auf aktuelle Dirty-Semantik nachgezogen:
  - Explizites Dirty-Marking nur fuer snapshot-transparente lokale Seams (`panel_properties_state_mut`, `viewport_input_context_mut`),
  - automatische Invalidierung fuer `chrome_state_mut` und `dialog_ui_state_mut` dokumentiert.

## API.md-Sync
- `crates/fs25_auto_drive_host_bridge/API.md` synchronisiert:
  - Snapshot-Semantik von `build_viewport_geometry_snapshot(...)` und `HostViewportGeometrySnapshot` auf vollstaendigen Geometry-Transport korrigiert.
  - Methodendokumentation fuer `chrome_state_mut()` und `dialog_ui_state_mut()` auf automatische Snapshot-Invalidierung aktualisiert.
  - Hinweise zu lokalen UI-Seams differenziert (automatisch vs. explizit `mark_snapshot_dirty()`).

## ROADMAP-Sync
- `docs/ROADMAP.md` um Abschnitt `Sync-Update 2026-04-17 (Commits 1-11)` ergaenzt:
  - CI-Security/Lint-Gates,
  - Host-Bridge Snapshot-Invarianten und Full-Geometry-Semantik,
  - Host-Bridge/FFI-Modularisierung,
  - Coverage/Proptest/Fuzzing-Baseline,
  - Governance/TODO-Tracking,
  - nachlaufende Test-/Doc-Contract-Fixes.

## Architektur-Sync
- `docs/ARCHITECTURE_PLAN.md` aktualisiert:
  - differenzierte Snapshot-Invalidierung lokaler Session-Seams,
  - FFI-/Polling-Hinweis auf vollstaendige, sortierte Geometry-Listen.

## Rules-Sync
- `.windsurf/rules/projekt.md` aktualisiert:
  - Host-Bridge-Vertrag um Full-Geometry-Snapshot und Dirty-Invalidierungsregeln erweitert.

## Validierung
- `make check-doc-contracts` erfolgreich.
