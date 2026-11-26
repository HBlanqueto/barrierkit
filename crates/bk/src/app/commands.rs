use std::borrow::Cow;
use std::iter;

use freya_core::prelude::{Modifiers, NamedKey};

use crate::app::{Action, REDO, Shortcut, ShortcutKey, UNDO};
use crate::i18n::text;
use crate::main_menu::{MainMenu, Menu};
use crate::menu::{CheckState, MenuItem, normalize_separators};

pub const NEW_WINDOW: Action = Action::new("new-window", || text!("new-window"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('n')));
pub const CLOSE_WINDOW: Action = Action::new("close-window", || text!("window-close"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('w')));
pub const ABOUT: Action = Action::new("about", || text!("about"));
pub const SETTINGS: Action = Action::new("settings", || text!("settings-item"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char(',')));
pub const QUIT: Action =
    Action::new("quit", || text!("quit")).shortcut(Shortcut::ctrl(ShortcutKey::Char('q')));
pub const CUT: Action =
    Action::new("cut", || text!("cut")).shortcut(Shortcut::ctrl(ShortcutKey::Char('x')));
pub const COPY: Action =
    Action::new("copy", || text!("copy")).shortcut(Shortcut::ctrl(ShortcutKey::Char('c')));
pub const PASTE: Action =
    Action::new("paste", || text!("paste")).shortcut(Shortcut::ctrl(ShortcutKey::Char('v')));
pub const DELETE: Action = Action::new("delete", || text!("delete"));
pub const SELECT_ALL: Action = Action::new("select-all", || text!("select-all"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('a')));
pub const FIND: Action =
    Action::new("find", || text!("find-item")).shortcut(Shortcut::ctrl(ShortcutKey::Char('f')));
pub const FIND_AND_REPLACE: Action = Action::new("find-and-replace", || text!("find-and-replace"))
    .shortcut(Shortcut::new(
        Modifiers::CONTROL.union(Modifiers::ALT),
        ShortcutKey::Char('f'),
    ));
pub const FIND_NEXT: Action = Action::new("find-next", || text!("find-next-item"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('g')));
pub const FIND_PREVIOUS: Action = Action::new("find-previous", || text!("find-previous-item"))
    .shortcut(Shortcut::new(
        Modifiers::CONTROL.union(Modifiers::SHIFT),
        ShortcutKey::Char('g'),
    ));
pub const USE_SELECTION_FOR_FIND: Action =
    Action::new("use-selection-for-find", || text!("use-selection-for-find"))
        .shortcut(Shortcut::ctrl(ShortcutKey::Char('e')));
pub const JUMP_TO_SELECTION: Action =
    Action::new("jump-to-selection", || text!("jump-to-selection"))
        .shortcut(Shortcut::ctrl(ShortcutKey::Char('j')));
pub const MINIMIZE: Action = Action::new("minimize", || text!("window-minimize"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('m')));
pub const ZOOM: Action = Action::new("zoom", || text!("window-zoom"));
pub const BRING_ALL_TO_FRONT: Action =
    Action::new("bring-all-to-front", || text!("bring-all-to-front"));
pub const TOGGLE_FULL_SCREEN: Action =
    Action::new("toggle-full-screen", || text!("enter-full-screen")).shortcut(Shortcut::new(
        Modifiers::empty(),
        ShortcutKey::Named(NamedKey::F11),
    ));
pub const HELP: Action = Action::new("help", || text!("help")).shortcut(Shortcut::new(
    Modifiers::empty(),
    ShortcutKey::Named(NamedKey::F1),
));

pub(crate) const WINDOW_ACTIVATE: &str = "window-activate:";
pub(crate) const WINDOW_OPEN: &str = "window-open:";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandGroupPlacement {
    AppInfo,
    AppSettings,
    AppTermination,
    NewItem,
    SaveItem,
    UndoRedo,
    Pasteboard,
    TextEditing,
    WindowSize,
    SingleWindowList,
    WindowArrangement,
    WindowList,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Position {
    Before,
    After,
    Replacing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandGroup {
    placement: CommandGroupPlacement,
    position: Position,
    items: Vec<MenuItem>,
}

impl CommandGroup {
    pub fn before(placement: CommandGroupPlacement, items: Vec<MenuItem>) -> Self {
        Self::at(placement, Position::Before, items)
    }

    pub fn after(placement: CommandGroupPlacement, items: Vec<MenuItem>) -> Self {
        Self::at(placement, Position::After, items)
    }

    pub fn replacing(placement: CommandGroupPlacement, items: Vec<MenuItem>) -> Self {
        Self::at(placement, Position::Replacing, items)
    }

    fn at(placement: CommandGroupPlacement, position: Position, items: Vec<MenuItem>) -> Self {
        Self {
            placement,
            position,
            items,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Commands {
    text_editing: bool,
    groups: Vec<CommandGroup>,
    menus: Vec<Menu>,
}

pub(crate) struct Layout {
    pub(crate) app: String,
    pub(crate) settings: bool,
    pub(crate) new_window: bool,
    pub(crate) window_scenes: Vec<(&'static str, Cow<'static, str>)>,
    pub(crate) windows: Vec<(u32, Cow<'static, str>)>,
    pub(crate) active: Option<u32>,
    pub(crate) zoom: bool,
}

impl Commands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text_editing(mut self) -> Self {
        self.text_editing = true;
        self
    }

    pub fn group(mut self, group: CommandGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    pub(crate) fn menu_bar(&self, layout: &Layout) -> MainMenu {
        use CommandGroupPlacement::*;
        let group = |placements: &[CommandGroupPlacement]| self.items(placements, layout);
        let mut menus = vec![
            Menu::new(
                layout.app.clone(),
                group(&[AppInfo, AppSettings, AppTermination]),
            ),
            Menu::new(text!("menu-file"), group(&[NewItem, SaveItem])),
            Menu::new(
                text!("menu-edit"),
                group(&[UndoRedo, Pasteboard, TextEditing]),
            ),
            Menu::new(
                text!("menu-view"),
                vec![TOGGLE_FULL_SCREEN.item(layout.zoom)],
            ),
        ];
        menus.extend(self.menus.iter().cloned());
        menus.push(Menu::new(text!("menu-window"), self.window_items(layout)));
        menus.push(Menu::new(text!("help"), group(&[Help])));
        menus.retain(|menu| !menu.items.is_empty());
        MainMenu::new(menus)
    }

    pub(crate) fn window_items(&self, layout: &Layout) -> Vec<MenuItem> {
        use CommandGroupPlacement::*;
        self.items(
            &[WindowSize, SingleWindowList, WindowArrangement, WindowList],
            layout,
        )
    }

    fn items(&self, placements: &[CommandGroupPlacement], layout: &Layout) -> Vec<MenuItem> {
        let mut items = placements
            .iter()
            .flat_map(|placement| {
                iter::once(MenuItem::Separator).chain(self.group_items(*placement, layout))
            })
            .collect();
        normalize_separators(&mut items);
        items
    }

    fn group_items(&self, placement: CommandGroupPlacement, layout: &Layout) -> Vec<MenuItem> {
        let mut items = standard_group(placement, self.text_editing, layout);
        for group in self
            .groups
            .iter()
            .filter(|group| group.placement == placement)
        {
            match group.position {
                Position::Before => {
                    items.splice(0..0, group.items.clone());
                }
                Position::After => items.extend(group.items.clone()),
                Position::Replacing => items = group.items.clone(),
            }
        }
        items
    }
}

fn labeled(mut item: MenuItem, label: String) -> MenuItem {
    if let MenuItem::Action { label: shown, .. } = &mut item {
        *shown = label.into();
    }
    item
}

fn standard_group(
    placement: CommandGroupPlacement,
    text_editing: bool,
    layout: &Layout,
) -> Vec<MenuItem> {
    use CommandGroupPlacement::*;
    match placement {
        AppInfo => vec![labeled(
            ABOUT.item(true),
            text!("about-app", app = layout.app.as_str()),
        )],
        AppSettings => Vec::from_iter(layout.settings.then(|| SETTINGS.item(true))),
        AppTermination => vec![labeled(
            QUIT.item(true),
            text!("quit-app", app = layout.app.as_str()),
        )],
        NewItem => Vec::from_iter(layout.new_window.then(|| NEW_WINDOW.item(true))),
        SaveItem => vec![CLOSE_WINDOW.item(true)],
        UndoRedo => vec![UNDO.item(false), REDO.item(false)],
        Pasteboard => [CUT, COPY, PASTE, DELETE, SELECT_ALL]
            .iter()
            .map(|action| action.item(false))
            .collect(),
        TextEditing if text_editing => vec![MenuItem::submenu(
            text!("find"),
            [
                FIND,
                FIND_AND_REPLACE,
                FIND_NEXT,
                FIND_PREVIOUS,
                USE_SELECTION_FOR_FIND,
                JUMP_TO_SELECTION,
            ]
            .iter()
            .map(|action| action.item(false))
            .collect(),
        )],
        TextEditing => Vec::new(),
        WindowSize => vec![MINIMIZE.item(true), ZOOM.item(layout.zoom)],
        SingleWindowList => layout
            .window_scenes
            .iter()
            .map(|(id, title)| MenuItem::action(format!("{WINDOW_OPEN}{id}"), title.clone()))
            .collect(),
        WindowArrangement => vec![BRING_ALL_TO_FRONT.item(true)],
        WindowList => layout
            .windows
            .iter()
            .map(|(number, title)| {
                MenuItem::action(format!("{WINDOW_ACTIVATE}{number}"), title.clone())
                    .with_check(CheckState::Checkbox(layout.active == Some(*number)))
            })
            .collect(),
        Help => vec![labeled(
            HELP.item(false),
            text!("help-app", app = layout.app.as_str()),
        )],
    }
}
