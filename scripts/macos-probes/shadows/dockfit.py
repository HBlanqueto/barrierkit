import sys, math
rows = [list(map(float, l.split())) for l in open(sys.argv[1]) if l.strip()]
log = dict(l.split() for l in open(sys.argv[2]) if " " in l and not l.startswith("active"))
crit, cancel, info = float(log["critical"]), float(log["cancel"]), float(log["informational"])
REST = 1350
pts = [(t, REST - top if (top >= 0 and bot <= 1409 and top < REST) else 0) for t, top, bot, *_ in rows if t > crit]
segs, cur = [], []
for t, h in pts:
    if h > 0: cur.append((t, h))
    elif cur: segs.append(cur); cur = []
if cur: segs.append(cur)
for s in segs:
    t0 = s[0][0]; peak = max(s, key=lambda p: p[1])
    label = "crit" if t0 < cancel else "info"
    print("%s start %+.3f s after request, airborne %.3f s (first..last >0 sample), peak %d px at %.3f s" % (label, t0 - (crit if label == "crit" else info), s[-1][0] - t0, peak[1], peak[0] - t0))
starts = [s[0][0] for s in segs if s[0][0] < cancel]
print("crit bounce period", ["%.3f" % (b - a) for a, b in zip(starts, starts[1:])])
s = max(segs[:3], key=len)
t0, t1 = s[0][0] - 0.0167, s[-1][0] + 0.0167
T = t1 - t0; H = max(h for _, h in s)
def rms(f): return math.sqrt(sum((f((t - t0) / T) * H - h) ** 2 for t, h in s) / len(s))
print("airborne T %.3f peak %d" % (T, H))
print("parabola 4u(1-u) rms %.2f px" % rms(lambda u: 4 * u * (1 - u)))
print("sine sin(pi u) rms %.2f px" % rms(lambda u: math.sin(math.pi * u)))
print("samples:", " ".join("%.3f:%d" % (t - t0, h) for t, h in s))
best = min((rms(lambda u, p=p / 100: 1 - abs(2 * u - 1) ** p), p / 100) for p in range(100, 400))
print("1-|2u-1|^p best p %.2f rms %.2f px" % (best[1], best[0]))
small = segs[1]; ts0 = small[0][0] - 0.0167
print("small hop samples:", " ".join("%.3f:%d" % (t - ts0, h) for t, h in small))
print("gap big end -> small start %.3f s" % (segs[1][0][0] - segs[0][-1][0]))
