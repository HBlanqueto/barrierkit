import sys, math, os
# an.py tag: every measure of one mode, in device px and in pt (px / scale)
D = os.path.dirname(os.path.abspath(__file__)) + "/data/" + sys.argv[1]
W, H = map(int, open(D + "/dims").read().split())
G = [0, 16, 32, 48, 64, 96, 128, 160, 192, 224, 240, 255]
_c = {}
def img(n):
    if n not in _c: _c[n] = open(f"{D}/{n}.raw", "rb").read()
    return _c[n]
def px(n, x, y): return img(n)[y * W + x]
def patch(n, x, y, w, h): return sum(px(n, i, j) for j in range(y, y + h) for i in range(x, x + w)) / (w * h)
log = open(D + "/log.txt").read()
info = log.split("\n")[0].split(); s = float(info[4])
dock = [round(float(v) * s) for v in log.split("docklist")[1].split()[:4]]
metas = {}
for blk in log.split("== ")[1:]:
    lines = blk.splitlines()
    metas[lines[0]] = {l.split()[0]: list(map(int, l.split()[1:5])) for l in lines[2:] if "key=" in l}
    if lines[0] == "menu wins":
        menu = [l.split() for l in lines if l.startswith("Finder 101 ")]
        metas["menu"] = [round(int(v) * s) for v in menu[0][2:6]] if menu else None
S = lambda v: round(v * s)
out = []
def say(k, v, unit_px=None):
    if unit_px is None: out.append(f"{k}: {v}")
    else: out.append(f"{k}: {unit_px:.2f} px = {unit_px / s:.2f} pt  {v}")

def flat_row(n, x0, x1, y0, y1):
    best = None
    for y in range(y0, y1):
        v = [px(n, x, y) for x in range(x0, x1)]; m = sum(v) / len(v); sd = math.sqrt(sum((a - m) ** 2 for a in v) / len(v))
        if best is None or sd < best[0]: best = (sd, y)
    return best[1]

def kern(ser, y, x0, x1, ex, pxp, label):
    tone = [patch(f"{ser}-g{g}", *pxp) for g in G]
    def inv(o):
        pr = sorted(zip(tone, G))
        if o <= pr[0][0]: return pr[0][1]
        for (t0, g0), (t1, g1) in zip(pr, pr[1:]):
            if t0 <= o <= t1: return g0 + (g1 - g0) * (o - t0) / (t1 - t0 or 1)
        return pr[-1][1]
    say(f"{label} tone@128", f"(raw out at in 0/64/128/192/255: {tone[0]:.0f} {tone[4]:.0f} {tone[6]:.1f} {tone[8]:.0f} {tone[11]:.0f})")
    res = {}
    for name, a, b, kind in [("e0255", 0, 255, "step"), ("e2550", 255, 0, "step"), ("e064", 0, 64, "step"), ("lb", 255, 0, "line"), ("lw", 0, 255, "line")]:
        d = [(x, inv(px(f"{label}-{name}", x, y))) for x in range(x0, x1)]
        lw = 1.0
        best = None
        for s8 in list(range(2, 80)) + list(range(80, 640, 4)):
            sg = s8 / 8
            for sh10 in range(-30, 31, 2):
                sh = sh10 / 10; err = 0
                for x, v in d:
                    t = x + 0.5 - ex - sh
                    if kind == "step": m = a + (b - a) * 0.5 * (1 + math.erf(t / (sg * math.sqrt(2))))
                    else: m = a + (b - a) * 0.5 * (math.erf(t / (sg * math.sqrt(2))) - math.erf((t - lw) / (sg * math.sqrt(2))))
                    err += (m - v) ** 2
                if best is None or err < best[0]: best = (err, sg, sh)
        res[name] = best
        say(f"{label} sigma {name}", f"rms {math.sqrt(best[0] / len(d)):.1f} shift {best[2]:+.1f}", best[1])
    return res

def corner(wn, bn, x0, x1, y0, y1, c, label):
    import subprocess
    L = os.path.dirname(os.path.abspath(__file__)) + "/../light"
    r = subprocess.run(["python3", L + "/corner.py", f"{D}/{wn}.png" if wn != "-" else "-", f"{D}/{bn}.png", *map(str, (x0, x1, y0, y1)), c], capture_output=True, text=True).stdout.splitlines()[-1]
    R = float(r.split()[2])
    say(f"{label} corner radius", r.split(",", 1)[1].strip(), R)

# 1 Dock
cx = dock[0] + dock[2] // 2
col = [px("dock-g128", cx, y) for y in range(dock[1] - S(10), min(H, dock[1] + dock[3] + S(10)))]
ys = [dock[1] - S(10) + i for i, v in enumerate(col) if abs(v - 128) > 3]
top, bot = ys[0], ys[-1]
mid = top + S(7)
row = [px("dock-g128", x, mid) for x in range(0, W)]
xs = [x for x, v in enumerate(row) if abs(v - 128) > 3]
left, right = xs[0], xs[-1]
say("dock AX list (x y w h)", f"{dock} px = {[round(v / s, 1) for v in dock]} pt")
say("dock glass height", f"top {top} bottom {bot}", bot - top + 1)
say("dock glass width", f"left {left} right {right}", right - left + 1)
say("menu bar height (visible top inset)", f"{metas['dark'] and log.split('== dark')[1].split('visible')[1].split()[1]} pt")
say("dock reserved bottom", f"visible {log.split('visible')[1].split()[:2]} pt")
say("dock rim col@g128 (top..top+6)", " ".join(str(px("dock-g128", cx, y)) for y in range(top - 1, top + S(6) + 1)))
say("dock rim col@g0 (top..top+6)", " ".join(str(px("dock-g0", cx, y)) for y in range(top - 1, top + S(6) + 1)))
kern("dock", mid, cx - S(40), cx + S(40), cx, (cx - S(100), top + S(8), S(20), S(6)), "dock")
corner("dock-g255", "dock-g0", left - S(4), left + S(68), top - S(6), top + S(54), "tl", "dock")

# 2 shadow
def phic(x): return 0.5 * math.erfc(x / math.sqrt(2))
def shadow(shot, frame, label):
    x0, y0, w, h = frame
    bg = 255.0
    def sample(side, d):
        v = 0; n = 0
        for t in range(-S(20), S(20) + 1):
            if side == "top": v += px(shot, x0 + w // 2 + t, y0 - d)
            elif side == "bottom": v += px(shot, x0 + w // 2 + t, y0 + h - 1 + d)
            else: v += px(shot, x0 - d, y0 + h // 2 + t)
            n += 1
        return v / n
    data = [(side, d - 0.5, 1 - sample(side, d) / bg) for side in ("top", "bottom", "left") for d in range(S(1) + 1, S(60) + 1, max(1, S(1)))]
    def fitgrid(sgs, sps, dys):
        best = None
        for sg in sgs:
            for sp in sps:
                for dy in dys:
                    off = {"top": -dy, "bottom": dy, "left": 0}
                    m = [phic((t - sp - off[side]) / sg) for side, t, a in data]
                    mm = sum(q * q for q in m)
                    if mm == 0: continue
                    A = sum(q * a for q, (_, _, a) in zip(m, data)) / mm
                    rms = math.sqrt(sum((A * q - a) ** 2 for q, (_, _, a) in zip(m, data)) / len(data))
                    if best is None or rms < best[0]: best = (rms, A, sg, sp, dy)
        return best
    b = fitgrid([s * v / 2 for v in range(4, 64)], [s * v for v in range(-10, 11, 2)], [s * v for v in range(0, 30, 2)])
    b = fitgrid([b[2] + s * v / 4 for v in range(-4, 5)], [b[3] + s * v / 2 for v in range(-4, 5)], [max(0, b[4] + s * v / 2) for v in range(-4, 5)])
    say(f"{label} shadow", f"A {b[1]:.3f} spread {b[3] / s:+.1f} pt dy {b[4]:.1f} px = {b[4] / s:.2f} pt rms {b[0]:.4f}", b[2])
    say(f"{label} alpha top/bottom/left at d=1,4,10,20,40 pt", " | ".join(" ".join(f"{1 - sample(sd, S(d)) / bg:.3f}" for d in (1, 4, 10, 20, 40)) for sd in ("top", "bottom", "left")))
for sc in ("dark", "aqua"):
    for n, fr in metas[sc].items():
        state = "active" if (n == "titled-" + sc) else "inactive"
        shadow(f"{sc}-g255", fr, f"{n.split('-')[1]} {state}")
        if state == "active":
            x0, y0, w, h = fr
            say(f"{n} outline row on black (x0-3pt..x0+3pt)", " ".join(str(px(f"{sc}-g0", x, y0 + h // 2)) for x in range(x0 - S(3), x0 + S(3) + 1)))
            say(f"{n} outline col on black top (y0-3pt..y0+3pt)", " ".join(str(px(f"{sc}-g0", x0 + w // 2, y)) for y in range(y0 - S(3), y0 + S(3) + 1)))

# 3 window corners, popover, menu
for n, (x0, y0, w, h) in metas["noshadow"].items():
    corner("noshadow-g255", "noshadow-g0", x0 - S(4), x0 + S(40), y0 - S(4), y0 + S(40), "tl", n)
    corner("noshadow-g255", "noshadow-g0", x0 - S(4), x0 + S(40), y0 + h - S(40), y0 + h + S(4) - 1, "bl", n)
pw = metas["pop"]["popover"]
r = flat_row("pop-g0", pw[0], pw[0] + pw[2], pw[1] + pw[3] // 2, pw[1] + pw[3] // 2 + 1)
rowv = [px("pop-g0", x, pw[1] + pw[3] // 2) for x in range(pw[0], pw[0] + pw[2])]
pl = pw[0] + min(i for i, v in enumerate(rowv) if v >= 6); pr = pw[0] + max(i for i, v in enumerate(rowv) if v >= 6)
colv = [px("pop-g0", pl + S(60), y) for y in range(pw[1], pw[1] + pw[3])]
pb = pw[1] + max(i for i, v in enumerate(colv) if v >= 6); pt_ = pw[1] + min(i for i, v in enumerate(colv) if v >= 6)
say("popover body", f"x {pl}..{pr} y {pt_}..{pb} px ({(pr - pl + 1) / s:.1f} x {(pb - pt_ + 1) / s:.1f} pt)")
corner("pop-g255", "pop-g0", pl - S(4), pl + S(40), pb - S(40), pb + S(4), "bl", "popover")
corner("-", "pop-g0", pl - S(4), pl + S(40), pb - S(40), pb + S(4), "bl", "popover(black-only)")
pcx = (pl + pr) // 2; pcy = (pt_ + pb) // 2 + S(10)
kern("pop", pcy, pcx - S(80), pcx + S(80), pcx, (pcx - S(10), pcy - S(10), S(20), S(20)), "popover")
for n in ("vev-sidebar", "vev-windowBackground"):
    x0, y0, w, h = metas["vev"][n]; vx = x0 + w // 2; vy = y0 + h // 2
    kern("vev", vy, vx - S(110), vx + S(110), vx, (vx - S(10), vy - S(10), S(20), S(20)), n)
if metas.get("menu"):
    mx, my, mw, mh = metas["menu"]
    mx0, my0 = max(0, mx), max(S(31), my); mx1, my1 = min(W, mx + mw), min(H, my + mh)
    yc = (my0 + my1) // 2
    rowv = [px("menu-g0", x, yc) for x in range(mx0, mx1)]
    ml = mx0 + min(i for i, v in enumerate(rowv) if v >= 6); mr = mx0 + max(i for i, v in enumerate(rowv) if v >= 6)
    colv = [px("menu-g0", ml + S(60), y) for y in range(my0, my1)]
    mb = my0 + max(i for i, v in enumerate(colv) if v >= 6)
    say("menu body", f"x {ml}..{mr} bottom {mb} px (width {(mr - ml + 1) / s:.1f} pt, bottom {mb / s:.1f} pt)")
    corner("menu-g255", "menu-g0", max(0, ml - S(4)), ml + S(30), mb - S(30), mb + S(4), "bl", "menu")
    corner("-", "menu-g0", max(0, ml - S(4)), ml + S(30), mb - S(30), mb + S(4), "bl", "menu(black-only)")
    fx0, fx1 = (ml + mr) // 2, mr - S(8)
    fy = flat_row("menu-g128", fx0, fx1, my0 + S(10), mb - S(10))
    mcx = (fx0 + fx1) // 2
    kern("menu", fy, max(ml + S(4), mcx - S(40)), min(mr - S(4), mcx + S(40)), mcx, (mcx - S(8), fy, S(16), 1), "menu")
    a = [(d - 0.5, 1 - px("menu-g255", mr + d, fy) / 255) for d in range(S(2) + 1, S(60))]
    pts = [(d, v) for d, v in a if v > 0.004]
    if len(pts) > 3:
        b = min((sum((A * math.exp(-d / t) - v) ** 2 for d, v in pts), A, t) for A in [i / 1000 for i in range(5, 500, 4)] for t in [s * j / 2 for j in range(2, 80)])
        al = " ".join("%.3f" % (1 - px("menu-g255", mr + S(d), fy) / 255) for d in (1, 5, 10, 20))
        say("menu right shadow A exp(-d/tau)", f"A {b[1]:.3f} rms {math.sqrt(b[0] / len(pts)):.4f} alpha d=1,5,10,20pt: {al}", b[2])
print("\n".join(out))
