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
2. Unterstuetzte Quellen: PNG, JPEG, DDS sowie ZIP-Archive mit Bilddateien
3. Bei ZIP-Dateien mit genau einem Bild laedt der Editor den Treffer direkt.
4. Bei ZIP-Dateien mit mehreren Bildern oeffnet sich der Dialog **Bild aus ZIP waehlen**. Dort koennen Sie optional auf Overview-Dateien filtern.

### Automatische Erkennung

Wenn eine AutoDrive-XML geladen wird, prueft der Editor fuer Hintergrunddaten automatisch:

1. ein gespeichertes Layer-Bundle mit `overview_terrain.png` als Pflichtbasis im selben Verzeichnis wie die XML
2. `overview.png` im selben Verzeichnis wie die XML
3. `overview.jpg` als Rueckfall fuer aeltere Setups
4. passende Map-Mod-ZIPs im uebergeordneten `mods/`-Verzeichnis, falls eine neue Uebersichtskarte erzeugt werden soll

Wenn ein Layer-Bundle gefunden wird, setzt der Editor daraus sofort wieder das Hintergrundbild mit Ihren gespeicherten Standard-Layern zusammen. Die sichtbaren PNG-Dateien werden dabei bei Bedarf nacheinander von Platte geladen und direkt in das Ergebnisbild geblendet. Fehlende optionale Overlay-Dateien deaktivieren nur die jeweiligen Layer-Eintraege; ohne `overview_terrain.png` bleibt das Layer-System inaktiv und der Editor faellt auf `overview.png` oder `overview.jpg` zurueck.

Wenn etwas gefunden wird, erscheint der Dialog **Nach dem Laden erkannt**. Er bestaetigt automatisch geladene Assets und bietet bei passenden ZIPs direkt die Generierung einer neuen Uebersichtskarte an.
Falls kein passender ZIP erkannt wurde oder der Workflow manuell gestartet wird, kann derselbe Dialog auch eine ZIP-Datei ueber **ZIP-Datei auswaehlen** oeffnen.

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

Es gibt zwei Wege:

1. **Datei -> Uebersichtskarte generieren** und im Dialog **ZIP-Datei auswaehlen** eine FS25-Map-ZIP waehlen.
2. Nach dem Oeffnen einer XML im Dialog **Nach dem Laden erkannt** entweder einen gefundenen Mod-ZIP direkt ueber **Uebersichtskarte generieren** uebernehmen oder ueber **ZIP-Datei auswaehlen** manuell waehlen.
3. Im Dialog **Uebersichtskarte - Layer-Optionen** die sichtbaren Layer und die Quelle fuer Feldpolygone festlegen.

### Verarbeitungsschritte

1. **GRLE lesen** — Fieldinfo- und Density-Layer
2. **GDM lesen** — Heightmap-Daten
3. **Hillshading** — Berechnung fuer topographisches Relief
4. **Komposit** — Felder, Huegel, Strassen, Wasser werden zusammengefuehrt
5. **Export** als `fs25_overview_map_TIMESTAMP.png`

### Ausgabe

Die generierte Karte wird sofort als aktuelle Hintergrundkarte geladen.

Die Anzeige startet zwar als Preview der frisch generierten Kombi-Karte, das Layer-Menue ist aber sofort verfuegbar: Die einzelnen Layer-Dateien (`overview_terrain.png`, `overview_hillshade.png`, `overview_farmland_borders.png`, `overview_farmland_ids.png`, `overview_poi_markers.png`, `overview_legend.png`) werden direkt waehrend der Generierung im Zielverzeichnis gespeichert und als Layer-Katalog aktiviert.

Wenn bereits eine AutoDrive-XML geoeffnet ist, fragt der Editor anschliessend, ob das aktuelle Bild als `overview.png` im Savegame-/XML-Verzeichnis gespeichert werden soll. In diesem Save-Schritt werden nur noch `overview.png` sowie `overview.json` aktualisiert; die einzelnen Layer-Dateien wurden bereits bei der Generierung geschrieben und das Layer-Menue bleibt waehrenddessen nutzbar. Existieren dort bereits Dateien, koennen sie direkt ueberschrieben werden.

Beim naechsten Oeffnen derselben XML wird bevorzugt das gespeicherte Layer-Bundle wieder aktiviert. Nur ohne Terrain-Basis faellt der Editor auf die kombinierte `overview.png` beziehungsweise `overview.jpg` zurueck.

Die Layer-Auswahl aus dem Dialog wird fuer spaetere Generierungen als Standard gemerkt. Falls dieses Speichern fehlschlaegt, zeigt der Editor eine Warnung in der Statusleiste an. Die bereits erzeugte Uebersichtskarte bleibt dabei erhalten und kann weiterhin genutzt oder als `overview.png` gespeichert werden.

---

## Automatische Erkennung (Post-Load)

Nach dem Laden einer AutoDrive-XML prueft der Editor automatisch:

1. **Flag-Bereinigung** — Flags 2 und 4 (FS22-Artefakte) werden auf 0 zurueckgesetzt
2. **Verbindungsgeometrie** — Winkel und Laengen werden neu berechnet
3. **Spatial-Index** — KD-Baum wird aufgebaut (fuer Snap, Selection, Nearest-Node)
4. **Heightmap** — `terrain.heightmap.png` neben der XML wird automatisch gesetzt
5. **Hintergrundkarte** — bevorzugt wird ein gespeichertes Layer-Bundle neben der XML aktiviert; ohne `overview_terrain.png` werden `overview.png` oder `overview.jpg` automatisch geladen
6. **Map-Mod-ZIPs** — Passende Archive im `mods/`-Verzeichnis werden im Dialog zur Uebersichtskarten-Generierung angeboten

**Status-Ausgaben:**

- Der Dialog **Nach dem Laden erkannt** fasst automatisch geladene Heightmap, Hintergrundbild und passende ZIP-Treffer zusammen.
- Oben in der Statusleiste erscheinen kurze Meldungen wie „Hintergrundkarte geladen" oder „3 Flags bereinigt"
- Auch Persistenzwarnungen, zum Beispiel beim Merken der Uebersichts-Layer-Voreinstellungen, werden dort sichtbar eingeblendet statt still verworfen.

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
