use ctappearance::ColorScheme;
use ctdesign::{DURATION_BASE, HAIRLINE, SHADOW, SHADOW_DARK, TITLEBAR_REVEAL};

use crate::chrome::{RestingEdge, Reveal};
use crate::styles::button::rounded;
use crate::styles::{Animation, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue};

/// `WindowChrome::new(child).style(..)` takes how the title bar reveals and the
/// look of its resting edge and pill. Its paints are `focused` while the window
/// is active.
#[derive(Clone, Debug)]
pub struct WindowChromeStyle {
    /// Tengen: `Always` (9 (`docs/barrierkit/09-titlebars.md#window-chrome`)).
    pub reveal: Reveal,
    /// Tengen: `None`; `Line` draws `line_edge`, `Accent` `accent_edge`.
    pub resting_edge: RestingEdge,
    pub pill_inset: f32,
    pub pill_padding: (f32, f32),
    /// Tengen: `RoundedRectangle { radius: TITLEBAR_REVEAL.pill_radius, style: Circular }`.
    pub pill_shape: Shape,
    /// Tengen: `Role(RaisedBackground)`.
    pub pill_background: StateValue<ShapeStyle>,
    /// Tengen: `(Color of the sidebar line, TITLEBAR_REVEAL.edge_line)`.
    pub line_edge: (ShapeStyle, f32),
    /// Tengen: `(Accent, TITLEBAR_REVEAL.edge_accent)`.
    pub accent_edge: (ShapeStyle, f32),
    /// Tengen: `Some((Role(Outline), HAIRLINE))`, drawn inside.
    pub pill_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[SHADOW.contact]` (`SHADOW_DARK.contact` in dark) with its `opacity`, under a
    /// drawer and the pill.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: a hidden bar slides and the lights fade
    /// in; `None` makes them jump.
    pub animation: Animation,
}

impl ControlStyle for WindowChromeStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let line = ShapeStyle::Color(resolved.sidebar.palette.line);
        let contact = if resolved.scheme == ColorScheme::Light {
            SHADOW.contact
        } else {
            SHADOW_DARK.contact
        };
        Self {
            reveal: Reveal::Always,
            resting_edge: RestingEdge::None,
            pill_inset: TITLEBAR_REVEAL.pill_inset,
            pill_padding: (
                TITLEBAR_REVEAL.pill_padding_x,
                TITLEBAR_REVEAL.pill_padding_y,
            ),
            pill_shape: rounded(TITLEBAR_REVEAL.pill_radius),
            pill_background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground)),
            line_edge: (line, TITLEBAR_REVEAL.edge_line),
            accent_edge: (ShapeStyle::Accent, TITLEBAR_REVEAL.edge_accent),
            pill_border: Some((ShapeStyle::Role(Role::Outline), HAIRLINE)),
            shadows: vec![ShadowStyle {
                color: ctdesign::Rgba {
                    a: contact.opacity,
                    ..contact.color
                },
                ..contact.into()
            }],
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
        }
    }
}
