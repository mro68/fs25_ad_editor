# Bearbeitung: Selektion, Verbindungen, Marker, Undo

← [Werkzeuge](03-werkzeuge.md) | [Zurueck zur Uebersicht](index.md)

## Selektion

### Selektionsmodi

| Modus | Aktivierung | Beschreibung |
|-------|-------------|--------------|
| **Einzelselektion** | Linksklick | Ersetzt die aktuelle Selektion durch den angeklickten Node |
| **Additive Selektion** | Ctrl+Linksklick | Fuegt den Node zur bestehenden Selektion hinzu |
| **Pfad-Selektion** | Shift+Linksklick | Selektiert alle Nodes auf dem kuerzesten Pfad zwischen dem zuletzt selektierten Node (Anker) und dem angeklickten Node |
| **Gruppen-Selektion** | Doppelklick | Selektiert alle Nodes einer Gruppe (bis zur naechsten Kreuzung oder Sackgasse) |
| **Rechteck-Selektion** | Shift+Drag | Alle Nodes innerhalb des aufgezogenen Rechtecks |
| **Lasso-Selektion** | Alt+Drag | Alle Nodes innerhalb des freihand gezeichneten Polygons |
| **Alles selektieren** | Ctrl+A | Alle Nodes im Netzwerk selektieren |
| **Selektion aufheben** | Escape | Selektion komplett leeren |

### Additive Modi

Alle Selektionsmodi koennen mit **Ctrl** kombiniert werden, um die bestehende Selektion zu erweitern anstatt sie zu ersetzen:

- `Ctrl+Shift+Drag` → Rechteck-Selektion additiv
- `Ctrl+Alt+Drag` → Lasso-Selektion additiv
- `Ctrl+Doppelklick` → Gruppe additiv hinzufuegen

### Selektion verschieben

Bei Drag auf einem bereits selektierten Node werden **alle selektierten Nodes gemeinsam verschoben**. Ein Undo-Snapshot wird automatisch beim Start des Drag erstellt.

---

## Verbindungen bearbeiten

### Verbindung erstellen

| Methode | Beschreibung |
|---------|--------------|
| **Connect-Tool (T)** | Zwei Nodes nacheinander anklicken |
| **Shortcut `C`** | Bei genau 2 selektierten Nodes → Regular-Verbindung erstellen |
| **Kontextmenue** | Rechtsklick bei genau 2 Nodes → "Nodes verbinden" |

### Verbindung entfernen

| Methode | Beschreibung |
|---------|--------------|
| **Shortcut `X`** | Bei genau 2 selektierten Nodes → Verbindung(en) trennen |
| **Kontextmenue** | Bei 2+ selektierten Nodes → "Alle trennen" |

### Richtung aendern

Ueber das **Kontextmenue** (Rechtsklick bei 2+ selektierten Nodes):

| Richtung | Symbol | Beschreibung |
|----------|--------|-------------|
| **Regular** | ↦ | Einbahnstrasse (Start → Ende) |
| **Dual** | ⇆ | Bidirektional (beide Richtungen) |
| **Reverse** | ↤ | Umgekehrt (Ende → Start) |
| **Invertieren** | ⇄ | Start und Ende tauschen |

### Prioritaet aendern

Ueber das **Kontextmenue**:

| Prioritaet | Symbol | Beschreibung |
|-----------|--------|-------------|
| **Regular** | 🛣 | Hauptstrasse |
| **SubPriority** | 🛤 | Nebenstrasse (duenner dargestellt, Gelb-Markierung) |

### Farbcodierung

| Farbe | Bedeutung |
|-------|-----------|
| **Gruen** | Regular-Verbindung (Einrichtung) |
| **Blau** | Dual-Verbindung (bidirektional) |
| **Orange** | Reverse-Verbindung |

---

## Map-Marker

Map-Marker sind benannte Ziele auf der Karte (z. B. „Hof", „Feld 1", „Silo").

### Marker erstellen

1. Einen einzelnen Node selektieren
2. Rechtsklick → **"� Marker erstellen"**
3. Im Dialog Name und Gruppe eingeben
4. Bestaetigen

### Marker bearbeiten

1. Den Node mit bestehendem Marker selektieren
2. Rechtsklick → **"✏ Marker aendern"**
3. Name/Gruppe anpassen
4. Bestaetigen

### Marker loeschen

1. Den Node mit Marker selektieren
2. Rechtsklick → **"📍✕ Marker loeschen"**

### Darstellung

Marker werden als **Pin-Symbole** dargestellt:

- Pin-Spitze sitzt exakt auf dem Node-Zentrum
- Farbe und Umrissstärke sind in den **Optionen** konfigurierbar
- Groesse: 2.0 Welteinheiten

> **Tipp:** Umrissstärke und Markerfarbe koennen unter **Datei → Einstellungen → Map-Marker** angepasst werden.

---

## Undo / Redo

Alle destruktiven Operationen erzeugen automatisch einen Undo-Snapshot:

| Shortcut | Aktion |
|----------|--------|
| `Ctrl+Z` | Rueckgaengig (Undo) |
| `Ctrl+Y` oder `Shift+Ctrl+Z` | Wiederherstellen (Redo) |

Auch ueber **Edit → Undo / Redo** im Menue verfuegbar (mit Anzeige ob verfuegbar).

**Operationen mit Undo-Support:**

- Nodes hinzufuegen / loeschen
- Nodes verschieben
- Verbindungen erstellen / entfernen / aendern
- Marker erstellen / bearbeiten / loeschen
- Bulk-Operationen (Richtung, Prioritaet, Invertierung, Trennen)

---

← [Werkzeuge](03-werkzeuge.md) | [Zurueck zur Uebersicht](index.md) | → [Karte & Hintergrund](05-karte.md)
