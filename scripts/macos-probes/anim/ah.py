import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
def top(x):
    b = bbox(x[1]["G"], sc, minc=3)
    return b[1] if b and b[3] >= 58 else (b[1] if b else 1440.0)
def ybot(x):
    b = bbox(x[1]["G"], sc, minc=3)
    return b[1] + b[3] if b else 1440.0
for t, a in [(t, a) for t, a in ev if a.startswith("m:") and t > ev[1][0]]:
    tag = "show" if a.endswith("1439") else "hide"
    report(tag + " G top", series(fr, t, top, win=1.4, eps=1.5))
    s = series(fr, t, top, win=1.4, eps=1.5)
    print("   pts", " ".join("%.0f:%.2f" % (u * 1000, p) for u, p in s["pts"][::3] if u < 0.4))
