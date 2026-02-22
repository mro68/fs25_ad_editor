# Code Style Guide

## Sprache
- **Code, Variablen, Typen, Funktionen:** Englisch
- **Kommentare, Docstrings, README:** Deutsch
- **User-facing Messages:** Deutsch
- **Debug-Logs:** Englisch

## Rust Conventions
- Standard Rust formatting (`cargo fmt`)
- Clippy lints aktiviert (`cargo clippy`)
- Dokumentationskommentare für public API

## Beispiele

```rust
/// Lädt eine AutoDrive-Konfiguration aus einer XML-Datei.
/// 
/// # Argumente
/// * `path` - Pfad zur XML-Datei
/// 
/// # Fehler
/// Gibt einen Fehler zurück, wenn die Datei nicht gelesen werden kann
/// oder das XML-Format ungültig ist.
pub fn load_config(path: &Path) -> Result<RoadMap, LoadError> {
    // Implementierung hier
}

// Temporäre Variable für Node-ID-Mapping
let mut node_id_map = HashMap::new();

// Verbindungen zwischen Nodes aufbauen
for (source_id, target_ids) in connections {
    // ...
}
```

## Struktur
- `src/app/` - AppController, Intents/Commands, Use-Cases, AppState
- `src/core/` - Datenmodelle und Business-Logik
- `src/xml/` - XML-Parsing und Serialization
- `src/render/` - wgpu Rendering-Pipeline
- `src/ui/` - egui Interface-Code

## Tests
- Unit-Tests direkt in Modulen (`#[cfg(test)]`)
- Integration-Tests in `tests/`
- Test-Fixtures in `tests/fixtures/`

## Dokumentations-Pflicht
Bei jeder Codeänderung müssen folgende Dokumente synchron gehalten werden:

| Änderungstyp | Was aktualisieren |
|---|---|
| Neue/geänderte öffentliche Funktion / Struct / Enum | Docstring (`///`), zugehöriges `src/*/API.md` |
| Neues Feature abgeschlossen | `docs/ROADMAP.md` → `[x]` setzen |
| Neue Architektur-Entscheidung / neues Modul | `.windsurf/rules/projekt.md` oder `docs/ARCHITECTURE_PLAN.md` |
| Breaking Change in Core/Render/App/XML | Alle betroffenen `API.md`-Dateien |
| Refactoring ohne API-Änderung | Docstrings prüfen, ggf. anpassen |

**Regel:** Kein Commit mit Codeänderungen ohne passende Doku-Aktualisierung im selben Commit.
