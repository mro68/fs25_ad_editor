# Start & Dateiverwaltung

← [Zurueck zur Uebersicht](index.md)

## Ueberblick

Der FS25 AutoDrive Editor dient zum Erstellen und Bearbeiten von AutoDrive-Kursen fuer den Farming Simulator 25. Er laedt XML-Konfigurationsdateien (`AutoDrive_config*.xml`), stellt das Strassennetzwerk grafisch dar und ermoeglicht das Bearbeiten von Wegpunkten (Nodes), Verbindungen (Connections) und Map-Markern.

**Kernfeatures:**
- GPU-beschleunigtes Rendering fuer 100.000+ Wegpunkte
- Rect- und Lasso-Selektion
- Verbindungs-Bearbeitung (Richtung, Prioritaet, Invertierung)
- Map-Marker erstellen und verwalten
- Heightmap-Support fuer Y-Koordinaten beim Export
- Hintergrund-Karte (PNG/JPG/DDS) als Orientierungshilfe
- Vollstaendiges Undo/Redo-System

---

## Datei oeffnen

| Aktion | Weg |
|--------|-----|
| Menue | **File → Open...** |
| Shortcut | `Ctrl+O` |

Oeffnet einen Datei-Dialog zur Auswahl einer AutoDrive-XML-Konfigurationsdatei. Nach dem Laden wird die Kamera automatisch auf die Bounding-Box des Netzwerks zentriert.

**Automatische Erkennung:** Nach dem Laden prueft der Editor automatisch:
- Ob eine `terrain.heightmap.png` im selben Verzeichnis liegt → wird direkt als Heightmap gesetzt
- Ob im Mods-Verzeichnis (`../../mods/` relativ zum Savegame) ein passender Map-Mod-ZIP zum Kartennamen existiert → Dialog bietet Uebersichtskarten-Generierung an

Das Matching beruecksichtigt Umlaute (ae↔ae, oe↔oe, ue↔ue, ss↔ss), ist case-insensitive und behandelt Leerzeichen/Unterstriche als Wildcard.

Mehr zur Auto-Detection: [Karte & Hintergrund → Automatische Erkennung](05-karte.md#automatische-erkennung-post-load)

---

## Datei speichern

| Aktion | Weg |
|--------|-----|
| Speichern | **File → Save** oder `Ctrl+S` |
| Speichern unter | **File → Save As...** |

Beim Speichern wird geprueft, ob eine Heightmap geladen ist. Falls nicht, erscheint eine Warnung, dass Y-Koordinaten nicht korrekt geschrieben werden koennen.

---

## Programm beenden

| Aktion | Weg |
|--------|-----|
| Menue | **File → Exit** |

---

← [Zurueck zur Uebersicht](index.md) | → [Benutzeroberflaeche](02-oberflaeche.md)
