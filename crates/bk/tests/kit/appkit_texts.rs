use std::{fs, path::PathBuf};

use bk::color_picker::ColorPicker;
use bk::combo_box::ComboBox;
use bk::controls::{SecureField, TextField};
use bk::date::{CivilDate, CivilDateTime};
use bk::date_picker::{DateElements, DatePicker};
use bk::image_well::ImageWell;
use bk::password::PasswordField;
use bk::path_control::PathControl;
use bk::symbol::Symbol;
use bk::text_editor::TextEditor;
use bk::token_field::TokenField;
use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};

use super::appkit::{
    Builder, controls, layout, render_scene, scene_content, scene_tag, state_string, system_blue,
};

const FOCUSABLE: [&str; 9] = [
    "tf-empty",
    "tf-filled",
    "search",
    "search-text",
    "combo",
    "secure",
    "token",
    "datepicker",
    "textview",
];

fn date_time() -> CivilDateTime {
    CivilDateTime::new(CivilDate::new(2027, 1, 15), 10, 7)
}

fn texts() -> Vec<(&'static str, Builder)> {
    let field = || {
        TextField::new(state_string(), "Placeholder")
            .width(Size::px(140.0))
            .into_element()
    };
    vec![
        ("tf-empty", field),
        ("tf-filled", || {
            TextField::new(use_state(|| "Query".to_string()), "Placeholder")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("tf-square", || {
            TextField::new(use_state(|| "Square".to_string()), "Placeholder")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("tf-rounded", || {
            TextField::new(use_state(|| "Rounded".to_string()), "Placeholder")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("secure-empty", || {
            SecureField::new(use_state(PasswordField::default), "Password")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("token-empty", || {
            TokenField::new(use_state(Vec::new), "Tokens")
                .width(Size::px(160.0))
                .into_element()
        }),
        ("search-text", || {
            bk::controls::SearchField::new(use_state(|| "Query".to_string()), "Search")
                .width(Size::px(160.0))
                .into_element()
        }),
        ("combo-empty", || {
            ComboBox::new(state_string(), "Choose")
                .item("One")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("textview", || {
            rect()
                .width(Size::px(180.0))
                .child(
                    TextEditor::new(
                        use_state(|| {
                            "Some text in a text view that wraps onto several lines for the measurement."
                                .to_string()
                        }),
                        "",
                    )
                    .height(Size::px(72.0)),
                )
                .into_element()
        }),
        ("datetime", || {
            rect()
                .margin(Gaps::new(3.0, 0.0, 0.0, 0.0))
                .child(DatePicker::new("Date", date_time()).elements(DateElements::DateAndTime))
                .into_element()
        }),
        ("time", || {
            rect()
                .margin(Gaps::new(3.0, 0.0, 0.0, 0.0))
                .child(DatePicker::new("Date", date_time()).elements(DateElements::Time))
                .into_element()
        }),
        ("well-default", || {
            ColorPicker::new("Color", system_blue()).into_element()
        }),
        ("path-standard", || {
            PathControl::new("Path")
                .item(Symbol::Folder, "Users")
                .item(Symbol::Folder, "Shared")
                .item(Symbol::Folder, "Documents")
                .item(Symbol::Folder, "Notes")
                .into_element()
        }),
        ("imagewell-gray-empty", || {
            ImageWell::new("Image", use_state(|| None::<PathBuf>)).into_element()
        }),
    ]
}

fn builders() -> Vec<(&'static str, Builder)> {
    let mut all = texts();
    let own: Vec<&str> = all.iter().map(|(name, _)| *name).collect();
    all.extend(
        controls()
            .into_iter()
            .filter(|(name, _)| !own.contains(name)),
    );
    all
}

#[test]
#[ignore]
fn appkit_texts() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        for black in [false, true] {
            let placed = builders()
                .into_iter()
                .filter_map(|(name, build)| positions.get(name).map(|&(x, y, ..)| (x, y, build)))
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
}

#[test]
#[ignore]
fn appkit_focus() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for (name, build) in builders() {
        let Some(&(x, y, w, h)) = positions.get(name) else {
            continue;
        };
        if !FOCUSABLE.contains(&name) {
            continue;
        }
        for dark in [false, true] {
            for black in [false, true] {
                fs::write(
                    out.join(format!(
                        "kit{}-focus-{name}-{}-{}.png",
                        scene_tag(),
                        if dark { "dark" } else { "light" },
                        if black { "black" } else { "white" }
                    )),
                    render_scene(
                        vec![(x, y, build)],
                        dark,
                        black,
                        scene_content(),
                        Some((x + w / 2.0, y + h / 2.0)),
                    ),
                )
                .unwrap();
            }
        }
    }
}
