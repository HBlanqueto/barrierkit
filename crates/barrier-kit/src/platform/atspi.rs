use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Waker;

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};

pub(crate) struct Atspi {
    pub(crate) adapter: accesskit_unix::Adapter,
    requested: Arc<AtomicBool>,
    actions: Arc<Mutex<Vec<ActionRequest>>>,
}

impl Atspi {
    pub(crate) fn new(waker: Waker) -> Self {
        let requested = Arc::new(AtomicBool::new(false));
        let actions = Arc::new(Mutex::new(Vec::new()));
        let adapter = accesskit_unix::Adapter::new(
            InitialTree {
                requested: requested.clone(),
                waker: waker.clone(),
            },
            Actions {
                queue: actions.clone(),
                waker,
            },
            Inactive,
        );
        Self {
            adapter,
            requested,
            actions,
        }
    }

    pub(crate) fn tree_requested(&self) -> bool {
        self.requested.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn take_actions(&self) -> Vec<ActionRequest> {
        self.actions
            .lock()
            .map(|mut queue| std::mem::take(&mut *queue))
            .unwrap_or_default()
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

struct Actions {
    queue: Arc<Mutex<Vec<ActionRequest>>>,
    waker: Waker,
}

impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(request);
        }
        self.waker.wake_by_ref();
    }
}

struct Inactive;

impl DeactivationHandler for Inactive {
    fn deactivate_accessibility(&mut self) {}
}
