# FS25 AutoDrive Editor – Bedienungsanleitung

Der FS25 AutoDrive Editor dient zum Erstellen und Bearbeiten von AutoDrive-Kursen fuer den Farming Simulator 25. Er laedt XML-Konfigurationsdateien (`AutoDrive_config*.xml`), stellt das Strassennetzwerk grafisch dar und ermoeglicht das Bearbeiten von Wegpunkten (Nodes), Verbindungen (Connections) und Map-Markern.

**Kernfeatures:**

- GPU-beschleunigtes Rendering fuer 100.000+ Wegpunkte
- Rect- und Lasso-Selektion
- Einheitlicher Route-Tool-Katalog ueber Sidebar, Menue, Floating-Menues und Command Palette
- Verbindungs-Bearbeitung (Richtung, Prioritaet, Invertierung)
- Map-Marker erstellen und verwalten
- Heightmap-Support fuer Y-Koordinaten beim Export
- Hintergrund-Karte (PNG/JPG/DDS) als Orientierungshilfe
- Vollstaendiges Undo/Redo-System

---

## Inhaltsverzeichnis

| Seite | Themen |
|-------|--------|
| [Start & Dateiverwaltung](01-start.md) | Ueberblick, Datei oeffnen/speichern |
| [Benutzeroberflaeche](02-oberflaeche.md) | Fensteraufbau, Tastatur-Shortcuts, Maus-Bedienung |
| [Werkzeuge](03-werkzeuge.md) | Vollstaendiger Tool-Katalog: Select, Connect, Add Node, Grundbefehle, Bearbeiten, Analyse, Tool-Edit-Vertrag |
| [Bearbeitung](04-bearbeitung.md) | Selektion, Verbindungen, Map-Marker, Undo/Redo |
| [Karte & Hintergrund](05-karte.md) | Kamera, Hintergrundbild, Uebersichtskarte, Auto-Detection, Heightmap |
| [Extras](06-extras.md) | Streckenteilung, Duplikat-Bereinigung, Optionen, Farbcodierung, Dateiformat |
| [Typische Workflows](07-workflows.md) | Schritt-fuer-Schritt-Anleitungen fuer Tool-Findbarkeit, Analyse-Tools und spaetere Nachbearbeitung |

---

## Schnellreferenz: Tastatur-Shortcuts

### Global

| Shortcut | Aktion |
|----------|--------|
| `Ctrl+O` | Datei oeffnen |
| `Ctrl+S` | Datei speichern |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Shift+Ctrl+Z` | Redo |
| `Ctrl+A` | Alle Nodes selektieren |
| `Ctrl+C` | Selektion kopieren |
| `Ctrl+V` | Einfuegen-Vorschau starten |
| `K` / `Ctrl+K` | Command Palette umschalten |
| `Escape` | Route-Tool abbrechen, Selektion aufheben oder zum Select-Tool zurueckkehren |

### Kataloge und Menues

| Shortcut | Aktion |
|----------|--------|
| `T` | Werkzeug-Menue |
| `G` | Grundbefehle-Menue |
| `B` | Bearbeiten-Menue |
| `A` | Analyse-Menue |
| `R` | Richtung und Strassenart |
| `Z` | Zoom-Menue |

### Bearbeitung und Navigation

| Shortcut | Aktion |
|----------|--------|
| `Delete` / `Backspace` | Selektierte Nodes loeschen |
| `C` | Verbindung erstellen bei genau 2 selektierten Nodes |
| `X` | Verbindung trennen bei genau 2 selektierten Nodes |
| `Enter` | Aktives Route-Tool ausfuehren |
| `+` / `-` | Stufenweise hinein- oder herauszoomen |
| `Pfeiltasten` | Kamera schwenken oder im aktiven Route-Tool Node-Anzahl / Segmentlaenge anpassen |

---

## Schnellreferenz: Farbcodierung

| Element | Farbe | Bedeutung |
|---------|-------|-----------|
| Node | Cyan | Normaler Wegpunkt (Regular) |
| Node | Gelb | Sub-Prioritaet (Nebenstrasse) |
| Node | Magenta | Selektiert |
| Node | Rot | Warnung |
| Verbindung | Gruen | Regular (Einbahn) |
| Verbindung | Blau | Dual (bidirektional) |
| Verbindung | Orange | Reverse (umgekehrt) |
| Marker | Rot/Dunkelrot | Map-Marker (Pin) |
| Gruppen-Icon → | Weiss | Grenzknoten: Eingang (externe Verbindung rein) |
| Gruppen-Icon ← | Weiss | Grenzknoten: Ausgang (Verbindung raus) |
| Gruppen-Icon ↔ | Weiss | Grenzknoten: Bidirektional |
