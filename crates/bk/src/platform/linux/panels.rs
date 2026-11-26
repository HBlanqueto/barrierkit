use std::cell::RefCell;
use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ctdesign::{DIALOG_SHEET_HEIGHT, DIALOG_SHEET_WIDTH};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use zbus::zvariant::Value;

use crate::document::SaveChoice;
use crate::file_dialog::{ContentType, FileDialog};
use crate::i18n::text;
use crate::platform::linux::portal::choose;
pub use crate::platform::linux::portal::{open_files, open_uri, save_file};
use crate::platform::{WindowControls, WindowOptions};
use crate::styles::Theme;
use crate::views::feedback::{Alert, sheet_window_root};

// A Wayland parent window needs an xdg-foreign handle (portal window-identifiers), so the dialog has none.
pub(crate) async fn present(dialog: FileDialog, _window: &WindowControls) -> Vec<PathBuf> {
    let mut options = HashMap::new();
    let method = match &dialog.save_as {
        Some(name) => {
            options.insert("current_name", Value::from(name.as_str()));
            "SaveFile"
        }
        None => {
            options.insert("multiple", Value::from(dialog.multiple));
            options.insert("directory", Value::from(dialog.chooses_folders()));
            "OpenFile"
        }
    };
    let extensions: Vec<&str> = dialog
        .types
        .iter()
        .filter_map(|kind| match kind {
            ContentType::Extension(extension) => Some(extension.as_str()),
            ContentType::Folder => None,
        })
        .collect();
    if !extensions.is_empty() {
        let patterns = extensions.iter().map(|extension| (0u32, glob(extension)));
        let filter = (extensions.join(", "), patterns.collect::<Vec<_>>());
        options.insert("filters", Value::from(vec![filter]));
    }
    if let Some(directory) = &dialog.directory {
        let mut folder = directory.as_os_str().as_bytes().to_vec();
        folder.push(0);
        options.insert("current_folder", Value::from(folder));
    }
    choose(method, &dialog.title, options)
        .await
        .unwrap_or_default()
}

// Portal globs are case-sensitive and the spec spells `*.[tT][xX][tT]` for any case, as macOS matches extensions.
fn glob(extension: &str) -> String {
    let any_case = extension.chars().map(|letter| match letter {
        letter if letter.is_alphabetic() => {
            format!("[{}{}]", letter.to_lowercase(), letter.to_uppercase())
        }
        other => other.to_string(),
    });
    format!("*.{}", any_case.collect::<String>())
}

// The kit's own Alert in a sheet window stands in for NSAlert; its buttons keep the macOS order.
pub(crate) async fn confirm_save(window: &WindowControls, name: &str) -> SaveChoice {
    let (sender, receiver) = futures_channel::oneshot::channel();
    let sender = Rc::new(RefCell::new(Some(sender)));
    let answer = |choice: SaveChoice| {
        let sender = Rc::downgrade(&sender);
        Callback::new(move |_: ()| {
            if let Some(sender) = sender.upgrade()
                && let Some(sender) = sender.borrow_mut().take()
            {
                let _ = sender.send(choice);
            }
        })
    };
    let alert = Alert::new(
        text!("save-changes-message", name = name),
        text!("save-changes-detail"),
    )
    .confirm(text!("save"), answer(SaveChoice::Save))
    .add_button(text!("dont-save"), answer(SaveChoice::DontSave))
    .cancel(text!("cancel"), answer(SaveChoice::Cancel));
    let content = State::create_global(alert.into_element());
    let opened: Rc<RefCell<Option<WindowControls>>> = Rc::default();
    let registered = opened.clone();
    window.open_sheet(
        AppComponent::from(move || {
            // The sheet owns the sender: closing it with its parent answers Cancel.
            let _ = &sender;
            sheet_window_root(content, &Theme::tengen(), &registered, &Rc::default())
        }),
        WindowOptions::new("", "sheet").size((DIALOG_SHEET_WIDTH, DIALOG_SHEET_HEIGHT)),
    );
    let choice = receiver.await.unwrap_or(SaveChoice::Cancel);
    if let Some(sheet) = opened.borrow_mut().take() {
        sheet.close();
    }
    choice
}

pub(crate) fn mark_document(_window: &WindowControls, _edited: bool, _url: Option<&Path>) {}
