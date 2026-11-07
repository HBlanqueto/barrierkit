use std::path::{Path, PathBuf};

use freya_core::prelude::*;
use torin::prelude::{Area, Size};

use crate::controls::{dimmed, inner_border, shadowed};
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::media::use_artwork;
use crate::styles::{ImageWellStyle, LayoutDirection, Paints, Theme, resolve};

#[derive(PartialEq)]
pub struct ImageWell {
    label: &'static str,
    image: State<Option<PathBuf>>,
    style: Theme,
}

impl ImageWell {
    pub fn new(label: &'static str, image: State<Option<PathBuf>>) -> Self {
        Self {
            label,
            image,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(ImageWellStyle) -> ImageWellStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: ImageWellStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(control_state(interaction, false), style.effect);
        let mut image = self.image;
        let mut area = use_state(Area::default);
        let mut targeted = use_state(|| false);
        let border = if *targeted.read() {
            Some(&style.drop_border)
        } else {
            style.border.as_ref()
        };
        let size = style.size;
        let radius = style
            .shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let inset = style.inset;
        let inner = CornerRadius {
            top_left: (radius.top_left - inset).max(0.0),
            top_right: (radius.top_right - inset).max(0.0),
            bottom_right: (radius.bottom_right - inset).max(0.0),
            bottom_left: (radius.bottom_left - inset).max(0.0),
            ..radius
        };
        let element = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Image)
            .a11y_alt(self.label)
            .width(Size::px(size))
            .height(Size::px(size))
            .padding(inset)
            .corner_radius(radius)
            .overflow(Overflow::Clip)
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(border, |el, border| {
                el.border(inner_border(&paints, border))
            })
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
                    .child(WellImage {
                        path,
                        corner_radius: inner,
                    })
            }));
        interaction.focus_ring(
            dimmed(shadowed(element, &style.shadows), style.opacity),
            radius.top_left,
        )
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
    corner_radius: CornerRadius,
}

impl Component for WellImage {
    fn render(&self) -> impl IntoElement {
        let artwork = use_artwork(self.path.clone());
        rect()
            .expanded()
            .corner_radius(self.corner_radius)
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
