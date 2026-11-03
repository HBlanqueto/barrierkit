use barrier_kit::platform::{Decorations, WindowOptions, run_window};
use freya_core::integration::AppComponent;
use vow_design::WINDOW_MIN_SIZE;

fn main() {
    tracing_subscriber::fmt::init();
    marea::assets::register_fonts();
    marea::assets::resolve_remote();
    run_window(
        AppComponent::new(marea::Marea),
        WindowOptions::new("Marea", "marea")
            .decorations(Decorations::Client)
            .size((1160, 740))
            .min_size(Some(WINDOW_MIN_SIZE)),
    )
    .expect("marea");
}
