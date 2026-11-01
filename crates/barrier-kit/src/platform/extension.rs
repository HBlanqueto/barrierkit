use std::any::Any;

use freya_core::integration::Runner;
use sctk::reexports::client::{
    QueueHandle,
    protocol::{wl_output::WlOutput, wl_seat::WlSeat},
};

use super::runtime::Runtime;

pub(crate) struct Extension {
    pub(crate) state: Box<dyn Any>,
    pub(crate) hooks: ExtensionHooks,
}

#[derive(Clone, Copy)]
pub(crate) struct ExtensionHooks {
    pub(crate) provide_contexts: fn(&Runtime, &mut Runner),
    pub(crate) new_seat: fn(&mut Runtime, &WlSeat),
    pub(crate) new_output: fn(&mut Runtime, &QueueHandle<Runtime>, &WlOutput),
    pub(crate) poll: fn(&mut Runtime),
}

impl Extension {
    pub(crate) fn state<T: 'static>(&self) -> Option<&T> {
        self.state.downcast_ref::<T>()
    }

    pub(crate) fn state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.downcast_mut::<T>()
    }
}

impl Runtime {
    #[allow(dead_code)]
    pub(crate) fn extension<T: 'static>(&self) -> Option<&T> {
        self.extension.as_ref()?.state::<T>()
    }

    #[allow(dead_code)]
    pub(crate) fn extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extension.as_mut()?.state_mut::<T>()
    }

    #[allow(dead_code)]
    pub(crate) fn install_extension(&mut self, ext: Extension) {
        if self.extension.is_some() {
            debug_assert!(false, "only one extension slot is supported");
            tracing::error!("BarrierKit: extension already installed");
        }
        self.extension = Some(ext);
    }

    pub(crate) fn extension_hooks(&self) -> Option<ExtensionHooks> {
        self.extension.as_ref().map(|ext| ext.hooks)
    }
}
