//! Pieces for content over photos and video: scrims that melt media into the window,
//! a blurred material, the light and the translucent round buttons, and artwork loading.

use std::path::PathBuf;

use freya_core::elements::image::ImageHandle;
use freya_core::prelude::*;
use torin::prelude::{Direction, Gaps, Size};
use vow_design::{MEDIA, Rgba, TEXT};

pub use crate::glass::{material, top_glass};
use crate::interaction::use_interaction;
use crate::style::Palette;
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrimEdge {
    Leading,
    Bottom,
}

pub fn ground() -> Rgba {
    crate::backdrop::tinted().map_or_else(
        || Palette::current().window_background,
        |backdrop| backdrop.average(),
    )
}

pub fn scrim(edge: ScrimEdge) -> Rect {
    let window = ground();
    // Freya measures gradient angles from top to bottom, turning clockwise: 270° runs left to right.
    let (angle, stops) = match edge {
        ScrimEdge::Leading => (270.0, MEDIA.scrim_side),
        ScrimEdge::Bottom => (0.0, MEDIA.scrim_bottom),
    };
    let gradient = stops.iter().fold(
        LinearGradient::new().angle(angle),
        |gradient, &(at, alpha)| gradient.stop((color(Rgba { a: alpha, ..window }), at * 100.0)),
    );
    rect().background(gradient).interactive(Interactive::No)
}

#[derive(PartialEq)]
pub struct MediaButton {
    label: String,
    symbol: Option<Symbol>,
    on_press: Option<Callback<(), ()>>,
}

impl MediaButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            symbol: None,
            on_press: None,
        }
    }

    pub fn symbol(mut self, symbol: Symbol) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

impl Component for MediaButton {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let on_press = self.on_press.clone();
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .height(Size::px(MEDIA.button_height))
            .padding(Gaps::new(
                0.0,
                MEDIA.button_padding_x,
                0.0,
                MEDIA.button_padding_x,
            ))
            .corner_radius(MEDIA.button_radius)
            .direction(Direction::Horizontal)
            .spacing(MEDIA.button_font_size / 2.0)
            .center()
            .background(color(MEDIA.button_fill))
            .maybe(interaction.pressed(), |el| {
                el.scale(vow_design::BUTTON.pressed_scale)
            })
            .on_press(move |_| {
                if let Some(callback) = &on_press {
                    callback.call(());
                }
            })
            .maybe_child(
                self.symbol
                    .map(|icon| symbol(icon, MEDIA.button_font_size, MEDIA.button_ink)),
            )
            .child(
                label()
                    .text(self.label.clone())
                    .font_family(TEXT.family)
                    .font_size(MEDIA.button_font_size)
                    .font_weight(600)
                    .max_lines(1)
                    .color(color(MEDIA.button_ink)),
            );
        interaction.focus_ring(button, MEDIA.button_radius)
    }
}

#[derive(PartialEq)]
pub struct RoundButton {
    symbol: Symbol,
    label: &'static str,
    selected: bool,
    on_press: Option<Callback<(), ()>>,
}

impl RoundButton {
    pub fn new(symbol: Symbol, label: &'static str) -> Self {
        Self {
            symbol,
            label,
            selected: false,
            on_press: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

impl Component for RoundButton {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let on_press = self.on_press.clone();
        let (fill, ink) = if self.selected {
            (MEDIA.button_fill, MEDIA.button_ink)
        } else {
            (MEDIA.round_glass, MEDIA.button_fill)
        };
        let size = MEDIA.round;
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(size / 2.0)
            .center()
            .background(color(fill))
            .border(
                Border::new()
                    .fill(color(MEDIA.round_rim))
                    .width(vow_design::HAIRLINE)
                    .alignment(BorderAlignment::Inner),
            )
            .maybe(interaction.pressed(), |el| {
                el.scale(vow_design::BUTTON.pressed_scale)
            })
            .on_press(move |_| {
                if let Some(callback) = &on_press {
                    callback.call(());
                }
            })
            .child(symbol(self.symbol, MEDIA.round_icon, ink));
        interaction.focus_ring(button, size / 2.0)
    }
}

pub fn use_artwork(path: PathBuf) -> State<Option<ImageHandle>> {
    use_artwork_from(move || std::fs::read(&path).ok())
}

type Job = Box<dyn FnOnce() + Send>;
type Worker = std::sync::OnceLock<Option<std::sync::mpsc::Sender<Job>>>;

// A grid of covers would otherwise start one thread per cover at once.
fn run_on(worker: &'static Worker, name: &str, job: impl FnOnce() + Send + 'static) {
    let worker = worker.get_or_init(|| {
        let (jobs, queue) = std::sync::mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name(name.into())
            .spawn(move || queue.into_iter().for_each(|job| job()))
            .ok()
            .map(|_| jobs)
    });
    if let Some(worker) = worker {
        let _ = worker.send(Box::new(job));
    }
}

fn fetch_in_background(job: impl FnOnce() + Send + 'static) {
    static FETCH: Worker = Worker::new();
    run_on(&FETCH, "barrier-fetch", job);
}

pub(crate) fn decode_in_background(job: impl FnOnce() + Send + 'static) {
    static DECODE: Worker = Worker::new();
    run_on(&DECODE, "barrier-artwork", job);
}

pub fn use_artwork_from(
    load: impl FnOnce() -> Option<Vec<u8>> + Send + 'static,
) -> State<Option<ImageHandle>> {
    let mut artwork = use_state(|| None);
    use_hook(move || {
        let (sender, receiver) = futures_channel::oneshot::channel();
        fetch_in_background(move || {
            let Some(bytes) = load() else {
                return;
            };
            decode_in_background(move || {
                let decoded = freya_engine::prelude::Image::from_encoded(
                    freya_engine::prelude::Data::new_copy(&bytes),
                )
                .and_then(|image| image.make_raster_image(None, None))
                .map(|image| ImageHandle::new(image, Bytes::new()));
                let _ = sender.send(decoded);
            });
        });
        spawn(async move {
            if let Ok(Some(handle)) = receiver.await {
                artwork.set(Some(handle));
            }
        });
    });
    artwork
}
