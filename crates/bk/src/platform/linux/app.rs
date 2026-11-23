use crate::platform::{app::Host, runtime::Runtime};

impl Runtime {
    pub(crate) fn host(&self) -> Host {
        Host {
            resolved: self.resolved,
            ping: self.ping.clone(),
            queue: self.queue.clone(),
            popup_commands: self.popup_commands.clone(),
            popup_next_id: self.popup_next_id.clone(),
            last_input_serial: self.last_input_serial.clone(),
            timers: self.timers.clone(),
            wake: self.wake.clone(),
            font_collection: self.graphics.font_collection.clone(),
            font_manager: self.graphics.font_manager.clone(),
            defaults_revision: self.defaults_revision,
            glass: self.glass_supported(),
            backdrop: self.backdrop_supported(),
            native_glass: false,
            native_sheets: false,
            native_popovers: false,
            window_background: false,
            knobs: self.knobs_supported(),
        }
    }
}
