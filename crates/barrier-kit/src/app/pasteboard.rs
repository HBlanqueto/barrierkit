use freya_clipboard::copypasta::ClipboardProvider;
use freya_core::prelude::*;

type Provider = State<Option<Box<dyn ClipboardProvider>>>;

pub fn copy(text: impl Into<String>) {
    if let Some(mut provider) = try_consume_root_context::<Provider>()
        && let Some(clipboard) = provider.write().as_mut()
        && let Err(error) = clipboard.set_contents(text.into())
    {
        tracing::warn!(%error, "pasteboard: no se pudo copiar");
    }
}

pub fn paste() -> Option<String> {
    let mut provider = try_consume_root_context::<Provider>()?;
    let mut guard = provider.write();
    guard.as_mut()?.get_contents().ok()
}
