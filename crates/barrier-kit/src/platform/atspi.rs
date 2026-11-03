use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Waker;

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};

pub(crate) struct Atspi {
    pub(crate) adapter: accesskit_unix::Adapter,
    requested: Arc<AtomicBool>,
}

impl Atspi {
    pub(crate) fn new(waker: Waker) -> Self {
        let requested = Arc::new(AtomicBool::new(false));
        let adapter = accesskit_unix::Adapter::new(
            InitialTree {
                requested: requested.clone(),
                waker,
            },
            NoActions,
            NoActions,
        );
        Self { adapter, requested }
    }

    pub(crate) fn tree_requested(&self) -> bool {
        self.requested.swap(false, Ordering::AcqRel)
    }
}

// accesskit_unix calls the handlers from its own thread; the tree is built on the UI thread.
struct InitialTree {
    requested: Arc<AtomicBool>,
    waker: Waker,
}

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.requested.store(true, Ordering::Release);
        self.waker.wake_by_ref();
        None
    }
}

struct NoActions;

impl ActionHandler for NoActions {
    fn do_action(&mut self, _: ActionRequest) {}
}

impl DeactivationHandler for NoActions {
    fn deactivate_accessibility(&mut self) {}
}
