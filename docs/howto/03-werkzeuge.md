# Werkzeuge (Tools)

← [Benutzeroberflaeche](02-oberflaeche.md) | [Zurueck zur Uebersicht](index.md)

## Select-Tool

Das Standard-Werkzeug fuer Auswahl und Verschiebung von Nodes.

**Funktionen:**

- Einzelklick: Node selektieren (Pick-Radius: 12px)
- Ctrl+Klick: Additiv selektieren
- Shift+Klick: Pfad-Selektion (kuerzester Pfad von Anker zu Ziel)
- Doppelklick: Segment zwischen Kreuzungen selektieren
- Drag auf selektiertem Node: Alle selektierten Nodes verschieben
- Drag auf leerem Bereich: Kamera schwenken

---

## Connect-Tool

Erstellt Verbindungen zwischen zwei Nodes.

**Workflow:**

1. Ersten Node anklicken → in Toolbar erscheint "Startknoten: [ID] → Waehle Zielknoten"
2. Zweiten Node anklicken → Verbindung wird erstellt
3. Werkzeug bleibt aktiv fuer weitere Verbindungen

**Standard-Einstellungen:**

- Richtung: Regular (Einbahn vom Start zum Ziel)
- Prioritaet: Regular (Hauptstrasse)

---

## Add-Node-Tool

Platziert neue Wegpunkte auf der Karte.

**Workflow:**

- Klick auf eine beliebige Stelle → neuer Node wird an der Welt-Position eingefuegt
- Der neue Node erhaelt automatisch die naechste freie ID

---

## Route-Tools

Erstellt Strecken und Kurse ueber vordefinierte Geometrien. Die Route-Tools werden ueber das **Werkzeuge-Floating-Menue (T)** aufgerufen. Im Route-Modus stehen drei Sub-Tools zur Verfuegung:

### 📏 Gerade Strecke

Zeichnet eine gerade Linie zwischen zwei Punkten mit automatischen Zwischen-Nodes.

**Workflow:**

1. Startpunkt klicken
2. Endpunkt klicken → Vorschau erscheint
3. Enter → Strecke wird erstellt

**Einstellungen:** Min. Abstand (Segment-Laenge) und Anzahl Nodes.

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

Zeichnet einen interpolierenden Spline, der durch **alle geklickten Punkte** fuehrt. Im Gegensatz zur Bézier-Kurve (die Steuerpunkte nur annaehert) verlaeuft der Spline exakt durch jeden gesetzten Punkt.

**Workflow:**

1. Beliebig viele Punkte nacheinander klicken (mindestens 2)
2. Vorschau wird fortlaufend aktualisiert (Cursor = naechster Punkt)
3. Enter → Spline wird erstellt

**Einstellungen:** Min. Abstand (Segment-Laenge) und Anzahl Nodes.

**Besonderheiten:**

- Ab 3 Punkten entsteht eine glatte Kurve (Catmull-Rom-Interpolation)
- Mit 2 Punkten wird eine gerade Strecke erzeugt
- Verkettung: Nach Enter wird der letzte Endpunkt automatisch als neuer Startpunkt uebernommen
- Nachbearbeitung: Segment-Laenge / Node-Anzahl koennen nach Erstellung geaendert werden

---

## Gemeinsame Eigenschaften aller Route-Tools

- **Enter** bestaetigt und erstellt die Route
- **Escape** bricht ab und setzt das Tool zurueck
- **Verkettung:** Nach Erstellung wird der letzte Endpunkt als neuer Startpunkt uebernommen. Das Tool bleibt aktiv — der naechste Klick setzt den neuen Endpunkt. So koennen zusammenhaengende Strecken nahtlos hintereinander erstellt werden.
- **Nachbearbeitung:** Segment-Laenge/Node-Anzahl koennen nach Erstellung per Slider angepasst werden. Die zuletzt erstellte Strecke wird automatisch geloescht und mit den neuen Parametern neu berechnet.
- **Snap:** Start- und Endpunkte rasten auf existierende Nodes ein (Snap-Radius: 3m)
- **Segment erstellen (Checkbox):** Steuert, ob die erstellte Route als benanntes Segment registriert wird. Wenn aktiviert, wird die Strecke im Segment-Verzeichnis gelistet und kann per Doppelklick als Ganzes selektiert werden. Deaktivieren, wenn nur lose Nodes ohne Segment-Zugehoerigkeit gewuenscht sind.

---

## Tangent-Ausrichtung (Kurve und Spline)

Wenn Start- oder Endpunkt einer **kubischen Bézier-Kurve** oder eines **Splines** auf einen existierenden Node snapt, kann die lokale Tangente an einer vorhandenen Verbindung ausgerichtet werden:

1. Route-Tool (Kurve oder Spline) aktivieren
2. Start- oder Endpunkt auf einen existierenden Node klicken (Snap)
3. Im **Eigenschaften-Panel** erscheint eine Tangent-Auswahl (ComboBox):
   - **Manuell** — keine automatische Tangente
   - **→ Node #42 (NO)** — Tangente entlang der Verbindung zum Nachbar-Node (mit Kompassrichtung)
4. Bei Auswahl einer Tangente wird der zugehoerige Kontrollpunkt automatisch entlang der Verbindungsrichtung platziert
5. Der Tangent-Vorschlag kann durch manuelles Klicken/Drag ueberschrieben werden

> **Hinweis:** Tangent-Ausrichtung ist nur bei kubischen Kurven und Splines verfuegbar, da diese separate Kontrollpunkte fuer Start und Ende haben.

---

## Control-Point-Drag (nur Kurve)

Bei der **Bézier-Kurve** koennen die Steuerpunkte nach dem Setzen per Drag verschoben werden:

1. In der Kontrollpunkt-Phase auf einen Steuerpunkt klicken und ziehen
2. Die Kurve wird in Echtzeit aktualisiert
3. Loslassen fixiert die neue Position

Erkannte Drag-Ziele sind der/die Kontrollpunkt(e) sowie Start- und Endpunkt.

---

← [Benutzeroberflaeche](02-oberflaeche.md) | [Zurueck zur Uebersicht](index.md) | → [Bearbeitung](04-bearbeitung.md)
