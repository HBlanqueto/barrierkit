//! Sidebar example: the seed of Master Tengen App Show.
//!
//! Run with `./dev.sh example sidebar`.

use barrier_kit::controls::{ToolButton, ToolGroup};
use barrier_kit::platform::{Decorations, WindowOptions, run_window};
use barrier_kit::sidebar::{Sidebar, SidebarItem, SidebarLayout, SidebarSection};
use barrier_kit::titlebar::{Titlebar, resize_edges};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Direction, Size};

const ICON_FOLDER: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\"><rect x=\"3\" y=\"3\" width=\"10\" height=\"10\" fill=\"currentColor\"/></svg>";
const ICON_STAR: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\"><circle cx=\"8\" cy=\"8\" r=\"5\" fill=\"currentColor\"/>             </svg>";
const ICON_ADD: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\"><path d=\"M8 3v10M3 8h10\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\"/></svg>";
const ICON_REMOVE: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\"><path d=\"M3 8h10\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\"/></svg>";

const LONG_TITLE: &str =
    "Master Tengen App Show — una ventana con un título deliberadamente largo que se recorta";

fn rgba(color: vow_design::Rgba) -> Color {
    let [r, g, b, a] = color.to_rgba8();
    Color::from_argb(a, r, g, b)
}

struct Show;

impl App for Show {
    fn render(&self) -> impl IntoElement {
        let resolved = barrier_kit::theme::resolved();
        let window_text = vow_appearance::palette(resolved.scheme).text.focused;
        let section_ink = resolved.sidebar.palette.item_ink;

        let sidebar = Sidebar::new()
            .traffic_lights(true)
            .child(
                SidebarSection::new("Favoritos")
                    .expanded(true)
                    .child(SidebarItem::new("Documentos").icon(ICON_FOLDER))
                    .child(SidebarItem::new("Destacados").icon(ICON_STAR)),
            )
            .child(
                SidebarSection::new("Contenido alto").expanded(true).child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(80.0))
                        .center()
                        .child(
                            label()
                                .text("Un hijo que no es SidebarItem")
                                .color(rgba(section_ink)),
                        ),
                ),
            );

        let content = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(
                Titlebar::new().with_sidebar(true).title(LONG_TITLE).left(
                    ToolGroup::new()
                        .child(ToolButton::new("Añadir", ICON_ADD))
                        .child(ToolButton::new("Quitar", ICON_REMOVE)),
                ),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .center()
                    .child(
                        label()
                            .text("Contenido de la ventana")
                            .color(rgba(window_text)),
                    ),
            );

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(SidebarLayout::new(sidebar, content))
            .child(resize_edges())
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    run_window(
        AppComponent::new(Show),
        WindowOptions::new("Master Tengen App Show", "sidebar")
            .decorations(Decorations::Client)
            .size((720, 480)),
    )
    .expect("sidebar example");
}
