# Werkzeuge (Tools)

← [Benutzeroberfläche](02-oberflaeche.md) | [Zurück zur Übersicht](index.md)

## Select-Tool (1)

Das Standard-Werkzeug für Auswahl und Verschiebung von Nodes.

**Funktionen:**
- Einzelklick: Node selektieren (Pick-Radius: 12px)
- Ctrl+Klick: Additiv selektieren
- Shift+Klick: Pfad-Selektion (kürzester Pfad von Anker zu Ziel)
- Doppelklick: Segment zwischen Kreuzungen selektieren
- Drag auf selektiertem Node: Alle selektierten Nodes verschieben
- Drag auf leerem Bereich: Kamera schwenken

---

## Connect-Tool (2)

Erstellt Verbindungen zwischen zwei Nodes.

**Workflow:**
1. Ersten Node anklicken → in Toolbar erscheint "Startknoten: [ID] → Wähle Zielknoten"
2. Zweiten Node anklicken → Verbindung wird erstellt
3. Werkzeug bleibt aktiv für weitere Verbindungen

**Standard-Einstellungen:**
- Richtung: Regular (Einbahn vom Start zum Ziel)
- Priorität: Regular (Hauptstraße)

---

## Add-Node-Tool (3)

Platziert neue Wegpunkte auf der Karte.

**Workflow:**
- Klick auf eine beliebige Stelle → neuer Node wird an der Welt-Position eingefügt
- Der neue Node erhält automatisch die nächste freie ID

---

## Route-Tools (4)

Erstellt Strecken und Kurse über vordefinierte Geometrien. Im Route-Modus stehen drei Sub-Tools zur Verfügung:

### 📏 Gerade Strecke

Zeichnet eine gerade Linie zwischen zwei Punkten mit automatischen Zwischen-Nodes.

**Workflow:**
1. Startpunkt klicken
2. Endpunkt klicken → Vorschau erscheint
3. Enter → Strecke wird erstellt

**Einstellungen:** Min. Abstand (Segment-Länge) und Anzahl Nodes.

---

### 🔀 Kurve (Bézier)

Zeichnet eine Bézier-Kurve (Grad 2 oder 3) mit Steuerpunkten.

**Workflow:**
1. Startpunkt klicken
2. Endpunkt klicken
3. Steuerpunkt(e) klicken → Vorschau erscheint
4. Optional: Punkte per Drag anpassen
5. Enter → Kurve wird erstellt

**Einstellungen:** Grad (Quadratisch/Kubisch), Min. Abstand, Anzahl Nodes.

---

### 〰️ Spline (Catmull-Rom)

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

---

## Gemeinsame Eigenschaften aller Route-Tools

- **Enter** bestätigt und erstellt die Route
- **Escape** bricht ab und setzt das Tool zurück
- **Verkettung:** Nach Erstellung wird der letzte Endpunkt als neuer Startpunkt übernommen. Das Tool bleibt aktiv — der nächste Klick setzt den neuen Endpunkt. So können zusammenhängende Strecken nahtlos hintereinander erstellt werden.
- **Nachbearbeitung:** Segment-Länge/Node-Anzahl können nach Erstellung per Slider angepasst werden. Die zuletzt erstellte Strecke wird automatisch gelöscht und mit den neuen Parametern neu berechnet.
- **Snap:** Start- und Endpunkte rasten auf existierende Nodes ein (Snap-Radius: 3m)

---

## Tangent-Ausrichtung (Kurve und Spline)

Wenn Start- oder Endpunkt einer **kubischen Bézier-Kurve** oder eines **Splines** auf einen existierenden Node snapt, kann die lokale Tangente an einer vorhandenen Verbindung ausgerichtet werden:

1. Route-Tool (Kurve oder Spline) aktivieren
2. Start- oder Endpunkt auf einen existierenden Node klicken (Snap)
3. Im **Eigenschaften-Panel** erscheint eine Tangent-Auswahl (ComboBox):
   - **Manuell** — keine automatische Tangente
   - **→ Node #42 (NO)** — Tangente entlang der Verbindung zum Nachbar-Node (mit Kompassrichtung)
4. Bei Auswahl einer Tangente wird der zugehörige Kontrollpunkt automatisch entlang der Verbindungsrichtung platziert
5. Der Tangent-Vorschlag kann durch manuelles Klicken/Drag überschrieben werden

> **Hinweis:** Tangent-Ausrichtung ist nur bei kubischen Kurven und Splines verfügbar, da diese separate Kontrollpunkte für Start und Ende haben.

---

## Control-Point-Drag (nur Kurve)

Bei der **Bézier-Kurve** können die Steuerpunkte nach dem Setzen per Drag verschoben werden:

1. In der Kontrollpunkt-Phase auf einen Steuerpunkt klicken und ziehen
2. Die Kurve wird in Echtzeit aktualisiert
3. Loslassen fixiert die neue Position

Erkannte Drag-Ziele sind der/die Kontrollpunkt(e) sowie Start- und Endpunkt.

---

← [Benutzeroberfläche](02-oberflaeche.md) | [Zurück zur Übersicht](index.md) | → [Bearbeitung](04-bearbeitung.md)
