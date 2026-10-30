use freya_core::integration::AppComponent;
use sctk::{
    delegate_session_lock,
    reexports::{
        calloop::EventLoop,
        calloop_wayland_source::WaylandSource,
        client::{Connection, Proxy, QueueHandle, protocol::wl_output},
    },
    session_lock::{
        SessionLock, SessionLockHandler, SessionLockSurface, SessionLockSurfaceConfigure,
    },
};
use std::{cell::Cell, rc::Rc, sync::Arc};

use super::{
    RunError,
    app::AppInstance,
    popup::Popups,
    runtime::{Connected, Runtime, insert_ping, make_sender},
    shell_state::ShellState,
    surface::{ExtensionRole, SurfaceRole, SurfaceWindow},
    timer::{PingWake, Timers},
};

/// Unlocks the session from the lock app. Provided only to lock surfaces.
#[derive(Clone)]
pub struct SessionUnlock {
    unlock: Rc<dyn Fn()>,
}

impl SessionUnlock {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn unlock(&self) {
        (self.unlock)();
    }
}

/// Runs the session lock: one Freya surface per output.
pub fn run_lock(app: AppComponent) -> Result<(), RunError> {
    let Connected {
        conn,
        event_queue,
        mut runtime,
        globals,
    } = Runtime::connect(app)?;
    let qh = runtime.qh.clone();
    ShellState::install(&mut runtime, &globals, &qh, true);
    let lock = {
        let session_lock_state = runtime
            .extension::<ShellState>()
            .expect("ShellState not installed after install")
            .session_lock
            .as_ref()
            .ok_or(RunError::MissingGlobal("ext_session_lock_manager_v1"))?;
        session_lock_state
            .lock(&qh)
            .map_err(|_| RunError::MissingGlobal("ext_session_lock_manager_v1 (lock refused)"))?
    };
    let unlock_lock = lock.clone();
    {
        let shell = runtime
            .extension_mut::<ShellState>()
            .expect("ShellState not installed after install");
        shell.active_lock = Some(lock);
    }

    let mut event_loop: EventLoop<Runtime> = EventLoop::try_new()
        .map_err(|_| RunError::EventLoop("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut runtime)?;
    let flush_conn = conn.clone();
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .map_err(|_| RunError::EventLoop("inserting the Wayland source"))?;
    let signal = event_loop.get_signal();
    runtime.loop_signal = Some(signal.clone());
    {
        let shell = runtime
            .extension_mut::<ShellState>()
            .expect("ShellState not installed");
        shell.session_unlock = Some(SessionUnlock {
            unlock: Rc::new(move || {
                unlock_lock.unlock();
                let _ = flush_conn.flush();
                signal.stop();
            }),
        });
    }
    event_loop
        .run(None, &mut runtime, |runtime| {
            if runtime.needs_exit
                && let Some(signal) = &runtime.loop_signal
            {
                signal.stop();
            }
        })
        .map_err(|_| RunError::EventLoop("calloop event loop failed"))?;
    Ok(())
}

impl Runtime {
    pub(super) fn create_lock_surface(
        &mut self,
        qh: &QueueHandle<Self>,
        output: &wl_output::WlOutput,
    ) -> Option<usize> {
        if self.surfaces.iter().any(|s| s.output == Some(output.id())) {
            return None;
        }
        let lock = self
            .extension::<ShellState>()
            .expect("create_lock_surface called without ShellState")
            .active_lock
            .clone()?;
        let surface = self.compositor.create_surface(qh);
        let lock_surface = lock.create_lock_surface(surface.clone(), output, qh);
        let waker = futures_util::task::waker(Arc::new(PingWake(self.ping.clone())));
        let sender = make_sender(self.queue.clone(), self.ping.clone(), surface.id());
        let popups = Popups {
            queue: self.popup_commands.clone(),
            ping: self.ping.clone(),
            next_id: self.popup_next_id.clone(),
            last_serial: self.last_input_serial.clone(),
            owner: surface.id(),
            current: None,
            surface_count: self.surface_count.clone(),
            opens_left: Rc::new(Cell::new(false)),
        };
        let app = AppInstance::new(
            self.app.clone(),
            1.0,
            (1, 1),
            sender,
            waker,
            None,
            Timers {
                queue: self.timers.clone(),
                ping: self.ping.clone(),
                wake: self.wake.clone(),
            },
            popups,
            None,
            self.graphics.font_collection.clone(),
            self.graphics.font_manager.clone(),
            None,
            None,
            self.resolved,
            |r| {
                if let Some(hooks) = self.extension_hooks() {
                    (hooks.provide_contexts)(self, r);
                }
            },
        );
        let (fractional, viewport) = self.create_fractional(&surface);
        self.surfaces.push(SurfaceWindow {
            surface,
            role: SurfaceRole::Extension(Box::new(lock_surface)),
            popup: None,
            shared: self.graphics.clone(),
            graphics: None,
            app,
            logical_size: (0, 0),
            buffer_size: (0, 0),
            scale: 1.0,
            output: Some(output.id()),
            needs_redraw: true,
            frame_pending: false,
            draw_count: 0,
            configure_count: 0,
            pending_configure: None,
            appmenu: None,
            appmenu_address: None,
            menu_handle: None,
            fractional,
            viewport,
        });
        self.surface_count.set(self.surfaces.len());
        Some(self.surfaces.len() - 1)
    }
}

impl SessionLockHandler for Runtime {
    fn locked(&mut self, _: &Connection, qh: &QueueHandle<Self>, session_lock: SessionLock) {
        {
            let shell = self
                .extension_mut::<ShellState>()
                .expect("session_lock locked event without ShellState");
            shell.active_lock = Some(session_lock);
            shell.session_locked = true;
        }
        let outputs: Vec<wl_output::WlOutput> = self.output.outputs().collect();
        for output in outputs {
            self.create_lock_surface(qh, &output);
        }
    }

    fn finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        let shell = self
            .extension_mut::<ShellState>()
            .expect("session_lock finished event without ShellState");
        shell.session_locked = false;
        eprintln!("malaysia-shell lock: the compositor refused the session lock");
        std::process::exit(1);
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        surface: SessionLockSurface,
        configure: SessionLockSurfaceConfigure,
        _serial: u32,
    ) {
        let wl_surface = surface.wl_surface().clone();
        let Some(index) = self.surface_for(&wl_surface) else {
            return;
        };
        let scale = self.surfaces[index].scale;
        self.surfaces[index].configure(configure.new_size, scale);
        self.surfaces[index].request_redraw(qh);
    }
}

impl ExtensionRole for SessionLockSurface {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

delegate_session_lock!(Runtime);
