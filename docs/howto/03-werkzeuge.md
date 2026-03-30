# Werkzeuge (Tools)

← [Benutzeroberflaeche](02-oberflaeche.md) | [Zurueck zur Uebersicht](index.md)

## Select-Tool

Das Standard-Werkzeug fuer Auswahl und Verschiebung von Nodes.

**Funktionen:**

- Einzelklick: Node selektieren (Pick-Radius: 12px)
- Ctrl+Klick: Additiv selektieren
- Shift+Klick: Pfad-Selektion (kuerzester Pfad von Anker zu Ziel)
- Doppelklick: Gruppe zwischen Kreuzungen selektieren
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
- **Gruppe erstellen (Checkbox):** Steuert, ob die erstellte Route als benannte Gruppe registriert wird. Wenn aktiviert, wird die Strecke im Gruppen-Verzeichnis gelistet und kann per Doppelklick als Ganzes selektiert werden. Deaktivieren, wenn nur lose Nodes ohne Gruppen-Zugehoerigkeit gewuenscht sind.

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

## Ausweichstrecken-Tool ⤴ (B-Menue)

Generiert eine parallele Ausweichstrecke zu einer selektierten Kette von Nodes. Die Ausweichstrecke wird seitlich versetzt mit S-foermigen An- und Abfahrten verbunden, sodass Traktoren die Hauptstrecke umfahren koennen.

**Voraussetzung:** Eine zusammenhaengende Kette von Nodes muss selektiert sein, bevor das Tool aktiviert wird.

**Workflow:**

1. Nodes einer Streckenkette mit Shift+Klick oder Doppelklick selektieren
2. **B** druecken → Floating-Menue → ⤴ Ausweichstrecke waehlen
3. Im Eigenschaften-Panel erscheint die Konfiguration (Versatz, Abstand)
4. Vorschau der Ausweichstrecke ist sofort sichtbar
5. **Enter** → Ausweichstrecke wird erstellt (neue Nodes + Verbindungen)
6. **Escape** → Abbrechen

**Konfiguration:**

| Parameter | Beschreibung | Standardwert |
|-----------|--------------|-------------|
| **Versatz** | Seitlicher Abstand zur Originalkette (positiv = links, negativ = rechts) | 8 m |
| **Abstand** | Node-Abstand auf der Hauptstrecke (S-Kurven: halber Abstand) | 6 m |
| **Richtung** | Verbindungsrichtung der erzeugten Nodes | Dual |
| **Strassenart** | Prioritaet der erzeugten Verbindungen | Regular |

**Info-Anzeige im Panel:**

- Neue Nodes: Anzahl der erzeugten Bypass-Knoten
- Kette: Anzahl Nodes in der Quell-Kette
- Uebergangslaenge: Berechnetete Laenge der S-Kurven-Uebergaenge

**Tipps:**

- Groesserer Versatz erfordert eine laengere Kette (min. 2× Versatz Laenge)
- Negativer Versatz legt die Ausweichstrecke auf die rechte Seite
- Nach Erstellung bleiben die Original-Nodes unveraendert — die Ausweichstrecke ist eine eigenstaendige Gruppe
- Kette zu kurz → Vorschau bleibt leer (Meldung im Panel)

---

## Parkplatz-Tool 🅿 (B-Menue)

Erzeugt ein vollstaendiges Parkplatz-Layout mit mehreren Parkreihen, Wendekreis sowie uni­direktionaler Ein- und Ausfahrt. Map-Marker werden automatisch an den Parkpositionen gesetzt.

**Workflow:**

1. **B** druecken → Floating-Menue → 🅿 Parkplatz waehlen
2. **Phase Idle** — Vorschau folgt dem Cursor; Ausgangspunkt per Klick setzen
3. **Alt+Mausrad** waehrend Idle — Layout um den eingestellten Winkelschritt drehen
4. Klick → Position wird fixiert, **Phase Configuring** beginnt
5. Im Eigenschaften-Panel alle Parameter anpassen (Reihen, Abstaende, Einfahrt usw.)
6. **Enter** → Layout wird erstellt
7. **Escape** → Abbrechen und Zurueck zu Idle

**Phasen-Uebersicht:**

| Phase | Beschreibung | Aktion |
|-------|--------------|--------|
| **Idle** | Vorschau folgt Cursor, Rotation per Alt+Scroll | Klick → Configuring |
| **Configuring** | Position fixiert, Config-Panel aktiv | Enter (erstellen) oder Escape (abbrechen) |
| **Adjusting** | Vorschau folgt Cursor erneut (Repositionierung) | Klick → zurueck zu Configuring |

> **Tipp:** Waehrend Configuring kann durch Klick in den Viewport die Repositioniering (Adjusting) gestartet werden — so laesst sich die Position nachtraeglich korriegieren ohne das Tool neu zu starten.

**Konfiguration:**

| Parameter | Beschreibung | Standard | Bereich |
|-----------|--------------|---------|---------|
| **Reihen** | Anzahl Parkreihen | 2 | 1–10 |
| **Abstand** | Abstand zwischen benachbarten Reihen | 6 m | 4–20 m |
| **Laenge** | Laenge jeder Reihe (Ost-West-Ausdehnung) | 25 m | 10–100 m |
| **Max. Node-Abstand** | Maximaler Abstand aufeinanderfolgender Nodes in der Reihe | 5 m | 2–20 m |
| **Einfahrt** | Position der Einfahrt entlang der Reihe (0 = Ost, 1 = West) | 0,50 | 0–1 |
| **Ausfahrt** | Position der Ausfahrt entlang der Reihe (0 = Ost, 1 = West) | 0,75 | 0–1 |
| **Rampenlaenge** | Laenge der 45°-Rampen fuer Ein-/Ausfahrt | 3 m | 2–20 m |
| **Einfahrt-Seite** | Seite der Einfahrt aus Sicht des Markers | Rechts | Links / Rechts |
| **Ausfahrt-Seite** | Seite der Ausfahrt aus Sicht des Markers | Links | Links / Rechts |
| **Gruppe** | Name der Map-Marker-Gruppe fuer alle Parkbuchten | Parkplatz | Freitext |
| **Richtung** | Verbindungsrichtung der erzeugten Nodes | Dual | — |
| **Strassenart** | Prioritaet der erzeugten Verbindungen | Regular | — |
| **Winkelschritt** | Drehungs-Schrittweite bei Alt+Mausrad | 5° | Freitext |

**Tipps:**

- Einfahrt und Ausfahrt auf verschiedene Seiten legen verhindert Gegenverkehr im Parkplatz
- Rampenlaenge auf 4–6 m setzen fuer groessere Fahrzeuge mit breitem Wendekreis
- Mit Alt+Mausrad vor dem Fixieren den Parkplatz an bestehende Wege ausrichten
- Alle erzeugten Marker erscheinen unter dem konfigurierten Gruppennamen im Map-Marker-Panel

---

## Felderkennung-Tool (FieldBoundary)

Zeichnet automatisch Wegpunkte entlang der Grenze eines Feldes nach. Der Editor liest die Feldgrenzen aus der geladenen Karte (GRLE- oder PNG-Feldlayer) und erzeugt daraus einen geschlossenen Ring aus Nodes.

**Workflow:**

1. Karte mit Felddaten laden (Datei → Hintergrundkarte öffnen)
2. FieldBoundary-Tool aus dem Tool-Menü auswählen
3. Feld auf der Karte anklicken → Vorschau des Grenz-Rings erscheint
4. Parameter im Konfigurations-Panel anpassen (Offset, Toleranz usw.)
5. **Enter** → Ring wird als Gruppe von Nodes erstellt

**Konfiguration:**

| Parameter | Beschreibung |
|-----------|--------------|
| **Offset** | Abstand vom Feldrand nach innen (negativ = nach außen) in Metern |
| **Toleranz** | Vereinfachungs-Schwelle für den Ring (douglas-peucker) |
| **Knotenabstand** | Mindestabstand zwischen aufeinanderfolgenden Nodes |
| **Eckenwinkel** | Winkel-Schwelle zur Erkennung von Ecken (0° = keine Ecken) |
| **Ecken verrunden** | Checkbox — verrundet konvexe Ecken durch einen Bogenbogen |
| **Radius** | Verrundungs-Radius (nur sichtbar wenn "Ecken verrunden" aktiviert) |
| **Gruppe erstellen** | Ob der Ring als benannte Gruppe registriert wird |

**Eckenverrundung:**

- Nur **konvexe** Ecken werden verrundet — konkave Ecken (einspringende Winkel) bleiben scharf
- Verrundete Nodes erhalten intern den Flag `RoundedCorner` und werden bei der Distanzberechnung übersprungen
- Der Bogen wird gleichmäßig über Tangentenpunkte aufgespannt; der Radius wird auf max. 40 % der kürzeren Kante begrenzt
- Überschreibbar: Nach dem Erstellen kann der Ring wie jede andere Gruppe nachbearbeitet werden

**Alle Felder nachzeichnen:**

Im Hauptmenü unter **Werkzeuge → Alle Felder nachzeichnen** steht ein Dialog bereit, um alle Felder der Karte in einem Schritt zu verarbeiten. Die Konfigurations-Optionen (inkl. Eckenverrundung) gelten für alle erzeugten Ringe.

**Tipps:**

- Kleiner Offset (z. B. 2 m) erzeugt eine fahrbahre Innenlinie
- Bei unruhigen Feldgrenzen hilft eine höhere Toleranz (z. B. 1,5 m)
- Eckenverrundung mit Radius 3–5 m verbessert die Einfahrten in enge Ecken für AutoDrive

---

← [Benutzeroberflaeche](02-oberflaeche.md) | [Zurueck zur Uebersicht](index.md) | → [Bearbeitung](04-bearbeitung.md)
