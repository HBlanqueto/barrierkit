import sys, math
sys.path.insert(0, sys.argv[1])
from an import *
fr, ev, sc = load(sys.argv[2])
rest = float(sys.argv[3])
def x0(x):
    b = bbox(x[1]["Y"][:336], sc)
    return b[0] if b and b[0] > 800 else None
for t, a in [(t, a) for t, a in ev if a in ("k:123", "k:124")]:
    pts = [(x[0] - t, x0(x)) for x in fr if 0 < x[0] - t < 1.3]
    pts = [(u, v) for u, v in pts if v is not None and v < 2556]
    sign = 1 if a == "k:123" else -1
    best = None
    for D in range(2500, 2800, 16):
        for w in [i / 2 for i in range(30, 70)]:
            for z in [i / 20 for i in range(16, 50)]:
                for d in [i / 1000 for i in range(0, 40, 4)]:
                    e = 0
                    for u, v in pts:
                        p = spring(u - d, w, z)
                        m = rest + D * (p if sign > 0 else 1 - p)
                        e += (m - v) ** 2
                    if best is None or e < best[0]: best = (e, D, w, z, d)
    e, D, w, z, d = best
    q = lambda f: next(u for u in [i / 1000 for i in range(2000)] if spring(u, w, z) >= f)
    print(a, "n", len(pts), "travel %d w0 %.1f z %.2f delay %.0f ms rms %.1f px | t10 %.0f t50 %.0f t90 %.0f" % (D, w, z, d * 1000, math.sqrt(e / len(pts)), q(.1) * 1000, q(.5) * 1000, q(.9) * 1000))
