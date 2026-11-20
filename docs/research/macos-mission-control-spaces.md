# macOS Mission Control and Spaces

**Goal:** describe the overview surface that shows every window and the Spaces
bar, and separate it from the normal desktop slide.
**Scope:** the open Mission Control view and the Spaces bar. It excludes
full-screen app spaces, Stage Manager and window tiling.
**Reader:** anyone who changes the compositor's space switching or adds an
overview mode.
**Input:** one Mission Control capture with two spaces and one window.
**Output:** observed structure, a Malaysia comparison and ordered gaps.

## Steps

1. Read the limits in
   [`macos-observations.md`](macos-observations.md).
2. Use this file for the overview, not for the normal space slide.
3. Do not treat the window thumbnail as a fixed size.
4. Check the comparison before adding an overview layer.
5. Do not copy window contents or application icons.

---

## 1. Structure

Mission Control replaces the desktop with an overview:

- A **Spaces bar** at the top centre.
- The current desktop, dimmed and blurred.
- Scaled **window thumbnails** laid out over the desktop.
- A **title pill** near the focused thumbnail.
- The Dock is hidden.

The captured state has two spaces, `Escritorio 1` and `Escritorio 2`, and one
window. The focused window has a blue outline. The desktop behind the window
is blurred, so the window is the only sharp element.

The normal desktop does not look like this. The normal state is a wallpaper,
windows and the Dock, with no overview.

## 2. Observed geometry

The capture is 1x. Exact values were not sampled in this pass, so the
following are **inferred** from the image:

| Element | Observation |
|---|---|
| Spaces bar | Top centre, above the desktop |
| Space labels | Two text labels separated by a divider |
| Selected space | Highlighted |
| Window thumbnail | One, centred in the desktop area |
| Title pill | Under or near the thumbnail |
| Focus outline | Blue, around the thumbnail |
| Desktop | Blurred and dimmed |
| Dock | Hidden |

No exact bar height, thumbnail width, gap or blur radius is available. Do not
copy the label text or the divider width as tokens.

## 3. Behaviour

- Mission Control opens as an overview, not as a slide.
- Clicking a window focuses it and closes the overview.
- Clicking a space switches to that space.
- Dragging a window to a space moves it.
- The overview hides the Dock.
- The overview uses the same wallpaper as the desktop, blurred and dimmed.

The open and close animations were not measured. The transition may scale the
windows from their desktop rectangles; this is the usual macOS behaviour, but
it is not confirmed from this capture.

## 4. Comparison with Malaysia

| Element | macOS (inferred) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Overview mode | Present | Absent | — | New compositor layer |
| Spaces bar | Top centre | No overview bar | `SPACE_GAP` | No bar |
| Space labels | Present | Spaces exist without labels | `malaysia-scene::spaces` | Add labels |
| Window thumbnails | Scaled windows | Only minimise thumbnails | `minimize.rs` | No overview thumbnails |
| Focus outline | Blue around window | Focus ring exists | `focus_ring` | No overview use |
| Desktop blur | Yes | Blur primitives exist | `blur.rs` | No overview composition |
| Desktop dim | Yes | Not measured | — | Add a dim layer |
| Dock hidden | Yes | Likely not hidden | `dock/mod.rs` | Add a hide state |
| Normal space slide | Not measured | 480 ms cubic bezier | `SPACE_SLIDE_MS` | Keep separate |
| Focus bar | Not measured | `#000` bar, `#fff` text | `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | Verify |

The largest gap is the missing **overview mode**. Malaysia has spaces and a
slide, but it does not have a screen that shows every window at once.

## 5. What not to copy

- Window contents from the capture.
- Application icons or titles.
- Apple's Space names or their translations.
- Apple's exact wallpaper or thumbnail shadow.

Use live window thumbnails and project labels.

## 6. Prioritized changes

1. **Add an overview layer in the compositor.** It needs a blurred desktop,
   a dim layer, window thumbnails and a focus outline. This is the core of
   Mission Control.
2. **Add a Spaces bar with text labels.** Put it at the top centre and mark
   the selected space. Reuse the existing space model.
3. **Hide the Dock and the normal windows while the overview is open.**
   Otherwise the overview reads as a transparent panel, not a mode.
4. **Draw a blue focus outline around the focused thumbnail.** Reuse the
   focus-ring colour and width.
5. **Animate the thumbnails from their desktop rectangles.** Measure the
   duration before tuning it; do not copy 480 ms from the space slide.
6. **Keep the normal slide separate.** Mission Control is not a space
   transition.

## 7. Open questions

- What is the Spaces bar height and its gap from the top?
- How many columns of windows are used before the overview scrolls?
- Does the overview show all windows or only the current space?
- Does the Dock hide fully or fade?
- What is the exact blue of the focus outline?
- How does the transition behave with Reduce Motion?

## 8. Implementation prompt

```text
You are verifying Mission Control behaviour from measurements. Read this
file and macos-blur.md first. Work in ~/src/malaysia only, main only:
git pull --ff-only before editing and before each push, one commit per
session, English messages, no code comments.

Measured facts:
- macOS 27.2 dims neither the wallpaper nor the windows in Mission
  Control; the background is pixel-identical to the desktop in both
  appearances. Do not add a background blur or scrim.
- The Spaces bar is clear with glass thumbnails.
- The focus outline is blue.

Tasks:
1. malaysia: verify the overview renders no background material; if a
   blur or scrim exists, remove it.
2. Keep the Spaces bar clear; draw thumbnails with the existing window
   surface glass.

Definition of done: ./dev.sh check passes; the overview background is
pixel-identical to the desktop; main is pushed. Report changed files and
commits.
```
