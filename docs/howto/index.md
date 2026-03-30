# FS25 AutoDrive Editor – Bedienungsanleitung

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

## Inhaltsverzeichnis

| Seite | Themen |
|-------|--------|
| [Start & Dateiverwaltung](01-start.md) | Ueberblick, Datei oeffnen/speichern |
| [Benutzeroberflaeche](02-oberflaeche.md) | Fensteraufbau, Tastatur-Shortcuts, Maus-Bedienung |
| [Werkzeuge](03-werkzeuge.md) | Select, Connect, Add Node, Gerade, Kurve, Spline, Tangenten, Ausweichstrecke, Parkplatz |
| [Bearbeitung](04-bearbeitung.md) | Selektion, Verbindungen, Map-Marker, Undo/Redo |
| [Karte & Hintergrund](05-karte.md) | Kamera, Hintergrundbild, Uebersichtskarte, Auto-Detection, Heightmap |
| [Extras](06-extras.md) | Streckenteilung, Duplikat-Bereinigung, Optionen, Farbcodierung, Dateiformat |
| [Typische Workflows](07-workflows.md) | Schritt-fuer-Schritt-Anleitungen fuer haeufige Aufgaben |

---

## Schnellreferenz: Tastatur-Shortcuts

| Shortcut | Aktion |
|----------|--------|
| `Ctrl+O` | Datei oeffnen |
| `Ctrl+S` | Datei speichern |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Shift+Ctrl+Z` | Redo |
| `Ctrl+A` | Alle Nodes selektieren |
| `Escape` | Selektion aufheben / Tool zuruecksetzen |
| `T` | Werkzeuge Floating-Menue (Select, Connect, Add Node, Route-Tools) |
| `B` | Bearbeitungstools Floating-Menue |
| `G` | Grundbefehle Floating-Menue |
| `R` | Richtung & Strassenart Floating-Menue |
| `Z` | Zoom Floating-Menue |
| `Delete` / `Backspace` | Selektierte Nodes loeschen |
| `C` | Verbindung erstellen (2 Nodes selektiert) |
| `X` | Verbindung trennen (2 Nodes selektiert) |
| `Enter` | Route-Tool: bestaetigen und erstellen |

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
