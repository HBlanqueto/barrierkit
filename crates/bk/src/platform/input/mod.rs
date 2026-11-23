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

#[cfg(test)]
mod tests {
    use super::inserts_text;

    use freya_core::prelude::Modifiers;

    #[test]
    fn inserts_text_allows_shift_but_rejects_ctrl_alt_meta() {
        let cases = [
            (Modifiers::empty(), true),
            (Modifiers::SHIFT, true),
            (Modifiers::CONTROL, false),
            (Modifiers::ALT, false),
            (Modifiers::META, false),
            (Modifiers::CONTROL | Modifiers::SHIFT, false),
            (Modifiers::ALT | Modifiers::SHIFT, false),
            (Modifiers::META | Modifiers::SHIFT, false),
        ];
        for (modifiers, expected) in cases {
            assert_eq!(
                inserts_text(modifiers),
                expected,
                "{modifiers:?} should be {expected}"
            );
        }
    }
}
