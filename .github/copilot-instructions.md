# GitHub Copilot Instructions für fs25_auto_drive_editor

## Sprache
- **Code, Variablen, Funktionen:** Englisch
- **Kommentare, Docstrings, User-Messages:** Deutsch
- **Logs (Debug):** Englisch

## Projekt-Kontext
Rust-Editor für AutoDrive-Kurse (Farming Simulator 25). Ziel: 100k+ Wegpunkte flüssig rendern.

## Tech-Stack
- UI: egui
- Rendering: wgpu (GPU-Instancing, Culling)
- Spatial: kiddo (KD-Tree)
- XML: quick-xml (Structure of Arrays Format)
- Image: image crate (DDS-Support)

## Wichtige Design-Entscheidungen
1. **HashMap statt Array:** Node-IDs als Keys für Robustheit
2. **Flag-Bereinigung:** Flags 2/4 → 0 beim Laden (FS22/FS25)
3. **Delimiter:** `,` für Listen, `;` für verschachtelt (out/incoming)
4. **Performance:** GPU-Batching + Viewport-Culling für 100k Nodes
5. **App-Flow:** UI emittiert `AppIntent`, Controller mappt auf `AppCommand`, Use-Cases mutieren State

## Siehe auch
- `.windsurf/rules/` für detaillierte Architektur-Docs
- `docs/DEVELOPMENT.md` für Analyse-Erkenntnisse
