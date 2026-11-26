import sys, math
sys.path.insert(0, sys.argv[1])
from an import bez
pts = [(float(t) / 1000, float(d)) for t, d in (p.split(":") for p in sys.argv[2].split())]
best = None
g = [i / 10 for i in range(11)]
for A in range(1200, 2700, 100):
  for T in [i / 100 for i in range(30, 70, 4)]:
    for st in [i / 100 for i in range(0, 11, 2)]:
      for x1 in g:
        for x2 in g:
          e = sum((A * (1 - bez(x1, 0, x2, 1, min(1, max(0, (u - st) / T)))) - v) ** 2 for u, v in pts)
          if best is None or e < best[0]: best = (e, A, T, st, x1, x2)
e, A, T, st, x1, x2 = best
print("remaining = A(1-bez): A %d px T %.0f ms start %.0f ms bezier (%.1f,0,%.1f,1) rms %.1f px n %d" % (A, T * 1000, st * 1000, x1, x2, math.sqrt(e / len(pts)), len(pts)))
