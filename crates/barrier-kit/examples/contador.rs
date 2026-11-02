//! Counter demo app showing the global menu export.

use std::cell::RefCell;
use std::rc::Rc;

use barrier_kit::app::{
    Action, Instance, Launch, Shortcut, ShortcutKey, UndoStack, copy, launch, load_defaults,
    save_defaults,
};
use barrier_kit::app_menu::{AppMenu, Menu, MenuHandle};
use barrier_kit::controls::Button;
use barrier_kit::menu::{CheckState, MenuItem};
use barrier_kit::platform::{Decorations, WindowControls, WindowOptions, run_window};
use barrier_kit::titlebar::Titlebar;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use keyboard_types::Modifiers;
use serde::{Deserialize, Serialize};
use torin::prelude::{Alignment, Direction, Size};

const ID_NEW_WINDOW: &str = "new-window";
const ID_CLOSE_WINDOW: &str = "close-window";
const ID_COPY_VALUE: &str = "copy-value";
const ID_UNDO: &str = "undo";
const ID_REDO: &str = "redo";
const ID_INCREMENT: &str = "increment";
const ID_DECREMENT: &str = "decrement";
const ID_RESET: &str = "reset";
const ID_STEP_1: &str = "step-1";
const ID_STEP_5: &str = "step-5";
const ID_STEP_10: &str = "step-10";
const ID_ALLOW_NEGATIVE: &str = "allow-negative";
const ID_ABOUT: &str = "about";

const NEW_WINDOW: Action =
    Action::new(ID_NEW_WINDOW, "Nueva ventana").shortcut(Shortcut::ctrl(ShortcutKey::Char('n')));
const CLOSE_WINDOW: Action =
    Action::new(ID_CLOSE_WINDOW, "Cerrar ventana").shortcut(Shortcut::ctrl(ShortcutKey::Char('w')));
const COPY_VALUE: Action =
    Action::new(ID_COPY_VALUE, "Copiar valor").shortcut(Shortcut::ctrl(ShortcutKey::Char('c')));
const UNDO: Action =
    Action::new(ID_UNDO, "Deshacer").shortcut(Shortcut::ctrl(ShortcutKey::Char('z')));
const REDO: Action = Action::new(ID_REDO, "Rehacer").shortcut(Shortcut::new(
    Modifiers::CONTROL.union(Modifiers::SHIFT),
    ShortcutKey::Char('z'),
));
const INCREMENT: Action =
    Action::new(ID_INCREMENT, "Incrementar").shortcut(Shortcut::ctrl(ShortcutKey::Up));
const DECREMENT: Action =
    Action::new(ID_DECREMENT, "Decrementar").shortcut(Shortcut::ctrl(ShortcutKey::Down));
const RESET: Action =
    Action::new(ID_RESET, "Reiniciar").shortcut(Shortcut::ctrl(ShortcutKey::Char('r')));
const STEP_1: Action = Action::new(ID_STEP_1, "1");
const STEP_5: Action = Action::new(ID_STEP_5, "5").shortcut(Shortcut::ctrl(ShortcutKey::Char('5')));
const STEP_10: Action = Action::new(ID_STEP_10, "10");
const ALLOW_NEGATIVE: Action = Action::new(ID_ALLOW_NEGATIVE, "Permitir negativos");
const ABOUT: Action = Action::new(ID_ABOUT, "Acerca de Contador");
const APP_ID: &str = "org.malaysia.Contador";

#[derive(Default, Serialize, Deserialize)]
struct Prefs {
    step: i32,
    allow_negative: bool,
}

fn separator() -> MenuItem {
    MenuItem::Separator
}

fn build_menu(count: i32, step: i32, allow_negative: bool) -> AppMenu {
    AppMenu::new(vec![
        Menu::new(
            "Archivo",
            vec![NEW_WINDOW.item(false), separator(), CLOSE_WINDOW.item(true)],
        ),
        Menu::new(
            "Edición",
            vec![
                UNDO.item(false),
                REDO.item(false),
                separator(),
                COPY_VALUE.item(true),
            ],
        ),
        Menu::new(
            "Contador",
            vec![
                INCREMENT.item(true),
                DECREMENT.item(true),
                separator(),
                RESET.item(count != 0),
                separator(),
                MenuItem::Submenu {
                    id: Some("step-submenu".to_string()),
                    label: "Paso".to_string(),
                    enabled: true,
                    icon: None,
                    items: vec![
                        STEP_1.item_with(true, CheckState::Radio(step == 1)),
                        STEP_5.item_with(true, CheckState::Radio(step == 5)),
                        STEP_10.item_with(true, CheckState::Radio(step == 10)),
                    ],
                },
                separator(),
                ALLOW_NEGATIVE.item_with(true, CheckState::Checkbox(allow_negative)),
            ],
        ),
        Menu::new("Ayuda", vec![ABOUT.item(true)]),
    ])
}

fn sync_menu(
    handle: &MenuHandle,
    count: i32,
    step: i32,
    allow_negative: bool,
    history: &UndoStack<i32>,
) {
    for (id, enabled_now) in [(ID_UNDO, history.can_undo()), (ID_REDO, history.can_redo())] {
        handle.update(id, |item| {
            if let MenuItem::Action { enabled, .. } = item {
                *enabled = enabled_now;
            }
        });
    }
    handle.update(ID_RESET, |item| {
        if let MenuItem::Action { enabled, .. } = item {
            *enabled = count != 0;
        }
    });
    handle.update(ID_STEP_1, |item| {
        if let MenuItem::Action { check, .. } = item {
            *check = CheckState::Radio(step == 1);
        }
    });
    handle.update(ID_STEP_5, |item| {
        if let MenuItem::Action { check, .. } = item {
            *check = CheckState::Radio(step == 5);
        }
    });
    handle.update(ID_STEP_10, |item| {
        if let MenuItem::Action { check, .. } = item {
            *check = CheckState::Radio(step == 10);
        }
    });
    handle.update(ID_ALLOW_NEGATIVE, |item| {
        if let MenuItem::Action { check, .. } = item {
            *check = CheckState::Checkbox(allow_negative);
        }
    });
}

struct Contador {
    instance: Option<Instance>,
    action_rx: Rc<RefCell<Option<std::sync::mpsc::Receiver<String>>>>,
}

impl App for Contador {
    fn render(&self) -> impl IntoElement {
        let mut count = use_state(|| 0i32);
        let prefs = use_hook(|| {
            let prefs: Prefs = load_defaults(APP_ID);
            (
                if prefs.step > 0 { prefs.step } else { 1 },
                prefs.allow_negative,
            )
        });
        let mut step = use_state(|| prefs.0);
        let mut allow_negative = use_state(|| prefs.1);
        let mut menu_handle: State<Option<MenuHandle>> = use_state(|| None);

        {
            let instance = self.instance.clone();
            use_hook(move || {
                menu_handle.set(WindowControls::get().menu_handle());
                if let Some(instance) = &instance {
                    instance.listen();
                }
            });
        }

        let mut pending = use_state(Vec::<String>::new);
        let mut history = use_state(UndoStack::<i32>::default);
        let rx = use_hook(|| {
            Rc::new(RefCell::new(
                self.action_rx
                    .borrow_mut()
                    .take()
                    .expect("menu action receiver already taken"),
            ))
        });
        while let Ok(id) = rx.borrow_mut().try_recv() {
            pending.write().push(id);
        }
        let ids: Vec<String> = {
            let guard = pending.read();
            guard.clone()
        };
        pending.write().clear();
        for id in ids {
            let previous = *count.read();
            match id.as_str() {
                ID_UNDO => {
                    let undone = history.write().undo(previous);
                    if let Some(value) = undone {
                        count.set(value);
                    }
                }
                ID_REDO => {
                    let redone = history.write().redo(previous);
                    if let Some(value) = redone {
                        count.set(value);
                    }
                }
                ID_COPY_VALUE => copy(previous.to_string()),
                ID_INCREMENT => {
                    let s = *step.read();
                    let current = *count.read();
                    count.set(current.saturating_add(s));
                }
                ID_DECREMENT => {
                    let s = *step.read();
                    let current = *count.read();
                    let next = current.saturating_sub(s);
                    if next < 0 && !*allow_negative.read() {
                        count.set(0);
                    } else {
                        count.set(next);
                    }
                }
                ID_RESET => count.set(0),
                ID_STEP_1 => step.set(1),
                ID_STEP_5 => step.set(5),
                ID_STEP_10 => step.set(10),
                ID_ALLOW_NEGATIVE => {
                    let current = *allow_negative.read();
                    allow_negative.set(!current);
                }
                ID_CLOSE_WINDOW => std::process::exit(0),
                _ => {}
            }
            if ![ID_UNDO, ID_REDO].contains(&id.as_str()) && *count.read() != previous {
                history.write().record(previous);
            }
            if [ID_STEP_1, ID_STEP_5, ID_STEP_10, ID_ALLOW_NEGATIVE].contains(&id.as_str()) {
                let prefs = Prefs {
                    step: *step.read(),
                    allow_negative: *allow_negative.read(),
                };
                if let Err(error) = save_defaults(APP_ID, &prefs) {
                    tracing::warn!(%error, "contador: no se guardaron las preferencias");
                }
            }
            tracing::info!(action = id, count = *count.read(), "contador: action");
        }

        if let Some(handle) = menu_handle.read().as_ref() {
            sync_menu(
                handle,
                *count.read(),
                *step.read(),
                *allow_negative.read(),
                &history.read(),
            );
        }

        let palette = barrier_kit::theme::resolved();
        let colors = vow_appearance::palette(palette.scheme);
        let [tr, tg, tb, ta] = colors.text.focused.to_rgba8();
        let text = Color::from_argb(ta, tr, tg, tb);

        let display = format!("{}", *count.read());

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(Titlebar::new().on_close(|()| std::process::exit(0)))
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .center()
                    .direction(Direction::Vertical)
                    .spacing(24.0)
                    .main_align(Alignment::Center)
                    .child(label().text(display).font_size(96.0).color(text))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(16.0)
                            .child(
                                Button::new("−").on_press(move |_| {
                                    pending.write().push(ID_DECREMENT.to_string())
                                }),
                            )
                            .child(
                                Button::new("+").on_press(move |_| {
                                    pending.write().push(ID_INCREMENT.to_string())
                                }),
                            ),
                    ),
            )
            .into_element()
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let instance = match launch(APP_ID) {
        Ok(Launch::Forwarded) => return,
        Ok(Launch::Primary(instance)) => Some(instance),
        Err(error) => {
            tracing::warn!(%error, "contador: sin instancia única");
            None
        }
    };
    let (action_tx, action_rx) = std::sync::mpsc::channel::<String>();
    let app = AppComponent::new(Contador {
        instance,
        action_rx: Rc::new(RefCell::new(Some(action_rx))),
    });
    let prefs: Prefs = load_defaults(APP_ID);
    let menu = build_menu(0, prefs.step.max(1), prefs.allow_negative);
    run_window(
        app,
        WindowOptions::new("Contador", "Contador")
            .app_id(APP_ID)
            .decorations(Decorations::Client)
            .size((480, 320))
            .menu(menu)
            .on_menu_action(move |id| {
                tracing::info!(action = id, "contador: menu action");
                eprintln!(
                    "contador: action={id} thread={:?}",
                    std::thread::current().id()
                );
                let _ = action_tx.send(id.to_string());
            }),
    )
    .expect("contador");
}
