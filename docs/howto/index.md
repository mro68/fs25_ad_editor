# FS25 AutoDrive Editor – Bedienungsanleitung

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

## Inhaltsverzeichnis

| Seite | Themen |
|-------|--------|
| [Start & Dateiverwaltung](01-start.md) | Überblick, Datei öffnen/speichern |
| [Benutzeroberfläche](02-oberflaeche.md) | Fensteraufbau, Tastatur-Shortcuts, Maus-Bedienung |
| [Werkzeuge](03-werkzeuge.md) | Select, Connect, Add Node, Gerade, Kurve, Spline, Tangenten |
| [Bearbeitung](04-bearbeitung.md) | Selektion, Verbindungen, Map-Marker, Undo/Redo |
| [Karte & Hintergrund](05-karte.md) | Kamera, Hintergrundbild, Übersichtskarte, Auto-Detection, Heightmap |
| [Extras](06-extras.md) | Streckenteilung, Duplikat-Bereinigung, Optionen, Farbcodierung, Dateiformat |
| [Typische Workflows](07-workflows.md) | Schritt-für-Schritt-Anleitungen für häufige Aufgaben |

---

## Schnellreferenz: Tastatur-Shortcuts

| Shortcut | Aktion |
|----------|--------|
| `Ctrl+O` | Datei öffnen |
| `Ctrl+S` | Datei speichern |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Shift+Ctrl+Z` | Redo |
| `Ctrl+A` | Alle Nodes selektieren |
| `Escape` | Selektion aufheben / Tool zurücksetzen |
| `1` | Select-Tool |
| `2` | Connect-Tool |
| `3` | Add-Node-Tool |
| `Delete` / `Backspace` | Selektierte Nodes löschen |
| `C` | Verbindung erstellen (2 Nodes selektiert) |
| `X` | Verbindung trennen (2 Nodes selektiert) |
| `Enter` | Route-Tool: bestätigen und erstellen |

---

## Schnellreferenz: Farbcodierung

| Element | Farbe | Bedeutung |
|---------|-------|-----------|
| Node | Cyan | Normaler Wegpunkt (Regular) |
| Node | Gelb | Sub-Priorität (Nebenstraße) |
| Node | Magenta | Selektiert |
| Node | Rot | Warnung |
| Verbindung | Grün | Regular (Einbahn) |
| Verbindung | Blau | Dual (bidirektional) |
| Verbindung | Orange | Reverse (umgekehrt) |
| Marker | Rot/Dunkelrot | Map-Marker (Pin) |
