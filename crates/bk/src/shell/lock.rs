use freya_core::integration::AppComponent;
use freya_core::prelude::WritableUtils;
use sctk::{
    delegate_session_lock,
    reexports::client::{Connection, Proxy, QueueHandle, protocol::wl_output},
    session_lock::{
        SessionLock, SessionLockHandler, SessionLockSurface, SessionLockSurfaceConfigure,
    },
};
use std::rc::Rc;

use crate::platform::{
    RunError,
    app::{AppInstance, InstanceRole},
    runtime::{Connected, Runtime},
    surface::{ExtensionRole, SurfaceRole},
};
use crate::shell::ShellState;

#[derive(Clone)]
pub struct SessionUnlock {
    unlock: Rc<dyn Fn()>,
    requested: Rc<std::cell::Cell<bool>>,
    locked: freya_core::prelude::State<bool>,
}

impl SessionUnlock {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn unlock(&self) {
        (self.unlock)();
    }

    pub fn locked(&self) -> bool {
        *self.locked.read()
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

    let unlock_conn = conn.clone();
    let requested = Rc::new(std::cell::Cell::new(false));
    let locked = freya_core::prelude::State::create_global(false);
    runtime.run(conn, event_queue, move |runtime, signal| {
        runtime
            .extension_mut::<ShellState>()
            .expect("ShellState not installed")
            .session_unlock = Some(SessionUnlock {
            requested: requested.clone(),
            locked,
            unlock: Rc::new(move || {
                // ext-session-lock-v1: unlock_and_destroy is only valid once locked has arrived.
                if !unlock_lock.is_locked() {
                    requested.set(true);
                    return;
                }
                unlock_lock.unlock();
                // ext-session-lock-v1: wait for unlock_and_destroy to be processed before exiting.
                let _ = unlock_conn.roundtrip();
                signal.stop();
            }),
        });
        let outputs: Vec<wl_output::WlOutput> = runtime.output.outputs().collect();
        for output in outputs {
            runtime.create_lock_surface(&qh, &output);
        }
    })
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
        let app = self
            .extension::<ShellState>()
            .expect("create_lock_surface called without ShellState")
            .app
            .clone();
        let app = AppInstance::new(self, surface.id(), app, 1.0, (1, 1), InstanceRole::Surface);
        let mut window =
            self.surface_window(surface, SurfaceRole::Extension(Box::new(lock_surface)), app);
        window.output = Some(output.id());
        self.surfaces.push(window);
        Some(self.surfaces.len() - 1)
    }
}

impl SessionLockHandler for Runtime {
    fn locked(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        let unlock = self
            .extension::<ShellState>()
            .and_then(|shell| shell.session_unlock.clone());
        let Some(unlock) = unlock else {
            return;
        };
        let mut locked = unlock.locked;
        locked.set(true);
        self.request_redraw_all = true;
        self.ping.ping();
        if unlock.requested.get() {
            unlock.unlock();
        }
    }

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
