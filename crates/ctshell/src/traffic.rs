use ctappearance::traffic::{Corner, Glyph, button_rect, rounded};
use ctdesign::{Rgba, Segment, TITLEBAR, TRAFFIC_GLYPHS, Triangle};

pub fn group_rect() -> (f64, f64, f64, f64) {
    let (x, y, size) = button_rect(0);
    let count = TITLEBAR.buttons.order.len() as f64;
    let gap = f64::from(TITLEBAR.buttons.spacing);
    (x, y, count * size + (count - 1.0) * gap, size)
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
