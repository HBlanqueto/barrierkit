use super::*;
use freya_engine::prelude::{FontCollection, FontMgr};

/// One Freya app: its runner, tree and render contexts.
pub(super) struct AppInstance {
    runner: Runner,
    pub(super) tree: Tree,
    nodes_state: NodesState<NodeId>,
    events_sender: futures_channel::mpsc::UnboundedSender<EventsChunk>,
    events_receiver: futures_channel::mpsc::UnboundedReceiver<EventsChunk>,
    platform: Platform,
    _animation_clock: AnimationClock,
    pub(super) ticker_sender: RenderingTickerSender,
    pub(super) font_collection: FontCollection,
    pub(super) font_manager: FontMgr,
    fallback_fonts: Vec<Cow<'static, str>>,
    scale_factor: f64,
    size: Size2D,
    needs_layout: bool,
    waker: Waker,
    pub(super) menu: Option<crate::app_menu::AppMenu>,
    pub(super) on_menu_action: Option<MenuActionHandler>,
    pub(super) resolved: State<Resolved>,
    accessibility: AccessibilityTree,
    accessibility_initialized: bool,
    pub(super) requested_focus: Option<AccessibilityFocusStrategy>,
}

impl AppInstance {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        app: AppComponent,
        scale_factor: f64,
        size: (u32, u32),
        sender: Rc<dyn Fn(UserEvent)>,
        waker: Waker,
        toplevels: Option<Toplevels>,
        session_unlock: Option<SessionUnlock>,
        activation: Activation,
        window: Option<WindowControls>,
        timers: Timers,
        popups: Popups,
        popup_handle: Option<PopupHandle>,
        font_collection: FontCollection,
        font_manager: FontMgr,
        menu: Option<crate::app_menu::AppMenu>,
        on_menu_action: Option<MenuActionHandler>,
        resolved: State<Resolved>,
    ) -> Self {
        let (events_sender, events_receiver) = futures_channel::mpsc::unbounded();
        let mut runner = Runner::new(move || integration(app.clone()).into_element());

        runner.provide_root_context(ScreenReader::new);
        runner.provide_root_context(move || resolved);
        runner.provide_root_context(move || popups.clone());
        let (ticker_sender, ticker) = RenderingTicker::new();
        runner.provide_root_context(|| ticker);
        let animation_clock = runner.provide_root_context(AnimationClock::new);
        runner.provide_root_context(AssetCacher::create);

        let mut tree = Tree::default();

        let platform = runner.provide_root_context({
            let size = Size2D::new(size.0 as f32, size.1 as f32);
            move || Platform {
                focused_accessibility_id: State::create(ACCESSIBILITY_ROOT_ID),
                focused_accessibility_node: State::create(accesskit::Node::new(Role::Window)),
                root_size: State::create(size),
                scale_factor: State::create(scale_factor),
                navigation_mode: State::create(NavigationMode::NotKeyboard),
                preferred_theme: State::create(PreferredTheme::Light),
                is_app_focused: State::create(true),
                accent_color: State::create(AccentColor::default()),
                sender,
            }
        });

        if let Some(toplevels) = toplevels {
            runner.provide_root_context(move || toplevels);
        }
        if let Some(unlock) = session_unlock {
            runner.provide_root_context(move || unlock);
        }
        runner.provide_root_context(move || activation);
        if let Some(window) = window {
            runner.provide_root_context(move || window);
        }
        if let Some(handle) = popup_handle {
            runner.provide_root_context(move || handle);
        }
        runner.provide_root_context(move || timers);
        runner.provide_root_context(|| State::create(None::<Box<dyn ClipboardProvider>>));
        runner.provide_root_context(|| tree.accessibility_generator.clone());
        runner.provide_root_context(|| font_collection.clone());

        // Build the root node before the first layout pass: measuring an empty
        // tree panics inside Torin. Accessibility is initialized only after the
        // first layout, when every node already has a layout entry.
        let mutations = runner.sync_and_update();
        let _ = runner.run_in(|| tree.apply_mutations(mutations));

        let accessibility = AccessibilityTree::default();

        Self {
            runner,
            tree,
            nodes_state: NodesState::default(),
            events_sender,
            events_receiver,
            platform,
            _animation_clock: animation_clock,
            ticker_sender,
            font_collection,
            font_manager,
            fallback_fonts: default_fonts(),
            scale_factor,
            size: Size2D::new(size.0 as f32, size.1 as f32),
            needs_layout: true,
            waker,
            menu,
            on_menu_action,
            resolved,
            accessibility,
            accessibility_initialized: false,
            requested_focus: None,
        }
    }

    /// Mirrors the compositor's keyboard focus into the Freya platform state.
    pub(super) fn set_focused(&mut self, focused: bool) {
        self.platform.is_app_focused.set(focused);
        self.waker.wake_by_ref();
    }

    pub(super) fn resize(&mut self, size: (u32, u32), scale_factor: f64) {
        self.size = Size2D::new(size.0 as f32, size.1 as f32);
        self.scale_factor = scale_factor;
        self.platform.root_size.set(self.size);
        self.platform.scale_factor.set(scale_factor);
        self.tree.layout.reset();
        self.tree.text_cache.reset();
        self.needs_layout = true;
    }

    /// Drains Freya's events and UI events; returns true when the tree is dirty.
    /// Polls Freya once. `handle_events` returns ready while dirty scopes are
    /// pending; looping on it without calling `sync_and_update` spins forever,
    /// so the caller clears them in `update` and polls again on its next wake.
    pub(super) fn poll(&mut self) -> bool {
        let mut cx = Context::from_waker(&self.waker);
        let ready = {
            let future = pin!(self.runner.handle_events());
            future.poll(&mut cx).is_ready()
        };

        let mut dirty = ready;
        while let Ok(chunk) = self.events_receiver.try_recv() {
            match chunk {
                EventsChunk::Processed(events) => {
                    EventsExecutorAdapter {
                        runner: &mut self.runner,
                    }
                    .run(&mut self.nodes_state, events);
                    dirty = true;
                }
                EventsChunk::Batch(events) => {
                    for event in events {
                        self.runner.handle_event(
                            event.node_id,
                            event.name,
                            event.data,
                            event.bubbles,
                        );
                    }
                    dirty = true;
                }
            }
        }
        dirty
    }

    pub(super) fn update(&mut self) -> bool {
        let requested_focus = self.requested_focus.take();
        let mutations = self.runner.sync_and_update();
        let result = self.runner.run_in(|| self.tree.apply_mutations(mutations));
        if result.needs_render {
            self.needs_layout = true;
        }
        self.layout();

        // Initialize the accessibility tree once, after the first layout pass.
        // `init()` needs every node to have a layout entry, so it must run
        // strictly after `measure_layout`. Focus requests that arrived earlier
        // are applied right after init so they take effect in the same frame.
        if !self.accessibility_initialized {
            let _ = self.accessibility.init(&mut self.tree);
            self.accessibility_initialized = true;
        }
        if let Some(strategy) = requested_focus {
            self.tree.accessibility_diff.request_focus(strategy);
        }

        let accessibility_update = self
            .accessibility
            .process_updates(&mut self.tree, &self.events_sender);
        self.platform
            .focused_accessibility_id
            .set_if_modified(accessibility_update.focus);
        if let Some(node_id) = self.accessibility.focused_node_id()
            && let Some(layout_node) = self.tree.layout.get(&node_id)
        {
            self.platform.focused_accessibility_node.set_if_modified(
                AccessibilityTree::create_node(node_id, layout_node, &self.tree),
            );
        }
        result.needs_render
    }

    pub(super) fn layout(&mut self) {
        if !self.needs_layout {
            return;
        }
        self.tree.measure_layout(
            self.size,
            &mut self.font_collection,
            &self.font_manager,
            &self.events_sender,
            self.scale_factor,
            &self.fallback_fonts,
        );
        self.needs_layout = false;
    }

    pub(super) fn push(&mut self, event: PlatformEvent) {
        let mut adapter = EventsMeasurerAdapter {
            scale_factor: self.scale_factor,
            tree: &mut self.tree,
        };
        let processed = adapter.run(&mut vec![event], &mut self.nodes_state, None);
        let _ = self
            .events_sender
            .unbounded_send(EventsChunk::Processed(processed));
        // The event must be dispatched on the next loop wake; without this the
        // frame callback draws before Freya has processed the input.
        self.waker.wake_by_ref();
    }
}
