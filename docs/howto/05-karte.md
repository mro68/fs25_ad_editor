# Karte, Hintergrund & Heightmap

← [Bearbeitung](04-bearbeitung.md) | [Zurueck zur Uebersicht](index.md)

## Kamera und Viewport

### Navigation

| Aktion | Beschreibung |
|--------|--------------|
| **Scroll** | Zoom in/out |
| **Mittlere Maustaste + Drag** | Kamera verschieben (Pan) |
| **Rechte Maustaste + Drag** | Kamera verschieben (Pan, alternativ) |
| `Pfeiltasten` | Kamera verschieben, solange kein aktives Route-Tool gezeichnet wird |
| `+` / `-` | Stufenweise hinein- oder herauszoomen |
| `Z` | Zoom-Floating-Menue oeffnen (Hinein, Heraus, Auf komplette Map, Auf Auswahl) |

Die Ansicht laesst sich ausserdem ueber **Ansicht -> Kamera zuruecksetzen** oder ueber die Command Palette zuruecksetzen.

### Zoom

- Zoom via Scrollrad
- Zoom-Level wird in der Statusleiste angezeigt
- **Zoom Floating-Menue (`Z`):** Bietet zwei Schnelloptionen:
  - **Auf komplette Map** — zeigt alle Nodes im Viewport
  - **Auf Auswahl** — zoomt auf die aktuell selektierten Nodes
- Viewport-Culling: Nodes und Verbindungen ausserhalb des sichtbaren Bereichs werden nicht gerendert (Performance)

---

## Hintergrund-Karte

### Laden

1. **Ansicht -> Hintergrund laden...** oder **Ansicht -> Hintergrund aendern...**
2. Unterstuetzte Formate: PNG, JPEG, DDS

### Automatische Erkennung

Wenn eine AutoDrive-XML geladen wird und `AutoDrive_config.xml` im selben Verzeichnis eine FS25-Map-Struktur erkennt, versucht der Editor automatisch:

1. eine `fs25_overview_map_*.png` zu finden (generierte Uebersichtskarte)
2. eine `satellite_XXXXXX.dds` oder `terrain_XXXXXX.dds` zu laden

Der Status wird in der Statusleiste angezeigt.

### Positionierung

Die Hintergrundkarte wird automatisch mit den Weltkoordinaten der XML-Datei ausgerichtet. Bei FS25-Standard-Maps stimmt das Alignment ohne weiteres Eingreifen.

### Anzeige ein-/ausblenden

Ueber die linke Seitenleiste im Abschnitt **Hintergrund** oder ueber **Ansicht -> Hintergrund laden... / Hintergrund aendern...**.

### DDS-Unterstuetzung

DDS (DXT1, DXT5, BC7) werden nativ geladen und intern in RGBA8 konvertiert. Keine externen Tools notwendig.

---

## Uebersichtskarten-Generierung

Der Editor enthaelt ein Hilfswerkzeug zum Generieren hochwertiger Uebersichtskarten aus FS25-Maps.

### Voraussetzung

Eine installierte FS25-Map-Entpackstruktur mit:

- `map/` Verzeichnis (enthaelt GRLE/GDM-Dateien)
- Standardmaessiger FS25-Map-Struktur

### Starten

**Datei -> Uebersichtskarte generieren**

### Verarbeitungsschritte

1. **GRLE lesen** — Fieldinfo- und Density-Layer
2. **GDM lesen** — Heightmap-Daten
3. **Hillshading** — Berechnung fuer topographisches Relief
4. **Komposit** — Felder, Huegel, Strassen, Wasser werden zusammengefuehrt
5. **Export** als `fs25_overview_map_TIMESTAMP.png`

### Ausgabe

Die generierte Karte landet im selben Verzeichnis wie die Map-Dateien und wird bei der naechsten XML-Ladeoperation automatisch als Hintergrund erkannt.

---

## Automatische Erkennung (Post-Load)

Nach dem Laden einer AutoDrive-XML prueft der Editor automatisch:

1. **Flag-Bereinigung** — Flags 2 und 4 (FS22-Artefakte) werden auf 0 zurueckgesetzt
2. **Verbindungsgeometrie** — Winkel und Laengen werden neu berechnet
3. **Spatial-Index** — KD-Baum wird aufgebaut (fuer Snap, Selection, Nearest-Node)
4. **Hintergrundkarte** — Wenn im Map-Verzeichnis eine passende Karte gefunden wird, wird sie automatisch geladen

**Status-Ausgaben:**

- Oben in der Statusleiste erscheinen kurze Meldungen wie „Hintergrundkarte geladen" oder „3 Flags bereinigt"

---

## Heightmap

Die Heightmap speichert Hoehendaten pro Weltposition (FS25-Format).

### Laden

**Datei → Heightmap laden** — Waehlt eine `terrain.png` oder kompatible Heightmap.

### Verwendung

- Beim Erstellen neuer Nodes wird die Hoehe (Y-Koordinate) automatisch aus der Heightmap abgefragt
- Im Properties-Panel wird die interpolierte Hoehe des selektierten Node angezeigt
- Die Hoehenabfrage geschieht via Bilinear-Interpolation auf der Heightmap-Textur

### Kein Laden noetig

Wenn die Heightmap automatisch erkannt wird (gleicher Pfad wie die XML), wird sie beim Oeffnen der Datei automatisch geladen.

---

← [Bearbeitung](04-bearbeitung.md) | [Zurueck zur Uebersicht](index.md) | → [Extras & Optionen](06-extras.md)
