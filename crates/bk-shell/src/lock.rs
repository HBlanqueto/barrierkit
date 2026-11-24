use bk::{
    extension::{AppInstance, Connected, ExtensionRole, InstanceRole, Runtime, SurfaceRole},
    platform::RunError,
};
use freya_core::integration::AppComponent;
use freya_core::prelude::WritableUtils;
use sctk::reexports::{
    client::{
        Connection, Dispatch, Proxy, QueueHandle,
        protocol::{wl_output, wl_surface::WlSurface},
    },
    protocols::ext::session_lock::v1::client::{
        ext_session_lock_manager_v1::{self, ExtSessionLockManagerV1},
        ext_session_lock_surface_v1::{self, ExtSessionLockSurfaceV1},
        ext_session_lock_v1::{self, ExtSessionLockV1},
    },
};
use std::rc::Rc;

use crate::{Shell, ShellState};

#[derive(Clone)]
pub struct SessionUnlock {
    unlock: Rc<dyn Fn()>,
    requested: Rc<std::cell::Cell<bool>>,
    locked: freya_core::prelude::State<bool>,
    keyboard_layout: freya_core::prelude::State<Option<String>>,
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

    pub fn keyboard_layout(&self) -> Option<String> {
        self.keyboard_layout.read().clone()
    }
}

// ext-session-lock-v1: a dropped lock is destroyed; one that locked and was not unlocked keeps the session locked.
pub(crate) struct Lock(ExtSessionLockV1);

impl Drop for Lock {
    fn drop(&mut self) {
        self.0.destroy();
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
    let shell = runtime
        .extension_mut::<ShellState>()
        .expect("ShellState not installed after install");
    let lock = shell
        .session_lock
        .as_ref()
        .ok_or(RunError::MissingGlobal("ext_session_lock_manager_v1"))?
        .lock(&qh, Shell);
    let unlock_lock = lock.clone();
    shell.active_lock = Some(Lock(lock));

    let unlock_conn = conn.clone();
    let requested = Rc::new(std::cell::Cell::new(false));
    let locked = freya_core::prelude::State::create_global(false);
    let keyboard_layout = freya_core::prelude::State::create_global(None);
    runtime.keyboard_layout = Some(keyboard_layout);
    runtime.run(conn, event_queue, move |runtime, signal| {
        runtime
            .extension_mut::<ShellState>()
            .expect("ShellState not installed")
            .session_unlock = Some(SessionUnlock {
            requested: requested.clone(),
            locked,
            keyboard_layout,
            unlock: Rc::new(move || {
                // ext-session-lock-v1: unlock_and_destroy is only valid once locked has arrived.
                if !*locked.peek() {
                    requested.set(true);
                    return;
                }
                unlock_lock.unlock_and_destroy();
                // ext-session-lock-v1: wait for unlock_and_destroy to be processed before exiting.
                let _ = unlock_conn.roundtrip();
                signal.stop();
            }),
        });
        let outputs: Vec<wl_output::WlOutput> = runtime.output.outputs().collect();
        for output in outputs {
            create_lock_surface(runtime, &qh, &output);
        }
    })
}

pub(crate) fn create_lock_surface(
    rt: &mut Runtime,
    qh: &QueueHandle<Runtime>,
    output: &wl_output::WlOutput,
) {
    if rt.surfaces.iter().any(|s| s.output == Some(output.id())) {
        return;
    }
    let Some(shell) = rt.extension::<ShellState>() else {
        return;
    };
    let Some(Lock(lock)) = &shell.active_lock else {
        return;
    };
    let surface = rt.compositor.create_surface(qh);
    let role = LockRole {
        lock: lock.get_lock_surface(&surface, output, qh, Shell),
        surface: surface.clone(),
    };
    let app = AppInstance::new(
        &rt.host(),
        surface.id(),
        shell.app.clone(),
        1.0,
        (1, 1),
        InstanceRole::Surface,
        |runner| rt.provide_extension_contexts(runner),
    );
    let mut window = rt.surface_window(surface, SurfaceRole::Extension(Box::new(role)), app);
    window.output = Some(output.id());
    rt.surfaces.push(window);
}

#[derive(Debug)]
struct LockRole {
    lock: ExtSessionLockSurfaceV1,
    surface: WlSurface,
}

// ext_session_lock_surface_v1: the role object is destroyed before its wl_surface.
impl Drop for LockRole {
    fn drop(&mut self) {
        self.lock.destroy();
        self.surface.destroy();
    }
}

impl ExtensionRole for LockRole {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Dispatch<ExtSessionLockManagerV1, Shell> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtSessionLockManagerV1,
        _: ext_session_lock_manager_v1::Event,
        _: &Shell,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtSessionLockV1, Shell> for Runtime {
    fn event(
        rt: &mut Self,
        _: &ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &Shell,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_session_lock_v1::Event::Locked => {
                let Some(unlock) = rt
                    .extension::<ShellState>()
                    .and_then(|shell| shell.session_unlock.clone())
                else {
                    return;
                };
                let mut locked = unlock.locked;
                locked.set(true);
                rt.request_redraw_all = true;
                rt.ping.ping();
                if unlock.requested.get() {
                    unlock.unlock();
                }
            }
            ext_session_lock_v1::Event::Finished => {
                tracing::error!("BarrierKit: the compositor ended the session lock");
                std::process::exit(1);
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtSessionLockSurfaceV1, Shell> for Runtime {
    fn event(
        rt: &mut Self,
        lock_surface: &ExtSessionLockSurfaceV1,
        event: ext_session_lock_surface_v1::Event,
        _: &Shell,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let ext_session_lock_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        else {
            return;
        };
        lock_surface.ack_configure(serial);
        let Some(index) = rt.surfaces.iter().position(|s| {
            s.role
                .as_extension_role::<LockRole>()
                .is_some_and(|role| role.lock == *lock_surface)
        }) else {
            return;
        };
        let scale = rt.surfaces[index].scale;
        rt.surfaces[index].configure((width, height), scale);
        rt.surfaces[index].draw_after_configure(&rt.compositor, qh);
    }
}
