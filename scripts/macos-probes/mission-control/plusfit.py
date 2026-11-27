import sys
sys.path.insert(0, sys.path[0] + "/../anim")
from lc import load, px
from an import series, report

def thumbs(line, gap=6, minlen=12):
    out = []
    for x in range(2560):
        r, g, b = px(line, x)
        if abs(r - 171) > 20 and b - r < 80:
            if out and x - out[-1][1] <= gap: out[-1][1] = x
            else: out.append([x, x])
    return [(l, r) for l, r in out if r - l + 1 >= minlen and (r < 2470 or r - l >= 150)]

base, mode = sys.argv[1], sys.argv[2]
fr, ev = load(base)
te = [t for t, a in ev if a.startswith(("u:", "d:"))][-1]
pre = [t for t, a in ev if a.startswith(("u:", "d:"))][0]
cache = {f[0]: thumbs(f[1][3]) for f in fr if f[0] > pre - 0.05}
for f in fr:
    if f[0] in cache and (mode == "v" or f[0] < te + 0.12):
        print("%6.0f" % ((f[0] - te) * 1000), cache[f[0]], "lab", thumbs(f[1][5], 12, 3)[:1] + thumbs(f[1][5], 12, 3)[-1:], "plus", px(f[2][-1], 77)[0], px(f[2][-1], 19)[0])
n = len(cache[min(cache)])
for k in range(n):
    s = series([f for f in fr if f[0] in cache], te, lambda f: (cache[f[0]][k][0] + cache[f[0]][k][1]) / 2 if len(cache[f[0]]) > k else None, win=1.5)
    report("thumb %d centre" % k, s)
last = cache[max(cache)]
if len(last) > n:
    s0 = series([f for f in fr if f[0] in cache], te, lambda f: (cache[f[0]][0][0] + cache[f[0]][0][1]) / 2, win=1.5)
    t0 = te + s0["delay"] / 1000
    prog = dict((round(t, 4), p) for t, p in s0["pts"])
    final = [f for f in fr if f[0] == max(cache)][0][1][3]
    L1 = last[-1][0]
    for f in fr:
        if f[0] not in cache or f[0] < te or len(cache[f[0]]) <= n: continue
        l, r = cache[f[0]][-1]
        p = prog.get(round(f[0] - t0, 4))
        full = r - l >= 155 and r < 2555
        x0 = (l - L1 * p) / (1 - p) if full and p is not None and p < 0.95 else None
        diff = sum(abs(f[1][3][(l + i) * 3 + 1] - final[(L1 + i) * 3 + 1]) for i in range(10, 150)) / 140 if full else None
        print("new %4.0f L %d R %d w %d p %s x0 %s diff %s" % ((f[0] - te) * 1000, l, r, r - l + 1, p and round(p, 3), x0 and round(x0), diff and round(diff, 1)))
