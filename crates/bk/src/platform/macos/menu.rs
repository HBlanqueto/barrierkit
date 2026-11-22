use std::cell::RefCell;
use std::collections::HashMap;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSControlStateValueMixed, NSControlStateValueOff, NSControlStateValueOn,
    NSEventModifierFlags, NSMenu, NSMenuItem,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};

use super::with_runtime;
use crate::main_menu::MenuHandle;
use crate::menu::{CheckState, MenuItem, shortcut_keys};

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Off,
    On,
    Mixed,
}

#[derive(Debug, PartialEq)]
struct KeyEquivalent {
    key: String,
    command: bool,
    option: bool,
    control: bool,
    shift: bool,
}

#[derive(Debug, PartialEq)]
enum Entry {
    Action {
        id: String,
        title: String,
        key: Option<KeyEquivalent>,
        enabled: bool,
        state: State,
    },
    Submenu {
        title: String,
        enabled: bool,
        items: Vec<Entry>,
    },
    Header {
        title: String,
    },
    Separator,
}

// AppKit's function-key characters are in the Unicode private use area (NSEvent.h).
fn key_character(keysym: &str) -> Option<String> {
    let function = |code: u32| char::from_u32(code).map(String::from);
    if let Some(number) = keysym
        .strip_prefix('F')
        .and_then(|digits| digits.parse::<u32>().ok())
    {
        return (1..=12)
            .contains(&number)
            .then(|| function(0xF703 + number))
            .flatten();
    }
    let named = match keysym {
        "Escape" => "\u{1b}",
        "Return" => "\r",
        "Tab" => "\t",
        "space" => " ",
        "BackSpace" => "\u{8}",
        "Delete" => return function(0xF728),
        "Left" => return function(0xF702),
        "Right" => return function(0xF703),
        "Up" => return function(0xF700),
        "Down" => return function(0xF701),
        "Home" => return function(0xF729),
        "End" => return function(0xF72B),
        "Page_Up" => return function(0xF72C),
        "Page_Down" => return function(0xF72D),
        "comma" => ",",
        "period" => ".",
        "minus" => "-",
        "plus" => "+",
        "equal" => "=",
        "slash" => "/",
        "question" => "?",
        "backslash" => "\\",
        "semicolon" => ";",
        "apostrophe" => "'",
        "bracketleft" => "[",
        "bracketright" => "]",
        single if single.chars().count() == 1 => return Some(single.to_lowercase()),
        _ => return None,
    };
    Some(named.to_string())
}

// The kit writes Ctrl for the primary key, which is Command here; Super has no Mac key, so it is Control.
fn key_equivalent(shortcut: &str) -> Option<KeyEquivalent> {
    let mut parts = shortcut_keys(shortcut);
    let key = key_character(&parts.pop()?)?;
    let held = |name: &str| parts.iter().any(|part| part == name);
    Some(KeyEquivalent {
        key,
        command: held("Control"),
        option: held("Alt"),
        control: held("Super"),
        shift: held("Shift"),
    })
}

fn state(check: &CheckState) -> State {
    match check {
        CheckState::Checkbox(true) | CheckState::Radio(true) => State::On,
        CheckState::Mixed => State::Mixed,
        _ => State::Off,
    }
}

fn entries(items: &[MenuItem]) -> Vec<Entry> {
    items
        .iter()
        .filter_map(|item| match item {
            MenuItem::Action {
                id,
                label,
                shortcut,
                enabled,
                check,
                ..
            } => Some(Entry::Action {
                id: id.clone(),
                title: label.to_string(),
                key: shortcut.as_deref().and_then(key_equivalent),
                enabled: *enabled,
                state: state(check),
            }),
            MenuItem::Submenu {
                label,
                enabled,
                items,
                ..
            } => Some(Entry::Submenu {
                title: label.to_string(),
                enabled: *enabled,
                items: entries(items),
            }),
            MenuItem::Header { label, .. } => Some(Entry::Header {
                title: label.to_string(),
            }),
            MenuItem::Separator => Some(Entry::Separator),
            MenuItem::Palette { .. }
            | MenuItem::SearchField { .. }
            | MenuItem::SearchResult { .. } => None,
        })
        .collect()
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKMenuTarget"]
    struct Target;

    unsafe impl NSObjectProtocol for Target {}

    impl Target {
        #[unsafe(method(fire:))]
        fn fire(&self, item: &NSMenuItem) {
            if let Some(id) = item
                .representedObject()
                .and_then(|object| object.downcast::<NSString>().ok())
            {
                fire(&id.to_string());
            }
        }
    }
);

impl Target {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSObject's designated initializer.
        unsafe { msg_send![super(this), init] }
    }
}

struct Installed {
    mtm: MainThreadMarker,
    main: Retained<NSMenu>,
    target: Retained<Target>,
    handles: HashMap<u64, MenuHandle>,
    shown: Option<Shown>,
}

struct Shown {
    owner: u64,
    revision: u32,
}

thread_local! {
    static MENU: RefCell<Option<Installed>> = const { RefCell::new(None) };
}

fn text(value: &str) -> Retained<NSString> {
    NSString::from_str(value)
}

fn app_item(
    mtm: MainThreadMarker,
    title: &str,
    action: objc2::runtime::Sel,
    key: &str,
    mask: NSEventModifierFlags,
) -> Retained<NSMenuItem> {
    // SAFETY: the selectors are NSApplication's own actions, reached through the responder chain.
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &text(title),
            Some(action),
            &text(key),
        )
    };
    item.setKeyEquivalentModifierMask(mask);
    item
}

fn app_menu(mtm: MainThreadMarker, name: &str) -> Retained<NSMenu> {
    let command = NSEventModifierFlags::Command;
    let menu = NSMenu::new(mtm);
    let add = |item: Retained<NSMenuItem>| menu.addItem(&item);
    add(app_item(
        mtm,
        &crate::i18n::text!("about-app", app = name),
        sel!(orderFrontStandardAboutPanel:),
        "",
        command,
    ));
    add(NSMenuItem::separatorItem(mtm));
    add(app_item(
        mtm,
        &crate::i18n::text!("hide-app", app = name),
        sel!(hide:),
        "h",
        command,
    ));
    add(app_item(
        mtm,
        crate::i18n::text!("hide-others"),
        sel!(hideOtherApplications:),
        "h",
        command | NSEventModifierFlags::Option,
    ));
    add(app_item(
        mtm,
        crate::i18n::text!("show-all"),
        sel!(unhideAllApplications:),
        "",
        command,
    ));
    add(NSMenuItem::separatorItem(mtm));
    add(app_item(
        mtm,
        &crate::i18n::text!("quit-app", app = name),
        sel!(terminate:),
        "q",
        command,
    ));
    menu
}

pub(super) fn install(mtm: MainThreadMarker, app: &NSApplication, name: &str) {
    let main = NSMenu::new(mtm);
    main.setAutoenablesItems(false);
    let item = NSMenuItem::new(mtm);
    item.setSubmenu(Some(&app_menu(mtm, name)));
    main.addItem(&item);
    app.setMainMenu(Some(&main));
    MENU.with(|slot| {
        *slot.borrow_mut() = Some(Installed {
            mtm,
            main,
            target: Target::new(mtm),
            handles: HashMap::new(),
            shown: None,
        });
    });
}

fn modifier_mask(key: &KeyEquivalent) -> NSEventModifierFlags {
    let mut mask = NSEventModifierFlags::empty();
    mask.set(NSEventModifierFlags::Command, key.command);
    mask.set(NSEventModifierFlags::Option, key.option);
    mask.set(NSEventModifierFlags::Control, key.control);
    mask.set(NSEventModifierFlags::Shift, key.shift);
    mask
}

fn configure(installed: &Installed, item: &NSMenuItem, entry: &Entry) {
    let (title, enabled, id, key, state) = match entry {
        Entry::Action {
            id,
            title,
            key,
            enabled,
            state,
        } => (title, *enabled, Some(id), key.as_ref(), *state),
        Entry::Submenu { title, enabled, .. } => (title, *enabled, None, None, State::Off),
        Entry::Header { title } => (title, false, None, None, State::Off),
        Entry::Separator => return,
    };
    item.setTitle(&text(title));
    item.setEnabled(enabled);
    item.setState(match state {
        State::Off => NSControlStateValueOff,
        State::On => NSControlStateValueOn,
        State::Mixed => NSControlStateValueMixed,
    });
    item.setKeyEquivalent(&text(key.map_or("", |key| key.key.as_str())));
    item.setKeyEquivalentModifierMask(key.map_or(NSEventModifierFlags::empty(), modifier_mask));
    let represented = id.map(|id| text(id));
    let (action, target) = match id {
        Some(_) => (Some(sel!(fire:)), Some(&*installed.target as &AnyObject)),
        None => (None, None),
    };
    // SAFETY: `fire:` is `Target`'s action, `Target` lives in `Installed` as long as the menu, and the represented object is an NSString.
    unsafe {
        item.setRepresentedObject(represented.as_deref().map(|id| -> &AnyObject { id }));
        item.setAction(action);
        item.setTarget(target);
    }
    if let Entry::Submenu { title, items, .. } = entry {
        let submenu = item.submenu().unwrap_or_else(|| {
            let submenu = NSMenu::initWithTitle(NSMenu::alloc(installed.mtm), &text(title));
            submenu.setAutoenablesItems(false);
            item.setSubmenu(Some(&submenu));
            submenu
        });
        submenu.setTitle(&text(title));
        apply(installed, &submenu, 0, items);
    }
}

fn same_kind(item: &NSMenuItem, entry: &Entry) -> bool {
    match entry {
        Entry::Separator => item.isSeparatorItem(),
        Entry::Submenu { .. } => item.hasSubmenu(),
        Entry::Action { .. } | Entry::Header { .. } => {
            !item.isSeparatorItem() && !item.hasSubmenu()
        }
    }
}

// Items that keep their kind are updated in place, so a menu that is open does not lose its items.
fn apply(installed: &Installed, menu: &NSMenu, skip: usize, entries: &[Entry]) {
    let count = menu.numberOfItems() as usize;
    let item = |index: usize| menu.itemAtIndex((skip + index) as isize);
    let in_place = count == skip + entries.len()
        && entries
            .iter()
            .enumerate()
            .all(|(index, entry)| item(index).is_some_and(|item| same_kind(&item, entry)));
    if !in_place {
        for _ in skip..count {
            menu.removeItemAtIndex(skip as isize);
        }
        for entry in entries {
            let item = match entry {
                Entry::Separator => NSMenuItem::separatorItem(installed.mtm),
                _ => NSMenuItem::new(installed.mtm),
            };
            menu.addItem(&item);
        }
    }
    for (index, entry) in entries.iter().enumerate() {
        if let Some(item) = item(index) {
            configure(installed, &item, entry);
        }
    }
}

fn show_in(installed: &mut Installed, owner: u64) {
    let Some(handle) = installed.handles.get(&owner) else {
        return;
    };
    let (menu, revision) = handle.snapshot();
    let entries: Vec<Entry> = menu
        .menus
        .iter()
        .map(|menu| Entry::Submenu {
            title: menu.title.to_string(),
            enabled: true,
            items: entries(&menu.items),
        })
        .collect();
    apply(installed, &installed.main, 1, &entries);
    installed.shown = Some(Shown { owner, revision });
}

pub(super) fn register(window: u64, handle: MenuHandle) {
    MENU.with(|slot| {
        if let Some(installed) = slot.borrow_mut().as_mut() {
            installed.handles.insert(window, handle);
            show_in(installed, window);
        }
    });
}

pub(super) fn show(window: u64) {
    MENU.with(|slot| {
        if let Some(installed) = slot.borrow_mut().as_mut()
            && installed.shown.as_ref().map(|shown| shown.owner) != Some(window)
        {
            show_in(installed, window);
        }
    });
}

pub(super) fn forget(window: u64) {
    MENU.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(installed) = slot.as_mut() else {
            return;
        };
        installed.handles.remove(&window);
        if installed.shown.as_ref().map(|shown| shown.owner) == Some(window) {
            match installed.handles.keys().min().copied() {
                Some(next) => show_in(installed, next),
                None => {
                    apply(installed, &installed.main, 1, &[]);
                    installed.shown = None;
                }
            }
        }
    });
}

pub(super) fn sync() {
    MENU.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(installed) = slot.as_mut() else {
            return;
        };
        let Some(Shown { owner, revision }) = installed.shown else {
            return;
        };
        if installed
            .handles
            .get(&owner)
            .is_some_and(|handle| handle.revision() != revision)
        {
            show_in(installed, owner);
        }
    });
}

fn fire(id: &str) {
    let Some(owner) = MENU.with(|slot| {
        slot.borrow()
            .as_ref()?
            .shown
            .as_ref()
            .map(|shown| shown.owner)
    }) else {
        return;
    };
    let Some(actions) = with_runtime(|runtime| {
        runtime
            .window(owner)
            .map(|window| window.app.menu_actions.clone())
    })
    .flatten() else {
        return;
    };
    actions.dispatch(id);
    with_runtime(|runtime| {
        runtime.poll_all();
        runtime.install_timers();
        runtime.redraw_all();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::MenuItem;

    fn key(shortcut: &str) -> KeyEquivalent {
        key_equivalent(shortcut).expect("the shortcut has a key")
    }

    fn action(shortcut: Option<&'static str>, enabled: bool, check: CheckState) -> MenuItem {
        MenuItem::Action {
            id: "id".into(),
            label: "Label".into(),
            shortcut: shortcut.map(Into::into),
            enabled,
            check,
            icon: None,
            badge: None,
        }
    }

    #[test]
    fn ctrl_becomes_command() {
        let save = key("Ctrl+S");
        assert!(save.command && !save.control && !save.option && !save.shift);
        assert_eq!(save.key, "s");
    }

    #[test]
    fn shift_and_alt_keep_their_meaning() {
        let save_as = key("Ctrl+Shift+S");
        assert!(save_as.command && save_as.shift && !save_as.option);
        let option = key("Ctrl+Alt+K");
        assert!(option.command && option.option && !option.shift);
    }

    #[test]
    fn super_becomes_control_because_the_mac_has_no_super_key() {
        let key = key("Super+L");
        assert!(key.control && !key.command);
    }

    #[test]
    fn letters_are_lowercase_so_shift_is_only_the_modifier() {
        assert_eq!(key("Ctrl+Shift+Z").key, "z");
    }

    #[test]
    fn arrows_use_appkit_function_characters() {
        assert_eq!(key("Ctrl+Left").key, "\u{f702}");
        assert_eq!(key("Ctrl+Up").key, "\u{f700}");
    }

    #[test]
    fn function_keys_count_from_f1() {
        assert_eq!(key("F1").key, "\u{f704}");
        assert_eq!(key("F12").key, "\u{f70f}");
        assert!(key_equivalent("F13").is_none());
    }

    #[test]
    fn punctuation_names_become_their_character() {
        assert_eq!(key("Ctrl+comma").key, ",");
        assert_eq!(key("Ctrl+,").key, ",");
        assert_eq!(key("Ctrl+bracketleft").key, "[");
    }

    #[test]
    fn return_escape_tab_and_space_are_control_characters() {
        assert_eq!(key("Ctrl+Return").key, "\r");
        assert_eq!(key("Escape").key, "\u{1b}");
        assert_eq!(key("Ctrl+Tab").key, "\t");
        assert_eq!(key("Ctrl+space").key, " ");
    }

    #[test]
    fn a_shortcut_without_a_key_has_no_equivalent() {
        assert!(key_equivalent("Ctrl+Shift").is_none());
        assert!(key_equivalent("").is_none());
    }

    #[test]
    fn an_unknown_key_name_has_no_equivalent() {
        assert!(key_equivalent("Ctrl+Insert").is_none());
    }

    #[test]
    fn checkbox_and_radio_are_on_only_when_active() {
        assert_eq!(state(&CheckState::Checkbox(true)), State::On);
        assert_eq!(state(&CheckState::Radio(true)), State::On);
        assert_eq!(state(&CheckState::Checkbox(false)), State::Off);
        assert_eq!(state(&CheckState::Radio(false)), State::Off);
    }

    #[test]
    fn mixed_shows_the_mixed_mark_and_the_rest_show_nothing() {
        assert_eq!(state(&CheckState::Mixed), State::Mixed);
        assert_eq!(state(&CheckState::None), State::Off);
        assert_eq!(state(&CheckState::Minimized), State::Off);
    }

    #[test]
    fn a_disabled_action_stays_disabled() {
        let [Entry::Action { enabled, .. }] =
            entries(&[action(Some("Ctrl+S"), false, CheckState::None)])[..]
        else {
            panic!("one action");
        };
        assert!(!enabled);
    }

    #[test]
    fn an_action_keeps_its_id_title_key_and_state() {
        let entry = entries(&[action(Some("Ctrl+S"), true, CheckState::Checkbox(true))]);
        assert_eq!(
            entry,
            vec![Entry::Action {
                id: "id".into(),
                title: "Label".into(),
                key: Some(key("Ctrl+S")),
                enabled: true,
                state: State::On,
            }]
        );
    }

    #[test]
    fn submenus_keep_their_children_and_their_enabled_flag() {
        let items = [MenuItem::Submenu {
            id: None,
            label: "Open Recent".into(),
            enabled: false,
            items: vec![MenuItem::Separator],
            icon: None,
        }];
        assert_eq!(
            entries(&items),
            vec![Entry::Submenu {
                title: "Open Recent".into(),
                enabled: false,
                items: vec![Entry::Separator],
            }]
        );
    }

    #[test]
    fn palettes_and_search_fields_have_no_native_counterpart() {
        let items = [
            MenuItem::palette(vec![]),
            MenuItem::SearchField {
                placeholder: "Search".into(),
            },
            MenuItem::Separator,
        ];
        assert_eq!(entries(&items), vec![Entry::Separator]);
    }
}
