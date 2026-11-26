import sys
sys.path.insert(0, sys.argv[1])
from an import load, bbox, series, report, fit_bez
for base in sys.argv[2:]:
    fr, ev, sc = load(base)
    t0 = [t for t, a in ev if a.startswith("t:")]
    gaps = [round((b[0] - a[0]) * 1000, 1) for a, b in zip(fr, fr[1:]) if any(te < a[0] < te + 0.7 for te in t0)]
    print("##", base, "frames in transitions", len(gaps), "gap ms max", max(gaps), "share >6ms %.2f" % (sum(g > 6 for g in gaps) / len(gaps)))
    for k, te in enumerate(t0):
        for name, f in [("R width", lambda x: bbox(x[1]["R"], sc)[2] if bbox(x[1]["R"], sc) else None),
                        ("R left", lambda x: bbox(x[1]["R"], sc)[0] if bbox(x[1]["R"], sc) else None),
                        ("R top", lambda x: bbox(x[1]["R"], sc)[1] if bbox(x[1]["R"], sc) else None)]:
            s = report("%s %s" % ("enter" if k % 2 == 0 else "exit", name), series(fr, te, f, win=0.9, eps=4.5))
            if s and name == "R width":
                pts = [(t, p) for t, p in s["pts"] if t <= s["last"] / 1000 + 0.02]
                e, x1, y1, x2, y2 = fit_bez(pts, s["last"] / 1000)
                print("    bezier over last: (%.1f, %.1f, %.1f, %.1f) rms %.3f" % (x1, y1, x2, y2, e))
