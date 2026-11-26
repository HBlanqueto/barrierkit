import collections, math, os

HERE = os.path.dirname(os.path.abspath(__file__))
MULTI = (20, 80, 2540, 1341)
SINGLE = (20, 97, 2540, 1324)
SINGLE_MAX = (2520, 1220)
BEST = dict(g=13.0, step=0.2, iters=3000, tie=(1.0, 1.0), order='fwd')


def load():
    data = collections.OrderedDict()
    for line in open(os.path.join(HERE, 'layout-results.txt')).readlines()[1:]:
        name, orig, thumb, _ = [p.strip() for p in line.split('|')]
        o = list(map(float, orig.split()))
        t = list(map(int, thumb.split()))
        data.setdefault(name, []).append((o, (t[0] - 1, t[1] - 1, t[2] + 2, t[3] + 2)))
    return data


def clamp(r, a):
    x, y, w, h = r
    x = min(max(x, a[0]), a[2] - w) if w <= a[2] - a[0] else (a[0] + a[2] - w) / 2
    y = min(max(y, a[1]), a[3] - h) if h <= a[3] - a[1] else (a[1] + a[3] - h) / 2
    return [x, y, w, h]


def overlapping(a, b, g):
    e = 1e-6
    return (a[0] < b[0] + b[2] + g - e and b[0] < a[0] + a[2] + g - e
            and a[1] < b[1] + b[3] + g - e and b[1] < a[1] + a[3] + g - e)


def scaled(windows, s, area):
    return [clamp([o[0] + o[2] * (1 - s) / 2, o[1] + o[3] * (1 - s) / 2, o[2] * s, o[3] * s], area) for o in windows]


def push(a, b, p):
    dx = b[0] + b[2] / 2 - a[0] - a[2] / 2
    dy = b[1] + b[3] / 2 - a[1] - a[3] / 2
    if abs(dx) < 1e-9 and abs(dy) < 1e-9:
        dx, dy = p['tie']
    length = math.hypot(dx, dy)
    ux, uy = dx / length, dy / length
    needed = []
    if abs(ux) > 1e-9:
        needed.append(((a[2] + b[2]) / 2 + p['g'] - abs(dx)) / abs(ux))
    if abs(uy) > 1e-9:
        needed.append(((a[3] + b[3]) / 2 + p['g'] - abs(dy)) / abs(uy))
    t = min(needed) * p['step'] / 2
    a[0] -= ux * t; a[1] -= uy * t
    b[0] += ux * t; b[1] += uy * t


def settle(rects, area, p):
    pairs = [(i, j) for i in range(len(rects)) for j in range(i + 1, len(rects))]
    if p['order'] == 'rev':
        pairs.reverse()
    for _ in range(p['iters']):
        touched = False
        for i, j in pairs:
            if overlapping(rects[i], rects[j], p['g']):
                touched = True
                push(rects[i], rects[j], p)
        rects[:] = [clamp(r, area) for r in rects]
        if not touched:
            return True
    return False


def layout(windows, p=BEST):
    if len(windows) == 1:
        o = windows[0]
        return scaled(windows, min(1, SINGLE_MAX[0] / o[2], SINGLE_MAX[1] / o[3]), SINGLE)
    a = MULTI
    top = min(1, (a[2] - a[0]) / max(o[2] for o in windows), (a[3] - a[1]) / max(o[3] for o in windows))
    rects = scaled(windows, top, a)
    if settle(rects, a, p):
        return rects
    lo, hi, best = 0.05, top, None
    for _ in range(18):
        mid = (lo + hi) / 2
        rects = scaled(windows, mid, a)
        if settle(rects, a, p):
            lo, best = mid, rects
        else:
            hi = mid
    return best


def edge_errors(pred, meas):
    return [pred[0] - meas[0], pred[1] - meas[1], pred[0] + pred[2] - meas[0] - meas[2], pred[1] + pred[3] - meas[1] - meas[3]]


def score(data, p):
    rows, every = [], []
    for name, ws in data.items():
        pred = layout([o for o, _ in ws], p)
        errs = [e for r, (_, m) in zip(pred, ws) for e in edge_errors(r, m)]
        every += errs
        rows.append((name, len(ws), ws[0][1][2] / ws[0][0][2], pred[0][2] / ws[0][0][2],
                     max(abs(e) for e in errs), math.sqrt(sum(e * e for e in errs) / len(errs))))
    return rows, max(abs(e) for e in every), math.sqrt(sum(e * e for e in every) / len(every))


def main():
    data = load()
    rows, worst, rms = score(data, BEST)
    out = ['best: own-centre scale, clamp, pairwise centre-line push (step %.2f, gap %.0f, tie %s), max s by bisection' % (BEST['step'], BEST['g'], BEST['tie']),
           '%-12s %2s %7s %7s %8s %8s' % ('arrangement', 'n', 's_meas', 's_pred', 'max_px', 'rms_px')]
    out += ['%-12s %2d %7.3f %7.3f %8.1f %8.1f' % r for r in rows]
    out.append('overall max %.1f rms %.1f' % (worst, rms))
    out.append('sensitivity (overall max / rms):')
    for key, values in (('g', (11.0, 12.0, 14.0, 15.0)), ('step', (0.1, 0.15, 0.3, 0.5, 1.0)), ('order', ('rev',)), ('tie', ((1.0, 0.0),))):
        for v in values:
            _, w, r = score(data, dict(BEST, **{key: v}))
            out.append('  %s=%s  %.1f / %.1f' % (key, v, w, r))
    text = '\n'.join(out) + '\n'
    open(os.path.join(HERE, 'layoutfit-results.txt'), 'w').write(text)
    print(text, end='')


if __name__ == '__main__':
    main()
