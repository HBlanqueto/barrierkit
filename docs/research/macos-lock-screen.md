# macOS lock screen

**Goal:** state what is known about the macOS lock screen, separate measured
facts from estimates, and give Malaysia a checklist for its own lock screen.
**Scope:** the locked state, the clock and date, the user avatar, the password
field and the unlock transition.
**Reader:** anyone who changes `malaysia-shell::lock` or the lock tokens.
**Input:** public Apple descriptions and the live macOS design language.
**Output:** an explicit evidence gap, a token comparison and a capture plan.

## Steps

1. Read the safety note before trying to reproduce this file.
2. Treat every geometry value here as **estimated** unless it is marked
   otherwise.
3. Compare Malaysia's tokens with the estimates only after a real capture.
4. Do not copy Apple wallpaper, avatars or glyphs.
5. Record the host in the observation log when a capture is possible.

---

## 1. Safety note

The macOS lock screen was **not captured**. The host has one display and one
session. Locking it would have stopped the agent and required a manual
unlock. That is an unacceptable risk for an automated research session.

This file therefore contains **no measured lock-screen values**. The geometry
below is inferred from public Apple design descriptions and from the visual
language of the captured surfaces. It is useful as a hypothesis, not as a
specification.

## 2. Expected structure

A modern macOS lock screen has, from top to bottom:

- A dimmed or blurred desktop wallpaper.
- A very large time, centred near the top.
- A date line under the time.
- A user avatar and name near the bottom.
- A password or passcode field.
- A Touch ID prompt or a "touch to unlock" hint.
- Login buttons and, on some machines, a power or restart control.

The layout is centred and uses the same SF Pro family as the rest of the
system. The wallpaper is the dominant colour source. The lock screen is not a
glass panel; it is a full-screen layer over the wallpaper.

This structure is well known, but the exact sizes on macOS 27 were not
measured.

## 3. Estimated metrics

These are **estimates** for a first comparison only:

| Element | Estimate | Why |
|---|---:|---|
| Clock size | 120 to 160 | The Malaysia token is 140 |
| Date size | 24 to 30 | The Malaysia token is 28 |
| Clock top | about 9% of height | The Malaysia token is `top-percent: 9` |
| Avatar | about 52 | Matches the Malaysia token |
| Password field | about 22x120 | Matches the Malaysia token |
| Field mask size | about 18 | Matches the Malaysia token |
| Bottom margin | about 55 | Matches the Malaysia token |
| Spacing | about 12 | Matches the Malaysia token |

The fact that these ranges agree with the Malaysia tokens does **not** prove
them. Malaysia may have copied the values from an older macOS or from a
different display size.

## 4. Known behaviour

From public Apple documentation and the live system:

- The lock screen appears after sleep, after a screen lock and on login.
- Unlocking fades to the desktop.
- A wrong password produces a horizontal shake and an error state.
- The clock updates live.
- Reduce Motion and Reduce Transparency affect the transition and the
  wallpaper treatment.
- Touch ID can unlock without typing a password.

The durations and curves were not measured. The Malaysia tokens
`LOCK_FADE_MS=1100`, `LOCK_SETTLE_MS=600`, `LOCK_RISE_MS=900`,
`LOCK_RISE_OFFSET=120` and `LOCK_LOGIN_MS=600` are project values.

## 5. Comparison with Malaysia

| Element | macOS | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Clock | Not measured | 140 | `LOCK_CLOCK_FONT_SIZE` | Verify |
| Date | Not measured | 28 | `LOCK_DATE_FONT_SIZE` | Verify |
| Clock top | Not measured | 9% | `LOCK_TOP_PERCENT` | Verify |
| Bottom margin | Not measured | 55 | `LOCK_BOTTOM_MARGIN` | Verify |
| Spacing | Not measured | 12 | `LOCK_SPACING` | Verify |
| Avatar | Not measured | 52 | `LOCK_AVATAR_SIZE` | Verify |
| Password field | Not measured | 22x120 | `LOCK_FIELD_*` | Verify |
| Mask font | Not measured | 18 | `LOCK_MASK_FONT_SIZE` | Verify |
| Ink | Not measured | `#ffffff` | `LOCK_INK` | Verify |
| Field fill | Not measured | `#ffffff4d` | `LOCK_FIELD_FILL` | Verify |
| Avatar fill | Not measured | `#5c5d61` | `LOCK_AVATAR_FILL` | Verify |
| Error | Not measured | `#ff6b6b73` | `LOCK_FIELD_ERROR` | Verify |
| Field ink | Not measured | `#ffffffbf` | `LOCK_FIELD_INK` | Verify |
| Fade | Not measured | 1100 | `LOCK_FADE_MS` | Verify |
| Settle | Not measured | 600 | `LOCK_SETTLE_MS` | Verify |
| Rise | Not measured | 900, 120 offset | `LOCK_RISE_*` | Verify |
| Login | Not measured | 600 | `LOCK_LOGIN_MS` | Verify |

There is no measured gap. There is a **measurement gap**.

## 6. What not to copy

- Apple wallpaper.
- The user's avatar or name.
- Apple's Touch ID glyph or lock glyph.
- Apple's button labels or translations.
- A screenshot of a lock screen as a background.

## 7. Capture plan

1. Use a disposable macOS virtual machine or a second physical host, never
   the only session.
2. Set a known wallpaper with flat colour patches.
3. Capture the lock screen at 1x and 2x.
4. Measure the clock bounding box, date, avatar, field and bottom controls.
5. Record the unlock transition at 60 fps.
6. Capture light and dark wallpapers to see how the ink adapts.
7. Turn Reduce Motion and Reduce Transparency on and repeat the transition.

Until this plan runs, keep the lock-screen doc marked as estimated.

## 8. Prioritized changes

1. **Do not tune Malaysia's lock screen from this file.** The values are not
   measured.
2. **Schedule the VM capture.** It is the only way to close the area.
3. **Add a lock-screen contrast test.** The white ink must survive a light
   wallpaper.
4. **Verify the field and error states.** They are the most important
   interaction on the screen.
5. **Verify the transition under Reduce Motion.** A fade is the safe
   fallback.

## 9. Open questions

- Is the clock size fixed or proportional to the display?
- Does the date sit above or below the clock on macOS 27?
- Is the avatar always shown, or only on multi-user machines?
- Does the password field use a glass fill or a flat translucent fill?
- Does the lock screen show a Dock or menu bar on any configuration?
