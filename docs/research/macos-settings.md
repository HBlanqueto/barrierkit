# macOS 27 System Settings

**Goal:** record how System Settings is laid out and what each pane offers on
macOS 27.2.1, so Malaysia Settings can be made to match it option by option.
**Scope:** the window, the sidebar, the form chrome and the panes that matter
most (Appearance, Accessibility > Display, Desktop & Dock, Wallpaper,
Notifications, Wi-Fi, Bluetooth, Network, Energy, General), with the settings
that decide blur. It excludes the content of third-party panes, Apple Account,
Family, iCloud and Screen Time, and it changes no setting.
**Reader:** anyone who edits `malaysia-settings`, `bk::patterns::settings`, the
`settings` tokens of `ctdesign` or the appearance keys of `ctappearance`.
**Input:** a live macOS 27.2.1 session read through accessibility, raw window
captures and `defaults read`.
**Output:** the measured window, sidebar and form metrics, the options of each
pane in English and Spanish (es-419), the keys behind them and the list of what
was not measured.

## Steps

1. Read the method and its limits (section 1) before using a number.
2. Take the window, sidebar and form metrics (sections 2 to 4) for geometry.
3. Take a pane's options, names and values from its entry in section 5.
4. Take the keys and effects of the blur settings from section 6.
5. Check section 7 before calling something identical.

---

## 1. Environment and method

Measured on 2026-11-22.

| Item | Value |
|---|---|
| macOS | 27.2.1, build 26B131 |
| Display | 2560x1440 at 180 Hz, scale 1.0 (a point is a pixel) |
| Appearance during the pass | Dark; Reduce Transparency off; Increase Contrast off |
| System language | Spanish (es-419) |
| Window | one System Settings window at screen position (75, 331) |

Method, all read-only:

- **Structure and geometry:** `AXUIElement` through `~/src/probes/axdump`
  (role, subrole, title, description, value, identifier, position, size,
  enabled, selected, slider `AXMinValue` and `AXMaxValue`). It is the same
  tree `osascript` reaches through System Events, dumped natively because the
  SwiftUI panes are too large for System Events. `~/src/probes/axpress` presses a
  sidebar button by identifier; that only navigates.
- **Fonts:** the `AXFont` attribute of `AXAttributedStringForRange` of each
  static text (name and size). Headings, buttons, popups and the sidebar
  labels expose no font; their size is **inferred** from the height of their
  frame (13 pt gives 16, 11 pt gives 14, 15 pt gives 19).
- **Pixels:** `screencapture -x -R75,331,740,803` of the window, raw, never
  converted: the screen is wide gamut. The captures hold the user's names and
  networks and stay outside the repository.
- **English:** the host is started per process with `-AppleLanguages "(en)"`
  (`open -a "System Settings" --args -AppleLanguages "(en)"`); Spanish is the
  same launch without the argument. **Limit:** panes that live in an extension
  process (General, parts of Wallpaper, and every Apple Account and Family
  page) ignore the argument and keep the system language, so the English text
  of General is **not measured** here. No persistent language was changed.
- **Screen:** every walk pressed and captured only while no other session drove the screen.
- **Settings keys:** `defaults read` (never `write`). A key that does not exist
  means the setting is at its default; a key that exists was written by the
  user or by the system, so its value is **this Mac's value, not the default**.
  Defaults were not measured on a fresh account.

The window state is app state, not a macOS setting; navigating panes changes
only which pane System Settings reopens.

## 2. Window

| Property | English | Spanish (es-419) |
|---|---|---|
| Size | 740 x 803 | 865 x 803 |
| Sidebar width (splitter `value`) | 231, splitter 1 wide and disabled | 270 |
| Detail width | 508 (scroll area), cards 468 | 594, cards 554 |
| Title bar and toolbar height | 52 | 52 |
| Title | pane name, SF Pro Semibold 15, left aligned at 317 from the window edge | same |
| Back and Forward | a group 73 x 36 at (240, 8), two segments 36 and 37 wide | same |
| Traffic lights | 16 x 16 frames at (18, 18), (41, 18), (64, 18) from the window corner | same |
| Resizable | `AXSize` is settable; minimum and maximum were not measured | same |

The width differs by language (sidebar 231 against 270); whether the cause is
the longest label or the saved frame of this session was not determined. The
window title is the name of the selected pane.

## 3. Sidebar

- **Search field:** (10, 61) from the window corner, 212 x 28, with a `Search`
  button 25 wide inside. Placeholder `Search` / `Buscar`.
- **List:** a scroll area under the field, 232 x 705, starting 98 below the
  window top; a row is the whole width of the list.
- **Rows:** 32 high, stride 32. The label button is 24 high, inset 7 from the
  cell. The cell is 195 wide and starts 10 from the window edge. The selected
  row is the accent pill; the icon is a rounded tile of the pane's color.
- **Account row:** 46 high, avatar tile 38, name and `Apple Account` /
  `Cuenta de Apple` under it; a 32 row `Family` / `Familia` follows.
- **Group gap:** 13 between the last row of a group and the next group; a group
  has no title.
- **Order:** a fixed order in some groups and alphabetical by the **localized**
  label in others, with General first in its group. The ids are stable.

| Group | Fixed or sorted | Items (English), identifier `com.apple.settings.<id>` |
|---|---|---|
| 1 | fixed | `wifi` Wi-Fi, `bluetooth` Bluetooth, `network` Network, `vpn` VPN, `battery` Energy (Battery on a laptop) |
| 2 | General first, then sorted | `general` General, `accessibility` Accessibility, `appearance` Appearance, `desktopAndDock` Desktop & Dock, `displayAndBrightness` Displays, `controlCenter` Menu Bar, `siri` Siri, `search` Spotlight, `wallpaper` Wallpaper |
| 3 | fixed | `notifications` Notifications, `sounds` Sound, `focus` Focus, `screenTime` Screen Time |
| 4 | sorted | `lockScreen` Lock Screen, `privacyAndSecurity` Privacy & Security, `passcodeAndBiometrics` Touch ID & Password, `usersAndGroups` Users & Groups |
| 5 | sorted | `gameCenter` Game Center, `iCloud` iCloud, `internetAccounts` Internet Accounts, `walletAndApplePay` Wallet & Apple Pay |
| 6 | fixed | `keyboard` Keyboard, `mouse` Mouse, `gameControllers` Game Controllers, `printerAndScanner` Printers & Scanners |
| 7 | after the system ones | third-party panes, text only |

The Spanish names of the same ids: Wi-Fi, Bluetooth, Red, VPN, Energía,
General, Accesibilidad, Aspecto, Escritorio y Dock, Pantallas, Barra de menús,
Siri, Spotlight, Fondo de pantalla, Notificaciones, Sonido, Enfoque, Tiempo en
pantalla, Pantalla bloqueada, Privacidad y seguridad, Touch ID y contraseña,
Usuarios y grupos, Game Center, iCloud, Cuentas de Internet, Wallet y Apple Pay,
Teclado, Mouse, Controles de videojuegos, Impresoras y escáneres. The sorted
groups follow Spanish order (Accesibilidad, Aspecto, Barra de menús, Escritorio
y Dock, Fondo de pantalla, Pantallas, Siri, Spotlight). Spotlight is `search`,
and the pane called Menu Bar is `controlCenter`: there is no separate Control
Center pane in 27.2.1.

Panes that depend on hardware drop out: a Mac without a trackpad has no
Trackpad row, and a Mac without a battery shows Energy.

## 4. Form

Common to the detail panes (English window, 508 detail; add 86 to widths in
Spanish).

| Element | Value |
|---|---|
| Scroll area | x 232 from the window edge, below the toolbar; the vertical scroller is 19 wide, overlay |
| Content margin | 20 on both sides; a card is 468 wide at x 252 |
| Card inner padding | 10: text starts 10 inside the card and controls end 10 before its right edge |
| First card | 12 below the toolbar |
| Gap between cards | 10 |
| Gap before a heading | 30 from the previous card; 26 from the heading to the card under it |
| Heading | 16 high, 10 inside the card edge (inferred 13 pt Semibold) |
| Row with a switch | 36 high plus a 1 px divider; a group of four is 147 |
| Row with a popup button | about 38 (popup 24 high) |
| Row of a list of destinations (a button) | 44 in Accessibility, 48 in General, 52 in Network and Notifications, 36 in the Wi-Fi network lists; the divider is 1 |
| Label | SF Pro Regular 13, frame 16 |
| Secondary text | SF Pro Regular 11, frame 14 (two lines 28), under the label |
| Switch | 36 x 16, right edge at the control edge |
| Popup button | 24 high, width of its content, right aligned |
| Placard (icon, title, text) | text starts 50 from the card's left edge (icon size not measured); title Regular 13, text Regular 11 |
| Help button | 24 x 24 at the right edge, 10 below the last card |
| Window title | Semibold 15, frame 52 |
| Sub-pages | the toolbar shows the Back chevron and the sidebar keeps the parent selected |

`not measured`: card corner radius and fill, divider color and inset, switch
and popup fills, the sidebar label font, hover and pressed states of a row.

## 5. Panes

Values in the tables are the live values of this Mac unless the column says
otherwise. `key` names the preference behind an option where it was read.

### 5.1 Appearance (`appearance`)

Pane title `Appearance` / `Aspecto`. Cards in order:

1. **Appearance / Aspecto**: three cards 74 x 66: `Light` / `Claro`, `Dark` /
   `Obscuro`, `Auto` / `Automático`. Live: Dark. Help text of each:
   `Use a light appearance for buttons, menus, and windows.` and the same for
   dark; `Automatically adjusts the appearance of buttons, menus and windows
   throughout the day.` Key `AppleInterfaceStyle` (`Dark`, absent for Light);
   Auto is `AppleInterfaceStyleSwitchesAutomatically` (absent here).
2. **Liquid Glass** (same label in Spanish): a preview card 285 x 180 (a
   scrolling sample article with a glass toolbar) and a **slider** with
   accessibility name `Liquid Glass Tint Amount` / `Cantidad de tinte de Liquid
   Glass`, range 0 to 1 (`AXMinValue`, `AXMaxValue`), live value 0, three tick
   marks (left, middle, right). Left icon `Capsule Shape On A Rectangle With A
   Liquid Glass Effect`, right icon `Capsule Shape On A Rectangle`. Key
   `NSGlassTintAmount` (0 here; `NSGlassBuddyTintAmount` is 1 and
   `NSGlassEverEditedInBuddy` is 1). Its effect was measured per process on
   2026-11-22 ([`macos-liquid-glass.md`](macos-liquid-glass.md) section 13):
   the regular glass goes from its blur of 6.5 px (dark) or 8.2 px (light) to
   15.5 or 17.5 px and from its tone to a tinted panel, the clear glass from
   no blur to 16.5 px, with piecewise-linear inputs. An earlier comment in
   `ctappearance` (a blur that grows with the square of the slider, a tint that
   barely moves) did not hold: the blur grows about linearly.

Heading **Theme / Tema**:

3. **Color / Color**: nine 32 x 32 swatches 36 apart: Multicolor, Blue,
   Purple, Pink, Red, Orange, Yellow, Green, Graphite (`Multicolor`, `Azul`,
   `Morado`, `Rosa`, `Rojo`, `Naranja`, `Amarillo`, `Verde`, `Grafito`); the
   name of the selected one in Regular 11 under it. Default Multicolor (key
   `AppleAccentColor` absent). 
4. **Text highlight color / Color de resaltado del texto**: popup, live
   `Automatic` / `Automático` (`AppleHighlightColor` absent).
5. **Icon & widget style / Estilo de íconos y widgets**: four 38 x 52 choices:
   `Default` / `Predeterminado`, `Dark` / `Obscuro`, `Clear` / `Transparente`,
   `Tinted` / `Teñido`. Live: Default.
6. **Folder color / Color de carpetas**: popup, live `Automatic`.

Heading **Windows / Ventanas**:

7. **Sidebar icon size / Íconos de la barra lateral**: popup, live `Medium` /
   `Medianos`; the other values were not read (not measured).
8. **Tint window background with wallpaper color / Teñir el fondo de las
   ventanas con el color del fondo de pantalla**: a switch. Live on, key
   `AppleReduceDesktopTinting` = 0 (the key is the inverse: 1 turns the tint
   off). Read 2026-11-26; this note said off and 1 before.
9. **Show scroll bars / Mostrar barras de desplazamiento**: radio group:
   `Automatically based on mouse or trackpad` / `Automáticamente según el mouse
   o trackpad` (live), `When scrolling` / `Durante el desplazamiento`,
   `Always` / `Siempre`. Key `AppleShowScrollBars` = `Automatic`.
10. **Click in the scroll bar to / Hacer clic en la barra de desplazamiento
    para**: `Jump to the next page` / `Ir a la página siguiente` (live),
    `Jump to the spot that’s clicked` / `Ir a donde se hizo clic`. Key
    `AppleScrollerPagingBehavior`, absent, with the next page selected.

Footer: button `More in Accessibility Settings…` / `Más en la configuración de
Accesibilidad…` and a Help button. Card geometry: first card 468 x 319, Theme
cards 101, 129, Windows cards 75 and 189.

### 5.2 Accessibility (`accessibility`) and its Display page

The pane is a list of destinations in 44 high rows under headings:

| Heading | Rows (English) |
|---|---|
| (placard) | `Accessibility`: `Personalize Mac in ways that work best for you with accessibility features for vision, hearing, motor, speech, and cognitive needs. Learn more…` |
| Vision | VoiceOver, Zoom, Hover Text, Display, Motion, Read & Speak, Audio Descriptions |
| Physical and Motor | Voice Control, Keyboard, Pointer Control, Switch Control |
| Hearing | Hearing Devices, Audio, RTT, Subtitles and Captioning, Live Captions, Name Recognition |
| Speech | Live Speech, Personal Voice, Vocal Shortcuts |
| General | Siri, Shortcut |

**Display** (`AX_FEATURE_DISPLAY`), cards in order, English / Spanish:

| Option | Control | Live | Key (`com.apple.universalaccess`) |
|---|---|---|---|
| Invert colors | switch | off | `classicInvertColor` selects the mode |
| Invert colors mode: Smart, Classic | radio, disabled while off | Smart | |
| Increase contrast | switch | off | `increaseContrast` = 0 |
| Reduce transparency | switch | off | `reduceTransparency` = 0 |
| Show borders | switch | off | |
| Differentiate without color | switch | off | `differentiateWithoutColor` = 0 |
| Dim flashing lights | switch, detail `Video content that depicts repeated flashing or strobing lights will be automatically dimmed.` | off | |
| Show window title icons | switch | off | `showWindowTitlebarIcons` = 0 |
| Display contrast | slider 246 wide, `AXMinValue` 0 to `AXMaxValue` 1 | 0 | |

Heading **Text**: `Text size` (a 66 high button: `Text size, Set your preferred
reading size for supported apps and system features., Custom`), `Menu bar
size` with `Default` and `Large` (detail `Some apps or system features may
need a log out for the changes to menu bar font size to take effect.`),
`Prefer horizontal text` (switch).

Heading **Pointer**: `Shake mouse pointer to locate` (switch, detail `Quickly
move the mouse pointer back and forth to make it bigger.`, live on), `Pointer
size` (slider 1 to 4, live 1), `Pointer outline color` and `Pointer fill color` (color wells
48 x 24; live white and black), `Reset Colors` (button).

Heading **Color Filters**: `Color filters` (switch), `Filter type` (popup,
live `Grayscale`), `Intensity` (slider 0.25 to 1, live 1).

Increase contrast and Reduce transparency are two independent switches here;
whether turning on Increase contrast also turns on Reduce transparency (as it
did in older releases) was **not measured**, because it needs a setting change.

### 5.3 Desktop & Dock (`desktopAndDock`)

Headings and options in order (English; the live value in brackets):

- **Dock**: `Dock Size` and `Dock Magnification Size` sliders in one 69 high
  card, 200 and 220 wide (`tilesize` 59 gives 0.384 over 16 to 128;
  `magnification` on, `largesize` 128). `Dock position on screen` [Bottom]
  (key `orientation`, absent), `Minimized window animation` [Genie Effect]
  (`mineffect` absent), `Window title bar double-click action` [Zoom],
  `Minimize windows into application icon` [off],
  `Automatically hide and show the Dock` [off], `Animate opening applications`
  [on], `Show indicators for open applications` [on], `Show suggested and
  recent apps in Dock` [off] (`show-recents` 0).
- **Desktop & Stage Manager**: `Show items` with the checkboxes `On Desktop`
  [on] and `In Stage Manager` [off]; `Click wallpaper to show desktop`
  [`Only in Stage Manager`] with detail `Click wallpaper to move windows out
  of the way, revealing your desktop items and widgets.`
  (`EnableStandardClickToShowDesktop` = 0); `Stage Manager` [off] with detail
  `Stage Manager arranges your recent windows into a single strip for reduced
  clutter and quick access.`; `Show recent apps in Stage Manager` [on]; `Show
  windows from an application` [`All at Once`].
- **Widgets**: `Show Widgets` with `On Desktop` [on] and `In Stage Manager`
  [on]; `Dim widgets on desktop` [`Automatically`]; `Default web browser`.
- **Windows**: `Prefer tabs when opening documents` [`In Full Screen`]; `Ask to
  keep changes when closing documents` [on] (`NSCloseAlwaysConfirmsChanges`);
  `Close windows when quitting an application` [off] with detail `When enabled,
  open documents and windows will not be restored when you re-open an
  application.`; `Drag windows to left or right edge of screen to tile`
  [on]; `Drag windows to menu bar to fill screen` [on]; `Hold ⌥ key while
  dragging windows to tile` [on]; `Tiled windows have margins` [off]
  (`EnableTiledWindowMargins` = 0).
- **Mission Control** with the text `Mission Control shows an overview of your
  open windows and thumbnails of full-screen applications, all arranged in a
  unified view.`: `Automatically rearrange Spaces based on most recent use`
  [on]; `When switching to an application, switch to a Space with open windows
  for the application` [on]; `Group windows by application` [off]; `Displays
  have separate Spaces` [on]; `Drag windows to top of screen to enter Mission
  Control` [on]; buttons `Shortcuts…`, `Hot Corners…` and Help.

Spanish names of the main rows: `Escritorio y Dock`, `Dock`, `Tamaño del Dock`,
`Posición del Dock en la pantalla`, `Ocultar y mostrar el Dock
automáticamente`.

### 5.4 Wallpaper (`wallpaper`)

A header card (not a form): the selected wallpaper as a 160 x 100 thumbnail at
the top-left, and to its right a card 306 x 75 with the wallpaper's name, a
popup for the fit (`Fill Screen` / `Llenar la pantalla`) and `Show on all
Spaces` (switch, on). Under it two buttons `Screen Saver…` and `Clock
Appearance…` (114 and 146 wide, 24 high). Below, a scroll area `Wallpapers`
with horizontal shelves, each with a title in Regular 13 and a `Show All` link
at the right (the first shelf holds the dynamic wallpapers, the second the
user's photos with a `Picture` menu button first). A tile is 109 x 101; shelves
are 110 high and 36 apart. The shelf titles were captured in Spanish (`Fondos de
pantalla dinámicos`, `Tus fotos`, `Paisaje`, `Urbano`, `Subacuático`, `Tierra`);
the English titles were **not measured** (the pane is an extension process).

### 5.5 Notifications (`notifications`)

Placard (icon 30, `Notifications`, `Customize when and how notifications
appear, if they play a sound, and which apps can send them. Learn more…`), then
**Notification Center** with the text `Notification Center shows your
notifications in the top-right corner of your screen. You can show and hide
Notification Center…`; one card: `Show previews` popup [`When Unlocked`];
`Show Notifications:` with `when display is sleeping` [off], `when screen is
locked` [on], `when mirroring or sharing the display` [popup `Notifications
Off`]; a 36 high button row `Summarize notifications`, state `On`. Then
**Application Notifications**: one card of 52 high rows, each with the app
icon, name and a summary (`Off`, or `Badges, Sounds, Desktop, Time Sensitive`).

### 5.6 Wi-Fi (`wifi`)

Placard `Wi‑Fi` (note the non-breaking hyphen) with the text `Set up Wi‑Fi to
wirelessly connect your Mac to the internet. Turn on Wi‑Fi, then choose a
network to join. Learn More…` and the master switch (44 x 20) at the card's
right; in the same card the current network: name, `Connected`, a lock and
signal glyph and a `Details…` button. Headings: `Personal Hotspots`, `Known
Network`, `Other Networks` (with a 16 px busy indicator at the right); each a
card of 36 high rows (checkmark column, name, lock, signal, `Show Detail`
button 17 x 17). `Other…` button, then a card `Ask to join networks` [`Notify`]
with the detail `Known networks will be joined automatically. If no known
networks are available, you will be notified of available networks.`, and
`Ask to join hotspots` [`Ask To Join`] with `Allow this Mac to automatically
discover nearby personal hotspots when no Wi‑Fi network is available.`

### 5.7 Bluetooth (`bluetooth`)

Placard `Bluetooth`, `Connect to accessories you can use for activities such as
streaming music, typing, and gaming. Learn more…`, master switch, and the
line `This Mac is discoverable as “<name>” while Bluetooth Settings is
open.` Headings `My Devices` (a card of 58 high rows: name, `Connected - 30%`
or `Not Connected`, `Show Detail`) and `Nearby Devices` (a busy indicator and
`Searching…`), a Help button.

### 5.8 Network (`network`) and VPN

Rows of 52: one card with the active service (`Wi-Fi, Connected`), one with
`Firewall, Active`; heading `Other Services` with a card of services (`Ethernet`,
`Thunderbolt Bridge`, `iPhone USB`: `Not connected`), a `More` menu button
(`…`) and Help. VPN has the same structure.

### 5.9 Energy (`battery`)

On a desktop Mac the pane is `Energy` and has one card `Low Power Mode` [off]
and one card: `Prevent automatic sleeping when the display is off` [on], `Put
hard disks to sleep when possible` [off], `Wake for network access` [on] (detail
`While sleeping, your Mac can receive incoming network traffic…`), `Start up
when power is connected` with detail `Automatically start after a power
failure.` [popup `After power failure`], and Help. A laptop pane (`Battery`) was
**not measured**.

### 5.10 General (`general`)

Captured in Spanish only. A placard card 164 high (`General` and `Administra tu
configuración y tus preferencias generales para la Mac, como las actualizaciones
de software, el idioma…`) then cards of 48 high rows: `Información`,
`Actualización de software`, `Almacenamiento`; `AppleCare y garantía`; `AirDrop y
continuidad`; `Autorrelleno y contraseñas`, `Compartir`, `Disco de arranque`,
`Elementos de inicio y extensiones`, `Fecha y hora`, `Idioma y región`, `Time
Machine`; `Admón. de dispositivos`; `Transferir o restablecer`. Their ids end in
`about`, `softwareUpdate`, `storage`, `appleCareAndWarranty`, `airDropAndHandoff`,
`passwords`, `sharing`, `startupDisk`, `loginItems`, `dateAndTime`,
`languageAndRegion`, `timeMachine`, `deviceManagement`, `transferOrReset`
(English names: About, Software Update, Storage, AppleCare & Warranty, AirDrop &
Handoff, AutoFill & Passwords, Sharing, Startup Disk, Login Items & Extensions,
Date & Time, Language & Region, Time Machine, Device Management, Transfer or
Reset, from the ids; the English text was **not measured**).

### 5.11 Other panes, one line each

- **Menu Bar** (`controlCenter`): `Automatically hide and show the menu bar`
  [`In Full Screen Only`]; `Show menu bar background` [off]; `Recent documents,
  applications, and servers` [10]; `Show suggestions in Control Gallery`; heading
  `Menu Bar Controls` with a card of checkboxes in this order: Clock (with
  `Clock Options…`), Spotlight, Wi‑Fi, Bluetooth, AirDrop, Focus, Screen Mirroring,
  Display, Sound, Now Playing (the popup `Show When Active` beside the last
  seven), Fast User Switching (popup `Icon`), Text Input, Time Machine, Timer,
  Weather; and heading `Allow in the Menu Bar` with a switch per app.
- **Displays** (`displayAndBrightness`): a 157 high tile strip of displays, then
  a card with the resolution list (24 high rows), `Show all resolutions`,
  `Color profile`, `Refresh rate`, `Rotation`, `When connected to TV`, and the
  buttons `Advanced…`, `Night Shift…`, Help.
- **Sound** (`sounds`): `Sound Effects` (alert sound, `Play sound effects
  through`, `Alert volume`, `Play sound on startup`, `Play user interface sound
  effects`, `Play feedback when volume is changed`) then `Output & Input` with
  an Output and Input tab control and a list of devices.
- **Focus** (`focus`): rows `Do Not Disturb`, `Reduce Interruptions`, `Sleep`,
  `Add Focus…`, `Share across devices` switch, `Focus status`.
- **Lock Screen** (`lockScreen`): `Turn display off when inactive`, `Require
  password after screen saver begins or display is turned off`, `Show user name
  and photo`, `Show password hints`, `Show message when locked` with `Set…`,
  `Show Live Activities when locked`, heading `When Switching User`, `Login
  window shows` (radio), `Show the Sleep, Restart, and Shut Down buttons`,
  `Accessibility Options…`.
- **Keyboard** (`keyboard`): key repeat and delay sliders, `Press 🌐︎ key to`,
  `Keyboard navigation`, `Keyboard Shortcuts…`, headings `Text Input` and
  `Dictation`.

The remaining panes (Siri, Spotlight, Screen Time, Privacy & Security, Touch ID
& Password, Users & Groups, Mouse, Printers & Scanners, Game Controllers,
iCloud, Internet Accounts, Wallet & Apple Pay, Game Center) were walked and their
trees saved, and are not described here.

## 6. Settings that decide blur and tint

| macOS setting | Where (English / Spanish) | Key | Values | Effect |
|---|---|---|---|---|
| Light, Dark, Auto | Appearance > Appearance | `AppleInterfaceStyle`, `AppleInterfaceStyleSwitchesAutomatically` | `Dark` or absent; absent here | Selects the light or dark material set; measured in [`macos-materials.md`](macos-materials.md) and [`macos-blur.md`](macos-blur.md) |
| Liquid Glass | Appearance > Liquid Glass | `NSGlassTintAmount` | 0 to 1 slider, 0 here | one slider for the glass of every `NSGlassEffectView`: blur sigma, tint and tone; measured per process in [`macos-liquid-glass.md`](macos-liquid-glass.md) section 13; the system surfaces were not measured |
| Tint window background with wallpaper color | Appearance > Windows | `AppleReduceDesktopTinting` (1 = off) | switch; on here | Window backgrounds pick up the wallpaper color in dark; effect values in [`macos-blur.md`](macos-blur.md) and in the [2026-11-26 measurements, section 4](macos-measurements-2026-11-26.md#4-wallpaper-tint) |
| Increase contrast | Accessibility > Display | `increaseContrast` (`com.apple.universalaccess`) | 0 or 1 | Alone it keeps every blur ([`macos-materials.md`](macos-materials.md) section 10.4) |
| Reduce transparency | Accessibility > Display | `reduceTransparency` (`com.apple.universalaccess`) | 0 or 1 | Every blurred `NSVisualEffectView` material becomes a solid; the Dock becomes the scheme solid; the menu glass and Control Center are unchanged ([`macos-defaults.md`](macos-defaults.md) section 7b) |
| Reduce transparency with Increase contrast | both | both keys | | A different, stronger solid in 11 of 13 materials |
| Color, Text highlight color | Appearance > Theme | `AppleAccentColor`, `AppleHighlightColor` | absent here | Controls and selection colors, not blur |

macOS exposes no blur radius, blur strength or glass strength other than the
Liquid Glass tint slider. A blur multiplier does not exist in System Settings.

## 7. Not measured

| Item | Why |
|---|---|
| Effect of the Liquid Glass slider on the Dock, the menu bar and Control Center | the per-process effect is measured ([`macos-liquid-glass.md`](macos-liquid-glass.md) section 13); the system surfaces need the setting itself |
| Defaults of every option | the live values are the user's; a fresh account was not reset |
| Whether Increase contrast switches Reduce transparency on | needs a setting change |
| Window minimum and maximum size, and why the width is 740 or 865 | needs resizing or a second language run with the same frame |
| Card, divider, switch and popup colors and radii; hover and pressed states | wide-gamut captures were taken, not analysed |
| English text of General and of the extension panes | the extension process ignores `-AppleLanguages` |
| Fonts of headings, buttons, popups and sidebar labels | no accessibility font; sizes inferred from frames |
| The laptop Battery pane and Trackpad | this Mac is a desktop |
| Third-party panes, Apple Account, Family | out of scope |

## 8. Evidence

All under `~/src/captures/settings-2026-11-22` (not committed):

- `en/enf-com.apple.settings.<id>.txt` and `enf-acc-display.txt`: the accessibility
  tree of each pane in English with fonts and slider ranges; `en/en-*.png`: raw
  window captures; `es/es-*.txt`, `es/es-*.png`: the same in Spanish.
- `tools/`: the walk scripts. The Swift sources are `~/src/probes/axdump.swift`
  and `axpress.swift` (build with `swiftc -O`; the terminal needs Accessibility
  permission). `defaults read` keys of section 6 and 5.3 are in the text above.
