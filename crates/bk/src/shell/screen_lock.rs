use std::time::Duration;

use freya_core::prelude::{State, WritableUtils};
use infinity_protocol::client::session::infinity_session_v1::infinity_session_v1::{
    self, InfinitySessionV1,
};
use sctk::reexports::client::{Connection, Dispatch, QueueHandle};

use crate::platform::runtime::Runtime;
use crate::shell::ShellState;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScreenLock {
    #[default]
    Unlocked,
    Locking(Duration),
}

pub fn use_screen_lock() -> State<ScreenLock> {
    freya_core::prelude::consume_root_context()
}

impl Dispatch<InfinitySessionV1, ()> for Runtime {
    fn event(
        runtime: &mut Self,
        _: &InfinitySessionV1,
        event: infinity_session_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let lock = match event {
            infinity_session_v1::Event::Locking { duration } => {
                ScreenLock::Locking(Duration::from_millis(duration.into()))
            }
            infinity_session_v1::Event::Unlocked => ScreenLock::Unlocked,
            _ => return,
        };
        let Some(shell) = runtime.extension::<ShellState>() else {
            return;
        };
        let mut state = shell.screen_lock;
        state.set(lock);
        runtime.request_redraw_all = true;
        runtime.ping.ping();
    }
}
