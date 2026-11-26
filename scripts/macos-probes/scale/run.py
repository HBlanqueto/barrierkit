import subprocess, sys, os, time
# run.py tag: every capture for one display mode into data/<tag>/ ; sp must not be running
D = "/private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/scale"
L = D + "/../light"
tag = sys.argv[1]
out = f"{D}/data/{tag}"
os.makedirs(out, exist_ok=True)
G = [0, 16, 32, 48, 64, 96, 128, 160, 192, 224, 240, 255]
sh = lambda *a: subprocess.run(a, capture_output=True, text=True).stdout
def mode(m):
    try: os.remove(f"{D}/ack")
    except FileNotFoundError: pass
    open(f"{D}/mode.tmp", "w").write(m); os.replace(f"{D}/mode.tmp", f"{D}/mode")
    for _ in range(300):
        if os.path.exists(f"{D}/ack"): return
        time.sleep(0.02)
    print("no ack", m)
def shot(name, m):
    mode(m); time.sleep(0.25); sh("screencapture", "-x", "-m", f"{out}/{name}.png")
def meta(): return {l.split()[0]: l.split()[1:] for l in open(f"{D}/meta").read().splitlines()}
def series(p): [shot(f"{p}-g{g}", f"gray:{g}") for g in G]
def kern(p, x):
    for n, m in [("e0255", f"edgev:{x}:0:255"), ("e2550", f"edgev:{x}:255:0"), ("e064", f"edgev:{x}:0:64"), ("lw", f"line:{x}:0:255"), ("lb", f"line:{x}:255:0")]: shot(f"{p}-{n}", m)
def center(r): return int(r[0]) + int(r[2]) // 2
log = open(f"{out}/log.txt", "w")
open(f"{D}/mode", "w").write("off")
sp = subprocess.Popen([D + "/sp"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(1.5)
info = sh(D + "/dk"); log.write(info)
scale = float(info.split()[4])
dock = [float(v) for v in info.split("docklist")[1].split()]
mode("scene:none")
series("dock"); kern("dock", int((dock[0] + dock[2] / 2) * scale))
for s, grays in [("dark", (255, 128, 0)), ("aqua", (255, 128, 0)), ("noshadow", (255, 0))]:
    mode(f"scene:{s}")
    for g in grays: shot(f"{s}-g{g}", f"gray:{g}")
    log.write(f"== {s}\n" + open(f"{D}/meta").read())
for s, names in [("pop", ["popover"]), ("vev", ["vev-sidebar", "vev-windowBackground"])]:
    mode(f"scene:{s}"); mode("gray:128")
    m = meta(); log.write(f"== {s}\n" + open(f"{D}/meta").read())
    series(s)
    for n in names: kern(n, center(m[n]))
mode("scene:none"); mode("gray:128")
sh(L + "/ui", "press", "com.apple.finder", "0", "menubar"); time.sleep(1.2)
wl = sh(L + "/ui", "wins"); log.write("== menu wins\n" + wl)
menu = [l.split() for l in wl.splitlines() if l.startswith("Finder 101 ")]
if menu:
    x, y, w, h = map(int, menu[0][2:6])
    series("menu"); kern("menu", int((x + w / 2) * scale))
osascript = sh("osascript", "-e", 'tell application "System Events" to key code 53'); time.sleep(0.8)
log.write("== after esc\n" + sh(L + "/ui", "wins"))
mode("quit"); time.sleep(0.5); sp.terminate()
