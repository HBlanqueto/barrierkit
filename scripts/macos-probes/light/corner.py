import subprocess, sys, math, os
# corner.py white.png black.png x0 x1 y0 y1 corner(tl|bl|tr|br)
# coverage per pixel from the white/black pair, inset of each row from the straight side, fitted circle radius
L = os.path.dirname(os.path.abspath(__file__))
w, b = sys.argv[1], sys.argv[2]
x0, x1, y0, y1 = map(int, sys.argv[3:7]); corner = sys.argv[7]
def rows(f): return {y: [int(l.split()[2]) for l in subprocess.run([L + "/rd", f, "row", str(y), str(x0), str(x1)], capture_output=True, text=True).stdout.splitlines()] for y in range(y0, y1 + 1)}
B = rows(b)
W = rows(w) if w != '-' else {y: [255] * len(v) for y, v in B.items()}
if w == '-':
    vals = sorted(v for r in B.values() for v in r if v > 4); plate = vals[len(vals) // 2]
    W = {y: [255 - min(v, plate) * 0 for v in r] for y, r in B.items()}
inner = [W[y][i] - B[y][i] for y in W for i in range(len(W[y]))]
mid = sorted(v for v in inner if 50 <= v < 250); dmin = mid[len(mid) // 2]
cov = {y: [min(1, max(0, (255 - (W[y][i] - B[y][i])) / (255 - dmin))) for i in range(len(W[y]))] for y in W} if w != '-' else {y: [min(1, v / plate) for v in r] for y, r in B.items()}
ins = {}
for y, c in cov.items():
    seq = c if corner in ("tl", "bl") else c[::-1]
    ins[y] = len(seq) - sum(seq)
ins = {y: v for y, v in ins.items() if v < 0.9 * (x1 - x0)}; full = min(ins.values())
ys = sorted(ins)
if corner in ("bl", "br"): ys = ys[::-1]
prof = [ins[y] - full for y in ys]
print("inset per row from the edge:", " ".join(f"{v:.1f}" for v in prof[:40]))
best = None
for R10 in range(20, 800):
    R = R10 / 10
    for off10 in range(-20, 21):
        off = off10 / 10
        e = 0
        for h, v in enumerate(prof):
            t = h + 0.5 + off
            m = R - math.sqrt(max(0, R * R - (R - t) ** 2)) if t < R else 0
            e += (m - v) ** 2
        if best is None or e < best[0]: best = (e, R, off)
print(f"circle radius {best[1]:.1f} px, edge offset {best[2]:+.1f}, rms {math.sqrt(best[0] / len(prof)):.2f}, inner diff {dmin}")
