use std::{fs, path::PathBuf};

use bk::style::{Palette, text};
use bk::styles::{OutlineViewStyle, Role, ShapeStyle, StateValue, TableStyle};
use bk::symbol::Symbol;
use bk::views::form::{Form, Row, Section};
use bk::views::group::{DisclosureGroup, GroupBox};
use bk::views::list::List;
use bk::views::split_panes::SplitPanes;
use bk::views::table::{SortOrder, Table};
use bk::views::tree::{OutlineView, TreeNode};
use ctdesign::BODY;
use freya_core::prelude::*;
use torin::prelude::{Direction, Size};

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

fn containers() -> Vec<(&'static str, Builder)> {
    vec![
        ("split-v-thin", || panes(Direction::Horizontal)),
        ("swiftui-hsplit", || panes(Direction::Horizontal)),
        ("split-h-thin", || panes(Direction::Vertical)),
        ("swiftui-vsplit", || panes(Direction::Vertical)),
    ]
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
