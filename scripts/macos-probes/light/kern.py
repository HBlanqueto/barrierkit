import subprocess, sys, math
# kern.py prefix y x0 x1 edge_x patch_x patch_y patch_w patch_h
# tone from the gray series at the patch, inverted; then fits sigma (px) of a Gaussian to the steps and the lines
L = "/private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/light"
p, y, x0, x1, ex = sys.argv[1], *map(int, sys.argv[2:6])
px, py, pw, ph = map(int, sys.argv[6:10])
rd = lambda *a: subprocess.run([L + "/rd", *map(str, a)], capture_output=True, text=True).stdout
G = [0, 16, 32, 48, 64, 96, 128, 160, 192, 224, 240, 255]
tone = [float(rd(f"{L}/data/{p}-g{g}.png", "patch", px, py, pw, ph).split()[1]) for g in G]
print("tone(G)", " ".join(f"{t:.0f}" for t in tone))
def inv(o):
    pairs = sorted(zip(tone, G))
    if o <= pairs[0][0]: return pairs[0][1]
    for (t0, g0), (t1, g1) in zip(pairs, pairs[1:]):
        if t0 <= o <= t1: return g0 + (g1 - g0) * (o - t0) / (t1 - t0 or 1)
    return pairs[-1][1]
def prof(name): return [(int(l.split()[0]), int(l.split()[2])) for l in rd(f"{L}/data/{p}-{name}.png", "row", y, x0, x1).splitlines()]
def fit(name, a, b, kind):
    d = [(x, inv(o)) for x, o in prof(name)]
    best = None
    for s10 in list(range(2, 80)) + list(range(80, 480, 4)):
        s = s10 / 8
        for sh10 in range(-20, 21):
            sh = sh10 / 10
            err = 0
            for x, v in d:
                t = x + 0.5 - ex - sh
                if kind == "step": m = a + (b - a) * 0.5 * (1 + math.erf(t / (s * math.sqrt(2))))
                else:
                    m = a + (b - a) * (0.5 * (math.erf((t) / (s * math.sqrt(2))) - math.erf((t - 1) / (s * math.sqrt(2)))))
                err += (m - v) ** 2
            if best is None or err < best[0]: best = (err, s, sh)
    rms = math.sqrt(best[0] / len(d))
    raw = [o for _, o in prof(name)]
    print(f"{name}: sigma {best[1]:.2f} px shift {best[2]:+.1f} rms(in-levels) {rms:.1f}  raw {' '.join(str(o) for o in raw[(ex - x0) - 6:(ex - x0) + 7])}")
for n, a, b, k in [("e0255", 0, 255, "step"), ("e2550", 255, 0, "step"), ("e064", 0, 64, "step"), ("lw", 0, 255, "line"), ("lb", 255, 0, "line")]:
    if n in (sys.argv[10].split(",") if len(sys.argv) > 10 else [n]): fit(n, a, b, k)
