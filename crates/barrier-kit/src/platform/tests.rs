#![allow(clippy::module_inception)]

#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod tests {
    use super::input::seat::{
        RepeatState, cursor_shape_from_icon, dispatch_touch_cancels, find_index_by_id,
        released_repeated_key, should_repeat_key, stop_repeat,
    };
    use super::surface::{
        buffer_size, destroy_fractional_objects, fractional_scale_enabled, physical_point,
        scale_from_wire,
    };
    use super::timer::{Timers, Wake, WakeQueue};
    use super::toplevels::{InfoEvent, apply_info_event};
    use super::window::clamp_toplevel_size_dimension;
    use super::*;
    use sctk::seat::keyboard::Keysym;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn wakeup_runs_its_callback_on_the_loop_thread() {
        let (ping, source) = calloop::ping::make_ping().unwrap();
        let callbacks: WakeQueue = Rc::new(RefCell::new(Vec::new()));
        let mut event_loop = EventLoop::<()>::try_new().unwrap();
        let queue = callbacks.clone();
        event_loop
            .handle()
            .insert_source(source, move |_, _, ()| {
                for callback in queue.borrow().clone() {
                    callback();
                }
            })
            .unwrap();
        let timers = Timers {
            queue: Rc::new(RefCell::new(Vec::new())),
            ping: ping.clone(),
            wake: Wake { ping, callbacks },
        };
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        let handle = timers.wakeup(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        let thread_handle = handle.clone();
        let writer = std::thread::spawn(move || thread_handle.wake());
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while seen.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
            event_loop
                .dispatch(Duration::from_millis(10), &mut ())
                .unwrap();
        }
        writer.join().unwrap();
        assert_eq!(seen.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn toplevel_size_dimension_clamps_to_unbounded() {
        assert_eq!(clamp_toplevel_size_dimension(u32::MAX), 0);
        assert_eq!(clamp_toplevel_size_dimension(i32::MAX as u32 + 1), 0);
        assert_eq!(
            clamp_toplevel_size_dimension(i32::MAX as u32),
            i32::MAX as u32
        );
        assert_eq!(clamp_toplevel_size_dimension(760), 760);
        assert_eq!(clamp_toplevel_size_dimension(0), 0);
    }

    #[test]
    fn scale_from_wire_converts_fractional_scale_values() {
        assert!((scale_from_wire(120) - 1.0).abs() < f64::EPSILON);
        assert!((scale_from_wire(150) - 1.25).abs() < f64::EPSILON);
        assert!((scale_from_wire(180) - 1.5).abs() < f64::EPSILON);
        assert!((scale_from_wire(240) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn buffer_size_rounds_logical_size_at_fractional_scale() {
        assert_eq!(buffer_size((801, 601), 1.25), (1001, 751));
    }

    #[test]
    fn fractional_scale_mode_needs_both_globals() {
        assert!(fractional_scale_enabled(true, true, 1.25));
        assert!(!fractional_scale_enabled(true, false, 1.25));
        assert!(!fractional_scale_enabled(false, true, 1.25));
        assert!(!fractional_scale_enabled(false, false, 1.25));
    }

    #[test]
    fn pointer_and_touch_coordinates_become_physical() {
        assert_eq!(
            physical_point((10.0, 20.0), 1.5),
            CursorPoint::from((15.0, 30.0))
        );
        assert_eq!(
            physical_point((3.0, 4.0), 2.0),
            CursorPoint::from((6.0, 8.0))
        );
        assert_eq!(
            physical_point((10.0, 20.0), 1.25),
            CursorPoint::from((12.5, 25.0))
        );
    }

    #[test]
    fn fractional_scale_hit_test_respects_one_pixel_border() {
        let scale = 1.25;
        let border_physical = 1.0 * scale;
        let inside = physical_point((0.5, 0.5), scale);
        let outside = physical_point((1.5, 1.5), scale);
        assert!(inside.x < border_physical);
        assert!(inside.y < border_physical);
        assert!(outside.x >= border_physical);
        assert!(outside.y >= border_physical);
    }

    #[test]
    fn losing_focus_cancels_repeat_and_invalidates_old_timers() {
        let event = KeyEvent {
            time: 0,
            raw_code: 0,
            keysym: Keysym::new(0),
            utf8: None,
        };
        let mut repeat = Some(RepeatState {
            event,
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: false,
            pending: true,
            generation: 7,
        });
        let mut generation = 7;
        stop_repeat(&mut repeat, &mut generation);
        assert!(repeat.is_none());
        assert_eq!(generation, 8);
    }

    #[test]
    fn modifier_keys_do_not_start_repeat() {
        let shift = KeyEvent {
            time: 0,
            raw_code: 50,
            keysym: Keysym::Shift_L,
            utf8: None,
        };
        assert!(!should_repeat_key(&shift));
    }

    #[test]
    fn regular_keys_start_repeat() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        assert!(should_repeat_key(&a));
    }

    #[test]
    fn releasing_repeated_key_stops_repeat() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        let repeat = Some(RepeatState {
            event: a.clone(),
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: true,
            pending: false,
            generation: 1,
        });
        assert!(released_repeated_key(&repeat, &a));
    }

    #[test]
    fn releasing_a_different_key_keeps_repeat_running() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        let b = KeyEvent {
            time: 0,
            raw_code: 56,
            keysym: Keysym::b,
            utf8: Some("b".into()),
        };
        let repeat = Some(RepeatState {
            event: b,
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: true,
            pending: false,
            generation: 1,
        });
        assert!(!released_repeated_key(&repeat, &a));
    }

    #[test]
    fn pressing_b_then_releasing_a_keeps_b_repeating() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        let b = KeyEvent {
            time: 0,
            raw_code: 56,
            keysym: Keysym::b,
            utf8: Some("b".into()),
        };

        // Press A: it starts repeating.
        let mut repeat: Option<RepeatState> = None;
        if should_repeat_key(&a) {
            repeat = Some(RepeatState {
                event: a.clone(),
                delay: Duration::from_millis(200),
                rate: Duration::from_millis(40),
                first: true,
                pending: false,
                generation: 1,
            });
        }

        // Press B while A is held: B becomes the repeated key.
        if should_repeat_key(&b) {
            repeat = Some(RepeatState {
                event: b.clone(),
                delay: Duration::from_millis(200),
                rate: Duration::from_millis(40),
                first: true,
                pending: false,
                generation: 2,
            });
        }

        // Release A: repeat must stay alive because B is the repeated key.
        assert!(!released_repeated_key(&repeat, &a));
        assert!(
            repeat
                .as_ref()
                .is_some_and(|state| state.event.keysym == b.keysym)
        );
    }

    #[test]
    fn interleaved_info_events_update_their_own_toplevel() {
        let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
        apply_info_event(&mut infos, 0, InfoEvent::Title("Editor".into()));
        apply_info_event(&mut infos, 1, InfoEvent::Title("Navegador".into()));
        apply_info_event(&mut infos, 0, InfoEvent::AppId("editor".into()));
        apply_info_event(
            &mut infos,
            1,
            InfoEvent::State {
                active: true,
                maximized: true,
                minimized: false,
            },
        );
        apply_info_event(
            &mut infos,
            0,
            InfoEvent::State {
                active: false,
                maximized: false,
                minimized: true,
            },
        );

        assert_eq!(infos[0].title, "Editor");
        assert_eq!(infos[0].app_id, "editor");
        assert!(!infos[0].active);
        assert!(!infos[0].maximized);
        assert!(infos[0].minimized);
        assert_eq!(infos[1].title, "Navegador");
        assert!(infos[1].active);
        assert!(infos[1].maximized);
        assert!(!infos[1].minimized);

        apply_info_event(&mut infos, 0, InfoEvent::Closed);
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].title, "Navegador");
    }

    #[test]
    fn app_menu_event_updates_only_its_toplevel() {
        let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
        apply_info_event(
            &mut infos,
            0,
            InfoEvent::AppMenu(Some(AppMenuAddress {
                service: ":1.2".to_string(),
                path: "/MenuBar/1".to_string(),
            })),
        );
        assert_eq!(
            infos[0].app_menu.as_ref().map(|a| a.service.as_str()),
            Some(":1.2")
        );
        assert!(infos[1].app_menu.is_none());

        apply_info_event(&mut infos, 0, InfoEvent::AppMenu(None));
        assert!(infos[0].app_menu.is_none());
    }

    #[test]
    fn gtk_menus_event_updates_only_its_toplevel() {
        let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
        apply_info_event(
            &mut infos,
            0,
            InfoEvent::GtkMenus(GtkMenusAddress::new(
                ":1.3".to_string(),
                "org.gtk.Demo".to_string(),
                "/menubar".to_string(),
                "/appmenu".to_string(),
                "/application".to_string(),
                "/window".to_string(),
            )),
        );
        assert_eq!(
            infos[0]
                .gtk_menus
                .as_ref()
                .map(|a| a.unique_bus_name.as_str()),
            Some(":1.3")
        );
        assert!(infos[1].gtk_menus.is_none());

        apply_info_event(&mut infos, 0, InfoEvent::GtkMenus(None));
        assert!(infos[0].gtk_menus.is_none());
    }

    #[test]
    fn gtk_menus_address_needs_a_bus_name() {
        assert!(
            GtkMenusAddress::new(
                "".to_string(),
                "org.gtk.Demo".to_string(),
                "/menubar".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .is_none()
        );
    }

    #[test]
    fn gtk_menus_address_accepts_app_menu_only() {
        let address = GtkMenusAddress::new(
            ":1.2".to_string(),
            "org.gtk.Demo".to_string(),
            "".to_string(),
            "/appmenu".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert!(address.is_some());
        assert_eq!(address.unwrap().app_menu_path, "/appmenu");
    }

    #[test]
    fn gtk_menus_address_accepts_menubar_only() {
        let address = GtkMenusAddress::new(
            ":1.2".to_string(),
            "org.gtk.Demo".to_string(),
            "/menubar".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert!(address.is_some());
        assert_eq!(address.unwrap().menubar_path, "/menubar");
    }

    #[test]
    fn gtk_menus_address_rejects_both_paths_empty() {
        assert!(
            GtkMenusAddress::new(
                ":1.2".to_string(),
                "org.gtk.Demo".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .is_none()
        );
    }

    #[test]
    fn sender_tags_user_events_with_surface_id() {
        let (ping, _) = calloop::ping::make_ping().unwrap();
        let queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>> = Rc::new(RefCell::new(Vec::new()));
        let owner = ObjectId::null();
        let sender = make_sender(queue.clone(), ping, owner.clone());
        sender(UserEvent::RequestRedraw);
        let events = drain_user_events(&queue);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, owner);
    }

    #[test]
    fn cursor_icon_maps_to_wayland_shape() {
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::Default),
            CursorShape::Default
        );
        assert_eq!(cursor_shape_from_icon(CursorIcon::Text), CursorShape::Text);
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::Pointer),
            CursorShape::Pointer
        );
        assert_eq!(cursor_shape_from_icon(CursorIcon::Grab), CursorShape::Grab);
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::Grabbing),
            CursorShape::Grabbing
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::NotAllowed),
            CursorShape::NotAllowed
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::NResize),
            CursorShape::NResize
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::EResize),
            CursorShape::EResize
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::EwResize),
            CursorShape::EwResize
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::NsResize),
            CursorShape::NsResize
        );
    }

    #[test]
    fn touch_cancel_dispatches_cancelled_for_every_active_contact() {
        let mut touch_surfaces: std::collections::HashMap<i32, ObjectId> =
            std::collections::HashMap::new();
        let surface_a = ObjectId::null();
        let surface_b = ObjectId::null();
        // Simulate two fingers down on surface A and one on surface B.
        touch_surfaces.insert(0, surface_a.clone());
        touch_surfaces.insert(1, surface_a.clone());
        touch_surfaces.insert(2, surface_b.clone());

        let mut cancelled = Vec::new();
        dispatch_touch_cancels(&mut touch_surfaces, |id, surface_id| {
            cancelled.push((id, surface_id));
        });

        assert!(
            touch_surfaces.is_empty(),
            "touch_surfaces must be empty after cancel"
        );
        assert_eq!(cancelled.len(), 3);
        assert!(cancelled.contains(&(0, surface_a.clone())));
        assert!(cancelled.contains(&(1, surface_a.clone())));
        assert!(cancelled.contains(&(2, surface_b.clone())));
    }

    #[test]
    fn touch_routing_uses_the_stored_surface_id() {
        // We cannot build real WlSurface objects in a unit test, so we model
        // the surfaces as (label, id) tuples. Surface B was created first,
        // surface A second; a touch started on A must route to A, not to B.
        let surfaces = vec![('B', 1usize), ('A', 2usize)];
        let touch_id = 7;
        let stored_surface_id = 2usize;

        // Simulate down storing A's id for this touch.
        let mut touch_surfaces: std::collections::HashMap<i32, usize> =
            std::collections::HashMap::new();
        touch_surfaces.insert(touch_id, stored_surface_id);

        // Simulate motion / up: the stored id is looked up in the surface list.
        let routed_id = touch_surfaces.get(&touch_id).copied();
        let index = find_index_by_id(&surfaces, routed_id.unwrap(), |item| item.1);

        assert_eq!(routed_id, Some(stored_surface_id));
        assert_eq!(
            index,
            Some(1),
            "motion/up for the same touch id must reach surface A"
        );
    }

    #[test]
    fn destroy_fractional_objects_is_safe_when_missing() {
        let mut fractional: Option<WpFractionalScaleV1> = None;
        let mut viewport: Option<WpViewport> = None;
        destroy_fractional_objects(&mut fractional, &mut viewport);
        assert!(fractional.is_none());
        assert!(viewport.is_none());
    }

    // Manual test for destroy order:
    // Run with WAYLAND_DEBUG=1 and close a window. Verify that
    // wp_fractional_scale_v1@...destroy and wp_viewport@...destroy lines appear
    // before the wl_surface@...destroy line for the same surface.

    // Manual test for the configure overwrite behaviour:
    // Run with WAYLAND_DEBUG=1 while resizing. Two configures arriving in the
    // same batch (same size, different state) must result in the second state
    // being applied; the code now stores only the latest configure in
    // `pending_configure: Option<_>`.
}

#[cfg(test)]
mod drag_band_tests {
    use super::*;
    use crate::ProbeExt;
    use crate::sidebar::{Sidebar, SidebarItem, SidebarLayout};
    use freya_core::integration::AppComponent;
    use freya_core::prelude::{App, IntoElement, rect};
    use freya_testing::TestingRunner;
    use torin::prelude::{Size, Size2D};
    use vow_appearance::{AppearanceConfig, SidebarStyle, resolve};
    use window::WindowCommand;

    struct DragCase;

    impl App for DragCase {
        fn render(&self) -> impl IntoElement {
            SidebarLayout::new(
                Sidebar::new()
                    .traffic_lights(true)
                    .child(SidebarItem::new("Uno").probe("island-first-item")),
                rect().width(Size::fill()).height(Size::fill()),
            )
        }
    }

    #[test]
    fn a_mouse_down_on_the_island_top_band_moves_the_window() {
        let queue: Rc<RefCell<Vec<WindowCommand>>> = Rc::new(RefCell::new(Vec::new()));
        let controls_queue = queue.clone();
        let resolved = resolve(
            &AppearanceConfig {
                sidebar_style: SidebarStyle::Island,
                ..Default::default()
            },
            None,
        );
        let app = AppComponent::new(DragCase);
        let (mut test, _) = TestingRunner::new(
            app,
            Size2D::new(400.0, 400.0),
            move |runner| {
                runner.provide_root_context(move || {
                    let (ping, _source) = calloop::ping::make_ping().unwrap();
                    WindowControls {
                        queue: controls_queue.clone(),
                        ping,
                        pointer_serial: Rc::new(Cell::new(0)),
                        status: State::create(WindowStatus {
                            activated: true,
                            maximized: false,
                            decoration: DecorationResponse::Client,
                            fixed: false,
                        }),
                        title: "Ventana".to_string(),
                        fixed: false,
                        surface_id: ObjectId::null(),
                        menu_handle: None,
                    }
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.sync_and_update();
        test.press_cursor((100.0, 4.0));
        assert!(
            queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on the island top band must request a window move"
        );

        queue.borrow_mut().clear();
        test.press_cursor((30.0, 30.0));
        assert!(
            !queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on a traffic light must not move the window"
        );

        queue.borrow_mut().clear();
        let (x, y) = test
            .find(|node, el| {
                (el.accessibility().builder.author_id() == Some("island-first-item")).then(|| {
                    let area = node.layout().area;
                    (
                        area.origin.x + area.size.width / 2.0,
                        area.origin.y + area.size.height / 2.0,
                    )
                })
            })
            .expect("the first island row exists");
        test.press_cursor((f64::from(x), f64::from(y)));
        assert!(
            !queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on the first island row must not move the window"
        );
    }
}
