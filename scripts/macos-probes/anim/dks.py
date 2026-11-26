import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
rest = {"R": 1549.5, "G": 1612.5, "Y": 1738.5, "C": 1801.5}
moves = [(t, a) for t, a in ev if a.startswith("m:") and a.endswith("1395")]
prof = {}
for i, (t, a) in enumerate(moves):
    px = float(a[2:].split(",")[0])
    nxt = moves[i + 1][0] if i + 1 < len(moves) else t + 0.16
    last = [x for x in fr if x[0] < nxt - 0.003]
    if not last: continue
    x = last[-1]
    for c, rc in rest.items():
        b = bbox(x[1][c], sc, minc=3)
        if b: prof.setdefault(round((px - rc) / 5) * 5, []).append(b[3])
for d in sorted(prof):
    if abs(d) <= 330: print("%5d %5.2f tiles  size %s" % (d, d / 63, " ".join("%.0f" % v for v in prof[d])))
