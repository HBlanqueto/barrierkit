use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSApplication, NSApplicationDelegate, NSApplicationTerminateReply};
use objc2_foundation::{NSArray, NSNotification, NSObject, NSObjectProtocol, NSURL};

use crate::document::{OpenedUrl, deliver_open, first_edited};
use crate::platform::macos::{drop_runtime, with_runtime};

define_class!(
    // The app answers Finder, `open` and Cmd-Q here.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKApplicationDelegate"]
    struct AppDelegate;

    // SAFETY: AppDelegate is an NSObject subclass.
    unsafe impl NSObjectProtocol for AppDelegate {}

    // SAFETY: each method keeps the selector and signature of NSApplicationDelegate.
    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(application:openURLs:))]
        fn open_urls(&self, _: &NSApplication, urls: &NSArray<NSURL>) {
            for url in urls {
                deliver_open(opened(&url));
            }
            with_runtime(|runtime| {
                runtime.poll_all();
                runtime.redraw_all();
            });
        }

        #[unsafe(method(applicationWillTerminate:))]
        fn will_terminate(&self, _: &NSNotification) {
            drop_runtime();
        }

        // A document with unsaved changes asks first; the user quits again after answering.
        #[unsafe(method(applicationShouldTerminate:))]
        fn should_terminate(&self, _: &NSApplication) -> NSApplicationTerminateReply {
            let window = first_edited()
                .and_then(|id| with_runtime(|runtime| runtime.window_of(id.raw())).flatten());
            match window {
                Some(window) => {
                    window.performClose(None);
                    NSApplicationTerminateReply::TerminateCancel
                }
                None => NSApplicationTerminateReply::TerminateNow,
            }
        }
    }
);

fn opened(url: &NSURL) -> OpenedUrl {
    match url.isFileURL().then(|| url.path()).flatten() {
        Some(path) => OpenedUrl::File(path.to_string().into()),
        None => OpenedUrl::Other(
            url.absoluteString()
                .map(|text| text.to_string())
                .unwrap_or_default(),
        ),
    }
}

pub(super) fn install(mtm: MainThreadMarker, application: &NSApplication) {
    let delegate: Retained<AppDelegate> = {
        let this = AppDelegate::alloc(mtm).set_ivars(());
        // SAFETY: init is NSObject's initializer, called once on the allocated object.
        unsafe { msg_send![super(this), init] }
    };
    application.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    // NSApplication keeps its delegate unretained; this one lives as long as the process.
    std::mem::forget(delegate);
}
