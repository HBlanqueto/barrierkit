#!/usr/bin/env python3
import argparse
import collections
import glob
import io
import json
import os
import sys

import numpy as np
from PIL import Image, ImageCms

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CAPTURES = os.path.expanduser(os.environ.get("APPKIT_CAPTURES", "~/src/captures"))
KIT = os.environ.get("APPKIT_OUT", os.path.join(ROOT, "target", "appkit"))
SCALE = int(os.environ.get("APPKIT_SCALE", "4"))
MARGIN = 6
CONTROLS = [
    "push-small", "push-regular", "push-large", "default-regular", "push-disabled",
    "check-off-regular", "check-on-regular", "radio-off-regular", "radio-on-regular",
    "switch-off-regular", "switch-on-regular", "switch-on-disabled", "slider-regular",
    "segmented-regular", "textfield-regular", "search", "popup-regular",
    "stepper-regular", "progress-bar", "ring-regular", "datepicker", "level", "combo",
    "separator", "help", "box", "search-filled", "textfield-filled",
    "slider-circular", "pulldown", "disclosure", "calendar", "level-warn", "level-crit", "discrete-5", "discrete-10", "rating-3",
    "relevancy-3", "secure", "token", "check-mixed", "check-on-disabled", "circular", "slider-ticks", "colorwell", "colorwell-blue",
    "default-disabled", "check-off-disabled", "radio-off-disabled", "radio-on-disabled",
    "switch-off-disabled", "slider-disabled", "segmented-disabled",
    "popup-disabled", "stepper-disabled", "combo-disabled",
]
TEXTS = [
    "tf-empty", "tf-filled", "tf-square", "tf-rounded", "secure-empty", "token-empty", "search", "search-text", "combo", "combo-empty",
    "textview-system", "datepicker", "datetime", "time", "calendar", "path-standard", "imagewell-gray-empty",
    "tf-mini", "tf-small", "tf-regular", "tf-large", "tf-xl", "tf-disabled-filled", "tf-plain",
    "search-mini", "search-small", "search-regular", "search-large", "search-xl",
    "search-text-mini", "search-text-small", "search-text-regular", "search-text-large", "search-text-xl", "search-disabled-text",
    "combo-mini", "combo-small", "combo-regular", "combo-large", "combo-xl",
    "datepicker-mini", "datepicker-small", "datepicker-regular", "datepicker-large", "datepicker-xl", "date-field", "datepicker-disabled",
    "well-default", "well-default-disabled", "well-minimal", "well-minimal-disabled", "well-expanded", "well-expanded-disabled",
]
SIZE_NAMES = ("mini", "small", "regular", "large", "xl")
BUTTONS = [
    f"{kind}-{size}" for kind in (
        "push", "default", "popup", "pulldown", "segmented", "check-off", "check-on", "check-mixed",
        "radio-off", "radio-on", "switch-off", "switch-on", "slider", "stepper", "push-disabled", "popup-disabled",
        "check-off-disabled", "check-on-disabled", "radio-off-disabled", "radio-on-disabled", "switch-on-disabled",
        "slider-disabled", "stepper-disabled",
    ) for size in SIZE_NAMES
]
BUTTONS += [
    "bezel-" + name for name in (
        "push", "flexible-push", "round-rect", "recessed", "accessory-bar", "accessory-bar-action", "small-square",
        "textured-rounded", "toolbar", "textured-square", "inline", "push-disclosure", "rounded-disclosure", "help", "circular",
    )
]
BUTTONS += ["segmented-disabled"]
BUTTONS += [
    f"bezel-{name}-disabled" for name in ("push", "small-square", "textured-rounded", "textured-square", "inline", "push-disclosure")
]
ACCENT_CONTROLS = [
    "default-regular", "default-disabled", "check-on-regular", "check-mixed", "check-on-disabled", "radio-on-regular",
    "radio-on-disabled", "switch-on-regular", "switch-on-disabled", "slider-regular", "slider-ticks", "slider-circular",
    "slider-disabled", "segmented-regular", "segmented-disabled", "stepper-regular", "popup-regular", "pulldown",
    "progress-bar", "ring-regular", "token", "datepicker", "combo", "search-filled",
]
ACCENTS = ["multicolor", "blue", "purple", "pink", "red", "orange", "yellow", "green", "graphite"]
ACCENT = os.environ.get("APPKIT_ACCENT", "blue")
SWIFTUI = [
    f"{kind}-{size}" for kind in (
        "bordered", "prominent", "switch-off", "switch-on", "checkbox-off", "checkbox-on", "togglebutton-off", "togglebutton-on",
        "menu", "radio", "slider", "stepper",
    ) for size in SIZE_NAMES
] + [
    "automatic", "plain", "link", "destructive", "cancel", "bordered-disabled", "prominent-disabled", "bordered-capsule",
    "bordered-circle", "bordered-rounded", "inline", "switch-on-disabled",
]
FAMILIES = {"controls": CONTROLS, "texts": TEXTS, "buttons": BUTTONS, "accents": ACCENT_CONTROLS, "swiftui": SWIFTUI}
CAPTURE_DIRS = {"swiftui": "swiftuibuttons"}
GLYPH_WIDTH = {name: 18 for name in ("check-off-regular", "check-on-regular", "radio-off-regular", "radio-on-regular")}
GLYPH_WIDTH.update({
    f"{kind}{state}-{size}": box + 2
    for kind in ("check-off", "check-on", "check-mixed", "radio-off", "radio-on")
    for state in ("", "-pressed", "-disabled")
    for size, box in zip(SIZE_NAMES, (12, 14, 16, 18, 18))
})
APPKIT = {"light": "aqua", "dark": "darkAqua"}
CONTRAST = {"light": "hc-aquaHC", "dark": "hc-darkHC"}


def family_dir(family):
    return os.path.join("accents", "controls", ACCENT) if family == "accents" else CAPTURE_DIRS.get(family, family)


def capture(family, mode, backdrop, tag):
    name = (CONTRAST if tag.endswith("hc") else APPKIT)[mode]
    if family == "accents":
        name = name.removeprefix("hc-")
    return os.path.join(CAPTURES, family_dir(family), f"{name}-{backdrop}@{SCALE}x.png")


def kit_image(mode, backdrop, tag):
    suffix = f"-{tag}" if tag else ""
    return os.path.join(KIT, f"kit{suffix}-{mode}-{backdrop}.png")


def rgb(path):
    return np.asarray(Image.open(path).convert("RGB")).astype(int)


def items(family):
    with open(os.path.join(CAPTURES, family_dir(family), "aqua-white.json")) as handle:
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


def label_box(pixels, backdrop, name):
    """The box of the label ink inside a bezel (the groups of ink columns, less the indicator of a pop-up), in pixels."""
    painted = abs(pixels - backdrop).max(-1) > 8
    ys, xs = np.nonzero(painted)
    if not len(ys):
        return None
    left, right, top, bottom = xs.min() + 4 * SCALE, xs.max() + 1 - 4 * SCALE, ys.min() + 3 * SCALE, ys.max() + 1 - 3 * SCALE
    inside = pixels[top:bottom, left:right]
    fill = pixels[(top + bottom) // 2, left]
    ink = abs(inside - fill).max(-1) > 60
    columns = np.nonzero(ink.any(0))[0]
    if not len(columns):
        return None
    groups, start = [], columns[0]
    for before, after in zip(columns, columns[1:]):
        if after - before > 3 * SCALE:
            groups.append((start, before))
            start = after
    groups.append((start, columns[-1]))
    if name.startswith(("popup", "pulldown")) and len(groups) > 1:
        groups = groups[:-1]
    rows = np.nonzero(ink[:, groups[0][0]:groups[-1][1] + 1].any(1))[0]
    return left + groups[0][0] - SCALE, left + groups[-1][1] + 1 + SCALE, top + rows.min() - SCALE, top + rows.max() + 1 + SCALE


def bezel_diff(a, k, backdrop, name):
    """Share of pixels that differ outside the label boxes of both sides, as a percentage of the crop."""
    keep = np.ones(a.shape[:2], bool)
    for pixels in (a, k):
        box = label_box(pixels, backdrop, name)
        if box:
            keep[box[2]:box[3], box[0]:box[1]] = False
    return ((abs(a - k) > 3).any(-1) & keep).mean() * 100


def layout(args):
    with open(args.probe_json) as handle:
        data = json.load(handle)
    height = data["contentH"]
    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, "w") as out:
        if "contentW" in data:
            out.write(f"# {int(data['contentW'])}x{int(height)}\n")
        for item in data["items"]:
            out.write(f"{item['name']}\t{item['x']}\t{height - item['y'] - item['h']}\t{item['w']}\t{item['h']}\n")


def controls(args):
    height, found = items(args.family)
    kit_tag = args.tag if args.family == "controls" else "-".join(filter(None, (args.family, args.tag)))
    if args.family == "accents":
        kit_tag = "-".join(filter(None, ("accents", ACCENT, args.tag)))
    for mode in ("light", "dark"):
        for backdrop, level in ((("white", 255),) if mode == "light" else (("black", 0),)) if args.family == "accents" else (("white", 255), ("black", 0)):
            appkit = rgb(capture(args.family, mode, backdrop, args.tag))
            kit = rgb(kit_image(mode, backdrop, kit_tag))
            color = np.array([level] * 3)
            print(f"\n=== {mode} on {backdrop}")
            print(f"{'control':20} {'%diff':>6} {'%bezel':>6} {'AppKit x y w h':>24} {'kit x y w h':>24} {'AppKit':>8} {'kit':>8}")
            for name in FAMILIES[args.family]:
                box = region(name, height, found)
                a, k = crop(appkit, box), crop(kit, box)
                diff = (abs(a - k) > 3).any(-1).mean() * 100
                ba, bk = bounds(a, color), bounds(k, color)
                show = lambda b: "%5.1f %5.1f %5.1f %5.1f" % b if b else "none"
                print(f"{name:20} {diff:6.1f} {bezel_diff(a, k, color, name):6.1f} {show(ba):>24} {show(bk):>24} {dominant(a, color):>8} {dominant(k, color):>8}")


def picker(args):
    """The accent picker of System Settings (a 1x capture of its Appearance pane) against the kit's `ColorWell::accents`."""
    appkit = rgb(os.path.join(CAPTURES, "accents", f"settings-{args.mode}-srgb.png"))[435:495, 500:850]
    kit = rgb(os.path.join(KIT, f"kit-picker-{args.mode}.png"))
    diff = (abs(appkit - kit) > 3).any(-1)
    background = appkit[0, 0]
    print(f"pixels that differ: {diff.mean() * 100:.1f}%")
    for label, row in (("circle row", 26), ("label row", 48)):
        print(f"{label}: AppKit {bounds(appkit[row:row + 1], background)} kit {bounds(kit[row:row + 1], background)}")
    for index, x in enumerate(range(35, 340, 36)):
        print(f"swatch {index}: AppKit #%02x%02x%02x kit #%02x%02x%02x" % (*appkit[26, x], *kit[26, x]))
    ink = lambda image: bounds(image[40:60], background)
    print(f"ring and name box (x y w h): AppKit {bounds(appkit, background)} kit {bounds(kit, background)}; name {ink(appkit)} | {ink(kit)}")


def fill_mismatch(a, k, backdrop):
    """The share of AppKit's dominant fill that the kit does not draw within 3 levels, as a percentage."""
    painted = a[abs(a - backdrop).max(-1) > 8]
    if not len(painted):
        return 0.0
    fill = np.array(collections.Counter(map(tuple, painted)).most_common(1)[0][0])
    mask = abs(a - fill).max(-1) <= 3
    return (mask & (abs(a - k).max(-1) > 3)).sum() / mask.sum() * 100


def accents(args):
    """Rows are controls, columns accents. The first table is the share of each control's dominant fill that the kit
    misses by more than 3 levels (the accent), the second the share of all pixels that differ (text and shape too).
    Light is on white and dark on black."""
    global ACCENT
    for mode in (args.mode,) if args.mode else ("light", "dark"):
        title = f"{mode}{' with Increase Contrast' if args.tag else ''}"
        fills, differs = collections.defaultdict(list), collections.defaultdict(list)
        for accent in ACCENTS:
            ACCENT = accent
            height, found = items("accents")
            backdrop, level = ("white", 255) if mode == "light" else ("black", 0)
            appkit = rgb(capture("accents", mode, backdrop, args.tag))
            kit = rgb(kit_image(mode, backdrop, "-".join(filter(None, ("accents", accent, args.tag)))))
            color = np.array([level] * 3)
            for name in ACCENT_CONTROLS:
                a, k = crop(appkit, region(name, height, found)), crop(kit, region(name, height, found))
                fills[name].append(fill_mismatch(a, k, color))
                differs[name].append((abs(a - k) > 3).any(-1).mean() * 100)
        for label, table in (("% of the dominant fill the kit misses", fills), ("% of pixels that differ", differs)):
            print(f"\n=== {title}: {label}")
            print(f"{'control':20}" + "".join(f"{name[:9]:>10}" for name in ACCENTS))
            for name in ACCENT_CONTROLS:
                print(f"{name:20}" + "".join(f"{value:10.1f}" for value in table[name]))
            print(f"{'worst':20}" + "".join(f"{max(table[name][i] for name in ACCENT_CONTROLS):10.1f}" for i in range(len(ACCENTS))))


PRESSED = [
    "push-pressed", "default-pressed", "check-off-pressed", "check-on-pressed",
    "radio-off-pressed", "radio-on-pressed", "popup-pressed", "calendar-focused",
]


BUTTONS_PRESSED = [
    f"{kind}-pressed-{size}" for kind in ("push", "default", "popup", "check-off", "check-on", "radio-off", "radio-on")
    for size in SIZE_NAMES
] + [
    f"bezel-{name}-pressed" for name in ("push", "small-square", "textured-rounded", "textured-square", "inline", "push-disclosure")
]
PRESSED_FAMILIES = {"controls": PRESSED, "buttons": BUTTONS_PRESSED, "accents": ["default-pressed", "check-on-pressed", "radio-on-pressed", "popup-pressed"]}


def pressed(args):
    height, found = items(args.family)
    suffix = "-".join(filter(None, ("" if args.family == "controls" else args.family, ACCENT if args.family == "accents" else "", args.tag)))
    suffix = f"-{suffix}" if suffix else ""
    for mode in ("light", "dark"):
        for backdrop, level in ((("white", 255),) if mode == "light" else (("black", 0),)) if args.family == "accents" else (("white", 255), ("black", 0)):
            appkit = rgb(capture(args.family, mode, backdrop, args.tag))
            color = np.array([level] * 3)
            print(f"\n=== pressed {mode} on {backdrop}")
            print(f"{'control':20} {'%diff':>6} {'AppKit x y w h':>24} {'kit x y w h':>24} {'AppKit':>8} {'kit':>8}")
            for name in PRESSED_FAMILIES[args.family]:
                kit = rgb(os.path.join(KIT, f"kit{suffix}-pressed-{name}-{mode}-{backdrop}.png"))
                box = region(name, height, found)
                a, k = crop(appkit, box), crop(kit, box)
                diff = (abs(a - k) > 3).any(-1).mean() * 100
                ba, bk = bounds(a, color), bounds(k, color)
                show = lambda b: "%5.1f %5.1f %5.1f %5.1f" % b if b else "none"
                print(f"{name:20} {diff:6.1f} {show(ba):>24} {show(bk):>24} {dominant(a, color):>8} {dominant(k, color):>8}")


FOCUSED = [
    "tf-empty", "tf-filled", "search", "search-text", "combo", "secure", "token", "datepicker", "textview-system",
]


def focused(args):
    height, found = items("texts")
    kit_tag = "-".join(filter(None, ("texts", args.tag)))
    suffix = f"-{kit_tag}"
    for mode in ("light", "dark"):
        for backdrop, level in (("white", 255), ("black", 0)):
            name_prefix = (CONTRAST if args.tag.endswith("hc") else APPKIT)[mode]
            color = np.array([level] * 3)
            print(f"\n=== focused {mode} on {backdrop}")
            print(f"{'control':20} {'%diff':>6} {'AppKit x y w h':>24} {'kit x y w h':>24} {'AppKit':>8} {'kit':>8}")
            for name in FOCUSED:
                appkit = rgb(os.path.join(CAPTURES, "texts", f"{name_prefix}-{backdrop}-focus-{name}@{SCALE}x.png"))
                kit = rgb(os.path.join(KIT, f"kit{suffix}-focus-{name}-{mode}-{backdrop}.png"))
                box = region(name, height, found)
                a, k = crop(appkit, box, 10), crop(kit, box, 10)
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


TOOLBAR_GLYPHS = ["chevron.left", "chevron.right", "plus", "sidebar.left", "gearshape", "square.and.arrow.up", "ellipsis", "magnifyingglass", "trash", "arrow.left"]
TOOLBAR_KIT = ["chevron.left", "chevron.right", "plus", "sidebar.left", "ellipsis", "magnifyingglass"]
TOOLBAR_CELL = 44


def toolbar_snapshots(prefix):
    base = os.path.join(CAPTURES, "navbuttons", prefix)
    with open(base + ".json") as handle:
        tree = json.load(handle)["tree"]
    buttons = sorted((n for n in tree if "Hosting" in n["class"] and "AppKitButton" in n["class"]), key=lambda n: n["left"])
    result = []
    for node in buttons:
        path = glob.glob(f"{base}-{node['index']}-*.png")[0]
        result.append(np.array(Image.open(path).convert("RGBA")).astype(float)[..., 3] / 255)
    return result


def ink_box(mask):
    ys, xs = np.nonzero(mask > 0.1)
    return xs.min() / 4, (xs.max() + 1) / 4, ys.min() / 4, (ys.max() + 1) / 4


def toolbar(args):
    dark = args.mode == "dark"
    kit = np.array(Image.open(os.path.join(KIT, f"kit-toolbar-{args.mode}.png")).convert("RGB")).astype(float).mean(-1) / 255

    def darkness(gray, background):
        return np.clip(gray - background if dark else background - gray, 0, 1) / 0.847

    print("glyph              AppKit ink w x h   kit ink w x h   AppKit mass  kit mass  diff%")
    alpha = toolbar_snapshots(f"glyphs-{args.mode}")
    for slot, name in enumerate(TOOLBAR_KIT):
        a = alpha[TOOLBAR_GLYPHS.index(name)]
        cell = kit[16:16 + 144, (4 + TOOLBAR_CELL * slot) * 4:(4 + TOOLBAR_CELL * slot) * 4 + 144]
        k = darkness(cell, 0.0 if dark else 1.0)
        ba, bk = ink_box(a / a.max()), ink_box(k / k.max())
        drawn = a if dark else 1 - a
        diff = (np.abs(drawn - cell) * 255 > 3).mean() * 100
        print(f"{name:18s} {ba[1] - ba[0]:6.2f} x {ba[3] - ba[2]:5.2f}  {bk[1] - bk[0]:6.2f} x {bk[3] - bk[2]:5.2f}  {a.sum() / 16 / .847:11.2f} {k.sum() / 16:9.2f} {diff:6.1f}")

    for bar, prefix in (("unified", "pair"), ("compact", "pair-compact")):
        pair_bar(args.mode, bar, f"{prefix}-{args.mode}")


def pair_bar(mode, bar, prefix):
    dark = mode == "dark"
    base = os.path.join(CAPTURES, "navbuttons", prefix)
    with open(base + ".json") as handle:
        tree = json.load(handle)["tree"]
    buttons = sorted((n for n in tree if "Hosting" in n["class"] and "AppKitButton" in n["class"]), key=lambda n: n["left"])
    first = buttons[0]["left"]
    width = round(buttons[-1]["left"] + buttons[-1]["width"] - first)
    height = round(buttons[0]["height"])
    canvas = np.zeros((height * 4, width * 4))
    for node in buttons:
        part = np.array(Image.open(glob.glob(f"{base}-{node['index']}-*.png")[0]).convert("RGBA")).astype(float)[..., 3] / 255
        x0 = round((node["left"] - first) * 4)
        canvas[:part.shape[0], x0:x0 + part.shape[1]] = part
    kit_base = os.path.join(KIT, f"kit-bar-{bar}-{mode}")
    with open(kit_base + ".json") as handle:
        box = json.load(handle)
    scale = 4
    image = np.array(Image.open(kit_base + ".png").convert("RGB")).astype(float).mean(-1) / 255
    left, top = round(box["x"] * scale), round(box["y"] * scale)
    pair = image[top:top + height * scale, left:left + width * scale]
    background = np.median(pair[height * 2:height * 4, width * 2:width * 4 - 8])
    ink = np.clip(pair - background if dark else background - pair, 0, 1) / 0.847
    inner = slice(height, height * 3)
    print(f"\n{bar} navigation pair {width} x {height} (kit {box['w']:g} x {box['h']:g}), ink x0 x1 y0 y1 (AppKit | kit)")
    halves = (("back", slice(16, width * 2)), ("forward", slice(width * 2, width * 4 - 16)))
    for label, columns in halves:
        ba = ink_box(canvas[inner, columns] / canvas.max())
        bk = ink_box(ink[inner, columns] / max(ink[inner, columns].max(), 1e-6))
        origin = columns.start / 4

        def fmt(item):
            return "%6.2f %6.2f %6.2f %6.2f" % (item[0] + origin, item[1] + origin, item[2] + height / 4, item[3] + height / 4)

        print(f"{label:8s} {fmt(ba)} | {fmt(bk)}")


def sidebar(args):
    with open(os.path.join(CAPTURES, "sidebar", f"swiftui-{args.mode}.json")) as handle:
        tree = json.load(handle)["tree"]
    appkit = rgb(os.path.join(CAPTURES, "sidebar", f"swiftui-{args.mode}-srgb.png"))
    kit = rgb(os.path.join(KIT, f"kit-sidebar-{args.mode}.png"))
    hexa = lambda c: "#%02x%02x%02x" % tuple(c)
    box = lambda b: "%6.1f %6.1f %6.1f %6.1f" % b if b else "none"

    def ink(image, x0, x1, y0, y1, background, threshold=60):
        mask = abs(image[y0:y1, x0:x1] - background).max(-1) > threshold
        ys, xs = np.nonzero(mask)
        if not len(ys):
            return None
        return x0 + xs.min(), x0 + xs.max() + 1, y0 + ys.min(), y0 + ys.max() + 1

    print(f"{'':22} {'AppKit':>26} | kit")
    for name, (x, y) in {"sidebar": (100, 540), "content": (500, 300), "hairline": (500, 52), "selection": (200, 119)}.items():
        print(f"{name + ' color':22} {hexa(appkit[y, x]):>26} | {hexa(kit[y, x])}")
    edge = lambda image: next(x for x in range(1, 900) if abs(image[300, x] - image[300, x - 1]).max() > 3)
    print(f"{'edge x':22} {edge(appkit):>26} | {edge(kit)}")
    print("x0 x1 y0 y1 of the ink per row (AppKit | kit); the selected row is its pill")
    for top, height in [(int(n["top"]), int(n["height"])) for n in tree if n["class"] == "ListTableRowView"]:
        found = [ink(image, 4, 216, top, top + height, image[top + height // 2, 214], 40 if top == 103 else 60) for image in (appkit, kit)]
        print(f"row y={top:3d} h={height:2d}  {box(found[0])} | {box(found[1])}")
    print("traffic lights, toggle, title and plus (AppKit | kit)")
    for label, region, threshold in (("lights", (4, 90, 4, 44), 40), ("toggle", (150, 217, 2, 50), 8), ("title", (222, 700, 4, 50), 40), ("plus", (840, 899, 4, 50), 40)):
        column = region[0] if label == "lights" else 214 if label == "toggle" else 500
        found = [ink(image, *region, image[30, column], threshold) for image in (appkit, kit)]
        print(f"{label:8} {box(found[0])} | {box(found[1])}")


LISTS = [
    "list-inset", "list-striped", "list-plain", "list-bordered", "table", "outline-group",
    "form-grouped", "form-columns", "groupbox", "disclosure",
]
LISTS_FOCUSABLE = ["list-inset", "list-plain", "list-bordered", "table", "outline-group"]


def lists(args):
    look = (CONTRAST if args.tag == "hc" else APPKIT)[args.mode]
    tag = f"{args.family}-hc" if args.tag == "hc" else args.family
    with open(os.path.join(CAPTURES, args.family, f"{look}-none.json")) as handle:
        data = json.load(handle)
    height, found = data["contentH"], {item["name"]: item for item in data["items"]}
    print(f"{'item':16} {'focus':>5} {'%diff':>6} {'AppKit x y w h':>24} {'kit x y w h':>24}")
    for name in args.names.split(",") if args.names else (LISTS if args.family == "lists" else CONTAINERS):
        box = region(name, height, found)
        for focused in (False, True) if name in LISTS_FOCUSABLE and args.family == "lists" else (False,):
            appkit = rgb(os.path.join(CAPTURES, args.family, f"{look}-{name if focused else 'none'}@{SCALE}x.png"))
            focus = f"focus-{name}-" if focused else ""
            kit_path = os.path.join(KIT, f"kit-{tag}-{focus}{args.mode}-white.png")
            if not os.path.exists(kit_path):
                continue
            kit = rgb(kit_path)
            whole = tuple(int(round(v)) for v in box)
            a, k = crop(appkit, whole, 0), crop(kit, whole, 0)
            diff = (abs(a - k) > 3).any(-1).mean() * 100
            backdrop = a[0, 0]
            ba, bk = bounds(a, backdrop), bounds(k, k[0, 0])
            show = lambda b: "%5.1f %5.1f %5.1f %5.1f" % b if b else "none"
            print(f"{name:16} {'yes' if focused else 'no':>5} {diff:6.1f} {show(ba):>24} {show(bk):>24}")
            if args.sheet:
                tile = Image.new("RGB", (a.shape[1] * 3 + 16, a.shape[0]), (255, 0, 255))
                tile.paste(Image.fromarray(a.astype("uint8")), (0, 0))
                tile.paste(Image.fromarray(k.astype("uint8")), (a.shape[1] + 8, 0))
                tile.paste(Image.fromarray(((abs(a - k) > 3).any(-1) * 255).astype("uint8")).convert("RGB"), (a.shape[1] * 2 + 16, 0))
                os.makedirs(KIT, exist_ok=True)
                tile.save(os.path.join(KIT, f"sheet-{tag}-{name}{'-focus' if focused else ''}-{args.mode}.png"))


ICON_CELL = 32
ICON_COLUMNS = 8
ICON_SIZE = 20
ICON_POINTS = 16


def icons(args):
    with open(os.path.join(CAPTURES, "icons", "ref.json")) as handle:
        em = {item["name"]: item["em"] for item in json.load(handle) if item["ok"]}
    with open(os.path.join(KIT, "lab-icons.txt")) as handle:
        names = handle.read().split("\n")
    sheet = np.array(Image.open(os.path.join(KIT, "lab-icons.png")).convert("RGB")).astype(float).mean(-1) / 255
    cell = ICON_CELL * SCALE
    print(f"{'symbol':32} {'AppKit w x h':>13} {'kit w x h':>13} {'d box':>6} {'d mass%':>8} {'d center':>9}")
    passed = near = 0
    for index, name in enumerate(names):
        row, column = divmod(index, ICON_COLUMNS)
        ink = 1 - sheet[row * cell:(row + 1) * cell, column * cell:(column + 1) * cell]
        ys, xs = np.nonzero(ink > 0.1 * ink.max())
        kit = ((xs.max() + 1 - xs.min()) / SCALE, (ys.max() + 1 - ys.min()) / SCALE, ink.sum() / SCALE ** 2,
               ((xs.min() + xs.max() + 1) / 2 / SCALE - ICON_CELL / 2, (ys.min() + ys.max() + 1) / 2 / SCALE - ICON_CELL / 2))
        e = em[name]
        ref = ((e["x1"] - e["x0"]) * ICON_POINTS, (e["y1"] - e["y0"]) * ICON_POINTS, e["mass"] * ICON_POINTS ** 2,
               ((e["x0"] + e["x1"]) / 2 * ICON_POINTS, (e["y0"] + e["y1"]) / 2 * ICON_POINTS))
        box = max(abs(kit[0] - ref[0]), abs(kit[1] - ref[1]))
        mass = (kit[2] / ref[2] - 1) * 100
        offset = max(abs(kit[3][0] - ref[3][0]), abs(kit[3][1] - ref[3][1]))
        passed += box <= 0.5 and abs(mass) <= 5
        near += box <= 0.5 and abs(mass) <= 10
        print(f"{name:32} {ref[0]:6.2f} x {ref[1]:5.2f} {kit[0]:6.2f} x {kit[1]:5.2f} {box:6.2f} {mass:+8.1f} {offset:9.2f}")
    print(f"{passed} of {len(names)} within 0.5 of the ink box and 5% of the mass")
    print(f"{near} of {len(names)} within 0.5 of the ink box and 10% of the mass")


CONTAINERS = ["split-v-thin", "split-h-thin", "swiftui-hsplit", "swiftui-vsplit"]


LIGHTS_PROBES = {"light": "aqua", "dark": "darkAqua", "hc-light": "aquaHC", "hc-dark": "darkAquaHC"}
LIGHTS_BOX, LIGHTS_MARGIN = 22, 4
LIGHT_NAMES = ("close", "min", "zoom")


def lights_shot(prefix, state):
    """The window capture of a state (sRGB), or None."""
    path = os.path.join(CAPTURES, "lights", f"shot-{prefix}-{state}-screen.png")
    if not os.path.exists(path):
        return None
    converted = path.replace("-screen.png", "-srgb.png")
    if not os.path.exists(converted):
        os.system(f'sips -m "/System/Library/ColorSync/Profiles/sRGB Profile.icc" "{path}" --out "{converted}" >/dev/null 2>&1')
    return rgb(converted)


def lights_layer(prefix, state, light, background):
    """The 22 x 22 raster of a light's base layer over `background`, in sRGB (the layer images carry their own profile, mostly Display P3), or None."""
    path = os.path.join(CAPTURES, "lights", f"{prefix}-unified-{state}-{light}-layer0.png")
    if not os.path.exists(path):
        return None
    raster = Image.open(path)
    profile = raster.info.get("icc_profile")
    raster = raster.convert("RGBA")
    if profile:
        raster = ImageCms.profileToProfile(raster, ImageCms.ImageCmsProfile(io.BytesIO(profile)), ImageCms.createProfile("sRGB"), renderingIntent=ImageCms.Intent.RELATIVE_COLORIMETRIC, outputMode="RGBA")
    image = np.asarray(raster).astype(float)
    alpha = image[..., 3:4] / 255
    return np.rint(image[..., :3] * alpha + np.array(background, float) * (1 - alpha)).astype(int)


def lights(args):
    prefix = LIGHTS_PROBES[args.mode]
    kit = rgb(os.path.join(KIT, f"kit-lights-{args.mode}.png"))
    with open(os.path.join(KIT, "layout-lights.tsv")) as handle:
        rows = [line.split("\t") for line in handle.read().splitlines()]
    background = kit[2, 2]
    shots = {"hover": "hover-close", "press-close": "press-close", "press-min": "press-min", "press-zoom": "press-zoom", "fullscreen": "fs"}
    print(f"{args.mode}: kit {SCALE}x over #%02x%02x%02x; mean = mean |difference| in 8-bit levels, >3 and >8 = share of pixels with any channel further than that" % tuple(background))
    print(f"{'state':15} {'light':5} {'mean':>5} {'>3':>6} {'>8':>6} {'core >3':>8} {'core >8':>8} {'pixel AppKit':>13} {'pixel kit':>10}")
    totals = []
    for name, x, y in rows:
        x, y = int(float(x)), int(float(y))
        for index, light in enumerate(LIGHT_NAMES):
            left, top = x + index * 23 - LIGHTS_MARGIN, y - LIGHTS_MARGIN
            mine = kit[top * SCALE:(top + LIGHTS_BOX) * SCALE, left * SCALE:(left + LIGHTS_BOX) * SCALE]
            if SCALE > 1:
                mine = mine.reshape(LIGHTS_BOX, SCALE, LIGHTS_BOX, SCALE, 3).mean(axis=(1, 3)).round().astype(int)
            if name in shots:
                shot = lights_shot(prefix, shots[name])
                top, left = (39, 9) if name == "fullscreen" else (19, 19)
                theirs = None if shot is None else shot[top - LIGHTS_MARGIN:top - LIGHTS_MARGIN + LIGHTS_BOX, left + index * 23 - LIGHTS_MARGIN:left + index * 23 - LIGHTS_MARGIN + LIGHTS_BOX]
            elif name in ("rest", "inactive", "inactive-hover"):
                theirs = lights_layer(prefix, "inactive-rest" if name == "inactive" else "rest", light, background)
                if name == "inactive-hover":
                    shot = lights_shot(prefix, "hover-close")
                    theirs = None if shot is None else shot[19 - LIGHTS_MARGIN:19 - LIGHTS_MARGIN + LIGHTS_BOX, 19 + index * 23 - LIGHTS_MARGIN:19 + index * 23 - LIGHTS_MARGIN + LIGHTS_BOX]
            else:
                theirs = None
            if theirs is None:
                continue
            diff = abs(mine - theirs).max(-1)
            centre = np.hypot(*np.mgrid[:LIGHTS_BOX, :LIGHTS_BOX] - LIGHTS_BOX / 2 + 0.5)
            core = diff[centre <= 5.5]
            totals.append((diff.mean(), (diff > 3).mean(), (diff > 8).mean(), (core > 3).mean(), (core > 8).mean()))
            mid = LIGHTS_BOX // 2 - 3
            hexa = lambda c: "#%02x%02x%02x" % tuple(c)
            print(f"{name:15} {light:5} {diff.mean():5.1f} {(diff > 3).mean() * 100:5.1f}% {(diff > 8).mean() * 100:5.1f}% {(core > 3).mean() * 100:7.1f}% {(core > 8).mean() * 100:7.1f}% {hexa(theirs[mid, LIGHTS_BOX // 2]):>13} {hexa(mine[mid, LIGHTS_BOX // 2]):>10}")
    if totals:
        mean, share3, share8, core3, core8 = np.mean(totals, axis=0)
        print(f"all: mean {mean:.2f}, >3 {share3 * 100:.1f}%, >8 {share8 * 100:.1f}%; inside 5.5 of the centre (no edge band) >3 {core3 * 100:.1f}%, >8 {core8 * 100:.1f}%")


def luminance(color):
    linear = np.where(np.asarray(color, float) / 255 <= 0.04045, np.asarray(color, float) / 255 / 12.92, ((np.asarray(color, float) / 255 + 0.055) / 1.055) ** 2.4)
    return float(linear @ [0.2126, 0.7152, 0.0722])


def contrast_ratio(a, b):
    high, low = sorted((luminance(a), luminance(b)), reverse=True)
    return (high + 0.05) / (low + 0.05)


def lights_tint(args):
    """Legibility of the tinted lights: the glyph against its light, the lights against the backdrop and against each other."""
    with open(os.path.join(KIT, "layout-lights.tsv")) as handle:
        rows = {line.split("\t")[0]: (int(float(line.split("\t")[1])), int(float(line.split("\t")[2]))) for line in handle.read().splitlines()}
    hexa = lambda c: "#%02x%02x%02x" % tuple(int(v) for v in c)
    print(f"{'tint':18} {'light':5} {'fill':>8} {'glyph':>8} {'glyph/fill':>10} {'fill/back':>9}  (WCAG 2.2 contrast ratio; the standard lights are the reference)")
    for suffix in args.tints.split(","):
        name = f"kit-lights-{args.mode}{'-' + suffix if suffix != 'standard' else ''}.png"
        image = rgb(os.path.join(KIT, name))
        background = image[2, 2]
        for index, light in enumerate(LIGHT_NAMES):
            rest_x, rest_y = rows["rest"]
            hover_x, hover_y = rows["hover"]
            fill = image[(rest_y + 7) * SCALE, (rest_x + index * 23 + 7) * SCALE]
            glyph = image[(hover_y + 7) * SCALE, (hover_x + index * 23 + 7) * SCALE] if light != "zoom" else image[(hover_y + 5) * SCALE, (hover_x + index * 23 + 5) * SCALE]
            print(f"{suffix:18} {light:5} {hexa(fill):>8} {hexa(glyph):>8} {contrast_ratio(glyph, fill):10.2f} {contrast_ratio(fill, background):9.2f}")


def main():
    parser = argparse.ArgumentParser(description="Compare a kit scene with AppKit captures.")
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("icons", help="print the symbol ink box and mass table against AppKit").set_defaults(run=icons)
    one = commands.add_parser("layout", help="write the scene layout from a probe JSON")
    one.add_argument("probe_json")
    one.add_argument("output")
    one.set_defaults(run=layout)
    two = commands.add_parser("controls", help="print the per-control difference table")
    two.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    two.add_argument("--family", default="controls", choices=sorted(FAMILIES), help="the probe family")
    two.set_defaults(run=controls)
    ten = commands.add_parser("picker", help="compare the accent picker with System Settings")
    ten.add_argument("mode", choices=("light", "dark"))
    ten.set_defaults(run=picker)
    nine = commands.add_parser("accents", help="print the control difference for every accent")
    nine.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    nine.add_argument("--mode", choices=("light", "dark"), help="one appearance only")
    nine.set_defaults(run=accents)
    focus = commands.add_parser("focused", help="print the difference table for focused text controls")
    focus.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    focus.set_defaults(run=focused)
    five = commands.add_parser("pressed", help="print the per-control difference table for pressed controls")
    five.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    five.add_argument("--family", default="controls", choices=sorted(PRESSED_FAMILIES), help="the probe family")
    five.set_defaults(run=pressed)
    three = commands.add_parser("sheet", help="write a side-by-side image of named items")
    three.add_argument("family", choices=("controls", "views", "texts"))
    three.add_argument("mode", choices=("light", "dark"))
    three.add_argument("backdrop", choices=("white", "black"))
    three.add_argument("names", help="comma separated item names")
    three.add_argument("--tag", default="", help="views, views-hc or hc")
    three.set_defaults(run=sheet)
    four = commands.add_parser("table", help="print the table header, text and separator positions")
    four.add_argument("mode", choices=("light", "dark"))
    four.set_defaults(run=table)
    six = commands.add_parser("toolbar", help="print the toolbar glyph and navigation pair comparison")
    six.add_argument("mode", choices=("light", "dark"))
    six.set_defaults(run=toolbar)
    lights_parser = commands.add_parser("lights", help="print the per-state difference of the traffic lights (1x, from the lights probe)")
    lights_parser.add_argument("mode", choices=sorted(LIGHTS_PROBES))
    lights_parser.set_defaults(run=lights)
    tint_parser = commands.add_parser("lights-tint", help="print the legibility of the tinted traffic lights (scenes rendered with APPKIT_LIGHTS_SUFFIX)")
    tint_parser.add_argument("mode", choices=sorted(LIGHTS_PROBES))
    tint_parser.add_argument("tints", help="comma separated suffixes, standard for none")
    tint_parser.set_defaults(run=lights_tint)
    seven = commands.add_parser("sidebar", help="print the sidebar window comparison")
    seven.add_argument("mode", choices=("light", "dark"))
    seven.set_defaults(run=sidebar)
    eight = commands.add_parser("lists", help="print the per-item difference table of the lists, tables and forms")
    eight.add_argument("mode", choices=("light", "dark"))
    eight.add_argument("--tag", default="", help="empty, or hc for Increase Contrast")
    eight.add_argument("--names", default="", help="comma separated item names")
    eight.add_argument("--sheet", action="store_true", help="write a side-by-side image per item")
    eight.set_defaults(run=lists)
    args = parser.parse_args()
    args.run(args)


if __name__ == "__main__":
    sys.exit(main())
    eight.add_argument("--family", default="lists", choices=("lists", "containers"), help="the probe family")
