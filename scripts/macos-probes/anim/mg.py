import sys; sys.path.insert(0, sys.argv[1]); from an import *
fr, ev, sc = load(sys.argv[2])
def info(x):
    rows = x[1]["G"]
    ys = [y for y, r in enumerate(rows) if r[2] >= 2]
    if not ys: return None
    b = bbox(rows, sc)
    w = lambda y: 2 * (rows[y][1] - rows[y][0] + 1)
    cx = sum((rows[y][0] + rows[y][1]) for y in ys) / len(ys)
    return b, w(ys[0]), w(ys[len(ys) // 2]), w(ys[-1]), sum(rows[y][2] for y in ys) * 4
evs = [(t, a) for t, a in ev if a.startswith("t:")]
for t, a in evs:
    tag = "mini " if a.endswith("/mini") else "restore"
    after = [(x[0], info(x)) for x in fr if t <= x[0] < t + 1.2]
    pre = info([x for x in fr if x[0] < t][-1])
    ch = [(u, i) for u, i in after if i != pre]
    if not ch: print(tag, "no change"); continue
    t0 = ch[0][0] - 1 / 360
    lastch = max(u for u, i in after if i != after[-1][1])
    print("%s delay %.0f last %.0f ms" % (tag, (t0 - t) * 1000, (lastch - t0) * 1000))
    if len(sys.argv) > 3:
        for u, i in after[::int(sys.argv[3])]:
            if u < t0 - 0.01 or u > lastch + 0.02: continue
            if i is None: print("   %4.0f gone" % ((u - t0) * 1000)); continue
            b, wt, wm, wb, area = i
            print("   %4.0f bbox x %4.0f y %4.0f w %4.0f h %4.0f  centre %4.0f,%4.0f  wTop %4d wMid %4d wBot %4d area %7d" % ((u - t0) * 1000, b[0], b[1], b[2], b[3], b[0] + b[2] / 2, b[1] + b[3] / 2, wt, wm, wb, area))
