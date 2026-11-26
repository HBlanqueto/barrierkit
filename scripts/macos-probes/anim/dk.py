import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
def h(c): return lambda x: (bbox(x[1][c], sc, minc=3) or (0, 0, 0, None))[3]
for t, a in [(t, a) for t, a in ev if a.startswith("m:") and t > ev[1][0]]:
    tag = "enter" if a.endswith("1395") else "leave"
    report(tag + " G size", series(fr, t, h("G"), win=0.9, eps=1.5))
    report(tag + " R size", series(fr, t, h("R"), win=0.9, eps=1.5), fit=False)
    report(tag + " C size", series(fr, t, h("C"), win=0.9, eps=1.5), fit=False)
