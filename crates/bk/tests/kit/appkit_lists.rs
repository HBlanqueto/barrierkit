use std::{fs, path::PathBuf};

use bk::controls::{Checkbox, TextField};
use bk::style::{Palette, text};
use bk::styles::{
    FormStyle, OutlineViewStyle, Role, ShapeStyle, StateValue, TabType, TabViewStyle, TableStyle,
};
use bk::symbol::Symbol;
use bk::views::form::{Form, Row, Section};
use bk::views::group::{DisclosureGroup, GroupBox};
use bk::views::list::List;
use bk::views::split_panes::SplitPanes;
use bk::views::table::{SortOrder, Table};
use bk::views::tabs::TabView;
use bk::views::tree::{OutlineView, TreeNode};
use ctdesign::{BODY, ControlSize};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use super::appkit::{Builder, layout, render_scene, scene_content, scene_tag};

const NAMES: [&str; 5] = [
    "adr-0047.pdf",
    "notas.txt",
    "portada.png",
    "datos.csv",
    "music.mp3",
];

fn label(content: &'static str) -> Element {
    text(content, BODY, Palette::current().label).into_element()
}

fn emphasized(style: TableStyle) -> TableStyle {
    TableStyle {
        selected_row_background: StateValue::new(ShapeStyle::Role(Role::SelectedContent)),
        selected_label: StateValue::new(ShapeStyle::Role(Role::OnSelectedContent)),
        ..style
    }
}

fn list(restyle: fn(TableStyle) -> TableStyle) -> Element {
    rect()
        .width(Size::px(200.0))
        .height(Size::px(140.0))
        .child(
            List::new("Files", NAMES.map(String::from).to_vec())
                .selected(vec![1])
                .style(restyle),
        )
        .into_element()
}

fn lists() -> Vec<(&'static str, Builder)> {
    vec![
        ("list-automatic", || list(|style| style)),
        ("list-inset", || list(|style| style)),
        ("list-striped", || list(TableStyle::striped)),
        ("list-plain", || list(|_| TableStyle::plain_list())),
        ("list-bordered", || list(|_| TableStyle::bordered_list())),
        ("table", || table(|style| style)),
        ("outline-group", || outline(false, |style| style)),
        ("outline-open", || outline(true, |style| style)),
        ("form-grouped", || {
            rect()
                .width(Size::px(360.0))
                .height(Size::px(260.0))
                .child(
                    Form::new()
                        .section(
                            Section::titled("Section")
                                .row(Row::toggle("Toggle", true, |_| {}))
                                .row(Row::value("Name", "Value"))
                                .row(Row::value("Labeled", "Value")),
                        )
                        .section(Section::new().row(Row::new("Other"))),
                )
                .into_element()
        }),
        ("form-columns", || {
            rect()
                .width(Size::px(360.0))
                .child(
                    Form::new().style(FormStyle::columns).section(
                        Section::new()
                            .row(Row::new("").control(Checkbox::new("Toggle").checked(true)))
                            .row(Row::new("Name").control(TextField::new(
                                use_state(|| String::from("Value")),
                                "Name",
                            )))
                            .row(Row::value("Labeled", "Value")),
                    ),
                )
                .into_element()
        }),
        ("list-sections", || {
            rect()
                .width(Size::px(200.0))
                .height(Size::px(180.0))
                .child(
                    List::new("Files", Vec::new())
                        .section("Favorites", vec![NAMES[0].into(), NAMES[1].into()])
                        .section("Locations", vec![NAMES[3].into(), NAMES[4].into()])
                        .selected(vec![1]),
                )
                .into_element()
        }),
        ("groupbox", || {
            rect()
                .width(Size::px(130.0))
                .child(
                    GroupBox::new(Some("Title")).child(
                        rect()
                            .width(Size::px(120.0))
                            .height(Size::px(30.0))
                            .center()
                            .child(label("Content")),
                    ),
                )
                .into_element()
        }),
        ("disclosure-closed", || {
            rect()
                .width(Size::px(160.0))
                .child(DisclosureGroup::new("Group").child(label("Child")))
                .into_element()
        }),
        ("disclosure", || {
            rect()
                .width(Size::px(160.0))
                .child(
                    DisclosureGroup::new("Group")
                        .expanded(true)
                        .child(label("Child")),
                )
                .into_element()
        }),
    ]
}

fn emphasized_outline(style: OutlineViewStyle) -> OutlineViewStyle {
    OutlineViewStyle {
        selected_row_background: StateValue::new(ShapeStyle::Role(Role::SelectedContent)),
        selected_label: StateValue::new(ShapeStyle::Role(Role::OnSelectedContent)),
        selected_icon: StateValue::new(ShapeStyle::Role(Role::OnSelectedContent)),
        selected_chevron: StateValue::new(ShapeStyle::Role(Role::OnSelectedContent)),
        ..style
    }
}

fn outline(open: bool, restyle: fn(OutlineViewStyle) -> OutlineViewStyle) -> Element {
    let folder = |title: &str| TreeNode::new(title).icon(Symbol::Folder);
    rect()
        .width(Size::px(200.0))
        .height(Size::px(if open { 160.0 } else { 140.0 }))
        .child(
            OutlineView::new(
                "Places",
                vec![
                    folder("Favorites")
                        .expanded(open)
                        .child(folder("AirDrop"))
                        .child(folder("Recents")),
                    folder("Locations").expanded(open).child(folder("Desktop")),
                ],
            )
            .selected(vec![if open { vec![0, 0] } else { vec![1] }])
            .style(restyle),
        )
        .into_element()
}

fn table(restyle: fn(TableStyle) -> TableStyle) -> Element {
    let row = |name: &str, kind: &str| vec![name.to_string(), kind.to_string()];
    rect()
        .width(Size::px(300.0))
        .height(Size::px(170.0))
        .child(
            Table::new("Files", vec!["Name", "Kind"])
                .rows(vec![
                    row("adr-0047.pdf", "PDF"),
                    row("notas.txt", "Text"),
                    row("portada.png", "Image"),
                    row("datos.csv", "Table"),
                    row("music.mp3", "Audio"),
                ])
                .sort(Some(SortOrder {
                    column: 0,
                    ascending: true,
                }))
                .selected(vec![1])
                .style(restyle),
        )
        .into_element()
}

const FILES: [(&str, &str, Symbol); 6] = [
    ("adr-0047.pdf", "PDF", Symbol::Document),
    ("notas.txt", "Text", Symbol::Document),
    ("portada.png", "Image", Symbol::Photo),
    ("datos.csv", "Table", Symbol::Document),
    ("music.mp3", "Audio", Symbol::MusicNote),
    ("video.mov", "Movie", Symbol::Film),
];

fn unstriped(style: TableStyle) -> TableStyle {
    TableStyle {
        alternate_row_background: StateValue::new(ShapeStyle::Clear),
        ..style
    }
}

fn classic(columns: Vec<&'static str>, restyle: fn(TableStyle) -> TableStyle) -> Element {
    let two = columns.len() > 1;
    rect()
        .width(Size::px(300.0))
        .height(Size::px(170.0))
        .child(
            Table::new("Files", columns)
                .rows(
                    FILES
                        .iter()
                        .map(|(name, kind, _)| {
                            let mut row = vec![name.to_string()];
                            if two {
                                row.push(kind.to_string());
                            }
                            row
                        })
                        .collect(),
                )
                .icons(FILES.iter().map(|file| file.2).collect())
                .scroll_bar(false)
                .selected(vec![1])
                .style(restyle),
        )
        .into_element()
}

fn tables() -> Vec<(&'static str, Builder)> {
    vec![
        ("table-automatic", || {
            classic(vec!["Name", "Kind"], unstriped)
        }),
        ("table-fullwidth", || {
            classic(
                vec!["Name", "Kind"],
                |_| unstriped(TableStyle::full_width()),
            )
        }),
        ("table-inset", || classic(vec!["Name", "Kind"], unstriped)),
        ("table-sourcelist", || {
            classic(vec!["Name"], |_| TableStyle::source_list())
        }),
        ("table-plain", || {
            classic(vec!["Name", "Kind"], |_| TableStyle::plain())
        }),
        ("table-fullwidth-striped", || {
            classic(vec!["Name", "Kind"], |_| TableStyle::full_width())
        }),
        ("table-small", || {
            classic(vec!["Name", "Kind"], |style| {
                unstriped(style).row_size(ControlSize::Small)
            })
        }),
        ("table-medium", || {
            classic(vec!["Name", "Kind"], |style| {
                unstriped(style).row_size(ControlSize::Regular)
            })
        }),
        ("table-large", || {
            classic(vec!["Name", "Kind"], |style| {
                unstriped(style).row_size(ControlSize::Large)
            })
        }),
        ("outline-inset", || {
            classic_outline(|_| OutlineViewStyle::classic())
        }),
        ("outline-fullwidth", || {
            classic_outline(|_| OutlineViewStyle::full_width())
        }),
    ]
}

fn classic_outline(restyle: fn(OutlineViewStyle) -> OutlineViewStyle) -> Element {
    let folder = |title: &str| TreeNode::new(title).icon(Symbol::Folder);
    rect()
        .width(Size::px(300.0))
        .height(Size::px(170.0))
        .child(
            OutlineView::new(
                "Places",
                vec![
                    folder("Favorites")
                        .expanded(true)
                        .child(folder("AirDrop"))
                        .child(folder("Recents"))
                        .child(folder("Applications")),
                    folder("Locations")
                        .expanded(true)
                        .child(folder("Desktop"))
                        .child(folder("Documents")),
                ],
            )
            .selected(vec![vec![0, 1]])
            .scroll_bar(false)
            .style(restyle),
        )
        .into_element()
}

fn emphasized_tables() -> Vec<(&'static str, Builder)> {
    vec![("table-inset", || {
        classic(vec!["Name", "Kind"], |style| emphasized(unstriped(style)))
    })]
}

#[test]
#[ignore]
fn appkit_tables() {
    render_family(tables(), emphasized_tables());
}

fn emphasized_lists() -> Vec<(&'static str, Builder)> {
    vec![
        ("list-inset", || list(emphasized)),
        ("list-plain", || {
            list(|_| emphasized(TableStyle::plain_list()))
        }),
        ("list-bordered", || {
            list(|_| emphasized(TableStyle::bordered_list()))
        }),
        ("table", || table(emphasized)),
        ("outline-group", || outline(false, emphasized_outline)),
        ("outline-open", || outline(true, emphasized_outline)),
    ]
}

fn render_family(builders: Vec<(&'static str, Builder)>, emphasized: Vec<(&'static str, Builder)>) {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    let modes: Vec<bool> = match std::env::var("APPKIT_MODE").as_deref() {
        Ok("light") => vec![false],
        Ok("dark") => vec![true],
        _ => vec![false, true],
    };
    for &dark in &modes {
        for black in [false, true] {
            let placed = builders
                .iter()
                .filter_map(|&(name, build)| positions.get(name).map(|&(x, y, ..)| (x, y, build)))
                .collect();
            fs::write(
                out.join(format!(
                    "kit{}-{}-{}.png",
                    scene_tag(),
                    if dark { "dark" } else { "light" },
                    if black { "black" } else { "white" }
                )),
                render_scene(placed, dark, black, scene_content(), None),
            )
            .unwrap();
        }
    }
    for (name, build) in emphasized {
        let Some(&(x, y, ..)) = positions.get(name) else {
            continue;
        };
        for &dark in &modes {
            for black in [false, true] {
                fs::write(
                    out.join(format!(
                        "kit{}-focus-{name}-{}-{}.png",
                        scene_tag(),
                        if dark { "dark" } else { "light" },
                        if black { "black" } else { "white" }
                    )),
                    render_scene(vec![(x, y, build)], dark, black, scene_content(), None),
                )
                .unwrap();
            }
        }
    }
}

fn panes(direction: Direction) -> Element {
    rect()
        .width(Size::px(160.0))
        .height(Size::px(100.0))
        .child(
            SplitPanes::new("Panes", direction)
                .pane(rect().expanded())
                .pane(rect().expanded()),
        )
        .into_element()
}

fn page(title: &'static str) -> Element {
    rect()
        .expanded()
        .main_align(Alignment::End)
        .padding(Gaps::new(0.0, 0.0, 12.0, 12.0))
        .child(
            rect()
                .height(Size::px(17.0))
                .child(text(title, BODY, Palette::current().label)),
        )
        .into_element()
}

fn tabs(tab_type: TabType, select: usize, size: ControlSize) -> Element {
    rect()
        .width(Size::px(220.0))
        .height(Size::px(140.0))
        .child(
            TabView::new("Tabs")
                .tab(Symbol::Folder, "One", page("Page One"))
                .tab(Symbol::Folder, "Two", page("Page Two"))
                .tab(Symbol::Folder, "Three", page("Page Three"))
                .selected(select)
                .size(size)
                .style(move |style: TabViewStyle| TabViewStyle { tab_type, ..style }),
        )
        .into_element()
}

fn containers() -> Vec<(&'static str, Builder)> {
    vec![
        ("split-v-thin", || panes(Direction::Horizontal)),
        ("swiftui-hsplit", || panes(Direction::Horizontal)),
        ("split-h-thin", || panes(Direction::Vertical)),
        ("swiftui-vsplit", || panes(Direction::Vertical)),
        ("tabs-top", || tabs(TabType::Top, 0, ControlSize::Regular)),
        ("tabs-top-second", || {
            tabs(TabType::Top, 1, ControlSize::Regular)
        }),
        ("tabs-bottom", || {
            tabs(TabType::Bottom, 0, ControlSize::Regular)
        }),
        ("tabs-left", || tabs(TabType::Left, 0, ControlSize::Regular)),
        ("tabs-none-bezel", || {
            tabs(TabType::NoTabsBezelBorder, 0, ControlSize::Regular)
        }),
        ("tabs-none-line", || {
            tabs(TabType::NoTabsLineBorder, 0, ControlSize::Regular)
        }),
        ("tabs-none", || {
            tabs(TabType::NoTabsNoBorder, 0, ControlSize::Regular)
        }),
        ("tabs-top-small", || {
            tabs(TabType::Top, 0, ControlSize::Small)
        }),
        ("tabs-top-mini", || tabs(TabType::Top, 0, ControlSize::Mini)),
    ]
}

fn file_names() -> Vec<String> {
    (0..40).map(|row| format!("file-{row}.txt")).collect()
}

fn long_list(restyle: fn(TableStyle) -> TableStyle) -> Element {
    rect()
        .width(Size::px(300.0))
        .height(Size::px(180.0))
        .child(
            List::new("Files", file_names())
                .selected(vec![1])
                .style(restyle),
        )
        .into_element()
}

fn long_table(restyle: fn(TableStyle) -> TableStyle) -> Element {
    let kinds = ["PDF", "Text", "Image"];
    rect()
        .width(Size::px(300.0))
        .height(Size::px(180.0))
        .child(
            Table::new("Files", vec!["Name", "Kind"])
                .rows(
                    file_names()
                        .into_iter()
                        .enumerate()
                        .map(|(row, name)| vec![name, kinds[row % 3].to_string()])
                        .collect(),
                )
                .selected(vec![1])
                .style(restyle),
        )
        .into_element()
}

fn scrollers() -> Vec<(&'static str, Builder)> {
    vec![
        ("swiftui-list-inset", || long_list(|style| style)),
        ("swiftui-list-plain", || {
            long_list(|_| TableStyle::plain_list())
        }),
        ("swiftui-table", || long_table(|style| style)),
        ("appkit-table-inset", || {
            long_table(|style| emphasized(unstriped(style)))
        }),
        ("appkit-table-fullwidth", || {
            long_table(|_| unstriped(TableStyle::full_width()))
        }),
        ("appkit-list-sourcelist", || {
            long_list(|_| TableStyle::source_list())
        }),
    ]
}

#[test]
#[ignore]
fn appkit_scrollers() {
    render_family(scrollers(), Vec::new());
}

#[test]
#[ignore]
fn appkit_lists() {
    render_family(lists(), emphasized_lists());
}

#[test]
#[ignore]
fn appkit_containers() {
    render_family(containers(), Vec::new());
}
