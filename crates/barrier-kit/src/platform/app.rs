use accesskit::Role;
use freya_clipboard::copypasta::ClipboardProvider;

use crate::app::DefaultsRevision;
use freya_components::{cache::AssetCacher, integration::integration};
use freya_core::prelude::{Code, Key, Modifiers, NamedKey};
use freya_core::{
    accessibility::tree::{ACCESSIBILITY_ROOT_ID, AccessibilityTree},
    element::IntoElement,
    integration::{
        AccentColor, AnimationClock, AppComponent, EventsChunk, EventsExecutorAdapter,
        EventsMeasurerAdapter, KeyboardEventName, MouseEventName, NavigationMode, NodeId, Platform,
        PlatformEvent, PreferredTheme, RenderingTicker, RenderingTickerSender, Runner,
        ScreenReader, Tree, UserEvent, WritableUtils, default_fonts,
    },
    prelude::{AccessibilityFocusStrategy, State},
};
use freya_engine::prelude::{FontCollection, FontMgr};
use ragnarok::{EventsExecutorRunner, EventsMeasurerRunner, NodesState};
use sctk::reexports::client::backend::ObjectId;
use std::{
    borrow::Cow,
    cell::Cell,
    pin::pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Waker},
};
use torin::prelude::Size2D;
use vow_appearance::Resolved;

use super::{
    popup::{PopupHandle, PopupId, Popups},
    runtime::Runtime,
    timer::{PingWake, Timers},
    window::WindowControls,
};

const MAX_SETTLE_PASSES: usize = 4;

pub(crate) struct AppInstance {
    runner: Runner,
    pub(crate) tree: Tree,
    nodes_state: NodesState<NodeId>,
    events_sender: futures_channel::mpsc::UnboundedSender<EventsChunk>,
    events_receiver: futures_channel::mpsc::UnboundedReceiver<EventsChunk>,
    platform: Platform,
    _animation_clock: AnimationClock,
    pub(crate) ticker_sender: RenderingTickerSender,
    pub(crate) font_collection: FontCollection,
    pub(crate) font_manager: FontMgr,
    fallback_fonts: Vec<Cow<'static, str>>,
    scale_factor: f64,
    size: Size2D,
    needs_layout: bool,
    waker: Waker,
    pub(crate) menu_actions: crate::app::MenuActions,
    pub(crate) resolved: State<Resolved>,
    accessibility: AccessibilityTree,
    accessibility_initialized: bool,
    atspi: Option<super::atspi::Atspi>,
    pub(crate) requested_focus: Option<AccessibilityFocusStrategy>,
    pub(crate) pending_shortcuts: Vec<Key>,
}

impl AppInstance {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        rt: &Runtime,
        owner: ObjectId,
        app: AppComponent,
        scale_factor: f64,
        size: (u32, u32),
        window: Option<WindowControls>,
        popup: Option<(PopupId, Rc<Cell<bool>>)>,
        menu_handle: Option<crate::app_menu::MenuHandle>,
    ) -> Self {
        let waker = futures_util::task::waker(Arc::new(PingWake(rt.ping.clone())));
        let sender: Rc<dyn Fn(UserEvent)> = {
            let (queue, ping, owner) = (rt.queue.clone(), rt.ping.clone(), owner.clone());
            Rc::new(move |event| {
                queue.borrow_mut().push((owner.clone(), event));
                ping.ping();
            })
        };
        let popups = Popups {
            queue: rt.popup_commands.clone(),
            ping: rt.ping.clone(),
            next_id: rt.popup_next_id.clone(),
            last_serial: rt.last_input_serial.clone(),
            owner,
            current: popup.as_ref().map(|(id, _)| *id),
            opens_left: popup
                .as_ref()
                .map_or_else(Rc::default, |(_, left)| left.clone()),
            scroll_shift: Rc::default(),
        };
        let popup_handle = popup.map(|(id, _)| PopupHandle {
            id,
            popups: popups.clone(),
        });
        let timers = Timers {
            queue: rt.timers.clone(),
            ping: rt.ping.clone(),
            wake: rt.wake.clone(),
        };
        let font_collection = rt.graphics.font_collection.clone();
        let font_manager = rt.graphics.font_manager.clone();
        let resolved = rt.resolved;
        let clipboard: Box<dyn ClipboardProvider> = Box::new(rt.clipboard.clone());
        let defaults_revision = rt.defaults_revision;
        let (events_sender, events_receiver) = futures_channel::mpsc::unbounded();
        let mut runner = Runner::new(move || {
            crate::chrome::window_root(integration(app.clone())).into_element()
        });

        runner.provide_root_context(ScreenReader::new);
        runner.provide_root_context(move || resolved);
        runner.provide_root_context(move || popups.clone());
        let menu_actions = runner.provide_root_context(crate::app::MenuActions::default);
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

        if let Some(handle) = menu_handle {
            runner.provide_root_context(move || handle);
        }
        if let Some(hooks) = rt.extension_hooks() {
            (hooks.provide_contexts)(rt, &mut runner);
        }
        let window_shown = window.is_some();
        if let Some(window) = window {
            runner.provide_root_context(move || window);
        }
        if let Some(handle) = popup_handle {
            runner.provide_root_context(move || handle);
        }
        runner.provide_root_context(move || timers);
        runner.provide_root_context(move || State::create(Some(clipboard)));
        runner.provide_root_context(move || DefaultsRevision(defaults_revision));
        runner.provide_root_context(|| tree.accessibility_generator.clone());
        runner.provide_root_context(|| font_collection.clone());

        let mutations = runner.sync_and_update();
        let _ = runner.run_in(|| tree.apply_mutations(mutations));

        let accessibility = AccessibilityTree::default();
        let atspi = window_shown.then(|| super::atspi::Atspi::new(waker.clone()));

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
            menu_actions,
            resolved,
            accessibility,
            accessibility_initialized: false,
            atspi,
            requested_focus: None,
            pending_shortcuts: Vec::new(),
        }
    }

    pub(crate) fn text_field_at(&self, (x, y): (f64, f64)) -> bool {
        let (x, y) = (x as f32, y as f32);
        self.tree.elements.iter().any(|(node, element)| {
            matches!(
                element.accessibility().builder.role(),
                Role::TextInput
                    | Role::MultilineTextInput
                    | Role::SearchInput
                    | Role::PasswordInput
            ) && self.tree.layout.get(node).is_some_and(|layout| {
                let area = layout.visible_area();
                (area.min_x()..area.max_x()).contains(&x)
                    && (area.min_y()..area.max_y()).contains(&y)
            })
        })
    }

    pub(crate) fn focused_role(&self) -> Role {
        self.platform.focused_accessibility_node.peek().role()
    }

    pub(crate) fn text_field_focused(&self) -> bool {
        matches!(
            self.focused_role(),
            Role::TextInput | Role::MultilineTextInput | Role::SearchInput | Role::PasswordInput
        )
    }

    pub(crate) fn set_focused(&mut self, focused: bool) {
        self.platform.is_app_focused.set(focused);
        if let Some(atspi) = &mut self.atspi {
            atspi.adapter.update_window_focus_state(focused);
        }
        self.waker.wake_by_ref();
    }

    pub(crate) fn resize(&mut self, size: (u32, u32), scale_factor: f64) {
        self.size = Size2D::new(size.0 as f32, size.1 as f32);
        self.scale_factor = scale_factor;
        self.platform.root_size.set(self.size);
        self.platform.scale_factor.set(scale_factor);
        self.tree.layout.reset();
        self.tree.text_cache.reset();
        self.needs_layout = true;
    }

    pub(crate) fn poll(&mut self) -> bool {
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

    pub(crate) fn accessibility_pending(&self) -> bool {
        self.requested_focus.is_some()
            || self
                .atspi
                .as_ref()
                .is_some_and(super::atspi::Atspi::pending)
    }

    pub(crate) fn update(&mut self) -> bool {
        let pressed = self.take_atspi_actions();
        let requested_focus = self.requested_focus.take();
        let mutations = self.runner.sync_and_update();
        let result = self.runner.run_in(|| self.tree.apply_mutations(mutations));
        if result.needs_render {
            self.needs_layout = true;
        }
        self.layout();

        let tree_requested = self
            .atspi
            .as_ref()
            .is_some_and(|atspi| atspi.tree_requested());
        let full_tree = (!self.accessibility_initialized || tree_requested)
            .then(|| self.accessibility.init(&mut self.tree));
        self.accessibility_initialized = true;
        if let Some(strategy) = requested_focus {
            if matches!(
                strategy,
                AccessibilityFocusStrategy::Forward(_) | AccessibilityFocusStrategy::Backward(_)
            ) {
                self.platform.navigation_mode.set(NavigationMode::Keyboard);
            }
            self.tree.accessibility_diff.request_focus(strategy);
        }

        let accessibility_update = self
            .accessibility
            .process_updates(&mut self.tree, &self.events_sender);
        let focus_moved =
            *self.platform.focused_accessibility_id.peek() != accessibility_update.focus;
        self.platform
            .focused_accessibility_id
            .set_if_modified(accessibility_update.focus);
        if let Some(atspi) = &mut self.atspi {
            match full_tree {
                Some(tree) => atspi.adapter.update_if_active(|| tree),
                None if focus_moved || !accessibility_update.nodes.is_empty() => {
                    atspi.adapter.update_if_active(|| accessibility_update)
                }
                None => {}
            }
        }
        if let Some(node_id) = self.accessibility.focused_node_id()
            && let Some(layout_node) = self.tree.layout.get(&node_id)
        {
            self.platform.focused_accessibility_node.set_if_modified(
                AccessibilityTree::create_node(node_id, layout_node, &self.tree),
            );
        }
        if pressed {
            self.press(Key::Named(NamedKey::Enter), Code::Enter, Modifiers::empty());
        }
        for key in std::mem::take(&mut self.pending_shortcuts) {
            self.press(key, Code::Unidentified, Modifiers::CONTROL);
        }
        result.needs_render
    }

    // A Click from a screen reader is Freya's press gesture (Enter) on the node it focuses first.
    fn take_atspi_actions(&mut self) -> bool {
        let Some(atspi) = &self.atspi else {
            return false;
        };
        let mut pressed = false;
        for request in atspi.take_actions() {
            match request.action {
                accesskit::Action::Focus => {}
                accesskit::Action::Click => pressed = true,
                _ => continue,
            }
            self.requested_focus = Some(AccessibilityFocusStrategy::Node(request.target_node));
        }
        pressed
    }

    // A size read through on_sized must reach this frame, or a live resize paints the previous layout.
    pub(crate) fn settle_sized_reactions(&mut self) {
        for _ in 0..MAX_SETTLE_PASSES {
            if !(self.poll() && self.update()) {
                return;
            }
        }
    }

    pub(crate) fn layout(&mut self) {
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

    fn press(&mut self, key: Key, code: Code, modifiers: Modifiers) {
        for name in [KeyboardEventName::KeyDown, KeyboardEventName::KeyUp] {
            self.push(PlatformEvent::Keyboard {
                name,
                key: key.clone(),
                code,
                modifiers,
            });
        }
    }

    pub(crate) fn push(&mut self, event: PlatformEvent) {
        if matches!(
            event,
            PlatformEvent::Mouse {
                name: MouseEventName::MouseDown,
                ..
            }
        ) {
            self.platform
                .navigation_mode
                .set_if_modified(NavigationMode::NotKeyboard);
        }
        let mut adapter = EventsMeasurerAdapter {
            scale_factor: self.scale_factor,
            tree: &mut self.tree,
        };
        let focused = self.accessibility.focused_node_id();
        let processed = adapter.run(&mut vec![event], &mut self.nodes_state, focused);
        let _ = self
            .events_sender
            .unbounded_send(EventsChunk::Processed(processed));
        self.waker.wake_by_ref();
    }
}
