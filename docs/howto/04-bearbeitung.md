# Bearbeitung: Selektion, Verbindungen, Gruppen, Marker, Undo

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

## Gruppen

Gruppen sind benannte Streckenabschnitte, die als Einheit verwaltet werden koennen. Jede Gruppe enthaelt eine Menge von Nodes und ermoeglicht eine gemeinsame Selektion und Nachbearbeitung.

### Was ist eine Gruppe?

Eine Gruppe entsteht, wenn ein gruppenfaehiges Route-Tool ein Ergebnis erzeugt oder wenn eine bestehende Selektion manuell zu einer Gruppe zusammengefasst wird. Gruppen koennen gesperrt werden, um versehentliche Aenderungen zu verhindern.

> **Wichtig:** Ergebnisse von **Feldweg erkennen** und **Farb-Pfad erkennen** sind bewusst **nicht** ueber einen spaeteren Tool-Edit-Pfad nachbearbeitbar. Sie bleiben normal selektierbar und manuell editierbar, erzeugen aber keinen **Tool bearbeiten**-Button.

### Gruppen selektieren

| Methode | Beschreibung |
|---------|--------------|
| **Doppelklick** auf Gruppen-Node | Selektiert alle Nodes der Gruppe |
| **Ctrl+Doppelklick** | Gruppe additiv zur bestehenden Selektion hinzufügen |

### Gruppen-Bearbeitungsmodus

Im Gruppen-Bearbeitungsmodus lassen sich Nodes einer Gruppe verschieben, hinzufügen oder löschen.

**Aktivieren:** Gruppe selektieren, dann per Rechtsklick **"Gruppe bearbeiten"** waehlen.

Das schwebende Fenster **"✏ Gruppen-Bearbeitung"** erscheint und zeigt die aktive Gruppen-ID.

| Aktion | Beschreibung |
|--------|--------------|
| Nodes verschieben | Drag auf selektierten Node (wie im Normalbetrieb) |
| Nodes hinzufügen | Add-Node-Tool (T → Add Node) |
| Nodes löschen | `Delete` |
| **🔧 Tool bearbeiten** | Oeffnet das urspruengliche Tool erneut, aber nur solange fuer die Gruppe noch ein gueltiger Tool-Snapshot gespeichert ist. Der Button fehlt bewusst bei manuell erzeugten Gruppen, ephemeren Analyse-Tools sowie Gruppen mit bereits invalidiertem Snapshot. |
| **✓ Übernehmen** oder `Enter` | Änderungen auf die Gruppe anwenden |
| **✕ Abbrechen** oder `Escape` | Alle Änderungen rückgängig machen |

> **Hinweis:** Wenn du im Gruppen-Bearbeitungsmodus die Struktur einer Gruppe manuell veraenderst und uebernimmst, wird der gespeicherte Tool-Snapshot absichtlich verworfen. Danach bleibt die Gruppe normal bearbeitbar, aber nicht mehr ueber **Tool bearbeiten** rehydrierbar.

### Grenzknoten-Icons (Eingang / Ausgang / Bidirektional)

Wenn eine Gruppe selektiert ist, markiert der Editor automatisch alle **Grenzknoten** — Nodes, die Verbindungen zu Nodes außerhalb der Gruppe haben — mit einem Icon:

| Icon | Bedeutung | Beschreibung |
|------|-----------|--------------|
| **Eingang** (→) | Externer Zufluss | Verbindung von außen führt in die Gruppe hinein |
| **Ausgang** (←) | Externer Abfluss | Verbindung aus der Gruppe führt nach außen |
| **Bidirektional** (↔) | Ein- und Ausgang | Verbindung verläuft in beide Richtungen über die Gruppengrenze |

Die Icons erscheinen **unterhalb** des jeweiligen Nodes.

#### Checkbox: "Rand-Icons an allen Gruppen-Grenzknoten anzeigen"

Im Gruppen-Bearbeitungsmodus ist im Bearbeitungs-Panel diese Checkbox verfügbar:

- **Deaktiviert (Standard):** Icons erscheinen nur bei Nodes mit tatsächlicher externer Verbindung — also Nodes, die wirklich mit einem Node außerhalb jeder registrierten Gruppe verbunden sind
- **Aktiviert:** Icons erscheinen bei allen Grenzknoten der Gruppe, auch wenn aktuell noch keine externe Verbindung besteht. Nützlich um potenzielle Übergangspunkte beim Aufbau eines neuen Kurses zu erkunden.

> **Tipp:** Die Einstellung wird in den Optionen gespeichert und bleibt nach einem Neustart aktiv.

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
