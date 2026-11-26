import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
def edge(x):
    lum = x[2]
    if lum[0] < 34: return 0.0
    seen = False
    for y in range(3, 40):
        if lum[y] >= 36: seen = True
        elif seen and lum[y] <= 33:
            return 2 * (y - 1 + max(0, min(1, (lum[y] - 28) / 10)))
    return 0.0
def gw(c): return lambda x: (bbox(x[1][c], sc) or (0, 0, None, 0))[2]
def gcx(c): return lambda x: (lambda b: b and b[0] + b[2] / 2)(bbox(x[1][c], sc))
def gcy(c): return lambda x: (lambda b: b and b[1] + b[3] / 2)(bbox(x[1][c], sc))
keys = [(t, a) for t, a in ev if a != "end" and not a.startswith("s:")]
for t, a in keys:
    tag = "enter" if a == "k:126" else "exit "
    report(tag + " G width", series(fr, t, gw("G"), eps=3))
    report(tag + " G cx", series(fr, t, gcx("G"), eps=3), fit=False)
    report(tag + " G cy", series(fr, t, gcy("G"), eps=3), fit=False)
    report(tag + " M width", series(fr, t, gw("M"), eps=3), fit=False)
    report(tag + " strip", series(fr, t, edge, eps=1.5))
