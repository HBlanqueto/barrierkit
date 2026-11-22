use freya_clipboard::copypasta::{
    ClipboardContext, ClipboardProvider, nop_clipboard::NopClipboardContext,
};

#[derive(Clone)]
pub(crate) struct Clipboard {
    system: bool,
}

impl Clipboard {
    pub(crate) fn system() -> Self {
        Self { system: true }
    }

    pub(crate) fn primary() -> Self {
        Self { system: false }
    }

    pub(crate) fn provider(&self) -> Box<dyn ClipboardProvider> {
        match ClipboardContext::new() {
            Ok(context) if self.system => Box::new(context),
            _ => Box::new(NopClipboardContext::new().expect("the nop clipboard cannot fail")),
        }
    }

    pub(crate) fn set_rich(&self, text: String, _html: String) {
        let _ = self.provider().set_contents(text);
    }
}
