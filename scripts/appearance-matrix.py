#!/usr/bin/env python3
"""Captures each surface twice on this Mac, the SwiftUI/AppKit probe and the kit's example, in light and dark,
and prints the share of pixels that differ per element. Run it in the system's current Reduce Transparency state
(`--transparency blur` or `rt`, which it checks against the system and refuses to change).

Usage: scripts/appearance-matrix.py --transparency blur|rt [--surface NAME ...] [--keep DIR]
Needs: numpy and Pillow, an unlocked session, ~/src/probes (the SwiftUI probes), `cargo build -p bk --examples`.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROBES = os.environ.get("BARRIERKIT_PROBES", os.path.expanduser("~/src/probes"))
EXAMPLES = os.path.join(os.environ.get("CARGO_TARGET_DIR", os.path.join(ROOT, "target")), "debug", "examples")
LOCK = "/tmp/barrierkit-screen.lock"
TOLERANCE = 8
SRGB = "/System/Library/ColorSync/Profiles/sRGB Profile.icc"


class Surface:
    """One surface: the kit's example, the SwiftUI/AppKit probe, the elements to compare and, per transparency state, the
    share of pixels each may differ by (`ceilings`, the measured number plus a little; the cause is in
    docs/research/appearance-matrix.md). `live` marks the surfaces of the live half."""

    def __init__(self, name, kit, probe, elements, ceilings, kit_args=(), probe_args=(), live=False, lights=None):
        self.name, self.kit, self.probe, self.elements = name, kit, probe, elements
        self.ceilings, self.kit_args, self.probe_args, self.live = ceilings, list(kit_args), list(probe_args), live
        # Where the window's red light is, when it has one the kit does not draw itself: it tells a key window from another.
        self.lights = lights


NOTES = [
    ("lights", (10, 10, 100, 42)),
    ("sidebar toggle", (170, 2, 218, 50)),
    ("sidebar rows", (10, 52, 210, 152)),
    ("sidebar background", (0, 160, 220, 480)),
    ("title", (230, 10, 330, 42)),
    ("toolbar group", (670, 2, 758, 50)),
    ("title bar edge", (222, 48, 758, 56)),
    ("empty state", (300, 200, 700, 360)),
    ("detail background", (222, 56, 758, 200)),
]
NOTES_BLUR = {"lights": 12.0, "sidebar toggle": 26.0, "sidebar rows": 10.0, "sidebar background": 1.0, "title": 11.0,
              "toolbar group": 16.0, "title bar edge": 1.0, "empty state": 5.0, "detail background": 1.0}

# With Reduce Transparency on the sidebar, the toolbar group and the title bar are solids: measured after the token change.
NOTES_RT = {**NOTES_BLUR, "toolbar group": 18.0}

SURFACES = [
    Surface(
        "notes-plain", "notes_sidebar", "csdtwin", NOTES,
        {"blur": NOTES_BLUR, "rt": NOTES_RT}, kit_args=["plain"], lights=(10, 10, 100, 42),
    ),
    Surface(
        "notes-hybrid", "notes_sidebar", "csdtwin", NOTES,
        {"blur": {**NOTES_BLUR, "lights": 1.0}, "rt": {**NOTES_RT, "lights": 1.0}}, kit_args=["hidden-title-bar"], live=True,
        lights=(10, 10, 100, 42),
    ),
    Surface(
        "glass-toolbar", "glass_toolbar", "glasstwin",
        [("toolbar group", (668, 2, 758, 50)), ("title bar veil", (0, 0, 660, 52)), ("rows", (0, 60, 760, 480))],
        {"blur": {"toolbar group": 100.0, "title bar veil": 100.0, "rows": 12.0},
         "rt": {"toolbar group": 76.0, "title bar veil": 20.0, "rows": 6.0}}, live=True,
        lights=(10, 5, 90, 35),
    ),
    Surface(
        "glass-container", "glass_container", "glasscontainer",
        [("regular pair", (30, 30, 130, 90)), ("clear pair", (30, 130, 130, 190)), ("rows", (200, 0, 460, 300))],
        {"blur": {"regular pair": 32.0, "clear pair": 30.0, "rows": 1.0},
         "rt": {"regular pair": 32.0, "clear pair": 30.0, "rows": 1.0}}, live=True,
    ),
    Surface(
        "popover", "popups", "matrixpopups",
        [("arrow", (212, 236, 268, 262)), ("body", (130, 262, 350, 380))],
        {"blur": {"arrow": 62.0, "body": 88.0}, "rt": {"arrow": 62.0, "body": 88.0}}, probe_args=["popover"], live=True,
    ),
    Surface(
        "sheet", "popups", "matrixpopups",
        [("sheet", (25, 28, 455, 308)), ("veil", (0, 310, 480, 428))],
        {"blur": {"sheet": 2.0, "veil": 2.0}, "rt": {"sheet": 2.0, "veil": 2.0}}, probe_args=["sheet"], kit_args=["sheet"],
    ),
]


def run(*command, **kwargs):
    return subprocess.run(command, check=True, capture_output=True, text=True, **kwargs).stdout


def system_reduces_transparency():
    script = "ObjC.import('AppKit'); $.NSWorkspace.sharedWorkspace.accessibilityDisplayShouldReduceTransparency"
    return run("osascript", "-l", "JavaScript", "-e", script).strip() == "true"


def window_of(pid):
    script = f"""
        ObjC.import('CoreGraphics');
        var all = ObjC.deepUnwrap(ObjC.castRefToObject(
            $.CGWindowListCopyWindowInfo($.kCGWindowListOptionOnScreenOnly, 0)));
        var mine = all.filter(function (w) {{ return w.kCGWindowOwnerPID == {pid} && w.kCGWindowLayer == 0; }});
        // The main window is the biggest: a popover or a sheet of the same process may be listed first.
        mine.sort(function (a, b) {{ return b.kCGWindowBounds.Width * b.kCGWindowBounds.Height - a.kCGWindowBounds.Width * a.kCGWindowBounds.Height; }});
        mine.length ? mine[0].kCGWindowNumber : 0"""
    try:
        return int(run("osascript", "-l", "JavaScript", "-e", script).strip())
    except (subprocess.CalledProcessError, ValueError):
        return 0


def bundle(work, executable, name):
    app = os.path.join(work, name + ".app")
    shutil.rmtree(app, ignore_errors=True)
    os.makedirs(os.path.join(app, "Contents", "MacOS"))
    shutil.copy(executable, os.path.join(app, "Contents", "MacOS", name))
    with open(os.path.join(app, "Contents", "Info.plist"), "w") as plist:
        plist.write(f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>{name}</string>
<key>CFBundleIdentifier</key><string>local.barrierkit.matrix.{name}</string>
<key>CFBundleName</key><string>{name}</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
""")
    subprocess.run(["codesign", "--force", "--sign", "-", app], capture_output=True)
    return app


def launch(work, executable, name, args, env):
    app = bundle(work, executable, name)
    command = ["open", "-n"]
    for key, value in env.items():
        command += ["--env", f"{key}={value}"]
    subprocess.run(command + [app, "--args"] + args, check=True)
    for _ in range(60):
        found = subprocess.run(["pgrep", "-n", "-f", f"{app}/Contents/MacOS/{name}"], capture_output=True, text=True).stdout.strip()
        pid = int(found) if found else None
        window = window_of(pid) if pid else 0
        if window:
            time.sleep(2.5)
            return pid, window
        time.sleep(0.25)
    raise RuntimeError(f"no window for {name}")


def grab(work, window, out):
    raw = os.path.join(work, "raw.png")
    subprocess.run(["screencapture", "-x", "-o", "-l", str(window), raw], check=True)
    subprocess.run(["sips", "-m", SRGB, raw, "--out", out], check=True, capture_output=True)


def stop(pid):
    subprocess.run(["kill", str(pid)], capture_output=True)
    time.sleep(0.5)


def active(image, lights):
    # A window that is not key draws its lights gray, and another session's probe can take the key window.
    x0, y0, x1, y1 = lights
    region = np.array(Image.open(image).convert("RGB"))[y0:y1, x0:x1].astype(int)
    return bool(((region[:, :, 0] > 200) & (region[:, :, 1] < 120) & (region[:, :, 2] < 120)).any())


def capture(work, executable, name, args, env, out, lights=None):
    for _ in range(6):
        pid, window = launch(work, executable, name, args, env)
        try:
            grab(work, window, out)
        finally:
            stop(pid)
        if lights is None or active(out, lights):
            return
    raise RuntimeError(f"{name} was never the key window")


def kit_config(work, scheme):
    directory = os.path.join(work, "cfg-" + scheme)
    os.makedirs(directory, exist_ok=True)
    with open(os.path.join(directory, "appearance.domain"), "w") as domain:
        domain.write(f'color_scheme = "{scheme}"\ntraffic_lights = "standard"\n')
    return directory


def percent_different(reference, actual, rect):
    x0, y0, x1, y1 = rect
    ref, act = reference[y0:y1, x0:x1].astype(int), actual[y0:y1, x0:x1].astype(int)
    counted = ref[:, :, 3] == 255
    worst = np.abs(ref[:, :, :3] - act[:, :, :3]).max(axis=2)
    return 100.0 * float(((worst > TOLERANCE) & counted).sum()) / max(int(counted.sum()), 1)


def report(name, cell, reference, actual, elements, worse, label=""):
    if reference.shape != actual.shape:
        # A window whose frame is fractional comes out a pixel or two short.
        height, width = (min(a, b) for a, b in zip(reference.shape[:2], actual.shape[:2]))
        print(f"{name:15} {cell:11} sizes {reference.shape[:2]} and {actual.shape[:2]}: compared at {height} x {width}")
        reference, actual = reference[:height, :width], actual[:height, :width]
    for element_name, rect, ceiling in elements:
        percent = percent_different(reference, actual, rect)
        print(f"{name:15} {cell:11} {label}{element_name:22} {percent:5.1f}%")
        if percent > ceiling:
            worse.append(f"{name} {cell} {label}{element_name}: {percent:.1f}% over {ceiling}%")


holding_lock = False


def take_lock():
    global holding_lock
    waiting = False
    while True:
        try:
            os.mkdir(LOCK)
            holding_lock = True
            with open(os.path.join(LOCK, "pid"), "w") as owner:
                owner.write(str(os.getpid()))
            return
        except FileExistsError:
            try:
                with open(os.path.join(LOCK, "pid")) as owner:
                    holder = owner.read().strip()
            except FileNotFoundError:
                holder = ""
            # An empty pid file means the owner is between mkdir and writing it: still held.
            if holder and not pid_alive(int(holder)):
                shutil.rmtree(LOCK, ignore_errors=True)
                continue
            if not waiting:
                print(f"waiting for {LOCK} held by PID {holder or 'unknown'}", file=sys.stderr)
                waiting = True
            time.sleep(2)


def pid_alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def release_lock():
    global holding_lock
    shutil.rmtree(LOCK)
    holding_lock = False


def live_capture(work, directory, chosen):
    """Before the flip: opens the kit's examples, captures them, waits until the system turns Reduce Transparency the other
    way (the user does it), captures them again with the same processes. Writes <surface>-before.png and -after.png."""
    os.makedirs(directory, exist_ok=True)
    running = []
    try:
        for surface in chosen:
            env = {"MALAYSIA_CONFIG_DIR": kit_config(work, "light")}
            pid, window = launch(work, os.path.join(EXAMPLES, surface.kit), surface.kit, surface.kit_args, env)
            running.append((surface, pid, window))
            grab(work, window, os.path.join(directory, f"{surface.name}-before.png"))
        start = system_reduces_transparency()
        print("live: captured, waiting for the system to turn Reduce Transparency " + ("off" if start else "on"), flush=True)
        # Other sessions use the screen while the user is at the settings: the lock is theirs until the flip.
        release_lock()
        for _ in range(1800):
            if system_reduces_transparency() != start:
                break
            time.sleep(1)
        else:
            sys.exit("live: Reduce Transparency never flipped")
        take_lock()
        time.sleep(4)
        for surface, _, window in running:
            grab(work, window, os.path.join(directory, f"{surface.name}-after.png"))
        print("live: captured after the flip", flush=True)
    finally:
        for _, pid, _ in running:
            stop(pid)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--transparency", choices=["blur", "rt"], required=True)
    parser.add_argument("--surface", action="append")
    parser.add_argument("--keep", help="directory that keeps the captures")
    parser.add_argument("--references", action="store_true",
                        help="with rt: also captures the SwiftUI side of the five twins into crates/bk/tests/reference")
    parser.add_argument("--live-capture", metavar="DIR", help="the live half, before and after the user flips Reduce Transparency (see live_capture)")
    parser.add_argument("--live-compare", metavar="DIR", help="with rt: compares a fresh kit capture with the live one of DIR")
    options = parser.parse_args()
    if not os.path.isdir(PROBES):
        sys.exit(f"appearance-matrix: {PROBES} not found; set BARRIERKIT_PROBES to the directory of the SwiftUI probes")

    if system_reduces_transparency() != (options.transparency == "rt"):
        sys.exit(f"the system's Reduce Transparency is {'on' if system_reduces_transparency() else 'off'}, "
                 f"not what --transparency {options.transparency} asks; this script never changes it")
    chosen = [s for s in SURFACES if not options.surface or s.name in options.surface]
    work = tempfile.mkdtemp(prefix="barrierkit-matrix-")
    take_lock()
    worse = []
    try:
        if options.live_capture:
            live_capture(work, options.live_capture, [s for s in chosen if s.live])
            return
        if options.references:
            for twin in ("notes", "tasks", "form", "controls", "standalone"):
                for scheme in ("light", "dark"):
                    out = os.path.join(ROOT, "crates", "bk", "tests", "reference", f"swiftui-{twin}-{scheme}-rt.png")
                    release_lock()
                    run(os.path.join(ROOT, "scripts", "twin-reference.sh"), twin, scheme, out)
                    take_lock()
                    print(f"reference {twin} {scheme}-rt captured")
        for surface in chosen:
            for scheme in ("light", "dark"):
                cell = f"{scheme}-{options.transparency}"
                base = os.path.join(work, f"{surface.name}-{cell}")
                probe_args = surface.probe_args + (["dark"] if scheme == "dark" else [])
                capture(work, os.path.join(PROBES, surface.probe), surface.probe, probe_args, {}, base + "-swift.png", surface.lights)
                kit_env = {"MALAYSIA_CONFIG_DIR": kit_config(work, scheme)}
                capture(work, os.path.join(EXAMPLES, surface.kit), surface.kit, surface.kit_args, kit_env, base + "-kit.png", surface.lights)
                reference = np.array(Image.open(base + "-swift.png").convert("RGBA"))
                actual = np.array(Image.open(base + "-kit.png").convert("RGBA"))
                ceilings = surface.ceilings[options.transparency]
                elements = [(name, rect, ceilings.get(name, 100.0)) for name, rect in surface.elements]
                report(surface.name, cell, reference, actual, elements, worse)
                if options.live_compare and scheme == "light" and surface.live:
                    live = np.array(Image.open(os.path.join(options.live_compare, f"{surface.name}-after.png")).convert("RGBA"))
                    report(surface.name, cell, actual, live, elements, worse, label="live against cold ")
                if options.keep:
                    os.makedirs(options.keep, exist_ok=True)
                    for kind in ("swift", "kit"):
                        shutil.copy(f"{base}-{kind}.png", options.keep)
    finally:
        if holding_lock:
            release_lock()
        shutil.rmtree(work, ignore_errors=True)
    if worse:
        sys.exit("over the ceiling: " + "; ".join(worse))


if __name__ == "__main__":
    main()
