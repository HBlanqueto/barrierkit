# Linux verification

**Goal:** tell, feature by feature, whether the Linux backend works after the
sessions that changed it on a Mac.
**Scope:** from a clean checkout on the owner's Linux machine to a filled
result table. It starts at `nix develop` and ends when every row has a result.
Who provides each feature on each system is in
[Who provides what on each platform](18-platform.md#who-provides-what-on-each-platform);
this guide does not repeat it.
**Reader:** the owner, on Linux, with a GPU and Vulkan.
**Inputs:** this checkout; a Malaysia session for the rows marked *Malaysia*;
a GNOME or KDE session for the rows marked *portal*.
**Output:** a result for each row of the table in step 5, the compiler and clippy
output of step 2, and the logs of the rows that failed.

## Why it is needed

The Linux code moved to `crates/bk/src/platform/linux/` on 2026-11-23
(`2683357e`) and changed in later commits that only the macOS `./dev.sh check`
saw. macOS does not compile `platform/linux/`, `crates/ctappearance/src/watch.rs`
or anything under `cfg(feature = "shell")`. Since the Linux VM went away on
2026-11-22 23:01, no session compiled them.

## Steps

1. Enter the dev shell at the root of the checkout:

   ```sh
   nix develop
   ```

2. Run the checks. On Linux `./dev.sh check` builds with `bk/shell,bk/tracy`
   and runs clippy with `-D warnings`, so it compiles every line in the
   last column of step 5:

   ```sh
   ./dev.sh check
   cargo test --workspace --features bk/shell
   ```

   A compile error stops here: fix it before step 3, since the recipes build
   the same code.
3. Open a session for the row: *Malaysia* rows under Malaysia
   ([Try it under Malaysia](18-platform.md#try-it-under-malaysia) says how to
   point `WAYLAND_DISPLAY` at it), *portal* rows under GNOME or KDE, *any*
   rows under either.
4. Launch the recipe of the row, with the log on. The recipes install no
   logger: add `bk::init_tracing();` as the first line of the recipe's `main`,
   and do not commit it.

   ```sh
   RUST_LOG=bk=debug,ctappearance=debug cargo run -p bk --example <recipe>
   ```

5. Do what the row says, compare with the expected result, and write the
   row's result: pass, or what differed.

| Feature | Session | Recipe | Do and look | Expected | Lines never compiled |
|---|---|---|---|---|---|
| Window, title bar, traffic lights | any | `hello_world` | Open, drag by the bar, minimize, maximize, close | The kit draws the bar and lights; each light acts | `linux/window.rs` command loop (`ea65d78a`) |
| Window styles | any | `window_designs`, `full_screen` | Open each window style | `TitleBar` shows the kit's bar; `HiddenTitleBar` and `Plain` show only what the app draws | `linux/window.rs` (`ea65d78a`, `2b6f9976`) |
| App id | any | `hello_world` | Look at the app id the compositor shows (Malaysia's Dock, `WAYLAND_DEBUG=1` `set_app_id`) | `com.example.hello`, with no Malaysia prefix | `linux/window.rs` `options.app_id` (`2b6f9976`) |
| Settings and restoration | any | `app_storage` | Tick Bold, resize the window, quit, open again | Bold stays ticked; the size comes back; `~/.config/com.example.hello/com.example.hello.domain` and `~/.local/share/com.example.hello/com.example.hello.window.state` exist | `linux/mod.rs` `app_folders`, `linux/window.rs` `restoration_file` (`3f1ab331`) |
| Settings changed outside | any | none | Use an app that reads its settings with `use_defaults` ([13](13-app.md#settings-and-restoration)); while it runs, edit its `.domain` file. `app_storage` cannot show it: `use_app_storage` reads once | The view shows the new value without a restart | `ctappearance/src/watch.rs` `watch_app_folder`, `linux/runtime.rs` `watch_app_folders` (`3f1ab331`) |
| Full screen | any | `full_screen` | Press the green light, then again | The window fills the output and comes back to its size | `linux/window.rs` (`ea65d78a`) |
| Focus mode | Malaysia | `full_screen` | Enter full screen; move the pointer to the top | Malaysia's bar slides down over the window; the title bar follows `focus_bar` ([9](09-titlebars.md#full-screen-and-focus-mode)) | `linux/focus.rs` (moved, `2683357e`) |
| Sidebar glass | Malaysia | `notes_sidebar` | Look at the sidebar over the wallpaper | Malaysia draws the sidebar material (`set_backdrop` behind the window at version 10, `set_glass` below) | `linux/glass.rs` (`ea65d78a`) |
| In-window glass, scroll edge | Malaysia | `glass_window`, `glass_toolbar` | Follow [Try it under Malaysia](18-platform.md#try-it-under-malaysia) | As that section says | `linux/chrome_surface.rs` (`ea65d78a`) |
| Glass container, glass effect | Malaysia | `glass_container`, `glass_effect` | Look at the glass shapes over the content | The compositor blurs each shape; on another compositor they are solid | `linux/chrome_surface.rs` (`ea65d78a`) |
| Inspector | any | `inspector` | Show the inspector | The kit draws it solid: native inspector glass is macOS only | none |
| Popover | any | `popups` | Open the popover | An `xdg_popup` beside its anchor; under Malaysia it has its arrow shape | `linux/popup.rs` (moved, `184bd711`) |
| Sheets and alerts | any | `presentations` | Open the alert | A modal sheet window over its parent; the parent takes no input | `linux/window.rs` `Sheet` (`ea65d78a`) |
| Documents | any | `documents` | Type, then close the window | The kit's "Save changes?" alert, buttons Save, Don't Save, Cancel | `linux/panels.rs` `confirm_save` (`8102cdfb`) |
| File dialogs | portal | `file_dialogs` | Press Import…, then Export… | The portal's chooser opens in the temporary folder; Import shows only `png` and `jpg` files, in any case; Export offers `Picture.png` | `linux/panels.rs` `present` (`1802dc7d`) |
| Pasteboard | any | `pasteboard` | Copy the text, paste it in another app; copy the image, then press Paste | Text pastes; after Copy image, Paste still shows the text: Linux holds text only, so the image copy changes nothing | `linux/clipboard.rs` (`ea65d78a`) |
| Light and dark | portal | `hello_world` | Switch the desktop between light and dark | The window follows while it runs | `linux/system.rs` (`e6f64057`) |
| High contrast, reduced motion | portal | `with_animation` | Turn on the desktop's high contrast, then reduced motion, and press the button | The colors take the increased-contrast palette; the change jumps with no animation | `linux/system.rs` `record`, `appearance` (`1802dc7d`) |
| Accessibility | any | `accessibility` | Read the window with Orca or Accerciser | Each control has its role and label | `linux/atspi.rs` (moved, `184bd711`) |
| Keyboard, text input | any | `documents` | Type, use an input method, right-click the field | Text and composed text arrive; the edit menu opens | `linux/seat.rs`, `linux/input.rs` (`ea65d78a`) |
| Single instance, notifications, secrets, status item | any | none | Use an app that calls `app::launch`, `notifications::post`, `Secrets` or `StatusItem` ([13](13-app.md#single-instance)) | A second launch raises the first window; a banner; a stored secret; an icon in the status area | `linux/instance.rs`, `linux/notifications.rs`, `linux/keychain.rs` (`ea65d78a`) |
| Shell surfaces | Malaysia | none | Run Malaysia's menu bar and Dock built at this rev | They draw and take input | everything under `cfg(feature = "shell")` |

6. Send back the table, the output of step 2, and the log of each failed row.
