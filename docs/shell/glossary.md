# Shell glossary

**Goal:** find the name of a shell item and the chapter that documents it.
**Scope:** every public item of `bk_shell` and `ctshell`, with its SwiftUI,
AppKit and UIKit equivalent where one exists. The kit's names are in
[20. Glossary](../barrierkit/20-glossary.md).
**Reader:** whoever writes or reads Malaysia's shell, compositor, portal or
System Settings.
**Input:** a name. **Output:** its counterpart and the chapter to read.

## Steps

1. Search this page for the name, or for the SwiftUI or AppKit name you know.
2. Follow the chapter link of its row.
3. To add or rename a public item, name it by
   [How names are chosen](../barrierkit/20-glossary.md#how-names-are-chosen)
   and add its row here in the same commit. A moved or renamed item also gets
   a row in [Renamed items](../barrierkit/20-glossary.md#renamed-items).

## bk-shell

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `bk_shell::run_layer` | fn | — | — | — | Runs an app in a wlr layer surface (menu bar, Dock, background) | [3](03-layer-surfaces.md) |
| `bk_shell::LayerSpec` | struct | — | — | — | Namespace, layer, anchors, exclusive zone, keyboard and size of a layer surface | [3](03-layer-surfaces.md) |
| `bk_shell::LayerGeometry` | struct | — | ≈ `NSWindow.setFrame(_:display:)` | — | Anchors, exclusive zone, size and keyboard of a running layer surface | [3](03-layer-surfaces.md) |
| `bk_shell::CardFrames` | struct | — | — | — | Corner radius and rectangles of the cards a layer surface draws | [3](03-layer-surfaces.md) |
| `bk_shell::PlateFrames` | struct | — | — | — | Corner, as a share of each plate's shorter side, and rectangles of the icon plates a layer surface has the compositor draw as glass | [3](03-layer-surfaces.md) |
| `bk_shell::set_plate_frames` | fn | — | — | — | Asks the compositor to draw the glass of each icon plate of the calling layer surface; only the Dock's is drawn | [3](03-layer-surfaces.md) |
| `bk_shell::set_card_frames` | fn | — | — | — | Asks the compositor for the window frame (shadow, inner and outer border) around each card of the calling layer surface | [3](03-layer-surfaces.md) |
| `bk_shell::set_layer_geometry` | fn | — | ≈ `NSWindow.setFrame(_:display:)` | — | Moves, resizes or refocuses the layer surface of the calling component | [3](03-layer-surfaces.md) |
| `bk_shell::Layer` | enum | — | ≈ `NSWindow.Level` | — | Stacking layer of a layer surface: `Background`, `Bottom`, `Top`, `Overlay` | [3](03-layer-surfaces.md) |
| `bk_shell::Anchors` | struct | — | — | — | Edges a layer surface sticks to; `TOP` and `BOTTOM` constants | [3](03-layer-surfaces.md) |
| `bk_shell::KeyboardInteractivity` | enum | — | — | — | Keyboard focus of a layer surface: `None`, `Exclusive`, `OnDemand` | [3](03-layer-surfaces.md) |
| `bk_shell::run_lock` | fn | — | — | — | Locks the session and draws an app on every output | [5](05-lock-screen.md) |
| `bk_shell::SessionUnlock` | struct | — | — | — | Handle that ends the session lock | [5](05-lock-screen.md) |
| `bk_shell::SessionUnlock::get`, `unlock`, `locked` | method | — | — | — | The handle of the running lock; ends the lock; whether the compositor has confirmed it | [5](05-lock-screen.md) |
| `bk_shell::SessionUnlock::keyboard_layout` | method | — | ≈ `TISCopyCurrentKeyboardInputSource` | — | Name of the active keyboard layout, from the compositor's keymap | [5](05-lock-screen.md) |
| `bk_shell::ScreenLock` | enum | — | — | — | Whether the session is locking, and for how long the fade lasts | [5](05-lock-screen.md) |
| `bk_shell::use_screen_lock` | fn | — | — | — | Hook that follows the session lock (`infinity_session_v1`) | [5](05-lock-screen.md) |
| `bk_shell::BlurRegion` | struct | `.background(.ultraThinMaterial)` | `NSVisualEffectView` | `UIVisualEffectView` | Rectangles the compositor blurs behind a surface | [7](07-glass-and-blur.md) |
| `bk_shell::set_blur_region` | fn | — | — | — | Asks the compositor to blur behind the calling surface (`ext_background_effect_v1`) | [7](07-glass-and-blur.md) |
| `bk_shell::InputRegion` | struct | `.contentShape(_:)` | `NSView.hitTest(_:)` | `UIView.hitTest(_:with:)` | Rectangles where a surface takes the pointer | [3](03-layer-surfaces.md) |
| `bk_shell::set_input_region` | fn | — | — | — | Limits where the calling surface takes the pointer (`wl_surface.set_input_region`) | [3](03-layer-surfaces.md) |
| `bk_shell::Toplevels` | struct | — | — | — | Every app's windows, the shown space and desktop, and actions on windows | [4](04-desktop-windows.md) |
| `bk_shell::Toplevels::get`, `activate`, `close`, `minimize`, `unset_minimized`, `set_maximized`, `arrange`, `arrange_pair`, `arrange_quarters`, `restore`, `set_fullscreen`, `force_quit`, `send_shortcut` | method | — | — | — | The window list of the running surface, and the actions on a window by its id | [4](04-desktop-windows.md) |
| `bk_shell::MinimizeTarget` | struct | — | `NSDockTile` | — | A window and the rectangle it minimizes to | [4](04-desktop-windows.md) |
| `bk_shell::capture_window` | fn | — | `CGWindowListCreateImage` | — | Copies a window's pixels once (`ext-image-copy-capture-v1`) | [4](04-desktop-windows.md) |
| `bk_shell::WindowImage` | struct | — | `CGImage` | `CGImage` | A window's pixels: width, height and RGBA bytes | [4](04-desktop-windows.md) |
| `bk_shell::CaptureWindow` | struct | — | — | — | The request `capture_window` sends to the runner | [4](04-desktop-windows.md) |
| `bk_shell::set_minimize_target` | fn | — | — | — | Tells the compositor where a window minimizes to (`set_rectangle`) | [4](04-desktop-windows.md) |
| `bk_shell::ToplevelInfo` | struct | — | — | — | One window of any app: id, title, app id, state, menu addresses | [4](04-desktop-windows.md) |
| `bk_shell::FocusBar` | enum | — | — | — | The bar over the shown space: `None`, `Hidden`, `Revealed` | [4](04-desktop-windows.md) |
| `bk_shell::FocusBar::over_focus` | method | — | — | — | Whether the shown space is a focus space | [4](04-desktop-windows.md) |
| `bk_shell::GtkMenusAddress` | struct | — | — | — | D-Bus address of a GTK window's global menu | [4](04-desktop-windows.md) |
| `bk_shell::InfinityLayout` | enum | — | — | — | Placement for `Toplevels::arrange`: fill, center, halves, quarters | [4](04-desktop-windows.md) |
| `bk_shell::InfinityPairLayout` | enum | — | — | — | Pair tiling: `SideBySide`, `SideBySideReversed`, `Stacked`, `StackedReversed` | [4](04-desktop-windows.md) |
| `bk_shell::InfinityWindowManagerV1` | struct | — | — | — | Proxy of Malaysia's private window-management protocol, used by `Toplevels` | [4](04-desktop-windows.md) |
| `bk_shell::Activation` | struct | — | — | — | Requests xdg-activation tokens for the apps the shell launches | [4](04-desktop-windows.md) |
| `bk_shell::Activation::get`, `request` | method | — | — | — | The activation handle; asks for a token tied to the last input event | [4](04-desktop-windows.md) |
| `bk_shell::use_status_items` | fn | — | — | — | Hook that hosts other apps' StatusNotifierItem icons and menus | [6](06-status-items.md) |
| `bk_shell::TrayItem` | struct | — | ≈ `NSStatusItem` | — | Another app's status item, as the host reads it | [6](06-status-items.md) |
| `bk_shell::StatusHost` | struct | — | — | — | Sends activate, secondary activate and menu clicks to status items | [6](06-status-items.md) |
| `bk_shell::StatusHost::activate`, `secondary_activate`, `click` | method | — | — | — | Sends `Activate`, `SecondaryActivate` or a dbusmenu `clicked` event to an item | [6](06-status-items.md) |
| `bk_shell::Pixmap` | type | — | ≈ `NSBitmapImageRep` | ≈ `UIImage` | RGBA bitmap of a status item's icon: a `WindowImage` | [6](06-status-items.md) |
| `bk_shell::use_shell_theme` | fn | — | — | — | Hook: Freya component theme from the resolved shell colors | [7](07-glass-and-blur.md) |
| `bk_shell::for_scheme` | fn | — | — | — | Freya theme for the default light or dark appearance | [7](07-glass-and-blur.md) |

## Settings files

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctshell::wallpaper::wallpaper_selection_path` | fn | — | — | — | Path of `wallpaper-selection.state`, the wallpaper each desktop shows | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::read_wallpaper_selection` | fn | — | — | — | Reads the selection; the default without a file, `Err` on bad TOML | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::write_wallpaper_selection` | fn | — | — | — | Writes the selection atomically | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperSelection` | struct | — | ≈ `NSWorkspace.desktopImageURL(for:)` and its options | — | `wallpaper-selection.state`: sources, scales, crops, dynamic appearances and paused animations per desktop and output, and `shuffle` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperSelection::replace_source` | method | — | — | — | Rewrites the desktops that show one source, or takes it from them, and leaves the others | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperSelection::show_on_all_desktops` | method | — | ≈ «Show on all Spaces» | — | Makes a desktop's wallpaper, framing, appearance and pause the shared ones | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperSelection::has_own`, `own_desktops`, `make_own`, `source`, `set_source`, `scale_for`, `set_scale`, `crop_for`, `set_crop`, `appearance_for`, `set_appearance`, `paused_for`, `set_paused` | method | — | ≈ `NSWorkspace.setDesktopImageURL(_:for:options:)` | — | Which desktops have their own wallpaper, and the source, scale, crop per output, appearance and pause of a desktop or of all | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperScale` | enum | — | ≈ `NSImageScaling` | ≈ `UIView.ContentMode` | `Fill`, `Fit`, `Stretch`, `Center`, `Custom` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::ALL_DESKTOPS` | constant | — | — | — | Key `"default"`: the wallpaper and crop of every desktop | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::desktop_key` | fn | — | — | — | Key of a desktop in the wallpaper files: its number, or `ALL_DESKTOPS` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperSource` | enum | — | — | — | What a desktop shows: `BuiltIn`, `File`, `ShuffleColors`, `ShuffleCollection`; `parse` and `value` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperAppearance` | enum | — | ≈ `ColorScheme` | — | `Light` or `Dark`: the still a dynamic wallpaper keeps | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperShuffle` | enum | — | ≈ «Change picture» | — | How often a shuffle changes: `Login`, `Hourly`, `Daily`; `seconds` gives the interval, `None` at login | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperKind` | enum | — | — | — | `Image`, `Video` or `Shader`; `of` reads it from the file extension | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::wallpapers_in` | fn | — | — | — | Wallpaper files of a folder, sorted | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::system_wallpapers_dir` | fn | — | — | — | The first `$XDG_DATA_DIRS/wallpapers` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::custom_colors_dir` | fn | — | — | — | `data_dir()/wallpapers/colors`, the user's own color wallpapers | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::wallpaper_colors` | fn | — | — | — | The wallpaper colors: the system's `solid-colors`, then the user's | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperCollection` | struct | — | ≈ a Photos album as wallpaper | — | `name`, `folders` and `images`; `wallpapers()` lists them once | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::WallpaperConfig` | struct | — | — | — | Contents of `wallpaper.domain`: `collections`; `collection` finds one by name | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::wallpaper_config_path` | fn | — | — | — | Path of `wallpaper.domain` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::read_wallpaper_config` | fn | — | — | — | Reads the collections; the default without a file | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::write_wallpaper_config` | fn | — | — | — | Writes the collections atomically | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::thumbnail_path` | fn | — | — | — | The x-large thumbnail of a file in the freedesktop thumbnail cache | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::frame_path` | fn | — | — | — | The light or dark still of a dynamic wallpaper, in `~/.cache/malaysia/wallpaper-frames` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::thumbnail_is_fresh` | fn | — | — | — | Whether a thumbnail's `Thumb::MTime` matches its file | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::thumbnail_image_size` | fn | — | — | — | The original image size a thumbnail records in `Thumb::Image::Width` and `Height` | [8](08-desktop-settings.md) |
| `ctshell::wallpaper::modified_seconds` | fn | — | — | — | A file's modification time in seconds | [8](08-desktop-settings.md) |
| `ctshell::menubar::MenubarConfig` | struct | — | — | — | `menubar.domain`: `app_menus` the `show_*` status menus, `style` and `bar_height` | [8](08-desktop-settings.md) |
| `ctshell::menubar::MenubarConfig::height` | method | — | — | — | Menu bar height in logical pixels for `bar_height` | [8](08-desktop-settings.md) |
| `ctshell::menubar::colors_over` | function | — | — | — | `MenubarColors` of a translucent or floating menu bar over a wallpaper color: dark over light, light over dark | [8](08-desktop-settings.md) |
| `ctshell::menubar::MenubarColors` | struct | — | — | — | `ink`, `glass` and `selection` (pill behind an open menu title) of the menu bar | [8](08-desktop-settings.md) |
| `ctshell::menubar::classic_bar` | function | — | — | — | Fill of the Classic menu bar: the theme bar at `MENUBAR_CLASSIC_ALPHA`, opaque with reduced transparency | [8](08-desktop-settings.md) |
| `ctshell::menubar::MenubarStyle` | enum | — | — | — | `Solid`, `Translucent`, `Floating`: how the menu bar draws its background | [8](08-desktop-settings.md) |
| `ctshell::menubar::MenubarHeight` | enum | — | — | — | `Slim`, `Wide`: the menu bar height | [8](08-desktop-settings.md) |
| `ctshell::dock::DockConfig` | struct | — | `com.apple.dock` defaults | — | `dock.domain`: size, magnification, position, effect, hiding and what the Dock shows | [8](08-desktop-settings.md) |
| `ctshell::dock::DockConfig::tilesize`, `largesize` | method | — | — | — | Icon size clamped to 16–128, and the magnified size (at least `DOCK_MAGNIFY_MIN` times the icon size) | [8](08-desktop-settings.md) |
| `ctshell::dock::DockOrientation` | enum | — | `orientation` | — | `Left`, `Bottom`, `Right` | [8](08-desktop-settings.md) |
| `ctshell::dock::MinimizeEffect` | enum | — | `mineffect` | — | `Genie`, `Scale`, `Suck` | [8](08-desktop-settings.md) |
| `ctshell::notifications::NotificationsConfig` | struct | — | ≈ System Settings › Notifications | — | The desktop's notification settings: `show_previews`, `when_locked`, `apps`, `focus`, `widgets` | [8](08-desktop-settings.md) |
| `ctshell::notifications::NotificationsConfig::app`, `app_mut`, `focus`, `focus_mut` | method | — | — | — | An app's or a Focus's entry, or its default; the `_mut` ones insert the default first | [8](08-desktop-settings.md) |
| `ctshell::notifications::AppNotifications` | struct | — | ≈ `UNNotificationSettings` | ≈ `UNNotificationSettings` | One app's notification settings: where they show, style, sound, badge, grouping, mute | [8](08-desktop-settings.md) |
| `ctshell::notifications::AppNotifications::muted` | method | — | — | — | Whether `muted_until` is after a time | [8](08-desktop-settings.md) |
| `ctshell::notifications::ShowPreviews` | enum | — | ≈ `UNShowPreviewsSetting` | ≈ `UNShowPreviewsSetting` | `Always`, `WhenUnlocked`, `Never` | [8](08-desktop-settings.md) |
| `ctshell::notifications::AlertStyle` | enum | — | ≈ `UNAlertStyle` | ≈ `UNAlertStyle` | `Temporary` (banner) or `Persistent` (alert) | [8](08-desktop-settings.md) |
| `ctshell::notifications::Grouping` | enum | — | — | — | `Automatic` or `Off` | [8](08-desktop-settings.md) |
| `ctshell::notifications::Focus`, `Focus::ALL` | enum, constant | — | ≈ a Focus of System Settings | — | `DoNotDisturb`, `Sleep`, `Personal`, `Work` | [8](08-desktop-settings.md) |
| `ctshell::notifications::FocusSettings` | struct | — | — | — | The apps a Focus allows, and whether time-sensitive notifications pass | [8](08-desktop-settings.md) |
| `ctshell::notifications::Widget`, `Widget::ALL` | enum, constant | — | — | — | A widget of Notification Center: `Calendar`, `Batteries` | [8](08-desktop-settings.md) |

## Glass and traffic lights

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctshell::resolve::glass_lightness`, `glass_rim_lift`, `glass_blur_passes`, `glass_edge`, `GlassEdge`, `glass_shadow`, `GlassShadow` | fn, struct | — | — | — | The values of `ctdesign::GLASS` for a scheme: the lightness that shows through, the rim, the blur passes, the edge and the shadow | [7](07-glass-and-blur.md) |
| `ctshell::shell_glass::over`, `tone`, `chroma` | fn | — | — | — | The colour a `ShellGlass` shows over a blurred backdrop pixel in device values, and its luma tone and chroma gain | [7](07-glass-and-blur.md) |
| `ctshell::shell_glass::kawase_sigma` | fn | — | — | — | The Gaussian sigma of a Dual Kawase of given passes and offset | [7](07-glass-and-blur.md) |
| `ctshell::liquid_glass::Glass`, `GlassState`, `looks`, `look`, `adapted` | enum, enum, fn | `Glass` | `NSGlassEffectView.Style` | `UIGlassEffect` | Regular or clear; active, inactive, reduced, increased contrast or both; the set of a style in a scheme; one look of it; the look for a shape of a given shorter side | [7](07-glass-and-blur.md) |
| `ctshell::liquid_glass::face`, `blur_sigma`, `lens_offset`, `rim_levels`, `smooth_union`, `half_resolution`, `rounded_rect_distance` | fn | ≈ `glassEffect(_:in:)` | ≈ `NSGlassEffectContainerView` | ≈ `UIGlassContainerEffect` | The colour the glass shows over one sample; the Gaussian sigma at a depth; how far inward the lens reads; the levels the rim and shadow add; the union of two shapes of a container; the read of a line through a box of 2, 4 or 8 pixels; the signed distance and normal of a rounded rectangle | [7](07-glass-and-blur.md) |
| `ctshell::liquid_glass::Image`, `Image::new`, `Image::half_resolution`, `Shape`, `render` | struct, struct, fn | — | — | — | The CPU reference: an image, a rounded rectangle and the window as macOS shows it with the glass over the backdrop | [7](07-glass-and-blur.md) |
| `ctshell::design::GlassTint`, `LIQUID_GLASS_TINT_REGULAR_LIGHT`, `_DARK`, `LIQUID_GLASS_TINT_CLEAR_LIGHT`, `_DARK`, `ctshell::liquid_glass::tint`, `tinted`, `Tinted`, `Tinted::untinted`, `ctappearance::AppearanceConfig::liquid_glass_tint` | struct, const, fn, config key | ≈ `NSGlassTintAmount` | ≈ the Liquid Glass slider of System Settings › Appearance | — | The knots, read scale thresholds and tone shifts of the slider per style and scheme; the active look at a position (the look, the read factor, the blur fill opacity and sigma); the key, 0 to 1, default 0 | [7](07-glass-and-blur.md) |
| `ctshell::material::blur_region` | fn | — | — | — | A Gaussian over one line of a region that sees only itself | [7](07-glass-and-blur.md) |
| `ctshell::menu::panel_over` | fn | — | — | — | The colour a menu's glass shows over a backdrop, as the compositor's shader computes it | [7](07-glass-and-blur.md) |
| `ctshell::traffic::group_rect` | fn | — | — | — | Rectangle of the three lights | [9](09-shell-tokens.md) |
| `ctshell::traffic::glyph_svg` | fn | — | — | — | SVG of a glyph | [9](09-shell-tokens.md) |

## Tokens: `ctshell::design`

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctshell::design::dock_divider` | fn | — | — | — | The Dock's divider over a backdrop colour: a white or black overlay that moves the glass's luminance by `DockSurface::divider` | [9](09-shell-tokens.md) |
| `ctshell::design::CIRCLE_EXPONENT` | constant | — | — | — | The superellipse exponent of a circle, 2 | [9](09-shell-tokens.md) |
| `ctshell::design::ShellGlass`, `SHELL_GLASS_<SURFACE>_DARK`, `SHELL_GLASS_LEVELS`, `SHELL_GLASS_CHROMA_KNOTS` | struct, const | — | — | — | The glass of a desktop surface in a scheme (blur `sigma`, wide `halo_sigma` and `halo_weight`, 12-point luma `tone`, 5-point `chroma` gain) and the levels and knots its tables are read at; `menu`, `spotlight`, `apps`, `notification-card`, `control-center`, `control-center-panel`, `dock` and `spaces-bar` (a `MaterialLook`), dark | [7](07-glass-and-blur.md) |
| `ctshell::design::GlassLook`, `GlassSet`, `GlassRim`, `GlassScale`, `GlassLens`, `RIM_BINS` | struct, const | ≈ `Glass` | ≈ `NSGlassEffectView` | ≈ `UIGlassEffect` | One look of a glass (blur, blur shares by depth, `white`, `black`, `saturation`, `max_luma`, `fill`, `refraction`, `tone_delta`), the five looks of a glass in a scheme (active, inactive, Reduce Transparency, Increase Contrast, both) with its `rim` tables and `scale` rules, and the lens profile | [7](07-glass-and-blur.md) |
| `ctshell::design::LIQUID_GLASS_REGULAR_LIGHT`, `_DARK`, `LIQUID_GLASS_CLEAR_LIGHT`, `_DARK`, `LIQUID_GLASS_LENS`, `LIQUID_GLASS_BLUR_CAP`, `LIQUID_GLASS_UNION` | constant | ≈ `Glass.regular`, `Glass.clear` | ≈ `NSGlassEffectView.Style` | ≈ `UIGlassEffect.Style.regular`, ≈ `UIGlassEffect.Style.clear` | The `GlassSet` of each style and scheme, the lens, the widest blur in pixels and the share of the container spacing that blends two shapes | [7](07-glass-and-blur.md) |
| `ctshell::design::WINDOW_EXPAND_MIN_MS`, `WINDOW_EXPAND_MAX_MS` | constant | — | — | — | Compositor: shortest and longest time a window takes to maximize or restore: 300, 560 ms | [9](09-shell-tokens.md) |
| `ctshell::design::ALERT_LAYER_NAMESPACE`, `NOTIFICATIONS_LAYER_NAMESPACE`, `MENUBAR_LAYER_NAMESPACE`, `DOCK_LAYER_NAMESPACE`, `WALLPAPER_LAYER_NAMESPACE` | constant | — | — | — | Layer namespaces of shell alerts, notifications, the menu bar, the Dock and the wallpaper | [9](09-shell-tokens.md) |
| `ctshell::design::NOTIFICATION_WIDTH`, `NOTIFICATION_RADIUS`, `NOTIFICATION_PADDING`, `NOTIFICATION_GAP`, `NOTIFICATION_MARGIN`, `NOTIFICATION_ICON_SIZE`, `NOTIFICATION_TEXT_GAP`, `NOTIFICATION_CLOSE_SIZE`, `NOTIFICATION_TITLE_FONT`, `NOTIFICATION_SHADOW`, `NOTIFICATION_SHADOW_BLUR`, `NOTIFICATION_VEIL_SPREAD`, `NOTIFICATION_EMPTY_TOP`, `NOTIFICATION_EMPTY_GAP`, `NOTIFICATION_STACK_PEEK`, `NOTIFICATION_STACK_INSET`, `NOTIFICATION_BODY_LINES`, `NOTIFICATION_MAX_BANNERS`, `NOTIFICATION_BANNER_MS`, `NOTIFICATION_SLIDE_IN_MS`, `NOTIFICATION_SLIDE_OUT_MS`, `NOTIFICATION_SLIDE_BOUNCE`, `NOTIFICATION_EDGE`, `NOTIFICATION_SWIPE`, `LIVE_ACTIVITY_WIDTH`, `LIVE_ACTIVITY_HEIGHT`, `LIVE_ACTIVITY_RADIUS`, `LIVE_ACTIVITY_GAP`, `LIVE_ACTIVITY_ENDED_MS` | constant | — | — | — | Notification banner, Notification Center and Live Activity sizes, timing and slides | [9](09-shell-tokens.md) |
| `ctshell::design::LAUNCHER_WIDTH`, `LAUNCHER_HEIGHT`, `LAUNCHER_PADDING`, `LAUNCHER_SEARCH_HEIGHT`, `LAUNCHER_RESULT_HEIGHT`, `LAUNCHER_RESULT_GAP`, `LAUNCHER_RESULT_RADIUS`, `LAUNCHER_ICON_SIZE`, `LAUNCHER_MAX_RESULTS`, `LAUNCHER_CORNER_RADIUS`, `LAUNCHER_FONT_SIZE`, `LAUNCHER_SUBTITLE_FONT_SIZE`, `LAUNCHER_SCALE_FROM`, `LAUNCHER_OPEN_MS`, `LAUNCHER_CLOSE_MS` | constant | — | — | — | Shell launcher sizes and animation | [9](09-shell-tokens.md) |
| `ctshell::design::LOCK_CLOCK_FONT_SIZE`, `LOCK_DATE_FONT_SIZE`, `LOCK_TOP_PERCENT`, `LOCK_BOTTOM_MARGIN`, `LOCK_SPACING`, `LOCK_AVATAR_SIZE`, `LOCK_KEYBOARD_WIDTH`, `LOCK_FIELD_HEIGHT`, `LOCK_FIELD_WIDTH`, `LOCK_MASK_FONT_SIZE`, `LOCK_INK`, `LOCK_FIELD_FILL`, `LOCK_AVATAR_FILL`, `LOCK_FIELD_ERROR`, `LOCK_FIELD_INK` | constant | — | — | — | Lock screen sizes and colors over the wallpaper | [9](09-shell-tokens.md) |
| `ctshell::design::LOCK_FADE_MS`, `LOCK_SETTLE_MS`, `LOCK_RISE_MS`, `LOCK_RISE_OFFSET`, `LOCK_LOGIN_MS` | constant | — | — | — | Timing of the transition from the desktop to the lock screen | [9](09-shell-tokens.md) |
| `ctshell::design::ABOUT_WIDTH`, `ABOUT_HEIGHT`, `ABOUT_DRAWING_TOP`, `ABOUT_DRAWING_WIDTH`, `ABOUT_DRAWING_HEIGHT`, `ABOUT_TITLE_GAP`, `ABOUT_TITLE_SIZE`, `ABOUT_SPECS_GAP`, `ABOUT_SPEC_ROW_GAP`, `ABOUT_COLUMN_GAP`, `ABOUT_VALUE_WIDTH`, `ABOUT_BUTTON_GAP`, `ABOUT_FOOTER_BOTTOM` | constant | — | — | — | Sizes of the shell's «About this computer» window | [9](09-shell-tokens.md) |
| `ctshell::design::ABOUT_APP_WIDTH`, `ABOUT_APP_HEIGHT`, `ABOUT_APP_ICON` | constant | — | — | — | Size of the shell's «About ‹app›» window and of its icon | [9](09-shell-tokens.md) |
| `ctshell::design::CONFIRM_WIDTH`, `CONFIRM_HEIGHT`, `CONFIRM_ICON`, `CONFIRM_GLYPH` | constant | — | — | — | Sizes of the shell's restart, shut down and log out confirmation | [9](09-shell-tokens.md) |
| `ctshell::design::FORCE_QUIT_WIDTH`, `FORCE_QUIT_HEIGHT`, `FORCE_QUIT_ROW_HEIGHT`, `FORCE_QUIT_ICON` | constant | — | — | — | Sizes of the shell's «Force Quit» window | [9](09-shell-tokens.md) |
| `ctshell::design::MENUBAR_BATTERY_ICON`, `MENUBAR_BATTERY_LOW`, `MENUBAR_BATTERY_TRACK`, `MENUBAR_BATTERY_BODY_WIDTH`, `MENUBAR_BATTERY_BODY_HEIGHT`, `MENUBAR_BATTERY_DIGIT_PITCH`, `MENUBAR_BATTERY_DIGIT_HEIGHT`, `MENUBAR_BATTERY_HUNDRED_HEIGHT`, `MENUBAR_BATTERY_HUNDRED_GAP`, `MENUBAR_BATTERY_BOLT_GAP`, `MENUBAR_BATTERY_LEVEL_MIN` | constant | — | — | — | Size of the menu bar battery, the percentage from which it is red, the opacity of its empty part and the metrics of its knocked-out glyphs | [9](09-shell-tokens.md) |
| `ctshell::design::CURSOR_SHAKE_SCALE`, `CURSOR_SHAKE_GROW_MS`, `CURSOR_SHAKE_SHRINK_MS`, `CURSOR_SHAKE_TICK_MS` | constant | — | — | — | Compositor: cursor growth when the pointer is shaken | [9](09-shell-tokens.md) |
| `ctshell::design::CURSOR_SHAKE_MIN_DX`, `CURSOR_SHAKE_MIN_SWING`, `CURSOR_SHAKE_MAX_SWING_MS`, `CURSOR_SHAKE_MAX_TOTAL_MS`, `CURSOR_SHAKE_REVERSALS`, `CURSOR_SHAKE_ACTIVE_MS` | constant | — | — | — | Compositor: the distances, times and reversals that count as a shake | [9](09-shell-tokens.md) |
| `ctshell::design::BACKGROUND_BLUR_PASSES`, `BACKGROUND_BLUR_OFFSET` | constant | — | ≈ `NSVisualEffectView` blur | ≈ `UIVisualEffectView` | Compositor: background blur strength (malaysia ADR 0071) | [9](09-shell-tokens.md) |
| `ctshell::design::SCREENSHOT_DIM`, `SCREENSHOT_IDLE_DIM`, `SCREENSHOT_BORDER`, `SCREENSHOT_BORDER_WIDTH` | constant | — | — | — | Compositor: screenshot selection overlay | [9](09-shell-tokens.md) |
| `ctshell::design::SPACE_GAP`, `SPACE_GAP_COLOR`, `SPACE_SLIDE_MS`, `SPACE_SLIDE_EASE` | constant | — | — | — | Compositor: gap and slide between Spaces | [9](09-shell-tokens.md) |
| `ctshell::design::TILE_MARGIN`, `TILE_GAP`, `DESKTOP_CLEAR` | constant | — | — | — | Compositor: tiled window margins; color behind everything | [9](09-shell-tokens.md) |
| `ctshell::design::WALLPAPER_FPS`, `WALLPAPER_THUMBNAIL`, `WALLPAPER_FRAME` | constant | — | — | — | Wallpaper: animation frame rate, thumbnail size, dynamic still size | [9](09-shell-tokens.md) |
| `ctshell::design::CORNER_NOTIFICATION`, `CORNER_NOTIFICATION_CENTER`, `CORNER_CONTROL_CENTER_MODULE`, `CORNER_SPOTLIGHT` | constant | — | `NSView.effectiveCornerRadii` | — | The measured radius and curve of a banner, Notification Center panel, Control Center module and Spotlight panel | [9](09-shell-tokens.md) |
| `ctshell::design::GLASS_MASK` | constant | — | — | — | Base color of the mask that fades glass bars | [9](09-shell-tokens.md) |
| `ctshell::design::MENUBAR_HEIGHT_WIDE`, `MENUBAR_DARK_INK_ABOVE`, `MENUBAR_CLASSIC_ALPHA`, `MENUBAR_OVER_LIGHT_INK`, `MENUBAR_OVER_LIGHT_GLASS`, `MENUBAR_OVER_LIGHT_SELECTION`, `MENUBAR_OVER_DARK_INK`, `MENUBAR_OVER_DARK_GLASS`, `MENUBAR_OVER_DARK_SELECTION`, `MENUBAR_FONT_SIZE`, `MENUBAR_FONT_SIZE_LARGE`, `MENUBAR_APP_NAME_WEIGHT`, `MENUBAR_MARGIN`, `MENUBAR_MARGIN_RIGHT`, `MENUBAR_FADED_ALPHA`, `MENUBAR_TITLE_PADDING_X`, `MENUBAR_TEXT_LIFT`, `MENUBAR_TITLE_MAX_WIDTH`, `MENUBAR_SYSTEM_MENU_MIN_WIDTH`, `MENUBAR_TITLE_BLEED`, `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | constant | — | — | — | Shell menu bar sizes; focus-mode bar colors | [9](09-shell-tokens.md) |
| `ctshell::design::DOCK_*` | constant | — | — | — | Dock sizes and timings, the name label's text and the menu gap; the Dock's frame, label corner and menu shapes stay in `ctdesign` | [9](09-shell-tokens.md) |
