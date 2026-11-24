# 8. Desktop settings

**Goal:** read, change and follow the desktop's own settings files.
**Scope:** the files of the Dock, the menu bar, the notifications, and the
wallpaper catalog and selection; their types in `ctshell`; and
`ctappearance::watch`. The files an app reads too, `appearance.domain`,
`window.domain` and `wallpaper.state`, are in
[4. Appearance and tokens](../barrierkit/04-appearance-and-tokens.md#user-appearance-ctappearance).
Malaysia's [options catalog](https://github.com/HBlanqueto/malaysia/blob/main/docs/configuration/options.md)
says what each option does on the desktop.
**Reader:** whoever works on Malaysia's shell, compositor, wallpaper or
System Settings.
**Input:** `ctappearance::config_dir()` and `data_dir()`
([4](../barrierkit/04-appearance-and-tokens.md#reading-and-writing)).
**Output:** typed settings, written back atomically and read again when
they change.

## Steps

1. Find the file, its type and its readers in [The files](#the-files).
2. Read it. A wallpaper file has its own reader; the others parse with `toml`,
   and a missing file is the default:

   ```rust
   let path = config_dir().join("dock.domain");
   let text = std::fs::read_to_string(&path).unwrap_or_default();
   let mut dock: DockConfig = toml::from_str(&text)?;
   ```

3. Change a field and write the file back:

   ```rust
   dock.tilesize = 48;
   ctappearance::write_toml(&path, &dock)?;
   ```

4. Watch the folders with `AppearanceWatch::new()`, put it in the event loop,
   and read the file again when its event comes out of `drain()`
   ([Watching](#watching)).

## The files

| File | Type | Written by | Read by |
|---|---|---|---|
| `config_dir()/dock.domain` | `dock::DockConfig` | System Settings › Desktop & Dock; the Dock (`tilesize` when its separator is dragged, `show-downloads` from the separator's menu) | The Dock; the compositor (`mineffect`) |
| `config_dir()/menubar.domain` | `menubar::MenubarConfig` | System Settings: each `show_*` in its own pane, `style` and `bar_height` in Desktop & Dock | The menu bar and the lock screen; the compositor (`app_menus`) |
| `config_dir()/notifications.domain` | `notifications::NotificationsConfig` | System Settings › Notifications; «Mute for 1 Hour» and «Mute for Today» (`muted_until`) | `malaysia-shell notifications`, for every notification; the lock screen; the Dock (`badge`) |
| `config_dir()/wallpaper.domain` | `wallpaper::WallpaperConfig` | System Settings › Wallpaper («Add…», «Edit…») | `malaysia-wallpaper` (shuffle); System Settings |
| `data_dir()/wallpaper-selection.state` | `wallpaper::WallpaperSelection` | System Settings › Wallpaper | `malaysia-wallpaper` |

The writers and readers are Malaysia's
([options](https://github.com/HBlanqueto/malaysia/blob/main/docs/configuration/options.md)).
Each reader applies a change live, unless the catalog says otherwise.

### Reading and writing

| Files | Read | Write |
|---|---|---|
| `dock.domain`, `menubar.domain`, `notifications.domain` | `toml::from_str` | `ctappearance::write_toml(path, &config)` |
| `wallpaper.domain` | `read_wallpaper_config(&wallpaper_config_path())` | `write_wallpaper_config(path, &config)` |
| `wallpaper-selection.state` | `read_wallpaper_selection(&wallpaper_selection_path())` | `write_wallpaper_selection(path, &selection)` |

- The three `.domain` types take `#[serde(default)]`: a missing key takes its
  default, and an invalid enum value takes its default and logs a warning
  (`ctappearance::lenient`). A value of the wrong type fails the parse.
- `read_wallpaper_config` and `read_wallpaper_selection` return the default
  when the file cannot be read, and `Err` when it does not parse.
- Every writer goes through `write_toml`: it writes the whole value to a
  temporary file and renames it over the file. A file there that does not
  parse is first copied to `<name>.bak`. Comments and keys the type does not
  know are not kept, unlike `write_appearance`.

## Dock

`DockConfig` is `dock.domain`. Its keys are those of
`defaults write com.apple.dock`, except `show-downloads`: macOS keeps the
Downloads folder in `persistent-others`.

| Key | Type | Default |
|---|---|---|
| `autohide` | bool | `false` |
| `orientation` | `DockOrientation`: `left`, `bottom`, `right` | `bottom` |
| `magnification` | bool | `false` |
| `tilesize` | integer, px | `DOCK_TILESIZE` (64) |
| `largesize` | integer, px | `DOCK_LARGESIZE` (64) |
| `mineffect` | `MinimizeEffect`: `genie`, `scale`, `suck` | `genie` |
| `show-process-indicators` | bool | `true` |
| `static-only` | bool | `false` |
| `minimize-to-application` | bool | `false` |
| `show-recents` | bool | `true` |
| `show-downloads` | bool | `true` |
| `launchanim` | bool | `true` |
| `autohide-delay` | seconds | `0.0` |
| `autohide-time-modifier` | factor | `1.0` |

```toml
# ~/.config/malaysia/dock.domain
autohide = true
orientation = "left"
tilesize = 48
mineffect = "scale"
```

Read the sizes through the methods, not the fields:

| Method | Returns |
|---|---|
| `tilesize()` | `tilesize` clamped to `DOCK_TILESIZE_MIN` (16) and `DOCK_TILESIZE_MAX` (128) |
| `largesize()` | With `magnification`, `largesize` clamped the same way and at least `DOCK_MAGNIFY_MIN` (1.3) times `tilesize()`; without it, `tilesize()` |

## Menu bar

`MenubarConfig` is `menubar.domain`:

| Key | Type | Default |
|---|---|---|
| `app_menus` | bool: the active app's own menus on the bar | `true` |
| `show_wifi`, `show_bluetooth`, `show_battery`, `show_battery_percentage`, `show_vpn` | bool: the status menus of the bar | `true` |
| `show_printer` | bool: a printer while a print job is queued, on the bar and the lock screen | `true` |
| `style` | `MenubarStyle`: `solid` (Classic), `translucent`, `floating` (a clear bar; only the open title's pill is blurred) | `floating` |
| `bar_height` | `MenubarHeight`: `slim`, `wide` | `slim` |

`height()` returns the bar's height in logical pixels: `MENUBAR_HEIGHT` (30)
when slim, `MENUBAR_HEIGHT_WIDE` (38) when wide. The bar and the lock screen
both use it.

The `style` picks the bar's colors:

| Style | Fill and ink |
|---|---|
| `solid` | `classic_bar(theme_bar, reduce_transparency)`: the theme's `shell-bar` at `MENUBAR_CLASSIC_ALPHA` (0.7), opaque with transparency reduced |
| `translucent`, `floating` | `colors_over(wallpaper)` returns `MenubarColors`: `ink`, `glass` and `selection`, the pill behind an open menu title |

`colors_over` darkens the wallpaper color by the alpha of
`MENUBAR_OVER_LIGHT_GLASS`. Above
`MENUBAR_DARK_INK_ABOVE` (0.25) relative luminance it takes the
`MENUBAR_OVER_LIGHT_*` set, otherwise `MENUBAR_OVER_DARK_*`, whatever the
color scheme. `WallpaperGrid::across(output, rect)` gives the wallpaper colors
along the bar, blurred over `MENUBAR_TINT_SPREAD` of its width.

## Notifications

`NotificationsConfig` is `notifications.domain`:

| Key | Type | Default |
|---|---|---|
| `show_previews` | `ShowPreviews`: `always`, `when-unlocked`, `never` | `always` |
| `when_locked` | bool | `true` |
| `apps` | table of app id to `AppNotifications` | empty |
| `focus` | table of `Focus` (`do-not-disturb`, `sleep`, `personal`, `work`) to `FocusSettings` | empty |
| `widgets` | list of `Widget`: `calendar`, `batteries` | both |

The app id is the desktop file id without `.desktop`. An invalid key in
`focus` empties the whole table, with a warning.

| `AppNotifications` key | Type | Default |
|---|---|---|
| `allow`, `desktop`, `center`, `lock_screen` | bool | `true` |
| `style` | `AlertStyle`: `temporary`, `persistent` | `temporary` |
| `critical` | bool | `false` |
| `time_sensitive`, `sound`, `badge` | bool | `true` |
| `grouping` | `Grouping`: `automatic`, `off` | `automatic` |
| `muted_until` | Unix time, `Option<i64>` | absent |

`FocusSettings` has `allowed_apps` (empty) and `time_sensitive` (`true`).

```toml
# ~/.config/malaysia/notifications.domain
show_previews = "when-unlocked"

[apps."org.gnome.Calendar"]
style = "persistent"

[focus.work]
allowed_apps = ["org.gnome.Calendar"]
```

| Method | What it does |
|---|---|
| `app(id)`, `focus(focus)` | The entry, or its default |
| `app_mut(id)`, `focus_mut(focus)` | The entry to change; inserts the default first |
| `AppNotifications::muted(now)` | `true` while `muted_until` is after `now` |
| `Focus::ALL`, `Widget::ALL` | Every value, in order |

## Wallpaper catalog

`WallpaperConfig` is `wallpaper.domain`: the user's `collections`. A
`WallpaperCollection` has a `name`, `folders` and `images`.

```toml
# ~/.config/malaysia/wallpaper.domain
[[collections]]
name = "Viajes"
folders = ["/home/ana/Pictures/Viajes"]
images = ["/home/ana/Pictures/faro.jpg"]
```

`collection(name)` finds one. `WallpaperCollection::wallpapers()` lists the
wallpapers of its folders, then its images that exist, each once.

`ctshell::wallpaper` also finds the wallpapers on disk and their previews:

| Function | Returns |
|---|---|
| `WallpaperKind::of(path)` | `Image` (PNG, JPEG, HEIC, HEIF), `Video` (MP4, MOV, WebM, MKV), `Shader` (`.frag`, `.glsl`) or `None`, by extension |
| `wallpapers_in(folder)` | The wallpapers of a folder, sorted |
| `system_wallpapers_dir()` | The first `$XDG_DATA_DIRS/wallpapers` that exists |
| `custom_colors_dir()` | `data_dir()/wallpapers/colors`, the user's own colors |
| `wallpaper_colors()` | The system's `solid-colors`, then the user's colors |
| `thumbnail_path(file)` | `$XDG_CACHE_HOME/thumbnails/x-large/<md5>.png`, named by the MD5 of `file_uri(file)`, as the [thumbnail spec](https://specifications.freedesktop.org/thumbnail-spec/latest/) asks |
| `thumbnail_is_fresh(thumbnail, file)` | `true` while the thumbnail's `Thumb::MTime` equals the file's modification time |
| `thumbnail_image_size(thumbnail)` | The original's size, from `Thumb::Image::Width` and `Height` |
| `frame_path(file, appearance)` | `$XDG_CACHE_HOME/malaysia/wallpaper-frames/<md5>-light.png` or `-dark.png`: a still of a dynamic wallpaper |
| `modified_seconds(file)` | The file's modification time, in Unix seconds |

`malaysia-wallpaper` writes the thumbnails and stills, because System Settings
cannot decode HEIC, video or GLSL. System Settings treats a HEIC with both
stills as dynamic.

## Wallpaper selection

`WallpaperSelection` is `wallpaper-selection.state`: what each desktop shows
and how. Every map is keyed by `desktop_key(desktop)`: a desktop number from
1, or `ALL_DESKTOPS` (`"default"`) for every desktop. A desktop without its
own key shows the shared one, with its scale, crop, appearance and pause. A
crop can also be keyed `"<desktop>/<output>"`, which wins over the desktop's
own.

| Key | Holds | Default |
|---|---|---|
| `desktops` | A `WallpaperSource` as text: `builtin`, a `file://` URI, `shuffle:colors` or `shuffle:collection:<name>` | No key: the built-in wallpaper |
| `scale`, `scales` | The `WallpaperScale` of the shared wallpaper and of each desktop's own: `fill`, `fit`, `stretch`, `center`, `custom` | `fill` |
| `crop` | `[x, y, width, height]`, fractions of the image; with `custom` it covers the screen like `fill` | None |
| `appearance` | `WallpaperAppearance` (`light`, `dark`): the still a dynamic wallpaper keeps | None: it follows the system |
| `paused` | Desktop keys whose animated wallpaper stands still | None |
| `shuffle` | `WallpaperShuffle`: `login`, `hourly`, `daily`; `seconds()` is `None`, 3600 or 86400 | `hourly` |

```toml
# ~/.local/share/malaysia/wallpaper-selection.state
scale = "custom"
shuffle = "daily"

[desktops]
default = "file:///run/current-system/sw/share/wallpapers/bandura.jpg"
2 = "shuffle:collection:Viajes"

[crop]
default = [0.2, 0.15, 0.55, 0.62]
"default/HDMI-A-1" = [0.3, 0.1, 0.5, 0.62]
```

`WallpaperSource::parse(text)` and `value()` convert between the text and the
enum. Change the selection through its methods, so the keys stay consistent:

| Method | What it does |
|---|---|
| `source(desktop)`, `scale_for(desktop)`, `crop_for(desktop, output)`, `appearance_for(desktop)`, `paused_for(desktop)` | Read with the fallbacks above; `None` as `desktop` is the shared choice |
| `set_source(desktop, source)` | Sets a desktop's wallpaper and drops its crops, appearance and pause; `custom` turns to `fill`. With `None` as `desktop` it drops every desktop's own choices |
| `replace_source(old, new)` | Rewrites the desktops that show `old`; with `None` they lose it |
| `set_scale`, `set_crop`, `set_appearance`, `set_paused` | Write one choice of the desktop that owns it; `set_crop` also sets `custom` |
| `has_own(desktop)`, `own_desktops()` | Whether a desktop has its own key, and which do |
| `make_own(desktop)` | Gives a desktop a copy of the shared choices |
| `show_on_all_desktops(desktop)` | Makes a desktop's choices the shared ones, like macOS «Show on all Spaces» |

## Watching

On Linux, `ctappearance::watch::AppearanceWatch` watches `config_dir()` and
`data_dir()` with inotify ([4](../barrierkit/04-appearance-and-tokens.md#watching)):

1. `AppearanceWatch::new()` creates both folders, mode `0700`, if they are
   missing, and watches them.
2. The watch is a file descriptor (`AsFd`): add it to the event loop.
3. When it is readable, `drain()` returns the `AppearanceEvent`s of the files
   that changed.
4. Read the file of each event again.

| File | Event |
|---|---|
| `dock.domain`, `menubar.domain`, `notifications.domain`, `wallpaper.domain` | `Defaults(Some(name))`, with the file name |
| `wallpaper-selection.state` | `WallpaperSelection` |
| Any, after a lost event or a moved or deleted folder | Every event, with `Defaults(None)` |

```rust
let dock_changed = watch.drain()?.iter().any(|event| {
    matches!(event, AppearanceEvent::Defaults(name) if name.as_deref().is_none_or(|name| name == "dock.domain"))
});
```

Next: [9. Shell tokens](09-shell-tokens.md).
