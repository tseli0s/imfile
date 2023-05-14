# Rust-only file dialog for ImGui
<h6><i><b>NOTE:</b> This crate is still a work in progress. Some functionality is still not implemented.</i></h6>
This crate offers a 100% Rust file dialog for ImGui. I created this library as there are basically no other
solutions for this matter.
While this crate primarily targets [imgui-rs](https://crates.io/crates/imgui/), it should be possible to port
it to C++ with a some modifications. This may be important for projects striving for 100% safety.

# Features
- Lightweight and simple file dialog with embedded file browser
- Compatible with `imgui-rs` >= 0.11.0
- No extra dependencies

# Example
Basic usage:
```rust
use imfile::FileDialog;

fn main() {
    // set up your imgui::Ui here

    // This returns None if no file was selected
    if let Some(file) = FileDialog::new()
        .for_save() // Default is open
        .title("Title") // Default is "Open File" or "Save file" depending on the dialog type
        .accept_text("Open file") // Default is open
        .dir_only() // Only allow directories instead of files
        .spawn(&ui); // Create the dialog using the imgui::Ui
    {
        println!("File chosen: {}", file.display());
    } else {
        println!("No file selected.");
    }
}
```

# TODOs
- Add icons for the widgets
- Add file filters
- Set side panel navigator (eg. Disk, Recents, ...)

# License
The crate is licensed under the MIT license.
