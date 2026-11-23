use ctdesign::{
    ButtonKind, Rgba, Segment, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TRAFFIC_CLOSE_MINIMIZE_DARKEN,
    TRAFFIC_GLYPHS, TRAFFIC_PRESS, TRAFFIC_ZOOM_LIGHTEN, TrafficLook, TrafficLooks, Triangle,
};

use crate::TrafficLightStyle;

const MIN_GLYPH_CONTRAST: f32 = 3.0;

pub fn inset(bar_height: u32) -> f64 {
    f64::from(if bar_height == TITLEBAR_CLIENT_HEIGHT {
        TITLEBAR.buttons.inset_compact
    } else {
        TITLEBAR.buttons.inset
    })
}

pub fn button_rect(index: usize) -> (f64, f64, f64) {
    let diameter = f64::from(TITLEBAR.buttons.diameter);
    let gap = f64::from(TITLEBAR.buttons.spacing);
    let top = (f64::from(TITLEBAR.thickness) - diameter) / 2.0;
    (
        inset(TITLEBAR.thickness) + index as f64 * (diameter + gap),
        top,
        diameter,
    )
}

pub fn group_rect() -> (f64, f64, f64, f64) {
    let (x, y, size) = button_rect(0);
    let count = TITLEBAR.buttons.order.len() as f64;
    let gap = f64::from(TITLEBAR.buttons.spacing);
    (x, y, count * size + (count - 1.0) * gap, size)
}

pub fn hit(point: (f64, f64)) -> Option<ButtonKind> {
    let (px, py) = point;
    TITLEBAR
        .buttons
        .order
        .iter()
        .enumerate()
        .find_map(|(index, button)| {
            let (x, y, size) = button_rect(index);
            let half = size / 2.0;
            ((px - x - half).hypot(py - y - half) <= half).then_some(button.kind)
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrafficInput {
    pub focused: bool,
    pub group_hovered: bool,
    pub pressed: Option<ButtonKind>,
    pub zoom_supported: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Close,
    Minimize,
    ZoomEnter,
    ZoomExit,
    Plus,
}

impl Glyph {
    pub fn of(index: usize, fixed_width: bool, fullscreen: bool) -> Self {
        match (index, fixed_width, fullscreen) {
            (0, ..) => Self::Close,
            (1, ..) => Self::Minimize,
            (_, _, true) => Self::ZoomExit,
            (_, true, false) => Self::Plus,
            _ => Self::ZoomEnter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonVisual {
    pub fill: [Rgba; 4],
    pub ring: [Rgba; 3],
    pub glyph: Option<Rgba>,
    pub scale: f32,
    pub dodge: f32,
}

pub fn visual(looks: &TrafficLooks, input: TrafficInput) -> [ButtonVisual; 3] {
    let lights = [looks.close, looks.minimize, looks.zoom];
    std::array::from_fn(|index| {
        let kind = TITLEBAR.buttons.order[index].kind;
        let admitted = match kind {
            ButtonKind::Maximize => input.zoom_supported,
            ButtonKind::Minimize => !input.fullscreen,
            ButtonKind::Close => true,
        };
        let lit = admitted && (input.focused || input.group_hovered);
        let pressed = lit && input.pressed == Some(kind);
        let look = if lit { lights[index] } else { looks.inactive };
        ButtonVisual {
            fill: look.fill,
            ring: look.ring,
            glyph: (lit && input.group_hovered).then_some(look.glyph),
            scale: if pressed { TRAFFIC_PRESS.scale } else { 1.0 },
            dodge: if pressed { TRAFFIC_PRESS.dodge } else { 0.0 },
        }
    })
}

/// The three lights tinted for `style`: each keeps the measured structure of `standard` (alpha, the
/// lightness and saturation of every stop relative to its base) with the hue of its tone, and its glyph
/// stays at 3:1 against its fill.
pub fn looks(
    style: TrafficLightStyle,
    standard: &TrafficLooks,
    accent: Rgba,
    dominant: Option<Rgba>,
) -> TrafficLooks {
    let derived = |source: Rgba| {
        [
            source.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN),
            source.darken(TRAFFIC_CLOSE_MINIMIZE_DARKEN),
            source.lighten(TRAFFIC_ZOOM_LIGHTEN),
        ]
    };
    let tones = match style {
        TrafficLightStyle::Standard => return *standard,
        TrafficLightStyle::Accent => derived(accent),
        TrafficLightStyle::Wallpaper => derived(dominant.unwrap_or(accent)),
    };
    TrafficLooks {
        close: retint(standard.close, tones[0]),
        minimize: retint(standard.minimize, tones[1]),
        zoom: retint(standard.zoom, tones[2]),
        ..*standard
    }
}

fn retint(look: TrafficLook, tone: Rgba) -> TrafficLook {
    let (_, base_saturation, base_lightness) = look.base.to_hsl();
    let (hue, saturation, lightness) = tone.to_hsl();
    let shift = |color: Rgba| {
        let (_, s, l) = color.to_hsl();
        let lightness = if l <= base_lightness {
            lightness * l / base_lightness
        } else {
            lightness + (l - base_lightness) * (1.0 - lightness) / (1.0 - base_lightness)
        };
        let saturation = if base_saturation > 0.0 {
            (s * saturation / base_saturation).min(1.0)
        } else {
            saturation
        };
        Rgba::from_hsl(hue, saturation, lightness, color.a)
    };
    let fill = look.fill.map(shift);
    TrafficLook {
        base: tone,
        fill,
        ring: look.ring.map(shift),
        glyph: legible(shift(look.glyph), fill[1]),
    }
}

/// `ink` pushed toward black or white, whichever the `ground` contrasts with, until it reaches the 3:1 of WCAG 2.2 (1.4.11).
fn legible(ink: Rgba, ground: Rgba) -> Rgba {
    let ground = Rgba { a: 1.0, ..ground };
    let darker = ground.relative_luminance() > 0.18;
    let pushed = |step: usize| {
        let amount = step as f32 / 10.0;
        if darker {
            ink.darken(amount)
        } else {
            ink.lighten(amount)
        }
    };
    (0..=10)
        .map(pushed)
        .find(|candidate| candidate.contrast_ratio(ground) >= MIN_GLYPH_CONTRAST)
        .unwrap_or_else(|| pushed(10))
}

pub fn glyph_svg(glyph: Glyph, color: Rgba) -> String {
    let [r, g, b, a] = color.to_rgba8();
    let color = format!("#{r:02x}{g:02x}{b:02x}");
    let opacity = f32::from(a) / 255.0;
    let size = TITLEBAR.buttons.diameter;
    let body = match glyph {
        Glyph::Close => stroked(TRAFFIC_GLYPHS.close, "round", &color, opacity),
        Glyph::Minimize => stroked(TRAFFIC_GLYPHS.minimize, "butt", &color, opacity),
        Glyph::Plus => stroked(TRAFFIC_GLYPHS.plus, "butt", &color, opacity),
        Glyph::ZoomEnter | Glyph::ZoomExit => {
            let triangles = if glyph == Glyph::ZoomExit {
                TRAFFIC_GLYPHS.zoom_exit
            } else {
                TRAFFIC_GLYPHS.zoom
            };
            let paths = triangles.iter().map(rounded_path).collect::<String>();
            format!("<g fill=\"{color}\" fill-opacity=\"{opacity}\">{paths}</g>")
        }
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 {size} {size}\">{body}</svg>"
    )
}

fn stroked(segments: &[Segment], cap: &str, color: &str, opacity: f32) -> String {
    let path = segments
        .iter()
        .map(|segment| {
            format!(
                "M{} {} L{} {}",
                segment.a.0, segment.a.1, segment.b.0, segment.b.1
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "<path d=\"{path}\" fill=\"none\" stroke=\"{color}\" stroke-opacity=\"{opacity}\" stroke-width=\"{}\" stroke-linecap=\"{cap}\"/>",
        TRAFFIC_GLYPHS.stroke
    )
}

/// A corner of a rounded triangle: the path comes in at `enter`, bends toward `vertex` and leaves at `exit`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    pub enter: (f32, f32),
    pub vertex: (f32, f32),
    pub exit: (f32, f32),
}

/// The corners of `triangle` rounded on `TRAFFIC_GLYPHS.zoom_corner`, in order; the outline runs from the exit of the first to its enter.
pub fn rounded(triangle: &Triangle) -> [Corner; 3] {
    let points = triangle.0;
    std::array::from_fn(|at| {
        let vertex = points[at];
        let toward = |end: (f32, f32)| {
            let (dx, dy) = (end.0 - vertex.0, end.1 - vertex.1);
            let length = dx.hypot(dy);
            (dx / length, dy / length)
        };
        let (before, after) = (toward(points[(at + 2) % 3]), toward(points[(at + 1) % 3]));
        let angle = (before.0 * after.0 + before.1 * after.1)
            .clamp(-1.0, 1.0)
            .acos();
        let reach = TRAFFIC_GLYPHS.zoom_corner / (angle / 2.0).tan();
        let along = |(ux, uy): (f32, f32)| (vertex.0 + ux * reach, vertex.1 + uy * reach);
        Corner {
            enter: along(before),
            vertex,
            exit: along(after),
        }
    })
}

fn rounded_path(triangle: &Triangle) -> String {
    let [first, second, third] = rounded(triangle);
    let curve = |corner: Corner| {
        let ((ex, ey), (vx, vy), (ox, oy)) = (corner.enter, corner.vertex, corner.exit);
        format!(" L{ex} {ey} Q{vx} {vy} {ox} {oy}")
    };
    format!(
        "<path d=\"M{} {}{}{}{} Z\"/>",
        first.exit.0,
        first.exit.1,
        curve(second),
        curve(third),
        curve(first)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrafficPointer {
    hovered: bool,
    armed: Option<ButtonKind>,
    over_armed: bool,
}

impl TrafficPointer {
    pub fn on_motion(&mut self, hit: Option<ButtonKind>, over_group: bool) -> bool {
        let before = (self.hovered, self.over_armed);
        self.hovered = over_group;
        self.over_armed = self.armed.is_some() && self.armed == hit;
        before != (self.hovered, self.over_armed)
    }

    pub fn on_group_enter(&mut self) -> bool {
        let changed = !self.hovered;
        self.hovered = true;
        changed
    }

    pub fn on_button_enter(&mut self, hit: ButtonKind) -> bool {
        let over_armed = self.armed == Some(hit);
        let changed = self.over_armed != over_armed;
        self.over_armed = over_armed;
        changed
    }

    pub fn on_leave_button(&mut self) -> bool {
        let changed = self.armed.is_some() || self.over_armed;
        self.armed = None;
        self.over_armed = false;
        changed
    }

    pub fn on_press(&mut self, hit: Option<ButtonKind>, over_group: bool) -> bool {
        let before = (self.hovered, self.armed, self.over_armed);
        self.hovered = over_group;
        self.armed = hit;
        self.over_armed = hit.is_some();
        before != (self.hovered, self.armed, self.over_armed)
    }

    pub fn on_release(&mut self, hit: Option<ButtonKind>, over_group: bool) -> Option<ButtonKind> {
        let result = (self.armed == hit).then_some(self.armed).flatten();
        self.hovered = over_group;
        self.armed = None;
        self.over_armed = false;
        result
    }

    pub fn on_leave(&mut self) -> bool {
        let changed = self.hovered || self.armed.is_some();
        self.hovered = false;
        self.armed = None;
        self.over_armed = false;
        changed
    }

    pub fn hovered(&self) -> bool {
        self.hovered
    }

    pub fn pressed_visual(&self) -> Option<ButtonKind> {
        if self.over_armed { self.armed } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctdesign::TRAFFIC_LIGHTS_LIGHT;

    fn input(focused: bool, group_hovered: bool, pressed: Option<ButtonKind>) -> TrafficInput {
        TrafficInput {
            focused,
            group_hovered,
            pressed,
            zoom_supported: true,
            fullscreen: false,
        }
    }

    #[test]
    fn a_light_is_hit_inside_its_circle_and_not_in_the_corner_of_its_box() {
        let (x, y, size) = button_rect(1);
        assert_eq!(
            hit((x + size / 2.0, y + size / 2.0)),
            Some(ButtonKind::Minimize)
        );
        assert_eq!(hit((x + 1.0, y + 1.0)), None);
    }

    #[test]
    fn without_focus_or_hover_the_lights_are_the_inactive_look() {
        let visual = visual(&TRAFFIC_LIGHTS_LIGHT, input(false, false, None));
        assert!(
            visual
                .iter()
                .all(|light| light.fill == TRAFFIC_LIGHTS_LIGHT.inactive.fill)
        );
        assert!(visual.iter().all(|light| light.glyph.is_none()));
    }

    #[test]
    fn hover_lights_and_marks_the_group_even_when_unfocused() {
        let visual = visual(&TRAFFIC_LIGHTS_LIGHT, input(false, true, None));
        assert_eq!(visual[0].fill, TRAFFIC_LIGHTS_LIGHT.close.fill);
        assert_eq!(visual[1].glyph, Some(TRAFFIC_LIGHTS_LIGHT.minimize.glyph));
    }

    #[test]
    fn pressing_scales_and_brightens_only_the_pressed_light() {
        let visual = visual(
            &TRAFFIC_LIGHTS_LIGHT,
            input(true, true, Some(ButtonKind::Close)),
        );
        assert_eq!(visual[0].scale, TRAFFIC_PRESS.scale);
        assert_eq!(visual[0].dodge, TRAFFIC_PRESS.dodge);
        assert_eq!((visual[1].scale, visual[1].dodge), (1.0, 0.0));
        assert_eq!(visual[1].fill, TRAFFIC_LIGHTS_LIGHT.minimize.fill);
    }

    #[test]
    fn in_full_screen_minimize_is_disabled_and_zoom_shows_its_exit_glyph() {
        let visual = visual(
            &TRAFFIC_LIGHTS_LIGHT,
            TrafficInput {
                fullscreen: true,
                ..input(true, true, None)
            },
        );
        assert_eq!(visual[1].fill, TRAFFIC_LIGHTS_LIGHT.inactive.fill);
        assert_eq!(visual[1].glyph, None);
        assert_eq!(Glyph::of(2, false, true), Glyph::ZoomExit);
        assert_eq!(Glyph::of(2, true, false), Glyph::Plus);
    }

    #[test]
    fn unsupported_zoom_is_always_inactive_and_without_a_glyph() {
        let visual = visual(
            &TRAFFIC_LIGHTS_LIGHT,
            TrafficInput {
                zoom_supported: false,
                ..input(true, true, Some(ButtonKind::Maximize))
            },
        );
        assert_eq!(visual[2].fill, TRAFFIC_LIGHTS_LIGHT.inactive.fill);
        assert_eq!(visual[2].glyph, None);
        assert_eq!((visual[2].scale, visual[2].dodge), (1.0, 0.0));
    }

    #[test]
    fn the_standard_style_is_the_measured_look_and_a_tint_keeps_its_alpha_and_takes_the_hue() {
        let accent = Rgba::rgb8(0, 122, 255);
        let standard = looks(
            TrafficLightStyle::Standard,
            &TRAFFIC_LIGHTS_LIGHT,
            accent,
            None,
        );
        assert_eq!(standard, TRAFFIC_LIGHTS_LIGHT);
        let tinted = looks(
            TrafficLightStyle::Accent,
            &TRAFFIC_LIGHTS_LIGHT,
            accent,
            None,
        );
        let alphas = |look: TrafficLook| look.fill.map(|color| color.a);
        assert_eq!(alphas(tinted.close), alphas(TRAFFIC_LIGHTS_LIGHT.close));
        assert_eq!(tinted.inactive, TRAFFIC_LIGHTS_LIGHT.inactive);
        let (hue, _, _) = tinted.close.fill[1].to_hsl();
        assert!((hue - accent.to_hsl().0).abs() < 0.02);
    }

    #[test]
    fn pressing_one_button_and_releasing_on_another_does_not_fire() {
        let mut pointer = TrafficPointer::default();
        assert!(pointer.on_press(Some(ButtonKind::Close), true));
        assert_eq!(pointer.pressed_visual(), Some(ButtonKind::Close));
        assert!(pointer.on_motion(Some(ButtonKind::Minimize), true));
        assert_eq!(pointer.pressed_visual(), None);
        assert_eq!(pointer.on_release(Some(ButtonKind::Minimize), true), None);
    }

    #[test]
    fn pressing_and_releasing_the_same_button_fires() {
        let mut pointer = TrafficPointer::default();
        pointer.on_press(Some(ButtonKind::Close), true);
        assert_eq!(
            pointer.on_release(Some(ButtonKind::Close), true),
            Some(ButtonKind::Close)
        );
    }

    #[test]
    fn leaving_with_an_armed_button_does_not_fire() {
        let mut pointer = TrafficPointer::default();
        pointer.on_press(Some(ButtonKind::Close), true);
        assert!(pointer.on_leave());
        assert_eq!(pointer.on_release(None, false), None);
    }

    #[test]
    fn a_group_leave_followed_by_a_light_leave_keeps_the_glyphs_hidden() {
        let mut pointer = TrafficPointer::default();
        pointer.on_group_enter();
        pointer.on_leave();
        pointer.on_leave_button();
        assert!(!pointer.hovered());
        assert_eq!(pointer.pressed_visual(), None);
    }

    #[test]
    fn a_light_leave_clears_the_press_but_not_the_group_hover() {
        let mut pointer = TrafficPointer::default();
        pointer.on_group_enter();
        pointer.on_press(Some(ButtonKind::Close), true);
        pointer.on_leave_button();
        assert!(pointer.hovered());
        assert_eq!(pointer.pressed_visual(), None);
    }
}
