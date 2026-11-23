use std::cell::RefCell;
use std::path::PathBuf;

use block2::RcBlock;
use dispatch2::DispatchQueue;
use futures_channel::oneshot;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponse, NSModalResponseOK, NSOpenPanel, NSSavePanel, NSWorkspace};
use objc2_foundation::{NSString, NSURL};

// AppKit shows a panel from the main thread, and the kit's `async` functions may run anywhere.
fn on_main<T: Send + 'static>(
    work: impl FnOnce(MainThreadMarker, oneshot::Sender<T>) + Send + 'static,
) -> oneshot::Receiver<T> {
    let (sender, receiver) = oneshot::channel();
    DispatchQueue::main().exec_async(move || match MainThreadMarker::new() {
        Some(mtm) => work(mtm, sender),
        None => drop(sender),
    });
    receiver
}

fn path_of(url: &NSURL) -> Option<PathBuf> {
    url.path().map(|path| PathBuf::from(path.to_string()))
}

pub(crate) async fn open_files(
    title: &str,
    multiple: bool,
    directory: bool,
) -> Option<Vec<PathBuf>> {
    let title = title.to_owned();
    on_main(move |mtm, sender| {
        let panel = NSOpenPanel::openPanel(mtm);
        panel.setMessage(Some(&NSString::from_str(&title)));
        panel.setCanChooseFiles(!directory);
        panel.setCanChooseDirectories(directory);
        panel.setAllowsMultipleSelection(multiple);
        let sender = RefCell::new(Some(sender));
        let finished = panel.clone();
        let block = RcBlock::new(move |response: NSModalResponse| {
            let chosen = (response == NSModalResponseOK).then(|| {
                finished
                    .URLs()
                    .iter()
                    .filter_map(|url| path_of(&url))
                    .collect::<Vec<_>>()
            });
            if let Some(sender) = sender.borrow_mut().take() {
                let _ = sender.send(chosen);
            }
        });
        panel.beginWithCompletionHandler(&block);
    })
    .await
    .ok()
    .flatten()
}

pub(crate) async fn save_file(title: &str, current_name: &str) -> Option<PathBuf> {
    let (title, name) = (title.to_owned(), current_name.to_owned());
    on_main(move |mtm, sender| {
        let panel = NSSavePanel::savePanel(mtm);
        panel.setMessage(Some(&NSString::from_str(&title)));
        panel.setNameFieldStringValue(&NSString::from_str(&name));
        let sender = RefCell::new(Some(sender));
        let finished = panel.clone();
        let block = RcBlock::new(move |response: NSModalResponse| {
            let chosen = (response == NSModalResponseOK)
                .then(|| finished.URL().and_then(|url| path_of(&url)))
                .flatten();
            if let Some(sender) = sender.borrow_mut().take() {
                let _ = sender.send(chosen);
            }
        });
        panel.beginWithCompletionHandler(&block);
    })
    .await
    .ok()
    .flatten()
}

pub(crate) fn open_uri(uri: &str) {
    let uri = uri.to_owned();
    DispatchQueue::main().exec_async(move || {
        let opened = NSURL::URLWithString(&NSString::from_str(&uri))
            .is_some_and(|url| NSWorkspace::sharedWorkspace().openURL(&url));
        if !opened {
            tracing::warn!(uri, "BarrierKit: the system could not open the address");
        }
    });
}
