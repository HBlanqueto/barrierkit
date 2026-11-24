use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};

use block2::RcBlock;
use ctappearance::{AppearanceConfig, ColorScheme, ScrollBars, load_appearance, set_system_scheme};
use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AllocAnyThread, define_class, msg_send};
use objc2_app_kit::{
    NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication,
    NSPreferredScrollerStyleDidChangeNotification, NSScroller, NSScrollerStyle, NSWorkspace,
    NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification,
};
use objc2_foundation::{
    NSArray, NSDictionary, NSKeyValueChangeKey, NSKeyValueObservingOptions, NSNotification,
    NSNotificationCenter, NSObject, NSObjectNSKeyValueObserverRegistration, NSString, ns_string,
};

use super::with_runtime;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Flags {
    pub(super) reduce_transparency: bool,
    pub(super) increase_contrast: bool,
    pub(super) reduce_motion: bool,
    pub(super) legacy_scrollers: bool,
}

// The system's accessibility settings add to the file's: either asks for the plainer look.
pub(super) fn merged(config: AppearanceConfig, flags: Flags) -> AppearanceConfig {
    AppearanceConfig {
        reduce_transparency: config.reduce_transparency || flags.reduce_transparency,
        contrast: config.contrast || flags.increase_contrast,
        reduced_motion: config.reduced_motion || flags.reduce_motion,
        scroll_bars: match config.scroll_bars {
            ScrollBars::Automatic if flags.legacy_scrollers => ScrollBars::Always,
            ScrollBars::Automatic => ScrollBars::WhenScrolling,
            chosen => chosen,
        },
        ..config
    }
}

fn flags() -> Flags {
    let workspace = NSWorkspace::sharedWorkspace();
    Flags {
        reduce_transparency: workspace.accessibilityDisplayShouldReduceTransparency(),
        increase_contrast: workspace.accessibilityDisplayShouldIncreaseContrast(),
        reduce_motion: workspace.accessibilityDisplayShouldReduceMotion(),
        legacy_scrollers: objc2::MainThreadMarker::new()
            .is_some_and(|mtm| NSScroller::preferredScrollerStyle(mtm) == NSScrollerStyle::Legacy),
    }
}

fn scheme() -> Option<ColorScheme> {
    let mtm = objc2::MainThreadMarker::new()?;
    let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();
    // SAFETY: both names are AppKit's own constants.
    let (light, dark) = unsafe { (NSAppearanceNameAqua, NSAppearanceNameDarkAqua) };
    let best =
        appearance.bestMatchFromAppearancesWithNames(&NSArray::from_slice(&[light, dark]))?;
    Some(if &*best == dark {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    })
}

pub(crate) fn appearance() -> AppearanceConfig {
    set_system_scheme(scheme());
    merged(load_appearance(), flags())
}

fn refresh() {
    DispatchQueue::main().exec_async(|| {
        with_runtime(|runtime| runtime.refresh_appearance());
    });
}

define_class!(
    // Light and Dark reach the app as a change of `NSApp.effectiveAppearance`, observed by key-value observing.
    #[unsafe(super(NSObject))]
    #[name = "BKAppearanceObserver"]
    struct AppearanceObserver;

    impl AppearanceObserver {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn observe_value(
            &self,
            _: Option<&NSString>,
            _: Option<&AnyObject>,
            _: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
            _: *mut c_void,
        ) {
            refresh();
        }
    }
);

// The settings change while the app runs: the runner resolves the appearance again.
pub(super) fn observe(application: &NSApplication) {
    let observer: Retained<AppearanceObserver> = {
        let this = AppearanceObserver::alloc().set_ivars(());
        // SAFETY: init is NSObject's initializer, called once on the allocated object.
        unsafe { msg_send![super(this), init] }
    };
    // SAFETY: the key path names a KVO-compliant property of NSApplication; the observer lives as long as the process.
    unsafe {
        application.addObserver_forKeyPath_options_context(
            &observer,
            ns_string!("effectiveAppearance"),
            NSKeyValueObservingOptions::empty(),
            null_mut(),
        );
    }
    std::mem::forget(observer);
    let block = RcBlock::new(|_: NonNull<NSNotification>| refresh());
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
    // SAFETY: the name is AppKit's own constant, and the block lives as long as the process.
    let scrollers = unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(NSPreferredScrollerStyleDidChangeNotification),
            None,
            None,
            &block,
        )
    };
    std::mem::forget((observer, scrollers, block));
}
