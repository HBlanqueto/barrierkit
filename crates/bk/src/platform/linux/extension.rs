use std::any::Any;

use freya_core::integration::Runner;
use sctk::reexports::client::{
    QueueHandle,
    backend::ObjectId,
    protocol::{wl_output::WlOutput, wl_seat::WlSeat},
};

#[cfg(feature = "shell")]
pub use crate::platform::linux::glass::packed;
use crate::platform::linux::runtime::Runtime;

pub struct Extension {
    #[cfg(feature = "shell")]
    pub state: Box<dyn Any>,
    pub hooks: ExtensionHooks,
}

#[derive(Clone, Copy)]
pub struct ExtensionHooks {
    pub provide_contexts: fn(&Runtime, &mut Runner),
    pub new_seat: fn(&mut Runtime, &WlSeat),
    pub new_output: fn(&mut Runtime, &QueueHandle<Runtime>, &WlOutput),
    pub poll: fn(&mut Runtime),
    pub kit_event: fn(&mut Runtime, &ObjectId, Box<dyn Any>),
}

impl Runtime {
    #[cfg(feature = "shell")]
    pub fn extension<T: 'static>(&self) -> Option<&T> {
        self.extension.as_ref()?.state.downcast_ref::<T>()
    }

    #[cfg(feature = "shell")]
    pub fn extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extension.as_mut()?.state.downcast_mut::<T>()
    }

    pub fn provide_extension_contexts(&self, runner: &mut Runner) {
        if let Some(hooks) = self.extension_hooks() {
            (hooks.provide_contexts)(self, runner);
        }
    }

    pub(crate) fn extension_hooks(&self) -> Option<ExtensionHooks> {
        self.extension.as_ref().map(|ext| ext.hooks)
    }
}
