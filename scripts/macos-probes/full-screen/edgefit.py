import sys, math
sys.path.insert(0, sys.argv[1])
from an import spring
pts = [(float(t) / 1000, float(d)) for t, d in (p.split(":") for p in " ".join(sys.argv[2:]).split())]
for travel in (2560, 2624, None):
    best = None
    for D in ([travel] if travel else range(1800, 3400, 40)):
        for w in [i / 2 for i in range(10, 80)]:
            for z in [i / 20 for i in range(14, 70)]:
                for d in [i / 1000 for i in range(40, 110, 3)]:
                    e = sum((D * spring(u - d, w, z) - v) ** 2 for u, v in pts)
                    if best is None or e < best[0]: best = (e, D, w, z, d)
    e, D, w, z, d = best
    q = lambda f: next(u for u in [i / 1000 for i in range(3000)] if spring(u, w, z) >= f) * 1000
    print("travel %s -> D %d w0 %.1f z %.2f start %.0f ms rms %.1f px | t10 %.0f t50 %.0f t90 %.0f t99 %.0f" % (travel, D, w, z, d * 1000, math.sqrt(e / len(pts)), q(.1), q(.5), q(.9), q(.99)))
