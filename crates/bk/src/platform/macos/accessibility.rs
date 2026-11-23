use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::task::Waker;

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};
use accesskit_macos::SubclassingAdapter;
use objc2_app_kit::NSView;

use crate::platform::{app::AppInstance, macos::with_runtime};

// Lets VoiceOver and the AX API read the kit's accessibility tree through the window's content view.
pub(crate) struct Bridge {
    adapter: SubclassingAdapter,
    requested: Rc<Cell<bool>>,
    engaged: Rc<Cell<bool>>,
    actions: Rc<RefCell<Vec<ActionRequest>>>,
}

impl Bridge {
    // The window's view does not exist yet; `attach_accessibility` creates the bridge for it.
    pub(crate) fn for_instance(_window: bool, _waker: &Waker) -> Option<Self> {
        None
    }

    // Must run before the view is shown or observed; accesskit_macos subclasses its class.
    pub(crate) fn new(view: &NSView, waker: Waker, window: u64) -> Self {
        let requested = Rc::new(Cell::new(false));
        let engaged = Rc::new(Cell::new(false));
        let actions = Rc::new(RefCell::new(Vec::new()));
        // SAFETY: `view` is a live NSView; the adapter retains it.
        let adapter = unsafe {
            SubclassingAdapter::new(
                std::ptr::from_ref(view).cast_mut().cast(),
                InitialTree {
                    requested: requested.clone(),
                    engaged: engaged.clone(),
                    window,
                    waker: waker.clone(),
                },
                Actions {
                    queue: actions.clone(),
                    waker,
                },
            )
        };
        Self {
            adapter,
            requested,
            engaged,
            actions,
        }
    }

    pub(crate) fn engaged(&self) -> bool {
        self.engaged.get()
    }

    pub(crate) fn pending(&self) -> bool {
        self.requested.get() || !self.actions.borrow().is_empty()
    }

    pub(crate) fn tree_requested(&self) -> bool {
        self.requested.replace(false)
    }

    pub(crate) fn take_actions(&self) -> Vec<ActionRequest> {
        std::mem::take(&mut *self.actions.borrow_mut())
    }

    pub(crate) fn update(&mut self, tree: TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(|| tree) {
            events.raise();
        }
    }

    pub(crate) fn set_focused(&mut self, focused: bool) {
        if let Some(events) = self.adapter.update_view_focus_state(focused) {
            events.raise();
        }
    }
}

// Falls back to a placeholder window, filled on the next update, when the runner is busy.
struct InitialTree {
    requested: Rc<Cell<bool>>,
    engaged: Rc<Cell<bool>>,
    window: u64,
    waker: Waker,
}

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.engaged.set(true);
        let window = self.window;
        let tree = with_runtime(|runtime| {
            runtime
                .window(window)
                .and_then(|found| found.app.initial_accessibility_tree())
        })
        .flatten();
        if tree.is_none() {
            self.requested.set(true);
            self.waker.wake_by_ref();
        }
        tree
    }
}

struct Actions {
    queue: Rc<RefCell<Vec<ActionRequest>>>,
    waker: Waker,
}

impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        self.queue.borrow_mut().push(request);
        self.waker.wake_by_ref();
    }
}

impl AppInstance {
    pub(crate) fn attach_accessibility(&mut self, view: &NSView, window: u64) {
        self.bridge = Some(Bridge::new(view, self.waker.clone(), window));
    }

    // The first query of a screen reader or of the AX API gets the whole tree at once.
    pub(crate) fn initial_accessibility_tree(&mut self) -> Option<TreeUpdate> {
        if !self.accessibility_initialized {
            return None;
        }
        let mut tree = self.accessibility.init(&mut self.tree, "");
        self.marked = crate::accessibility::resolve_marks(&mut tree);
        Some(tree)
    }
}
