use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;
use freya_core::elements::image::ImageHandle;
use freya_core::prelude::*;
use freya_engine::prelude::{
    ColorType, Font, FontStyle, Image as SkImage, Paint, Paragraph as SkParagraph,
    ParagraphBuilder, ParagraphStyle, Point, Slant, TextStyle, Typeface, Weight, Width,
    raster_n32_premul,
};
use pdf_writer::types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, Str};
use subsetter::GlyphRemapper;
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{BODY, CAPTION, PRINT, SPACE_2, SPACE_3};

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

    fn size_in_points(&self) -> (f32, f32) {
        (points(self.width), points(self.height))
    }

    fn content_in_points(&self) -> (f32, f32, f32, f32) {
        let [top, right, bottom, left] = self.margins.map(points);
        let (x, y) = (PRINT.margin_x, PRINT.margin_y);
        let (top, right, bottom, left) = (top.max(y), right.max(x), bottom.max(y), left.max(x));
        let (width, height) = self.size_in_points();
        (left, top, width - left - right, height - top - bottom)
    }
}

fn points(mm: f32) -> f32 {
    mm / MM_PER_INCH * POINTS_PER_INCH
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

pub fn page_of(starts: &[f32], top: f32) -> usize {
    starts.iter().rposition(|&start| start <= top).unwrap_or(0)
}

struct TextLayout {
    paragraph: SkParagraph,
    lines: Vec<(f32, f32)>,
    starts: Vec<f32>,
    content: (f32, f32, f32, f32),
}

fn text_layout(text: &str, setup: &PageSetup) -> TextLayout {
    let content = setup.content_in_points();
    let mut style = TextStyle::new();
    style.set_font_size(BODY.size as f32);
    style.set_font_families(&[BODY.family]);
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
    paragraph.layout(content.2);
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
    let starts = page_starts(&lines, content.3);
    TextLayout {
        paragraph,
        lines,
        starts,
        content,
    }
}

pub fn text_pages(text: &str, setup: &PageSetup) -> Vec<SkImage> {
    let layout = text_layout(text, setup);
    let (left, top, width, height) = layout.content;
    let starts = &layout.starts;
    starts
        .iter()
        .enumerate()
        .filter_map(|(index, &start)| {
            let end = starts.get(index + 1).copied().unwrap_or(f32::MAX);
            let mut surface = raster_n32_premul(setup.size_in_pixels())?;
            let canvas = surface.canvas();
            canvas.clear(freya_engine::prelude::Color::WHITE);
            canvas.scale((PRINT.dpi / POINTS_PER_INCH, PRINT.dpi / POINTS_PER_INCH));
            canvas.clip_rect(
                freya_engine::prelude::Rect::from_xywh(left, top, width, (end - start).min(height)),
                None,
                None,
            );
            layout.paragraph.paint(canvas, (left, top - start));
            Some(surface.image_snapshot())
        })
        .collect()
}

struct Glyphs {
    font: usize,
    size: f32,
    page: usize,
    ids: Vec<u16>,
    positions: Vec<(f32, f32)>,
    starts: Vec<u32>,
}

struct EmbeddedFont {
    typeface: Typeface,
    remapper: GlyphRemapper,
    texts: Vec<(u16, String)>,
}

fn glyph_runs(layout: &mut TextLayout, text: &str) -> (Vec<Glyphs>, Vec<EmbeddedFont>) {
    let mut runs = Vec::new();
    let mut fonts: Vec<EmbeddedFont> = Vec::new();
    let (lines, starts) = (layout.lines.clone(), layout.starts.clone());
    layout.paragraph.visit(|line, info| {
        let Some(info) = info else {
            return;
        };
        let typeface = info.font().typeface();
        let font = fonts
            .iter()
            .position(|font| font.typeface.unique_id() == typeface.unique_id())
            .unwrap_or_else(|| {
                fonts.push(EmbeddedFont {
                    typeface,
                    remapper: GlyphRemapper::new(),
                    texts: Vec::new(),
                });
                fonts.len() - 1
            });
        let origin = info.origin();
        runs.push(Glyphs {
            font,
            size: info.font().size(),
            page: page_of(&starts, lines.get(line).map_or(0.0, |line| line.0)),
            ids: info.glyphs().to_vec(),
            positions: info
                .positions()
                .iter()
                .map(|point| (origin.x + point.x, origin.y + point.y))
                .collect(),
            starts: info.utf8_starts()[..info.count()].to_vec(),
        });
    });
    let mut bounds: Vec<usize> = runs
        .iter()
        .flat_map(|run| run.starts.iter().map(|&start| start as usize))
        .chain([text.len()])
        .collect();
    bounds.sort_unstable();
    bounds.dedup();
    for run in &mut runs {
        let font = &mut fonts[run.font];
        let clusters = cluster_texts(text, &run.starts, &bounds);
        for (&id, cluster) in run.ids.iter().zip(&clusters) {
            let fresh = font.remapper.get(id).is_none();
            let new = font.remapper.remap(id);
            if fresh && !cluster.is_empty() {
                font.texts.push((new, cluster.clone()));
            }
        }
    }
    (runs, fonts)
}

// Glyphs come in visual order, so in a right-to-left run the starts fall: a cluster ends at the
// next larger start of the paragraph, and only its first glyph carries the text.
fn cluster_texts(text: &str, starts: &[u32], bounds: &[usize]) -> Vec<String> {
    let mut seen = Vec::new();
    starts
        .iter()
        .map(|&start| {
            let from = start as usize;
            if seen.contains(&from) {
                return String::new();
            }
            seen.push(from);
            let to = bounds
                .iter()
                .copied()
                .find(|&bound| bound > from)
                .unwrap_or(from);
            text.get(from..to).unwrap_or_default().to_owned()
        })
        .collect()
}

struct Bitmap {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
    alpha: Vec<u8>,
    frame: [f32; 4],
}

fn bitmap_run(run: &Glyphs, typeface: &Typeface) -> Option<Bitmap> {
    let scale = PRINT.dpi / POINTS_PER_INCH;
    let (first, last) = (run.positions.first()?, run.positions.last()?);
    let (x, y) = (first.0, last.1 - run.size);
    let (width, height) = (last.0 - first.0 + run.size * 1.5, run.size * 1.5);
    let mut surface = raster_n32_premul((
        (width * scale).ceil() as i32,
        (height * scale).ceil() as i32,
    ))?;
    let canvas = surface.canvas();
    canvas.clear(freya_engine::prelude::Color::TRANSPARENT);
    canvas.scale((scale, scale));
    let positions: Vec<Point> = run
        .positions
        .iter()
        .map(|&(px, py)| Point::new(px - x, py - y))
        .collect();
    canvas.draw_glyphs_at(
        &run.ids,
        positions.as_slice(),
        Point::new(0.0, 0.0),
        &Font::from_typeface(typeface.clone(), run.size),
        &Paint::default(),
    );
    let image = surface.image_snapshot();
    let pixels = image.peek_pixels()?;
    let bgra = pixels.color_type() == ColorType::BGRA8888;
    let (mut rgb, mut alpha) = (Vec::new(), Vec::new());
    for pixel in pixels.bytes()?.as_chunks::<4>().0 {
        let (r, g, b) = if bgra {
            (pixel[2], pixel[1], pixel[0])
        } else {
            (pixel[0], pixel[1], pixel[2])
        };
        let unpremultiply =
            |channel: u8| (u32::from(channel) * 255 / u32::from(pixel[3].max(1))) as u8;
        rgb.extend([unpremultiply(r), unpremultiply(g), unpremultiply(b)]);
        alpha.push(pixel[3]);
    }
    Some(Bitmap {
        width: image.width() as u32,
        height: image.height() as u32,
        rgb,
        alpha,
        frame: [x, y, width, height],
    })
}

fn write_bitmap(pdf: &mut Pdf, next: &mut impl FnMut() -> Ref, bitmap: &Bitmap) -> Ref {
    let (image, mask) = (next(), next());
    let (rgb, alpha) = (compressed(&bitmap.rgb), compressed(&bitmap.alpha));
    let mut xobject = pdf.image_xobject(image, &rgb);
    xobject.filter(Filter::FlateDecode);
    xobject
        .width(bitmap.width as i32)
        .height(bitmap.height as i32)
        .bits_per_component(8)
        .s_mask(mask);
    xobject.color_space().device_rgb();
    xobject.finish();
    let mut soft = pdf.image_xobject(mask, &alpha);
    soft.filter(Filter::FlateDecode);
    soft.width(bitmap.width as i32)
        .height(bitmap.height as i32)
        .bits_per_component(8);
    soft.color_space().device_gray();
    soft.finish();
    image
}

fn compressed(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(data);
    encoder.finish().unwrap_or_default()
}

fn subset_tag(index: usize) -> String {
    (0..6)
        .map(|digit| (b'A' + (index / 26usize.pow(digit) % 26) as u8) as char)
        .collect()
}

fn write_font(
    pdf: &mut Pdf,
    next: &mut impl FnMut() -> Ref,
    index: usize,
    font: &EmbeddedFont,
) -> Option<Ref> {
    let (data, face_index) = font.typeface.to_font_data()?;
    let face = ttf_parser::Face::parse(&data, face_index as u32).ok()?;
    let subset = subsetter::subset(&data, face_index as u32, &font.remapper).ok()?;
    let scale = 1000.0 / f32::from(face.units_per_em());
    let postscript = face
        .names()
        .into_iter()
        .filter(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        .find_map(|name| name.to_string())
        .unwrap_or_else(|| "Font".into());
    let base = format!("{}+{postscript}", subset_tag(index));
    let cff = face.tables().cff.is_some();
    let (type0, cid, descriptor, file, cmap) = (next(), next(), next(), next(), next());
    pdf.type0_font(type0)
        .base_font(Name(base.as_bytes()))
        .encoding_predefined(Name(b"Identity-H"))
        .descendant_font(cid)
        .to_unicode(cmap);
    let system_info = SystemInfo {
        registry: Str(b"Adobe"),
        ordering: Str(b"Identity"),
        supplement: 0,
    };
    let widths: Vec<f32> = font
        .remapper
        .remapped_gids()
        .map(|old| {
            face.glyph_hor_advance(ttf_parser::GlyphId(old))
                .unwrap_or(0) as f32
                * scale
        })
        .collect();
    let mut cid_font = pdf.cid_font(cid);
    cid_font
        .subtype(if cff {
            CidFontType::Type0
        } else {
            CidFontType::Type2
        })
        .base_font(Name(base.as_bytes()))
        .system_info(system_info)
        .font_descriptor(descriptor)
        .default_width(0.0);
    if !cff {
        cid_font.cid_to_gid_map_predefined(Name(b"Identity"));
    }
    cid_font.widths().consecutive(0, widths);
    cid_font.finish();
    let bbox = face.global_bounding_box();
    let mut flags = FontFlags::SYMBOLIC;
    flags.set(FontFlags::FIXED_PITCH, face.is_monospaced());
    flags.set(FontFlags::ITALIC, face.is_italic());
    let mut descriptor_dict = pdf.font_descriptor(descriptor);
    descriptor_dict
        .name(Name(base.as_bytes()))
        .flags(flags)
        .bbox(Rect::new(
            f32::from(bbox.x_min) * scale,
            f32::from(bbox.y_min) * scale,
            f32::from(bbox.x_max) * scale,
            f32::from(bbox.y_max) * scale,
        ))
        .italic_angle(face.italic_angle())
        .ascent(f32::from(face.ascender()) * scale)
        .descent(f32::from(face.descender()) * scale)
        .cap_height(f32::from(face.capital_height().unwrap_or(face.ascender())) * scale)
        .stem_v(10.0 + 0.244 * (f32::from(face.weight().to_number()) - 50.0));
    if cff {
        descriptor_dict.font_file3(file);
    } else {
        descriptor_dict.font_file2(file);
    }
    descriptor_dict.finish();
    let packed = compressed(&subset);
    let mut stream = pdf.stream(file, &packed);
    stream.filter(Filter::FlateDecode);
    if cff {
        stream.pair(Name(b"Subtype"), Name(b"OpenType"));
    } else {
        stream.pair(Name(b"Length1"), subset.len() as i32);
    }
    stream.finish();
    let mut unicode = UnicodeCmap::new(Name(b"Custom"), system_info);
    for (id, text) in &font.texts {
        unicode.pair_with_multiple(*id, text.chars());
    }
    pdf.cmap(cmap, &unicode.finish());
    Some(type0)
}

pub fn text_pdf(text: &str, setup: &PageSetup) -> Vec<u8> {
    let mut layout = text_layout(text, setup);
    let (left, top, _, _) = layout.content;
    let (page_width, page_height) = setup.size_in_points();
    let (runs, fonts) = glyph_runs(&mut layout, text);
    let mut pdf = Pdf::new();
    pdf.set_version(1, 7);
    let mut last = 0;
    let mut next = || {
        last += 1;
        Ref::new(last)
    };
    let (catalog, tree) = (next(), next());
    let font_refs: Vec<Option<Ref>> = fonts
        .iter()
        .enumerate()
        .map(|(index, font)| write_font(&mut pdf, &mut next, index, font))
        .collect();
    let names: Vec<String> = (0..fonts.len()).map(|index| format!("F{index}")).collect();
    let mut kids = Vec::new();
    let mut drawn = std::collections::HashMap::new();
    for (page, &start) in layout.starts.iter().enumerate() {
        let mut content = Content::new();
        let mut images = Vec::new();
        for run in runs.iter().filter(|run| run.page == page) {
            let font = &fonts[run.font];
            if font_refs[run.font].is_none() {
                let Some(bitmap) = bitmap_run(run, &font.typeface) else {
                    continue;
                };
                let [x, y, width, height] = bitmap.frame;
                let key = (run.font, run.size.to_bits(), run.ids.clone());
                let image = *drawn
                    .entry(key)
                    .or_insert_with(|| write_bitmap(&mut pdf, &mut next, &bitmap));
                let name = format!("Im{}", images.len());
                content
                    .save_state()
                    .transform([
                        width,
                        0.0,
                        0.0,
                        height,
                        left + x,
                        page_height - (top + y - start) - height,
                    ])
                    .x_object(Name(name.as_bytes()))
                    .restore_state();
                images.push((name, image));
                continue;
            }
            content.begin_text();
            content.set_font(Name(names[run.font].as_bytes()), run.size);
            for (id, (x, y)) in run.ids.iter().zip(&run.positions) {
                let id = font.remapper.get(*id).unwrap_or(0);
                content.set_text_matrix([
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    left + x,
                    page_height - (top + y - start),
                ]);
                content.show(Str(&id.to_be_bytes()));
            }
            content.end_text();
        }
        let (page_ref, content_ref) = (next(), next());
        pdf.stream(content_ref, &compressed(&content.finish()))
            .filter(Filter::FlateDecode);
        let mut page_dict = pdf.page(page_ref);
        page_dict
            .parent(tree)
            .media_box(Rect::new(0.0, 0.0, page_width, page_height))
            .contents(content_ref);
        let mut resources = page_dict.resources();
        let mut font_dict = resources.fonts();
        for (name, font) in names.iter().zip(&font_refs) {
            if let Some(font) = font {
                font_dict.pair(Name(name.as_bytes()), *font);
            }
        }
        font_dict.finish();
        let mut xobjects = resources.x_objects();
        for (name, image) in &images {
            xobjects.pair(Name(name.as_bytes()), *image);
        }
        xobjects.finish();
        resources.finish();
        page_dict.finish();
        kids.push(page_ref);
    }
    pdf.pages(tree).count(kids.len() as i32).kids(kids);
    pdf.catalog(catalog).pages(tree);
    pdf.finish()
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
                        .child(image(page.clone()).expanded()),
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
        ScrollView::new().expanded().child(
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
    fn a_line_belongs_to_the_last_page_that_starts_at_or_above_it() {
        let starts = [0.0, 20.0, 40.0];
        assert_eq!(page_of(&starts, 0.0), 0);
        assert_eq!(page_of(&starts, 20.0), 1);
        assert_eq!(page_of(&starts, 39.0), 1);
        assert_eq!(page_of(&starts, 55.0), 2);
    }

    #[test]
    fn a_right_to_left_run_maps_each_glyph_to_its_own_letters() {
        let text = "שלום";
        let starts = [6, 4, 2, 0];
        let bounds = [0, 2, 4, 6, 8];
        assert_eq!(
            super::cluster_texts(text, &starts, &bounds),
            ["ם", "ו", "ל", "ש"]
        );
    }

    #[test]
    fn a_cluster_of_several_glyphs_gives_its_text_to_the_first_one() {
        let text = "ae\u{301}b";
        let starts = [0, 1, 1, 4];
        let bounds = [0, 1, 4, 5];
        assert_eq!(
            super::cluster_texts(text, &starts, &bounds),
            ["a", "e\u{301}", "", "b"]
        );
    }

    #[test]
    fn subset_tags_are_six_capital_letters_and_differ_per_font() {
        assert_eq!(subset_tag(0), "AAAAAA");
        assert_eq!(subset_tag(1), "BAAAAA");
        assert_eq!(subset_tag(27), "BBAAAA");
    }
}
