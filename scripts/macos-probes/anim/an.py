import struct, sys, math
CLS = "RGBYMCD"

def load(base):
    ow, oh, nc, sx, sy, sw, sh = open(base + ".hdr").read().split()
    ow, oh, nc = int(ow), int(oh), int(nc)
    sx, sy, sw, sh = map(float, (sx, sy, sw, sh))
    size = 8 + nc * oh * 6 + oh
    raw = open(base + ".bin", "rb").read()
    frames = []
    for i in range(len(raw) // size):
        b = raw[i * size:(i + 1) * size]
        t = struct.unpack_from("d", b, 0)[0]
        v = struct.unpack_from("%dh" % (nc * oh * 3), b, 8)
        rows = {CLS[k]: [(v[(k * oh + y) * 3], v[(k * oh + y) * 3 + 1], v[(k * oh + y) * 3 + 2]) for y in range(oh)] for k in range(nc)}
        lum = b[8 + nc * oh * 6:]
        frames.append((t, rows, lum))
    ev = [(float(l.split()[0]), l.split(None, 1)[1]) for l in open(base + ".ev").read().splitlines()]
    scale = (sw / ow, sh / oh, sx, sy)
    return frames, ev, scale

def bbox(rows, scale, minc=2):
    ys = [y for y, r in enumerate(rows) if r[2] >= minc]
    if not ys: return None
    x0 = min(rows[y][0] for y in ys); x1 = max(rows[y][1] for y in ys)
    kx, ky, sx, sy = scale
    return (sx + x0 * kx, sy + ys[0] * ky, (x1 - x0 + 1) * kx, (ys[-1] - ys[0] + 1) * ky)

def spring(t, w, z):
    if t <= 0: return 0.0
    if z < 1:
        wd = w * math.sqrt(1 - z * z)
        return 1 - math.exp(-z * w * t) * (math.cos(wd * t) + z * w / wd * math.sin(wd * t))
    if abs(z - 1) < 1e-6:
        return 1 - math.exp(-w * t) * (1 + w * t)
    r1 = -w * (z - math.sqrt(z * z - 1)); r2 = -w * (z + math.sqrt(z * z - 1))
    return 1 - (r2 * math.exp(r1 * t) - r1 * math.exp(r2 * t)) / (r2 - r1)

def bez(x1, y1, x2, y2, x):
    lo, hi = 0.0, 1.0
    for _ in range(30):
        s = (lo + hi) / 2
        bx = 3 * (1 - s) ** 2 * s * x1 + 3 * (1 - s) * s * s * x2 + s ** 3
        if bx < x: lo = s
        else: hi = s
    s = (lo + hi) / 2
    return 3 * (1 - s) ** 2 * s * y1 + 3 * (1 - s) * s * s * y2 + s ** 3

def rms(f, pts):
    return math.sqrt(sum((f(t) - p) ** 2 for t, p in pts) / len(pts))

def fit_spring(pts):
    def go(ws, zs, dts):
        best = None
        for w in ws:
            for z in zs:
                for dt in dts:
                    e = rms(lambda t: spring(t + dt, w, z), pts)
                    if best is None or e < best[0]: best = (e, w, z, dt)
        return best
    e, w, z, dt = go(range(4, 160, 4), [i / 20 for i in range(6, 60)], (-0.008, 0, 0.008, 0.016))
    return go([w + i * 0.5 for i in range(-8, 9) if w + i * 0.5 > 0], [z + i * 0.01 for i in range(-5, 6) if z + i * 0.01 > 0.05], [dt + i * 0.002 for i in range(-4, 5)])

def fit_bez(pts, T):
    best = None
    g = [i / 10 for i in range(11)]
    for x1 in g:
        for y1 in [i / 10 for i in range(-2, 15)]:
            for x2 in g:
                for y2 in [i / 10 for i in range(-2, 15)]:
                    e = rms(lambda t: bez(x1, y1, x2, y2, min(1, max(0, t / T))), pts[::max(1, len(pts) // 25)])
                    if best is None or e < best[0]: best = (e, x1, y1, x2, y2)
    e, x1, y1, x2, y2 = best
    return (rms(lambda t: bez(x1, y1, x2, y2, min(1, max(0, t / T))), pts), x1, y1, x2, y2)

def tpct(pts, q):
    for t, p in pts:
        if p >= q: return t
    return None

def series(fr, te, f, win=1.2, eps=0.5):
    before = [x for x in fr if x[0] < te]
    v0 = f(before[-1]) if before else f(fr[0])
    after = [(x[0], f(x)) for x in fr if te <= x[0] < te + win]
    after = [(t, v) for t, v in after if v is not None]
    if v0 is None and after: v0 = after[0][1]
    ch = [(t, v) for t, v in after if abs(v - v0) > eps]
    if not ch: return None
    t0 = ch[0][0] - 1 / 360
    v1 = after[-1][1]
    last = max(t for t, v in after if abs(v - v1) > eps) if any(abs(v - v1) > eps for t, v in after) else ch[0][0]
    pts = [(t - t0, (v - v0) / (v1 - v0)) for t, v in after if t >= t0 and v1 != v0]
    return dict(delay=(t0 - te) * 1000, v0=v0, v1=v1, last=(last - t0) * 1000, pts=pts,
                t10=tpct(pts, .1), t50=tpct(pts, .5), t90=tpct(pts, .9),
                over=max(p for t, p in pts) - 1)

def report(name, s, fit=True):
    if s is None: print(name, "no change"); return
    ms = lambda x: "%.0f" % (x * 1000) if x is not None else "-"
    line = "%-22s v %.1f->%.1f delay %.0f t10 %s t50 %s t90 %s last %.0f over %.3f" % (name, s["v0"], s["v1"], s["delay"], ms(s["t10"]), ms(s["t50"]), ms(s["t90"]), s["last"], s["over"])
    if fit:
        pts = [(t, p) for t, p in s["pts"] if t <= s["last"] / 1000 + 0.05]
        e, w, z, dt = fit_spring(pts)
        line += " | spring w0 %.0f z %.2f dt %.0fms rms %.3f" % (w, z, dt * 1000, e)
    print(line)
    return s
