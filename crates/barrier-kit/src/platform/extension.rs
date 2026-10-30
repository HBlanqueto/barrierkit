use std::any::Any;

use freya_core::integration::Runner;
use sctk::reexports::client::{
    QueueHandle,
    protocol::{wl_output::WlOutput, wl_seat::WlSeat},
};

use super::runtime::Runtime;

/// A single extension slot installed into the runner.
///
/// `shell/` uses this to register its state and hooks without `platform/`
/// knowing anything about shells.
pub(crate) struct Extension {
    pub(crate) state: Box<dyn Any>,
    pub(crate) hooks: ExtensionHooks,
}

/// Function-pointer hooks. They are `fn` so the extension slot does not need
/// generics, and so the core can call them without taking the slot apart.
#[derive(Clone, Copy)]
pub(crate) struct ExtensionHooks {
    pub(crate) provide_contexts: fn(&Runtime, &mut Runner),
    pub(crate) new_seat: fn(&mut Runtime, &WlSeat),
    pub(crate) new_output: fn(&mut Runtime, &QueueHandle<Runtime>, &WlOutput),
    pub(crate) poll: fn(&mut Runtime),
}

impl Extension {
    /// Borrows the extension state as `T` if the held state has that type.
    pub(crate) fn state<T: 'static>(&self) -> Option<&T> {
        self.state.downcast_ref::<T>()
    }

    /// Mutably borrows the extension state as `T` if the held state has that type.
    pub(crate) fn state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.downcast_mut::<T>()
    }
}

impl Runtime {
    /// Borrows the extension state as `T` if the installed extension holds it.
    pub(crate) fn extension<T: 'static>(&self) -> Option<&T> {
        self.extension.as_ref()?.state::<T>()
    }

    /// Mutably borrows the extension state as `T` if the installed extension holds it.
    pub(crate) fn extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extension.as_mut()?.state_mut::<T>()
    }

    /// Installs the unique extension slot.
    pub(crate) fn install_extension(&mut self, ext: Extension) {
        if self.extension.is_some() {
            debug_assert!(false, "only one extension slot is supported");
            tracing::error!("BarrierKit: extension already installed");
        }
        self.extension = Some(ext);
    }

    /// Returns a copy of the installed hooks, if any.
    pub(crate) fn extension_hooks(&self) -> Option<ExtensionHooks> {
        self.extension.as_ref().map(|ext| ext.hooks)
    }
}
