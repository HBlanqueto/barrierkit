import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
def cy(x):
    b = bbox(x[1]["G"], sc); return b[1] + b[3] / 2 if b else None
def cx(x):
    b = bbox(x[1]["G"], sc); return b[0] + b[2] / 2 if b else None
def wtop(x):
    rows = x[1]["G"]; ys = [y for y, r in enumerate(rows) if r[2] >= 2]
    return 2 * (rows[ys[0]][1] - rows[ys[0]][0] + 1) if ys else None
def wbot(x):
    rows = x[1]["G"]; ys = [y for y, r in enumerate(rows) if r[2] >= 2]
    return 2 * (rows[ys[-1]][1] - rows[ys[-1]][0] + 1) if ys else None
for t, a in [(t, a) for t, a in ev if a.startswith("t:")]:
    tag = "mini" if a.endswith("/mini") else "rest"
    for name, f in (("cx", cx), ("cy", cy), ("wTop", wtop), ("wBot", wbot)):
        s = series(fr, t, f, win=0.9, eps=3)
        if s is None: print(tag, name, "none"); continue
        if name == "cy" and len(sys.argv) > 3:
            pts = [(u, p) for u, p in s["pts"] if u <= s["last"] / 1000 + 0.01]
            print("   bez cy rms %.4f (%.1f %.1f %.1f %.1f)" % fit_bez(pts, s["last"] / 1000))
        report(tag + " " + name, s, fit=False)
