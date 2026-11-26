import sys
sys.path.insert(0, sys.argv[1])
from an import *
fr, ev, sc = load(sys.argv[2])
classes = sys.argv[3]
def comp(c, i):
    def f(x):
        rows = x[1][c]
        kx, ky, sx, sy = sc
        lo, hi = 0, int(1345 / ky)
        b = bbox(rows[lo:hi], (kx, ky, sx, sy + lo * ky))
        if not b: return None
        return [b[0] + b[2] / 2, b[1] + b[3] / 2, b[2], b[3]][i]
    return f
keys = [t for t, a in ev if a == "k:126"]
for n, t in enumerate(keys):
    tag = "enter" if n % 2 == 0 else "exit "
    for c in classes:
        for i, nm in enumerate(["cx", "cy", "w", "h"]):
            try: report("%s %d %s %s" % (tag, n // 2, c, nm), series(fr, t, comp(c, i), win=1.0, eps=4))
            except ZeroDivisionError: print(tag, c, nm, "fit failed")
