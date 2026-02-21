import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
from matplotlib.patches import FancyArrowPatch

# --- XML-Rohdaten als Strings ---
id_str       = "1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36"
x_str        = "4.560,-7.727,-10.710,-22.629,-20.997,-23.740,-34.661,-29.193,-26.151,-14.163,0.593,3.616,7.682,-20.780,-27.151,-29.287,-31.441,-33.557,-35.660,-37.806,-39.825,-41.849,39.853,45.036,48.029,42.922,45.779,47.624,50.049,33.281,31.862,29.946,31.091,33.613,37.779,41.928"
z_str        = "-1679.041,-1668.809,-1668.346,-1666.496,-1644.232,-1645.531,-1650.689,-1679.502,-1679.786,-1681.010,-1682.820,-1683.458,-1684.100,-1634.625,-1635.042,-1635.177,-1635.312,-1635.446,-1635.582,-1635.720,-1635.850,-1635.981,-1677.771,-1672.502,-1667.215,-1678.551,-1677.334,-1676.144,-1674.997,-1681.256,-1681.925,-1683.022,-1683.069,-1683.053,-1683.227,-1683.233"
out_str      = "-1;3;4;-1;6;7;-1;9;8,10;9;12;11,13;12;15;16;17;18;19;20;21;22;-1;24,26,30;25;-1;27;28;29;29;31;32;33;32,34;33,35;34,36;35,36,36"
incoming_str = "-1;-1;2;3;-1;5;6;9;8,10;9;12;11,13;12;-1;-1;-1;-1;-1;-1;-1;-1;-1;-1;23;24;-1,23;26;27;28,29;-1;-1;33;32,34;33,35;34,36;35,36,36"
flags_str    = "0,0,0,0,1,1,1,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,0,0"

# --- Parsen der Strings in Listen ---
ids       = [int(i) for i in id_str.split(',')]
x_vals    = [float(v) for v in x_str.split(',')]
z_vals    = [float(v) for v in z_str.split(',')]
flags     = [int(f) for f in flags_str.split(',')]

# Splitte 'out' und 'incoming' pro Wegpunkt, behalte "-1" als int
outs      = [[int(o) for o in entry.split(',')] for entry in out_str.split(';')]
incomings = [[int(i) for i in entry.split(',')] for entry in incoming_str.split(';')]

# --- Koordinaten-Dictionary ---
coords = {ids[i]: (x_vals[i], z_vals[i]) for i in range(len(ids))}

# Wir interessieren uns nur für Punkte 23–36
selected = set(range(23, 37))

# --- Kanten extrahieren ---
directed = [
    (ids[i], tgt)
    for i in range(len(ids))
    for tgt in outs[i]
    if ids[i] in selected and tgt in selected and tgt != -1
]
edge_set = set(directed)

# --- Kriterien zur Klassifikation der Verbindungen ---

# 1) Bidirektional:
#    Zwischen a und b existieren Verbindungen in beiden Richtungen.
bidirectional = {
    tuple(sorted(e))
    for e in edge_set
    if (e[1], e[0]) in edge_set
}

# 2) Rückwärtsfahrend:
#    Wenn p1.out = p2, aber p1 NICHT in den originalen <incoming> von p2 steht,
#    dann darf man nur im Rückwärtsgang von p1 nach p2 fahren.
backwards = [
    (src, tgt)
    for (src, tgt) in directed
    if src not in incomings[ids.index(tgt)]
]

# 3) Einbahnstraße (ohne Bidirektion & ohne Rückwärtsfahrend):
#    Restliche gerichtete Verbindungen.
one_way = (
    edge_set
    - set(backwards)
    - {
        (a, b)
        for (a, b) in directed
        if tuple(sorted((a, b))) in bidirectional
    }
)

# 4) Priorisierung:
#    Die Einbahnstraßen werden nach dem Flag des ZIELpunkts (tgt) eingeteilt:
#    - Flag tgt = 0 → Prio (rot)
#    - Flag tgt = 1 → Subprio (orange)
prio    = [e for e in one_way if flags[e[1]-1] == 0]
subprio = [e for e in one_way if flags[e[1]-1] == 1]

# --- Visualisierung ---
plt.figure(figsize=(8, 6))

# Punkte plotten und labeln
for pt in selected:
    px, pz = coords[pt]
    plt.scatter(px, pz, color='black')
    plt.text(px + 0.3, pz + 0.3, str(pt), fontsize=8)

# Bidirektionale Verbindungen (grün)
for a, b in bidirectional:
    x1, z1 = coords[a]; x2, z2 = coords[b]
    plt.plot([x1, x2], [z1, z2], color='green', linewidth=2)

# Prio-Einbahn (rot)
for a, b in prio:
    x1, z1 = coords[a]; x2, z2 = coords[b]
    plt.plot([x1, x2], [z1, z2], color='red')

# Subprio-Einbahn (orange gestrichelt)
for a, b in subprio:
    x1, z1 = coords[a]; x2, z2 = coords[b]
    plt.plot([x1, x2], [z1, z2], linestyle='--', color='orange')

# Rückwärtsfahrend (blau gepunktet mit Pfeil)
for src, tgt in backwards:
    x1, z1 = coords[src]; x2, z2 = coords[tgt]
    arrow = FancyArrowPatch(
        (x1, z1), (x2, z2),
        arrowstyle='->', linestyle=':', color='blue',
        mutation_scale=15
    )
    plt.gca().add_patch(arrow)

# Legende
legend_elems = [
    Line2D([0], [0], color='green', lw=2, label='Bidirektional'),
    Line2D([0], [0], color='red', label='Prio (Flag tgt=0)'),
    Line2D([0], [0], linestyle='--', color='orange', label='Subprio (Flag tgt=1)'),
    Line2D([0], [0], linestyle=':', color='blue', label='Rückwärtsfahrend'),
]
plt.legend(handles=legend_elems, loc='best')

plt.axis('equal')
plt.xlabel('X-Koordinate')
plt.ylabel('Z-Koordinate')
plt.title('Wegpunkte 23–36 mit Rückwärtsprinzip und Priorisierung')
plt.show()
