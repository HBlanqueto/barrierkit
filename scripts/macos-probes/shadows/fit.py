import sys, math, collections
prof = collections.defaultdict(dict)
for f in sys.argv[1:]:
    for line in open(f).read().split("\n")[1:]:
        if not line: continue
        shot, win, state, side, d, px, a = line.split()
        if "white" not in shot: continue
        prof[(shot + " " + win, state)].setdefault(side, {})[int(d) - 1] = float(a)
def phic(x): return 0.5 * math.erfc(x / math.sqrt(2))
for key, sides in sorted(prof.items()):
    data = [(side, d, sides[side][d]) for side in (("top", "left") if "popover" in key[0] else ("top", "bottom", "left")) for d in range(1, 61)]
    best = None
    for si in range(10, 300, 5):
        s = si / 10
        for sp in range(-30, 31, 2):
            sp /= 2
            for dyi in range(0, 50, 1):
                dy = dyi / 2
                off = {"top": -dy, "bottom": dy, "left": 0}
                m = [phic((d - sp - off[side]) / s) for side, d, a in data]
                A = sum(mi * a for mi, (_, _, a) in zip(m, data)) / sum(mi * mi for mi in m)
                rms = math.sqrt(sum((A * mi - a) ** 2 for mi, (_, _, a) in zip(m, data)) / len(data))
                if best is None or rms < best[0]: best = (rms, A, s, sp, dy)
    print("%-16s %-8s A %.3f sigma %.1f spread %.1f dy %.1f rms %.4f" % (key[0], key[1], best[1], best[2], best[3], best[4], best[0]))
