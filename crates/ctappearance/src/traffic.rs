use ctdesign::{
    ButtonKind, Rgba, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TRAFFIC_CLOSE_MINIMIZE_DARKEN,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrafficPointer {
    hovered: bool,
    armed: Option<ButtonKind>,
    over_armed: bool,
}

impl TrafficPointer {
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
