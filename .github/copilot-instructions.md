# GitHub Copilot Instructions fuer fs25_auto_drive_editor

## Sprache

- **Code, Variablen, Funktionen:** Englisch
- **Kommentare, Docstrings, User-Messages:** Deutsch
- **Logs (Debug):** Englisch

## Projekt-Kontext

Rust-Editor fuer AutoDrive-Kurse (Farming Simulator 25). Ziel: 100k+ Wegpunkte fluessig rendern.

## Tech-Stack

- UI: egui
- Rendering: wgpu (GPU-Instancing, Culling)
- Spatial: kiddo (KD-Tree)
- XML: quick-xml (Structure of Arrays Format)
- Image: image crate (DDS-Support)

## Wichtige Design-Entscheidungen

1. **HashMap statt Array:** Node-IDs als Keys fuer Robustheit
2. **Flag-Bereinigung:** Flags 2/4 → 0 beim Laden (FS22/FS25)
3. **Delimiter:** `,` fuer Listen, `;` fuer verschachtelt (out/incoming)
4. **Performance:** GPU-Batching + Viewport-Culling fuer 100k Nodes
5. **App-Flow:** UI emittiert `AppIntent`, Controller mappt auf `AppCommand`, Use-Cases mutieren State

## Dokumentations-Pflicht

Bei **jeder** Codeaenderung muessen die betroffenen Dokumentationen synchron gehalten werden:

- **Docstrings (`///`):** Oeffentliche Funktionen/Structs/Enums immer dokumentieren; bei Signaturaenderung sofort anpassen.
- **`src/*/API.md`:** Aendert sich die oeffentliche API eines Moduls (Typen, Funktionen, Felder), muss das jeweilige `API.md` im selben Commit aktualisiert werden.
- **`docs/ROADMAP.md`:** Abgeschlossene Features als `[x]` markieren; neue geplante Features eintragen.
- **`docs/ARCHITECTURE_PLAN.md` / `.windsurf/rules/`:** Architektur-Entscheidungen (neue Layer, Pattern-Aenderungen) sofort nachziehen.
- Keine Codeaenderung ohne passende Doku-Aktualisierung — bei reinen Refactorings mindestens Docstrings pruefen.

## Siehe auch

- `.windsurf/rules/` fuer detaillierte Architektur-Docs
- `docs/DEVELOPMENT.md` fuer Analyse-Erkenntnisse
