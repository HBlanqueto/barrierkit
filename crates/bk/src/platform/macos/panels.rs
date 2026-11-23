use std::cell::RefCell;
use std::path::{Path, PathBuf};

use block2::RcBlock;
use dispatch2::DispatchQueue;
use futures_channel::oneshot;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSAlertThirdButtonReturn, NSModalResponse,
    NSModalResponseOK, NSOpenPanel, NSSavePanel, NSWorkspace,
};
use objc2_foundation::{NSArray, NSString, NSURL};

use super::with_runtime;
use crate::document::SaveChoice;
use crate::file_dialog::{ContentType, FileDialog};
use crate::i18n::text;
use crate::platform::WindowControls;

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

pub async fn open_files(title: &str, multiple: bool, directory: bool) -> Option<Vec<PathBuf>> {
    let kinds = if directory {
        vec![ContentType::Folder]
    } else {
        Vec::new()
    };
    let dialog = FileDialog::open()
        .title(title)
        .allows_multiple_selection(multiple)
        .allowed_content_types(kinds);
    Some(run(dialog, None).await)
}

pub async fn save_file(title: &str, current_name: &str) -> Option<PathBuf> {
    run(FileDialog::save(current_name).title(title), None)
        .await
        .into_iter()
        .next()
}

pub(crate) async fn present(dialog: FileDialog, window: &WindowControls) -> Vec<PathBuf> {
    run(dialog, Some(window.surface_id.raw())).await
}

fn extensions(dialog: &FileDialog) -> Vec<&str> {
    dialog
        .types
        .iter()
        .filter_map(|kind| match kind {
            ContentType::Extension(extension) => Some(extension.as_str()),
            ContentType::Folder => None,
        })
        .collect()
}

fn chooses_files(dialog: &FileDialog) -> bool {
    !dialog.chooses_folders() || !extensions(dialog).is_empty()
}

// allowedContentTypes needs a UTType; the extension list does the same on every macOS the kit supports.
#[allow(deprecated)]
fn prepare(
    mtm: MainThreadMarker,
    dialog: &FileDialog,
) -> (Retained<NSSavePanel>, Option<Retained<NSOpenPanel>>) {
    let open = dialog
        .save_as
        .is_none()
        .then(|| NSOpenPanel::openPanel(mtm));
    let panel = match &open {
        Some(open) => Retained::into_super(open.clone()),
        None => NSSavePanel::savePanel(mtm),
    };
    panel.setMessage(Some(&NSString::from_str(&dialog.title)));
    if let Some(name) = &dialog.save_as {
        panel.setNameFieldStringValue(&NSString::from_str(name));
        panel.setCanCreateDirectories(true);
    }
    if let Some(open) = &open {
        open.setCanChooseFiles(chooses_files(dialog));
        open.setCanChooseDirectories(dialog.chooses_folders());
        open.setAllowsMultipleSelection(dialog.multiple);
    }
    let extensions: Vec<_> = extensions(dialog)
        .into_iter()
        .map(NSString::from_str)
        .collect();
    if !extensions.is_empty() {
        panel.setAllowedFileTypes(Some(&NSArray::from_retained_slice(&extensions)));
    }
    if let Some(directory) = &dialog.directory {
        let path = NSString::from_str(&directory.to_string_lossy());
        panel.setDirectoryURL(Some(&NSURL::fileURLWithPath(&path)));
    }
    (panel, open)
}

// A sheet of its window when the runner knows it, a window of its own otherwise.
async fn run(dialog: FileDialog, window: Option<u64>) -> Vec<PathBuf> {
    on_main(move |mtm, sender| {
        let parent = window.and_then(|id| {
            with_runtime(|runtime| runtime.window(id).map(|found| found.window.clone())).flatten()
        });
        let sender = RefCell::new(Some(sender));
        let (panel, open) = prepare(mtm, &dialog);
        let finished = panel.clone();
        let block = RcBlock::new(move |response: NSModalResponse| {
            let chosen = match (&open, response == NSModalResponseOK) {
                (_, false) => Vec::new(),
                (Some(open), true) => open.URLs().iter().filter_map(|url| path_of(&url)).collect(),
                (None, true) => finished
                    .URL()
                    .and_then(|url| path_of(&url))
                    .into_iter()
                    .collect(),
            };
            if let Some(sender) = sender.borrow_mut().take() {
                let _ = sender.send(chosen);
            }
        });
        match parent {
            Some(parent) => panel.beginSheetModalForWindow_completionHandler(&parent, &block),
            None => panel.beginWithCompletionHandler(&block),
        }
    })
    .await
    .unwrap_or_default()
}

pub(crate) async fn confirm_save(window: &WindowControls, name: &str) -> SaveChoice {
    let (id, name) = (window.surface_id.raw(), name.to_owned());
    on_main(move |mtm, sender| {
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&NSString::from_str(&text!(
            "save-changes-message",
            name = name
        )));
        alert.setInformativeText(&NSString::from_str(text!("save-changes-detail")));
        alert.addButtonWithTitle(&NSString::from_str(text!("save")));
        alert.addButtonWithTitle(&NSString::from_str(text!("cancel")));
        let discard = alert.addButtonWithTitle(&NSString::from_str(text!("dont-save")));
        discard.setKeyEquivalent(&NSString::from_str("d"));
        let sender = RefCell::new(Some(sender));
        let block = RcBlock::new(move |response: NSModalResponse| {
            let choice = if response == NSAlertFirstButtonReturn {
                SaveChoice::Save
            } else if response == NSAlertThirdButtonReturn {
                SaveChoice::DontSave
            } else {
                SaveChoice::Cancel
            };
            if let Some(sender) = sender.borrow_mut().take() {
                let _ = sender.send(choice);
            }
        });
        let parent =
            with_runtime(|runtime| runtime.window(id).map(|found| found.window.clone())).flatten();
        match parent {
            Some(parent) => alert.beginSheetModalForWindow_completionHandler(&parent, Some(&block)),
            None => block.call((alert.runModal(),)),
        }
    })
    .await
    .unwrap_or(SaveChoice::Cancel)
}

pub(crate) fn mark_document(window: &WindowControls, edited: bool, url: Option<&Path>) {
    let (id, path) = (
        window.surface_id.raw(),
        url.map(|path| path.to_string_lossy().into_owned()),
    );
    DispatchQueue::main().exec_async(move || {
        let Some(found) =
            with_runtime(|runtime| runtime.window(id).map(|found| found.window.clone())).flatten()
        else {
            return;
        };
        found.setDocumentEdited(edited);
        let represented = path.map(|path| NSURL::fileURLWithPath(&NSString::from_str(&path)));
        found.setRepresentedURL(represented.as_deref());
    });
}

pub async fn open_uri(uri: &str, _ask: bool) {
    let uri = uri.to_owned();
    DispatchQueue::main().exec_async(move || {
        let opened = NSURL::URLWithString(&NSString::from_str(&uri))
            .is_some_and(|url| NSWorkspace::sharedWorkspace().openURL(&url));
        if !opened {
            tracing::warn!(uri, "BarrierKit: the system could not open the address");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_only_filter_chooses_folders_and_no_files() {
        let dialog = FileDialog::open().allowed_content_types(vec![ContentType::Folder]);
        assert!(dialog.chooses_folders());
        assert!(!chooses_files(&dialog));
    }

    #[test]
    fn extensions_leave_the_folder_kind_out_and_keep_files_choosable() {
        let dialog = FileDialog::open()
            .allowed_content_types(vec![ContentType::Folder, ContentType::extension("png")]);
        assert_eq!(extensions(&dialog), ["png"]);
        assert!(dialog.chooses_folders() && chooses_files(&dialog));
    }

    #[test]
    fn no_filter_chooses_any_file() {
        let dialog = FileDialog::open();
        assert!(chooses_files(&dialog) && !dialog.chooses_folders());
        assert!(extensions(&dialog).is_empty());
    }
}
