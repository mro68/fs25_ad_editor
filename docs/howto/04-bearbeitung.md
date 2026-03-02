# Bearbeitung: Selektion, Verbindungen, Marker, Undo

← [Werkzeuge](03-werkzeuge.md) | [Zurück zur Übersicht](index.md)

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

← [Werkzeuge](03-werkzeuge.md) | [Zurück zur Übersicht](index.md) | → [Karte & Hintergrund](05-karte.md)
