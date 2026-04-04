# API der Host-Bridge-Core-Crate

## Ueberblick

`fs25_auto_drive_host_bridge` ist die kanonische, toolkit-freie Bruecken-Crate ueber der Engine. Diese erste Commit-Stufe liefert bewusst nur das Scaffold (Crate-Struktur + Marker-Typ), damit die folgenden Commits Session-, DTO- und Adapter-Vertraege sauber und reviewbar in Schichten aufbauen koennen.

## Oeffentliche Typen

| Typ | Zweck |
|---|---|
| `HostBridgeScaffold` | Marker-Typ fuer das initiale Scaffold der neuen Bridge-Crate |

## Scope dieser Stufe

- Workspace- und Crate-Struktur der gemeinsamen Bridge existiert.
- Noch keine Session- oder Host-Adapter-API in dieser Crate.
- Der funktionale Umbau folgt in den nachgelagerten Commits.
