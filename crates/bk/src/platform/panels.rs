use std::path::{Path, PathBuf};

use crate::document::SaveChoice;
use crate::file_dialog::FileDialog;
use crate::platform::WindowControls;

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

// Unsupported: Linux windows do not ask before they close, so nothing calls this.
pub(crate) async fn confirm_save(_window: &WindowControls, _name: &str) -> SaveChoice {
    SaveChoice::Cancel
}

pub(crate) fn mark_document(_window: &WindowControls, _edited: bool, _url: Option<&Path>) {}
