# Werkzeuge (Tools)

← [Benutzeroberflaeche](02-oberflaeche.md) | [Zurueck zur Uebersicht](index.md)

## Werkzeug-Katalog und Zugriff

Der Editor nutzt einen gemeinsamen Katalog fuer alle Werkzeugsurfaces. Dieselben Eintraege erscheinen in:

- der linken Seitenleiste
- der Menueleiste unter **Route-Tools**
- den Floating-Menues **`T`**, **`G`**, **`B`** und **`A`**
- der Command Palette mit **`K`** oder **`Ctrl+K`**

| Gruppe | Shortcut | Tools | Deaktiviert wenn |
|--------|----------|-------|------------------|
| **Werkzeuge** | `T` | Select, Connect, Add Node | nie |
| **Grundbefehle** | `G` | Gerade Strecke, Bezier Grad 2, Bezier Grad 3, Spline, Geglaettete Kurve | nie |
| **Bearbeiten** | `B` | Ausweichstrecke, Parkplatz, Strecke versetzen | keine geordnete Kette bei chain-basierten Tools |
| **Analyse** | `A` | Feld erkennen, Feldweg erkennen, Farb-Pfad erkennen | fehlende Farmland-Daten oder fehlende Hintergrundkarte |

Wenn ein Tool Voraussetzungen hat, bleibt es sichtbar. Statt zu verschwinden, zeigt es seinen Disabled-Grund an.

## Gemeinsame Route-Tool-Bedienung

Alle Route-Tools teilen sich denselben Grundablauf:

- Aktivierung ueber Seitenleiste, Menue, Floating-Menue oder Command Palette
- Konfiguration im schwebenden **Route-Tool**-Panel
- **`Enter`** oder **Ausfuehren** bestaetigt das aktuelle Tool
- **`Escape`** oder **Abbrechen** setzt das Tool zurueck
- Start- und Endpunkte snappen auf bestehende Nodes
- **`Pfeil hoch / runter`** aendert waehrend des Zeichnens die Node-Anzahl
- **`Pfeil links / rechts`** aendert waehrend des Zeichnens die Segmentlaenge
- Richtung und Strassenart gelten fuer die neu erzeugten Verbindungen

## Sichtbarkeit und spaetere Bearbeitung

Nicht jedes Tool erzeugt spaeter denselben Bearbeitungsweg:

- **Mit spaeterem Tool-Edit**: Gerade Strecke, Bezier Grad 2, Bezier Grad 3, Spline, Geglaettete Kurve, Ausweichstrecke, Parkplatz, Strecke versetzen, Feld erkennen
- **Ohne spaeteres Tool-Edit**: Feldweg erkennen und Farb-Pfad erkennen

Fuer Tools ohne spaeteres Tool-Edit gilt:

- das Ergebnis bleibt normal selektierbar und manuell bearbeitbar
- im Gruppen-Bearbeitungsfenster erscheint kein Button **Tool bearbeiten**
- wenn Sie dieselbe Analyse erneut parametrisieren wollen, starten Sie das Tool neu

---

## Select (T)

Das Standard-Werkzeug fuer Auswahl, Verschiebung und Abschnittsarbeit.

**Workflow:**
1. Mit **`T`** das Werkzeug-Menue oeffnen oder den Button in der Seitenleiste klicken.
2. Node per Klick selektieren, per **`Ctrl`** additiv erweitern oder per **`Shift`** als Pfad ergaenzen.
3. Selektierte Nodes per Drag verschieben.

**Konfiguration:**
- Keine eigene Tool-Konfiguration.

**Tipps:**
- Doppelklick auf einen Gruppen-Node selektiert die ganze Gruppe.
- Doppelklick ausserhalb von Gruppen selektiert den Abschnitt zwischen den naechsten Kreuzungen.

## Connect (T)

Erstellt Verbindungen zwischen zwei vorhandenen Nodes.

**Workflow:**
1. Connect aktivieren.
2. Start-Node anklicken.
3. Ziel-Node anklicken, um die Verbindung zu erstellen.

**Konfiguration:**
- Richtung: aus dem aktuellen Standard fuer Verbindungen.
- Strassenart: aus dem aktuellen Standard fuer Verbindungen.

**Tipps:**
- Fuer genau zwei selektierte Nodes ist **`C`** der schnellste Weg.
- Mit **`X`** trennen Sie die Verbindung derselben Zwei-Node-Selektion wieder.

## Add Node (T)

Setzt neue Wegpunkte direkt in die Karte.

**Workflow:**
1. Add Node aktivieren.
2. In den Viewport klicken.
3. Der Editor legt an dieser Position einen neuen Node an.

**Konfiguration:**
- Keine eigene Tool-Konfiguration.

**Tipps:**
- Wenn eine Heightmap geladen ist, wird die Hoehe beim Anlegen mitberuecksichtigt.
- Add Node eignet sich gut fuer kleine manuelle Korrekturen zwischen Tool-Ergebnissen.
- Wenn noch keine AutoDrive-Karte geladen ist, erscheint eine Statusmeldung statt eines stillen Fehlschlags.

---

## Gerade Strecke (G)

Erstellt eine lineare Strecke mit gleichmaessiger Unterteilung zwischen zwei Punkten.

**Workflow:**
1. Mit **`G`** die Gruppe **Grundbefehle** oeffnen und **Gerade Strecke** waehlen.
2. Startpunkt setzen.
3. Endpunkt setzen und die Vorschau pruefen.
4. Mit **`Enter`** bestaetigen.

**Konfiguration:**
- Segmentlaenge: maximaler Abstand zwischen erzeugten Nodes.
- Node-Anzahl: alternative Kontrolle ueber die Unterteilung.

**Tipps:**
- Ideal fuer Zufahrten, Hofverbindungen und einfache Verlaengerungen.
- Nach dem Erstellen kann die Gruppe spaeter erneut per **Tool bearbeiten** geoeffnet werden.

## Bezier Grad 2 (G)

Erstellt eine quadratische Bezier-Kurve mit einem Kontrollpunkt.

**Workflow:**
1. **Bezier Grad 2** aktivieren.
2. Start- und Endpunkt setzen.
3. Den Kontrollpunkt platzieren oder per Drag nachziehen.
4. Mit **`Enter`** bestaetigen.

**Konfiguration:**
- Segmentlaenge.
- Node-Anzahl.

**Tipps:**
- Gut fuer weiche, einfache Biegungen mit wenig Bedienaufwand.
- Start, Ende und Kontrollpunkt koennen im Viewport nach dem Setzen per Drag verfeinert werden.

## Bezier Grad 3 (G)

Erstellt eine kubische Bezier-Kurve mit zwei Kontrollpunkten und optionaler Tangenten-Ausrichtung.

**Workflow:**
1. **Bezier Grad 3** aktivieren.
2. Startpunkt und Endpunkt setzen.
3. Kontrollpunkte setzen oder vorhandene Tangenten uebernehmen.
4. Die Form per Drag anpassen und mit **`Enter`** bestaetigen.

**Konfiguration:**
- Segmentlaenge.
- Node-Anzahl.
- Start-Tangente und End-Tangente, wenn an bestehende Nodes gesnappt wurde.

**Tipps:**
- Wenn Start oder Ende auf einen vorhandenen Node snappt, kann im Panel eine Verbindung als Tangente ausgewaehlt werden.
- Die automatische Tangente ist nur ein Vorschlag und kann jederzeit durch manuelle Handles ersetzt werden.

## Spline (G)

Erstellt einen Catmull-Rom-Spline, der durch alle gesetzten Punkte verlaeuft.

**Workflow:**
1. **Spline** aktivieren.
2. Beliebig viele Punkte nacheinander setzen.
3. Die laufende Vorschau beobachten.
4. Mit **`Enter`** abschliessen.

**Konfiguration:**
- Segmentlaenge.
- Node-Anzahl.
- Start- und End-Tangente, wenn auf bestehende Nodes gesnappt wurde.

**Tipps:**
- Mit nur zwei Punkten entsteht praktisch eine gerade Verbindung.
- Mit drei oder mehr Punkten eignet sich der Spline fuer lange, organische Verlaeufe.

## Geglaettete Kurve (G)

Erstellt eine geglaettete Route mit automatischen Ein- und Auslauf-Tangenten.

**Workflow:**
1. **Geglaettete Kurve** aktivieren.
2. Start- und Endpunkt setzen.
3. Optional weitere Zwischenpunkte in der Phase **Control Nodes** setzen.
4. Steerer- und Kontrollpunkte bei Bedarf im Viewport verschieben.
5. Mit **`Enter`** bestaetigen.

**Konfiguration:**
- Segmentlaenge.
- Maximaler Winkel pro Segment.
- Reset fuer automatisch berechnete Start-/End-Steerer.

**Tipps:**
- Das Tool ist hilfreich, wenn eine Route weich an bestehende Strassenwinkel angeschlossen werden soll.
- Manuell verschobene Steerer bleiben erhalten, bis sie explizit zurueckgesetzt werden.

---

## Ausweichstrecke (B)

Erzeugt eine parallele Umgehungsstrecke mit S-foermigen An- und Abfahrten.

**Workflow:**
1. Eine geordnete Node-Kette selektieren.
2. Mit **`B`** die Gruppe **Bearbeiten** oeffnen und **Ausweichstrecke** waehlen.
3. Versatz und Abstand im Tool-Panel einstellen.
4. Vorschau pruefen und mit **`Enter`** bestaetigen.

**Konfiguration:**
- Versatz: positiv = links, negativ = rechts.
- Abstand: Grundabstand der erzeugten Nodes.
- Richtung und Strassenart fuer die neue Strecke.

**Tipps:**
- Wenn keine geordnete Kette selektiert ist, bleibt das Tool sichtbar, aber deaktiviert.
- Die Original-Kette bleibt erhalten; die Ausweichstrecke wird als eigene, spaeter editierbare Gruppe angelegt.

## Parkplatz (B)

Erstellt ein Parkplatz-Layout mit Reihen, Rampen und Ein-/Ausfahrt.

**Workflow:**
1. **Parkplatz** aktivieren.
2. Den Ursprung per Klick setzen.
3. Vor oder waehrend des Platzierens mit **`Alt+Mausrad`** drehen.
4. In der Konfigurationsphase Reihen, Abstaende und Ein-/Ausfahrt einstellen.
5. Mit **`Enter`** bestaetigen.

**Konfiguration:**
- Reihen, Laenge und Abstaende.
- Einfahrt, Ausfahrt und Rampenlaenge.
- Einfahrts- und Ausfahrtsseite.
- Richtung und Strassenart.

**Tipps:**
- Ein Klick in den Viewport startet aus der Konfigurationsphase eine Repositionierung.
- Das Tool legt eine editierbare Gruppe an und setzt Einfahrt/Ausfahrt fuer die Gruppe explizit.

## Strecke versetzen (B)

Erzeugt einen oder zwei Parallelversatze entlang einer selektierten Kette.

**Workflow:**
1. Eine geordnete Node-Kette selektieren.
2. **Strecke versetzen** aktivieren.
3. Linken und/oder rechten Versatz einschalten und Distanzen setzen.
4. Optional festlegen, ob die Original-Kette erhalten bleibt.
5. Mit **`Enter`** bestaetigen.

**Konfiguration:**
- Linker Versatz aktiv / Distanz.
- Rechter Versatz aktiv / Distanz.
- Original behalten.
- Knotenabstand auf der Offset-Strecke.

**Tipps:**
- Das Tool bleibt sichtbar, auch wenn noch keine Kette selektiert ist; der Disabled-Hinweis zeigt dann den fehlenden Schritt.
- Wenn **Original behalten** deaktiviert ist, ersetzt die neue Strecke die innere Kette im selben Undo-Schritt.

---

## Feld erkennen (A)

Zeichnet einen Feldrand aus geladenen Farmland-Daten als geschlossenen Ring nach.

**Workflow:**
1. Eine Uebersichtskarte mit Farmland-Daten laden oder generieren.
2. Mit **`A`** die Analyse-Gruppe oeffnen und **Feld erkennen** waehlen.
3. In das gewuenschte Feld klicken.
4. Offset, Toleranz, Knotenabstand und optionale Eckenverrundung einstellen.
5. Mit **`Enter`** bestaetigen.

**Konfiguration:**
- Knotenabstand.
- Offset nach innen oder aussen.
- Toleranz fuer die Vereinfachung.
- Ecken-Erkennung und optionale Eckenverrundung.

**Tipps:**
- Das Ergebnis ist spaeter erneut per **Tool bearbeiten** oeffnbar.
- Ueber **Extras -> Alle Felder nachzeichnen** koennen dieselben Parameter fuer alle Felder im Stapel angewendet werden.

## Feldweg erkennen (A)

Berechnet eine Mittellinie zwischen zwei Feldseiten und erzeugt daraus einen Fahrpfad.

**Workflow:**
1. Farmland-Daten laden.
2. **Feldweg erkennen** aktivieren.
3. Modus **Fields** oder **Boundaries** waehlen.
4. Seite 1 sammeln und bestaetigen.
5. Seite 2 sammeln und die Vorschau berechnen lassen.
6. Mit **`Enter`** den Pfad einfuegen.

**Konfiguration:**
- Modus: ganze Felder oder einzelne Grenzsegmente.
- Node-Abstand.
- Vereinfachungs-Toleranz.
- An bestehende Nodes anschliessen.

**Tipps:**
- Der Pfad bleibt normal bearbeitbar, hat aber keinen spaeteren **Tool bearbeiten**-Pfad.
- Im Boundary-Modus funktioniert der Workflow am besten mit klar voneinander getrennten Feldseiten.

## Farb-Pfad erkennen (A)

Leitet ein Wegnetz direkt aus Farbstrukturen der Hintergrundkarte ab.

**Workflow:**
1. Eine Hintergrundkarte laden.
2. **Farb-Pfad erkennen** aktivieren.
3. Mit **`Alt+Drag`** eine oder mehrere Lasso-Regionen fuer Farbproben zeichnen.
4. Im Tool-Panel die Berechnung starten.
5. Netzstatistik und Anschlussmodus pruefen.
6. Mit **`Enter`** das Netz einfuegen.

**Konfiguration:**
- Exakter Farbvergleich oder Toleranz-Modus.
- Farb-Toleranz.
- Node-Abstand.
- Vereinfachungs-Toleranz.
- Rauschfilter.
- Anschlussmodus an bestehende Verbindungen.

**Tipps:**
- Waehlen Sie mehrere Lasso-Regionen, wenn der Zielpfad aus mehreren aehnlichen Farbinseln besteht.
- Das Ergebnis ist nicht ueber **Tool bearbeiten** rekonstruierbar; fuer andere Parameter muss das Sampling erneut gestartet werden.

---

← [Benutzeroberflaeche](02-oberflaeche.md) | [Zurueck zur Uebersicht](index.md) | → [Bearbeitung](04-bearbeitung.md)
