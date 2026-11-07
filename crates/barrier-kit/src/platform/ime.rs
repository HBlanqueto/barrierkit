use accesskit::Role;
use freya_core::integration::{KeyboardEventName, PlatformEvent};
use freya_core::prelude::{Code, Key, Modifiers, WritableUtils};
use sctk::reexports::client::backend::ObjectId;
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
    cursor_surface: Option<ObjectId>,
    entered: bool,
    content: Option<(ContentHint, ContentPurpose)>,
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
                runtime.ime.entered = true;
                runtime.sync_ime();
            }
            zwp_text_input_v3::Event::Leave { .. } => {
                if runtime.ime.content.take().is_some() {
                    input.disable();
                    input.commit();
                }
                runtime.ime.entered = false;
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

    pub(crate) fn set_ime_cursor(&mut self, surface: ObjectId, area: (i32, i32, i32, i32)) {
        self.ime.cursor_surface = Some(surface);
        if self.ime.cursor == Some(area) {
            return;
        }
        self.ime.cursor = Some(area);
        if let (Some(_), Some(input)) = (self.ime.content, &self.ime.input) {
            let (x, y, width, height) = area;
            input.set_cursor_rectangle(x, y, width, height);
            input.commit();
        }
    }

    // text-input-v3 «enable» follows the text focus: a focused field, or a surface that placed an IME cursor.
    fn wanted_content(&self) -> Option<(ContentHint, ContentPurpose)> {
        let surface = self.keyboard_focus.as_ref()?;
        let index = self.surface_for(surface)?;
        match self.surfaces[index].app.focused_role() {
            Role::PasswordInput => Some((
                ContentHint::SensitiveData | ContentHint::HiddenText,
                ContentPurpose::Password,
            )),
            Role::MultilineTextInput => Some((ContentHint::Multiline, ContentPurpose::Normal)),
            Role::TextInput | Role::SearchInput => {
                Some((ContentHint::None, ContentPurpose::Normal))
            }
            _ if self.ime.cursor_surface.as_ref() == Some(&surface.id()) => {
                Some((ContentHint::None, ContentPurpose::Terminal))
            }
            _ => None,
        }
    }

    pub(crate) fn sync_ime(&mut self) {
        let wanted = self.wanted_content().filter(|_| self.ime.entered);
        if wanted == self.ime.content {
            return;
        }
        let Some(input) = self.ime.input.clone() else {
            return;
        };
        match wanted {
            Some((hint, purpose)) => {
                if self.ime.content.is_none() {
                    input.enable();
                }
                input.set_content_type(hint, purpose);
                if let Some((x, y, width, height)) = self.ime.cursor {
                    input.set_cursor_rectangle(x, y, width, height);
                }
            }
            None => {
                input.disable();
                self.show_preedit(String::new());
            }
        }
        input.commit();
        self.ime.content = wanted;
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
