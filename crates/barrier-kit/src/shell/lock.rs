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

use crate::platform::{
    Popups, RunError,
    app::AppInstance,
    runtime::{Connected, Runtime, insert_ping, make_sender},
    surface::{ExtensionRole, SurfaceRole},
    timer::{PingWake, Timers},
};
use crate::shell::ShellState;

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

pub fn run_lock(app: AppComponent) -> Result<(), RunError> {
    let Connected {
        conn,
        event_queue,
        mut runtime,
        globals,
    } = Runtime::connect()?;
    let qh = runtime.qh.clone();
    ShellState::install(&mut runtime, app, &globals, &qh);
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
    runtime
        .extension_mut::<ShellState>()
        .expect("ShellState not installed after install")
        .active_lock = Some(lock);

    let mut event_loop: EventLoop<Runtime> = EventLoop::try_new()
        .map_err(|_| RunError::EventLoop("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut runtime)?;
    let unlock_conn = conn.clone();
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
                // ext-session-lock-v1: wait for unlock_and_destroy to be processed before exiting.
                let _ = unlock_conn.roundtrip();
                signal.stop();
            }),
        });
    }
    let outputs: Vec<wl_output::WlOutput> = runtime.output.outputs().collect();
    for output in outputs {
        runtime.create_lock_surface(&qh, &output);
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
            opens_left: Rc::new(Cell::new(false)),
            scroll_shift: Rc::new(Cell::new(0.0)),
        };
        let app = {
            let shell = self
                .extension::<ShellState>()
                .expect("create_lock_surface called without ShellState");
            shell.app.clone()
        };
        let app = AppInstance::new(
            app,
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
            self.resolved,
            self.clipboard.provider(),
            self.defaults_revision,
            |r| {
                if let Some(hooks) = self.extension_hooks() {
                    (hooks.provide_contexts)(self, r);
                }
            },
        );
        let mut window =
            self.surface_window(surface, SurfaceRole::Extension(Box::new(lock_surface)), app);
        window.output = Some(output.id());
        self.surfaces.push(window);
        Some(self.surfaces.len() - 1)
    }
}

impl SessionLockHandler for Runtime {
    fn locked(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {}

    fn finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
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
        self.surfaces[index].draw_after_configure(&self.compositor, qh);
    }
}

impl ExtensionRole for SessionLockSurface {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

delegate_session_lock!(Runtime);
