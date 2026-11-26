import math, os, sys

from layoutfit import HERE, MULTI, SINGLE, SINGLE_MAX, edge_errors, load

BEST = dict(step=0.75, gap=15.5, margin=12.5, slack=(2.0, 3.0), tie=0.25, jacobi=True, mean=True, sweeps=400, shrink=0.005)


def clamp_centre(c, size, lo, hi):
    half = size / 2
    return (lo + hi) / 2 if size > hi - lo else min(max(c, lo + half), hi - half)


def settle(windows, s, p):
    a = MULTI
    wall = (a[0] - p['slack'][0], a[1] - p['slack'][1], a[2] + p['slack'][0], a[3] + p['slack'][1])
    cx = [clamp_centre(o[0] + o[2] / 2, o[2] * s, a[0], a[2]) + i * p['tie'] for i, o in enumerate(windows)]
    cy = [clamp_centre(o[1] + o[3] / 2, o[3] * s, a[1], a[3]) + i * p['tie'] for i, o in enumerate(windows)]
    n = len(windows)
    for _ in range(p['sweeps']):
        mx, my, partners = [0.0] * n, [0.0] * n, [0] * n
        overlap = False
        for i in range(n):
            for j in range(i + 1, n):
                dx, dy = cx[j] - cx[i], cy[j] - cy[i]
                if not p['jacobi']:
                    dx += mx[j] - mx[i]
                    dy += my[j] - my[i]
                sx = (windows[i][2] + windows[j][2]) * s / 2
                sy = (windows[i][3] + windows[j][3]) * s / 2
                if abs(dx) >= sx + p['margin'] or abs(dy) >= sy + p['margin']:
                    continue
                overlap = True
                if abs(dx) < 1e-9 and abs(dy) < 1e-9:
                    dx = dy = 1e-9
                length = math.hypot(dx, dy)
                ux, uy = dx / length, dy / length
                need = min((sx + p['gap'] - abs(dx)) / abs(ux) if abs(ux) > 1e-12 else 1e18,
                           (sy + p['gap'] - abs(dy)) / abs(uy) if abs(uy) > 1e-12 else 1e18)
                t = p['step'] * need / 2
                mx[i] -= ux * t; my[i] -= uy * t
                mx[j] += ux * t; my[j] += uy * t
                partners[i] += 1; partners[j] += 1
        if not overlap:
            break
        cornered = False
        for i, o in enumerate(windows):
            if p['mean'] and partners[i]:
                mx[i] /= partners[i]; my[i] /= partners[i]
            x = clamp_centre(cx[i] + mx[i], o[2] * s, wall[0], wall[2])
            y = clamp_centre(cy[i] + my[i], o[3] * s, wall[1], wall[3])
            cornered = cornered or (x != cx[i] + mx[i] and y != cy[i] + my[i])
            cx[i], cy[i] = x, y
        if cornered:
            s *= 1 - p['shrink']
    return not overlap, [[cx[i] - o[2] * s / 2, cy[i] - o[3] * s / 2, o[2] * s, o[3] * s] for i, o in enumerate(windows)]


def layout(windows, p=BEST):
    if len(windows) == 1:
        o = windows[0]
        s = min(1, SINGLE_MAX[0] / o[2], SINGLE_MAX[1] / o[3])
        w, h = o[2] * s, o[3] * s
        return [[clamp_centre(o[0] + o[2] / 2, w, SINGLE[0], SINGLE[2]) - w / 2, clamp_centre(o[1] + o[3] / 2, h, SINGLE[1], SINGLE[3]) - h / 2, w, h]]
    a = MULTI
    top = min(1, (a[2] - a[0]) / max(o[2] for o in windows), (a[3] - a[1]) / max(o[3] for o in windows))
    settled, best = settle(windows, top, p)
    if settled:
        return best
    lo, hi = 0.05, top
    for _ in range(18):
        mid = (lo + hi) / 2
        settled, rects = settle(windows, mid, p)
        if settled:
            lo, best = mid, rects
        else:
            hi = mid
    return best


def score(data, p):
    rows, every = [], []
    for name, ws in data.items():
        pred = layout([o for o, _ in ws], p)
        errs = [e for r, (_, m) in zip(pred, ws) for e in edge_errors(r, m)]
        every += errs
        rows.append((name, len(ws), ws[0][1][2] / ws[0][0][2], pred[0][2] / ws[0][0][2], max(abs(e) for e in errs)))
    return rows, every


def main():
    data = load()
    if len(sys.argv) > 1:
        for key, values in (('step', (0.5, 0.6, 0.7, 0.75, 0.8, 1.0)), ('sweeps', (12, 40, 3000)), ('jacobi', (False,)), ('mean', (False,)), ('shrink', (0.0, 0.01, 0.02))):
            for v in values:
                p = dict(BEST, **{key: v})
                rows, every = score(data, p)
                print(key, v, 'within3 %d within10 %d of %d' % (sum(r[4] <= 3 for r in rows), sum(r[4] <= 10 for r in rows), len(rows)),
                      ' '.join('%s=%.0f' % (r[0], r[4]) for r in rows if r[0] in sys.argv[1].split(',')))
        return
    rows, every = score(data, BEST)
    out = ['push %.2f of the needed separation per sweep along the centre line, target gap %.1f, overlap margin %.1f, walls %s outside the area, tie %.2f per stacking index, scale times %.3f per sweep with a cornered window, at most %d sweeps' % (BEST['step'], BEST['gap'], BEST['margin'], BEST['slack'], BEST['tie'], 1 - BEST['shrink'], BEST['sweeps']),
           '%-12s %2s %7s %7s %8s' % ('arrangement', 'n', 's_meas', 's_pred', 'max_px')]
    out += ['%-12s %2d %7.3f %7.3f %8.1f' % r for r in rows]
    out.append('within 3: %d, within 10: %d, of %d arrangements' % (sum(r[4] <= 3 for r in rows), sum(r[4] <= 10 for r in rows), len(rows)))
    text = '\n'.join(out) + '\n'
    open(os.path.join(HERE, 'layoutfit2-results.txt'), 'w').write(text)
    print(text, end='')


if __name__ == '__main__':
    main()
