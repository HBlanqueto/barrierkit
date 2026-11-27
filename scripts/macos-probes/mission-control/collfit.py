import sys
sys.path.insert(0, sys.path[0] + "/../anim")
from lc import load, px
from an import series, report, spring

def strip_bottom(col):
    for y in range(199, 0, -1):
        if abs(px(col, y)[1] - 171) < 4 and abs(px(col, y - 1)[1] - 171) < 4: return y

def rightmost_thumb(line):
    run = None
    for x in range(2469, 1000, -1):
        r, g, b = px(line, x)
        on = abs(r - 171) > 20 and b - r < 80
        if on and run is None: run = [x, x]
        elif on: run[0] = x
        elif run and run[0] - x > 6: break
    return run[0] if run and run[1] - run[0] >= 150 else None

fr, ev = load(sys.argv[1])
tm = [t for t, a in ev if a.startswith("m:")][-1]
td = [t for t, a in ev if a.startswith("d:")][-1]
tu = [t for t, a in ev if a.startswith("u:")][-1]
print("down %.0f up %.0f ms after the move" % ((td - tm) * 1000, (tu - tm) * 1000))
report("strip bottom from move", series(fr, tm, lambda f: strip_bottom(f[2][0]), win=1.5))
pts = [(f[0] - tu, rightmost_thumb(f[1][3])) for f in fr if f[0] > tu and (strip_bottom(f[2][0]) or 0) >= 80]
pts = [(t, x) for t, x in pts if x is not None and t < 1.5]
x1 = pts[-1][1]
best = min((sum((x1 + (x0 - x1) * (1 - spring(t - ts, w, 1.0)) - x) ** 2 for t, x in pts) / len(pts), x0, ts, w)
           for x0 in range(2380, 2620, 10) for ts in [i / 1000 for i in range(0, 120, 4)] for w in range(10, 26))
print("new thumb: first full frame %.0f ms after up at left %d; critical spring fit start %.0f ms after up, from left %d, w0 %d, rms %.1f px" % (pts[0][0] * 1000, pts[0][1], best[2] * 1000, best[1], best[3], best[0] ** 0.5))
