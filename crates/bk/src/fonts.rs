use std::cell::RefCell;

use freya_engine::prelude::{FontCollection, FontMgr, FontStyle, TypefaceFontProvider};

thread_local! {
    static FONTS: RefCell<Option<(FontCollection, FontMgr)>> = const { RefCell::new(None) };
    static OWN: RefCell<Option<TypefaceFontProvider>> = const { RefCell::new(None) };
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
    let Some(typeface) = FontMgr::default().new_from_data(bytes, None) else {
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
