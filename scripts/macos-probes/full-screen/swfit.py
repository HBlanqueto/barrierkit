import sys, math
sys.path.insert(0, sys.argv[1])
from an import load, bbox, spring
fr, ev, sc = load(sys.argv[2])
gaps = []
for te, act in [(t, a) for t, a in ev if a.startswith("ok:")]:
    pts = []
    for f in fr:
        b = bbox(f[1]["R"], sc)
        if te < f[0] < te + 1.3 and b and b[3] > 1000 and 4 < b[0] < 2556:
            pts.append((f[0] - te, b[0]))
    win = [f[0] for f in fr if te < f[0] < te + 0.8]
    gaps += [(b - a) * 1000 for a, b in zip(win, win[1:])]
    right = act == "ok:124"
    model = lambda u, D, w, z, d: (D - D * spring(u - d, w, z)) if right else D * spring(u - d, w, z)
    best = None
    for D in range(2560, 2700, 16):
        for w in [i / 2 for i in range(30, 70)]:
            for z in [i / 20 for i in range(16, 50)]:
                for d in [i / 1000 for i in range(0, 200, 4)]:
                    e = sum((model(u, D, w, z, d) - v) ** 2 for u, v in pts)
                    if best is None or e < best[0]: best = (e, D, w, z, d)
    e, D, w, z, d = best
    e64 = min(sum((model(u, 2624, 23, 1.72, dd) - v) ** 2 for u, v in pts) for dd in [i / 1000 for i in range(0, 200, 2)])
    q = lambda f: next(u for u in [i / 1000 for i in range(2000)] if spring(u, w, z) >= f) * 1000
    print("%s n %d first %.0f ms | travel %d (gap %d) w0 %.1f z %.2f start %.0f ms rms %.1f px | t10 %.0f t50 %.0f t90 %.0f | rms of 2624/23/1.72 %.1f px" % (
        "Desktop->FS" if right else "FS->Desktop", len(pts), pts[0][0] * 1000, D, D - 2560, w, z, d * 1000, math.sqrt(e / len(pts)), q(.1), q(.5), q(.9), math.sqrt(e64 / len(pts))))
print("frame gaps in windows: n %d max %.1f share>6ms %.2f" % (len(gaps), max(gaps), sum(g > 6 for g in gaps) / len(gaps)))
