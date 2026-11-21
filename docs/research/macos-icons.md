# Icons macOS uses by default

**Goal:** list the icons AppKit and SwiftUI draw by default on macOS 27.2.1, say where each one is used, and record what the kit has of them.
**Scope:** the sidebar (Finder and SwiftUI), System Settings tiles, menus, toolbars, common actions, states, files, folders, devices and the symbol variants. It starts at the public names and ends at the kit's `Symbol` enum. It copies no Apple drawing.
**Reader:** whoever adds a `Symbol` to the kit or an icon to the theme.
**Input:** a Mac with macOS 27.2.1, the probes in `~/src/probes` and the bench in `scripts/appkit-compare.py`.
**Output:** the tables below and the decision in [ADR 0081](../decisions.md).

## Steps

1. Read the method and the reference frame (section 1).
2. Find the place you need in section 2: the public name, the ink and the kit column.
3. Read the coverage and the priorities in section 3 before drawing anything.
4. Draw with the rules in [Symbols](../barrierkit/04-appearance-and-tokens.md#symbols) and run `scripts/appkit-compare.py icons`.
5. Read section 4 for what is still missing.

## 1. Method and reference frame

Apple's symbols and icons are Apple's work. The kit and the theme never copy an outline, a PDF, a PNG, an ICNS or a table of names from the system. Only numbers leave the Mac: the ink box and the mass of a render. They stay in `~/src/captures/icons`, outside the repository.

1. **Names.** `~/src/probes/icon-names.txt` holds 2495 public names, written from Apple's public catalog. `NSImage(systemSymbolName:accessibilityDescription:)` resolves 2386 of them on macOS 27.2.1. Section 4 lists the rest.
2. **Ink.** `~/src/probes/icons symbols` draws each name at 16 pt, regular, medium scale, into a 4x bitmap. It reads the ink box (alpha above 10% of the peak) and the mass (the sum of alpha). `icons ref` repeats it from 14 to 20 pt and keeps the median of ink over point size, because the ink of a symbol jumps between sizes: `chevron.left` is 13.5 pt high at 17 pt and 16.25 at 18 pt.
3. **Captures.** The Finder and System Settings sidebars come from `screencapture` of one window each (`~/src/probes/winid`). Each glyph is matched to the public name whose render has the nearest shape. A glyph with no match below 0.13 is not identified.
4. **Standard actions.** `~/src/probes/menuicons.sh` builds a main menu of standard selectors with no image set. AppKit gives 47 of 58 items an image in `NSMenuItem.image`. Each is rendered and matched the same way.
5. **Constants.** `icons imagenames` resolves the `NSImageName*` C symbols of `NSImage.h` through `dlsym` and `NSImage(named:)`; `icons uttypes` reads the icon sizes of `NSWorkspace.icon(for:)`.

**Reference frame.** A kit symbol at 20 px on the 16 grid equals the SF Symbol at 16 pt, regular, medium scale. One grid unit is 1.25 px and one em is 12.8 grid units. The SwiftUI sidebar icons (`square.grid.2x2` 15 x 15, `doc` 14 x 18, `network` 17 x 17 px at 1x) and the toolbar glyphs fit it within 1 pt. The kit's sidebar tile is 20 px and its toolbar glyph 21 px.

**Weight, 2026-11-20 (AppKit raster).** `icons weights` drew each symbol with `NSImage.SymbolConfiguration` into a 4x bitmap and read the regular stroke as 1.25 pt at 16 pt: 1.0 grid unit. That is the `SYMBOL.stroke` the 56 symbols of that day were fitted at.

| Weight | Stroke at 16 pt | Weight | Stroke at 16 pt |
|---|---|---|---|
| ultraLight | 0.35 pt | thin | 0.55 pt |
| light | 0.94 pt | regular | 1.23 pt |
| medium | 1.47 pt | semibold | 1.65 pt |
| bold | 1.93 pt | heavy | 2.25 pt |
| black | 2.57 pt | | |

**Weight and scale, 2026-11-21 (SwiftUI vector).** `~/src/probes/iconsui` renders `Image(systemName:)` with `.font(.system(size:weight:))` and `.imageScale(_:)` through `ImageRenderer` at 8x, black on transparent, and reads the stroke as the area of a cross-section (sum of alpha across the bar, no threshold). It reads `minus` (one bar), `circle` (the ring) and `line.3.horizontal` (three bars). Data: `~/src/captures/icons/ui/{stroke,ink,alpha,layers}.json`.

1. At 16 pt, regular, medium scale the stroke is 1.40 pt (`minus`), 1.33 pt (`circle`) and 1.19 pt (`line.3.horizontal`): 1.31 pt on average, 1.05 grid units. Each symbol has its own stroke; the 1.25 pt of 2026-11-20 is the average of the small scale (1.17 pt) and the medium one, not a property of AppKit.
2. The stroke is linear in the point size: 0.0877, 0.0831 and 0.0742 em for the three symbols at every size from 10 to 32 pt (the ink of `circle` is exactly one em).
3. The mean stroke of the three, in points at 16 pt:

| Weight | Small | Medium | Large |
|---|---|---|---|
| ultraLight | 0.36 pt | 0.38 pt | 0.40 pt |
| thin | 0.54 pt | 0.59 pt | 0.64 pt |
| light | 0.90 pt | 1.00 pt | 1.09 pt |
| regular | 1.17 pt | 1.31 pt | 1.43 pt |
| medium | 1.38 pt | 1.59 pt | 1.78 pt |
| semibold | 1.55 pt | 1.80 pt | 2.05 pt |
| bold | 1.76 pt | 2.09 pt | 2.39 pt |
| heavy | 2.07 pt | 2.50 pt | 2.90 pt |
| black | 2.34 pt | 2.87 pt | 3.31 pt |

4. The scale changes the size by a fixed factor, the same for every symbol: small 0.793 and large 1.275 of medium (the mean over 15 symbols and both axes, each within 0.01). On 2026-11-20 an AppKit raster of `circle` gave 0.85 and 1.43; the vector render supersedes it. The stroke grows less than the size: 0.90 and 1.10 of medium at regular weight.
5. A heavier weight grows the ink less than the stroke: `circle` ink grows 1.12 pt from regular to black while its stroke grows 1.45 pt. A kit symbol that keeps its centerline and thickens its stroke overshoots by at most 0.4 pt at black.
6. `SYMBOL.stroke` is 1.0 since 2026-11-21 (it was 1.4). `symbol.weight-stroke-{small,medium,large}` in `components.json` hold the stroke of each weight as a multiple of the regular medium stroke, per grid unit of the scaled box; `symbol.scale-size` holds the size factors.

## 2. Inventory by place

### 2.1 Sidebar: Finder

Measured in a 700 x 600 Finder window in the dark appearance (`~/src/captures/icons/finder-window.png`), window inactive. Rows are 32 px apart, the icon ink starts at x = 18 to 22 and is 14 to 21 px wide, 14 to 19 high. Glyph and text have the same gray (`#909090`) in this capture, so the active color is not measured here: the SwiftUI sidebar below draws its icons in the accent.

| Row | Public name | Ink at 16 pt | Kit | Note |
|---|---|---|---|---|
| Recents | `clock` | 15.8 x 15.8 | `Clock` |  |
| A folder you add to Favorites | `folder` | 17.5 x 14.1 | `Folder` |  |
| Downloads | `arrow.down.circle` | 15.8 x 15.8 | `ArrowDownCircle` (new) |  |
| Desktop | `menubar.dock.rectangle` | 18.4 x 14.4 | `MenubarDockRectangle` |  |
| Documents | `doc` | 13.2 x 16.8 | `Document` |  |
| Movies | `film` | 18.4 x 14.4 | `Film` |  |
| Music | `music.note` | 9.5 x 15.6 | `MusicNote` (new) | match 0.28: the least sure |
| iCloud Drive | `icloud` | 19.7 x 13.6 | `ICloud` (new) |  |
| Home folder | `house` | 18.6 x 16.3 | `House` |  |
| Internal disk | `internaldrive` | 18.4 x 13.3 | `InternalDrive` (new) |  |
| External disk | `externaldrive` | 18.4 x 13.3 | `ExternalDrive` (new) | with an eject glyph at the right, 10 x 9 px |
| Trash | `trash` | 14.7 x 17.3 | `Trash` (new) |  |
| Applications | none public | 17 x 16 px at 1x | — | The best match is 0.24, so the glyph is not a public name |
| AirDrop | none public | — | — | `airdrop` does not resolve; the SwiftUI probe uses `airplayaudio` as a stand-in |
| Tags | `circle.fill` | | `Dot` | A colored dot; not in this sidebar, named from Finder's tag UI |

### 2.2 Sidebar: SwiftUI

`List` with `Label(_, systemImage:)` in a `NavigationSplitView`, read with `~/src/probes/sidebarui` ([metrics](macos-control-metrics.md#sidebar-and-window-macos-27)): rows 32 high, the icon ink starts at x = 21, the text at x = 46, icons in the accent and white when selected.

| Label | Name | Ink at 1x | Ink at 16 pt | Kit |
|---|---|---|---|---|
| AirDrop | `airplayaudio` | 17 x 17 | 15.8 x 15.6 | — |
| Recents | `clock` | selected | 15.8 x 15.8 | `Clock` |
| Applications | `square.grid.2x2` | 15 x 15 | 14.4 x 14.4 | `SquareGrid` |
| Documents | `doc` | 14 x 18 | 13.2 x 16.8 | `Document` |
| iCloud Drive | `icloud` | 21 x 15 | 19.7 x 13.6 | `ICloud` (new) |
| Network | `network` | 17 x 17 | 15.8 x 15.8 | `Network` |

### 2.3 System Settings

The sidebar tiles are 20 x 20 px at x = 19, 32 px apart, with a white glyph. Colors are the capture converted to sRGB. The kit has four tile colors (`settings.tile`) and its `SymbolTile` draws the glyph at `SYMBOL.fine_stroke`.

| Pane | Tile | Public name | Kit |
|---|---|---|---|
| Wi-Fi | `#008fff` | `wifi` | `Wifi` |
| Bluetooth | `#008fff` | no public match | — |
| Network | `#008fff` | no public match | — |
| VPN | `#008fff` | no public match | — |
| Battery | `#40cf62` | `bolt.fill` | `Bolt` |
| General | `#9b9b9f` | `gear` | — |
| Accessibility | `#008fff` | no public match | — |
| Appearance | `#0a0a0a` | no public match | — |
| Menu Bar | `#9b9b9f` | `switch.2` | — |
| Desktop & Dock | `#0a0a0a` | no public match | — |
| Wallpaper | `#22c6f0` | no public match | — |
| Displays | `#008fff` | no public match | — |
| Notifications | `#ff3d41` | `bell.badge.fill` | — |
| Sound | `#ff325d` | `speaker.wave.3.fill` | — |
| Focus | `#695dff` | `moon.fill` | — |
| Screen Time | `#695dff` | `hourglass` | — |

A row with no public name was not identified by shape at 1x: its glyph is private or too small to tell apart. The Family, Siri and Spotlight tiles are not matched either.

### 2.4 Menus

`NSMenuItem.image` is not drawn by a pop-up menu on macOS 27.2.1 ([metrics](macos-control-metrics.md)), but AppKit sets it by default on every item whose action is a standard one. The table lists what it sets; `likely` is a shape distance from 0.035 to 0.06 (the rectangle family is ambiguous). Whether the menu bar draws it was not confirmed: another probe held the focus during the capture.

| Menu | Item | Action | Symbol | Match | Kit |
|---|---|---|---|---|---|
| App | About | `orderFrontStandardAboutPanel:` | `info.circle` | certain | `InfoCircle` (new) |
| App | Hide | `hide:` | `rectangle.dashed` | likely | — |
| App | Hide Others | `hideOtherApplications:` | `rectangle.on.rectangle` | likely | — |
| App | Show All | `unhideAllApplications:` | `macwindow.on.rectangle` | certain | — |
| App | Quit | `terminate:` | `xmark.rectangle` | certain | — |
| File | New | `newDocument:` | `plus` | certain | `Plus` |
| File | Open | `openDocument:` | `arrow.up.forward` | certain | `ArrowUpForward` (new) |
| File | Close | `performClose:` | `xmark` | certain | `XMark` |
| File | Save | `saveDocument:` | `square.and.arrow.down` | likely | `SquareAndArrowDown` (new) |
| File | Move To | `moveDocument:` | `folder` | certain | `Folder` |
| File | Revert | `revertDocumentToSaved:` | `arrow.counterclockwise` | certain | `ArrowCounterclockwise` (new) |
| File | Export | `saveDocumentTo:` | `square.and.arrow.down.on.square` | certain | — |
| File | Page Setup | `runPageLayout:` | `doc.plaintext` | likely | — |
| File | Print | `printDocument:` | `printer` | likely | `Printer` |
| Edit | Undo | `undo:` | `arrow.uturn.backward` | certain | `ArrowUturnBackward` (new) |
| Edit | Redo | `redo:` | `arrow.uturn.forward` | certain | `ArrowUturnForward` (new) |
| Edit | Cut | `cut:` | `scissors` | likely | `Scissors` (new) |
| Edit | Copy | `copy:` | `doc.on.doc` | certain | `DocOnDoc` (new) |
| Edit | Paste | `paste:` | `doc.on.clipboard` | certain | — |
| Edit | Delete | `delete:` | `trash` | likely | `Trash` (new) |
| Edit | Start Dictation | `startDictation:` | `mic` | likely | `Mic` (new) |
| Format | Show Fonts | `orderFrontFontPanel:` | `textformat` | certain | — |
| Format | Bold | `toggleBoldface:` | `bold` | likely | — |
| Format | Italic | `toggleItalics:` | `italic` | certain | — |
| Format | Underline | `underline:` | `underline` | certain | — |
| Format | Align Left | `alignLeft:` | `text.alignleft` | likely | `TextAlignLeft` (new) |
| Format | Center | `alignCenter:` | `text.aligncenter` | likely | `TextAlignCenter` (new) |
| Format | Align Right | `alignRight:` | `text.alignright` | likely | `TextAlignRight` (new) |
| Format | Show Colors | `orderFrontColorPanel:` | `paintpalette` | certain | — |
| View | Customize Toolbar | `runToolbarCustomizationPalette:` | `wrench` | certain | — |
| View | Show Sidebar | `toggleSidebar:` | `sidebar.leading` | certain | `SidebarLeft` |
| View | Show All Tabs | `toggleTabOverview:` | `square.on.square` | likely | `SquareOnSquare` |
| Window | Minimize | `performMiniaturize:` | `plus.rectangle` | likely | — |
| Window | Bring All to Front | `arrangeInFront:` | `square.3.layers.3d` | certain | — |
| Window | (open window) | `makeKeyAndOrderFront:` | `macwindow` | certain | `MacWindow` (new) |

Items with an image whose shape matches no public name (private art): File > Duplicate (`duplicateDocument:`), File > Rename (`renameDocument:`), Edit > Paste and Match Style (`pasteAsPlainText:`), Edit > Select All (`selectAll:`), Edit > Emoji & Symbols (`orderFrontCharacterPalette:`), Format > Justify (`alignJustified:`), View > Zoom In (`zoomIn:`), View > Zoom Out (`zoomOut:`), Window > Zoom (`performZoom:`), Window > Fill (`_zoomFill:`), Window > Center (`_zoomCenter:`), Help > Help (`showHelp:`).

Standard items with no image: Edit > Find, Edit > Spelling and Grammar, Format > Bigger, Format > Smaller, Format > Show Ruler, View > Show Toolbar, View > Show Tab Bar, View > Enter Full Screen, View > Actual Size, Window > Move & Resize.

### 2.5 Toolbar

AppKit and SwiftUI draw a toolbar item's image as given: there is no default glyph for a custom item. The standard `NSToolbarItem.Identifier` items bring their own, read from `NSToolbarItem.image` in a live window:

| Identifier | Label | Symbol | Kit |
|---|---|---|---|
| `.toggleSidebar` | Sidebar | `sidebar.left` | `SidebarLeft` |
| `.toggleInspector` | Inspector | `sidebar.right` | `SidebarRight` (new) |
| `.print` | Print | `printer` | `Printer` |
| `.showFonts` | Fonts | `textformat` | — |
| `.cloudSharing` | Add People | `person.crop.circle.badge.plus` | — |
| `.showColors` | Colors | a bitmap, not a symbol (`NSColorPanel`) | — |

`.space`, `.flexibleSpace` and the two tracking separators have no image. Every toolbar symbol is drawn at the size in [Toolbar items](macos-control-metrics.md#toolbar-items-and-navigation-buttons).

### 2.6 Common actions

Ink is the SF Symbol at 16 pt. The kit column says `(new)` for the symbols drawn in this work.

**Share and export.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `square.and.arrow.up` | Share | 13.9 x 17.6 | `SquareAndArrowUp` (new) |
| `square.and.arrow.down` | Save, download | 13.9 x 16.7 | `SquareAndArrowDown` (new) |
| `square.and.arrow.down.on.square` | Export | 17.2 x 19.9 | — |
| `arrow.up.forward` | Open, external link | 11.7 x 11.7 | `ArrowUpForward` (new) |
| `link` | Copy link | 16.7 x 16.9 | — |
| `paperplane` | Send | 15.9 x 16.6 | `PaperPlane` (new) |
| `envelope` | Mail | 18.7 x 13.6 | `Envelope` (new) |

**Add and remove.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `plus` | Add | 13.0 x 13.1 | `Plus` |
| `plus.circle` | Add (row) | 15.8 x 15.8 | `PlusCircle` (new) |
| `minus` | Remove | 13.0 x 1.4 | `Minus` |
| `trash` | Delete | 14.7 x 17.3 | `Trash` (new) |
| `xmark` | Close | 12.4 x 12.5 | `XMark` |
| `xmark.circle` | Clear | 15.8 x 15.8 | `XMarkCircle` (new) |
| `folder.badge.plus` | New folder | 21.0 x 14.6 | — |
| `doc.badge.plus` | New document | 16.7 x 18.3 | — |

**Edit.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `pencil` | Edit | 12.7 x 12.3 | — |
| `square.and.pencil` | Compose | 15.4 x 15.5 | — |
| `doc.on.doc` | Copy | 15.7 x 19.8 | `DocOnDoc` (new) |
| `doc.on.clipboard` | Paste | 15.7 x 19.8 | — |
| `scissors` | Cut | 20.1 x 15.3 | `Scissors` (new) |
| `arrow.uturn.backward` | Undo | 14.7 x 14.7 | `ArrowUturnBackward` (new) |
| `arrow.uturn.forward` | Redo | 14.7 x 14.7 | `ArrowUturnForward` (new) |
| `arrow.counterclockwise` | Revert | 14.0 x 16.8 | `ArrowCounterclockwise` (new) |
| `arrow.clockwise` | Reload | 14.0 x 16.8 | `ArrowClockwise` (new) |
| `eraser` | Erase | 16.0 x 15.7 | — |

**Find, sort and filter.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `magnifyingglass` | Search | 15.5 x 15.4 | `MagnifyingGlass` |
| `arrow.up.arrow.down` | Sort | 18.4 x 15.2 | `ArrowUpArrowDown` (new) |
| `line.3.horizontal.decrease` | Filter | 15.7 x 9.5 | `Line3HorizontalDecrease` (new) |
| `line.3.horizontal.decrease.circle` | Filter on | 15.8 x 15.8 | — |
| `slider.horizontal.3` | Adjust | 15.5 x 12.9 | — |

**More, information and settings.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `ellipsis` | More | 14.7 x 2.9 | `Ellipsis` |
| `ellipsis.circle` | More (row) | 15.8 x 15.8 | `EllipsisCircle` |
| `info.circle` | Information | 15.8 x 15.8 | `InfoCircle` (new) |
| `questionmark.circle` | Help | 15.8 x 15.8 | `QuestionMarkCircle` (new) |
| `gearshape` | Settings | 16.8 x 16.8 | `GearShape` (new) |

**Navigation.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `chevron.left` | Back | 7.8 x 13.6 | `ChevronLeft` |
| `chevron.right` | Forward, disclosure | 7.8 x 13.6 | `ChevronRight` |
| `chevron.down` | Expand | 13.5 x 7.7 | `ChevronDown` |
| `arrow.left` | Left | 14.3 x 12.1 | `ArrowLeft` (new) |
| `arrow.right` | Right | 14.3 x 12.1 | `ArrowRight` (new) |
| `arrow.up` | Up | 11.9 x 14.7 | `ArrowUp` (new) |
| `arrow.down` | Down | 11.9 x 14.7 | — |
| `house` | Home | 18.6 x 16.3 | `House` |

**Layout and view.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `sidebar.left` | Sidebar | 18.4 x 14.4 | `SidebarLeft` |
| `sidebar.right` | Inspector | 18.4 x 14.4 | `SidebarRight` (new) |
| `list.bullet` | List | 15.5 x 11.5 | `ListBullet` |
| `square.grid.2x2` | Grid | 14.4 x 14.4 | `SquareGrid` |
| `macwindow` | Window | 18.4 x 14.4 | `MacWindow` (new) |
| `rectangle.3.group` | Columns | 22.0 x 14.1 | — |

**Text.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `textformat` | Fonts | 17.8 x 11.7 | — |
| `bold` | Bold | 9.4 x 11.2 | — |
| `italic` | Italic | 8.1 x 11.2 | — |
| `underline` | Underline | 9.0 x 14.1 | — |
| `text.alignleft` | Align left | 15.5 x 13.6 | `TextAlignLeft` (new) |
| `text.aligncenter` | Center | 15.5 x 13.6 | `TextAlignCenter` (new) |
| `text.alignright` | Align right | 15.5 x 13.6 | `TextAlignRight` (new) |
| `text.justify` | Justify | 15.5 x 13.6 | `TextJustify` (new) |

**Media.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `play.fill` | Play | 11.4 x 13.0 | `Play` |
| `pause.fill` | Pause | 9.6 x 13.1 | `Pause` |
| `backward.fill` | Previous | 20.8 x 11.9 | `Backward` |
| `forward.fill` | Next | 20.9 x 11.9 | `Forward` |
| `shuffle` | Shuffle | 17.9 x 14.1 | `Shuffle` |
| `repeat` | Repeat | 16.5 x 14.0 | — |
| `speaker.wave.2` | Volume | 17.4 x 14.0 | `SpeakerWave` |
| `mic` | Microphone | 11.3 x 16.8 | `Mic` (new) |
| `camera` | Camera | 19.1 x 15.1 | `Camera` (new) |

**Organize.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `tag` | Tag | 17.6 x 17.6 | `Tag` (new) |
| `flag` | Flag | 14.1 x 15.4 | `Flag` (new) |
| `pin` | Pin | 12.2 x 18.1 | — |
| `star` | Favorite | 17.6 x 16.8 | `Star` |
| `heart` | Like | 15.3 x 14.2 | `Heart` (new) |
| `bookmark` | Bookmark | 10.7 x 16.8 | `Bookmark` |
| `folder` | Folder | 17.5 x 14.1 | `Folder` |
| `person` | Person | 13.6 x 14.5 | `Person` |
| `person.crop.circle` | Account | 15.8 x 15.8 | `PersonCropCircle` (new) |

**Privacy and time.**

| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `lock` | Lock | 10.7 x 15.4 | `Lock` |
| `lock.open` | Unlock | 16.2 x 15.4 | — |
| `key` | Key | 10.5 x 19.7 | `Key` (new) |
| `eye` | Show | 21.7 x 13.3 | — |
| `eye.slash` | Hide | 21.7 x 14.3 | — |
| `clock` | Time | 15.8 x 15.8 | `Clock` |
| `calendar` | Date | 15.5 x 14.4 | `Calendar` (new) |

### 2.7 States


| Name | Use | Ink at 16 pt | Kit |
|---|---|---|---|
| `exclamationmark.triangle` | Warning | 16.4 x 14.7 | `ExclamationMarkTriangle` (new) |
| `exclamationmark.triangle.fill` | Warning (alert) | 16.4 x 14.7 | `ExclamationMarkTriangleFill` (new) |
| `exclamationmark.circle` | Problem | 15.8 x 15.8 | `ExclamationMarkCircle` (new) |
| `exclamationmark.circle.fill` | Problem (filled) | 15.8 x 15.8 | `ExclamationMarkCircleFill` (new) |
| `xmark.circle` | Failure, clear | 15.8 x 15.8 | `XMarkCircle` (new) |
| `xmark.circle.fill` | Clear (search field) | 15.8 x 15.8 | `XMarkCircleFill` (new) |
| `checkmark.circle` | Success | 15.8 x 15.8 | `CheckmarkCircle` (new) |
| `checkmark.circle.fill` | Success (filled) | 15.8 x 15.8 | `CheckmarkCircleFill` (new) |
| `info.circle` | Information | 15.8 x 15.8 | `InfoCircle` (new) |
| `info.circle.fill` | Information (filled) | 15.8 x 15.8 | `InfoCircleFill` (new) |
| `questionmark.circle` | Help | 15.8 x 15.8 | `QuestionMarkCircle` (new) |
| `questionmark.circle.fill` | Help (filled) | 15.8 x 15.8 | `QuestionMarkCircleFill` (new) |
| `plus.circle.fill` | Add (filled) | 15.8 x 15.8 | `PlusCircleFill` (new) |
| `minus.circle.fill` | Remove (filled) | 15.8 x 15.8 | `MinusCircleFill` (new) |
| `nosign` | Not allowed | 15.8 x 15.8 | `NoSign` (new) |
| `tray` | Empty | 18.4 x 13.3 | — |
| `magnifyingglass` | No results | 15.5 x 15.4 | `MagnifyingGlass` |
| `wifi.slash` | Offline | 17.9 x 14.3 | — |
| `icloud.slash` | iCloud off | 19.7 x 15.7 | — |
| `lock.slash` | Locked out | 14.8 x 16.0 | — |

`ContentUnavailableView` and the empty states of `List` and `Table` use a large `tray` or `magnifyingglass` with the title below; the kit's [`ContentUnavailableView`](../barrierkit/20-glossary.md) takes any `Symbol`.

### 2.8 Files, folders and devices

- **File and folder icons** are bitmaps. `NSWorkspace.icon(for:)` returns an `NSImage` of 32 x 32 pt for each of the 29 `UTType`s tried, with representations from 16 to 1024 pt at 1x and 2x. They are colored artwork and cannot be recreated from the public API: the theme draws its own ([ADR 0081](../decisions.md)).
- **Devices** have a monochrome SF Symbol each (below) and a colored bitmap in the system.
- **`NSImage.Name` constants**: 139 of 141 resolve (appendix A). 108 are monochrome templates (bitmaps), 31 are colored.

| Name | Device | Ink at 16 pt | Kit |
|---|---|---|---|
| `desktopcomputer` | Mac mini, iMac | 18.6 x 15.6 | `DesktopComputer` (new) |
| `laptopcomputer` | MacBook | 22.4 x 12.9 | — |
| `display` | Display | 18.6 x 15.6 | `Display` |
| `iphone` | iPhone | 10.4 x 16.5 | — |
| `ipad` | iPad | 13.1 x 17.8 | — |
| `applewatch` | Apple Watch | 11.4 x 16.8 | — |
| `appletv` | Apple TV | 15.5 x 15.7 | — |
| `homepod` | HomePod | 12.4 x 16.3 | — |
| `airpods` | AirPods | 15.5 x 15.2 | — |
| `keyboard` | Keyboard | 19.7 x 12.2 | `Keyboard` |
| `computermouse` | Mouse | 10.8 x 17.2 | — |
| `printer` | Printer | 17.5 x 16.8 | `Printer` |
| `externaldrive` | External disk | 18.4 x 13.3 | `ExternalDrive` (new) |
| `internaldrive` | Internal disk | 18.4 x 13.3 | `InternalDrive` (new) |
| `opticaldisc` | Disc | 15.8 x 15.8 | — |
| `tv` | Television | 19.6 x 15.6 | `Tv` |
| `hifispeaker` | Speaker | 12.0 x 16.3 | — |

### 2.9 Variants and rendering modes

Probed on the 2386 names that resolve:

| Variant | Names that have it |
|---|---|
| `.fill` | 866 (761 base names with both) |
| `.circle` and `.circle.fill` | 317 each |
| `.square` and `.square.fill` | 82 and 80 |
| `.slash` | 88 |
| `.badge.plus` | 50 |
| `.rtl` | 57 |

| Mode | Names |
|---|---|
| Monochrome | all 2386 |
| Hierarchical (the alpha of layers differs) | 1438 |
| Palette with 2 layers / with 3 layers | 1478 / 19 |
| Multicolor (`preferringMulticolor`) | 474 |
| Variable value (`variableValue`) | 315 |

**Hierarchical alpha, 2026-11-21.** `iconsui alpha` and `iconsui layers` render each name with `.symbolRenderingMode(.hierarchical)` and a black foreground, and read the alpha histogram. The levels are fixed: primary 1.0, secondary 0.5, tertiary 0.25 (`symbol.hierarchy` in `components.json`). 36 of the 122 SF names the kit has draw a second layer:

| Family | Dimmed layer | Alpha |
|---|---|---|
| `*.circle`, `accessibility`, `clock`, `ellipsis.circle`, `exclamationmark.triangle` | the ring or outline | 0.5 |
| `*.circle.fill`, `exclamationmark.triangle.fill` | the disc, under the mark | 0.25 |
| `square.and.arrow.up`, `square.and.arrow.down` | the box | 0.5 |
| `arrow.up.arrow.down` | the right arrow | 0.5 |
| `doc.on.doc`, `square.on.square`, `capsule.on.rectangle`, `menubar.dock.rectangle`, `captions.bubble` | the back shape or the frame | 0.5 |
| `list.bullet` | the bars; the bullets stay full | 0.5 |
| `speaker.wave.2` | the outer wave | 0.5 |
| `desktopcomputer`, `display`, `tv` | the screen, a filled area | 0.25 |
| `macwindow` | the frame; the three dots stay full | 0.5 |

The other 86 are drawn at 1.0 in both modes (`wifi`, `folder.fill`, `battery.100`, `sidebar.left`). The kit draws monochrome in one color (`currentColor`) and hierarchical for 30 of the 36 (the table's `desktopcomputer`, `display`, `tv` have no filled screen in the kit, and `macwindow` has no dots; `printer` has a layer of 5 pt², left out). A `.fill` variant is a second `Symbol` (`StarFill`, `XMarkCircleFill`).

## 3. Kit coverage and priority

The kit had 65 symbols. 2026-11-20 added 56 and 2026-11-21 adds 27. The `Symbol` enum has 151 variants.

| Group | In the kit | Of | Priority |
|---|---|---|---|
| Sidebar and Settings | 23 | 27 | P1 |
| Menus (standard actions) | 24 | 35 | P1 |
| Common actions | 71 | 82 | P1 |
| States | 16 | 20 | P1 |
| Devices | 7 | 17 | P3 |

The counts of 2026-11-20 are kept for the groups the new symbols do not touch; Menus gain `xmark.rectangle`, `plus.rectangle`, `rectangle.on.rectangle`, `rectangle.dashed` and `macwindow.on.rectangle`, and Common actions gain `arrow.down`, `link`, `pencil`, `square.and.pencil`, `lock.open`, `repeat`, `tray`, `doc.text`, `speaker.slash` and `speaker.wave.1`.

Priority follows use: the sidebar, the standard menus, the common actions and the states appear in every app (P1); devices and media transport appear in a few (P3).

### 3.1 Symbols drawn in this work

Each new symbol is rendered by the kit at 20 px and 4x (`appkit_icon_lab`) and compared with the SF Symbol at 16 pt (`scripts/appkit-compare.py icons`). The ink box of all 56 is within 0.5 of AppKit's. The mass is within 5% for 28 and within 10% for 51.

| Kit symbol | Apple name | Paint | d box | d mass |
|---|---|---|---|---|
| `ICloud` | `icloud` | stroke | 0.20 | -2.8% |
| `ArrowDownCircle` | `arrow.down.circle` | stroke | 0.06 | -5.4% |
| `MusicNote` | `music.note` | both | 0.15 | +16.5% |
| `InternalDrive` | `internaldrive` | stroke | 0.20 | -4.4% |
| `ExternalDrive` | `externaldrive` | stroke | 0.20 | +3.7% |
| `Trash` | `trash` | stroke | 0.08 | -3.4% |
| `Tag` | `tag` | stroke | 0.15 | +0.9% |
| `SquareAndArrowUp` | `square.and.arrow.up` | stroke | 0.35 | +9.1% |
| `SquareAndArrowDown` | `square.and.arrow.down` | stroke | 0.12 | +3.8% |
| `ArrowUpArrowDown` | `arrow.up.arrow.down` | stroke | 0.13 | -7.2% |
| `Line3HorizontalDecrease` | `line.3.horizontal.decrease` | stroke | 0.03 | -8.0% |
| `InfoCircle` | `info.circle` | stroke | 0.06 | -10.0% |
| `PlusCircle` | `plus.circle` | stroke | 0.06 | -2.7% |
| `MinusCircle` | `minus.circle` | stroke | 0.06 | -5.4% |
| `XMarkCircle` | `xmark.circle` | stroke | 0.06 | -4.9% |
| `CheckmarkCircle` | `checkmark.circle` | stroke | 0.06 | -7.7% |
| `ExclamationMarkCircle` | `exclamationmark.circle` | stroke | 0.06 | -3.0% |
| `QuestionMarkCircle` | `questionmark.circle` | stroke | 0.06 | -1.9% |
| `ExclamationMarkTriangle` | `exclamationmark.triangle` | stroke | 0.07 | +1.8% |
| `NoSign` | `nosign` | stroke | 0.06 | -4.8% |
| `ArrowLeft` | `arrow.left` | stroke | 0.19 | -9.3% |
| `ArrowRight` | `arrow.right` | stroke | 0.19 | -9.3% |
| `ArrowUp` | `arrow.up` | stroke | 0.08 | -9.9% |
| `ArrowUpForward` | `arrow.up.forward` | stroke | 0.32 | -24.9% |
| `Heart` | `heart` | stroke | 0.04 | -1.2% |
| `Key` | `key` | stroke | 0.29 | +1.0% |
| `Mic` | `mic` | stroke | 0.01 | -7.8% |
| `Camera` | `camera` | stroke | 0.11 | -5.9% |
| `Envelope` | `envelope` | stroke | 0.05 | +0.8% |
| `Calendar` | `calendar` | stroke | 0.08 | +1.1% |
| `Flag` | `flag` | stroke | 0.19 | -1.0% |
| `PaperPlane` | `paperplane` | stroke | 0.13 | -0.3% |
| `Scissors` | `scissors` | both | 0.33 | -5.4% |
| `XMarkCircleFill` | `xmark.circle.fill` | solid | 0.06 | -6.1% |
| `PlusCircleFill` | `plus.circle.fill` | solid | 0.06 | -6.8% |
| `MinusCircleFill` | `minus.circle.fill` | solid | 0.06 | -5.2% |
| `CheckmarkCircleFill` | `checkmark.circle.fill` | solid | 0.06 | -5.7% |
| `ExclamationMarkCircleFill` | `exclamationmark.circle.fill` | solid | 0.06 | -5.2% |
| `InfoCircleFill` | `info.circle.fill` | solid | 0.06 | -1.0% |
| `QuestionMarkCircleFill` | `questionmark.circle.fill` | solid | 0.06 | -9.2% |
| `ExclamationMarkTriangleFill` | `exclamationmark.triangle.fill` | solid | 0.07 | +0.4% |
| `GearShape` | `gearshape` | stroke | 0.00 | -3.2% |
| `ArrowClockwise` | `arrow.clockwise` | stroke | 0.50 | +1.5% |
| `ArrowCounterclockwise` | `arrow.counterclockwise` | stroke | 0.50 | -0.3% |
| `TextAlignLeft` | `text.alignleft` | stroke | 0.08 | -0.9% |
| `TextAlignCenter` | `text.aligncenter` | stroke | 0.08 | -0.1% |
| `TextAlignRight` | `text.alignright` | stroke | 0.08 | -1.0% |
| `TextJustify` | `text.justify` | stroke | 0.08 | -2.2% |
| `MacWindow` | `macwindow` | stroke | 0.33 | +17.6% |
| `SidebarRight` | `sidebar.right` | stroke | 0.33 | +5.8% |
| `DocOnDoc` | `doc.on.doc` | stroke | 0.01 | -14.0% |
| `ArrowUturnBackward` | `arrow.uturn.backward` | stroke | 0.09 | -8.9% |
| `ArrowUturnForward` | `arrow.uturn.forward` | stroke | 0.34 | -8.5% |
| `DesktopComputer` | `desktopcomputer` | stroke | 0.10 | -29.8% |
| `PersonCropCircle` | `person.crop.circle` | solid | 0.06 | -9.3% |
| `Eject` | `eject` | stroke | 0.10 | -2.9% |

The paint is `stroke` (an outline at the stroke of the `Image`), `solid` (filled, no stroke) or `both` (filled and stroked).

### 3.2 Symbols drawn on 2026-11-21

The same bench (`~/src/probes/icons-design`, `d4.py` and `emit4.py`), at stroke 1.0 against the 2026-11-20 targets. `d box` is the larger of the width and height error in grid units. The paint is as in 3.1.

| Kit symbol | Apple name | Paint | d box | d mass |
|---|---|---|---|---|
| `ArrowDown` | `arrow.down` | stroke | 0.06 | -10.0% |
| `Link` | `link` | stroke | 0.29 | -14.8% |
| `Pencil` | `pencil` | solid | 0.07 | -2.3% |
| `SquareAndPencil` | `square.and.pencil` | stroke | 0.09 | +11.6% |
| `LockOpen` | `lock.open` | stroke | 0.05 | +8.1% |
| `LockFill` | `lock.fill` | solid | 0.08 | -0.9% |
| `StopFill` | `stop.fill` | solid | 0.08 | -1.2% |
| `SpeakerSlash` | `speaker.slash` | stroke | 0.08 | +4.3% |
| `SpeakerWave1` | `speaker.wave.1` | stroke | 0.23 | +5.4% |
| `Repeat` | `repeat` | stroke | 0.04 | +9.5% |
| `CheckmarkSquare` | `checkmark.square` | stroke | 0.11 | -1.3% |
| `Tray` | `tray` | stroke | 0.16 | +3.4% |
| `DocText` | `doc.text` | stroke | 0.08 | +0.0% |
| `XMarkRectangle` | `xmark.rectangle` | stroke | 0.27 | -2.9% |
| `PlusRectangle` | `plus.rectangle` | stroke | 0.27 | -2.7% |
| `RectangleOnRectangle` | `rectangle.on.rectangle` | stroke | 0.11 | -10.2% |
| `RectangleDashed` | `rectangle.dashed` | stroke | 0.27 | -5.4% |
| `MacWindowOnRectangle` | `macwindow.on.rectangle` | stroke | 0.11 | -15.9% |
| `ArrowRightCircle` | `arrow.right.circle` | stroke | 0.05 | -4.3% |
| `ArrowLeftCircle` | `arrow.left.circle` | stroke | 0.05 | -4.3% |
| `ArrowUpCircle` | `arrow.up.circle` | stroke | 0.05 | -4.3% |
| `ChevronRightCircle` | `chevron.right.circle` | stroke | 0.05 | -7.8% |
| `ChevronLeftCircle` | `chevron.left.circle` | stroke | 0.05 | -7.8% |
| `PlayCircle` | `play.circle` | stroke | 0.05 | -6.9% |
| `PauseCircle` | `pause.circle` | stroke | 0.05 | -10.3% |
| `HeartFill` | `heart.fill` | both | 0.03 | -6.1% |
| `CameraFill` | `camera.fill` | both | 0.09 | -5.7% |

The lab (`scripts/appkit-compare.py icons`) agrees with the bench: the ink box of all 27 is within 0.36 of AppKit's and the mass is within 10% for 24 (`Link`, `SquareAndPencil`, `MacWindowOnRectangle` and `PauseCircle` are 10.6% to 15.7% off). Of the 83 symbols of the lab, 40 are within 0.5 of the ink box and 5% of the mass and 70 within 10%. `HeartFill` and `CameraFill` share the path of `Heart` and `Camera` and paint it filled and stroked (the lens of the camera is cut out by the even-odd rule). `ArrowDown` was the name of `arrow.down.to.line`; that symbol is now `ArrowDownToLine`.

The frame test of the same day found four of the 56 symbols of 2026-11-20 cut by the 16 grid, and moved them inside: `ICloud` 0.25 right, `Key` scaled to 0.985, `DocOnDoc` 0.12 up, `Scissors` scaled to 0.97 (its box error went from 0.33 to 0.55: SF's own ink is 20.1 pt wide, wider than the 20 px frame).

`Paperclip` (`paperclip`) was tried and left out: the spiral of the SF symbol needs a hand-fitted curve and the first fit did not read as a clip.

### 3.3 The 65 older symbols

The 65 older symbols were drawn before this frame, at the stroke of 1.4. Remeasured on 2026-11-21 at stroke 1.0 against the same targets, 35 of the 63 that have an SF name are more than 1.5 grid units off in the ink box: `Backward`, `Forward`, `Dot`, `ChevronLeftForwardSlashChevronRight` and `Battery` by 4 to 8; `Captions`, `Tv`, `StarFill`, `SquareOnSquare`, `GoBackward`, `GoForward`, `House`, `Display`, `SidebarLeft`, `Film`, `Photo`, `Circle`, `Keyboard`, `Bed`, `Briefcase`, `Shuffle`, `Checkmark`, `Star`, `ChevronDoubleRight`, `Folder` and `Printer` by 2 to 4. Most are too small, at 0.8 to 0.9 of SF's ink. The mass goes from 80% under (`Ellipsis`) to 30% over (`SquareGrid`). `TriangleLeft` and `TriangleRight` have no reference in `ref.json`.

They are left as they are: redrawing them moves the golden of every control that shows one, and those controls were sized to the old shapes. `Folder`, `House`, `SidebarLeft` and `Star` come first; the bench already holds a fitted `Folder` (`d1.py`, 0.1 units, mass +0.3%).

## 4. Left out

- **Wider than the grid.** 301 of the 2386 symbols are wider than 1.25 em, so their ink does not fit the 16 grid at 20 px: `eye`, `person.2`, `folder.badge.plus`, `macbook`, `ipad.landscape`, `tv.and.mediabox`. They need a symbol box wider than 16.
- **Solid symbols.** `desktopcomputer` is drawn as an outline; SF draws a solid body and a screen at partial alpha (a hierarchical layer). Its mass is 30% under.
- **Letter symbols.** `bold`, `italic`, `underline`, `textformat`: they need glyphs, not strokes.
- **Heavy detail.** `pencil`, `pin`, `wrench`, `hand.raised`, `lightbulb`, `doc.on.clipboard`, `person.crop.rectangle` and every `.badge` variant.
- **Private glyphs.** AirDrop, Applications, Bluetooth, VPN, Spotlight, Siri and the Family tile have no public name.
- **Environment weights and scales.** `Image::font_weight` and `Image::image_scale` draw the nine weights and three scales of section 1 (2026-11-21). SwiftUI's `.fontWeight(_:)` and `.imageScale(_:)` also work on a whole subtree; the kit has no such environment value, so a control that draws a symbol keeps regular weight and medium scale.
- **Rendering modes.** Palette and multicolor are not drawn. Hierarchical is, for 30 of the 36 symbols that have it (section 2.9); `variableValue` is not.
- **Names that do not resolve** on macOS 27.2.1 (of 2495): `airdrop`, `bell.and.waves.left.and.right.circle`, `bell.and.waves.left.and.right.circle.fill`, `bluetooth`, `calendar.today`, `chevron.down.to.line`, `chevron.up.to.line`, `circle.righthalf.striped.horizontal`, `clock.arrow.trianglehead.clockwise.rotate.90`, `doc.badge.arrow`, `doc.badge.bolt`, `doc.badge.bookmark`, `doc.badge.checkmark`, `doc.badge.envelope`, `doc.badge.heart`, `doc.badge.lock`, `doc.badge.paperclip`, `doc.badge.person`, `doc.badge.phone`, `doc.badge.pin`, `doc.badge.plus.fill`, `doc.badge.sparkle`, `doc.badge.star`, `doc.badge.tag`, `doc.badge.xmark`, `doc.on.doc.circle`, `doc.on.doc.circle.fill`, `flower`, `folder.badge.arrow`, `folder.badge.arrow.down`, and the rest of the 109 in `icon-names.txt`.
- **SwiftUI built-ins.** `ShareLink`, `PasteButton`, `EditButton` and the like draw their own symbols inside the framework. They were not probed.
- **Menu bar.** Whether the menu bar draws `NSMenuItem.image` for a standard item was not confirmed.

## Appendix A. `NSImage.Name` constants

Size is in points. A template is a monochrome bitmap tinted by the control.

| Constant | Size | Kind |
|---|---|---|
| `ActionTemplate` | 19 x 19 | template |
| `AddTemplate` | 18 x 16 | template |
| `Advanced` | 32 x 32 | color |
| `ApplicationIcon` | 128 x 128 | color |
| `BluetoothTemplate` | 16 x 20 | template |
| `Bonjour` | 32 x 32 | color |
| `BookmarksTemplate` | 22 x 17 | template |
| `Caution` | 32 x 32 | color |
| `ColorPanel` | 32 x 32 | color |
| `ColumnViewTemplate` | 24 x 17 | template |
| `Computer` | 32 x 32 | color |
| `DotMac` | 32 x 32 | color |
| `EnterFullScreenTemplate` | 20 x 18 | template |
| `Everyone` | 32 x 32 | color |
| `ExitFullScreenTemplate` | 20 x 18 | template |
| `FlowViewTemplate` | 18 x 10 | template |
| `Folder` | 32 x 32 | color |
| `FolderBurnable` | 32 x 32 | color |
| `FolderSmart` | 32 x 32 | color |
| `FollowLinkFreestandingTemplate` | 19 x 19 | template |
| `FontPanel` | 32 x 32 | color |
| `GoBackTemplate` | 12 x 16 | template |
| `GoForwardTemplate` | 12 x 16 | template |
| `GoLeftTemplate` | 12 x 16 | template |
| `GoRightTemplate` | 12 x 16 | template |
| `HomeTemplate` | 24 x 20 | template |
| `IChatTheaterTemplate` | 15 x 13 | template |
| `IconViewTemplate` | 19 x 17 | template |
| `Info` | 32 x 32 | color |
| `InvalidDataFreestandingTemplate` | 19 x 19 | template |
| `LeftFacingTriangleTemplate` | 12 x 16 | template |
| `ListViewTemplate` | 21 x 14 | template |
| `LockLockedTemplate` | 16 x 18 | template |
| `LockUnlockedTemplate` | 21 x 18 | template |
| `MenuMixedStateTemplate` | 18 x 4 | template |
| `MenuOnStateTemplate` | 18 x 16 | template |
| `MobileMe` | 32 x 32 | color |
| `MultipleDocuments` | 32 x 32 | color |
| `Network` | 32 x 32 | color |
| `PathTemplate` | 25 x 14 | template |
| `PreferencesGeneral` | 32 x 32 | color |
| `QuickLookTemplate` | 27 x 16 | template |
| `RefreshFreestandingTemplate` | 19 x 19 | template |
| `RefreshTemplate` | 17 x 20 | template |
| `RemoveTemplate` | 18 x 4 | template |
| `RevealFreestandingTemplate` | 19 x 19 | template |
| `RightFacingTriangleTemplate` | 12 x 16 | template |
| `ShareTemplate` | 19 x 22 | template |
| `SlideshowTemplate` | 24 x 17 | template |
| `SmartBadgeTemplate` | 14 x 14 | template |
| `StatusAvailable` | 16 x 16 | color |
| `StatusNone` | 16 x 16 | color |
| `StatusPartiallyAvailable` | 16 x 16 | color |
| `StatusUnavailable` | 16 x 16 | color |
| `StopProgressFreestandingTemplate` | 19 x 19 | template |
| `StopProgressTemplate` | 17 x 15 | template |
| `TouchBarAddDetailTemplate` | 19 x 19 | template |
| `TouchBarAddTemplate` | 18 x 16 | template |
| `TouchBarAlarmTemplate` | 18 x 19 | template |
| `TouchBarAudioInputMuteTemplate` | 17 x 20 | template |
| `TouchBarAudioInputTemplate` | 16 x 20 | template |
| `TouchBarAudioOutputMuteTemplate` | 17 x 20 | template |
| `TouchBarAudioOutputVolume` | — | does not resolve |
| `TouchBarAudioOutputVolumeHighTemplate` | 26 x 18 | template |
| `TouchBarAudioOutputVolumeLowTemplate` | 19 x 17 | template |
| `TouchBarAudioOutputVolumeMediumTemplate` | 23 x 17 | template |
| `TouchBarAudioOutputVolumeOffTemplate` | 16 x 17 | template |
| `TouchBarBookmarksTemplate` | 22 x 17 | template |
| `TouchBarColorPickerFill` | 18 x 30 | color |
| `TouchBarColorPickerFont` | 13 x 30 | color |
| `TouchBarColorPickerStroke` | 18 x 30 | color |
| `TouchBarCommunicationAudioTemplate` | 20 x 18 | template |
| `TouchBarCommunicationVideoTemplate` | 24 x 16 | template |
| `TouchBarComposeTemplate` | 21 x 20 | template |
| `TouchBarDeleteTemplate` | 18 x 21 | template |
| `TouchBarDownloadTemplate` | 19 x 19 | template |
| `TouchBarEnterFullScreenTemplate` | 20 x 18 | template |
| `TouchBarExitFullScreenTemplate` | 20 x 18 | template |
| `TouchBarFastForwardTemplate` | 24 x 15 | template |
| `TouchBarFolderCopyToTemplate` | 25 x 18 | template |
| `TouchBarFolderMoveToTemplate` | 23 x 17 | template |
| `TouchBarFolderTemplate` | 23 x 17 | template |
| `TouchBarGetInfoTemplate` | 19 x 19 | template |
| `TouchBarGoBackTemplate` | 12 x 16 | template |
| `TouchBarGoDownTemplate` | 18 x 10 | template |
| `TouchBarGoForwardTemplate` | 12 x 16 | template |
| `TouchBarGoUpTemplate` | 18 x 10 | template |
| `TouchBarHistoryTemplate` | 19 x 19 | template |
| `TouchBarIconViewTemplate` | 19 x 17 | template |
| `TouchBarListViewTemplate` | 21 x 14 | template |
| `TouchBarMailTemplate` | 24 x 16 | template |
| `TouchBarNewFolderTemplate` | 25 x 19 | template |
| `TouchBarNewMessageTemplate` | 21 x 18 | template |
| `TouchBarOpenInBrowserTemplate` | 19 x 19 | template |
| `TouchBarPauseTemplate` | 14 x 16 | template |
| `TouchBarPlayPauseTemplate` | 28 x 15 | template |
| `TouchBarPlayTemplate` | 14 x 16 | template |
| `TouchBarPlayheadTemplate` | 7 x 30 | color |
| `TouchBarQuickLookTemplate` | 27 x 16 | template |
| `TouchBarRecordStartTemplate` | 19 x 19 | template |
| `TouchBarRecordStopTemplate` | 18 x 16 | template |
| `TouchBarRefreshTemplate` | 17 x 20 | template |
| `TouchBarRemoveTemplate` | 18 x 4 | template |
| `TouchBarRewindTemplate` | 24 x 15 | template |
| `TouchBarRotateLeftTemplate` | 19 x 20 | template |
| `TouchBarRotateRightTemplate` | 19 x 20 | template |
| `TouchBarSearchTemplate` | 20 x 19 | template |
| `TouchBarShareTemplate` | 19 x 22 | template |
| `TouchBarSidebarTemplate` | 24 x 17 | template |
| `TouchBarSkipAhead15SecondsTemplate` | 19 x 21 | template |
| `TouchBarSkipAhead30SecondsTemplate` | 19 x 21 | template |
| `TouchBarSkipAheadTemplate` | 30 x 15 | template |
| `TouchBarSkipBack15SecondsTemplate` | 19 x 21 | template |
| `TouchBarSkipBack30SecondsTemplate` | 19 x 21 | template |
| `TouchBarSkipBackTemplate` | 30 x 15 | template |
| `TouchBarSkipToEndTemplate` | 19 x 15 | template |
| `TouchBarSkipToStartTemplate` | 19 x 15 | template |
| `TouchBarSlideshowTemplate` | 24 x 17 | template |
| `TouchBarTagIconTemplate` | 23 x 21 | template |
| `TouchBarTextBoldTemplate` | 14 x 14 | template |
| `TouchBarTextBoxTemplate` | 25 x 17 | template |
| `TouchBarTextCenterAlignTemplate` | 21 x 16 | template |
| `TouchBarTextItalicTemplate` | 13 x 14 | template |
| `TouchBarTextJustifiedAlignTemplate` | 21 x 16 | template |
| `TouchBarTextLeftAlignTemplate` | 21 x 16 | template |
| `TouchBarTextListTemplate` | 21 x 14 | template |
| `TouchBarTextRightAlignTemplate` | 21 x 16 | template |
| `TouchBarTextStrikethroughTemplate` | 18 x 15 | template |
| `TouchBarTextUnderlineTemplate` | 14 x 17 | template |
| `TouchBarUserAddTemplate` | 22 x 20 | template |
| `TouchBarUserGroupTemplate` | 27 x 18 | template |
| `TouchBarUserTemplate` | 18 x 17 | template |
| `TouchBarVolume` | — | does not resolve |
| `TouchBarVolumeDownTemplate` | 19 x 17 | template |
| `TouchBarVolumeUpTemplate` | 26 x 18 | template |
| `TrashEmpty` | 32 x 32 | color |
| `TrashFull` | 32 x 32 | color |
| `User` | 32 x 32 | color |
| `UserAccounts` | 32 x 32 | color |
| `UserGroup` | 32 x 32 | color |
| `UserGuest` | 32 x 32 | color |
