use std::{
    os::unix::net::UnixStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use infinity_protocol::server::style::infinity_window_style_v1::infinity_window_style_v1::{
    InfinityWindowStyleV1 as ServerStyle, Request as StyleRequest,
};
use sctk::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_subcompositor,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, Dispatch, EventQueue, Proxy, QueueHandle,
        globals::registry_queue_init,
        protocol::{wl_output, wl_surface::WlSurface},
    },
    registry::{ProvidesRegistryState, RegistryState},
    subcompositor::SubcompositorState,
};
use wayland_server::{
    Client, DataInit, Dispatch as ServerDispatch, Display, DisplayHandle,
    GlobalDispatch as ServerGlobalDispatch, New, Resource,
    backend::ClientData,
    protocol::{
        wl_compositor::{self, WlCompositor},
        wl_region::WlRegion,
        wl_subcompositor::{self, WlSubcompositor},
        wl_subsurface::{self, WlSubsurface},
        wl_surface::WlSurface as ServerSurface,
    },
};

use super::window_style::InfinityWindowStyleV1;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Seen {
    Subsurface {
        child: u32,
        parent: u32,
    },
    Position {
        child: u32,
        x: i32,
        y: i32,
    },
    Backdrop {
        surface: u32,
        blending: u32,
        material: u32,
        radius: u32,
        rects: Vec<i32>,
    },
    ScrollEdge {
        surface: u32,
        edge: u32,
        style: u32,
        depth: u32,
    },
    Glass {
        surface: u32,
        radius: u32,
        rects: Vec<i32>,
        base: u32,
        solid: u32,
        chroma: u32,
    },
}

type Log = Arc<Mutex<Vec<Seen>>>;

struct Server {
    seen: Log,
}

struct Nobody;

impl ClientData for Nobody {}

macro_rules! global {
    ($interface:ty) => {
        impl ServerGlobalDispatch<$interface, ()> for Server {
            fn bind(
                _: &mut Self,
                _: &DisplayHandle,
                _: &Client,
                resource: New<$interface>,
                _: &(),
                init: &mut DataInit<'_, Self>,
            ) {
                init.init(resource, ());
            }
        }
    };
}

global!(WlCompositor);
global!(WlSubcompositor);
global!(ServerStyle);

impl ServerDispatch<WlCompositor, ()> for Server {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &WlCompositor,
        request: wl_compositor::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                init.init(id, ());
            }
            wl_compositor::Request::CreateRegion { id } => {
                init.init(id, ());
            }
            _ => {}
        }
    }
}

impl ServerDispatch<ServerSurface, ()> for Server {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &ServerSurface,
        _: wayland_server::protocol::wl_surface::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
}

impl ServerDispatch<WlRegion, ()> for Server {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &WlRegion,
        _: wayland_server::protocol::wl_region::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
}

impl ServerDispatch<WlSubcompositor, ()> for Server {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &WlSubcompositor,
        request: wl_subcompositor::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let wl_subcompositor::Request::GetSubsurface {
            id,
            surface,
            parent,
        } = request
        {
            let child = surface.id().protocol_id();
            state.seen.lock().unwrap().push(Seen::Subsurface {
                child,
                parent: parent.id().protocol_id(),
            });
            init.init(id, child);
        }
    }
}

impl ServerDispatch<WlSubsurface, u32> for Server {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &WlSubsurface,
        request: wl_subsurface::Request,
        child: &u32,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        if let wl_subsurface::Request::SetPosition { x, y } = request {
            state.seen.lock().unwrap().push(Seen::Position {
                child: *child,
                x,
                y,
            });
        }
    }
}

fn integers(bytes: &[u8]) -> Vec<i32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| i32::from_ne_bytes(*chunk))
        .collect()
}

impl ServerDispatch<ServerStyle, ()> for Server {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &ServerStyle,
        request: StyleRequest,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        let seen = match request {
            StyleRequest::SetBackdrop {
                surface,
                blending,
                material,
                radius,
                rects,
            } => Seen::Backdrop {
                surface: surface.id().protocol_id(),
                blending: blending.into(),
                material: material.into(),
                radius,
                rects: integers(&rects),
            },
            StyleRequest::SetScrollEdge {
                surface,
                edge,
                style,
                depth,
            } => Seen::ScrollEdge {
                surface: surface.id().protocol_id(),
                edge: edge.into(),
                style: style.into(),
                depth,
            },
            StyleRequest::SetGlass {
                surface,
                radius,
                panels,
                base,
                solid,
                chroma,
                ..
            } => Seen::Glass {
                surface: surface.id().protocol_id(),
                radius,
                rects: integers(&panels),
                base,
                solid,
                chroma,
            },
            _ => return,
        };
        state.seen.lock().unwrap().push(seen);
    }
}

pub(super) struct FakeServer {
    pub(super) seen: Log,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl FakeServer {
    fn start(stream: UnixStream, style_version: Option<u32>) -> Self {
        let seen: Log = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let thread = std::thread::spawn({
            let (seen, stop) = (seen.clone(), stop.clone());
            move || {
                let mut display = Display::<Server>::new().unwrap();
                let mut handle = display.handle();
                handle.create_global::<Server, WlCompositor, _>(6, ());
                handle.create_global::<Server, WlSubcompositor, _>(1, ());
                if let Some(version) = style_version {
                    handle.create_global::<Server, ServerStyle, _>(version, ());
                }
                handle.insert_client(stream, Arc::new(Nobody)).unwrap();
                let mut state = Server { seen };
                while !stop.load(Ordering::Relaxed) {
                    display.dispatch_clients(&mut state).unwrap();
                    display.flush_clients().unwrap();
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        });
        Self {
            seen,
            stop,
            thread: Some(thread),
        }
    }

    pub(super) fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(super) struct Probe {
    registry: RegistryState,
    output: OutputState,
    pub(super) compositor: CompositorState,
    pub(super) subcompositor: SubcompositorState,
    pub(super) style: Option<InfinityWindowStyleV1>,
    pub(super) queue: QueueHandle<Probe>,
}

impl Probe {
    pub(super) fn connect(style_version: Option<u32>) -> (Self, EventQueue<Self>, FakeServer) {
        let (client, server) = UnixStream::pair().unwrap();
        let server = FakeServer::start(server, style_version);
        let connection = Connection::from_socket(client).unwrap();
        let (globals, mut queue) = registry_queue_init::<Self>(&connection).unwrap();
        let handle = queue.handle();
        let compositor = CompositorState::bind(&globals, &handle).unwrap();
        let subcompositor =
            SubcompositorState::bind(compositor.wl_compositor().clone(), &globals, &handle)
                .unwrap();
        let mut probe = Self {
            registry: RegistryState::new(&globals),
            output: OutputState::new(&globals, &handle),
            subcompositor,
            style: super::window_style::bind(&globals, &handle),
            compositor,
            queue: handle,
        };
        queue.roundtrip(&mut probe).unwrap();
        (probe, queue, server)
    }

    pub(super) fn surface(&self) -> WlSurface {
        self.compositor.create_surface(&self.queue)
    }
}

impl Dispatch<InfinityWindowStyleV1, ()> for Probe {
    fn event(
        _: &mut Self,
        _: &InfinityWindowStyleV1,
        _: <InfinityWindowStyleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl CompositorHandler for Probe {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: u32) {}

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for Probe {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    fn runtime_add_global(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: u32,
        _: &str,
        _: u32,
    ) {
    }

    fn runtime_remove_global(&mut self, _: &Connection, _: &QueueHandle<Self>, _: u32, _: &str) {}
}

impl OutputHandler for Probe {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

delegate_compositor!(Probe);
delegate_output!(Probe);
delegate_subcompositor!(Probe);
delegate_registry!(Probe);

#[test]
fn a_compositor_that_offers_version_ten_gets_version_ten() {
    let (probe, _queue, _server) = Probe::connect(Some(10));
    let style = probe.style.as_ref().unwrap();
    assert_eq!(style.version(), 10);
    assert!(super::window_style::offers_backdrop(style));
}

#[test]
fn a_compositor_that_offers_version_nine_gets_version_nine_and_no_backdrop() {
    let (probe, _queue, _server) = Probe::connect(Some(9));
    let style = probe.style.as_ref().unwrap();
    assert_eq!(style.version(), 9);
    assert!(!super::window_style::offers_backdrop(style));
}

#[test]
fn a_compositor_without_the_global_leaves_the_kit_without_it() {
    let (probe, _queue, _server) = Probe::connect(None);
    assert!(probe.style.is_none());
}
