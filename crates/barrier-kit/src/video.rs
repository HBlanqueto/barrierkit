//! Video for the kit's own apps. GStreamer decodes on the GPU through VA-API and hands
//! each frame over as a dmabuf that Skia samples in place; without dmabuf import the
//! frame is downloaded once, and without VA-API the CPU decodes. Sources are local files
//! or `http(s)` URIs streamed by `souphttpsrc`. Sound plays in the same pipeline, on its
//! clock, and every stream starts muted until `set_volume`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::os::fd::BorrowedFd;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    AlphaType, ColorType, Data, FilterMode, Image as SkImage, ImageInfo, MipmapMode, Paint, RRect,
    Rect as SkRect, SamplingOptions,
};
use gstreamer as gst;
use gstreamer::prelude::{ElementExt as _, *};
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use skia_safe::canvas::SrcRectConstraint;
use thiserror::Error;
use torin::prelude::Size;

use crate::platform::dmabuf::{self, Imported, Plane};
use crate::platform::{Timers, WakeHandle};

const GPU_CAPS: &str = "video/x-raw(memory:DMABuf),format=DMA_DRM,drm-format=AB24";
const PREROLL_TIMEOUT: Duration = Duration::from_secs(3);
const REMOTE_PREROLL_TIMEOUT: Duration = Duration::from_secs(15);
const IMPORT_CACHE: usize = 16;
const AUDIO: &str = "queue ! decodebin3 ! audioconvert ! audioresample ! volume name=volume volume=0 ! autoaudiosink";

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("GStreamer could not start: {0}")]
    Init(String),
    #[error("no decoder could open {0}")]
    Open(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decoding {
    GpuZeroCopy,
    GpuDownload,
    Cpu,
}

thread_local! {
    static REDRAW: RefCell<Option<WakeHandle>> = const { RefCell::new(None) };
}

fn redraw_handle() -> Option<WakeHandle> {
    REDRAW.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            freya_core::current_context::CurrentContext::try_with(|_| ())?;
            let timers = try_consume_root_context::<Timers>()?;
            let platform = Platform::get();
            *slot = Some(timers.wakeup(move || platform.send(UserEvent::RequestRedraw)));
        }
        slot.clone()
    })
}

struct Shared {
    pipeline: gst::Pipeline,
    latest: Mutex<Option<gst::Sample>>,
    serial: AtomicU64,
    decoding: Decoding,
}

impl Drop for Shared {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

#[derive(Clone)]
pub struct VideoStream(Arc<Shared>);

impl PartialEq for VideoStream {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

fn gpu_decoders_present() -> bool {
    ["qtdemux", "h264parse", "vah264dec", "vapostproc"]
        .iter()
        .all(|name| gst::ElementFactory::find(name).is_some())
}

fn description(source: &str, decoding: Decoding) -> String {
    match decoding {
        Decoding::GpuZeroCopy => format!(
            "{source} ! qtdemux name=demux ! h264parse ! vah264dec ! vapostproc ! {GPU_CAPS} ! appsink name=sink"
        ),
        // VA drivers disagree on the byte order of RGB32 (gstvavideoformat.c, gstreamer#3354): convert on the CPU.
        Decoding::GpuDownload => format!(
            "{source} ! qtdemux name=demux ! h264parse ! vah264dec ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink"
        ),
        Decoding::Cpu => format!(
            "{source} ! decodebin3 name=demux ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink"
        ),
    }
}

fn candidates() -> Vec<Decoding> {
    let zero_copy = dmabuf::current().is_some_and(|graphics| graphics.dmabuf.is_some());
    match (gpu_decoders_present(), zero_copy) {
        (true, true) => vec![Decoding::GpuZeroCopy, Decoding::GpuDownload, Decoding::Cpu],
        (true, false) => vec![Decoding::GpuDownload, Decoding::Cpu],
        _ => vec![Decoding::Cpu],
    }
}

fn attach_audio(pipeline: &gst::Pipeline, pad: &gst::Pad) {
    // qtdemux and decodebin3 name their pads by kind; decodebin3 exposes them before their caps are set.
    if !pad.name().starts_with("audio_") || pipeline.by_name("volume").is_some() {
        return;
    }
    let Ok(branch) = gst::parse::bin_from_description(AUDIO, true) else {
        tracing::warn!("vídeo: no se pudo crear la rama de audio");
        return;
    };
    let linked = pipeline.add(&branch).is_ok()
        && branch.sync_state_with_parent().is_ok()
        && branch
            .static_pad("sink")
            .is_some_and(|sink| pad.link(&sink).is_ok());
    if !linked {
        tracing::warn!("vídeo: la pista de audio no se pudo enlazar");
    }
}

fn quoted(location: &str) -> String {
    format!("\"{}\"", location.replace('"', "\\\""))
}

pub fn fetch(uri: &str) -> Result<Vec<u8>, VideoError> {
    gst::init().map_err(|error| VideoError::Init(error.to_string()))?;
    let failed = || VideoError::Open(uri.to_owned());
    let pipeline = gst::parse::launch(&format!(
        "souphttpsrc location={} ! appsink name=sink sync=false",
        quoted(uri)
    ))
    .map_err(|_| failed())?
    .downcast::<gst::Pipeline>()
    .map_err(|_| failed())?;
    let sink = pipeline
        .by_name("sink")
        .and_then(|sink| sink.downcast::<gst_app::AppSink>().ok())
        .ok_or_else(failed)?;
    pipeline
        .set_state(gst::State::Playing)
        .map_err(|_| failed())?;
    let mut bytes = Vec::new();
    while let Ok(sample) = sink.pull_sample() {
        if let Some(map) = sample
            .buffer()
            .and_then(|buffer| buffer.map_readable().ok())
        {
            bytes.extend_from_slice(&map);
        }
    }
    let _ = pipeline.set_state(gst::State::Null);
    if bytes.is_empty() {
        Err(failed())
    } else {
        Ok(bytes)
    }
}

impl VideoStream {
    pub fn open(path: impl AsRef<Path>, looping: bool) -> Result<Self, VideoError> {
        let location = quoted(&path.as_ref().to_string_lossy());
        Self::open_source(
            &format!("filesrc location={location}"),
            looping,
            PREROLL_TIMEOUT,
        )
    }

    pub fn open_uri(uri: &str, looping: bool) -> Result<Self, VideoError> {
        Self::open_source(
            &format!("souphttpsrc location={}", quoted(uri)),
            looping,
            REMOTE_PREROLL_TIMEOUT,
        )
    }

    pub fn open_uris(video: &str, audio: &str, looping: bool) -> Result<Self, VideoError> {
        Self::open_source(
            &format!(
                "souphttpsrc location={} ! {AUDIO}  souphttpsrc location={}",
                quoted(audio),
                quoted(video)
            ),
            looping,
            REMOTE_PREROLL_TIMEOUT,
        )
    }

    fn open_source(
        source: &str,
        looping: bool,
        preroll_timeout: Duration,
    ) -> Result<Self, VideoError> {
        gst::init().map_err(|error| VideoError::Init(error.to_string()))?;
        let wake = redraw_handle();
        for decoding in candidates() {
            match Self::start(source, decoding, looping, preroll_timeout, wake.clone()) {
                Some(stream) => {
                    tracing::info!(?decoding, source, "vídeo abierto");
                    return Ok(stream);
                }
                None => tracing::info!(?decoding, source, "vídeo: decodificador descartado"),
            }
        }
        Err(VideoError::Open(source.to_owned()))
    }

    fn start(
        source: &str,
        decoding: Decoding,
        looping: bool,
        preroll_timeout: Duration,
        wake: Option<WakeHandle>,
    ) -> Option<Self> {
        let pipeline = gst::parse::launch(&description(source, decoding))
            .ok()?
            .downcast::<gst::Pipeline>()
            .ok()?;
        if let Some(demux) = pipeline.by_name("demux") {
            let weak = pipeline.downgrade();
            demux.connect_pad_added(move |_, pad| {
                if let Some(pipeline) = weak.upgrade() {
                    attach_audio(&pipeline, pad);
                }
            });
        }
        let sink = pipeline
            .by_name("sink")?
            .downcast::<gst_app::AppSink>()
            .ok()?;
        sink.set_max_buffers(2);
        sink.set_drop(true);
        let shared = Arc::new(Shared {
            pipeline: pipeline.clone(),
            latest: Mutex::new(None),
            serial: AtomicU64::new(0),
            decoding,
        });
        let store = {
            let shared = Arc::downgrade(&shared);
            move |sample: gst::Sample| {
                if let Some(shared) = shared.upgrade() {
                    if let Ok(mut latest) = shared.latest.lock() {
                        *latest = Some(sample);
                    }
                    shared.serial.fetch_add(1, Ordering::Release);
                    if let Some(wake) = &wake {
                        wake.wake();
                    }
                }
            }
        };
        let preroll = store.clone();
        sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    store(sink.pull_sample().map_err(|_| gst::FlowError::Eos)?);
                    Ok(gst::FlowSuccess::Ok)
                })
                .new_preroll(move |sink| {
                    preroll(sink.pull_preroll().map_err(|_| gst::FlowError::Eos)?);
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        let weak = pipeline.downgrade();
        pipeline.bus()?.set_sync_handler(move |_, message| {
            match message.view() {
                gst::MessageView::Eos(_) if looping => {
                    if let Some(pipeline) = weak.upgrade() {
                        // EOS arrives on a streaming thread; seeking from it would deadlock.
                        pipeline.call_async(|pipeline| {
                            let _ = pipeline.seek_simple(
                                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                gst::ClockTime::ZERO,
                            );
                        });
                    }
                }
                gst::MessageView::Error(error) => {
                    tracing::warn!(error = %error.error(), "vídeo: error de GStreamer");
                }
                _ => {}
            }
            gst::BusSyncReply::Drop
        });
        pipeline.set_state(gst::State::Paused).ok()?;
        let (result, _, _) = pipeline.state(gst::ClockTime::from_nseconds(
            preroll_timeout.as_nanos() as u64,
        ));
        if result.is_err() || shared.latest.lock().ok()?.is_none() {
            let _ = pipeline.set_state(gst::State::Null);
            return None;
        }
        Some(Self(shared))
    }

    pub fn set_volume(&self, level: f64) {
        if let Some(volume) = self.0.pipeline.by_name("volume") {
            // Cubic, like GstStreamVolume's CUBIC format: a linear slider then sounds even.
            volume.set_property("volume", level.clamp(0.0, 1.0).powi(3));
        }
    }

    pub fn decoding(&self) -> Decoding {
        self.0.decoding
    }

    pub fn play(&self) {
        let _ = self.0.pipeline.set_state(gst::State::Playing);
    }

    pub fn pause(&self) {
        let _ = self.0.pipeline.set_state(gst::State::Paused);
    }

    pub fn playing(&self) -> bool {
        self.0.pipeline.current_state() == gst::State::Playing
    }

    pub fn seek(&self, seconds: f64) {
        let _ = self.0.pipeline.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
            gst::ClockTime::from_nseconds((seconds.max(0.0) * 1e9) as u64),
        );
    }

    pub fn position(&self) -> f64 {
        self.0
            .pipeline
            .query_position::<gst::ClockTime>()
            .map_or(0.0, |time| time.nseconds() as f64 / 1e9)
    }

    pub fn duration(&self) -> f64 {
        self.0
            .pipeline
            .query_duration::<gst::ClockTime>()
            .map_or(0.0, |time| time.nseconds() as f64 / 1e9)
    }

    fn latest(&self) -> (u64, Option<gst::Sample>) {
        let serial = self.0.serial.load(Ordering::Acquire);
        let sample = self.0.latest.lock().ok().and_then(|latest| latest.clone());
        (serial, sample)
    }
}

type BufferKey = (u64, u64);

#[derive(Default)]
struct Presenter {
    serial: Option<u64>,
    shown: Option<gst::Sample>,
    previous: Option<gst::Sample>,
    image: Option<SkImage>,
    imported: HashMap<BufferKey, (Imported, Option<SkImage>)>,
}

fn buffer_key(fd: BorrowedFd<'_>) -> Option<BufferKey> {
    let metadata = std::fs::File::from(fd.try_clone_to_owned().ok()?)
        .metadata()
        .ok()?;
    Some((metadata.dev(), metadata.ino()))
}

impl Presenter {
    fn refresh(&mut self, stream: &VideoStream, canvas: &freya_engine::prelude::Canvas) {
        let (serial, sample) = stream.latest();
        if self.serial == Some(serial) {
            return;
        }
        self.serial = Some(serial);
        let Some(sample) = sample else {
            return;
        };
        self.image = match stream.decoding() {
            Decoding::GpuZeroCopy => self.zero_copy(&sample, canvas),
            Decoding::GpuDownload | Decoding::Cpu => upload(&sample),
        };
        // The decoder recycles a buffer once released; keeping two lets the GPU finish the last frame.
        self.previous = self.shown.replace(sample);
    }

    fn zero_copy(
        &mut self,
        sample: &gst::Sample,
        canvas: &freya_engine::prelude::Canvas,
    ) -> Option<SkImage> {
        let info = gst_video::VideoInfoDmaDrm::from_caps(sample.caps()?).ok()?;
        let buffer = sample.buffer()?;
        let memory = buffer.peek_memory(0);
        let fd = memory
            .downcast_memory_ref::<gstreamer_allocators::DmaBufMemory>()?
            .fd();
        // SAFETY: the descriptor belongs to `memory`, which `sample` keeps alive for this call.
        let fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let key = buffer_key(fd)?;
        if !self.imported.contains_key(&key) {
            if self.imported.len() >= IMPORT_CACHE {
                self.imported.clear();
            }
            let (offsets, strides) = match buffer.meta::<gst_video::VideoMeta>() {
                Some(meta) => (meta.offset().to_vec(), meta.stride().to_vec()),
                None => (info.offset().to_vec(), info.stride().to_vec()),
            };
            let planes: Vec<Plane> = offsets
                .iter()
                .zip(strides)
                .map(|(offset, stride)| Plane {
                    offset: (*offset + memory.offset()) as u64,
                    stride: stride as u64,
                })
                .collect();
            let imported = dmabuf::current()?.import_dmabuf(
                fd,
                (info.width(), info.height()),
                info.fourcc(),
                info.modifier(),
                &planes,
            )?;
            self.imported.insert(key, (imported, None));
        }
        let (imported, image) = self.imported.get_mut(&key)?;
        if image.is_none() {
            let mut context = canvas.recording_context()?;
            *image = skia_safe::gpu::images::borrow_texture_from(
                &mut context,
                &imported.texture,
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                // VA drivers may leave alpha at zero; video is opaque whatever the buffer says.
                ColorType::RGB888x,
                AlphaType::Opaque,
                None,
            );
        }
        image.clone()
    }
}

fn upload(sample: &gst::Sample) -> Option<SkImage> {
    let info = gst_video::VideoInfo::from_caps(sample.caps()?).ok()?;
    let map = sample.buffer()?.map_readable().ok()?;
    let stride = *info.stride().first()? as usize;
    skia_safe::images::raster_from_data(
        &ImageInfo::new(
            (info.width() as i32, info.height() as i32),
            ColorType::RGB888x,
            AlphaType::Opaque,
            None,
        ),
        Data::new_copy(map.as_slice()),
        stride,
    )
}

fn cover(image: (f32, f32), area: SkRect) -> SkRect {
    let scale = (area.width() / image.0).max(area.height() / image.1);
    let (width, height) = (area.width() / scale, area.height() / scale);
    SkRect::from_xywh(
        (image.0 - width) / 2.0,
        (image.1 - height) / 2.0,
        width,
        height,
    )
}

struct VideoElement {
    stream: VideoStream,
    presenter: Rc<RefCell<Presenter>>,
    corner_radius: f32,
    layout: LayoutData,
}

impl PartialEq for VideoElement {
    fn eq(&self, other: &Self) -> bool {
        self.stream == other.stream
            && self.corner_radius == other.corner_radius
            && self.layout == other.layout
    }
}

impl ElementExt for VideoElement {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        (other.as_ref() as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_none_or(|other| other != self)
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        match (other.as_ref() as &dyn std::any::Any).downcast_ref::<Self>() {
            Some(other) if other == self => DiffModifies::empty(),
            Some(_) => DiffModifies::STYLE,
            None => DiffModifies::all(),
        }
    }

    fn layout(&'_ self) -> std::borrow::Cow<'_, LayoutData> {
        std::borrow::Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let mut presenter = self.presenter.borrow_mut();
        presenter.refresh(&self.stream, context.canvas);
        let Some(image) = presenter.image.clone() else {
            return;
        };
        let source = cover((image.width() as f32, image.height() as f32), bounds);
        let canvas = context.canvas;
        canvas.save();
        if self.corner_radius > 0.0 {
            canvas.clip_rrect(
                RRect::new_rect_xy(bounds, self.corner_radius, self.corner_radius),
                None,
                Some(true),
            );
        }
        canvas.draw_image_rect_with_sampling_options(
            &image,
            Some((&source, SrcRectConstraint::Fast)),
            bounds,
            SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
            &Paint::default(),
        );
        canvas.restore();
    }
}

#[derive(PartialEq)]
pub struct VideoView {
    stream: VideoStream,
    width: Size,
    height: Size,
    corner_radius: f32,
}

impl VideoView {
    pub fn new(stream: VideoStream) -> Self {
        Self {
            stream,
            width: Size::fill(),
            height: Size::fill(),
            corner_radius: 0.0,
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;
        self
    }

    pub fn corner_radius(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }
}

impl Component for VideoView {
    fn render(&self) -> impl IntoElement {
        let presenter = use_hook(|| Rc::new(RefCell::new(Presenter::default())));
        let layout = torin::node::Node {
            width: self.width.clone(),
            height: self.height.clone(),
            ..Default::default()
        };
        Element::Element {
            key: DiffKey::None,
            element: Rc::new(VideoElement {
                stream: self.stream.clone(),
                presenter,
                corner_radius: self.corner_radius,
                layout: layout.into(),
            }),
            elements: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_frame_in_a_tall_box_is_cropped_at_the_sides() {
        let source = cover((1920.0, 1080.0), SkRect::from_wh(100.0, 200.0));
        assert_eq!(source.height(), 1080.0);
        assert!((source.width() - 540.0).abs() < 0.01);
        assert!((source.left - 690.0).abs() < 0.01);
    }
}
