# Developer Log – Commit 1

Datum: 2026-04-17
Commit: 6373fac
Message: chore(engine/frontend): panic-hardening und clippy-hygiene

## Geaenderte Dateien
- crates/fs25_auto_drive_frontend_egui/src/ui/properties.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/dialog.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/editing.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/file_io.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/group.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/history.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/route_tool.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/selection.rs
- crates/fs25_auto_drive_engine/src/app/controller/by_feature/view.rs
- crates/fs25_auto_drive_engine/src/app/use_cases/background_map.rs

## Was wurde geaendert
- UI-Properties: `unwrap()`-Nutzung in Selektionspfaden robust gemacht.
  - Einzelnode-Rendering nutzt jetzt Guard auf leere Selektion statt `unwrap()`.
  - Zwei-Node-Rendering nutzt Guard auf unvollstaendige Selektion statt `unwrap()`.
- Controller by-feature: `unreachable!`-Catch-All-Arme in den benannten Dateien entfernt und durch fehlerbasierte Rueckgabe (`anyhow::bail!`) ersetzt, damit kein Panic-Pfad mehr auftritt.
- Engine Background-Map: unbenutzten Import (`GenericImageView`) im Testmodul entfernt (Clippy-Hygiene).

## Ausgefuehrte Checks
- `nocorrect cargo fmt --all -- --check` ✅
- `nocorrect cargo check` ✅
- `nocorrect cargo clippy -p fs25_auto_drive_engine -p fs25_auto_drive_frontend_egui --all-targets -- -D warnings` ✅

## Git/Scope-Hinweise
- Selektives Staging wurde verwendet (nur Commit-1-Scope-Dateien).
- Bereits zuvor gestagte, scope-fremde Datei `.github/agents/git-ops.agent.md` wurde vor dem Commit aus dem Index entfernt.
- Kein Push, kein Merge.

## Howto-Betroffenheit
- Keine inhaltliche Aenderung an UI-Workflows/Shortcuts; daher keine konkrete `docs/howto/*`-Aktualisierung erforderlich.

---

# Developer Log - Commit 2

Datum: 2026-04-17
Commit: f483d95
Message: ci(security): cargo-audit + cargo-deny integrieren

## Geaenderte Dateien
- .github/workflows/ci.yml
- deny.toml

## Was wurde geaendert
- Neuer CI-Job `security` in `.github/workflows/ci.yml`:
  - Installation von `cargo-audit` und `cargo-deny`
  - Ausfuehrung von `cargo audit`
  - Ausfuehrung von `cargo deny check --config deny.toml advisories bans sources`
- Neue `deny.toml` fuer `cargo-deny` mit Basiskonfiguration fuer:
  - advisories
  - bans
  - sources

## Ausgefuehrte Checks
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); YAML.load_file(".github/workflows/release.yml"); puts "workflow-yaml-ok"'` ✅
- `nocorrect cargo audit` ❌ (bestehende Advisories im Lockfile, u.a. `RUSTSEC-2026-0098`, `RUSTSEC-2026-0099`)
- `nocorrect cargo deny check --config deny.toml advisories bans sources` ❌ (`advisories FAILED, bans ok, sources ok`)

## Limits / Hinweise
- Die neuen Security-Checks sind integriert und lokal lauffaehig validiert.
- Fehlstatus stammt aus bereits vorhandenen Abhaengigkeitsproblemen, nicht aus YAML-/Tooling-Fehlkonfiguration.
- Keine Nebenrefactors; nur Commit-2-Scope umgesetzt.

---

# Developer Log - Commit 3

Datum: 2026-04-17
Commit: fcc78b7
Message: ci(quality): lint-hardening und todo-gate

## Geaenderte Dateien
- .github/workflows/ci.yml
- scripts/check_todo_gate.sh
- crates/fs25_auto_drive_host_bridge_ffi/src/lib.rs

## Was wurde geaendert
- CI-Workflow erweitert um einen dedizierten Lint-Hardening-Schritt fuer Domain-/Host-Bridge-Libraries (non-test-Targets):
  - `clippy::unwrap_used`
  - `clippy::expect_used`
  - `clippy::panic`
  - `clippy::todo`
- Neues CI-Gate `scripts/check_todo_gate.sh` eingefuehrt, das `todo!()` in Domain-/Host-Bridge-Produktionscode blockiert (`cargo clippy --lib -D clippy::todo`).
- Eng begrenzte FFI-Ausnahme dokumentiert:
  - In `into_c_string_ptr` ist `#[allow(clippy::expect_used)]` gesetzt, da der String vorher sanitisiert wird und die verbleibende `expect` nur die C-ABI-Konvertierungsgrenze absichert.

## Ausgefuehrte Checks
- `nocorrect cargo check` ✅
- `./scripts/check_todo_gate.sh` ✅
- `nocorrect cargo clippy --no-deps -p fs25_auto_drive_engine -p fs25_auto_drive_host_bridge -p fs25_auto_drive_host_bridge_ffi --lib -- -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic -W clippy::todo` ✅ (Warnings erwartet als Hardening-Baseline)

## Limits / Hinweise
- Strikte globale Deny-Regeln fuer `unwrap/expect/panic` sind im aktuellen Codebestand nicht sofort moeglich, ohne fachfremde Refactors ausserhalb des Commit-3-Scopes.
- Daher kleinste robuste Variante: harte CI-Sperre fuer `todo!()` (non-test) plus sichtbarer Hardening-Lauf fuer die weiteren Risikolints.

---

# Developer Log - Commit 4

Datum: 2026-04-17
Commit: 86fd6aa
Message: host-bridge: snapshot-dirty invarianten absichern

## Geaenderte Dateien
- crates/fs25_auto_drive_host_bridge/src/session/mod.rs
- crates/fs25_auto_drive_host_bridge_ffi/src/flutter_api.rs

## Was wurde geaendert
- Host-Bridge Snapshot-Dirty-Invariante gehaertet:
  - `dialog_ui_state_mut()` nutzt jetzt einen Drop-Guard (`HostDialogUiState`), der Snapshot-relevante Aenderungen (`show_command_palette`, `show_options_dialog`) sowie bereits gesetztes `chrome_dirty` erkennt und dann `snapshot_dirty` automatisch setzt.
  - `chrome_state_mut()` invalidiert den Session-Snapshot vorsorglich sofort, damit direkte lokale Mutationen keine stale Snapshot-Daten hinterlassen.
  - Docstrings der oeffentlichen Seams auf die neue Dirty-Semantik aktualisiert.
- Regression-/Invarianztests erweitert:
  - Neuer Regressionstest gegen stale Snapshot bei `dialog_ui_state_mut()` ohne manuelles `mark_snapshot_dirty()`.
  - Neuer Regressionstest fuer `chrome_state_mut()` mit automatischer Invalidation.
- FFI `unsafe impl Send/Sync` leicht abgesichert:
  - Buildzeit-Invarianztest (`Send + Sync`-Bound) fuer `FlutterSessionHandle`.
  - Thread-Smoketest mit geteiltem Handle (`Arc`) und parallelem Snapshot-/Action-Zugriff.

## Ausgefuehrte Checks
- `nocorrect cargo fmt --all` ✅
- `nocorrect cargo check` ✅
- `nocorrect cargo test -p fs25_auto_drive_host_bridge --lib` ✅
- `nocorrect cargo test -p fs25_auto_drive_host_bridge_ffi --lib` ✅

## Designentscheidungen (Commit-4-Scope)
- Keine grosse Refaktorierung (insb. nichts aus Commit 6 vorgezogen).
- `dialog_ui_state_mut()` bleibt read-mostly-freundlich: Dirty-Markierung nur bei erkannter Snapshot-Relevanz oder vorhandenem `chrome_dirty`.
- `chrome_state_mut()` wird konservativ als mutable Escape-Hatch behandelt und invalidiert immer, um unbeabsichtigte stale Snapshots sicher auszuschliessen.

## Howto-Betroffenheit
- Keine Aenderung an User-Workflows/Shortcuts; keine konkrete Aktualisierung in `docs/howto/*` erforderlich.

---

# Developer Log - Commit 5

Datum: 2026-04-17
Commit: 9bb9f9f
Message: host-bridge: snapshot-vertrag korrigieren

## Geaenderte Dateien
- crates/fs25_auto_drive_host_bridge/src/dispatch.rs
- crates/fs25_auto_drive_host_bridge/src/dispatch/snapshot.rs
- crates/fs25_auto_drive_host_bridge/src/dto/viewport.rs
- crates/fs25_auto_drive_host_bridge/src/session/mod.rs

## Was wurde geaendert
- Snapshot-Semantik im Host-Bridge-Code auf konsistente Full-Geometry-Bedeutung ausgerichtet:
  - Irrefuehrende "minimal"-Formulierungen in DTO-Docstrings und Snapshot-Build-Docstrings durch "vollstaendig" ersetzt.
  - Interne Builder-Benennung in `dispatch/snapshot.rs` auf Full-Geometry-Semantik angepasst.
- Snapshot-Tests auf die korrigierte Semantik ausgerichtet:
  - Bestehender Test in `dispatch.rs` auf Full-Geometry-Benennung umgestellt.
  - Neuer Regressionstest `build_viewport_geometry_snapshot_sorts_and_keeps_full_geometry_lists` ergaenzt, der Vollstaendigkeit und stabile Sortierung von Nodes/Connections/Markers absichert.

## Ausgefuehrte Checks
- `nocorrect cargo fmt --all` ✅
- `nocorrect cargo check` ✅
- `nocorrect cargo test -p fs25_auto_drive_host_bridge --lib` ✅ (92 passed)

## Offenes Doku-Delta fuer Commit 12
- `crates/fs25_auto_drive_host_bridge/API.md` an die nun explizite Full-Geometry-Semantik von `build_viewport_geometry_snapshot`/`HostViewportGeometrySnapshot` angleichen.
- Falls dort noch "klein/minimal" beschrieben: wording auf vollstaendigen Geometry-Transport korrigieren.
- Querverweise in uebergreifender Doku (`docs/ROADMAP.md`, ggf. `docs/ARCHITECTURE_PLAN.md`) erst im finalen Doku-Sync-Commit aktualisieren.

---

# Developer Log - Commit 6

Datum: 2026-04-17
Commit: d34753f
Message: refactor(host-bridge): session/dispatch modularisieren (phase 1)

## Geaenderte Dateien
- crates/fs25_auto_drive_host_bridge/src/session/mod.rs
- crates/fs25_auto_drive_host_bridge/src/session/mappings.rs (neu)
- crates/fs25_auto_drive_host_bridge/src/session/snapshots.rs (neu)
- crates/fs25_auto_drive_host_bridge/src/dispatch.rs
- crates/fs25_auto_drive_host_bridge/src/dispatch/tests.rs (neu)

## Was wurde geaendert
- `session` Phase-1-Split ohne API-Break:
  - Mapping-Logik aus `session/mod.rs` in `session/mappings.rs` extrahiert.
  - Snapshot-/Editing-Builder aus `session/mod.rs` in `session/snapshots.rs` extrahiert.
  - `session/mod.rs` auf Wiring und Session-Fassade fokussiert; oeffentliche Surface unveraendert.
- `dispatch` Phase-1-Split ohne Funktionsaenderung:
  - Groessenintensiven Inline-Testblock aus `dispatch.rs` in `dispatch/tests.rs` ausgelagert.
  - Produktionsmodul `dispatch.rs` enthaelt nur noch Submodul-Definitionen und Re-Exports.
- API/ABI-Stabilitaet gewahrt:
  - Keine Aenderung an oeffentlichen Re-Exports der Crate.
  - Keine Signatur-/Schemaaenderungen an Host-Bridge-DTOs oder Session-Methoden.

## Ausgefuehrte Checks
- `nocorrect cargo fmt --all` ✅
- `nocorrect cargo check -p fs25_auto_drive_host_bridge` ✅
- `nocorrect cargo test -p fs25_auto_drive_host_bridge` ✅ (92 passed)

## Verbleibende Restarbeit (bewusst ausserhalb Phase 1)
- Weitere Zerlegung von `session/mod.rs`-Impl-Methoden in thematische Untermodule (z. B. node-details/markers/connection-pair/snapshot-sync) in einer Folgephase.
- Optionale weitere Entkopplung sehr grosser Testdateien (`session`-Tests) in mehrere thematische Testmodule.
- Keine Vollsanierung von F03/F06/F08/F09 in diesem Commit; nur strukturelle Vorarbeit gemaess Commit-6-Scope.

---

# Developer Log – Commit 7

Datum: 2026-04-17
Commit: f685d67
Message: refactor(ffi): lib split + v4-stub-isolation

## Geaenderte Dateien (staged)
- `crates/fs25_auto_drive_host_bridge_ffi/src/lib.rs` — Monolith auf dünne Orchestrierungs-Crate-Root reduziert
- `crates/fs25_auto_drive_host_bridge_ffi/src/ffi_utils.rs` — NEU
- `crates/fs25_auto_drive_host_bridge_ffi/src/session_handle.rs` — NEU
- `crates/fs25_auto_drive_host_bridge_ffi/src/session_api.rs` — NEU
- `crates/fs25_auto_drive_host_bridge_ffi/src/texture_registration_v4.rs` — Duplikat-Makro entfernt

## Was wurde geaendert

### ffi_utils.rs (neu)
- Shared Error-State (`LAST_ERROR` thread_local), `clear_last_error`, `set_last_error`, `into_c_string_ptr`, `serialize_json`, `read_json_arg`, `decode_focus_node_id` extrahiert.

### session_handle.rs (neu)
- Opaker `HostBridgeSessionHandle`-Typ (Mutex-wrapped) und `with_session_mut`-Helper isoliert.

### session_api.rs (neu)
- `FS25AD_HOST_BRIDGE_ABI_VERSION` Konstante und alle 21 nicht-Flutter C-ABI-Exporte verschoben.
- Imports: `use crate::ffi_utils::*`, direkte Verwendung von `crate::session_handle::with_session_mut`.
- Makros per `use crate::{ffi_guard_bool, ffi_guard_ptr}` importiert (#[macro_export] in lib.rs).

### lib.rs (Umbau)
- Neue Modul-Deklarationen: `ffi_utils`, `session_handle`, `pub session_api`, optional `flutter_api`/`flutter_gpu`.
- Re-Exporte: `pub use session_api::*`, `pub use session_handle::HostBridgeSessionHandle`, `pub(crate) use session_handle::with_session_mut`, `pub(crate) use ffi_utils::{clear_last_error, set_last_error}`.
- `#[macro_export]` auf beiden Guard-Makros: Sichtbarkeit als Crate-Root-Symbole.
- Alle nicht-Flutter Session-Funktionen entfernt (jetzt in session_api.rs).
- Flutter-Imports korrekt mit `#[cfg(feature = "flutter")]` gegated (zuvor teilweise ungegated).
- Bug-Fix: `HostOverviewOptionsDialogSnapshot` Import fehlte ursprünglich.

### texture_registration_v4.rs
- Lokales `macro_rules! ffi_guard_bool!` (Duplikat) entfernt.
- `use crate::{..., ffi_guard_bool, ...}` stattdessen.

## Ausgefuehrte Checks
- `nocorrect cargo check -p fs25_auto_drive_host_bridge_ffi` ✅ (0 Fehler, 0 Warnings)
- `nocorrect cargo test -p fs25_auto_drive_host_bridge_ffi` ✅ (27 passed)

## ABI-Stabilitaet
- Alle `#[unsafe(no_mangle)]` Symbole unveraendert; nur die Quelldatei hat gewechselt.
- `pub use session_api::*` in lib.rs sorgt dafuer, dass externe Crates unveraenderten Zugriff haben.

## Verbleibende Restarbeit (bewusst ausserhalb Phase 1)
- `flutter_api.rs` hat eigene `decode_focus_node_id`-Kopie → zukuenftige Deduplizierung.
- Flutter-Exporte in lib.rs (~500 Zeilen) koennten in Folge-Commit weiter aufgeteilt werden.

---

# Developer Log - Commit 9

Datum: 2026-04-17
Commit: 0605779
Message: refactor(engine/frontend): argument-reduction + dead-code cleanup

## Geaenderte Dateien
- crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel/route_tool_panel.rs
- crates/fs25_auto_drive_frontend_egui/src/ui/edit_panel.rs
- crates/fs25_auto_drive_frontend_egui/src/ui/properties/selectors.rs
- crates/fs25_auto_drive_engine/src/app/tools/color_path/state.rs
- crates/fs25_auto_drive_engine/src/app/tools/color_path/sampling.rs

## Was wurde geaendert
- F15 (Argument-Reduktion):
  - In `route_tool_panel.rs` wurde der Audit-Hotspot `render_route_tool_panel(...)` von einer langen Parameterliste auf ein dediziertes Kontextobjekt (`RouteToolPanelContext`) umgestellt.
  - Das bisherige `#[allow(clippy::too_many_arguments)]` an dieser Funktion konnte dadurch entfernt werden.
  - Der Aufrufer in `edit_panel.rs` erstellt den Kontext zentral ueber `RouteToolPanelContext::new(...)`.
- F16 (Dead-Code-Cleanup):
  - In `selectors.rs` wurden ungenutzte vertikale Selector-Varianten inklusive `#[allow(dead_code)]` entfernt.
  - In `color_path/sampling.rs` wurden `dead_code`-Ausnahmen fuer testgenutzte Helper auf `#[cfg_attr(not(test), allow(dead_code))]` eingegrenzt.
  - In `color_path/state.rs` wurde `prepared_mask` analog enger begrenzt; `detection_bounds` bleibt mit expliziter Begruendung erlaubt, da der geplante Bounds-Flow noch nicht ins UI integriert ist.

## Ausgefuehrte Checks
- `nocorrect cargo fmt --all` ✅
- `nocorrect cargo check -p fs25_auto_drive_engine -p fs25_auto_drive_frontend_egui` ✅
- `nocorrect cargo clippy -p fs25_auto_drive_engine -p fs25_auto_drive_frontend_egui --all-targets -- -D warnings` ❌
  - Ursache: bestehende, scope-fremde Warnings/Clippy-Fehler in `crates/fs25_auto_drive_engine/src/app/tools/curve/geometry.rs` (bereits vor Commit-9-Scope vorhanden).
- `nocorrect cargo clippy -p fs25_auto_drive_engine -p fs25_auto_drive_frontend_egui --lib -- -D warnings` ✅
- `nocorrect cargo test -p fs25_auto_drive_engine -p fs25_auto_drive_frontend_egui --lib` ✅

## Git/Scope-Hinweise
- Initial wurde versehentlich ein lokaler Fehl-Commit mit bereits vorgemerkten Fremddateien erstellt; dieser wurde ohne Datenverlust via `git reset --mixed HEAD~1` zurueckgenommen.
- Anschliessend wurde strikt selektiv nur der Commit-9-Scope gestaged und neu committed.
- Kein Push, kein Merge.

## Howto-Betroffenheit
- Keine direkte Aenderung an Enduser-Workflows oder Shortcuts; `docs/howto/*` bleibt unveraendert.
- Tests in lib.rs koennen in eigenes Test-Modul verschoben werden.

---

# Developer Log – Commit 8

Datum: 2026-04-17
Commit: (noch zu erstellen)
Message: test(quality): coverage/proptest/fuzzing baseline

## Geaenderte Dateien

### Coverage-Basis
- `.github/workflows/ci.yml` — Neuer Job "Coverage Measurement" hinzugefuegt (llvm-cov mit HTML + LCOV Output)
- `Cargo.toml` (workspace root) — "fuzz" zu `workspace.members` hinzugefuegt

### Proptest-Property-Tests für Geometrie
- `crates/fs25_auto_drive_engine/Cargo.toml` — `proptest = "1.4"` zu dev-dependencies hinzugefuegt
- `crates/fs25_auto_drive_engine/src/app/tools/curve/geometry.rs` — 5 proptest Invarianten hinzugefuegt:
  - `prop_cubic_bezier_endpoints`: B(0)=P0, B(1)=P3 fuer kubische Bézier
  - `prop_quadratic_bezier_endpoints`: B(0)=P0, B(1)=P2 fuer quadratische Bézier
  - `prop_approx_length_monotonic`: Kurvenlaenge wächst mit Sample-Raten
  - `prop_approx_length_ge_direct_distance`: Dreiecksungleichung (Kurve >= Sehne)
  - `prop_compute_curve_positions_endpoints`: Arc-Length-Parametrisierung trifft exakt Start/End

### Fuzzing-Baseline (cargo-fuzz)
- `fuzz/Cargo.toml` — Neu, Workspace-Integration mit zwei Fuzz-Targets
- `fuzz/fuzz_targets/fuzz_xml_parser.rs` — Neu: Fuzzt `parse_autodrive_config`
- `fuzz/fuzz_targets/fuzz_curseplay_parser.rs` — Neu: Fuzzt `parse_curseplay`
- `fuzz/README.md` — Neu: Dokumentation zum lokalen Fuzzing + Corpus-Seeding

## Was wurde geaendert

### 1. Coverage-Integration (CI)

**Neue Job in ci.yml:**
```yaml
coverage:
  name: Coverage Measurement
  runs-on: ubuntu-latest
  steps:
    - Install llvm-tools-preview
    - cargo install cargo-llvm-cov
    - cargo llvm-cov --all --html --lcov --output-dir target/coverage
    - Upload coverage artifacts
```

Ablauf:
- Coverage wird separat vom Test-Job ausfuehrt (nicht blocking fuer Tests)
- HTML-Report + LCOV-Format fuer CI-Integration
- Artefakte fuer 30 Tage aufbewahrt
- Baseline: aktuell keine Schwelle durchgesetzt; dient zur Trend-Verfolgung

### 2. Proptest Property-Tests

**5 neue Tests in `crates/fs25_auto_drive_engine/src/app/tools/curve/geometry.rs`:**

```rust
#[cfg(test)]
mod proptest_invariants {
    use proptest::prelude::*;
    // ... 5 Tests mit parametrisierten Input-Generatoren
}
```

- Alle Tests nutzen proptest-Strategen für float-Ranges (-1000..1000 fuer Positionen)
- Invarianten-Checks: Numerische Grenzen, geometrische Eigenschaften
- Toleranzen fuer Floating-Point-Arithmetik bereits eingebaut

**Lokale Ausfuehrung:**
```bash
cargo test --package fs25_auto_drive_engine proptest_invariants -- --test-threads=1
```

Output (Beispiel):
```
test app::tools::curve::geometry::proptest_invariants::prop_cubic_bezier_endpoints ... ok
test app::tools::curve::geometry::proptest_invariants::prop_approx_length_ge_direct_distance ... ok
test app::tools::curve::geometry::proptest_invariants::prop_approx_length_monotonic ... ok
test app::tools::curve::geometry::proptest_invariants::prop_compute_curve_positions_endpoints ... ok
test app::tools::curve::geometry::proptest_invariants::prop_quadratic_bezier_endpoints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; X filtered out
```

### 3. Fuzzing mit cargo-fuzz

**Fuzz-Targets:**
1. `fuzz_xml_parser`: Fuzzt `parse_autodrive_config` mit beliebigem UTF-8-Input
2. `fuzz_curseplay_parser`: Fuzzt `parse_curseplay` mit beliebigem UTF-8-Input

**Workspace-Integration:**
- Workspace root `Cargo.toml` um `"fuzz"` Modul erweitet
- Fuzz-Projekt nutzt Workspace-Member-Struktur (kleaner als separate Duplikate)

**Lokale Ausfuehrung (Kommandos für Commit 12 / CI-Integration):**

```bash
# Einzelne Target fuzzen (benötigt nightly Rust)
cargo +nightly fuzz run fuzz_xml_parser -- -max_total_time=60
cargo +nightly fuzz run fuzz_curseplay_parser -- -max_total_time=60

# Mit Seed-Corpus (AutoDrive XML Samples)
cp ad_sample_data/AutoDrive_config.xml fuzz/corpus/fuzz_xml_parser/
cargo +nightly fuzz run fuzz_xml_parser

# Crash-Artefakte inspizieren
ls fuzz/artifacts/fuzz_xml_parser/
```

Weitere Details in `fuzz/README.md`.

## Ausgefuehrte Checks (Baseline-Validation)

- `nocorrect cargo check --workspace` ✅ (0 Fehler)
- `nocorrect cargo test --package fs25_auto_drive_engine proptest_invariants` ✅ (5 tests passed)
- `cargo +nightly fuzz build fuzz_xml_parser` ✅
- `cargo +nightly fuzz build fuzz_curseplay_parser` ✅
- CI-Syntax Check `.github/workflows/ci.yml` ✅

## Verifizierungs-Kommandos (lokal reproduzierbar)

### Property-Tests
```bash
# Kürzer (nur Proptest-Tests)
cargo test --package fs25_auto_drive_engine proptest_invariants

# Ausfuehrlich mit Output
cargo test --package fs25_auto_drive_engine proptest_invariants -- --test-threads=1 --nocapture
```

### Coverage
```bash
# Lokal komplett durchführen (braucht nightly)
cargo install cargo-llvm-cov
cargo llvm-cov --all --html --output-dir target/coverage
open target/coverage/index.html  # macOS
# oder
xdg-open target/coverage/index.html  # Linux
```

### Fuzzing
```bash
# Basis-Fuzz (1 Minute je Target)
cargo +nightly fuzz run fuzz_xml_parser -- -max_total_time=60

# Mit Testdaten
cp ad_sample_data/*.xml fuzz/corpus/fuzz_xml_parser/ 2>/dev/null
cargo +nightly fuzz run fuzz_xml_parser -- -max_total_time=120
```

## Scope-Hinweise

- **Kein neuer Feature-Code**: Rein Testinfrastruktur, Parser bleiben unverändert
- **CI-Integration nicht-blocking**: Coverage läuft parallel zu Tests, kein Impact auf bestehende Checks
- **Fuzzing noch nicht in CI**: Cargo-fuzz ist zeitaufwändig, manuell/nächtlich später aktivierbar
- **Proptest-Tests in Standard-Test-Suite**: Laufen mit `cargo test --workspace` mit (aktuell gering, aber skalierbar)

## Offene Doku-Delta für Commit 12

- `docs/DEVELOPMENT.md` um lokale Test-Kommandos ergänzen (Coverage, Proptest, Fuzzing-Reproduktion)
- `scripts/README.md` oder neuer `scripts/TESTING.md` für CI-Integration der neuen Test-Targets erstellen
- `.github/copilot-instructions.md` ggf. um Test-Hygiene erweitern (z. B. proptest für neue geometrische Funktionen)

## Findings-Abdeckung

- **F18** (Coverage-Messung): ✅ llvm-cov Baseline in CI
- **F19** (Property-Tests für Geometrie): ✅ 5 Proptest Invarianten für Bézier/Arc-Length
- **F20** (Fuzzing-Baseline XML): ✅ 2 Fuzz-Targets, Corpus-Setup dokumentiert

**Weitere Test-Infrastruktur (Optional für zukünftige Commits):**
- F21/F22 (CI Security): cargo-audit/cargo-deny bereits in Commit 2 integriert (prüfe ci.yml)
- F015 (todo-Gate): `scripts/check_todo_gate.sh` bereits aktiv in Commit 3
- Spline/Catmull-Rom Proptest: Zusätzliche Geometrie-Tests in Commit 9/10 denkbar

---

# Developer Log - Commit 10

Datum: 2026-04-17
Commit: (wird nach Commit-Erstellung eingetragen)
Message: docs(project): governance-dokumente und todo-tracking

## Geaenderte Dateien
- CONTRIBUTING.md
- CHANGELOG.md
- docs/TODO_TRACKING.md
- docs/ROADMAP.md
- docs/TOOL_ENCAPSULATION_REPORT.md
- docs/PERFORMANCE_REPORT_TEMPLATE.md
- memories/session/20260417_003625-best-practices-findings/developer_log.md

## Was wurde geaendert
- Governance-Basis eingefuehrt:
  - Neues Root-Dokument `CONTRIBUTING.md` mit Beitragspfad, Architekturgrenzen, Doku-Sync-Regeln und lokalem Check-Set.
- Changelog initialisiert:
  - Neues `CHANGELOG.md` mit `Unreleased`-Bereich und Initialeintrag fuer den Governance/TODO-Commit.
- TODO-Tracking zentralisiert (Finding F27):
  - Neues kanonisches Tracking in `docs/TODO_TRACKING.md` inkl. Pflege-Regeln und initialen offenen Punkten fuer F24-F27.
  - `docs/ROADMAP.md` um eine eindeutige Referenz auf das zentrale TODO-Tracking ergaenzt.
- Spezialdokumente datiert und status-klar markiert (Finding F26):
  - `docs/TOOL_ENCAPSULATION_REPORT.md` explizit als historischer Snapshot gekennzeichnet.
  - `docs/PERFORMANCE_REPORT_TEMPLATE.md` explizit als aktive Vorlage mit historischen Baselines gekennzeichnet.

## Ausgefuehrte Checks
- Plausibilitaetspruefung der Dokument- und Dateireferenzen (manuell) ✅
- Selektives Staging ausschliesslich der Commit-10-Dateien ✅

## Scope-Hinweise
- Vorgabe "nur Governance/TODO-Dokumente" eingehalten.
- Vorhandene unstaged Code-Aenderungen blieben unberuehrt.
- Kein Push, kein Merge.

