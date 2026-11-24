# 5. Lock screen

**Goal:** lock the session with a component of your own, end the lock when
the user may come back, and let the other shell surfaces follow the lock.
**Scope:** `run_lock`, `SessionUnlock` (`unlock`, `locked`,
`keyboard_layout`), the order of `ext-session-lock-v1` as the kit runs it,
what the compositor must do, and `use_screen_lock` with `ScreenLock`. The
password field is in [5. Controls](../barrierkit/05-controls.md#passwords);
checking the password is not in the kit. The blur behind a surface is in
[7. Glass and blur](07-glass-and-blur.md).
**Reader:** whoever writes Malaysia's lock screen, or a surface that changes
while the session locks.
**Input:** a root component that draws the lock screen, as in
[2](02-first-component.md), and a way to check the user.
**Output:** a process that locks the session, draws on every output and
returns when the session is unlocked.

## Steps

1. Write the root component. Paint an opaque background: the lock surface is
   cleared transparent.
2. Inside it, take the handle with `SessionUnlock::get()`
   ([`SessionUnlock`](#sessionunlock)).
3. When the user may come back, call `unlock()`.
4. Run the component with `run_lock(app)`. It returns `Ok(())` once the
   session is unlocked ([The order of the lock](#the-order-of-the-lock)).
5. Run it first inside a nested compositor
   ([10. Run and debug](10-run-and-debug.md#run-nested)). If the process dies while the
   session is locked, the session stays locked.

This program is `crates/bk-shell/examples/lock_screen.rs`. It shows the
keyboard layout and an Unlock button, and unlocks by itself after 30 seconds,
so a mistake cannot keep the session locked:

```rust
use std::time::Duration;

use bk::{platform::Timers, prelude::*};
use bk_shell::{SessionUnlock, run_lock};
use freya_core::integration::AppComponent;

// An example must never leave the session locked: it unlocks on its own.
const SAFETY_UNLOCK: Duration = Duration::from_secs(30);

fn lock_screen() -> impl IntoElement {
    let session = SessionUnlock::get();
    use_hook({
        let session = session.clone();
        move || Timers::get().after(SAFETY_UNLOCK, move || session.unlock())
    });
    VStack::new()
        .expanded()
        .background(color(Palette::current().window_background))
        .child(Text::new(session.keyboard_layout().unwrap_or_default()))
        .child(Button::new("Unlock").on_press(move |_| session.unlock()))
}

fn main() -> Result<(), RunError> {
    run_lock(AppComponent::from(lock_screen))
}
```

A real lock screen calls `unlock()` only after it checks the password from a
`SecureField`, and has no timer.

## `run_lock`

`run_lock(app: AppComponent) -> Result<(), RunError>` binds
`ext_session_lock_manager_v1`, version 1, beside the runner's globals. Without
it, it returns `RunError::MissingGlobal("ext_session_lock_manager_v1")` before
it locks. Its other errors are the runner's
([8. Windows](../barrierkit/08-windows.md#errors)).

Each output draws its own instance of `app`: the outputs share no component
state. Every instance gets these root contexts:

| Context | Chapter |
|---|---|
| `SessionUnlock` | [Below](#sessionunlock) |
| `Popups`, `Timers` | [8. Windows](../barrierkit/08-windows.md#running-a-window) |
| `Toplevels`, `Activation` | [4. Desktop windows](04-desktop-windows.md) |
| `use_screen_lock()` | [Below](#locking-notices-for-the-other-surfaces) |

A lock surface cannot be a popup's parent. `Popups::get().open` opens nothing
there and calls the popup's `on_dismissed`, so menus, popovers and tooltips do
not open on the lock screen.

## The order of the lock

The kit and the compositor run these steps:

1. The kit sends `ext_session_lock_manager_v1.lock` as soon as it connects,
   before its event loop starts.
2. When the loop starts, it creates `SessionUnlock` and one lock surface
   (`get_lock_surface`) for each output it knows.
3. The compositor sends each lock surface a `configure` with its exact size.
   The kit acknowledges it and commits its first frame at that size; it never
   commits a lock surface before.
4. The compositor sends `locked` once every output shows a lock frame or a
   blank color. `locked()` turns `true` and every surface draws again. An
   `unlock()` that came earlier runs now.
5. An output that appears gets its own lock surface; one that goes away loses
   its surface.
6. `unlock()` sends `unlock_and_destroy`, waits until the compositor has
   processed it (a roundtrip), and stops the loop. `run_lock` returns `Ok(())`.

If the compositor sends `finished` instead, because another client holds the
lock or it ends the lock its own way, the kit logs
`BarrierKit: the compositor ended the session lock` and the process exits with
status 1.

## What the compositor must do

`ext-session-lock-v1` puts these duties on the compositor:

1. Answer each `lock` with `locked` or `finished`; send `finished` at once if
   it will not lock.
2. Once locked, stop drawing and sending input to the other clients, and cover
   every output with an opaque color.
3. Send `locked` only after a locked frame is on every output. It may wait for
   the lock surfaces to draw, within a time limit.
4. Give each lock surface a `configure` with an exact size, which the client
   must match.
5. Choose which lock surface gets the keyboard, for example the first one, and
   move it on a click.
6. Keep the session locked if the lock client dies.

Malaysia also tells the other shell surfaces that the session is locking and
fades them out first ([Locking notices](#locking-notices-for-the-other-surfaces),
Malaysia ADR 0074).

## `SessionUnlock`

| Method | Returns or does |
|---|---|
| `SessionUnlock::get()` | The unlock handle. It exists only under `run_lock`; elsewhere it panics |
| `unlock()` | Ends the lock ([step 6](#the-order-of-the-lock)). Before `locked`, it waits for it |
| `locked()` | `true` once the compositor sends `locked`. A component that reads it renders again when it changes: start the entry animation then |
| `keyboard_layout()` | The name of the active keyboard layout in the compositor's keymap, such as «Spanish (Latin American)»; `None` before the first keymap |

`keyboard_layout()` follows `wl_keyboard.keymap` and the layout index of
`wl_keyboard.modifiers`. Only the surface with the keyboard focus receives
them; while the session is locked, that is a lock surface.

## Locking notices for the other surfaces

`use_screen_lock() -> State<ScreenLock>` lets a layer surface, such as the
menu bar, follow the lock. It reads `infinity_session_v1`, version 1, which
any client may bind (`crates/infinity-protocol/protocols/infinity-session-v1.xml`):

| `ScreenLock` | Event | Meaning |
|---|---|---|
| `Unlocked` (default) | `unlocked` | The session is unlocked |
| `Locking(duration)` | `locking` | A client asked to lock. The compositor hides every other surface over `duration`, then shows the lock screen; a zero `duration` means no transition |

```rust
let lock = use_screen_lock();
let locking = matches!(*lock.read(), ScreenLock::Locking(_));
```

Malaysia's menu bar uses it to close up into the lock screen's status row
while the compositor fades the rest out. Without `infinity_session_v1` the
state stays `Unlocked`.

Next: [6. Status items](06-status-items.md).
