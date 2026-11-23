use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ctdesign::{DIALOG_SHEET_HEIGHT, DIALOG_SHEET_WIDTH};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;

use crate::document::SaveChoice;
use crate::file_dialog::FileDialog;
use crate::i18n::text;
use crate::platform::{WindowControls, WindowOptions};
use crate::styles::Theme;
use crate::views::feedback::{Alert, sheet_window_root};

// The portal takes no parent window, filters or start folder (portal.rs); the dialog opens without them.
pub(crate) async fn present(dialog: FileDialog, _window: &WindowControls) -> Vec<PathBuf> {
    match &dialog.save_as {
        Some(name) => crate::portal::save_file(&dialog.title, name)
            .await
            .into_iter()
            .collect(),
        None => crate::portal::open_files(&dialog.title, dialog.multiple, dialog.chooses_folders())
            .await
            .unwrap_or_default(),
    }
}

// The kit's own Alert in a sheet window stands in for NSAlert; its buttons keep the macOS order.
pub(crate) async fn confirm_save(window: &WindowControls, name: &str) -> SaveChoice {
    let (sender, receiver) = futures_channel::oneshot::channel();
    let sender = Rc::new(RefCell::new(Some(sender)));
    let answer = |choice: SaveChoice| {
        let sender = sender.clone();
        Callback::new(move |_: ()| {
            if let Some(sender) = sender.borrow_mut().take() {
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
