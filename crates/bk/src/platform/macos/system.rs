use std::ptr::NonNull;

use block2::RcBlock;
use ctappearance::{AppearanceConfig, load_appearance};
use dispatch2::DispatchQueue;
use objc2_app_kit::{NSWorkspace, NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification};
use objc2_foundation::NSNotification;

use super::with_runtime;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Flags {
    pub(super) reduce_transparency: bool,
    pub(super) increase_contrast: bool,
    pub(super) reduce_motion: bool,
}

// The system's accessibility settings add to the file's: either asks for the plainer look.
pub(super) fn merged(config: AppearanceConfig, flags: Flags) -> AppearanceConfig {
    AppearanceConfig {
        reduce_transparency: config.reduce_transparency || flags.reduce_transparency,
        contrast: config.contrast || flags.increase_contrast,
        reduced_motion: config.reduced_motion || flags.reduce_motion,
        ..config
    }
}

fn flags() -> Flags {
    let workspace = NSWorkspace::sharedWorkspace();
    Flags {
        reduce_transparency: workspace.accessibilityDisplayShouldReduceTransparency(),
        increase_contrast: workspace.accessibilityDisplayShouldIncreaseContrast(),
        reduce_motion: workspace.accessibilityDisplayShouldReduceMotion(),
    }
}

pub(super) fn appearance() -> AppearanceConfig {
    merged(load_appearance(), flags())
}

// The settings change while the app runs: the runner resolves the appearance again.
pub(super) fn observe() {
    let block = RcBlock::new(|_: NonNull<NSNotification>| {
        DispatchQueue::main().exec_async(|| {
            with_runtime(|runtime| runtime.refresh_appearance());
        });
    });
    // SAFETY: the name is AppKit's own constant, and the block lives as long as the process.
    let observer = unsafe {
        NSWorkspace::sharedWorkspace()
            .notificationCenter()
            .addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification),
                None,
                None,
                &block,
            )
    };
    std::mem::forget((observer, block));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_system_setting_adds_to_the_file_and_never_removes_from_it() {
        let file = AppearanceConfig {
            contrast: true,
            ..AppearanceConfig::default()
        };
        let merged_config = merged(
            file,
            Flags {
                reduce_transparency: true,
                ..Flags::default()
            },
        );
        assert!(merged_config.contrast && merged_config.reduce_transparency);
        assert!(!merged_config.reduced_motion);
        let plain = merged(AppearanceConfig::default(), Flags::default());
        assert!(!plain.contrast && !plain.reduce_transparency && !plain.reduced_motion);
    }
}
