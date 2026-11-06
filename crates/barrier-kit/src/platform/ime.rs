use freya_core::integration::{KeyboardEventName, PlatformEvent};
use freya_core::prelude::{Code, Key, Modifiers, WritableUtils};
use sctk::reexports::client::protocol::wl_seat::WlSeat;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::text_input::zv3::client::zwp_text_input_manager_v3::ZwpTextInputManagerV3;
use wayland_protocols::wp::text_input::zv3::client::zwp_text_input_v3::{
    self, ContentHint, ContentPurpose, ZwpTextInputV3,
};

use super::runtime::Runtime;

#[derive(Default)]
pub(crate) struct Ime {
    pub(crate) input: Option<ZwpTextInputV3>,
    pub(crate) cursor: Option<(i32, i32, i32, i32)>,
    enabled: bool,
    preedit: Option<String>,
    commit: Option<String>,
}

impl Dispatch<ZwpTextInputManagerV3, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &ZwpTextInputManagerV3,
        _: <ZwpTextInputManagerV3 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTextInputV3, ()> for Runtime {
    fn event(
        runtime: &mut Self,
        input: &ZwpTextInputV3,
        event: zwp_text_input_v3::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_text_input_v3::Event::Enter { .. } => {
                input.enable();
                input.set_content_type(ContentHint::None, ContentPurpose::Terminal);
                if let Some((x, y, width, height)) = runtime.ime.cursor {
                    input.set_cursor_rectangle(x, y, width, height);
                }
                input.commit();
                runtime.ime.enabled = true;
            }
            zwp_text_input_v3::Event::Leave { .. } => {
                input.disable();
                input.commit();
                runtime.ime.enabled = false;
                runtime.show_preedit(String::new());
            }
            zwp_text_input_v3::Event::PreeditString { text, .. } => {
                runtime.ime.preedit = Some(text.unwrap_or_default());
            }
            zwp_text_input_v3::Event::CommitString { text } => runtime.ime.commit = text,
            zwp_text_input_v3::Event::Done { .. } => runtime.finish_ime(),
            _ => {}
        }
    }
}

impl Runtime {
    // sctk only reports seats added after start-up, so the initial seat attaches here too.
    pub(crate) fn attach_text_input(&mut self, seat: &WlSeat) {
        if let (Some(manager), None) = (&self.text_input_manager, &self.ime.input) {
            self.ime.input = Some(manager.get_text_input(seat, &self.qh, ()));
        }
    }

    pub(crate) fn set_ime_cursor(&mut self, area: (i32, i32, i32, i32)) {
        if self.ime.cursor == Some(area) {
            return;
        }
        self.ime.cursor = Some(area);
        if let (true, Some(input)) = (self.ime.enabled, &self.ime.input) {
            let (x, y, width, height) = area;
            input.set_cursor_rectangle(x, y, width, height);
            input.commit();
        }
    }

    fn finish_ime(&mut self) {
        let preedit = self.ime.preedit.take().unwrap_or_default();
        if let Some(text) = self.ime.commit.take()
            && let Some(index) = self
                .keyboard_focus
                .as_ref()
                .and_then(|surface| self.surface_for(surface))
        {
            for name in [KeyboardEventName::KeyDown, KeyboardEventName::KeyUp] {
                self.surfaces[index].app.push(PlatformEvent::Keyboard {
                    name,
                    key: Key::Character(text.clone()),
                    code: Code::Unidentified,
                    modifiers: Modifiers::empty(),
                });
            }
        }
        self.show_preedit(preedit);
        self.poll_all();
        self.redraw_all();
    }

    fn show_preedit(&mut self, text: String) {
        let focused = self.keyboard_focus.as_ref().map(Proxy::id);
        for context in self
            .surfaces
            .iter()
            .filter_map(|surface| surface.context.as_ref())
        {
            if Some(&context.controls.surface_id) == focused.as_ref() {
                let mut preedit = context.controls.ime_preedit;
                preedit.set_if_modified(text.clone());
            }
        }
    }
}
