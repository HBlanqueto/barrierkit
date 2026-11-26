use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::PoisonError;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSControlStateValueMixed, NSControlStateValueOff, NSControlStateValueOn,
    NSEventModifierFlags, NSMenu, NSMenuItem,
};
use objc2_foundation::{NSBundle, NSObject, NSObjectProtocol, NSString, ns_string};

use super::with_runtime;
use crate::app::{BRING_ALL_TO_FRONT, HELP, MINIMIZE, QUIT, SETTINGS, TOGGLE_FULL_SCREEN, ZOOM};
use crate::main_menu::{Menu, MenuHandle, field_edit_key};
use crate::menu::{CheckState, MenuItem, shortcut_keys};
use crate::status_item::STATUS_ITEM_ACTIVATE;

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
            MenuItem::Palette { .. } => None,
        })
        .collect()
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKMenuTarget"]
    #[ivars = Option<u64>]
    pub(super) struct Target;

    // SAFETY: Target is an NSObject subclass.
    unsafe impl NSObjectProtocol for Target {}

    impl Target {
        #[unsafe(method(fire:))]
        fn fire(&self, item: &NSMenuItem) {
            if let Some(id) = item
                .representedObject()
                .and_then(|object| object.downcast::<NSString>().ok())
            {
                fire(&id.to_string(), *self.ivars());
            }
        }

        #[unsafe(method(activate:))]
        fn activate(&self, _: &AnyObject) {
            fire(STATUS_ITEM_ACTIVATE, *self.ivars());
        }

        // The tag holds the kit's enabled state; a focused text field enables the Edit items it serves.
        #[unsafe(method(validateMenuItem:))]
        fn validate_menu_item(&self, item: &NSMenuItem) -> bool {
            item.tag() != 0
                || item
                    .representedObject()
                    .and_then(|object| object.downcast::<NSString>().ok())
                    .is_some_and(|id| field_serves(&id.to_string(), *self.ivars()))
        }
    }
);

impl Target {
    pub(super) fn new(mtm: MainThreadMarker, owner: Option<u64>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(owner);
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

// Items 2 and 3 are Settings… and its separator, shown while the window's menu has `SETTINGS`.
fn app_menu(
    mtm: MainThreadMarker,
    app: &NSApplication,
    target: &Target,
    name: &str,
) -> Retained<NSMenu> {
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
    let settings = app_item(mtm, SETTINGS.title(), sel!(fire:), ",", command);
    settings.setTag(1);
    // SAFETY: `fire:` is `Target`'s action and the target lives in `Installed` as long as the menu.
    unsafe {
        settings.setRepresentedObject(Some(&text(SETTINGS.id)));
        settings.setTarget(Some(target));
    }
    add(settings);
    add(NSMenuItem::separatorItem(mtm));
    let services = NSMenu::initWithTitle(NSMenu::alloc(mtm), &text(crate::i18n::text!("services")));
    let item = NSMenuItem::new(mtm);
    item.setTitle(&text(crate::i18n::text!("services")));
    item.setSubmenu(Some(&services));
    app.setServicesMenu(Some(&services));
    add(item);
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

// The menu bar names the app after its bundle, as AppKit does, and falls back to the app id.
fn bundle_name() -> Option<String> {
    NSBundle::mainBundle()
        .objectForInfoDictionaryKey(ns_string!("CFBundleName"))
        .and_then(|name| name.downcast::<NSString>().ok())
        .map(|name| name.to_string())
}

pub(super) fn install(mtm: MainThreadMarker, app: &NSApplication, name: &str) {
    let name = bundle_name().unwrap_or_else(|| name.to_owned());
    let main = NSMenu::new(mtm);
    main.setAutoenablesItems(false);
    let target = Target::new(mtm, None);
    let item = NSMenuItem::new(mtm);
    item.setSubmenu(Some(&app_menu(mtm, app, &target, &name)));
    main.addItem(&item);
    app.setMainMenu(Some(&main));
    MENU.with(|slot| {
        *slot.borrow_mut() = Some(Installed {
            mtm,
            main,
            target,
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
    item.setTag(enabled.into());
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
    if let Some((action, key, mask)) = id.and_then(|id| native(id, enabled)) {
        // SAFETY: NSWindow and NSApplication answer these selectors through the responder chain.
        unsafe {
            item.setAction(Some(action));
            item.setTarget(None);
        }
        item.setKeyEquivalent(&text(key));
        item.setKeyEquivalentModifierMask(mask);
        if id.is_some_and(|id| id == HELP.id)
            && let Some(name) = bundle_name()
        {
            item.setTitle(&text(&crate::i18n::text!("help-app", app = name)));
        }
    }
    if let Entry::Submenu { title, items, .. } = entry {
        let submenu = item.submenu().unwrap_or_else(|| {
            let submenu = NSMenu::initWithTitle(NSMenu::alloc(installed.mtm), &text(title));
            submenu.setAutoenablesItems(true);
            item.setSubmenu(Some(&submenu));
            submenu
        });
        submenu.setTitle(&text(title));
        apply(installed, &submenu, 0, items);
    }
}

// AppKit serves these as a Swift app's: it validates them, adds its alternates and tiling items next to them, and retitles full screen.
fn native(
    id: &str,
    enabled: bool,
) -> Option<(objc2::runtime::Sel, &'static str, NSEventModifierFlags)> {
    let (command, none) = (NSEventModifierFlags::Command, NSEventModifierFlags::empty());
    match id {
        _ if id == TOGGLE_FULL_SCREEN.id => Some((
            sel!(toggleFullScreen:),
            "f",
            command | NSEventModifierFlags::Control,
        )),
        _ if id == MINIMIZE.id => Some((sel!(performMiniaturize:), "m", command)),
        _ if id == ZOOM.id => Some((sel!(performZoom:), "", none)),
        _ if id == BRING_ALL_TO_FRONT.id => Some((sel!(arrangeInFront:), "", none)),
        _ if id == HELP.id && !enabled => Some((sel!(showHelp:), "?", command)),
        _ => None,
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
    let settings = menu.menus.iter().flat_map(|menu| &menu.items).any(
        |item| matches!(item, MenuItem::Action { id, enabled: true, .. } if id == SETTINGS.id),
    );
    let menus: Vec<&Menu> = menu
        .menus
        .iter()
        .filter(|menu| !holds(menu, QUIT.id))
        .collect();
    let entries: Vec<Entry> = menus
        .iter()
        .map(|menu| Entry::Submenu {
            title: menu.title.to_string(),
            enabled: true,
            items: entries(&menu.items),
        })
        .collect();
    apply(installed, &installed.main, 1, &entries);
    let submenu = |index: usize| {
        installed
            .main
            .itemAtIndex(index as isize)
            .and_then(|item| item.submenu())
    };
    if let Some(app_menu) = submenu(0) {
        for index in [2, 3] {
            if let Some(item) = app_menu.itemAtIndex(index) {
                item.setHidden(!settings);
            }
        }
    }
    // AppKit adds its tiling items to the menu it knows as Window, and the search field to Help.
    let app = NSApplication::sharedApplication(installed.mtm);
    let menu_with = |id: &str| {
        menus
            .iter()
            .position(|menu| holds(menu, id))
            .and_then(|index| submenu(index + 1))
    };
    app.setWindowsMenu(menu_with(MINIMIZE.id).as_deref());
    app.setHelpMenu(menu_with(HELP.id).as_deref());
    installed.shown = Some(Shown { owner, revision });
}

pub(super) fn status_menu(
    mtm: MainThreadMarker,
    target: &Retained<Target>,
    items: &[MenuItem],
) -> Retained<NSMenu> {
    let main = NSMenu::new(mtm);
    main.setAutoenablesItems(false);
    let installed = Installed {
        mtm,
        main: main.clone(),
        target: target.clone(),
        handles: HashMap::new(),
        shown: None,
    };
    apply(&installed, &main, 0, &entries(items));
    main
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

impl MenuHandle {
    fn revision(&self) -> u32 {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .revision
    }
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

fn holds(menu: &Menu, id: &str) -> bool {
    menu.items
        .iter()
        .any(|item| matches!(item, MenuItem::Action { id: own, .. } if own == id))
}

fn shown_owner(owner: Option<u64>) -> Option<u64> {
    owner.or_else(|| {
        MENU.with(|slot| {
            slot.borrow()
                .as_ref()?
                .shown
                .as_ref()
                .map(|shown| shown.owner)
        })
    })
}

fn field_serves(id: &str, owner: Option<u64>) -> bool {
    field_edit_key(id).is_some()
        && shown_owner(owner).is_some_and(|owner| {
            with_runtime(|runtime| {
                runtime
                    .window(owner)
                    .is_some_and(|window| window.app.text_field_focused())
            })
            .unwrap_or(false)
        })
}

fn fire(id: &str, owner: Option<u64>) {
    let Some(owner) = shown_owner(owner) else {
        return;
    };
    let actions = with_runtime(|runtime| {
        let window = runtime.window(owner)?;
        (!window.app.edit_focused_field(id)).then(|| window.app.menu_actions.clone())
    })
    .flatten();
    if let Some(actions) = actions {
        actions.dispatch(id);
    }
    with_runtime(|runtime| {
        runtime.poll_all();
        runtime.install_timers();
        runtime.redraw_all();
    });
}
