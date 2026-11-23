use objc2::rc::Retained;
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::{NSImage, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::NSString;

use super::menu::{Target, status_menu};
use crate::status_item::StatusItem;

// With a menu the system opens it on a click, so only an icon without items sends STATUS_ITEM_ACTIVATE.
pub(super) struct StatusBar {
    item: Retained<NSStatusItem>,
    _target: Retained<Target>,
}

impl StatusBar {
    pub(super) fn new(mtm: MainThreadMarker, owner: u64, status: &StatusItem) -> Self {
        let target = Target::new(mtm, Some(owner));
        let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        if let Some(button) = item.button(mtm) {
            match NSImage::imageNamed(&NSString::from_str(&status.icon_name)) {
                Some(image) => {
                    image.setTemplate(true);
                    button.setImage(Some(&image));
                }
                None => button.setTitle(&NSString::from_str(&status.title)),
            }
            button.setToolTip(Some(&NSString::from_str(&status.title)));
            if status.items.is_empty() {
                // SAFETY: `activate:` is `Target`'s action and `_target` keeps it alive as long as the button.
                unsafe {
                    button.setTarget(Some(&target));
                    button.setAction(Some(sel!(activate:)));
                }
            }
        }
        if !status.items.is_empty() {
            item.setMenu(Some(&status_menu(mtm, &target, &status.items)));
        }
        Self {
            item,
            _target: target,
        }
    }
}

impl Drop for StatusBar {
    fn drop(&mut self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.item);
    }
}
