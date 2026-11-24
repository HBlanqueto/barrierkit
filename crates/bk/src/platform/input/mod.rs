use freya_core::prelude::Modifiers;

thread_local! {
    static MODIFIER_FLAGS: std::cell::Cell<Modifiers> =
        const { std::cell::Cell::new(Modifiers::empty()) };
}

pub(super) fn store_modifier_flags(flags: Modifiers) {
    MODIFIER_FLAGS.set(flags);
}

pub fn modifier_flags() -> Modifiers {
    MODIFIER_FLAGS.get()
}

pub fn inserts_text(modifiers: Modifiers) -> bool {
    !modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META)
}
