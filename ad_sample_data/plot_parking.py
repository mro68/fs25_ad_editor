#!/usr/bin/env python3
"""Visualisiert die Parkbuchten aus Parking.xml (12 Nodes, 1 Parkplatz)."""

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

# ── Daten direkt aus Parking.xml ─────────────────────────────────────────────
ids = list(range(1, 13))

# x = Ost-West, z = Süd-Nord (wie im Editor)
x = [240.614, 268.117, 263.654, 256.822, 253.816, 250.453,
     245.713, 233.663, 251.956, 230.478, 231.613, 242.165]
z = [153.716, 152.204, 152.211, 152.210, 152.211, 152.230,
     152.210, 152.211, 150.594, 152.187, 151.890, 152.181]

# out-Verbindungen (1-basiert), getrennt durch ";"
out_raw = ";3;2,4;3,5;4,6;5,7;6,12;11,12;6;8;10;1,7,8"
out_lists = []
for entry in out_raw.split(";"):
    out_lists.append([int(v) for v in entry.split(",")] if entry else [])

# Marker: Node 2 = "parken kleine Halle 2 - 01"
marker_nodes = {2: "parken kleine Halle 2 - 01"}

pos = {i + 1: (x[i], z[i]) for i in range(12)}

# ── Rollenzuordnung ───────────────────────────────────────────────────────────
# Hauptreihe: 2-3-4-5-6-7 (bidirektional, z ≈ 152.2)
row_nodes   = {2, 3, 4, 5, 6, 7}
# Wendekreis / Teardrop: 8, 10, 11, 12 (westliches Ende)
loop_nodes  = {8, 10, 11, 12}
# Einfahrt: Node 9 (kommt von Süden, 45°)
entry_nodes = {9}
# Parken / Sonderfunktion: Node 1 (Ausfahrtziel / Querknoten)
exit_nodes  = {1}

COLOR_ROW   = "#2196F3"   # Blau  — Hauptreihe
COLOR_LOOP  = "#4CAF50"   # Grün  — Wendekreis
COLOR_ENTRY = "#E91E63"   # Pink  — Einfahrt
COLOR_EXIT  = "#FF9800"   # Orange — Ausfahrt / Sonderknoten
COLOR_DEF   = "#90A4AE"   # Grau  — Rest

def node_color(nid):
    if nid in entry_nodes: return COLOR_ENTRY
    if nid in exit_nodes:  return COLOR_EXIT
    if nid in row_nodes:   return COLOR_ROW
    if nid in loop_nodes:  return COLOR_LOOP
    return COLOR_DEF

# ── Verbindungstyp: bidirektional ermitteln ───────────────────────────────────
# Eine Verbindung gilt als bidirektional, wenn auch der Rückweg existiert.
def is_bidirectional(src, tgt):
    return src in out_lists[tgt - 1]

# ── Plot ──────────────────────────────────────────────────────────────────────
fig, ax = plt.subplots(figsize=(14, 7))
ax.set_aspect("equal")
ax.set_facecolor("#0a0a0a")
fig.patch.set_facecolor("#121212")
ax.set_title("Parking.xml — parken kleine Halle 2 (12 Nodes, 1 Parkplatz)",
             fontsize=13, fontweight="bold", color="white")
ax.set_xlabel("X  (größer = Westen)", color="#aaa")
ax.set_ylabel("Z  (größer = Norden)", color="#aaa")
ax.tick_params(colors="#666")
for spine in ax.spines.values():
    spine.set_edgecolor("#333")
ax.invert_xaxis()  # Ost=rechts, West=links (Editor-Konvention)

already_drawn = set()

for src_idx, targets in enumerate(out_lists):
    src = src_idx + 1
    sx, sz = pos[src]
    for tgt in targets:
        pair = (min(src, tgt), max(src, tgt))
        bidir = is_bidirectional(src, tgt)
        # Bidirektionale Verbindung nur einmal als gelbe Linie zeichnen
        if bidir and pair in already_drawn:
            continue
        tx, tz = pos[tgt]
        if bidir:
            already_drawn.add(pair)
            ax.plot([sx, tx], [sz, tz], color="#c8e600", lw=1.8, zorder=2)
            # Pfeil-Marker in der Mitte
            mx, mz = (sx + tx) / 2, (sz + tz) / 2
            dx, dz = tx - sx, tz - sz
            ax.annotate("", xy=(mx + dx * 0.01, mz + dz * 0.01),
                         xytext=(mx - dx * 0.01, mz - dz * 0.01),
                         arrowprops=dict(arrowstyle="-|>", color="#c8e600",
                                         lw=0, mutation_scale=10), zorder=3)
        else:
            # Unidirektional: Pfeil
            ax.annotate("", xy=(tx, tz), xytext=(sx, sz),
                         arrowprops=dict(arrowstyle="-|>", color="#5bb8ff", lw=1.4,
                                         shrinkA=7, shrinkB=7, mutation_scale=13), zorder=3)

# Nodes
for nid in ids:
    px, pz = pos[nid]
    c = node_color(nid)
    is_marker = nid in marker_nodes
    size = 220 if is_marker else 90
    lw   = 2.5  if is_marker else 0.8
    ec   = "white" if is_marker else c
    ax.scatter(px, pz, s=size, c=c, edgecolors=ec, linewidths=lw, zorder=5)
    ax.text(px, pz + 0.25, str(nid), ha="center", va="bottom",
            fontsize=7.5, fontweight="bold", color=c, zorder=6)

# Marker-Label
for nid, label in marker_nodes.items():
    px, pz = pos[nid]
    ax.annotate(f"★  {label}", xy=(px, pz),
                xytext=(px - 2.0, pz + 1.6),
                fontsize=9, fontweight="bold", color="white",
                arrowprops=dict(arrowstyle="->", color="white", lw=1),
                bbox=dict(boxstyle="round,pad=0.35", fc="#1a1a2e", ec="white", alpha=0.92),
                zorder=7)

# Einfahrt / Ausfahrt annotieren
ax.annotate("◆ EINFAHRT\n(externer Node 9)", xy=pos[9],
            xytext=(pos[9][0] + 3.5, pos[9][1] - 1.2),
            fontsize=8.5, fontweight="bold", color=COLOR_ENTRY,
            arrowprops=dict(arrowstyle="->", color=COLOR_ENTRY, lw=1.4),
            bbox=dict(boxstyle="round,pad=0.3", fc="#1a0010", ec=COLOR_ENTRY, alpha=0.92))

ax.annotate("▲ SONDER / AUSFAHRT\n(Node 1, Ziel von 12)", xy=pos[1],
            xytext=(pos[1][0] - 1.0, pos[1][1] + 1.5),
            fontsize=8.5, fontweight="bold", color=COLOR_EXIT,
            arrowprops=dict(arrowstyle="->", color=COLOR_EXIT, lw=1.4),
            bbox=dict(boxstyle="round,pad=0.3", fc="#1a0e00", ec=COLOR_EXIT, alpha=0.92))

# Legende
legend_items = [
    mpatches.Patch(color=COLOR_ROW,   label="Hauptreihe (2–7)"),
    mpatches.Patch(color=COLOR_LOOP,  label="Wendekreis (8, 10–12)"),
    mpatches.Patch(color=COLOR_ENTRY, label="Einfahrt (9)"),
    mpatches.Patch(color=COLOR_EXIT,  label="Sonder/Ausfahrt (1)"),
    plt.Line2D([0], [0], color="#c8e600", lw=2, label="bidirektional"),
    plt.Line2D([0], [0], color="#5bb8ff", lw=1.5,
               marker=">", markersize=6, label="unidirektional"),
]
ax.legend(handles=legend_items, loc="lower left", fontsize=8.5,
          facecolor="#1a1a1a", edgecolor="#555", labelcolor="white")

ax.grid(True, alpha=0.12, color="#555")
plt.tight_layout()
plt.savefig("parking_plot.png", dpi=150, facecolor=fig.get_facecolor())
print("Gespeichert: parking_plot.png")
