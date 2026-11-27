import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'anim'))
from an import load
fr, ev, sc = load(sys.argv[1])
aspect = float(sys.argv[2]) if len(sys.argv) > 2 else 1350 / 2400
for i, (t, a) in enumerate(ev):
    if not a.startswith('dg:'): continue
    n = list(map(float, a[3:].split(',')))
    nxt = next((tt for tt, aa in ev[i + 2:] if aa.startswith(('sh:', 'dg:', 'u:'))), None)
    if not nxt or n[3] < 150: continue
    tt, rows, lum = max([f for f in fr if f[0] < nxt - 0.01], key=lambda f: f[0])
    r = rows['R']; oy = int(n[3] / 2)
    x0, x1 = r[oy][0] * 2, (r[oy][1] + 1) * 2; w = x1 - x0
    hi = oy
    while r[hi + 1][2] >= 2: hi += 1
    bottom = (hi + 1) * 2; h = w * aspect; top = bottom - h
    print("p %d,%d w %d bottom %d top %.0f fx %.3f fy %.3f" % (n[2], n[3], w, bottom, top, (n[2] - x0) / w, (n[3] - top) / h))
