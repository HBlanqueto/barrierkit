use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;
use freya_core::elements::image::ImageHandle;
use freya_core::prelude::*;
use freya_engine::prelude::{
    ColorType, FontStyle, Image as SkImage, ParagraphBuilder, ParagraphStyle, Slant, TextStyle,
    Weight, Width, raster_n32_premul,
};
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{CAPTION, PRINT, SPACE_2, SPACE_3, TEXT};

use crate::i18n::text;
use crate::scroll::ScrollView;
use crate::style::{Palette, text};
use crate::theme::color;

const MM_PER_INCH: f32 = 25.4;
const POINTS_PER_INCH: f32 = 72.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageSetup {
    pub width: f32,
    pub height: f32,
    pub margins: [f32; 4],
}

impl Default for PageSetup {
    fn default() -> Self {
        Self {
            width: 210.0,
            height: 297.0,
            margins: [0.0; 4],
        }
    }
}

impl PageSetup {
    fn pixels(&self, mm: f32) -> f32 {
        mm / MM_PER_INCH * PRINT.dpi
    }

    pub fn size_in_pixels(&self) -> (i32, i32) {
        (
            self.pixels(self.width).round() as i32,
            self.pixels(self.height).round() as i32,
        )
    }

    pub fn content_in_pixels(&self) -> (f32, f32, f32, f32) {
        let point = PRINT.dpi / POINTS_PER_INCH;
        let [top, right, bottom, left] = self.margins.map(|mm| self.pixels(mm));
        let (x, y) = (PRINT.margin_x * point, PRINT.margin_y * point);
        let (top, right, bottom, left) = (top.max(y), right.max(x), bottom.max(y), left.max(x));
        let (width, height) = self.size_in_pixels();
        (
            left,
            top,
            width as f32 - left - right,
            height as f32 - top - bottom,
        )
    }
}

pub fn page_starts(lines: &[(f32, f32)], page_height: f32) -> Vec<f32> {
    let mut starts = vec![0.0];
    for &(top, bottom) in lines {
        let start = *starts.last().unwrap_or(&0.0);
        if bottom - start > page_height && top > start {
            starts.push(top);
        }
    }
    starts
}

pub fn text_pages(text: &str, setup: &PageSetup) -> Vec<SkImage> {
    let (left, top, width, height) = setup.content_in_pixels();
    let mut style = TextStyle::new();
    style.set_font_size(TEXT.size as f32 * PRINT.dpi / POINTS_PER_INCH);
    style.set_font_families(&[TEXT.family]);
    style.set_font_style(FontStyle::new(
        Weight::NORMAL,
        Width::NORMAL,
        Slant::Upright,
    ));
    style.set_color(freya_engine::prelude::Color::BLACK);
    let mut paragraph_style = ParagraphStyle::new();
    paragraph_style.set_text_style(&style);
    let mut builder = ParagraphBuilder::new(&paragraph_style, crate::fonts::collection());
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(width);
    let lines: Vec<(f32, f32)> = paragraph
        .get_line_metrics()
        .iter()
        .map(|line| {
            let baseline = line.baseline as f32;
            (
                baseline - line.ascent as f32,
                baseline + line.descent as f32,
            )
        })
        .collect();
    let starts = page_starts(&lines, height);
    starts
        .iter()
        .enumerate()
        .filter_map(|(index, &start)| {
            let end = starts.get(index + 1).copied().unwrap_or(f32::MAX);
            let mut surface = raster_n32_premul(setup.size_in_pixels())?;
            let canvas = surface.canvas();
            canvas.clear(freya_engine::prelude::Color::WHITE);
            canvas.save();
            canvas.clip_rect(
                freya_engine::prelude::Rect::from_xywh(left, top, width, (end - start).min(height)),
                None,
                None,
            );
            paragraph.paint(canvas, (left, top - start));
            canvas.restore();
            Some(surface.image_snapshot())
        })
        .collect()
}

pub fn pdf_from_images(setup: &PageSetup, pages: &[SkImage]) -> Vec<u8> {
    let rasters: Vec<(u32, u32, Vec<u8>)> = pages
        .iter()
        .filter_map(|page| {
            let pixels = page.peek_pixels()?;
            let bgra = pixels.color_type() == ColorType::BGRA8888;
            let rgb = pixels
                .bytes()?
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|pixel| {
                    if bgra {
                        [pixel[2], pixel[1], pixel[0]]
                    } else {
                        [pixel[0], pixel[1], pixel[2]]
                    }
                })
                .collect();
            Some((page.width() as u32, page.height() as u32, rgb))
        })
        .collect();
    let points = |mm: f32| mm / MM_PER_INCH * POINTS_PER_INCH;
    pdf((points(setup.width), points(setup.height)), &rasters)
}

pub fn pdf(page_size: (f32, f32), pages: &[(u32, u32, Vec<u8>)]) -> Vec<u8> {
    let mut objects: Vec<Vec<u8>> = vec![Vec::new(), Vec::new()];
    let mut kids = Vec::new();
    for (width, height, rgb) in pages {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        let _ = encoder.write_all(rgb);
        let pixels = encoder.finish().unwrap_or_default();
        let image = objects.len() + 1;
        let mut stream = format!(
            "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
             /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
            pixels.len()
        )
        .into_bytes();
        stream.extend(&pixels);
        stream.extend(b"\nendstream");
        objects.push(stream);
        let draw = format!("q {} 0 0 {} 0 0 cm /Im0 Do Q", page_size.0, page_size.1);
        objects
            .push(format!("<< /Length {} >>\nstream\n{draw}\nendstream", draw.len()).into_bytes());
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] \
                 /Resources << /XObject << /Im0 {image} 0 R >> >> /Contents {} 0 R >>",
                page_size.0,
                page_size.1,
                image + 1
            )
            .into_bytes(),
        );
        kids.push(format!("{} 0 R", objects.len()));
    }
    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        kids.len()
    )
    .into_bytes();
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend(object);
        out.extend(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for offset in offsets {
        out.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[derive(PartialEq)]
pub struct PrintPreview {
    pages: Vec<ImageHandle>,
    setup: PageSetup,
}

impl PrintPreview {
    pub fn new(setup: PageSetup, pages: &[SkImage]) -> Self {
        Self {
            pages: pages
                .iter()
                .map(|page| ImageHandle::new(page.clone(), Bytes::new()))
                .collect(),
            setup,
        }
    }
}

impl Component for PrintPreview {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let total = self.pages.len();
        let aspect = self.setup.height / self.setup.width;
        let mut width = use_state(|| 0.0f32);
        let page_height = *width.read() * aspect;
        let pages = self.pages.iter().enumerate().map(|(index, page)| {
            rect()
                .width(Size::fill())
                .direction(Direction::Vertical)
                .cross_align(Alignment::Center)
                .spacing(SPACE_2)
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(page_height))
                        .background(color(palette.content_background))
                        .shadow(Shadow::new().blur(SPACE_2).color(color(palette.separator)))
                        .child(image(page.clone()).width(Size::fill()).height(Size::fill())),
                )
                .child(text(
                    text!(
                        "print-page",
                        page = (index + 1).to_string(),
                        total = total.to_string()
                    ),
                    CAPTION,
                    palette.secondary_label,
                ))
                .into_element()
        });
        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(SPACE_3))
                    .spacing(SPACE_3)
                    .on_sized(move |event: Event<SizedEventData>| {
                        width.set_if_modified(event.area.width() - 2.0 * SPACE_3)
                    })
                    .a11y_role(AccessibilityRole::Group)
                    .a11y_alt(text!("print-preview"))
                    .children(pages),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_that_would_cross_the_page_bottom_starts_the_next_page() {
        let lines = [(0.0, 10.0), (10.0, 20.0), (20.0, 30.0), (30.0, 40.0)];
        assert_eq!(page_starts(&lines, 25.0), [0.0, 20.0]);
        assert_eq!(page_starts(&lines, 100.0), [0.0]);
    }

    #[test]
    fn the_pdf_cross_reference_points_at_every_object() {
        let out = pdf(
            (595.0, 842.0),
            &[(1, 1, vec![255, 255, 255]), (1, 1, vec![0, 0, 0])],
        );
        let find = |needle: &[u8]| out.windows(needle.len()).rposition(|w| w == needle);
        assert!(out.starts_with(b"%PDF-1.4\n") && out.ends_with(b"%%EOF\n"));
        assert!(find(b"/Count 2").is_some());
        let xref = find(b"xref\n0 ").unwrap();
        let table = String::from_utf8(out[xref..].to_vec()).unwrap();
        let entries: Vec<usize> = table
            .lines()
            .skip(3)
            .take_while(|line| line.ends_with(" n "))
            .map(|line| line[..10].parse().unwrap())
            .collect();
        assert_eq!(entries.len(), 8);
        for (index, offset) in entries.iter().enumerate() {
            assert!(out[*offset..].starts_with(format!("{} 0 obj", index + 1).as_bytes()));
        }
        let start: usize = table.lines().rev().nth(1).unwrap().parse().unwrap();
        assert_eq!(start, xref);
    }
}
