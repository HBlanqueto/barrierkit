use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use freya_core::prelude::*;

use crate::controlled::use_latest;
use crate::environment::use_environment;
use crate::platform::{WindowControls, panels};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentType {
    Folder,
    Extension(String),
}

impl ContentType {
    /// A file kind by its extension, without the dot: `ContentType::extension("png")`.
    pub fn extension(extension: impl Into<String>) -> Self {
        Self::Extension(extension.into())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileDialog {
    pub(crate) title: String,
    pub(crate) save_as: Option<String>,
    pub(crate) multiple: bool,
    pub(crate) types: Vec<ContentType>,
    pub(crate) directory: Option<PathBuf>,
}

impl FileDialog {
    pub fn open() -> Self {
        Self::default()
    }

    /// A save dialog that proposes `name`.
    pub fn save(name: impl Into<String>) -> Self {
        Self {
            save_as: Some(name.into()),
            ..Self::default()
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// An empty list allows every file; `ContentType::Folder` alone chooses a folder.
    pub fn allowed_content_types(mut self, types: Vec<ContentType>) -> Self {
        self.types = types;
        self
    }

    pub fn allows_multiple_selection(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// The folder the dialog starts in.
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    /// Shows the dialog as a sheet of `window` and returns the chosen paths; empty when the user cancels.
    pub async fn present(self, window: &WindowControls) -> Vec<PathBuf> {
        panels::present(self, window).await
    }

    pub(crate) fn chooses_folders(&self) -> bool {
        self.types.contains(&ContentType::Folder)
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct DefaultDirectory(pub(crate) Option<PathBuf>);

pub(crate) fn use_file_dialog(
    mut presented: State<bool>,
    dialog: FileDialog,
    on_completion: Callback<Vec<PathBuf>, ()>,
) {
    let directory = use_environment::<DefaultDirectory>().0;
    let dialog = FileDialog {
        directory: dialog.directory.or(directory),
        ..dialog
    };
    let latest = use_latest((dialog, on_completion));
    let showing = use_hook(|| Rc::new(Cell::new(false)));
    let window = try_consume_root_context::<WindowControls>();
    use_side_effect(move || {
        if !*presented.read() || showing.replace(true) {
            return;
        }
        let Some(window) = window.clone() else {
            presented.set(false);
            showing.set(false);
            return;
        };
        let (dialog, on_completion) = latest.borrow().clone();
        let showing = showing.clone();
        spawn(async move {
            let chosen = dialog.present(&window).await;
            showing.set(false);
            presented.set(false);
            if !chosen.is_empty() {
                on_completion.call(chosen);
            }
        });
    });
}
