use tracing::Level;
use tracing_subscriber::{Layer, filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    let targets = std::env::var("RUST_LOG")
        .ok()
        .and_then(|directives| directives.parse::<Targets>().ok())
        .unwrap_or_else(|| Targets::new().with_default(Level::INFO));
    let registry =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(targets));
    #[cfg(feature = "tracy")]
    let registry = registry.with(tracing_tracy::TracyLayer::default());
    registry.init();
}

#[cfg(feature = "tracy")]
pub(crate) fn frame_mark() {
    if let Some(client) = tracing_tracy::client::Client::running() {
        client.frame_mark();
    }
}
