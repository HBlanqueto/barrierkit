use std::rc::Rc;

use sctk::reexports::client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum, event_created_child,
    globals::GlobalList,
    protocol::{wl_buffer::WlBuffer, wl_shm},
};
use sctk::reexports::protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1::{Event as HandleEvent, ExtForeignToplevelHandleV1},
    ext_foreign_toplevel_list_v1::{self, Event as ListEvent, ExtForeignToplevelListV1},
};
use sctk::reexports::protocols::ext::image_capture_source::v1::client::{
    ext_foreign_toplevel_image_capture_source_manager_v1::ExtForeignToplevelImageCaptureSourceManagerV1,
    ext_image_capture_source_v1::ExtImageCaptureSourceV1,
};
use sctk::reexports::protocols::ext::image_copy_capture::v1::client::{
    ext_image_copy_capture_frame_v1::{Event as FrameEvent, ExtImageCopyCaptureFrameV1},
    ext_image_copy_capture_manager_v1::{ExtImageCopyCaptureManagerV1, Options},
    ext_image_copy_capture_session_v1::{Event as SessionEvent, ExtImageCopyCaptureSessionV1},
};
use sctk::shm::{Shm, raw::RawPool};

use crate::platform::runtime::Runtime;
use crate::shell::ShellState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct CaptureWindow {
    pub window: u64,
    pub done: Rc<dyn Fn(Option<WindowImage>)>,
}

pub fn capture_window(window: u64, done: impl Fn(Option<WindowImage>) + 'static) {
    crate::platform::KitSender::get().send(CaptureWindow {
        window,
        done: Rc::new(done),
    });
}

// A second ext_foreign_toplevel_list: the capture source needs its handles even when the
// windows are listed through zwlr_foreign_toplevel_management_v1.
pub(crate) struct Listing;

pub(crate) struct CaptureBuffer;

struct Listed {
    handle: ExtForeignToplevelHandleV1,
    title: String,
    app_id: String,
}

struct Pending {
    session: ExtImageCopyCaptureSessionV1,
    source: ExtImageCaptureSourceV1,
    size: Option<(u32, u32)>,
    format: Option<wl_shm::Format>,
    frame: Option<(ExtImageCopyCaptureFrameV1, RawPool, WlBuffer)>,
    done: Rc<dyn Fn(Option<WindowImage>)>,
}

pub(crate) struct Capture {
    _list: ExtForeignToplevelListV1,
    sources: ExtForeignToplevelImageCaptureSourceManagerV1,
    copy: ExtImageCopyCaptureManagerV1,
    shm: Shm,
    listed: Vec<Listed>,
    pending: Vec<Pending>,
}

impl Capture {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<Runtime>) -> Option<Self> {
        Some(Self {
            _list: globals.bind(qh, 1..=1, Listing).ok()?,
            sources: globals.bind(qh, 1..=1, ()).ok()?,
            copy: globals.bind(qh, 1..=1, ()).ok()?,
            shm: Shm::from(globals.bind::<wl_shm::WlShm, _, _>(qh, 1..=1, ()).ok()?),
            listed: Vec::new(),
            pending: Vec::new(),
        })
    }
}

impl Runtime {
    pub(crate) fn capture_window(&mut self, request: CaptureWindow) {
        let qh = self.qh.clone();
        let Some(shell) = self.extension_mut::<ShellState>() else {
            return (request.done)(None);
        };
        let handle = {
            let entries = shell.toplevels.entries.borrow();
            let Some(entry) = entries.iter().find(|entry| entry.info.id == request.window) else {
                return (request.done)(None);
            };
            entry.list.clone().or_else(|| {
                shell.capture.as_ref().and_then(|capture| {
                    capture
                        .listed
                        .iter()
                        .find(|listed| {
                            listed.app_id == entry.info.app_id && listed.title == entry.info.title
                        })
                        .map(|listed| listed.handle.clone())
                })
            })
        };
        let (Some(handle), Some(capture)) = (handle, shell.capture.as_mut()) else {
            return (request.done)(None);
        };
        let source = capture.sources.create_source(&handle, &qh, ());
        let session = capture
            .copy
            .create_session(&source, Options::empty(), &qh, ());
        capture.pending.push(Pending {
            session,
            source,
            size: None,
            format: None,
            frame: None,
            done: request.done,
        });
    }

    fn finish_capture(&mut self, index: usize, image: Option<WindowImage>) {
        let Some(capture) = self
            .extension_mut::<ShellState>()
            .and_then(|shell| shell.capture.as_mut())
        else {
            return;
        };
        let pending = capture.pending.remove(index);
        if let Some((frame, _, buffer)) = &pending.frame {
            frame.destroy();
            buffer.destroy();
        }
        pending.session.destroy();
        pending.source.destroy();
        (pending.done)(image);
        self.request_redraw_all = true;
        self.ping.ping();
    }

    fn capture_index(&self, matches: impl Fn(&Pending) -> bool) -> Option<usize> {
        self.extension::<ShellState>()?
            .capture
            .as_ref()?
            .pending
            .iter()
            .position(matches)
    }
}

// ext-image-copy-capture-v1: the session sends the buffer constraints, then `done`; the
// client then attaches a buffer to a frame and asks for it.
fn start_frame(capture: &mut Capture, index: usize, qh: &QueueHandle<Runtime>) -> bool {
    let pending = &mut capture.pending[index];
    let (Some((width, height)), Some(format)) = (pending.size, pending.format) else {
        return false;
    };
    let stride = width as i32 * 4;
    let Ok(mut pool) = RawPool::new((stride * height as i32) as usize, &capture.shm) else {
        return false;
    };
    let buffer = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride,
        format,
        CaptureBuffer,
        qh,
    );
    let frame = pending.session.create_frame(qh, ());
    frame.attach_buffer(&buffer);
    frame.damage_buffer(0, 0, width as i32, height as i32);
    frame.capture();
    pending.frame = Some((frame, pool, buffer));
    true
}

fn read_image(pending: &mut Pending) -> Option<WindowImage> {
    let (width, height) = pending.size?;
    let opaque = pending.format? == wl_shm::Format::Xrgb8888;
    let (_, pool, _) = pending.frame.as_mut()?;
    let bytes = &pool.mmap()[..(width * height * 4) as usize];
    // wl_shm ARGB8888 is little-endian: B, G, R, A in memory.
    let rgba = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|pixel| {
            [
                pixel[2],
                pixel[1],
                pixel[0],
                if opaque { 255 } else { pixel[3] },
            ]
        })
        .collect();
    Some(WindowImage {
        width,
        height,
        rgba,
    })
}

impl Dispatch<ExtImageCopyCaptureSessionV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        session: &ExtImageCopyCaptureSessionV1,
        event: SessionEvent,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let Some(index) = state.capture_index(|pending| pending.session == *session) else {
            return;
        };
        let started = {
            let Some(capture) = state
                .extension_mut::<ShellState>()
                .and_then(|shell| shell.capture.as_mut())
            else {
                return;
            };
            match event {
                SessionEvent::BufferSize { width, height } => {
                    capture.pending[index].size = Some((width, height));
                    return;
                }
                SessionEvent::ShmFormat {
                    format: WEnum::Value(format),
                } => {
                    let pending = &mut capture.pending[index];
                    if pending.format.is_none()
                        && matches!(format, wl_shm::Format::Argb8888 | wl_shm::Format::Xrgb8888)
                    {
                        pending.format = Some(format);
                    }
                    return;
                }
                SessionEvent::Done if capture.pending[index].frame.is_none() => {
                    start_frame(capture, index, qh)
                }
                SessionEvent::Stopped => false,
                _ => return,
            }
        };
        if !started {
            state.finish_capture(index, None);
        }
    }
}

impl Dispatch<ExtImageCopyCaptureFrameV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        frame: &ExtImageCopyCaptureFrameV1,
        event: FrameEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(index) = state.capture_index(|pending| {
            pending
                .frame
                .as_ref()
                .is_some_and(|(known, _, _)| known == frame)
        }) else {
            return;
        };
        let image = match event {
            FrameEvent::Ready => state
                .extension_mut::<ShellState>()
                .and_then(|shell| shell.capture.as_mut())
                .and_then(|capture| read_image(&mut capture.pending[index])),
            FrameEvent::Failed { .. } => None,
            _ => return,
        };
        state.finish_capture(index, image);
    }
}

impl Dispatch<ExtForeignToplevelListV1, Listing, Runtime> for Runtime {
    fn event(
        state: &mut Self,
        _: &ExtForeignToplevelListV1,
        event: ListEvent,
        _: &Listing,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ListEvent::Toplevel { toplevel } = event
            && let Some(capture) = state
                .extension_mut::<ShellState>()
                .and_then(|shell| shell.capture.as_mut())
        {
            capture.listed.push(Listed {
                handle: toplevel,
                title: String::new(),
                app_id: String::new(),
            });
        }
    }

    event_created_child!(Runtime, ExtForeignToplevelListV1, [
        ext_foreign_toplevel_list_v1::EVT_TOPLEVEL_OPCODE => (ExtForeignToplevelHandleV1, Listing)
    ]);
}

impl Dispatch<ExtForeignToplevelHandleV1, Listing, Runtime> for Runtime {
    fn event(
        state: &mut Self,
        handle: &ExtForeignToplevelHandleV1,
        event: HandleEvent,
        _: &Listing,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(capture) = state
            .extension_mut::<ShellState>()
            .and_then(|shell| shell.capture.as_mut())
        else {
            return;
        };
        let Some(index) = capture
            .listed
            .iter()
            .position(|listed| listed.handle == *handle)
        else {
            return;
        };
        match event {
            HandleEvent::Title { title } => capture.listed[index].title = title,
            HandleEvent::AppId { app_id } => capture.listed[index].app_id = app_id,
            HandleEvent::Closed => {
                capture.listed.remove(index);
                handle.destroy();
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtForeignToplevelImageCaptureSourceManagerV1, (), Runtime> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtForeignToplevelImageCaptureSourceManagerV1,
        _: <ExtForeignToplevelImageCaptureSourceManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtImageCaptureSourceV1, (), Runtime> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtImageCaptureSourceV1,
        _: <ExtImageCaptureSourceV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtImageCopyCaptureManagerV1, (), Runtime> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtImageCopyCaptureManagerV1,
        _: <ExtImageCopyCaptureManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, (), Runtime> for Runtime {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, CaptureBuffer, Runtime> for Runtime {
    fn event(
        _: &mut Self,
        _: &WlBuffer,
        _: <WlBuffer as Proxy>::Event,
        _: &CaptureBuffer,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
