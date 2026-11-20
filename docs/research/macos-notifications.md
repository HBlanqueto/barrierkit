# macOS notification banners

**Goal:** describe a notification banner in the top-right corner and compare
it with the notification tokens in Malaysia.
**Scope:** a single visible banner in dark mode. It excludes Notification
Center, stacks, grouping, actions and the notification server.
**Reader:** anyone who changes `malaysia-shell::notifications` or the
notification tokens.
**Input:** a banner produced by `display notification` and a screenshot.
**Output:** measured frame, typography, a Malaysia comparison and ordered
gaps.

## Steps

1. Read the limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the frame table for the banner.
3. Use the typography table before changing the title font.
4. Do not infer Notification Center from the banner.
5. Do not copy application icons or system glyphs.

---

## 1. Measured frame

| Property | Value |
|---|---:|
| Left | about 2210 |
| Right | about 2540 |
| Width | about 330 |
| Top | 30 |
| Bottom | about 100 |
| Height | about 70 |
| Right inset | about 20 |
| Corner radius | about 18 |
| Icon | 38 |
| Material | dark glass |

The banner starts at the menu-bar height, 30, and sits 20 pt from the right
edge. The width is 330 and the height is 70. The corner radius is about 18.

The banner is a dark glass card: the desktop shows through, blurred, and the
card has a light rim. The 2026-11-20 pass measured the glass over the checker:
`#86` to `#8c` in dark mode and `#b3b3b3` in light mode, so the banner is a
lighter veil in light mode, like the menu. Notification Center opened empty on
this host; its clock pill glass measured `#bc` over white, `#5c` over black
and `#9f` over checker in dark mode, and `#ed`, `#9e` and `#d2` in light mode.

## 2. Typography

| Line | Size | Weight |
|---|---:|---|
| Title | 13 | bold |
| Subtitle | 12 | regular |
| Body | 13 | regular |

The icon is 38x38. The title and subtitle form a two-line header. The body
sits below them. The banner is compact: 13 pt title, not a 17 pt title.

On hover, the banner reveals an **`Opciones`** control. That is the Spanish
label for Options. The control appears at the bottom or trailing edge of the
card and is not visible at rest.

No close button was visible at rest. Whether one appears on hover was not
captured.

## 3. Behaviour

- The banner appears at the top-right, under the menu bar.
- It is clickable and dismissible, but the exact hit targets were not
  measured.
- The banner likely opens Notification Center when clicked; that was not
  confirmed.
- Clicking the clock area did not open Notification Center on this host, so
  the center is not described here.
- The slide-in and slide-out durations, the bounce and the stack peek were
  not measured.
- The maximum number of visible banners was not measured.

Do not copy Malaysia's 450 ms slide or 260 ms slide-out as macOS values.
They are project values.

## 4. Comparison with Malaysia

| Element | macOS (measured) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Width | about 330 | 330 | `NOTIFICATION_WIDTH` | Matches |
| Height | about 70 | Content-driven | `NOTIFICATION_PADDING` | Verify with the same text |
| Radius | about 18 | 18 | `NOTIFICATION_RADIUS` | Matches |
| Right inset | about 20 | 20 | `NOTIFICATION_MARGIN` | Matches |
| Top edge | 30 | Bar height plus gap | `NOTIFICATION_GAP` | Verify the gap |
| Icon | 38 | 38 | `NOTIFICATION_ICON_SIZE` | Matches |
| Title | 13 bold | 13/18 bold | `HEADLINE` | Matches |
| Notification Center heading | Not measured | 17/22 bold | `NOTIFICATION_TITLE_FONT` | Unverified; only the heading uses it |
| Subtitle | 12 | No subtitle line | `CALLOUT` (12) | The kit style is 12; the banner shows the age in `CAPTION` |
| Body | 13 | 13/18 | `BODY` | Matches |
| Hover action | `Opciones` | Options action | `notifications/view.rs` | Verify shape |
| Glass | Dark glass | Notification glass | `notification_glass` | Reuse menu model |
| Motion | Not measured | 450 in, 260 out, bounce 0.15 | `NOTIFICATION_SLIDE_*` | Unverified |

The banner title is 13 pt bold in both. `NOTIFICATION_TITLE_FONT` (17/22) is
only the Notification Center heading, which was not measured.

## 5. What not to copy

- Application icons shown in the banner.
- Apple's notification action glyphs.
- Apple's notification text or translations.
- A screenshot of a banner as a material.

## 6. Prioritized changes

1. **Verify the height with the same content.** The 70 pt banner has a
   38 pt icon, three text lines and padding; the current token set can
   produce a different height.
2. **Measure the enter and exit motion.** Use the pre-rolled recorder and a
   banner with a fixed delay. Do not keep unverified timings.
3. **Measure the stack.** The peek and inset values are unverified.

## 7. Open questions

- Does the banner use the same glass as the application menu?
- What is the exact padding around the 38 pt icon?
- Is the hover action a button, a menu or a text link?
- How many banners stack before the oldest is replaced?
- Does Notification Center use the same card width?

## 8. Implementation prompt

```text
You are fixing the notification glass from measured values. Read this file
and macos-blur.md first. Work in ~/src/barrierkit and ~/src/malaysia,
main only: git pull --ff-only before editing and before each push, one
commit per session, English messages, no code comments.

Measured targets:
- Banner over checker: #86-8c dark / #b3b3b3 light.
- NC clock pill: white #bc / #ed, black #5c / #9e, checker #9f / #d2
  (dark / light).
- Banner typography: 13 pt bold title, 12 pt subtitle, 13 pt body.

Tasks:
1. barrierkit: align the notification glass tokens with the two veil sets
   in macos-blur.md section 5; four-backdrop tests per scheme.
2. barrierkit: reduce the banner title to 13 pt bold with a 12 pt
   subtitle.
3. malaysia: after the rev bump, verify the banner renders the resolved
   glass.

Do not change the slide-in or slide-out durations in this pass; they are
unverified project values.

Definition of done: ./dev.sh check passes in both repos; tests assert the
measured glass values; barrierkit main is pushed and malaysia pins the
new rev. Report changed tokens and commits.
```
