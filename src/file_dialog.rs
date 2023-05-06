use imgui::Condition;
use std::cmp::Ordering;
use std::fs;
use std::path::{PathBuf};

/// The file dialog offered by the crate for use with ImGui.
///
/// This type holds the definitions of the file dialog that this crate offers.
/// Using it involves a builder pattern, however the type wasn't named `FileDialogBuilder`
/// to make the crate remain minimal. To get started, create a new [`FileDialog`] using
/// [`FileDialog::new()`](crate::file_dialog::FileDialog::new):
/// ```no_run
/// use imfile::FileDialog;
/// // ...
///
/// let file_dialog = FileDialog::new();
/// ```
/// In order to "spawn" the dialog, you can use either [`spawn_borrowed`](crate::file_dialog::FileDialog::spawn_borrowed)
/// or [`spawn`](crate::file_dialog::FileDialog::spawn), the former intended to be used when you wish to reuse the same dialog
/// multiple times:
/// ```no_run
/// if let Some(filename) = file_dialog.spawn_borrowed(&ui) {
///     println!("Filename given: {}", filename.display());
/// }
/// ```
pub struct FileDialog {
    accept_text: String,
    cancel_text: String,
    title: String,
    filename: String, 
    is_open: bool,
    dirs_only: bool,
    show_hidden_files: bool,
}

impl FileDialog {
    /// Creates a new file dialog and returns it for future usage.
    /// You can also use [`FileDialog::default()`] since it does the same thing.
    #[inline]
    pub fn new() -> Self {
        Self {
            accept_text: String::from("Open"),
            cancel_text: String::from("Cancel"),
            title: String::from("Open File"),
            filename: String::new(),
            is_open: true,
            dirs_only: false,
            show_hidden_files: false
        }
    }

    /// Sets the title of the dialog.
    #[inline]
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the accept ("Open") text for the dialog.
    #[inline]
    pub fn accept_text<S: Into<String>>(mut self, accept_text: S) -> Self {
        self.accept_text = accept_text.into();
        self
    }

    /// Sets the reject text for the dialog.
    #[inline]
    pub fn cancel_text<S: Into<String>>(mut self, cancel_text: S) -> Self {
        self.cancel_text = cancel_text.into();
        self
    }

    /// Sets whether the dialog may be used exclusively to open directories.
    #[inline]
    pub fn dir_only(mut self) -> Self {
        self.dirs_only = true;
        self
    }

    /// Sets the dialog for save.
    #[inline]
    pub fn for_save(mut self) -> Self {
        self.is_open   = false;
        self.dirs_only = false;
        self
    }

    /// Spawns the dialog.
    ///
    /// This function spawns the dialog and optionally (Depending on whether the user chose an entry)
    /// returns a [`PathBuf`] with the path to the chosen file.\
    /// This is the **owned** version of the `spawn*` family of functions. After calling this function, you won't
    /// be able to reuse [`self`]. If you wish to continue owning [`self`], then see [`FileDialog::spawn_borrowed()`].
    ///
    /// **WARNING**: This dialog expects you to have a [`Ui`](imgui::Ui) ready that the function will immutably borrow.
    /// See the documentation of [imgui] for details.
    pub fn spawn(mut self, ui: &imgui::Ui) -> Option<PathBuf> {
        let mut path = None;
        ui.window(self.title.clone())
            .size([600.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                ui.child_window("Path Selection")
                    .horizontal_scrollbar(false)
                    .border(true)
                    .size([0.0, 32.0])
                    .build(||{
                        ui.button("Path: ");
                        ui.same_line();
                        std::env::current_dir().unwrap().iter().for_each(|dir|{
                            if ui.button(dir.to_string_lossy()) {
                                std::env::set_current_dir(dir)
                                    .map_err(|err| log::error!("Can't change directory to {}: {}", dir.to_string_lossy(), err.to_string()))
                                    .ok();
                            }
                            if ui.is_item_hovered() {
                                ui.tooltip_text(format!("Directory: {}", dir.to_string_lossy()));
                            }
                            ui.same_line();
                        })
                    });
                ui.child_window("Select file / directory")
                    .border(true)
                    .size([0.0, -32.0])
                    .build(|| {
                        let mut entries: Vec<_> = fs::read_dir(std::env::current_dir().unwrap())
                            .unwrap()
                            .filter_map(|entry| {
                                let entry = entry.expect("Filesystem entry error");
                                if self.show_hidden_files {
                                   Some(entry) 
                                } else {
                                    if !entry.path().starts_with(".") {
                                        Some(entry)
                                    } else {
                                        None
                                    }
                                }
                            })
                            .collect();
                        /* Sorting directories first to make it easier to navigate */
                        entries.sort_by(|a, b| {
                            if a.path().is_dir() && !b.path().is_dir() {
                                Ordering::Less
                            } else if !a.path().is_dir() && b.path().is_dir() {
                                Ordering::Greater
                            } else {
                                a.path().cmp(&b.path())
                            }
                        });
                        for entry in entries {
                            if entry.path().is_file() && !self.dirs_only {
                                if ui.button(format!("[file]\t{}", PathBuf::from(entry.path().iter().last().unwrap()).display())) {
                                    path = Some(entry.path());
                                }
                            } else if entry.path().is_dir() {
                                if ui.button(format!("[dir] \t{}", PathBuf::from(entry.path().iter().last().unwrap()).display())) {
                                    std::env::set_current_dir(entry.path())
                                        .map_err(|e|{
                                            log::error!("Can't access '{}': {}", entry.path().display(), e.to_string());
                                            path = None;
                                        })
                                        .ok();
                                }
                            }
                        }
                    });
                    ui.child_window("controls")
                        .border(false)
                        .build(||{
                            if !self.is_open {
                                ui.text(format!("Filename: {}", self.filename));
                            }
                            ui.same_line();
                            if ui.button("Back") {
                                let dir = {
                                    let mut tmp = std::env::current_dir().unwrap();
                                    tmp.pop();
                                    tmp
                                };
                                std::env::set_current_dir(dir).ok();
                            }
                            ui.same_line();
                            ui.button("Open");
                            ui.same_line();
                            if ui.checkbox("Hidden Files", &mut self.show_hidden_files) {
                                self.show_hidden_files = !self.show_hidden_files;
                            }
                        })
            });
            path
    }
}

impl Default for FileDialog {
    fn default() -> Self {
        Self::new()
    }
}
