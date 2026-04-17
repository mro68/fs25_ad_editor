# Contributing

Dieses Dokument beschreibt den bevorzugten Beitragspfad fuer dieses Repository.

## Beitragspfad

1. Issue oder Finding referenzieren (z. B. in Commit-Body oder PR-Beschreibung).
2. Kleine, klar abgegrenzte Commits erstellen.
3. Architekturgrenzen einhalten (`UI -> App -> Core -> Shared` und `Render -> Core -> Shared`).
4. Oeffentliche API-Aenderungen immer mit passender API-Dokumentation synchronisieren.
5. Fuer Doku- und Governance-Aenderungen die zentralen Tracking-Dokumente aktualisieren.

## Lokale Checks vor Commit

Die folgenden Checks sind der Standardpfad fuer lokale Verifikation:

```bash
nocorrect cargo fmt --all
nocorrect cargo check
nocorrect cargo test --lib
./scripts/check_api_docs_sync.sh
./scripts/check_layer_boundaries.sh
./scripts/check_todo_gate.sh
```

Hinweise:
- Bei grossen oder riskanten Aenderungen zusaetzlich `nocorrect cargo test` ausfuehren.
- Security-Checks (`cargo audit`, `cargo deny`) sind in CI integriert und koennen lokal bei Bedarf vorgezogen werden.

## Dokumentations-Sync

Bei jeder relevanten Codeaenderung:
- Docstrings (`///`) fuer neue oder geaenderte `pub`-Items pruefen/aktualisieren.
- Betroffene `API.md` im jeweiligen Modul synchronisieren.
- `docs/ROADMAP.md` fuer Feature-Status aktualisieren.
- Operative offene Punkte im zentralen TODO-Tracking pflegen: `docs/TODO_TRACKING.md`.

## Commit- und Scope-Regeln

- Unabhaengige Themen nicht in einem Commit mischen.
- Bereits vorhandene, scope-fremde lokale Aenderungen nicht mitschleppen.
- Selektiv stagen und Commit-Scope explizit halten.

## Verweise

- Architektur: `docs/ARCHITECTURE_PLAN.md`
- Entwicklung: `docs/DEVELOPMENT.md`
- Roadmap: `docs/ROADMAP.md`
- Zentrales TODO-Tracking: `docs/TODO_TRACKING.md`
