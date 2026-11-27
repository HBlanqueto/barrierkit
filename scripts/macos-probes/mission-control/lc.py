import struct, sys

def load(base):
    nr, nc, W, H = map(int, open(base + ".hdr").read().split())
    size = 8 + (nr * W + nc * H) * 3
    raw = open(base + ".bin", "rb").read()
    frames = []
    for i in range(len(raw) // size):
        o = i * size
        t = struct.unpack_from("d", raw, o)[0]
        rows = [raw[o + 8 + k * W * 3:o + 8 + (k + 1) * W * 3] for k in range(nr)]
        b = o + 8 + nr * W * 3
        cols = [raw[b + k * H * 3:b + (k + 1) * H * 3] for k in range(nc)]
        frames.append((t, rows, cols))
    ev = [(float(l.split()[0]), l.split(None, 1)[1]) for l in open(base + ".ev").read().splitlines()]
    return frames, ev

def px(line, i):
    return tuple(line[i * 3:i * 3 + 3])

def segs(line, bg=171, tol=6, minlen=4):
    out, s = [], None
    for x in range(2560):
        on = abs(line[x * 3 + 1] - bg) > tol
        if on and s is None: s = x
        if not on and s is not None:
            if x - s >= minlen: out.append((s, x - 1))
            s = None
    if s is not None: out.append((s, 2559))
    return out
