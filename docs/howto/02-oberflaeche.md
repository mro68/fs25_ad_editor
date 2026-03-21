# Benutzeroberflaeche

← [Start & Dateiverwaltung](01-start.md) | [Zurueck zur Uebersicht](index.md)

## Fensteraufbau

Das Hauptfenster besteht aus folgenden Bereichen:

```mermaid
block-beta
    columns 2
    menu["Menue-Leiste (File | Edit | View | Help)"]:2
    toolbar["Toolbar (Werkzeug | Delete | Background)"]:2
    viewport["Viewport\n(Karten-Darstellung)"] props["Eigenschaften\n(Panel)"]
    status["Statusleiste (Nodes | Connections | Zoom …)"]:2
```

### Menue-Leiste

- **File**: Oeffnen, Speichern, Heightmap, Exit
- **Edit**: Undo, Redo, Optionen
- **View**: Kamera-Reset, Zoom, Hintergrund-Karte, Render-Quality
- **Help**: About (Versionsinformation)

### Eigenschaften-Panel (rechte Seitenleiste)

Das Eigenschaften-Panel zeigt kontextabhaengig Infos zur aktuellen Selektion und enthaelt die Standard-Verbindungseinstellungen sowie bei aktivem Route-Tool die Route-Konfiguration.

| Inhalt | Bedingung |
|--------|-----------|
| „Keine Selektion" | Kein Node selektiert |
| Node-ID, Position, Flag, Marker-Controls | Genau 1 Node selektiert |
| Verbindungs-Details, Richtungs-/Prioritaets-ComboBox, Trennen-Button | Genau 2 Nodes selektiert |
| „N Nodes selektiert" | 3+ Nodes selektiert |
| Standard-Richtung und Strassenart (ComboBox) | Immer sichtbar (unterer Bereich) |
| Route-Tool-Konfiguration (Slider) | Nur wenn Route-Tool aktiv |

### Toolbar

Zeigt die verfuegbaren Werkzeuge:

- **⊹ Select** — Standard-Werkzeug: Nodes selektieren und verschieben
- **⟷ Connect** — Verbindungen zwischen Nodes erstellen
- **＋ Add Node** — Neue Nodes auf der Karte platzieren
- **Route-Tools** — Route-Werkzeuge: Gerade Strecke, Bézier-Kurve, Spline
- **🗑 Delete (Del)** — Selektierte Nodes loeschen (nur aktiv bei Selektion)
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
| `Ctrl+O` | Datei oeffnen |
| `Ctrl+S` | Datei speichern |
| `Ctrl+Z` | Undo (Rueckgaengig) |
| `Ctrl+Y` | Redo (Wiederherstellen) |
| `Shift+Ctrl+Z` | Redo (Alternative) |
| `Ctrl+A` | Alle Nodes selektieren |
| `Escape` | Selektion aufheben |

### Floating-Menue-Shortcuts

Werkzeuge und Aktionen werden ueber **Floating-Menues** aufgerufen. Jeder Tastendruck oeffnet ein Popup-Menue mit den zugehoerigen Optionen:

| Shortcut | Floating-Menue | Inhalt |
|----------|----------------|--------|
| `T` | Werkzeuge | Select, Connect, Add Node, Gerade, Kurve, Spline |
| `B` | Bearbeitungstools | Duplikate, Strecke aufteilen, sonstige Bearbeitungs-Aktionen |
| `G` | Grundbefehle | Datei oeffnen, speichern, Undo, Redo |
| `R` | Richtung & Strassenart | Einbahn vorwaerts, Zweirichtung, Einbahn rueckwaerts, Hauptstrasse, Nebenstrasse |
| `Z` | Zoom | Auf komplette Map zoomen, Auf Auswahl zoomen |

### Bearbeitungs-Shortcuts

| Shortcut | Aktion | Bedingung |
|----------|--------|-----------|
| `Delete` / `Backspace` | Selektierte Nodes loeschen | Mindestens 1 Node selektiert |
| `C` | Verbindung erstellen (Regular-Richtung) | Genau 2 Nodes selektiert |
| `X` | Verbindung zwischen Nodes trennen | Genau 2 Nodes selektiert |

---

## Maus-Bedienung

### Klick-Aktionen

| Maus-Aktion | Werkzeug | Ergebnis |
|-------------|----------|----------|
| **Linksklick** | Select | Node unter Mauszeiger selektieren (ersetzt bestehende Selektion) |
| **Ctrl+Linksklick** | Select | Node additiv zur Selektion hinzufuegen |
| **Shift+Linksklick** | Select | Pfad-Selektion: Selektiert alle Nodes auf dem kuerzesten Pfad zwischen Anker-Node und Ziel-Node |
| **Doppelklick** | Select | Gruppen-Selektion: Selektiert alle Nodes zwischen den naechsten Kreuzungen/Sackgassen |
| **Ctrl+Doppelklick** | Select | Gruppe additiv zur Selektion hinzufuegen |
| **Linksklick** | Connect | Erster Klick = Startknoten, Zweiter Klick = Zielknoten → Verbindung erstellen |
| **Linksklick** | Add Node | Neuen Node an Klickposition einfuegen |

### Drag-Aktionen (Ziehen mit gedrueckter Maustaste)

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
| **Mausrad auf numerichem Feld (Hover)** | Wert erhoehen (Distanzen: +0,1 m; Ganzzahlen: +1) |
| **Mausrad runter auf numerichem Feld (Hover)** | Wert verringern (Distanzen: −0,1 m; Ganzzahlen: −1) |

> **Tipp:** Alle numerischen Eingabefelder (Slider, Zahleneingaben) reagieren auf das Mausrad, sobald der Cursor darueber steht – kein Klick noetig.

### Kontextmenue (Rechtsklick)

#### Bei 2+ selektierten Nodes (mit Verbindungen dazwischen)

| Menuepunkt | Aktion |
|-----------|--------|
| 🔗 Nodes verbinden | Verbindung erstellen (bei genau 2 Nodes ohne Verbindung) |
| ↦ Regular (Einbahn) | Alle Verbindungen auf Regular-Richtung setzen |
| ⇆ Dual (beidseitig) | Alle Verbindungen auf Dual-Richtung setzen |
| ↤ Reverse (rueckwaerts) | Alle Verbindungen auf Reverse-Richtung setzen |
| ⇄ Invertieren | Start/End aller Verbindungen tauschen |
| 🛣 Hauptstrasse | Prioritaet aller Verbindungen auf Regular setzen |
| 🛤 Nebenstrasse | Prioritaet aller Verbindungen auf SubPriority setzen |
| ✕ Alle trennen | Alle Verbindungen zwischen selektierten Nodes entfernen |

#### Bei 1 selektiertem Node

| Menuepunkt | Aktion |
|-----------|--------|
| � Marker erstellen | Neuen Map-Marker auf diesem Node anlegen |
| ✏ Marker aendern | Bestehenden Marker bearbeiten (Name, Gruppe) |
| 📍✕ Marker loeschen | Marker von diesem Node entfernen |

---

← [Start & Dateiverwaltung](01-start.md) | [Zurueck zur Uebersicht](index.md) | → [Werkzeuge](03-werkzeuge.md)
