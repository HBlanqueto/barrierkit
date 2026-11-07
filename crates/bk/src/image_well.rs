use std::path::{Path, PathBuf};

use ctdesign::{HAIRLINE, IMAGE_WELL};
use freya_core::prelude::*;
use torin::prelude::{Area, Size};

use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::media::use_artwork;
use crate::style::Palette;
use crate::theme::color;

#[derive(PartialEq)]
pub struct ImageWell {
    label: &'static str,
    image: State<Option<PathBuf>>,
}

impl ImageWell {
    pub fn new(label: &'static str, image: State<Option<PathBuf>>) -> Self {
        Self { label, image }
    }
}

pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["png", "jpg", "jpeg", "gif", "webp", "bmp"]
                .contains(&extension.to_ascii_lowercase().as_str())
        })
}

impl Component for ImageWell {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let mut image = self.image;
        let mut area = use_state(Area::default);
        let mut targeted = use_state(|| false);
        let border = if *targeted.read() {
            Border::new()
                .width(IMAGE_WELL.drop_border)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.accent))
        } else {
            Border::new()
                .width(HAIRLINE)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.separator))
        };
        let element = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Image)
            .a11y_alt(self.label)
            .width(Size::px(IMAGE_WELL.size))
            .height(Size::px(IMAGE_WELL.size))
            .padding(IMAGE_WELL.inset)
            .corner_radius(IMAGE_WELL.radius)
            .overflow(Overflow::Clip)
            .background(color(palette.content_background))
            .border(border)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_global_file_hover(move |event: Event<FileEventData>| {
                let over = event.file_path.as_deref().is_none_or(is_image)
                    && area.peek().contains(event.cursor.to_f32());
                targeted.set_if_modified(over);
            })
            .on_global_file_hover_cancelled(move |_| targeted.set_if_modified(false))
            .on_file_drop(move |event: Event<FileEventData>| {
                targeted.set_if_modified(false);
                if let Some(path) = event.file_path.clone().filter(|path| is_image(path)) {
                    image.set(Some(path));
                }
            })
            // on_press and on_key_down share EventName::KeyDown, so the keyboard half lives here.
            .on_pointer_press(move |event: Event<PointerEventData>| {
                if event.is_primary() {
                    choose(image);
                }
            })
            .on_key_down(move |event: Event<KeyboardEventData>| {
                if event.is_press_event() {
                    choose(image);
                } else if matches!(
                    event.key,
                    Key::Named(NamedKey::Delete | NamedKey::Backspace)
                ) {
                    image.set(None);
                }
            })
            .maybe_child(self.image.read().clone().map(|path| {
                rect()
                    .key(path.to_string_lossy().into_owned())
                    .expanded()
                    .child(WellImage { path })
            }));
        interaction.focus_ring(element, IMAGE_WELL.radius)
    }
}

fn choose(mut image: State<Option<PathBuf>>) {
    spawn(async move {
        if let Some(path) = crate::portal::open_files(text!("choose-image"), false, false)
            .await
            .and_then(|paths| paths.into_iter().next())
        {
            image.set(Some(path));
        }
    });
}

#[derive(PartialEq)]
struct WellImage {
    path: PathBuf,
}

impl Component for WellImage {
    fn render(&self) -> impl IntoElement {
        let artwork = use_artwork(self.path.clone());
        rect()
            .expanded()
            .corner_radius(IMAGE_WELL.radius - IMAGE_WELL.inset)
            .overflow(Overflow::Clip)
            .maybe_child(
                artwork
                    .read()
                    .clone()
                    .map(|handle| image(handle).expanded().aspect_ratio(AspectRatio::Min)),
            )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_image_extensions_are_accepted_in_any_case() {
        assert!(super::is_image("/tmp/Foto.JPG".as_ref()));
        assert!(!super::is_image("/tmp/notas.txt".as_ref()));
        assert!(!super::is_image("/tmp/sin-extension".as_ref()));
    }
}
