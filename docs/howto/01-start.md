# Start & Dateiverwaltung

← [Zurück zur Übersicht](index.md)

## Überblick

Der FS25 AutoDrive Editor dient zum Erstellen und Bearbeiten von AutoDrive-Kursen für den Farming Simulator 25. Er lädt XML-Konfigurationsdateien (`AutoDrive_config*.xml`), stellt das Straßennetzwerk grafisch dar und ermöglicht das Bearbeiten von Wegpunkten (Nodes), Verbindungen (Connections) und Map-Markern.

**Kernfeatures:**
- GPU-beschleunigtes Rendering für 100.000+ Wegpunkte
- Rect- und Lasso-Selektion
- Verbindungs-Bearbeitung (Richtung, Priorität, Invertierung)
- Map-Marker erstellen und verwalten
- Heightmap-Support für Y-Koordinaten beim Export
- Hintergrund-Karte (PNG/JPG/DDS) als Orientierungshilfe
- Vollständiges Undo/Redo-System

---

## Datei öffnen

| Aktion | Weg |
|--------|-----|
| Menü | **File → Open...** |
| Shortcut | `Ctrl+O` |

Öffnet einen Datei-Dialog zur Auswahl einer AutoDrive-XML-Konfigurationsdatei. Nach dem Laden wird die Kamera automatisch auf die Bounding-Box des Netzwerks zentriert.

**Automatische Erkennung:** Nach dem Laden prüft der Editor automatisch:
- Ob eine `terrain.heightmap.png` im selben Verzeichnis liegt → wird direkt als Heightmap gesetzt
- Ob im Mods-Verzeichnis (`../../mods/` relativ zum Savegame) ein passender Map-Mod-ZIP zum Kartennamen existiert → Dialog bietet Übersichtskarten-Generierung an

Das Matching berücksichtigt Umlaute (ä↔ae, ö↔oe, ü↔ue, ß↔ss), ist case-insensitive und behandelt Leerzeichen/Unterstriche als Wildcard.

Mehr zur Auto-Detection: [Karte & Hintergrund → Automatische Erkennung](05-karte.md#automatische-erkennung-post-load)

---

## Datei speichern

| Aktion | Weg |
|--------|-----|
| Speichern | **File → Save** oder `Ctrl+S` |
| Speichern unter | **File → Save As...** |

Beim Speichern wird geprüft, ob eine Heightmap geladen ist. Falls nicht, erscheint eine Warnung, dass Y-Koordinaten nicht korrekt geschrieben werden können.

---

## Programm beenden

| Aktion | Weg |
|--------|-----|
| Menü | **File → Exit** |

---

← [Zurück zur Übersicht](index.md) | → [Benutzeroberfläche](02-oberflaeche.md)
