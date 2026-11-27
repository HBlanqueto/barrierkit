import sys
from lc import load, px

base, x = sys.argv[1], int(sys.argv[2])
fr, ev = load(base)
moves = [t for t, a in ev if a.startswith("m:") and int(a[2:].split(",")[1]) >= 60]
first, last = moves[0], moves[-1]
prev = None
for t, rows, cols in fr:
    shown = rows[1][(x - 6) * 3] > 230
    if t > first and shown != prev:
        print("%s at %.0f ms after first hover move, %.0f ms after last, value %d" % ("shown" if shown else "hidden", (t - first) * 1000, (t - last) * 1000, rows[1][(x - 6) * 3]))
    prev = shown
