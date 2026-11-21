use std::{fs, path::PathBuf};

use bk::styles::ImageStyle;
use bk::symbol::{Image, Symbol};
use bk::theme;
use ctdesign::Rgba;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Position, Size, Size2D};

const ICONS: [(&str, Symbol); 56] = [
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
];

const ICON_SIZE: f32 = 20.0;
const ICON_STROKE: f32 = 1.0;
const ICON_CELL: f32 = 32.0;
const ICON_COLUMNS: usize = 8;

struct IconLab;

impl App for IconLab {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        let mut root = rect().expanded().background((255, 255, 255));
        for (index, (_, symbol)) in ICONS.iter().enumerate() {
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
                    .child(
                        Image::system(*symbol)
                            .color(Rgba::rgb8(0, 0, 0))
                            .style(|s| ImageStyle {
                                size: ICON_SIZE,
                                stroke: ICON_STROKE,
                                ..s
                            }),
                    ),
            );
        }
        root
    }
}

#[test]
#[ignore]
fn appkit_icon_lab() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let rows = ICONS.len().div_ceil(ICON_COLUMNS);
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(IconLab),
        Size2D::new(
            ICON_COLUMNS as f32 * ICON_CELL * 4.0,
            rows as f32 * ICON_CELL * 4.0,
        ),
        |_| {},
        4.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    fs::write(out.join("lab-icons.png"), test.render().as_bytes()).unwrap();
    let names: Vec<_> = ICONS.iter().map(|(name, _)| *name).collect();
    fs::write(out.join("lab-icons.txt"), names.join("\n")).unwrap();
}
