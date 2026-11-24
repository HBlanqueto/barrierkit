use std::time::Duration;

use bk::{platform::Timers, prelude::*};
use bk_shell::{SessionUnlock, run_lock};
use freya_core::integration::AppComponent;

// An example must never leave the session locked: it unlocks on its own.
const SAFETY_UNLOCK: Duration = Duration::from_secs(30);

fn lock_screen() -> impl IntoElement {
    let session = SessionUnlock::get();
    use_hook({
        let session = session.clone();
        move || Timers::get().after(SAFETY_UNLOCK, move || session.unlock())
    });
    VStack::new()
        .expanded()
        .background(color(Palette::current().window_background))
        .child(Text::new(session.keyboard_layout().unwrap_or_default()))
        .child(Button::new("Unlock").on_press(move |_| session.unlock()))
}

fn main() -> Result<(), RunError> {
    run_lock(AppComponent::from(lock_screen))
}
