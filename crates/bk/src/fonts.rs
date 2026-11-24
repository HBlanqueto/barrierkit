use std::{borrow::Cow, cell::RefCell, collections::HashMap};

use freya_core::integration::default_fonts;
use freya_engine::prelude::{
    FontArguments, FontCollection, FontMgr, FontStyle, SkData, Slant, TypefaceFontProvider,
    VariationCoordinate, VariationPosition, Weight, Width,
};

use crate::platform::OPTICAL_SIZING;

type Optical = (Cow<'static, str>, Option<f32>);

thread_local! {
    static FONTS: RefCell<Option<(FontCollection, FontMgr)>> = const { RefCell::new(None) };
    static OWN: RefCell<Option<TypefaceFontProvider>> = const { RefCell::new(None) };
    static OPTICAL_FACES: RefCell<TypefaceFontProvider> = RefCell::new(TypefaceFontProvider::new());
    // shortcut: one entry per (family, size), never evicted; fine for the type scale, bound it if apps animate font sizes.
    static OPTICAL: RefCell<HashMap<(String, u32), Optical>> = RefCell::new(HashMap::new());
}

fn with_fonts<T>(f: impl FnOnce(&FontCollection, &FontMgr) -> T) -> T {
    FONTS.with(|slot| {
        let mut slot = slot.borrow_mut();
        let (collection, manager) = slot.get_or_insert_with(|| {
            let mut collection = FontCollection::new();
            let manager = FontMgr::custom_empty().unwrap_or_default();
            collection.set_default_font_manager(FontMgr::default(), None);
            collection.set_dynamic_font_manager(manager.clone());
            collection.paragraph_cache_mut().turn_on(false);
            (collection, manager)
        });
        f(collection, manager)
    })
}

pub(crate) fn collection() -> FontCollection {
    with_fonts(|collection, _| collection.clone())
}

pub(crate) fn manager() -> FontMgr {
    with_fonts(|_, manager| manager.clone())
}

pub fn register_font(bytes: &[u8]) -> bool {
    let Some(typeface) = FontMgr::default().new_from_data(SkData::new_copy(bytes), None) else {
        return false;
    };
    OWN.with(|own| {
        let mut own = own.borrow_mut();
        let provider = own.get_or_insert_with(TypefaceFontProvider::new);
        provider.register_typeface(typeface, None::<&str>);
        let manager: FontMgr = provider.clone().into();
        with_fonts(|collection, _| {
            collection.clone().set_asset_font_manager(Some(manager));
        });
    });
    OPTICAL.with(|optical| optical.borrow_mut().clear());
    OPTICAL_FACES.with(|provider| *provider.borrow_mut() = TypefaceFontProvider::new());
    true
}

fn managers() -> Vec<FontMgr> {
    let own = OWN.with(|own| own.borrow().clone().map(FontMgr::from));
    std::iter::once(FontMgr::default()).chain(own).collect()
}

pub(crate) fn family_names() -> Vec<String> {
    let mut names: Vec<String> = managers()
        .iter()
        .flat_map(|manager| manager.family_names().collect::<Vec<_>>())
        .collect();
    names.sort_by_key(|name| name.to_lowercase());
    names.dedup();
    names
}

pub(crate) fn styles(family: &str) -> Vec<FontStyle> {
    managers()
        .iter()
        .flat_map(|manager| {
            let mut set = manager.match_family(family);
            (0..set.count())
                .map(|index| set.style(index).0)
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(crate) fn fixed_pitch(family: &str) -> bool {
    managers().iter().any(|manager| {
        manager
            .match_family(family)
            .match_style(FontStyle::normal())
            .is_some_and(|typeface| typeface.is_fixed_pitch())
    })
}

/// The family to ask for and the letter spacing, if any, that lay `family` out at `size` as CoreText does: at its optical size and with its `trak` tracking.
pub(crate) fn optical(family: &str, size: f32) -> Optical {
    if !OPTICAL_SIZING {
        return (family.to_owned().into(), None);
    }
    OPTICAL.with(|optical| {
        optical
            .borrow_mut()
            .entry((family.to_owned(), size.to_bits()))
            .or_insert_with(|| at_size(family, size))
            .clone()
    })
}

// Skia's CoreText port pins the optical size of a face and turns its `trak` table off (SkCTFontCreateExactCopy.cpp).
fn at_size(family: &str, size: f32) -> Optical {
    let registered = OWN.with(|own| {
        own.borrow()
            .as_ref()
            .is_some_and(|own| own.match_family(family).count() > 0)
    });
    if registered {
        return (family.to_owned().into(), None);
    }
    let system = FontMgr::default();
    let resolved = std::iter::once(Cow::Owned(family.to_owned()))
        .chain(default_fonts())
        .find_map(|name| {
            let face = system.match_family_style(&name, FontStyle::normal())?;
            Some((name, face))
        });
    let Some((name, face)) = resolved else {
        return (family.to_owned().into(), None);
    };
    let spacing = face
        .copy_table_data(u32::from_be_bytes(*b"trak"))
        .zip(face.units_per_em())
        .and_then(|(table, units_per_em)| tracking(table.as_bytes(), units_per_em, size));
    let optical_axis = face.variation_design_parameters().and_then(|axes| {
        axes.into_iter()
            .find(|axis| axis.tag == VariationCoordinate::opsz)
    });
    if !optical_axis.is_some_and(|axis| size > axis.min) {
        return (family.to_owned().into(), spacing);
    }
    let alias = format!("{name} {size}");
    let coordinates = [VariationCoordinate {
        axis: VariationCoordinate::opsz,
        value: size,
    }];
    let arguments = FontArguments::new().set_variation_design_position(VariationPosition {
        coordinates: &coordinates,
    });
    // Off these weights CoreText stops matching ".AppleSystemUIFont" by name for the rest of the process.
    OPTICAL_FACES.with(|provider| {
        let mut provider = provider.borrow_mut();
        for weight in (100..=900).step_by(100) {
            for slant in [Slant::Upright, Slant::Italic] {
                let style = FontStyle::new(Weight::from(weight), Width::NORMAL, slant);
                if let Some(clone) = system
                    .match_family_style(&name, style)
                    .and_then(|face| face.clone_with_arguments(&arguments))
                {
                    provider.register_typeface(clone, Some(alias.as_str()));
                }
            }
        }
        let manager: FontMgr = provider.clone().into();
        with_fonts(|collection, _| collection.clone().set_dynamic_font_manager(manager));
    });
    (alias.into(), spacing)
}

// The track 0 row of the `trak` table, interpolated by point size and scaled from font units (Apple TrueType Reference Manual, `trak`).
fn tracking(bytes: &[u8], units_per_em: i32, size: f32) -> Option<f32> {
    let u16_at = |at: usize| Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?));
    let i32_at = |at: usize| Some(i32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    let fixed_at = |at: usize| Some(i32_at(at)? as f32 / 65536.0);
    let horizontal = usize::from(u16_at(6)?);
    let tracks = usize::from(u16_at(horizontal)?);
    let sizes = usize::from(u16_at(horizontal + 2)?);
    let size_table = usize::try_from(i32_at(horizontal + 4)?).ok()?;
    let entry = (0..tracks)
        .map(|track| horizontal + 8 + 8 * track)
        .find(|&entry| fixed_at(entry) == Some(0.0))?;
    let values = usize::from(u16_at(entry + 6)?);
    let points: Vec<(f32, f32)> = (0..sizes)
        .map(|index| {
            let units = u16_at(values + 2 * index)? as i16;
            Some((fixed_at(size_table + 4 * index)?, f32::from(units)))
        })
        .collect::<Option<_>>()?;
    let units = match points.iter().position(|&(at, _)| at >= size) {
        Some(0) => points.first()?.1,
        Some(index) => {
            let ((below, low), (above, high)) = (points[index - 1], points[index]);
            low + (size - below) / (above - below) * (high - low)
        }
        None => points.last()?.1,
    };
    Some(units * size / units_per_em as f32)
}
