import sys
sys.path.insert(0, sys.argv[1])
from an import *
fr, ev, sc = load(sys.argv[2])
def height(x):
    lum = x[2]
    last = -1
    for y in range(len(lum)):
        if lum[y] < 190: last = y
        elif y > last + 3 and last >= 0: break
    return float(last + 1)
for t, a in [(t, a) for t, a in ev if a.startswith("m:") or a.startswith("ok:")]:
    s = report(a, series(fr, t, height, win=1.4, eps=1.5))
    if s: print("   pts", " ".join("%.0f:%.0f" % (u * 1000, s["v0"] + p * (s["v1"] - s["v0"])) for u, p in s["pts"] if u < 0.45))
