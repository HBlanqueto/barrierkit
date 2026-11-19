use freya_core::integration::Runner;
use sctk::reexports::client::{
    QueueHandle,
    protocol::{wl_output::WlOutput, wl_seat::WlSeat},
};

use super::runtime::Runtime;

pub(crate) struct Extension {
    #[cfg(feature = "shell")]
    pub(crate) state: Box<dyn std::any::Any>,
    pub(crate) hooks: ExtensionHooks,
}

#[derive(Clone, Copy)]
pub(crate) struct ExtensionHooks {
    pub(crate) provide_contexts: fn(&Runtime, &mut Runner),
    pub(crate) new_seat: fn(&mut Runtime, &WlSeat),
    pub(crate) new_output: fn(&mut Runtime, &QueueHandle<Runtime>, &WlOutput),
    pub(crate) poll: fn(&mut Runtime),
}

impl Runtime {
    #[cfg(feature = "shell")]
    pub(crate) fn extension<T: 'static>(&self) -> Option<&T> {
        self.extension.as_ref()?.state.downcast_ref::<T>()
    }

    #[cfg(feature = "shell")]
    pub(crate) fn extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extension.as_mut()?.state.downcast_mut::<T>()
    }

    pub(crate) fn extension_hooks(&self) -> Option<ExtensionHooks> {
        self.extension.as_ref().map(|ext| ext.hooks)
    }
}
