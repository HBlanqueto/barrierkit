#!/usr/bin/env python3
import argparse
import collections
import json
import os
import sys

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CAPTURES = os.path.expanduser(os.environ.get("APPKIT_CAPTURES", "~/src/captures"))
KIT = os.environ.get("APPKIT_OUT", os.path.join(ROOT, "target", "appkit"))
SCALE = 4
MARGIN = 6
CONTROLS = [
    "push-small", "push-regular", "push-large", "default-regular", "push-disabled",
    "check-off-regular", "check-on-regular", "radio-off-regular", "radio-on-regular",
    "switch-off-regular", "switch-on-regular", "switch-on-disabled", "slider-regular",
    "segmented-regular", "textfield-regular", "search", "popup-regular",
    "stepper-regular", "progress-bar", "ring-regular", "datepicker", "level", "combo",
    "separator", "help", "box", "search-filled", "textfield-filled",
    "disclosure", "check-mixed", "check-on-disabled", "circular", "slider-ticks", "colorwell", "colorwell-blue",
    "default-disabled", "check-off-disabled", "radio-off-disabled", "radio-on-disabled",
    "switch-off-disabled", "slider-disabled", "segmented-disabled",
    "popup-disabled", "stepper-disabled", "combo-disabled",
]
GLYPH_WIDTH = {name: 18 for name in ("check-off-regular", "check-on-regular", "radio-off-regular", "radio-on-regular")}
APPKIT = {"light": "aqua", "dark": "darkAqua"}
CONTRAST = {"light": "hc-aquaHC", "dark": "hc-darkHC"}


def capture(family, mode, backdrop, tag):
    name = (CONTRAST if tag.endswith("hc") else APPKIT)[mode]
    return os.path.join(CAPTURES, family, f"{name}-{backdrop}@4x.png")


def kit_image(mode, backdrop, tag):
    suffix = f"-{tag}" if tag else ""
    return os.path.join(KIT, f"kit{suffix}-{mode}-{backdrop}.png")


def rgb(path):
    return np.asarray(Image.open(path).convert("RGB")).astype(int)


def items(family):
    with open(os.path.join(CAPTURES, family, "aqua-white.json")) as handle:
        data = json.load(handle)
    return data["contentH"], {item["name"]: item for item in data["items"]}


def region(name, content_height, found):
    item = found[name]
    width = GLYPH_WIDTH.get(name, item["w"])
    return item["x"], content_height - item["y"] - item["h"], width, item["h"]


def crop(image, box, margin=MARGIN):
    x, y, w, h = box
    return image[(y - margin) * SCALE:(y + h + margin) * SCALE, (x - margin) * SCALE:(x + w + margin) * SCALE]


def bounds(pixels, backdrop):
    mask = abs(pixels - backdrop).max(-1) > 8
    ys, xs = np.nonzero(mask)
    if not len(ys):
        return None
    return xs.min() / SCALE, ys.min() / SCALE, (xs.max() + 1 - xs.min()) / SCALE, (ys.max() + 1 - ys.min()) / SCALE


def dominant(pixels, backdrop):
    painted = pixels[abs(pixels - backdrop).max(-1) > 8]
    if not len(painted):
        return "-"
    return "#%02x%02x%02x" % collections.Counter(map(tuple, painted)).most_common(1)[0][0]


def layout(args):
    with open(args.probe_json) as handle:
        data = json.load(handle)
    height = data["contentH"]
    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, "w") as out:
        for item in data["items"]:
            out.write(f"{item['name']}\t{item['x']}\t{height - item['y'] - item['h']}\t{item['w']}\t{item['h']}\n")


def controls(args):
    height, found = items("controls")
    for mode in ("light", "dark"):
        for backdrop, level in (("white", 255), ("black", 0)):
            appkit = rgb(capture("controls", mode, backdrop, args.tag))
            kit = rgb(kit_image(mode, backdrop, args.tag))
            color = np.array([level] * 3)
            print(f"\n=== {mode} on {backdrop}")
            print(f"{'control':20} {'%diff':>6} {'AppKit x y w h':>24} {'kit x y w h':>24} {'AppKit':>8} {'kit':>8}")
            for name in CONTROLS:
                box = region(name, height, found)
                a, k = crop(appkit, box), crop(kit, box)
                diff = (abs(a - k) > 3).any(-1).mean() * 100
                ba, bk = bounds(a, color), bounds(k, color)
                show = lambda b: "%5.1f %5.1f %5.1f %5.1f" % b if b else "none"
                print(f"{name:20} {diff:6.1f} {show(ba):>24} {show(bk):>24} {dominant(a, color):>8} {dominant(k, color):>8}")


PRESSED = [
    "push-pressed", "default-pressed", "check-off-pressed", "check-on-pressed",
    "radio-off-pressed", "radio-on-pressed", "popup-pressed",
]


def pressed(args):
    height, found = items("controls")
    suffix = f"-{args.tag}" if args.tag else ""
    for mode in ("light", "dark"):
        for backdrop, level in (("white", 255), ("black", 0)):
            appkit = rgb(capture("controls", mode, backdrop, args.tag))
            color = np.array([level] * 3)
            print(f"\n=== pressed {mode} on {backdrop}")
            print(f"{'control':20} {'%diff':>6} {'AppKit x y w h':>24} {'kit x y w h':>24} {'AppKit':>8} {'kit':>8}")
            for name in PRESSED:
                kit = rgb(os.path.join(KIT, f"kit{suffix}-pressed-{name}-{mode}-{backdrop}.png"))
                box = region(name, height, found)
                a, k = crop(appkit, box), crop(kit, box)
                diff = (abs(a - k) > 3).any(-1).mean() * 100
                ba, bk = bounds(a, color), bounds(k, color)
                show = lambda b: "%5.1f %5.1f %5.1f %5.1f" % b if b else "none"
                print(f"{name:20} {diff:6.1f} {show(ba):>24} {show(bk):>24} {dominant(a, color):>8} {dominant(k, color):>8}")


def sheet(args):
    height, found = items(args.family)
    appkit = Image.open(capture(args.family, args.mode, args.backdrop, args.tag)).convert("RGB")
    kit = Image.open(kit_image(args.mode, args.backdrop, args.tag)).convert("RGB")
    tiles = []
    for name in args.names.split(","):
        x, y, w, h = region(name, height, found)
        w = min(w, 200)
        box = ((x - MARGIN) * SCALE, (y - MARGIN) * SCALE, (x + w + MARGIN) * SCALE, (y + h + MARGIN) * SCALE)
        a, k = appkit.crop(box), kit.crop(box)
        tile = Image.new("RGB", (a.width * 2 + 8, a.height), (255, 0, 255))
        tile.paste(a, (0, 0))
        tile.paste(k, (a.width + 8, 0))
        tiles.append(tile)
    out = Image.new("RGB", (max(t.width for t in tiles), sum(t.height + 6 for t in tiles)), (128, 128, 128))
    top = 0
    for tile in tiles:
        out.paste(tile, (0, top))
        top += tile.height + 6
    out = out.resize((out.width // 2, out.height // 2), Image.LANCZOS)
    os.makedirs(KIT, exist_ok=True)
    path = os.path.join(KIT, f"sheet-{args.tag or 'controls'}-{args.mode}-{args.backdrop}.png")
    out.save(path)
    print(path)


def table(args):
    height, found = items("views")
    box = region("table", height, found)
    level = 255 if args.mode == "light" else 0x1E
    appkit = crop(rgb(capture("views", args.mode, "white", "")), box, 0)
    kit = crop(rgb(kit_image(args.mode, "white", "views")), box, 0)

    def ink(image, top, bottom, left, right, light_text=False):
        sub = image[int(top * SCALE):int(bottom * SCALE), int(left * SCALE):int(right * SCALE)]
        if light_text:
            mask = sub.min(-1) > 200
        elif args.mode == "light":
            mask = sub.max(-1) < 150
        else:
            mask = sub.min(-1) > 150
        ys, xs = np.nonzero(mask.any(1)), np.nonzero(mask.any(0))
        if not len(ys[0]):
            return "none"
        return "%6.2f %6.2f %6.2f %6.2f" % (left + xs[0].min() / SCALE, top + ys[0].min() / SCALE, left + xs[0].max() / SCALE + .25, top + ys[0].max() / SCALE + .25)

    print("                  AppKit (x0 y0 x1 y1)    |  kit")
    print("header Name     ", ink(appkit, 0, 26, 12, 150), "|", ink(kit, 0, 26, 12, 150))
    print("header Kind     ", ink(appkit, 0, 26, 162, 310), "|", ink(kit, 0, 26, 162, 310))
    for row in range(6):
        top = 28 + 5 + 24 * row
        for column, (left, right) in enumerate(((12, 150), (162, 300)), 1):
            selected = row == 1
            print(f"row{row} col{column}      ", ink(appkit, top, top + 24, left, right, selected), "|", ink(kit, top, top + 24, left, right, selected))

    def separators(image):
        found_columns = np.nonzero((abs(image[8 * SCALE + 2, :, 0] - level) > 3) & (abs(image[20 * SCALE + 2, :, 0] - level) > 3))[0]
        return sorted({int(c // SCALE) for c in found_columns})

    print("separators AppKit:", separators(appkit), " kit:", separators(kit))


def main():
    parser = argparse.ArgumentParser(description="Compare a kit scene with AppKit captures.")
    commands = parser.add_subparsers(dest="command", required=True)
    one = commands.add_parser("layout", help="write the scene layout from a probe JSON")
    one.add_argument("probe_json")
    one.add_argument("output")
    one.set_defaults(run=layout)
    two = commands.add_parser("controls", help="print the per-control difference table")
    two.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    two.set_defaults(run=controls)
    five = commands.add_parser("pressed", help="print the per-control difference table for pressed controls")
    five.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    five.set_defaults(run=pressed)
    three = commands.add_parser("sheet", help="write a side-by-side image of named items")
    three.add_argument("family", choices=("controls", "views"))
    three.add_argument("mode", choices=("light", "dark"))
    three.add_argument("backdrop", choices=("white", "black"))
    three.add_argument("names", help="comma separated item names")
    three.add_argument("--tag", default="", help="views, views-hc or hc")
    three.set_defaults(run=sheet)
    four = commands.add_parser("table", help="print the table header, text and separator positions")
    four.add_argument("mode", choices=("light", "dark"))
    four.set_defaults(run=table)
    args = parser.parse_args()
    args.run(args)


if __name__ == "__main__":
    sys.exit(main())
