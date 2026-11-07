use freya_core::integration::{SingleThreadErasedEvent, UserEvent};
use freya_core::prelude::*;

use crate::i18n::text;

use crate::menu::MenuItem;
use crate::platform::{EditShortcut, PopupAnchor};
use crate::popup_button::open_popover_menu;
use crate::rich_text::{StyleRun, restyle_after_edit};
use crate::spelling::{ignore, learn, replace_chars, suggestion};

pub fn context_menu(
    element: Rect,
    items: Vec<MenuItem>,
    on_activate: impl Into<Callback<String, ()>>,
) -> Rect {
    context_menu_with(element, move || items.clone(), on_activate)
}

fn context_menu_with(
    element: Rect,
    items: impl Fn() -> Vec<MenuItem> + 'static,
    on_activate: impl Into<Callback<String, ()>>,
) -> Rect {
    let on_activate = on_activate.into();
    element.on_secondary_down(move |event: Event<PressEventData>| {
        let PressEventData::Mouse(mouse) = event.data() else {
            return;
        };
        let on_activate = on_activate.clone();
        open_popover_menu(
            items(),
            PopupAnchor::TopLeft,
            (
                mouse.global_location.x.round() as i32,
                mouse.global_location.y.round() as i32,
                1,
                1,
            ),
            None,
            move |id| on_activate.call(id),
        );
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Spelling {
    pub value: State<String>,
    pub index: State<Option<usize>>,
    pub styles: Option<State<Vec<StyleRun>>>,
}

const GUESS: &str = "guess:";
const IGNORE: &str = "spelling:ignore";
const LEARN: &str = "spelling:learn";

pub(crate) fn edit_menu(
    element: Rect,
    field: AccessibilityId,
    has_selection: bool,
    (can_undo, can_redo): (bool, bool),
    spelling: Option<Spelling>,
) -> Rect {
    let platform = Platform::get();
    let suggest = move || {
        let spelling = spelling?;
        let index = (*spelling.index.peek())?;
        suggestion(&spelling.value.peek(), index)
    };
    let items = move || {
        let mut items: Vec<MenuItem> = Vec::new();
        if let Some(found) = suggest() {
            if found.guesses.is_empty() {
                items.push(MenuItem::action("", text!("no-guesses")).with_enabled(false));
            }
            items.extend(
                found
                    .guesses
                    .into_iter()
                    .enumerate()
                    .map(|(index, guess)| MenuItem::action(format!("{GUESS}{index}"), guess)),
            );
            items.extend([
                MenuItem::Separator,
                MenuItem::action(IGNORE, text!("ignore-spelling")),
                MenuItem::action(LEARN, text!("learn-spelling")),
                MenuItem::Separator,
            ]);
        }
        items.extend([
            MenuItem::action("z", text!("undo")).with_enabled(can_undo),
            MenuItem::action("y", text!("redo")).with_enabled(can_redo),
            MenuItem::Separator,
            MenuItem::action("x", text!("cut")).with_enabled(has_selection),
            MenuItem::action("c", text!("copy")).with_enabled(has_selection),
            MenuItem::action("v", text!("paste")),
            MenuItem::Separator,
            MenuItem::action("a", text!("select-all")),
        ]);
        items
    };
    context_menu_with(element, items, move |key: String| {
        if key == IGNORE || key == LEARN {
            if let Some(found) = suggest() {
                if key == LEARN {
                    learn(found.word);
                } else {
                    ignore(found.word);
                }
            }
            return;
        }
        if let (Some(spelling), Some(index)) = (spelling, key.strip_prefix(GUESS)) {
            if let (Some(found), Ok(index)) = (suggest(), index.parse::<usize>())
                && let Some(guess) = found.guesses.get(index)
            {
                replace_word(spelling, found.range, guess);
            }
            return;
        }
        platform.send(UserEvent::Erased(SingleThreadErasedEvent(Box::new(
            EditShortcut { field, key },
        ))));
    })
}

fn replace_word(mut spelling: Spelling, range: std::ops::Range<usize>, guess: &str) {
    let old = spelling.value.peek().clone();
    let new = replace_chars(&old, range, guess);
    if let Some(mut styles) = spelling.styles {
        let next = restyle_after_edit(&styles.peek(), &old, &new);
        styles.set(next);
    }
    spelling.value.set(new);
}
