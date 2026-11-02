//! Window background tinted from an image or chosen colours. Inside it the
//! controls read its own ink and accent, whatever the system colour scheme.

use std::borrow::Cow;
use std::f32::consts::FRAC_1_SQRT_2;
use std::rc::Rc;

use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    AlphaType, Color4f, ColorType, Colors, Data, Gradient, Image, ImageInfo, Interpolation, Paint,
    PaintStyle, Rect as SkRect, TileMode, raster_n32_premul, shaders,
};
use torin::prelude::{Position, Size};
use vow_appearance::backdrop::{self as tint, ImagePalette, ink_on, palette_from_pixels};
use vow_design::{BACKDROP, DURATION_SLOW_MS, Rgba};

use crate::style::Palette;
use crate::theme::color;

#[derive(Clone, Debug, PartialEq)]
pub enum BackdropSource {
    None,
    Colors([Rgba; 3]),
    Image(Bytes),
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

impl Tint {
    pub(crate) fn fill(self, palette: &Palette) -> Fill {
        if !crate::platform::window_activated() {
            return color(palette.accent).into();
        }
        match (self, tinted()) {
            (Self::Color(c), _) => color(c).into(),
            (Self::Backdrop, Some(backdrop)) => backdrop
                .highlights()
                .iter()
                .enumerate()
                .fold(LinearGradient::new().angle(90.0), |gradient, (i, c)| {
                    gradient.stop((color(*c), i as f32 * 50.0))
                })
                .into(),
            _ => color(palette.accent).into(),
        }
    }
}

#[derive(PartialEq)]
pub struct Backdrop {
    source: BackdropSource,
    accent: Accent,
    child: Element,
}

impl Backdrop {
    pub fn new(source: BackdropSource, child: impl Into<Element>) -> Self {
        Self {
            source,
            accent: Accent::System,
            child: child.into(),
        }
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
    fn of(source: &BackdropSource) -> Self {
        let palette = match source {
            BackdropSource::None => None,
            BackdropSource::Colors(colors) => Some(ImagePalette {
                colors: *colors,
                vivid: colors[0],
            }),
            BackdropSource::Image(bytes) => image_palette(bytes),
        };
        Self {
            tint: palette.map(|p| tint::Backdrop::from_colors(p.colors)),
            vivid: palette.map(|p| p.vivid),
        }
    }
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

impl Component for Backdrop {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let system = Palette::system();
        let mut seen = use_state(|| (self.source.clone(), Look::of(&self.source)));
        if seen.peek().0 != self.source {
            seen.set((self.source.clone(), Look::of(&self.source)));
        }
        let look = seen.read().1;

        let target = look
            .tint
            .unwrap_or_else(|| tint::Backdrop::solid(resolved.surfaces.window));
        let mut from = use_state(|| target);
        let mut to = use_state(|| target);
        let reduced_motion = resolved.reduced_motion;
        let progress = use_animation_transition(to, move |_: tint::Backdrop, _| {
            AnimNum::new(0.0, 1.0)
                .time(if reduced_motion { 0 } else { DURATION_SLOW_MS })
                .ease(Ease::InOut)
                .function(Function::Cubic)
        });
        let shown = from.peek().mix(*to.peek(), progress.get().value());
        if *to.peek() != target {
            from.set(shown);
            to.set(target);
        }

        let accent = match (self.accent, look.tint, look.vivid) {
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
        if *shared.peek() != palette {
            shared.set(palette);
        }
        let mut tinted = use_state(|| look.tint);
        if *tinted.peek() != look.tint {
            tinted.set(look.tint);
        }
        use_hook(|| {
            provide_context(shared);
            provide_context(tinted);
        });

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(layer(shown))
            .child(self.child.clone())
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
            control_border: alpha(BACKDROP.ink_tertiary),
            window_background: alpha(0.0),
            content_background: sunken,
            control_background: alpha(BACKDROP.glass),
            control_hover: alpha(BACKDROP.glass_strong),
            accent,
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
    layout: LayoutData,
}

fn layer(backdrop: tint::Backdrop) -> Element {
    let layout = torin::node::Node {
        width: Size::fill(),
        height: Size::fill(),
        position: Position::new_absolute(),
        ..Default::default()
    };
    Element::Element {
        key: DiffKey::None,
        element: Rc::new(Layer {
            backdrop,
            layout: layout.into(),
        }),
        elements: Vec::new(),
    }
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
        for (degrees, layer) in BACKDROP.angles.iter().zip(self.backdrop.layers) {
            let (colors, offsets): (Vec<Color4f>, Vec<f32>) = (0..stops)
                .map(|i| {
                    let t = i as f32 / (stops - 1) as f32;
                    let fade = 1.0 - t * t * (3.0 - 2.0 * t);
                    let c = Rgba {
                        a: BACKDROP.opacity * fade,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn png(blocks: &[[u8; 3]]) -> Bytes {
        let (width, height) = (32 * blocks.len() as u32, 32);
        let mut pixels = Vec::new();
        for _ in 0..height {
            for block in blocks {
                for _ in 0..32 {
                    pixels.extend_from_slice(block);
                }
            }
        }
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&pixels)
            .unwrap();
        Bytes::from(out)
    }

    #[test]
    fn an_encoded_image_gives_back_its_colours() {
        let blocks = [[200, 50, 60], [30, 130, 140], [220, 170, 40]];
        let palette = image_palette(&png(&blocks)).expect("decodable png");
        for block in blocks {
            let found = palette.colors.iter().any(|c| {
                let [r, g, b, _] = c.to_rgba8();
                [r, g, b].iter().zip(block).all(|(p, q)| p.abs_diff(q) <= 4)
            });
            assert!(found, "missing {block:?} in {:?}", palette.colors);
        }
    }
}
