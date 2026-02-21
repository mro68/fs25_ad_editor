# FS25 AutoDrive Editor – Bedienungsanleitung

## Inhaltsverzeichnis

1. [Überblick](#überblick)
2. [Programmstart und Dateiverwaltung](#programmstart-und-dateiverwaltung)
3. [Benutzeroberfläche](#benutzeroberfläche)
4. [Tastatur-Shortcuts](#tastatur-shortcuts)
5. [Maus-Bedienung](#maus-bedienung)
6. [Werkzeuge (Tools)](#werkzeuge-tools)
7. [Selektion](#selektion)
8. [Verbindungen bearbeiten](#verbindungen-bearbeiten)
9. [Map-Marker](#map-marker)
10. [Kamera und Viewport](#kamera-und-viewport)
11. [Hintergrund-Karte](#hintergrund-karte)
12. [Heightmap](#heightmap)
13. [Optionen](#optionen)
14. [Undo / Redo](#undo--redo)
15. [Typische Workflows](#typische-workflows)

---

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

## Programmstart und Dateiverwaltung

### Datei öffnen

| Aktion | Weg |
|--------|-----|
| Menü | **File → Open...** |
| Shortcut | `Ctrl+O` |

Öffnet einen Datei-Dialog zur Auswahl einer AutoDrive-XML-Konfigurationsdatei. Nach dem Laden wird die Kamera automatisch auf die Bounding-Box des Netzwerks zentriert.

### Datei speichern

| Aktion | Weg |
|--------|-----|
| Speichern | **File → Save** oder `Ctrl+S` |
| Speichern unter | **File → Save As...** |

Beim Speichern wird geprüft, ob eine Heightmap geladen ist. Falls nicht, erscheint eine Warnung, dass Y-Koordinaten nicht korrekt geschrieben werden können.

### Programm beenden

| Aktion | Weg |
|--------|-----|
| Menü | **File → Exit** |

---

## Benutzeroberfläche

Das Hauptfenster besteht aus folgenden Bereichen:

```
┌─────────────────────────────────────────────┐
│ Menü-Leiste (File | Edit | View | Help)     │
├─────────────────────────────────────────────┤
│ Toolbar (Werkzeug | Delete | Background)    │
├─────────────────────────────────────────────┤
│                                             │
│                 Viewport                    │
│            (Karten-Darstellung)             │
│                                             │
├─────────────────────────────────────────────┤
│ Statusleiste (Nodes | Connections | Zoom …) │
└─────────────────────────────────────────────┘
```

### Menü-Leiste

- **File**: Öffnen, Speichern, Heightmap, Exit
- **Edit**: Undo, Redo, Optionen
- **View**: Kamera-Reset, Zoom, Hintergrund-Karte, Render-Quality
- **Help**: About (Versionsinformation)

### Toolbar

Zeigt die verfügbaren Werkzeuge:
- **⊹ Select (1)** — Standard-Werkzeug: Nodes selektieren und verschieben
- **⟷ Connect (2)** — Verbindungen zwischen Nodes erstellen
- **＋ Add Node (3)** — Neue Nodes auf der Karte platzieren- **Route-Tools (4)** — Route-Werkzeuge: Gerade Strecke, Bézier-Kurve, Spline- **🗑 Delete (Del)** — Selektierte Nodes löschen (nur aktiv bei Selektion)
- **Hintergrund-Controls** — Opacity-Slider und Sichtbarkeits-Toggle (rechts, nur wenn Hintergrund geladen)

### Statusleiste

Zeigt folgende Informationen (nur Anzeige, nicht interaktiv):
- Node-Count, Connection-Count, Marker-Count
- Map-Name (falls vorhanden)
- Zoom-Stufe und Kamera-Position
- Heightmap-Status (Dateiname oder "None")
- Anzahl selektierter Nodes
- FPS (rechts)

---

## Tastatur-Shortcuts

### Globale Shortcuts

| Shortcut | Aktion |
|----------|--------|
| `Ctrl+O` | Datei öffnen |
| `Ctrl+S` | Datei speichern |
| `Ctrl+Z` | Undo (Rückgängig) |
| `Ctrl+Y` | Redo (Wiederherstellen) |
| `Shift+Ctrl+Z` | Redo (Alternative) |
| `Ctrl+A` | Alle Nodes selektieren |
| `Escape` | Selektion aufheben |

### Werkzeug-Shortcuts

| Shortcut | Werkzeug |
|----------|----------|
| `1` | Select-Tool (Auswählen/Verschieben) |
| `2` | Connect-Tool (Verbindungen erstellen) |
| `3` | Add-Node-Tool (Nodes hinzufügen) |

### Bearbeitungs-Shortcuts

| Shortcut | Aktion | Bedingung |
|----------|--------|-----------|
| `Delete` / `Backspace` | Selektierte Nodes löschen | Mindestens 1 Node selektiert |
| `C` | Verbindung erstellen (Regular-Richtung) | Genau 2 Nodes selektiert |
| `X` | Verbindung zwischen Nodes trennen | Genau 2 Nodes selektiert |

---

## Maus-Bedienung

### Klick-Aktionen

| Maus-Aktion | Werkzeug | Ergebnis |
|-------------|----------|----------|
| **Linksklick** | Select | Node unter Mauszeiger selektieren (ersetzt bestehende Selektion) |
| **Ctrl+Linksklick** | Select | Node additiv zur Selektion hinzufügen |
| **Shift+Linksklick** | Select | Pfad-Selektion: Selektiert alle Nodes auf dem kürzesten Pfad zwischen Anker-Node und Ziel-Node |
| **Doppelklick** | Select | Segment-Selektion: Selektiert alle Nodes zwischen den nächsten Kreuzungen/Sackgassen |
| **Ctrl+Doppelklick** | Select | Segment additiv zur Selektion hinzufügen |
| **Linksklick** | Connect | Ers­ter Klick = Startknoten, Zweiter Klick = Zielknoten → Verbindung erstellen |
| **Linksklick** | Add Node | Neuen Node an Klickposition einfügen |

### Drag-Aktionen (Ziehen mit gedrückter Maustaste)

| Maus-Aktion | Ergebnis |
|-------------|----------|
| **Links-Drag auf selektiertem Node** | Alle selektierten Nodes gemeinsam verschieben |
| **Links-Drag auf leerem Bereich** | Kamera schwenken (Pan) |
| **Shift+Links-Drag** | Rechteck-Selektion → alle Nodes im Rechteck werden selektiert |
| **Shift+Ctrl+Links-Drag** | Rechteck-Selektion (additiv, erweitert bestehende Selektion) |
| **Alt+Links-Drag** | Lasso-Selektion → freigeformte Polygon-Selektion |
| **Alt+Ctrl+Links-Drag** | Lasso-Selektion (additiv, erweitert bestehende Selektion) |
| **Mittelklick-Drag** | Kamera schwenken (Pan) |
| **Rechtsklick-Drag** | Kamera schwenken (Pan) |

### Scroll-Aktionen

| Maus-Aktion | Ergebnis |
|-------------|----------|
| **Mausrad hoch** | Hineinzoomen (auf Mausposition) |
| **Mausrad runter** | Herauszoomen (von Mausposition) |

### Kontextmenü (Rechtsklick)

#### Bei 2+ selektierten Nodes (mit Verbindungen dazwischen)

| Menüpunkt | Aktion |
|-----------|--------|
| 🔗 Nodes verbinden | Verbindung erstellen (bei genau 2 Nodes ohne Verbindung) |
| ↦ Regular (Einbahn) | Alle Verbindungen auf Regular-Richtung setzen |
| ⇆ Dual (beidseitig) | Alle Verbindungen auf Dual-Richtung setzen |
| ↤ Reverse (rückwärts) | Alle Verbindungen auf Reverse-Richtung setzen |
| ⇄ Invertieren | Start/End aller Verbindungen tauschen |
| 🛣 Hauptstraße | Priorität aller Verbindungen auf Regular setzen |
| 🛤 Nebenstraße | Priorität aller Verbindungen auf SubPriority setzen |
| ✕ Alle trennen | Alle Verbindungen zwischen selektierten Nodes entfernen |

#### Bei 1 selektiertem Node

| Menüpunkt | Aktion |
|-----------|--------|
| 🗺 Marker erstellen | Neuen Map-Marker auf diesem Node anlegen |
| ✏ Marker ändern | Bestehenden Marker bearbeiten (Name, Gruppe) |
| ✕ Marker löschen | Marker von diesem Node entfernen |

---

## Werkzeuge (Tools)

### Select-Tool (1)

Das Standard-Werkzeug für Auswahl und Verschiebung von Nodes.

**Funktionen:**
- Einzelklick: Node selektieren (Pick-Radius: 12px)
- Ctrl+Klick: Additiv selektieren
- Shift+Klick: Pfad-Selektion (kürzester Pfad von Anker zu Ziel)
- Doppelklick: Segment zwischen Kreuzungen selektieren
- Drag auf selektiertem Node: Alle selektierten Nodes verschieben
- Drag auf leerem Bereich: Kamera schwenken

### Connect-Tool (2)

Erstellt Verbindungen zwischen zwei Nodes.

**Workflow:**
1. Ersten Node anklicken → in Toolbar erscheint "Startknoten: [ID] → Wähle Zielknoten"
2. Zweiten Node anklicken → Verbindung wird erstellt
3. Werkzeug bleibt aktiv für weitere Verbindungen

**Standard-Einstellungen:**
- Richtung: Regular (Einbahn vom Start zum Ziel)
- Priorität: Regular (Hauptstraße)

### Add-Node-Tool (3)

Platziert neue Wegpunkte auf der Karte.

**Workflow:**
- Klick auf eine beliebige Stelle → neuer Node wird an der Welt-Position eingefügt
- Der neue Node erhält automatisch die nächste freie ID

### Route-Tools (4)

Erstellt Strecken und Kurse über vordefinierte Geometrien. Im Route-Modus stehen drei Sub-Tools zur Verfügung:

#### 📏 Gerade Strecke

Zeichnet eine gerade Linie zwischen zwei Punkten mit automatischen Zwischen-Nodes.

**Workflow:**
1. Startpunkt klicken
2. Endpunkt klicken → Vorschau erscheint
3. Enter → Strecke wird erstellt

**Einstellungen:** Min. Abstand (Segment-Länge) und Anzahl Nodes.

#### 🔀 Kurve (Bézier)

Zeichnet eine Bézier-Kurve (Grad 2 oder 3) mit Steuerpunkten.

**Workflow:**
1. Startpunkt klicken
2. Endpunkt klicken
3. Steuerpunkt(e) klicken → Vorschau erscheint
4. Optional: Punkte per Drag anpassen
5. Enter → Kurve wird erstellt

**Einstellungen:** Grad (Quadratisch/Kubisch), Min. Abstand, Anzahl Nodes.

#### 〰️ Spline (Catmull-Rom)

Zeichnet einen interpolierenden Spline, der durch **alle geklickten Punkte** führt. Im Gegensatz zur Bézier-Kurve (die Steuerpunkte nur annähert) verläuft der Spline exakt durch jeden gesetzten Punkt.

**Workflow:**
1. Beliebig viele Punkte nacheinander klicken (mindestens 2)
2. Vorschau wird fortlaufend aktualisiert (Cursor = nächster Punkt)
3. Enter → Spline wird erstellt

**Einstellungen:** Min. Abstand (Segment-Länge) und Anzahl Nodes.

**Besonderheiten:**
- Ab 3 Punkten entsteht eine glatte Kurve (Catmull-Rom-Interpolation)
- Mit 2 Punkten wird eine gerade Strecke erzeugt
- Verkettung: Nach Enter wird der letzte Endpunkt automatisch als neuer Startpunkt übernommen
- Nachbearbeitung: Segment-Länge / Node-Anzahl können nach Erstellung geändert werden

#### Gemeinsame Eigenschaften aller Route-Tools

- **Enter** bestätigt und erstellt die Route
- **Escape** bricht ab und setzt das Tool zurück
- **Verkettung:** Nach Erstellung wird der letzte Endpunkt als neuer Startpunkt übernommen
- **Nachbearbeitung:** Segment-Länge/Node-Anzahl können nach Erstellung per Slider angepasst werden
- **Snap:** Start- und Endpunkte rasten auf existierende Nodes ein (Snap-Radius: 3m)

---

## Selektion

### Selektionsmodi

| Modus | Aktivierung | Beschreibung |
|-------|-------------|--------------|
| **Einzelselektion** | Linksklick | Ersetzt die aktuelle Selektion durch den angeklickten Node |
| **Additive Selektion** | Ctrl+Linksklick | Fügt den Node zur bestehenden Selektion hinzu |
| **Pfad-Selektion** | Shift+Linksklick | Selektiert alle Nodes auf dem kürzesten Pfad zwischen dem zuletzt selektierten Node (Anker) und dem angeklickten Node |
| **Segment-Selektion** | Doppelklick | Selektiert alle Nodes eines Segments (bis zur nächsten Kreuzung oder Sackgasse) |
| **Rechteck-Selektion** | Shift+Drag | Alle Nodes innerhalb des aufgezogenen Rechtecks |
| **Lasso-Selektion** | Alt+Drag | Alle Nodes innerhalb des freihand gezeichneten Polygons |
| **Alles selektieren** | Ctrl+A | Alle Nodes im Netzwerk selektieren |
| **Selektion aufheben** | Escape | Selektion komplett leeren |

### Additive Modi

Alle Selektionsmodi können mit **Ctrl** kombiniert werden, um die bestehende Selektion zu erweitern anstatt sie zu ersetzen:

- `Ctrl+Shift+Drag` → Rechteck-Selektion additiv
- `Ctrl+Alt+Drag` → Lasso-Selektion additiv
- `Ctrl+Doppelklick` → Segment additiv hinzufügen

### Selektion verschieben

Bei Drag auf einem bereits selektierten Node werden **alle selektierten Nodes gemeinsam verschoben**. Ein Undo-Snapshot wird automatisch beim Start des Drag erstellt.

---

## Verbindungen bearbeiten

### Verbindung erstellen

| Methode | Beschreibung |
|---------|--------------|
| **Connect-Tool (2)** | Zwei Nodes nacheinander anklicken |
| **Shortcut `C`** | Bei genau 2 selektierten Nodes → Regular-Verbindung erstellen |
| **Kontextmenü** | Rechtsklick bei genau 2 Nodes → "Nodes verbinden" |

### Verbindung entfernen

| Methode | Beschreibung |
|---------|--------------|
| **Shortcut `X`** | Bei genau 2 selektierten Nodes → Verbindung(en) trennen |
| **Kontextmenü** | Bei 2+ selektierten Nodes → "Alle trennen" |

### Richtung ändern

Über das **Kontextmenü** (Rechtsklick bei 2+ selektierten Nodes):

| Richtung | Symbol | Beschreibung |
|----------|--------|-------------|
| **Regular** | ↦ | Einbahnstraße (Start → Ende) |
| **Dual** | ⇆ | Bidirektional (beide Richtungen) |
| **Reverse** | ↤ | Umgekehrt (Ende → Start) |
| **Invertieren** | ⇄ | Start und Ende tauschen |

### Priorität ändern

Über das **Kontextmenü**:

| Priorität | Symbol | Beschreibung |
|-----------|--------|-------------|
| **Regular** | 🛣 | Hauptstraße |
| **SubPriority** | 🛤 | Nebenstraße (dünner dargestellt, Gelb-Markierung) |

### Farbcodierung

| Farbe | Bedeutung |
|-------|-----------|
| **Grün** | Regular-Verbindung (Einrichtung) |
| **Blau** | Dual-Verbindung (bidirektional) |
| **Orange** | Reverse-Verbindung |

---

## Map-Marker

Map-Marker sind benannte Ziele auf der Karte (z. B. „Hof", „Feld 1", „Silo").

### Marker erstellen

1. Einen einzelnen Node selektieren
2. Rechtsklick → **"🗺 Marker erstellen"**
3. Im Dialog Name und Gruppe eingeben
4. Bestätigen

### Marker bearbeiten

1. Den Node mit bestehendem Marker selektieren
2. Rechtsklick → **"✏ Marker ändern"**
3. Name/Gruppe anpassen
4. Bestätigen

### Marker löschen

1. Den Node mit Marker selektieren
2. Rechtsklick → **"✕ Marker löschen"**

### Darstellung

Marker werden als **rote Pin-Symbole** dargestellt:
- Pin-Spitze sitzt exakt auf dem Node-Zentrum
- Rote Füllung mit dunkelrotem Rand
- Größe: 2.0 Welteinheiten

---

## Kamera und Viewport

### Kamera-Steuerung

| Aktion | Weg |
|--------|-----|
| **Schwenken (Pan)** | Mittlere Maustaste / Rechte Maustaste ziehen, oder Links-Drag auf leerem Bereich |
| **Zoomen** | Mausrad (zoomt auf/von Mausposition) |
| **Zoom In** | View → Zoom In (Faktor 1.2) |
| **Zoom Out** | View → Zoom Out (Faktor 1/1.2) |
| **Kamera zurücksetzen** | View → Reset Camera |

### Automatische Zentrierung

Beim Laden einer Datei wird die Kamera automatisch auf die Bounding-Box des Netzwerks zentriert, sodass alle Nodes sichtbar sind.

### Render-Quality

Über **View → Render Quality**:

| Stufe | Beschreibung |
|-------|-------------|
| **Low** | Harte Kanten, maximale Performance |
| **Medium** | Standard Anti-Aliasing (empfohlen) |
| **High** | Breiteres Anti-Aliasing, weichere Kanten |

---

## Hintergrund-Karte

Eine Map-Übersicht (PNG, JPG oder DDS) kann als Hintergrund geladen werden, um Nodes und Verbindungen mit der Karte abzugleichen.

### Hintergrund laden

**View → Hintergrund laden...** — Datei-Dialog öffnet sich.

Optional kann beim Laden ein Center-Crop (quadratischer Ausschnitt) angegeben werden.

### Hintergrund-Controls (Toolbar)

Wenn ein Hintergrund geladen ist, erscheinen rechts in der Toolbar:

| Control | Beschreibung |
|---------|-------------|
| **Opacity-Slider** | Deckkraft einstellen (0.0 = unsichtbar bis 1.0 = voll sichtbar) |
| **👁 Sichtbar / 🚫 Ausgeblendet** | Hintergrund ein-/ausblenden |

---

## Heightmap

Die Heightmap wird für die korrekte Y-Koordinaten-Berechnung beim XML-Export benötigt.

### Heightmap laden

**File → Select Heightmap...** — Wählt eine PNG-Heightmap aus.

### Heightmap entfernen

**File → Clear Heightmap** — Entfernt die geladene Heightmap.

### Heightmap-Warnung

Beim Speichern ohne Heightmap erscheint eine Warnung. Sie können:
- **Bestätigen** → Speichern ohne Y-Koordinaten-Aktualisierung
- **Abbrechen** → Zurück zum Editor

---

## Optionen

Über **Edit → Optionen...** wird der Optionen-Dialog geöffnet. Alle Einstellungen werden als `fs25_auto_drive_editor.toml` neben der Binary gespeichert.

### Konfigurierbare Werte

| Kategorie | Option | Standard |
|-----------|--------|----------|
| **Nodes** | Node-Größe | 0.5 Welteinheiten |
| | Farbe Regular | Cyan |
| | Farbe SubPrio | Gelb |
| | Farbe Selektiert | Magenta |
| | Farbe Warnung | Rot |
| **Selektion** | Pick-Radius | 12 Pixel |
| | Größenfaktor Selektiert | 1.8× |
| **Connections** | Linienstärke Normal | 0.2 |
| | Linienstärke SubPrio | 0.1 |
| | Pfeil-Länge / Breite | 1.0 / 0.6 |
| | Farbe Regular | Grün |
| | Farbe Dual | Blau |
| | Farbe Reverse | Orange |
| **Marker** | Marker-Größe | 2.0 |
| | Füllfarbe | Rot |
| | Outline-Farbe | Dunkelrot |
| **Kamera** | Zoom-Schritt (Menü) | 1.2 |
| | Zoom-Schritt (Mausrad) | 1.1 |

---

## Undo / Redo

Alle destruktiven Operationen erzeugen automatisch einen Undo-Snapshot:

| Shortcut | Aktion |
|----------|--------|
| `Ctrl+Z` | Rückgängig (Undo) |
| `Ctrl+Y` oder `Shift+Ctrl+Z` | Wiederherstellen (Redo) |

Auch über **Edit → Undo / Redo** im Menü verfügbar (mit Anzeige ob verfügbar).

**Operationen mit Undo-Support:**
- Nodes hinzufügen / löschen
- Nodes verschieben
- Verbindungen erstellen / entfernen / ändern
- Marker erstellen / bearbeiten / löschen
- Bulk-Operationen (Richtung, Priorität, Invertierung, Trennen)

---

## Typische Workflows

### Neues Netzwerk bearbeiten

1. `Ctrl+O` → XML-Datei laden
2. Hintergrund laden (View → Hintergrund laden)
3. Heightmap laden (File → Select Heightmap)
4. Nodes bearbeiten (Select-Tool, Drag zum Verschieben)
5. `Ctrl+S` → Speichern

### Route erstellen (mit Route-Tools)

1. **Route-Tool (4)** aktivieren
2. Sub-Tool wählen: Gerade Strecke, Kurve oder Spline
3. Punkte auf der Karte klicken (Vorschau wird live angezeigt)
4. Segment-Länge / Node-Anzahl per Slider anpassen
5. **Enter** → Route wird erstellt
6. Für weitere Segmente: Verkettung nutzt automatisch den letzten Endpunkt

### Route manuell erstellen

1. **Add-Node-Tool (3)** aktivieren
2. Nacheinander Nodes auf der Karte platzieren
3. **Connect-Tool (2)** aktivieren
4. Jeweils Start- und Ziel-Node anklicken um Verbindungen zu erstellen
5. Alternativ: 2 Nodes selektieren → `C` für schnelle Verbindung

### Segment bearbeiten

1. **Doppelklick** auf ein Segment → selektiert alle Nodes bis zur nächsten Kreuzung
2. Rechtsklick → Richtung/Priorität ändern oder invertieren
3. Oder: `Delete` um das ganze Segment zu löschen

### Bulk-Bearbeitung

1. **Shift+Drag** (Rechteck) oder **Alt+Drag** (Lasso) um viele Nodes zu selektieren
2. Alternativ: `Ctrl+A` für alle Nodes
3. Rechtsklick → Bulk-Operationen auf allen Verbindungen zwischen selektierten Nodes

### Marker setzen

1. Ziel-Node selektieren (Linksklick)
2. Rechtsklick → "Marker erstellen"
3. Name eingeben (z. B. "Hof", "Feld 1")
4. Gruppe eingeben (z. B. "default")
5. Bestätigen

---

## Farbcodierung (Übersicht)

### Nodes

| Farbe | Bedeutung |
|-------|-----------|
| **Cyan** | Normaler Wegpunkt (Regular) |
| **Gelb** | Sub-Priorität (Nebenstraße) |
| **Magenta** | Selektiert |
| **Rot** | Warnung (Fehler/Problem) |

### Verbindungen

| Farbe | Bedeutung |
|-------|-----------|
| **Grün** | Regular (Einbahnstraße) |
| **Blau** | Dual (bidirektional) |
| **Orange** | Reverse (umgekehrt) |

### Marker

| Element | Farbe |
|---------|-------|
| **Füllung** | Rot |
| **Rand** | Dunkelrot |

---

## Dateiformat

Der Editor liest und schreibt AutoDrive-XML-Konfigurationsdateien (`AutoDrive_config*.xml`). Wichtige Details:

- **FS22/FS25-Kompatibilität:** Node-Flags 2 (AutoGenerated) und 4 (SplineGenerated) werden beim Import automatisch zu Regular (0) konvertiert
- **Listen-Trennzeichen:** Komma (`,`) für einfache Listen, Semikolon (`;`) für verschachtelte Listen (out/incoming)
- **Y-Koordinaten:** Werden beim Export aus der Heightmap berechnet (bikubische Interpolation)
