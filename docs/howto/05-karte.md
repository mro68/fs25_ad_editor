# Karte, Hintergrund & Heightmap

← [Bearbeitung](04-bearbeitung.md) | [Zurück zur Übersicht](index.md)

## Kamera und Viewport

### Navigation

| Aktion | Beschreibung |
|--------|--------------|
| **Scroll** | Zoom in/out |
| **Mittlere Maustaste + Drag** | Kamera verschieben (Pan) |
| **Rechte Maustaste + Drag** | Kamera verschieben (Pan, alternativ) |
| `F` | Auf die aktuelle Selektion zoomen |
| `Home` oder `Ctrl+0` | Ansicht auf alle Nodes zentrieren (Fit All) |

### Zoom

- Zoom via Scrollrad
- Zoom-Level wird in der Statusleiste angezeigt
- Viewport-Culling: Nodes und Verbindungen außerhalb des sichtbaren Bereichs werden nicht gerendert (Performance)

---

## Hintergrund-Karte

### Laden

1. **Datei → Hintergrundkarte öffnen** oder **Ctrl+Shift+O**
2. Unterstützte Formate: PNG, JPEG, DDS

### Automatische Erkennung

Wenn eine AutoDrive-XML geladen wird und `AutoDrive_config.xml` im selben Verzeichnis eine FS25-Map-Struktur erkennt, versucht der Editor automatisch:

1. eine `fs25_overview_map_*.png` zu finden (generierte Übersichtskarte)
2. eine `satellite_XXXXXX.dds` oder `terrain_XXXXXX.dds` zu laden

Der Status wird in der Statusleiste angezeigt.

### Positionierung

Die Hintergrundkarte wird automatisch mit den Weltkoordinaten der XML-Datei ausgerichtet. Bei FS25-Standard-Maps stimmt das Alignment ohne weiteres Eingreifen.

### Anzeige ein-/ausblenden

**Ansicht → Hintergrundkarte** oder Schaltfläche in der Toolbar.

### DDS-Unterstützung

DDS (DXT1, DXT5, BC7) werden nativ geladen und intern in RGBA8 konvertiert. Keine externen Tools notwendig.

---

## Übersichtskarten-Generierung

Der Editor enthält ein Hilfswerkzeug zum Generieren hochwertiger Übersichtskarten aus FS25-Maps.

### Voraussetzung

Eine installierte FS25-Map-Entpackstruktur mit:
- `map/` Verzeichnis (enthält GRLE/GDM-Dateien)
- Standardmäßiger FS25-Map-Struktur

### Starten

**Datei → Übersichtskarte generieren** (oder Ctrl+Shift+G)

### Verarbeitungsschritte

1. **GRLE lesen** — Fieldinfo- und Density-Layer
2. **GDM lesen** — Heightmap-Daten
3. **Hillshading** — Berechnung für topographisches Relief
4. **Komposit** — Felder, Hügel, Straßen, Wasser werden zusammengeführt
5. **Export** als `fs25_overview_map_TIMESTAMP.png`

### Ausgabe

Die generierte Karte landet im selben Verzeichnis wie die Map-Dateien und wird bei der nächsten XML-Ladeoperation automatisch als Hintergrund erkannt.

---

## Automatische Erkennung (Post-Load)

Nach dem Laden einer AutoDrive-XML prüft der Editor automatisch:

1. **Flag-Bereinigung** — Flags 2 und 4 (FS22-Artefakte) werden auf 0 zurückgesetzt
2. **Verbindungsgeometrie** — Winkel und Längen werden neu berechnet
3. **Spatial-Index** — KD-Baum wird aufgebaut (für Snap, Selection, Nearest-Node)
4. **Hintergrundkarte** — Wenn im Map-Verzeichnis eine passende Karte gefunden wird, wird sie automatisch geladen

**Status-Ausgaben:**
- Oben in der Statusleiste erscheinen kurze Meldungen wie „Hintergrundkarte geladen" oder „3 Flags bereinigt"

---

## Heightmap

Die Heightmap speichert Höhendaten pro Weltposition (FS25-Format).

### Laden

**Datei → Heightmap laden** — Wählt eine `terrain.png` oder kompatible Heightmap.

### Verwendung

- Beim Erstellen neuer Nodes wird die Höhe (Y-Koordinate) automatisch aus der Heightmap abgefragt
- Im Properties-Panel wird die interpolierte Höhe des selektierten Node angezeigt
- Die Höhenabfrage geschieht via Bilinear-Interpolation auf der Heightmap-Textur

### Kein Laden nötig

Wenn die Heightmap automatisch erkannt wird (gleicher Pfad wie die XML), wird sie beim Öffnen der Datei automatisch geladen.

---

← [Bearbeitung](04-bearbeitung.md) | [Zurück zur Übersicht](index.md) | → [Extras & Optionen](06-extras.md)
