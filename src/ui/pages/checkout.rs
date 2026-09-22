use std::path::PathBuf;

use adw::prelude::*;
use gtk::gio;

use crate::backend::capabilities::{probe, Capabilities};

pub struct CheckoutFields {
    pub checkout: PathBuf,
    pub capabilities: Capabilities,
}

pub struct CheckoutPage {
    pub widget: gtk::Box,
    entry: gtk::Entry,
}

impl CheckoutPage {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let entry = gtk::Entry::builder()
            .placeholder_text("/path/to/void-mklive or noid-mklive checkout")
            .hexpand(true)
            .build();
        let choose_button = gtk::Button::with_label("Browse…");

        let row = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(6).build();
        row.append(&entry);
        row.append(&choose_button);
        widget.append(&row);

        choose_button.connect_clicked({
            let entry = entry.clone();
            move |button| {
                let dialog = gtk::FileDialog::builder().title("Select mklive checkout").build();
                let root = button.root().and_downcast::<gtk::Window>();
                let entry = entry.clone();
                dialog.select_folder(root.as_ref(), gio::Cancellable::NONE, move |result| {
                    if let Ok(folder) = result {
                        if let Some(path) = folder.path() {
                            entry.set_text(&path.to_string_lossy());
                        }
                    }
                });
            }
        });

        Self { widget, entry }
    }

    pub fn collect(&self) -> (CheckoutFields, Vec<(String, String)>) {
        let text = self.entry.text().to_string();
        let path = PathBuf::from(&text);
        match probe(&path) {
            Ok(capabilities) => (
                CheckoutFields { checkout: path, capabilities },
                Vec::new(),
            ),
            Err(_) => (
                CheckoutFields { checkout: path, capabilities: Capabilities { supports_init: false } },
                vec![("checkout".to_string(), "mkiso.sh not found at this path".to_string())],
            ),
        }
    }
}
