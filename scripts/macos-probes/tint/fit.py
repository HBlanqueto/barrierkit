import os, sys, subprocess, collections
T = os.path.dirname(os.path.abspath(__file__)) + '/'
C = ['red', 'green', 'blue', 'white', 'black', 'gray', 'orange', 'navy', 'ice']
sw = sys.argv[1] if len(sys.argv) > 1 else 'on'

def px(f):
    o = subprocess.run([T + 'px', f, '200', '1400', '2400', '1400'], capture_output=True, text=True).stdout.split('\n')[0].split()
    return [float(v) for v in o[2:5]]

wall = {c: px(T + 'data/wp-%s-%s-on.png' % (c, c)) for c in C}
rows = collections.defaultdict(dict)
for c in C:
    for l in subprocess.run([T + 'an', '%s-%s' % (c, sw)], capture_output=True, text=True).stdout.splitlines():
        t, ap, st, lab, r, g, b = l.split()
        rows[(lab, ap, st)][c] = [float(r), float(g), float(b)]

lin = lambda v: (v / 255 / 12.92 if v / 255 <= 0.04045 else ((v / 255 + 0.055) / 1.055) ** 2.4)
enc = lambda v: 255 * (12.92 * v if v <= 0.0031308 else 1.055 * max(v, 0) ** (1 / 2.4) - 0.055)

def ls(xs, ys):
    n = len(xs); mx = sum(xs) / n; my = sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    k = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx if sxx else 0
    return my - k * mx, k

out = []
for key in sorted(rows):
    d = rows[key]
    if len(d) < len(C): continue
    res = []
    ks = []; ns = []; e_enc = []; e_lin = []
    for ch in range(3):
        xs = [wall[c][ch] for c in C]; ys = [d[c][ch] for c in C]
        a, k = ls(xs, ys); ks.append(k); ns.append(a / (1 - k) if k < 1 else 0)
        e_enc += [y - (a + k * x) for x, y in zip(xs, ys)]
        al, kl = ls([lin(x) for x in xs], [lin(y) for y in ys])
        e_lin += [y - enc(al + kl * lin(x)) for x, y in zip(xs, ys)]
    rms = lambda e: (sum(v * v for v in e) / len(e)) ** 0.5
    span = max(max(d[c]) for c in C) - min(min(d[c]) for c in C)
    out.append('%-24s %-4s %-8s k=%.3f %.3f %.3f n=%5.1f %5.1f %5.1f encRMS/max=%.2f/%.1f linRMS/max=%.2f/%.1f span=%.0f' % (
        key[0], key[1], key[2], *ks, *ns, rms(e_enc), max(map(abs, e_enc)), rms(e_lin), max(map(abs, e_lin)), span))
print('\n'.join(out))
print('wall', {c: [round(v) for v in wall[c]] for c in C})
