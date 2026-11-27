import sys
sys.path.insert(0, sys.argv[1])
from lc import load
base, cols = sys.argv[2], sys.argv[3]
wrows = [int(i) for i in sys.argv[4].split(",")] if len(sys.argv) > 4 else None
names = cols.split(","); fr, ev = load(base)
col = lambda f, n: [f[2][names.index(n)][y * 3 + 1] for y in range(len(f[2][0]) // 3)]
def edge(f):
    v = col(f, "strip"); return float(next(y for y in range(200) if v[y] > 100))
def visible(f, n, lo, top):
    v = col(f, n); e = int(edge(f)); return any(v[y] >= lo for y in range(min(e, top)))
def dock(f):
    v = col(f, "dock"); return float(next((y for y in range(1345, 1440) if abs(v[y] - 128) > 3), 1440))
def alpha(f): return (col(f, "empty")[700] - 128) / 40
def wl(f): return b"".join([f[1][i] for i in wrows] + [f[2][i] for i, n in enumerate(names) if n == "win"])
def diff(u, v): return any(abs(u[i] - v[i]) > 8 for i in range(1, len(u), 3))
for te, a in [(t, a) for t, a in ev if a.startswith("k")]:
    after = [f for f in fr if te <= f[0] < te + 1.0]
    before = [f for f in fr if f[0] < te][-1]
    if not after: continue
    ms = lambda t: "%.0f" % ((t - te) * 1000) if t else "-"
    def span(fn, eps):
        v0, v1 = fn(before), fn(after[-1])
        ch = [f[0] for f in after if abs(fn(f) - v0) > eps]
        st = [f[0] for f in after if abs(fn(f) - v1) > eps]
        return (ms(ch[0]) if ch else "-") + "-" + (ms(max(st)) if st else "-"), v0, v1
    entry = edge(before) < 20
    out = [base, a, "entry" if entry else "exit "]
    if wrows:
        b0, b1 = wl(before), wl(after[-1])
        fc = next((f[0] for f in after if diff(wl(f), b0)), None); ls = max((f[0] for f in after if diff(wl(f), b1)), default=None)
        out.append("win %s-%s" % (ms(fc), ms(ls)))
    out.append("strip %s" % span(edge, 0.5)[0])
    for n, lo, top in (("pill", 120, 200), ("plus", 150, 11), ("label", 140, 200)):
        vis = [f[0] for f in after if visible(f, n, lo, top)]
        out.append("%s %s" % (n, ("first " + ms(vis[0])) if entry and vis else ("last " + ms(vis[-1]) if vis else "-")))
    d = span(dock, 0.5)
    out.append("dock %s (%.0f->%.0f)" % d)
    if "empty" in names: out.append("emptypill %s" % span(alpha, 0.02)[0])
    print("  ".join(out))
