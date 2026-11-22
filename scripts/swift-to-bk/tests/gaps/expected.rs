use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("gaps")
        .window("Gaps", || {
            let mut choice = use_state(|| 0);
            VStack::new()
                // TODO(swift: ZStack.init) -> swiftui-api-gap.md §3.2 types, partial
                .child(
                    rect()
                        .child(Text::new("Back"))
                        // TODO(swift: View.foregroundStyle) -> translator: not lowered yet (kit: styles::TextStyle) (swiftui-api-gap.md §8.2 view modifiers, partial)
                        .child(Text::new("Front")),
                )
                // TODO(swift: View.tag) -> swiftui-api-gap.md §7.2 view modifiers, partial
                .child(Text::new("Tagged"))
                // TODO(swift: Picker.init) -> translator: not lowered yet (kit: PopUpButton::new(l).item(s).selected(i).on_change(cb))
                .child(
                    rect()
                        // TODO(swift: View.tag) -> swiftui-api-gap.md §7.2 view modifiers, partial
                        .child(Text::new("One"))
                        // TODO(swift: View.tag) -> swiftui-api-gap.md §7.2 view modifiers, partial
                        .child(Text::new("Two")),
                )
                // TODO(swift: View.keyboardShortcut) -> translator: not lowered yet (kit: Action, MainMenu, use_menu_action) (swiftui-api-gap.md §6.2 view modifiers, absent)
                .child(Button::new("Go").on_press(move |_| *choice.write() += 1))
        })
        .run()
}

// Translation report: 7 open
//   TODO(swift: ZStack.init) -> swiftui-api-gap.md §3.2 types, partial
//   TODO(swift: View.foregroundStyle) -> translator: not lowered yet (kit: styles::TextStyle) (swiftui-api-gap.md §8.2 view modifiers, partial)
//   TODO(swift: View.tag) -> swiftui-api-gap.md §7.2 view modifiers, partial (x3)
//   TODO(swift: Picker.init) -> translator: not lowered yet (kit: PopUpButton::new(l).item(s).selected(i).on_change(cb))
//   TODO(swift: View.keyboardShortcut) -> translator: not lowered yet (kit: Action, MainMenu, use_menu_action) (swiftui-api-gap.md §6.2 view modifiers, absent)
