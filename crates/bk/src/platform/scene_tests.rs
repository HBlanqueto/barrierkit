use ctappearance::{AppearanceConfig, resolve};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use std::{cell::RefCell, rc::Rc};
use torin::prelude::Size2D;

use super::window::{WindowCommand, WindowControls, WindowOptions, WindowStatus};
use crate::app::{
    Application, Commands, DismissWindowAction, MenuActions, OpenWindowAction, Window, WindowGroup,
    ZOOM, use_dismiss_window, use_open_window,
};
use crate::i18n::text;
use crate::main_menu::MenuHandle;
use crate::menu::{CheckState, MenuItem};

type Queue = Rc<RefCell<Vec<WindowCommand>>>;

struct Mounted {
    test: TestingRunner,
    actions: MenuActions,
    queue: Queue,
    menu: Option<MenuHandle>,
}

fn mount(app: AppComponent, options: &WindowOptions, fixed: bool) -> Mounted {
    let queue: Queue = Rc::default();
    let menu = options.menu.clone().map(|menu| MenuHandle::new(menu, None));
    let resolved = resolve(&AppearanceConfig::default(), None);
    let (mut test, actions) = TestingRunner::new(
        app,
        Size2D::new(400.0, 300.0),
        {
            let (queue, menu) = (queue.clone(), menu.clone());
            move |runner| {
                runner.provide_root_context(move || WindowControls {
                    queue: queue.clone(),
                    fixed,
                    ..super::tests::window_controls(WindowStatus::default())
                });
                runner.provide_root_context(move || State::create(resolved));
                if let Some(menu) = menu {
                    runner.provide_root_context(move || menu.clone());
                }
                runner.provide_root_context(MenuActions::default)
            }
        },
        1.0,
    );
    test.sync_and_update();
    Mounted {
        test,
        actions,
        queue,
        menu,
    }
}

fn launch(application: Application) -> Mounted {
    let (options, root) = application.launch().expect("the application has a window");
    mount(root, &options, false)
}

fn take(queue: &Queue) -> Vec<WindowCommand> {
    std::mem::take(&mut *queue.borrow_mut())
}

fn opened(queue: &Queue) -> Vec<(AppComponent, WindowOptions)> {
    take(queue)
        .into_iter()
        .filter_map(|command| match command {
            WindowCommand::Open(window) => Some(*window),
            _ => None,
        })
        .collect()
}

fn notes() -> Application {
    Application::new("notes")
        .window("Notes", rect)
        .scene(Window::new("inspector", "Inspector", rect))
        .commands(Commands::new())
}

fn items_of(menu: &MenuHandle, title: &str) -> Vec<MenuItem> {
    menu.snapshot()
        .0
        .menus
        .into_iter()
        .find(|menu| menu.title == title)
        .map(|menu| menu.items)
        .unwrap_or_default()
}

fn label_of(item: &MenuItem) -> &str {
    item.label().unwrap_or("")
}

#[test]
fn a_menu_action_opens_the_second_scene_with_its_title_and_the_app_menu() {
    let first = launch(notes());
    first.actions.dispatch("window-open:inspector");
    let mut windows = opened(&first.queue);
    assert_eq!(windows.len(), 1);
    let (_, options) = windows.remove(0);
    assert_eq!(options.title, "Inspector");
    assert_eq!(options.app_name, "notes");
    assert_eq!(options.scene, Some("inspector"));
    assert!(options.menu.is_some());
}

#[test]
fn the_launch_window_keeps_the_restoration_key_of_an_app_with_one_window() {
    let (options, _) = notes().launch().unwrap();
    assert_eq!(options.scene, None);
}

#[test]
fn opening_a_window_scene_again_brings_the_open_one_to_the_front() {
    let first = launch(notes());
    first.actions.dispatch("window-open:inspector");
    let (root, options) = opened(&first.queue).remove(0);
    let second = mount(root, &options, false);

    first.actions.dispatch("window-open:inspector");

    assert!(opened(&first.queue).is_empty());
    assert!(
        take(&second.queue)
            .iter()
            .any(|command| matches!(command, WindowCommand::MakeKey))
    );
}

#[test]
fn a_window_group_opens_a_new_window_each_time() {
    let first = launch(notes());
    first.actions.dispatch("new-window");
    let (root, options) = opened(&first.queue).remove(0);
    let _second = mount(root, &options, false);
    first.actions.dispatch("new-window");
    assert_eq!(opened(&first.queue).len(), 1);
}

#[test]
fn dismiss_window_closes_the_windows_of_that_id_and_nothing_else() {
    let dismiss: Rc<RefCell<Option<DismissWindowAction>>> = Rc::default();
    let handle = dismiss.clone();
    let first = launch(
        Application::new("notes")
            .window("Notes", move || {
                *handle.borrow_mut() = Some(use_dismiss_window());
                rect()
            })
            .scene(Window::new("inspector", "Inspector", rect))
            .commands(Commands::new()),
    );
    first.actions.dispatch("window-open:inspector");
    let (root, options) = opened(&first.queue).remove(0);
    let second = mount(root, &options, false);

    dismiss.borrow().as_ref().unwrap().dismiss_id("inspector");

    assert!(
        take(&second.queue)
            .iter()
            .any(|command| matches!(command, WindowCommand::Close))
    );
    assert!(take(&first.queue).is_empty());
}

#[test]
fn dismiss_closes_the_window_it_is_called_from() {
    let dismiss: Rc<RefCell<Option<DismissWindowAction>>> = Rc::default();
    let handle = dismiss.clone();
    let first = launch(Application::new("notes").window("Notes", move || {
        *handle.borrow_mut() = Some(use_dismiss_window());
        rect()
    }));
    dismiss.borrow().as_ref().unwrap().dismiss();
    assert!(
        take(&first.queue)
            .iter()
            .any(|command| matches!(command, WindowCommand::Close))
    );
}

#[test]
fn a_value_window_opens_once_per_value_and_receives_it() {
    let open: Rc<RefCell<Option<OpenWindowAction>>> = Rc::default();
    let shown: Rc<RefCell<Vec<u32>>> = Rc::default();
    let (handle, seen) = (open.clone(), shown.clone());
    let first = launch(
        Application::new("notes")
            .window("Notes", move || {
                *handle.borrow_mut() = Some(use_open_window());
                rect()
            })
            .scene(WindowGroup::for_value("note", "Note", move |id: u32| {
                seen.borrow_mut().push(id);
                rect()
            })),
    );
    let open = open.borrow().clone().unwrap();

    open.open_value(7_u32);
    let (root, options) = opened(&first.queue).remove(0);
    let second = mount(root, &options, false);
    assert_eq!(*shown.borrow(), vec![7]);

    open.open_value(7_u32);
    assert!(opened(&first.queue).is_empty());
    assert!(
        take(&second.queue)
            .iter()
            .any(|command| matches!(command, WindowCommand::MakeKey))
    );

    open.open_value(8_u32);
    assert_eq!(opened(&first.queue).len(), 1);
}

#[test]
fn close_window_closes_its_own_window_and_quit_closes_every_one() {
    let first = launch(notes());
    first.actions.dispatch("new-window");
    let (root, options) = opened(&first.queue).remove(0);
    let second = mount(root, &options, false);

    second.actions.dispatch("close-window");
    let closed = |queue: &Queue| {
        take(queue)
            .iter()
            .filter(|command| matches!(command, WindowCommand::Close))
            .count()
    };
    assert_eq!(closed(&second.queue), 1);
    assert_eq!(closed(&first.queue), 0);

    first.actions.dispatch("quit");
    for queue in [&first.queue, &second.queue] {
        assert!(
            take(queue)
                .iter()
                .any(|command| matches!(command, WindowCommand::Close))
        );
    }
}

#[test]
fn minimize_and_zoom_act_on_the_window_that_has_the_menu() {
    let first = launch(notes());
    first.actions.dispatch("minimize");
    first.actions.dispatch("zoom");
    let commands = take(&first.queue);
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, WindowCommand::Minimize))
    );
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, WindowCommand::ToggleMaximize))
    );
}

#[test]
fn the_window_menu_lists_the_open_windows_and_checks_the_one_that_owns_the_menu() {
    let mut first = launch(notes());
    let menu = first.menu.clone().unwrap();
    first.actions.dispatch("new-window");
    let (root, options) = opened(&first.queue).remove(0);
    let _second = mount(root, &options, false);
    first.test.sync_and_update();

    let windows = items_of(&menu, text!("menu-window"));
    let listed: Vec<(&str, bool)> = windows
        .iter()
        .filter_map(|item| match item {
            MenuItem::Action {
                label, check, id, ..
            } if id.starts_with("window-activate:") => {
                Some((label.as_ref(), *check == CheckState::Checkbox(true)))
            }
            _ => None,
        })
        .collect();
    assert_eq!(listed, vec![("Notes", true), ("Notes", false)]);
    assert!(
        windows.iter().any(|item| label_of(item) == "Inspector"),
        "a Window scene has its own item"
    );
}

#[test]
fn a_fixed_size_window_cannot_zoom() {
    let (options, root) = notes().launch().unwrap();
    let fixed = mount(root, &options, true);
    let menu = fixed.menu.unwrap();
    let zoom = items_of(&menu, text!("menu-window"))
        .into_iter()
        .find_map(|item| match item {
            MenuItem::Action { id, enabled, .. } if id == ZOOM.id => Some(enabled),
            _ => None,
        });
    assert_eq!(zoom, Some(false));
}
