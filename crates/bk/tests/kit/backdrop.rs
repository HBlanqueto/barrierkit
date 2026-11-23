use bk::backdrop::BackdropSource;
use freya_core::prelude::*;

use crate::controls::{backdrop_over, render};

fn untinted() -> Element {
    backdrop_over(BackdropSource::None)
}

#[test]
fn without_a_tint_the_window_follows_the_system_scheme() {
    let size = (420.0, 300.0);
    assert!(render(size, false, untinted) != render(size, true, untinted));
}
