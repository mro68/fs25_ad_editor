# Deprecation-Policy fuer Kompat-Re-Exports und Typaliase

Stand: 2026-07-07

## Zweck

Dieses Dokument definiert verbindliche Regeln fuer alle Kompatibilitaets-Re-Exports und Typaliase im Workspace
(ausgenommen `crates/fs25_auto_drive_host_bridge_ffi`, das eigene C-ABI-Versionierungsregeln in seinem `API.md` fuehrt).
Ohne einen dokumentierten Zeitplan bleiben Kompat-Aliase erfahrungsgemaess dauerhaft im Code, ohne dass klar ist,
ob sie noch gebraucht werden. Diese Policy erzwingt fuer jeden Kompat-Eintrag eine explizite Einordnung.

## Zwei Kategorien

### 1. Sunset-Aliase (mit geplanter Entfernung)

Werden mit Rusts `#[deprecated(...)]`-Attribut markiert, sobald ein Nachfolger existiert, und in einem konkreten
Meilenstein entfernt.

**Aktuelles Beispiel:** ColorPath-Legacy-Vertrag in `crates/fs25_auto_drive_engine`
([API.md](../crates/fs25_auto_drive_engine/API.md)):

| Legacy-Element | Ersetzt durch | Sunset-Ziel |
|---|---|---|
| `ColorPathPanelPhase::Preview` / `CenterlinePreview` / `JunctionEdit` / `Finalize` | `Idle` / `Sampling` / `Editing` | CP-11 |
| `ColorPathPanelAction::NextPhase` / `PrevPhase` / `ComputePreview` / `BackToSampling` | `StartSampling` / `Compute` / `Accept` / `Reset` | CP-11 |
| `ColorPathPanelState::can_next` / `can_back` | `can_compute` / `can_accept` | CP-11 |

Verhalten bis zum Sunset: Engine setzt `can_next`/`can_back` konstant auf `false`; die DTO-Schicht faltet alle
Legacy-Phasen auf `"editing"` zurueck. `#[allow(deprecated)]`-Seam siehe
[`color_path/config_ui.rs`](../crates/fs25_auto_drive_engine/src/app/tools/color_path/config_ui.rs).

### 2. Perma-Compat-Aliase (keine geplante Entfernung)

Reine Namensaliase fuer externe Konsumenten (Flutter-/FFI-Call-Sites), die aus Stabilitaetsgruenden dauerhaft
bestehen bleiben. Werden **nicht** mit `#[deprecated]` markiert (das waere fachlich falsch — sie sind nicht
"veraltet", sondern ein bewusster zweiter Name fuer denselben Typ), muessen aber hier gelistet sein.

**Aktuelles Beispiel:** `Engine*`-Typaliase in
[`crates/fs25_auto_drive_host_bridge/src/dto/mod.rs`](../crates/fs25_auto_drive_host_bridge/src/dto/mod.rs)
(~30 Eintraege), z. B. `EngineSessionAction = HostSessionAction`, `EngineRenderFrameSnapshot = HostRenderFrameSnapshot`,
`FlutterBridgeSession = HostBridgeSession` (siehe
[`session/mod.rs`](../crates/fs25_auto_drive_host_bridge/src/session/mod.rs)). Kanonischer Name ist stets das
`Host*`-Pendant; neue Engine-interne Nutzung soll ausschliesslich `Host*`-Typen verwenden, `Engine*`-Aliase sind
nur fuer bestehende externe Call-Sites gedacht.

**Weiteres Beispiel (kein Rust-Attribut, sondern JSON-Vertrag):** `fs25_map_overview::FieldDetectionSource` — der
fruehere JSON-Wert `fruits_gdm` ist seit Release 2.1.0 kein gueltiger Wert mehr im Feldquellenvertrag (siehe
[`crates/fs25_map_overview/API.md`](../crates/fs25_map_overview/API.md)). Dies zeigt, dass Deprecation nicht immer
ueber Rust-Code-Attribute laeuft, sondern auch reine Wire-Format-Vertraege betreffen kann — solche Faelle gehoeren
ebenfalls in dieses Dokument.

## Pflichtangaben pro Eintrag

Jeder neue Kompat-Eintrag (Alias, Re-Export, veralteter JSON-Wert) muss bei Einfuehrung hier ergaenzt werden mit:

1. **Kanonischer Name** — der bevorzugte, aktuelle Bezeichner.
2. **Kategorie** — Sunset oder Perma-Compat (siehe oben).
3. **Einfuehrungsgrund/-datum** — z. B. Migrations-Commit, betroffene externe Konsumenten.
4. **Sunset-Ziel** — konkreter Meilenstein (z. B. `CP-11`) oder explizit "kein Sunset geplant".
5. **Migrationshinweis** — wie bestehender Code auf den kanonischen Namen umgestellt wird.

## Prozessregeln

- Wird ein Sunset-Alias tatsaechlich entfernt, muss `docs/ROADMAP.md` im selben Commit aktualisiert werden
  (abgeschlossenes Feature als `[x]` markieren) und der Eintrag hier als "entfernt (Datum/Commit)" markiert werden,
  nicht geloescht — fuer Nachvollziehbarkeit.
- Neue Perma-Compat-Aliase duerfen nur fuer echte externe Stabilitaetsgarantien (Flutter/FFI-Call-Sites) angelegt
  werden, nicht als bequemer interner Zweitname.
- `scripts/check_api_docs_sync.sh` darf um zusaetzliche Contracts erweitert werden, sobald ein Sunset-Termin
  erreicht ist (Anti-Pattern-Check: Legacy-Name darf nach Sunset nicht mehr in API.md auftauchen) — analog zu den
  bestehenden `check_no_match`-Regeln fuer bereits entfernte APIs.

## Verweise

- [`docs/ARCHITECTURE_PLAN.md`](ARCHITECTURE_PLAN.md) — Layer-Grenzen und Architektur-Guardrails
- [`crates/fs25_auto_drive_engine/API.md`](../crates/fs25_auto_drive_engine/API.md) — ColorPath-Legacy-Vertrag
- [`crates/fs25_auto_drive_host_bridge/API.md`](../crates/fs25_auto_drive_host_bridge/API.md) — `Engine*`-Kompat-Aliase
- [`crates/fs25_map_overview/API.md`](../crates/fs25_map_overview/API.md) — `fruits_gdm`-Wire-Format-Deprecation
