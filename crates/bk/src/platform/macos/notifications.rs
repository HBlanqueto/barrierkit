use std::sync::Once;
use std::sync::atomic::{AtomicU32, Ordering};

use block2::{DynBlock, RcBlock};
use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AllocAnyThread, define_class, msg_send};
use objc2_foundation::{NSBundle, NSError, NSObject, NSObjectProtocol, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotification,
    UNNotificationPresentationOptions, UNNotificationRequest, UNNotificationResponse,
    UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};

use crate::notifications::{Notification, WAITING, lock, take};

static NEXT: AtomicU32 = AtomicU32::new(0);
static READY: Once = Once::new();

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "BKNotificationDelegate"]
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl UNUserNotificationCenterDelegate for Delegate {
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present(
            &self,
            _: &UNUserNotificationCenter,
            _: &UNNotification,
            done: &DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            done.call((
                UNNotificationPresentationOptions::Banner | UNNotificationPresentationOptions::List,
            ));
        }

        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn did_receive(
            &self,
            _: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            done: &DynBlock<dyn Fn()>,
        ) {
            answer(&response.notification().request().identifier().to_string());
            done.call(());
        }
    }
);

fn answer(identifier: &str) {
    if let Some(clicked) = id_of(identifier).and_then(|id| take(&WAITING, id)) {
        clicked(None);
    }
}

fn id_of(identifier: &str) -> Option<u32> {
    identifier.parse().ok()
}

// A bare binary has no bundle identifier, and UNUserNotificationCenter raises an exception without one.
fn bundled() -> bool {
    NSBundle::mainBundle().bundleIdentifier().is_some()
}

pub(crate) fn post(notice: Notification, clicked: impl FnOnce(Option<String>) + Send + 'static) {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    if bundled() {
        lock(&WAITING).push((id, Box::new(clicked)));
    }
    notify(id.to_string(), notice.summary, notice.body);
}

pub(crate) fn notify(identifier: String, title: String, body: String) {
    if !bundled() {
        tracing::warn!("notifications need an app bundle, and this process has none");
        return;
    }
    DispatchQueue::main().exec_async(move || add(&identifier, &title, &body));
}

fn add(identifier: &str, title: &str, body: &str) {
    let center = UNUserNotificationCenter::currentNotificationCenter();
    READY.call_once(|| {
        let delegate: Retained<Delegate> = unsafe { msg_send![Delegate::alloc(), init] };
        center.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        // The center keeps its delegate weakly, and it lives as long as the process.
        std::mem::forget(delegate);
        let denied = RcBlock::new(|granted: objc2::runtime::Bool, _: *mut NSError| {
            if !granted.as_bool() {
                tracing::warn!("notifications are not allowed for this app");
            }
        });
        center.requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert,
            &denied,
        );
    });
    let content = UNMutableNotificationContent::new();
    content.setTitle(&NSString::from_str(title));
    content.setBody(&NSString::from_str(body));
    let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
        &NSString::from_str(identifier),
        &content,
        None,
    );
    let owned = identifier.to_owned();
    let failed = RcBlock::new(move |error: *mut NSError| {
        // SAFETY: the system passes a valid NSError or null.
        if let Some(error) = unsafe { error.as_ref() } {
            tracing::warn!(%error, "could not post a notification");
            id_of(&owned).and_then(|id| take(&WAITING, id));
        }
    });
    center.addNotificationRequest_withCompletionHandler(&request, Some(&failed));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn a_click_runs_its_callback_once() {
        let runs = Arc::new(AtomicUsize::new(0));
        let counted = runs.clone();
        lock(&WAITING).push((
            u32::MAX,
            Box::new(move |_| {
                counted.fetch_add(1, Ordering::SeqCst);
            }),
        ));
        answer(&u32::MAX.to_string());
        answer(&u32::MAX.to_string());
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn an_identifier_the_kit_did_not_make_is_not_a_click() {
        answer("portal-id");
        assert_eq!(id_of("portal-id"), None);
    }
}
