use std::borrow::Cow;

use freya_core::prelude::{Modifiers, NamedKey};

use super::{Action, REDO, Shortcut, ShortcutKey, UNDO};
use crate::i18n::text;
use crate::main_menu::{MainMenu, Menu};
use crate::menu::{CheckState, MenuItem, normalize_separators};

pub const NEW_WINDOW: Action = Action::new("new-window", || text!("new-window"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('n')));
pub const CLOSE_WINDOW: Action = Action::new("close-window", || text!("window-close"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('w')));
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
pub const HELP: Action = Action::new("help", || text!("help")).shortcut(Shortcut::new(
    Modifiers::empty(),
    ShortcutKey::Named(NamedKey::F1),
));

pub(crate) const WINDOW_ACTIVATE: &str = "window-activate:";
pub(crate) const WINDOW_OPEN: &str = "window-open:";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandGroupPlacement {
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
            Menu::new(layout.app.clone(), group(&[AppSettings, AppTermination])),
            Menu::new(text!("menu-file"), group(&[NewItem, SaveItem])),
            Menu::new(
                text!("menu-edit"),
                group(&[UndoRedo, Pasteboard, TextEditing]),
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
        let mut items = Vec::new();
        for placement in placements {
            let group = self.group_items(*placement, layout);
            if !group.is_empty() && !items.is_empty() {
                items.push(MenuItem::Separator);
            }
            items.extend(group);
        }
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

fn labeled(item: MenuItem, label: String) -> MenuItem {
    match item {
        MenuItem::Action {
            id,
            shortcut,
            enabled,
            check,
            icon,
            badge,
            ..
        } => MenuItem::Action {
            id,
            label: label.into(),
            shortcut,
            enabled,
            check,
            icon,
            badge,
        },
        other => other,
    }
}

fn standard_group(
    placement: CommandGroupPlacement,
    text_editing: bool,
    layout: &Layout,
) -> Vec<MenuItem> {
    use CommandGroupPlacement::*;
    match placement {
        AppSettings => vec![SETTINGS.item(layout.settings)],
        AppTermination => vec![labeled(
            QUIT.item(true),
            text!("quit-app", app = layout.app.as_str()),
        )],
        NewItem => vec![NEW_WINDOW.item(layout.new_window)],
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

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout {
            app: "Notes".into(),
            settings: true,
            new_window: true,
            window_scenes: vec![("inspector", "Inspector".into())],
            windows: vec![(1, "Notes".into()), (2, "Notes".into())],
            active: Some(2),
            zoom: true,
        }
    }

    fn menu<'a>(menu_bar: &'a MainMenu, title: &str) -> &'a [MenuItem] {
        &menu_bar
            .menus
            .iter()
            .find(|menu| menu.title == title)
            .unwrap_or_else(|| panic!("no {title} menu"))
            .items
    }

    fn labels(items: &[MenuItem]) -> Vec<&str> {
        items
            .iter()
            .map(|item| item.label().unwrap_or("---"))
            .collect()
    }

    fn enabled(items: &[MenuItem], id: &str) -> bool {
        items
            .iter()
            .find_map(|item| match item {
                MenuItem::Action {
                    id: item_id,
                    enabled,
                    ..
                } if item_id == id => Some(*enabled),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no item {id}"))
    }

    #[test]
    fn the_standard_menus_follow_the_order_measured_on_macos() {
        let bar = Commands::new().menu_bar(&layout());
        let titles: Vec<&str> = bar.menus.iter().map(|menu| &*menu.title).collect();
        assert_eq!(
            titles,
            [
                "Notes",
                text!("menu-file"),
                text!("menu-edit"),
                text!("menu-window"),
                text!("help"),
            ]
        );
        assert_eq!(
            labels(menu(&bar, text!("menu-file"))),
            [NEW_WINDOW.title(), "---", CLOSE_WINDOW.title()]
        );
        assert_eq!(
            labels(menu(&bar, text!("menu-edit"))),
            [
                UNDO.title(),
                REDO.title(),
                "---",
                CUT.title(),
                COPY.title(),
                PASTE.title(),
                DELETE.title(),
                SELECT_ALL.title(),
            ]
        );
        assert_eq!(
            labels(menu(&bar, text!("menu-window"))),
            [
                MINIMIZE.title(),
                ZOOM.title(),
                "---",
                "Inspector",
                "---",
                BRING_ALL_TO_FRONT.title(),
                "---",
                "Notes",
                "Notes",
            ]
        );
    }

    #[test]
    fn the_app_menu_and_the_help_item_carry_the_app_name() {
        let bar = Commands::new().menu_bar(&layout());
        let app = menu(&bar, "Notes");
        assert_eq!(labels(app)[0], SETTINGS.title());
        assert_eq!(labels(app)[2], text!("quit-app", app = "Notes"));
        assert_eq!(
            labels(menu(&bar, text!("help"))),
            [text!("help-app", app = "Notes")]
        );
    }

    #[test]
    fn settings_and_new_window_are_enabled_only_when_a_scene_declares_them() {
        let mut layout = layout();
        let bar = Commands::new().menu_bar(&layout);
        assert!(enabled(menu(&bar, "Notes"), SETTINGS.id));
        assert!(enabled(menu(&bar, text!("menu-file")), NEW_WINDOW.id));
        layout.settings = false;
        layout.new_window = false;
        let bar = Commands::new().menu_bar(&layout);
        assert!(!enabled(menu(&bar, "Notes"), SETTINGS.id));
        assert!(!enabled(menu(&bar, text!("menu-file")), NEW_WINDOW.id));
    }

    #[test]
    fn the_items_that_need_a_handler_start_disabled_and_the_ones_the_kit_serves_do_not() {
        let bar = Commands::new().menu_bar(&layout());
        let edit = menu(&bar, text!("menu-edit"));
        for action in [UNDO, REDO, CUT, COPY, PASTE, DELETE, SELECT_ALL] {
            assert!(!enabled(edit, action.id), "{}", action.id);
        }
        assert!(!enabled(menu(&bar, text!("help")), HELP.id));
        assert!(enabled(menu(&bar, text!("menu-file")), CLOSE_WINDOW.id));
        assert!(enabled(menu(&bar, "Notes"), QUIT.id));
        let window = menu(&bar, text!("menu-window"));
        assert!(enabled(window, MINIMIZE.id));
        assert!(enabled(window, ZOOM.id));
    }

    #[test]
    fn zoom_is_disabled_for_a_window_that_cannot_resize() {
        let layout = Layout {
            zoom: false,
            ..layout()
        };
        assert!(!enabled(&Commands::new().window_items(&layout), ZOOM.id));
    }

    #[test]
    fn the_window_list_checks_the_active_window() {
        let items = Commands::new().window_items(&layout());
        let checks: Vec<(String, bool)> = items
            .iter()
            .filter_map(|item| match item {
                MenuItem::Action { id, check, .. } if id.starts_with(WINDOW_ACTIVATE) => {
                    Some((id.clone(), *check == CheckState::Checkbox(true)))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            checks,
            [
                ("window-activate:1".to_string(), false),
                ("window-activate:2".to_string(), true)
            ]
        );
    }

    #[test]
    fn a_group_goes_before_after_or_in_place_of_a_placement() {
        use CommandGroupPlacement::NewItem;
        let item = |id: &'static str| MenuItem::action(id, id);
        let ids = |commands: Commands| {
            let bar = commands.menu_bar(&layout());
            labels(menu(&bar, text!("menu-file")))
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        };
        let new_window = NEW_WINDOW.title().to_string();
        assert_eq!(
            ids(Commands::new().group(CommandGroup::after(NewItem, vec![item("note")]))),
            [new_window.as_str(), "note", "---", CLOSE_WINDOW.title()]
        );
        assert_eq!(
            ids(Commands::new().group(CommandGroup::before(NewItem, vec![item("note")]))),
            ["note", new_window.as_str(), "---", CLOSE_WINDOW.title()]
        );
        assert_eq!(
            ids(Commands::new().group(CommandGroup::replacing(NewItem, vec![item("note")]))),
            ["note", "---", CLOSE_WINDOW.title()]
        );
    }

    #[test]
    fn a_menu_left_without_items_is_not_published() {
        use CommandGroupPlacement::{AppSettings, AppTermination};
        let commands = Commands::new()
            .group(CommandGroup::replacing(AppSettings, Vec::new()))
            .group(CommandGroup::replacing(AppTermination, Vec::new()));
        let bar = commands.menu_bar(&layout());
        assert!(bar.menus.iter().all(|menu| menu.title != "Notes"));
    }

    #[test]
    fn a_command_menu_sits_between_edit_and_window() {
        let bar = Commands::new()
            .menu(Menu::new(
                "Notes tools",
                vec![MenuItem::action("tool", "Tool")],
            ))
            .menu_bar(&layout());
        let titles: Vec<&str> = bar.menus.iter().map(|menu| &*menu.title).collect();
        assert_eq!(
            titles[2..5],
            [text!("menu-edit"), "Notes tools", text!("menu-window")]
        );
    }

    #[test]
    fn text_editing_adds_the_find_submenu_with_the_measured_items() {
        let plain = Commands::new().menu_bar(&layout());
        assert!(
            !menu(&plain, text!("menu-edit"))
                .iter()
                .any(|item| item.label() == Some(text!("find")))
        );
        let bar = Commands::new().text_editing().menu_bar(&layout());
        let find = menu(&bar, text!("menu-edit"))
            .iter()
            .find_map(|item| match item {
                MenuItem::Submenu { label, items, .. } if label == text!("find") => Some(items),
                _ => None,
            })
            .expect("a Find submenu");
        let shortcuts: Vec<Option<String>> = find
            .iter()
            .map(|item| match item {
                MenuItem::Action { shortcut, .. } => shortcut.as_ref().map(|s| s.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(
            shortcuts,
            [
                Some("Ctrl+F".to_string()),
                Some("Ctrl+Alt+F".to_string()),
                Some("Ctrl+G".to_string()),
                Some("Ctrl+Shift+G".to_string()),
                Some("Ctrl+E".to_string()),
                Some("Ctrl+J".to_string()),
            ]
        );
        assert!(
            find.iter()
                .all(|item| matches!(item, MenuItem::Action { enabled: false, .. }))
        );
    }

    #[test]
    fn the_standard_shortcuts_map_command_to_control() {
        let text = |action: Action| action.shortcut.map(|shortcut| shortcut.text());
        assert_eq!(text(NEW_WINDOW).as_deref(), Some("Ctrl+N"));
        assert_eq!(text(CLOSE_WINDOW).as_deref(), Some("Ctrl+W"));
        assert_eq!(text(SETTINGS).as_deref(), Some("Ctrl+,"));
        assert_eq!(text(QUIT).as_deref(), Some("Ctrl+Q"));
        assert_eq!(text(MINIMIZE).as_deref(), Some("Ctrl+M"));
        assert_eq!(text(HELP).as_deref(), Some("F1"));
        assert_eq!(text(ZOOM), None);
        assert_eq!(text(DELETE), None);
    }
}
