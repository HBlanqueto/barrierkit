use crate::elements::forward_container;
use std::path::PathBuf;

use ctdesign::{MEDIA, Rgba};
use freya_core::elements::image::ImageHandle;
use freya_core::prelude::*;
use torin::prelude::{Direction, Gaps, Size};

use crate::animation::use_animated;
use crate::controls::{dimmed, inner_border, shadowed};
pub use crate::glass::{Material, TopGlass};
use crate::interaction::{control_state, use_interaction};
use crate::style::Palette;
use crate::styles::Painted;
use crate::styles::{LayoutDirection, MediaButtonStyle, Paints, RoundButtonStyle, Theme, resolve};
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

pub struct Scrim(Rect);

impl Scrim {
    pub fn new(edge: ScrimEdge) -> Self {
        Self(scrim(edge))
    }
}

forward_container!(Scrim);

fn scrim(edge: ScrimEdge) -> Rect {
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
    style: Theme,
}

impl MediaButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            symbol: None,
            on_press: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(MediaButtonStyle) -> MediaButtonStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: MediaButtonStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let scale = use_animated(
            if interaction.pressed() {
                style.pressed_scale
            } else {
                1.0
            },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let font = style.font;
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(radius)
            .direction(Direction::Horizontal)
            .spacing(style.icon_gap)
            .center()
            .painted(&paints, &style.background)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .maybe(scale != 1.0, |el| el.scale(scale))
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .maybe_child(self.symbol.map(|icon| {
                symbol(
                    icon,
                    style.icon_size,
                    paints.color(style.label.resolve(state)),
                )
            }))
            .child(
                label()
                    .text(self.label.clone())
                    .font_family(font.family)
                    .font_size(font.size as f32)
                    .font_weight(font.weight as i32)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            );
        interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), style.opacity),
            radius.top_left,
        )
    }
}

#[derive(PartialEq)]
pub struct RoundButton {
    symbol: Symbol,
    label: &'static str,
    selected: bool,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
}

impl RoundButton {
    pub fn new(symbol: Symbol, label: &'static str) -> Self {
        Self {
            symbol,
            label,
            selected: false,
            on_press: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(RoundButtonStyle) -> RoundButtonStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: RoundButtonStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let scale = use_animated(
            if interaction.pressed() {
                style.pressed_scale
            } else {
                1.0
            },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let (fill, ink) = if self.selected {
            (&style.selected, &style.selected_icon)
        } else {
            (&style.background, &style.icon)
        };
        let size = style.size;
        let radius = style
            .shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .center()
            .painted(&paints, fill)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .maybe(scale != 1.0, |el| el.scale(scale))
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(symbol(
                self.symbol,
                style.icon_size,
                paints.color(ink.resolve(state)),
            ));
        interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), style.opacity),
            radius.top_left,
        )
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
