import sys
sys.path.insert(0, sys.argv[1])
from an import load, bbox
fr, ev, sc = load(sys.argv[2])
log = [l.split() for l in open(sys.argv[3]).read().splitlines()]
marks = [(float(l[0]), " ".join(l[2:4])) for l in log if "FullScreen" in l[2] or l[2] == "cmd" or "SpaceDid" in l[2]]
t0 = [t for t, a in ev if a.startswith("t:")]
for k, te in enumerate(t0):
    print("== trigger", k, "at", te)
    for t, m in marks:
        if te - 0.01 < t < te + 1.5: print("   log %+7.1f %s" % ((t - te) * 1000, m))
    prev = None
    for f in fr:
        if not (te - 0.02 < f[0] < te + float(sys.argv[4] if len(sys.argv) > 4 else 1.0)): continue
        b = lambda c: "%5d %5d %5d %5d" % tuple(map(round, bbox(f[1][c], sc))) if bbox(f[1][c], sc) else "      none            "
        gap = (f[0] - prev) * 1000 if prev else 0
        print("%7.1f gap %5.1f R %s | G %s | D %s | lum %3d %3d" % ((f[0] - te) * 1000, gap, b("R"), b("G"), b("D"), f[2][20], f[2][300]))
        prev = f[0]
