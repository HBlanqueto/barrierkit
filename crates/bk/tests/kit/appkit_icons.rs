use std::{fs, path::PathBuf};

use bk::symbol::{FontWeight, Image, ImageScale, Symbol, SymbolRenderingMode};
use bk::theme;
use ctdesign::{
    Rgba, SYMBOL, SYMBOL_HIERARCHY, SYMBOL_SCALE_SIZE, SYMBOL_WEIGHT_STROKE_LARGE,
    SYMBOL_WEIGHT_STROKE_MEDIUM, SYMBOL_WEIGHT_STROKE_SMALL,
};
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Position, Size, Size2D};

const ICONS: [(&str, Symbol); 83] = [
    ("icloud", Symbol::ICloud),
    ("arrow.down.circle", Symbol::ArrowDownCircle),
    ("music.note", Symbol::MusicNote),
    ("internaldrive", Symbol::InternalDrive),
    ("externaldrive", Symbol::ExternalDrive),
    ("trash", Symbol::Trash),
    ("tag", Symbol::Tag),
    ("square.and.arrow.up", Symbol::SquareAndArrowUp),
    ("square.and.arrow.down", Symbol::SquareAndArrowDown),
    ("arrow.up.arrow.down", Symbol::ArrowUpArrowDown),
    (
        "line.3.horizontal.decrease",
        Symbol::Line3HorizontalDecrease,
    ),
    ("info.circle", Symbol::InfoCircle),
    ("plus.circle", Symbol::PlusCircle),
    ("minus.circle", Symbol::MinusCircle),
    ("xmark.circle", Symbol::XMarkCircle),
    ("checkmark.circle", Symbol::CheckmarkCircle),
    ("exclamationmark.circle", Symbol::ExclamationMarkCircle),
    ("questionmark.circle", Symbol::QuestionMarkCircle),
    ("exclamationmark.triangle", Symbol::ExclamationMarkTriangle),
    ("nosign", Symbol::NoSign),
    ("arrow.left", Symbol::ArrowLeft),
    ("arrow.right", Symbol::ArrowRight),
    ("arrow.up", Symbol::ArrowUp),
    ("arrow.up.forward", Symbol::ArrowUpForward),
    ("heart", Symbol::Heart),
    ("key", Symbol::Key),
    ("mic", Symbol::Mic),
    ("camera", Symbol::Camera),
    ("envelope", Symbol::Envelope),
    ("calendar", Symbol::Calendar),
    ("flag", Symbol::Flag),
    ("paperplane", Symbol::PaperPlane),
    ("scissors", Symbol::Scissors),
    ("xmark.circle.fill", Symbol::XMarkCircleFill),
    ("plus.circle.fill", Symbol::PlusCircleFill),
    ("minus.circle.fill", Symbol::MinusCircleFill),
    ("checkmark.circle.fill", Symbol::CheckmarkCircleFill),
    (
        "exclamationmark.circle.fill",
        Symbol::ExclamationMarkCircleFill,
    ),
    ("info.circle.fill", Symbol::InfoCircleFill),
    ("questionmark.circle.fill", Symbol::QuestionMarkCircleFill),
    (
        "exclamationmark.triangle.fill",
        Symbol::ExclamationMarkTriangleFill,
    ),
    ("gearshape", Symbol::GearShape),
    ("arrow.clockwise", Symbol::ArrowClockwise),
    ("arrow.counterclockwise", Symbol::ArrowCounterclockwise),
    ("text.alignleft", Symbol::TextAlignLeft),
    ("text.aligncenter", Symbol::TextAlignCenter),
    ("text.alignright", Symbol::TextAlignRight),
    ("text.justify", Symbol::TextJustify),
    ("macwindow", Symbol::MacWindow),
    ("sidebar.right", Symbol::SidebarRight),
    ("doc.on.doc", Symbol::DocOnDoc),
    ("arrow.uturn.backward", Symbol::ArrowUturnBackward),
    ("arrow.uturn.forward", Symbol::ArrowUturnForward),
    ("desktopcomputer", Symbol::DesktopComputer),
    ("person.crop.circle", Symbol::PersonCropCircle),
    ("eject", Symbol::Eject),
    ("arrow.down", Symbol::ArrowDown),
    ("link", Symbol::Link),
    ("pencil", Symbol::Pencil),
    ("square.and.pencil", Symbol::SquareAndPencil),
    ("lock.open", Symbol::LockOpen),
    ("lock.fill", Symbol::LockFill),
    ("stop.fill", Symbol::StopFill),
    ("speaker.slash", Symbol::SpeakerSlash),
    ("speaker.wave.1", Symbol::SpeakerWave1),
    ("repeat", Symbol::Repeat),
    ("checkmark.square", Symbol::CheckmarkSquare),
    ("tray", Symbol::Tray),
    ("doc.text", Symbol::DocText),
    ("xmark.rectangle", Symbol::XMarkRectangle),
    ("plus.rectangle", Symbol::PlusRectangle),
    ("rectangle.on.rectangle", Symbol::RectangleOnRectangle),
    ("rectangle.dashed", Symbol::RectangleDashed),
    ("macwindow.on.rectangle", Symbol::MacWindowOnRectangle),
    ("arrow.right.circle", Symbol::ArrowRightCircle),
    ("arrow.left.circle", Symbol::ArrowLeftCircle),
    ("arrow.up.circle", Symbol::ArrowUpCircle),
    ("chevron.right.circle", Symbol::ChevronRightCircle),
    ("chevron.left.circle", Symbol::ChevronLeftCircle),
    ("play.circle", Symbol::PlayCircle),
    ("pause.circle", Symbol::PauseCircle),
    ("heart.fill", Symbol::HeartFill),
    ("camera.fill", Symbol::CameraFill),
];

const ICON_SIZE: f32 = 20.0;
const ICON_CELL: f32 = 32.0;
const ICON_COLUMNS: usize = 8;
const ICON_PIXELS: usize = 4;
const HIERARCHICAL: [Symbol; 30] = [
    Symbol::ArrowDownCircle,
    Symbol::InfoCircle,
    Symbol::PlusCircle,
    Symbol::MinusCircle,
    Symbol::XMarkCircle,
    Symbol::CheckmarkCircle,
    Symbol::ExclamationMarkCircle,
    Symbol::QuestionMarkCircle,
    Symbol::EllipsisCircle,
    Symbol::Accessibility,
    Symbol::ExclamationMarkTriangle,
    Symbol::SquareAndArrowUp,
    Symbol::SquareAndArrowDown,
    Symbol::MenubarDockRectangle,
    Symbol::CapsuleOnRectangle,
    Symbol::CapsuleOnRectangleFill,
    Symbol::Captions,
    Symbol::XMarkCircleFill,
    Symbol::PlusCircleFill,
    Symbol::MinusCircleFill,
    Symbol::CheckmarkCircleFill,
    Symbol::ExclamationMarkCircleFill,
    Symbol::InfoCircleFill,
    Symbol::QuestionMarkCircleFill,
    Symbol::ExclamationMarkTriangleFill,
    Symbol::Clock,
    Symbol::ArrowUpArrowDown,
    Symbol::ListBullet,
    Symbol::SquareOnSquare,
    Symbol::DocOnDoc,
];

struct Cells(Vec<Image>);

impl App for Cells {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        let mut root = rect().expanded().background((255, 255, 255));
        for (index, image) in self.0.iter().enumerate() {
            root = root.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left((index % ICON_COLUMNS) as f32 * ICON_CELL)
                            .top((index / ICON_COLUMNS) as f32 * ICON_CELL),
                    )
                    .width(Size::px(ICON_CELL))
                    .height(Size::px(ICON_CELL))
                    .center()
                    .child(image.clone()),
            );
        }
        root
    }
}

fn black(symbol: Symbol) -> Image {
    Image::system(symbol)
        .color(Rgba::rgb8(0, 0, 0))
        .size(ICON_SIZE)
}

struct Sheet {
    width: usize,
    pixels: Vec<u8>,
}

impl Sheet {
    fn render(images: Vec<Image>) -> Self {
        let rows = images.len().div_ceil(ICON_COLUMNS);
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Cells(images)),
            Size2D::new(
                ICON_COLUMNS as f32 * ICON_CELL * ICON_PIXELS as f32,
                rows as f32 * ICON_CELL * ICON_PIXELS as f32,
            ),
            |_| {},
            ICON_PIXELS as f64,
        );
        test.sync_and_update();
        test.sync_and_update();
        let (width, _, pixels) = crate::image::decode(test.render().as_bytes());
        Self { width, pixels }
    }

    /// Black on white, so the coverage of a pixel is the distance of its red channel from white.
    fn coverage(&self, cell: usize, x: usize, y: usize) -> u8 {
        let side = ICON_CELL as usize * ICON_PIXELS;
        let column = (cell % ICON_COLUMNS) * side + x;
        let row = (cell / ICON_COLUMNS) * side + y;
        255 - self.pixels[(row * self.width + column) * 4]
    }

    fn cell(&self, cell: usize) -> impl Iterator<Item = (usize, usize, u8)> + '_ {
        let side = ICON_CELL as usize * ICON_PIXELS;
        (0..side)
            .flat_map(move |y| (0..side).map(move |x| (x, y)))
            .map(move |(x, y)| (x, y, self.coverage(cell, x, y)))
    }
}

#[test]
#[ignore]
fn appkit_icon_lab() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let rows = ICONS.len().div_ceil(ICON_COLUMNS);
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Cells(
            ICONS.iter().map(|(_, symbol)| black(*symbol)).collect(),
        )),
        Size2D::new(
            ICON_COLUMNS as f32 * ICON_CELL * ICON_PIXELS as f32,
            rows as f32 * ICON_CELL * ICON_PIXELS as f32,
        ),
        |_| {},
        ICON_PIXELS as f64,
    );
    test.sync_and_update();
    test.sync_and_update();
    fs::write(out.join("lab-icons.png"), test.render().as_bytes()).unwrap();
    let names: Vec<_> = ICONS.iter().map(|(name, _)| *name).collect();
    fs::write(out.join("lab-icons.txt"), names.join("\n")).unwrap();
}

#[test]
fn a_symbol_is_not_cut_by_its_frame() {
    let sheet = Sheet::render(ICONS.iter().map(|(_, symbol)| black(*symbol)).collect());
    let margin = (ICON_CELL as usize - ICON_SIZE as usize) / 2 * ICON_PIXELS;
    let frame = margin..ICON_CELL as usize * ICON_PIXELS - margin;
    let cut: Vec<_> = ICONS
        .iter()
        .enumerate()
        .filter_map(|(index, (name, _))| {
            sheet
                .cell(index)
                .find(|(x, y, coverage)| {
                    let outside = !frame.contains(x) || !frame.contains(y);
                    let on_the_edge = [x, y]
                        .iter()
                        .any(|axis| **axis == frame.start || **axis == frame.end - 1);
                    let allowed = match (outside, on_the_edge) {
                        (true, _) => 0,
                        (false, true) => 160,
                        (false, false) => 255,
                    };
                    *coverage > allowed
                })
                .map(|(x, y, coverage)| format!("{name} at ({x}, {y}): {coverage}"))
        })
        .collect();
    assert!(cut.is_empty(), "the frame cuts the ink of {cut:?}");
}

#[test]
fn a_weight_and_a_scale_set_the_stroke_in_the_symbol_tokens() {
    let weights = [
        FontWeight::UltraLight,
        FontWeight::Thin,
        FontWeight::Light,
        FontWeight::Regular,
        FontWeight::Medium,
        FontWeight::Semibold,
        FontWeight::Bold,
        FontWeight::Heavy,
        FontWeight::Black,
    ];
    let scales = [
        (ImageScale::Small, SYMBOL_WEIGHT_STROKE_SMALL),
        (ImageScale::Medium, SYMBOL_WEIGHT_STROKE_MEDIUM),
        (ImageScale::Large, SYMBOL_WEIGHT_STROKE_LARGE),
    ];
    let cases: Vec<_> = scales
        .iter()
        .enumerate()
        .flat_map(|(scale, (image_scale, strokes))| {
            weights
                .iter()
                .enumerate()
                .map(move |(weight, font_weight)| {
                    (scale, *image_scale, strokes[weight], *font_weight)
                })
        })
        .collect();
    let sheet = Sheet::render(
        cases
            .iter()
            .map(|(_, scale, _, weight)| {
                black(Symbol::Minus)
                    .image_scale(*scale)
                    .font_weight(*weight)
            })
            .collect(),
    );
    let middle = ICON_CELL as usize * ICON_PIXELS / 2;
    let off: Vec<_> = cases
        .iter()
        .enumerate()
        .filter_map(|(index, (scale, image_scale, strokes, weight))| {
            let drawn: f32 = (0..ICON_CELL as usize * ICON_PIXELS)
                .map(|y| f32::from(sheet.coverage(index, middle, y)) / 255.0)
                .sum::<f32>()
                / ICON_PIXELS as f32;
            let expected =
                SYMBOL.stroke * strokes * ICON_SIZE * SYMBOL_SCALE_SIZE[*scale] / SYMBOL.grid;
            ((drawn - expected).abs() > 0.05 + 0.05 * expected)
                .then(|| format!("{weight:?} {image_scale:?}: {drawn} px, tokens {expected} px"))
        })
        .collect();
    assert!(off.is_empty(), "bars of the wrong thickness: {off:?}");
}

#[test]
fn hierarchical_mode_draws_the_layers_at_the_measured_alpha() {
    let sheet = Sheet::render(
        HIERARCHICAL
            .iter()
            .map(|symbol| black(*symbol).symbol_rendering_mode(SymbolRenderingMode::Hierarchical))
            .collect(),
    );
    let levels = SYMBOL_HIERARCHY.map(|alpha| (alpha * 255.0).round() as u8);
    let flat: Vec<_> = HIERARCHICAL
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            let near = |level: u8| {
                sheet
                    .cell(*index)
                    .filter(|(_, _, coverage)| coverage.abs_diff(level) <= 2)
                    .count()
            };
            near(levels[0]) < 10 || near(levels[1]) + near(levels[2]) < 40
        })
        .map(|(_, symbol)| symbol)
        .collect();
    assert!(
        flat.is_empty(),
        "no full and dimmed layers in hierarchical mode: {flat:?}"
    );
}
