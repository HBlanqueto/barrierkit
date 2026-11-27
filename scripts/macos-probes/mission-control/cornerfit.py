import sys
sys.path.insert(0, sys.argv[1])
from lc import load

frames, events = load(sys.argv[2])
probe_row, probe_col, final_width, rest_width = int(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5]), float(sys.argv[6])

def green(f, x, y):
    return f[1][y][x * 3 + 1]

def crossing(values):
    for i in range(1, len(values)):
        if values[i] < 64 <= values[i - 1]:
            return i - 1 + (values[i - 1] - 64) / (values[i - 1] - values[i])
    return None

def measure(f):
    width = len(f[1][0]) // 3
    left = crossing([green(f, x, probe_row) for x in range(width)])
    top = crossing([green(f, probe_col, y) for y in range(len(f[1]))])
    if left is None or top is None: return None
    insets = []
    for y in range(int(top) + 1, int(top) + 25):
        x = crossing([green(f, x, y) for x in range(width)])
        insets.append(x - left if x is not None else 99)
    corner = next((i + 1 for i, v in enumerate(insets) if v <= 0.5), None)
    shadow = [1 - green(f, int(left) - d, probe_row) / 128 for d in range(1, min(80, int(left)))]
    extent = next((d + 1 for d, a in enumerate(shadow) if a < 0.02), None)
    above = [1 - green(f, probe_col, int(top) - d) / 128 for d in range(1, int(top))]
    top_extent = next((d + 1 for d, a in enumerate(above) if a < 0.02), None)
    return left, top, corner, insets[:4], extent, shadow[1] if len(shadow) > 1 else None, top_extent

for te, a in [(t, a) for t, a in events if a.startswith("k")]:
    rest = measure([f for f in frames if f[0] < te][-1])
    end = measure([f for f in frames if f[0] < te + 1.0][-1])
    print("==", a, "rest", rest[:3], "end", end[:3])
    for f in frames:
        if not te < f[0] < te + 0.6: continue
        m = measure(f)
        if not m: continue
        p = (m[0] - rest[0]) / (end[0] - rest[0]) if end[0] != rest[0] else 0
        s0, s1 = (1.0, final_width / rest_width) if rest[0] > end[0] else (final_width / rest_width, 1.0)
        print("%5.0f left %6.1f top %5.1f scale %.3f corner %s insets %s shadow-extent %s a(2) %.3f top-extent %s" % ((f[0] - te) * 1000, m[0], m[1], s0 + p * (s1 - s0), m[2], " ".join("%.1f" % v for v in m[3]), m[4], m[5] or 0, m[6]))
