import sys
sys.path.insert(0, sys.argv[1])
from an import series, report
from lc import load

base = sys.argv[2]
names = sys.argv[3].split(",")
frames, events = load(base)

def col(f, name):
    c = f[2][names.index(name)]
    return [c[y * 3 + 1] for y in range(len(c) // 3)]

def strip_edge(f):
    v = col(f, "strip")
    return float(next(y for y in range(200) if v[y] > 100))

def bright(f, name, lo):
    v = col(f, name)
    edge = int(strip_edge(f))
    ys = [y for y in range(edge) if v[y] >= lo]
    return (ys[-1] + 1, max(v[y] for y in ys)) if ys else (None, max(v[:edge] or [0]))

def dock_top(f):
    v = col(f, "dock")
    return float(next((y for y in range(1345, 1440) if abs(v[y] - 128) > 3), 1440))

def dock_bottom(f):
    v = col(f, "dock")
    return float(next((y for y in range(1439, 1345, -1) if abs(v[y] - 128) > 3 and v[y] < 140), 1440) + 1)

def pill_alpha(f):
    return (col(f, "empty")[700] - 128) / 40 if "empty" in names else None

win_rows = [int(i) for i in sys.argv[4].split(",")] if len(sys.argv) > 4 else []

def win_lines(f):
    return b"".join([f[1][i] for i in win_rows] + [f[2][i] for i, n in enumerate(names) if n == "win"])

def differs(u, v):
    return any(abs(u[i] - v[i]) > 8 for i in range(1, len(u), 3))

def windows(te):
    before = win_lines([f for f in frames if f[0] < te][-1])
    after = [f for f in frames if te <= f[0] < te + 1.2]
    final = win_lines(after[-1])
    first = next((f[0] for f in after if differs(win_lines(f), before)), None)
    settle = max((f[0] for f in after if differs(win_lines(f), final)), default=None)
    if first: print("windows                first %.0f settle %.0f" % ((first - te) * 1000, (settle - te) * 1000))

keys = [(t, a) for t, a in events if a.startswith("k")]
for te, a in keys:
    print("==", a, "at %.3f" % te)
    print(" ms    strip  pill(bot,max) label(bot,max) plus(bot,max) dock(top,bot) emptyA")
    for f in frames:
        if not te - 0.01 < f[0] < te + 0.75: continue
        print("%5.0f %5.0f  %-12s %-12s %-12s %4.0f %4.0f  %s" % ((f[0] - te) * 1000, strip_edge(f), bright(f, "pill", 120), bright(f, "label", 140), bright(f, "plus", 150), dock_top(f), dock_bottom(f), "%.2f" % pill_alpha(f) if "empty" in names else "-"))
    report("strip", series(frames, te, strip_edge, win=0.7, eps=0.5))
    report("dock top", series(frames, te, dock_top, win=1.0, eps=0.5))
    if win_rows: windows(te)
    if "empty" in names: report("empty pill alpha", series(frames, te, pill_alpha, win=1.0, eps=0.02))
