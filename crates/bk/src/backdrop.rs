use std::borrow::Cow;
use std::f32::consts::FRAC_1_SQRT_2;
use std::rc::Rc;

use ctappearance::backdrop::{self as tint, ImagePalette, ink_on, palette_from_pixels};
use ctdesign::{BACKDROP, Rgba, smoothstep};
use freya_animation::prelude::use_animation_transition;
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    AlphaType, Color4f, ColorType, Colors, Data, Gradient, Image, ImageInfo, Interpolation, Paint,
    PaintStyle, Rect as SkRect, TileMode, raster_n32_premul, shaders,
};
use torin::prelude::{Position, Size};

use crate::animation::Clock;
use crate::elements::{Overrides, forward_overrides};
use crate::style::Palette;
use crate::styles::{BackdropStyle, Theme, resolve};
use crate::theme::color;

#[derive(Clone, Debug)]
pub enum BackdropSource {
    None,
    Colors([Rgba; 3]),
    Image(Bytes),
}

impl PartialEq for BackdropSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Colors(a), Self::Colors(b)) => a == b,
            (Self::Image(a), Self::Image(b)) => a.as_ptr() == b.as_ptr() && a.len() == b.len(),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Accent {
    #[default]
    System,
    Own(Rgba),
    FromBackdrop,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Tint {
    #[default]
    Accent,
    Color(Rgba),
    Backdrop,
}

pub(crate) fn tinted() -> Option<tint::Backdrop> {
    try_consume_context::<State<Option<tint::Backdrop>>>().and_then(|state| *state.read())
}

pub(crate) fn highlights(backdrop: tint::Backdrop, ink: impl Fn(Rgba) -> Color) -> LinearGradient {
    backdrop
        .highlights()
        .iter()
        .enumerate()
        .fold(LinearGradient::new().angle(90.0), |gradient, (i, c)| {
            gradient.stop((ink(*c), i as f32 * 50.0))
        })
}

#[derive(PartialEq)]
pub struct Backdrop {
    source: BackdropSource,
    accent: Accent,
    child: Element,
    style: Theme,
    overrides: Overrides,
}

impl Backdrop {
    pub fn new(source: BackdropSource, child: impl Into<Element>) -> Self {
        Self {
            source,
            accent: Accent::System,
            child: child.into(),
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(BackdropStyle) -> BackdropStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = accent;
        self
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Look {
    tint: Option<tint::Backdrop>,
    vivid: Option<Rgba>,
}

impl Look {
    fn of(palette: Option<ImagePalette>) -> Self {
        Self {
            tint: palette.map(|p| tint::Backdrop::from_colors(p.colors)),
            vivid: palette.map(|p| p.vivid),
        }
    }
}

fn use_look(source: &BackdropSource) -> Look {
    let mut look = use_state(|| Look::of(None));
    let mut seen = use_state(|| None::<BackdropSource>);
    if seen.peek().as_ref() != Some(source) {
        seen.set(Some(source.clone()));
        match source {
            BackdropSource::None => look.set_if_modified(Look::of(None)),
            BackdropSource::Colors(colors) => look.set_if_modified(Look::of(Some(ImagePalette {
                colors: *colors,
                vivid: colors[0],
            }))),
            BackdropSource::Image(bytes) => {
                let (sender, receiver) = futures_channel::oneshot::channel();
                let bytes = bytes.clone();
                crate::media::decode_in_background(move || {
                    let _ = sender.send(Look::of(image_palette(&bytes)));
                });
                let source = source.clone();
                spawn(async move {
                    if let Ok(decoded) = receiver.await
                        && seen.peek().as_ref() == Some(&source)
                    {
                        look.set_if_modified(decoded);
                    }
                });
            }
        }
    }
    *look.read()
}

fn image_palette(bytes: &Bytes) -> Option<ImagePalette> {
    let image = Image::from_encoded(Data::new_copy(bytes))?;
    let side = BACKDROP.samples as i32;
    let mut surface = raster_n32_premul((side, side))?;
    surface.canvas().draw_image_rect(
        &image,
        None,
        SkRect::from_wh(side as f32, side as f32),
        &Paint::default(),
    );
    let info = ImageInfo::new((side, side), ColorType::RGBA8888, AlphaType::Unpremul, None);
    let mut pixels = vec![0u8; (side * side * 4) as usize];
    surface
        .read_pixels(&info, &mut pixels, side as usize * 4, (0, 0))
        .then_some(())?;
    palette_from_pixels(&pixels)
}

forward_overrides!(Backdrop);

impl Component for Backdrop {
    fn render(&self) -> impl IntoElement {
        let style: BackdropStyle = resolve(&self.style);
        let animation = style.animation;
        let resolved = crate::theme::resolved();
        let system = Palette::system();
        let look = use_look(&self.source);

        let target = look
            .tint
            .unwrap_or_else(|| tint::Backdrop::solid(resolved.surfaces.window));
        let mut from = use_state(|| target);
        let mut to = use_state(|| target);
        let duration_ms = use_reactive(&animation.duration_ms());
        let progress = use_animation_transition(to, move |_: tint::Backdrop, _| {
            Clock::new(0.0, 1.0).time(*duration_ms.peek())
        });
        let shown = from
            .peek()
            .mix(*to.peek(), animation.progress(progress.get().value()));
        if *to.peek() != target {
            from.set(shown);
            to.set(target);
        }

        let accent = match (self.accent, look.tint, look.vivid) {
            _ if !resolved.automatic_accent => resolved.accent,
            (Accent::Own(own), _, _) => own,
            (Accent::FromBackdrop, Some(backdrop), Some(vivid)) => backdrop.accent(vivid),
            _ => resolved.accent,
        };
        let palette = match look.tint {
            Some(backdrop) => Palette::over(backdrop, accent, system),
            None => Palette {
                accent,
                on_accent: ink_on(accent),
                ..system
            },
        };
        let mut shared = use_state(|| palette);
        shared.set_if_modified(palette);
        let mut tinted = use_state(|| look.tint);
        tinted.set_if_modified(look.tint);
        use_hook(|| {
            provide_context(shared);
            provide_context(tinted);
        });

        let root = rect()
            .expanded()
            .child(layer(shown, style.angles, style.opacity))
            .child(self.child.clone());
        self.overrides.apply(root)
    }
}

impl Palette {
    pub(crate) fn over(backdrop: tint::Backdrop, accent: Rgba, system: Palette) -> Self {
        let ink = backdrop.ink();
        let alpha = |a: f32| Rgba { a, ..ink };
        let sunken = Rgba {
            a: BACKDROP.sunken,
            ..ink_on(ink)
        };
        Self {
            label: ink,
            secondary_label: alpha(BACKDROP.ink_secondary),
            tertiary_label: alpha(BACKDROP.ink_tertiary),
            separator: alpha(BACKDROP.glass),
            outline: alpha(BACKDROP.glass),
            control_border: alpha(BACKDROP.ink_tertiary),
            window_background: alpha(0.0),
            content_background: sunken,
            control_background: alpha(BACKDROP.glass),
            system_fill: alpha(BACKDROP.glass),
            control_background_disabled: alpha(BACKDROP.glass / 2.0),
            text_background: sunken,
            control_hover: alpha(BACKDROP.glass_strong),
            accent,
            toggle_accent: accent,
            selected_content: accent,
            selected_content_inactive: backdrop.base.mix(ink, BACKDROP.ink_tertiary),
            combo_button: alpha(BACKDROP.glass),
            segment_divider: alpha(BACKDROP.ink_tertiary),
            ring_accent: accent,
            toggle_accent_disabled: accent,
            on_selected_content: ink_on(accent),
            separator_strong: alpha(BACKDROP.glass_strong),
            accent_inactive: backdrop.base.mix(ink, BACKDROP.ink_tertiary),
            on_accent: ink_on(accent),
            destructive: system.destructive,
            raised_background: backdrop.base,
            sunken_background: sunken,
            scrim: system.scrim,
            selected_control: alpha(BACKDROP.glass_strong),
            card_border: alpha(0.0),
        }
    }
}

#[derive(PartialEq)]
struct Layer {
    backdrop: tint::Backdrop,
    angles: [f32; 3],
    opacity: f32,
    layout: LayoutData,
}

fn layer(backdrop: tint::Backdrop, angles: [f32; 3], opacity: f32) -> Element {
    crate::glass::leaf(Layer {
        backdrop,
        angles,
        opacity,
        layout: crate::glass::layout(Size::fill(), Size::fill(), Position::new_absolute()),
    })
}

// CSS linear-gradient geometry: 0° points up, angles turn clockwise, and the
// line is long enough for the far corners to sit on its ends.
fn line(area: SkRect, degrees: f32) -> ((f32, f32), (f32, f32)) {
    let (sin, cos) = degrees.to_radians().sin_cos();
    let half = (area.width() * sin.abs() + area.height() * cos.abs()) / 2.0;
    let (x, y) = (area.center_x(), area.center_y());
    let (dx, dy) = (sin * half, -cos * half);
    ((x - dx, y - dy), (x + dx, y + dy))
}

impl ElementExt for Layer {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        crate::style_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let mut paint = Paint::default();
        paint.set_style(PaintStyle::Fill);
        paint.set_dither(true);
        paint.set_color(color(self.backdrop.base));
        context.canvas.draw_rect(bounds, &paint);

        let stops = BACKDROP.stops.max(2) as usize;
        for (degrees, layer) in self.angles.iter().zip(self.backdrop.layers) {
            let (colors, offsets): (Vec<Color4f>, Vec<f32>) = (0..stops)
                .map(|i| {
                    let t = i as f32 / (stops - 1) as f32;
                    let fade = 1.0 - smoothstep(t);
                    let c = Rgba {
                        a: self.opacity * fade,
                        ..layer
                    };
                    (Color4f::new(c.r, c.g, c.b, c.a), t * FRAC_1_SQRT_2)
                })
                .unzip();
            let gradient = Gradient::new(
                Colors::new(&colors, Some(&offsets), TileMode::Clamp, None),
                Interpolation::default(),
            );
            let (start, end) = line(bounds, *degrees);
            paint.set_shader(shaders::linear_gradient((start, end), &gradient, None));
            context.canvas.draw_rect(bounds, &paint);
        }
    }
}
