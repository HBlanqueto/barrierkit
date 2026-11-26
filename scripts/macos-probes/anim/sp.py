import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
gaps = [x[1]["B"][4][0] - x[1]["R"][4][1] - 1 for x in fr if x[1]["R"][4][2] and x[1]["B"][4][2] and x[1]["R"][4][0] == 0 and x[1]["B"][4][1] == 2559]
G = sorted(gaps)[len(gaps) // 2]
print("gap median", G, "min", min(gaps), "max", max(gaps), "n", len(gaps))
def d1x(x):
    r, b = x[1]["R"][4], x[1]["B"][4]
    if r[2] and r[0] == 0: return r[1] + 1 - 2560
    if b[2] and b[0] > 0: return b[0] - G - 2560
    if b[2]: return -2560 - G
    return None
for t, a in [(t, a) for t, a in ev if a.startswith("k:")]:
    s = report("right" if a == "k:124" else "left ", series(fr, t, d1x, win=1.3, eps=1))
    pts = [(t, p) for t, p in s["pts"] if t < 0.75]
    print("   bez", "rms %.4f (%.1f %.1f %.1f %.1f)" % fit_bez(pts, s["last"] / 1000), "pts", " ".join("%.0f:%.3f" % (t * 1000, p) for t, p in pts[::8]))
