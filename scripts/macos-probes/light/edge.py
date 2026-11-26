import subprocess, sys, math
# edge.py prefix col|row fixed a0 a1: profile across an edge over white, gray 128 and black, then a fit of the
# shadow alpha(d) = 1 - out/bare beyond the last glass pixel (where white and black differ by < 200) as A exp(-d/tau)
L = "/private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/light"
p, kind, fixed, a0, a1 = sys.argv[1], sys.argv[2], *map(int, sys.argv[3:6])
def prof(g):
    v = [int(l.split()[2]) for l in subprocess.run([L + "/rd", f"{L}/data/{p}-g{g}.png", kind, str(fixed), str(min(a0, a1)), str(max(a0, a1))], capture_output=True, text=True).stdout.splitlines()]
    return v if a1 > a0 else v[::-1]
W, G, B = prof(255), prof(128), prof(0)
inside = [i for i in range(len(W)) if B[i] >= 8 or W[i] - B[i] < 120]
last = max(inside)
if last + 2 < len(W) and (255 - W[last + 1]) > 1.5 * (255 - W[last + 2]) + 4:
    print('outer rim pixel (white,gray,black):', W[last + 1], G[last + 1], B[last + 1]); last += 1
print("pos white gray black:", " | ".join(f"{a0 + i if a1 > a0 else a0 - i}:{W[i]},{G[i]},{B[i]}" for i in range(max(0, last - 5), min(len(W), last + 6))))
out = [(d, 1 - W[last + d] / 255, 1 - G[last + d] / 128) for d in range(1, 61) if last + d < len(W)]
print("alpha_white(d=1,2,5,10,20,40):", " ".join(f"{a:.3f}" for d, a, _ in out if d in (1, 2, 5, 10, 20, 40)))
print("alpha_gray (d=1,2,5,10,20,40):", " ".join(f"{a:.3f}" for d, _, a in out if d in (1, 2, 5, 10, 20, 40)))
pts = [(d, a) for d, a, _ in out if a > 0.004]
if len(pts) > 3:
    best = min(((sum((A * math.exp(-d / t) - a) ** 2 for d, a in pts), A, t) for A in [i / 1000 for i in range(5, 500, 2)] for t in [j / 2 for j in range(2, 120)]))
    print(f"fit A exp(-d/tau): A {best[1]:.3f} tau {best[2]:.1f} px rms {math.sqrt(best[0] / len(pts)):.4f}")
else:
    print("no shadow above 0.004")
