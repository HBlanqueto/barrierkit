use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use freya_core::prelude::*;

use crate::file_dialog::FileDialog;
use crate::platform::{ObjectId, WindowControls, panels};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenedUrl {
    File(PathBuf),
    Other(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SaveChoice {
    Save,
    DontSave,
    Cancel,
}

type Writer = Rc<dyn Fn(&Path) -> io::Result<()>>;

#[derive(Clone, Copy)]
struct Registered {
    edited: State<bool>,
    closing: State<bool>,
}

thread_local! {
    static DOCUMENTS: RefCell<HashMap<ObjectId, Registered>> = RefCell::new(HashMap::new());
    static OPENED: Cell<Option<State<Vec<OpenedUrl>>>> = const { Cell::new(None) };
}

// The state is global so the platform can write it from outside the component tree, as it does for the window status.
fn opened() -> State<Vec<OpenedUrl>> {
    OPENED.with(|slot| {
        slot.get().unwrap_or_else(|| {
            let queue = State::create_global(Vec::new());
            slot.set(Some(queue));
            queue
        })
    })
}

/// Called by the platform when Finder, `open` or a link hands the app a file or an address.
pub(crate) fn deliver_open(url: OpenedUrl) {
    opened().write().push(url);
}

/// Called by the platform when the user closes a window; true when the document asks first and the window stays.
pub(crate) fn intercept_close(window: &ObjectId) -> bool {
    let Some(mut document) = DOCUMENTS.with(|all| all.borrow().get(window).copied()) else {
        return false;
    };
    if !*document.edited.peek() {
        return false;
    }
    document.closing.set(true);
    true
}

pub(crate) fn first_edited() -> Option<ObjectId> {
    DOCUMENTS.with(|all| {
        all.borrow()
            .iter()
            .find(|(_, document)| *document.edited.peek())
            .map(|(window, _)| window.clone())
    })
}

/// A window's document: the dirty flag, the file it lives in and how to write it.
#[derive(Clone)]
pub struct Document {
    edited: State<bool>,
    url: State<Option<PathBuf>>,
    window: WindowControls,
    writer: Rc<RefCell<Writer>>,
}

impl Document {
    pub fn is_edited(&self) -> bool {
        *self.edited.read()
    }

    /// An edited document shows the dot in the close button and asks "Save changes?" before its window closes.
    pub fn set_edited(&self, edited: bool) {
        let mut state = self.edited;
        state.set_if_modified(edited);
    }

    pub fn url(&self) -> Option<PathBuf> {
        self.url.read().clone()
    }

    pub fn set_url(&self, url: Option<PathBuf>) {
        let mut state = self.url;
        state.set_if_modified(url);
    }

    /// Writes the document to its file, asking for one first when it has none; true when it was written.
    pub async fn save(&self) -> bool {
        let known = self.url.peek().clone();
        let path = match known {
            Some(path) => path,
            None => {
                let name = self.window.title.to_string();
                match FileDialog::save(name)
                    .present(&self.window)
                    .await
                    .into_iter()
                    .next()
                {
                    Some(path) => path,
                    None => return false,
                }
            }
        };
        let writer = self.writer.borrow().clone();
        if let Err(error) = writer(&path) {
            tracing::warn!(%error, path = %path.display(), "BarrierKit: the document was not written");
            return false;
        }
        self.set_url(Some(path));
        self.set_edited(false);
        true
    }

    fn name(&self) -> String {
        self.url
            .peek()
            .as_deref()
            .and_then(Path::file_name)
            .map_or_else(
                || self.window.title.to_string(),
                |name| name.to_string_lossy().into_owned(),
            )
    }

    async fn confirm_close(&self) -> bool {
        match panels::confirm_save(&self.window, &self.name()).await {
            SaveChoice::Save => self.save().await,
            SaveChoice::DontSave => true,
            SaveChoice::Cancel => false,
        }
    }
}

/// `write` stores the document at a path; it runs for Save, so read the state it saves when it runs.
pub fn use_document(write: impl Fn(&Path) -> io::Result<()> + 'static) -> Document {
    let window = WindowControls::get();
    let edited = use_hook(|| State::create_global(false));
    let closing = use_hook(|| State::create_global(false));
    let url = use_hook(|| State::create_global(None::<PathBuf>));
    let writer: Writer = Rc::new(write);
    let slot = use_hook(|| Rc::new(RefCell::new(writer.clone())));
    *slot.borrow_mut() = writer;
    let document = Document {
        edited,
        url,
        window: window.clone(),
        writer: slot,
    };

    let key = window.id();
    use_hook({
        let key = key.clone();
        move || DOCUMENTS.with(|all| all.borrow_mut().insert(key, Registered { edited, closing }))
    });
    use_drop(move || {
        DOCUMENTS.with(|all| all.borrow_mut().remove(&key));
    });

    let marked = window.clone();
    use_side_effect(move || {
        panels::mark_document(&marked, *edited.read(), url.read().as_deref());
    });

    let closer = document.clone();
    use_side_effect(move || {
        if !*closing.read() {
            return;
        }
        let document = closer.clone();
        let mut closing = closing;
        spawn(async move {
            let close = document.confirm_close().await;
            closing.set(false);
            if close {
                document.set_edited(false);
                document.window.close();
            }
        });
    });
    document
}

pub(crate) fn use_open_url(on_open: Callback<OpenedUrl, ()>) {
    let latest = use_hook(|| Rc::new(RefCell::new(on_open.clone())));
    *latest.borrow_mut() = on_open;
    let mut queue = opened();
    use_side_effect(move || {
        if queue.read().is_empty() {
            return;
        }
        let items = std::mem::take(&mut *queue.write());
        let on_open = latest.borrow().clone();
        for item in items {
            on_open.call(item);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use freya_core::integration::AppComponent;
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;

    use super::*;
    use crate::modifiers::ViewModifiers;

    struct Case(Rc<RefCell<Vec<OpenedUrl>>>);

    impl App for Case {
        fn render(&self) -> impl IntoElement {
            let seen = self.0.clone();
            rect().on_open_url(move |url: OpenedUrl| seen.borrow_mut().push(url))
        }
    }

    #[test]
    fn an_address_that_arrives_before_the_window_renders_reaches_its_handler_once() {
        deliver_open(OpenedUrl::Other("notes://new".to_owned()));
        let seen = Rc::new(RefCell::new(Vec::new()));
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Case(seen.clone())),
            Size2D::new(100.0, 100.0),
            |_| {},
            1.0,
        );
        test.sync_and_update();
        test.sync_and_update();
        assert_eq!(*seen.borrow(), [OpenedUrl::Other("notes://new".to_owned())]);
    }

    #[test]
    fn an_edited_document_vetoes_the_close_and_asks_and_a_clean_one_does_not() {
        let window = ObjectId::null();
        let mut edited = State::create_global(false);
        let closing = State::create_global(false);
        DOCUMENTS.with(|all| {
            all.borrow_mut()
                .insert(window.clone(), Registered { edited, closing })
        });
        assert!(!intercept_close(&window));
        assert!(!*closing.peek());
        edited.set(true);
        assert!(intercept_close(&window));
        assert!(*closing.peek());
        DOCUMENTS.with(|all| all.borrow_mut().remove(&window));
        assert!(!intercept_close(&window));
    }
}
