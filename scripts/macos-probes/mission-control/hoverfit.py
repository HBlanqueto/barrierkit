import sys
sys.path.insert(0, sys.argv[1])
from an import *
fr, ev, sc = load(sys.argv[2])
def amount(x): return float(sum(r[2] for r in x[1]["D"]))
for t, a in [(t, a) for t, a in ev if a.startswith("m:")]:
    report(("in  " if a == "m:1700,1000" else "out ") + a, series(fr, t, amount, win=1.2, eps=200))
