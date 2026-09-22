use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use crate::backend::capabilities::Capabilities;
use crate::backend::mklive::Init;
use crate::backend::packages::{package_exists, PackageCheck};

const ARCHES: &[&str] = &["x86_64", "x86_64-musl", "aarch64", "armv7l"];
const VARIANTS: &[&str] = &["base", "xfce", "mate", "cinnamon", "gnome", "kde", "lxde", "lxqt", "enlightenment"];

pub struct OptionsFields {
    pub arch: String,
    pub init: Option<Init>,
    pub variant: String,
    pub kernel: Option<String>,
    pub packages: Vec<String>,
    pub output: PathBuf,
}

pub struct OptionsPage {
    pub widget: gtk::Box,
    arch_dropdown: gtk::DropDown,
    init_dropdown: gtk::DropDown,
    init_row: gtk::Box,
    variant_dropdown: gtk::DropDown,
    kernel_entry: gtk::Entry,
    output_entry: gtk::Entry,
    packages: Rc<RefCell<Vec<String>>>,
}

impl OptionsPage {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let arch_dropdown = gtk::DropDown::from_strings(ARCHES);
        let init_dropdown = gtk::DropDown::from_strings(&["runit", "dinit"]);
        let variant_dropdown = gtk::DropDown::from_strings(VARIANTS);
        let kernel_entry = gtk::Entry::builder().placeholder_text("linux (leave empty for default)").build();
        let output_entry = gtk::Entry::builder().placeholder_text("/path/to/output.iso").hexpand(true).build();
        let output_button = gtk::Button::with_label("Choose…");

        let output_row = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(6).build();
        output_row.append(&output_entry);
        output_row.append(&output_button);

        let init_row = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(12).build();
        init_row.append(&gtk::Label::builder().label("Init system").width_chars(16).xalign(0.0).build());
        init_row.append(&init_dropdown);
        init_row.set_visible(false);

        for (label, child) in [
            ("Architecture", arch_dropdown.clone().upcast::<gtk::Widget>()),
            ("Desktop / variant", variant_dropdown.clone().upcast::<gtk::Widget>()),
            ("Kernel", kernel_entry.clone().upcast::<gtk::Widget>()),
            ("Output ISO path", output_row.clone().upcast::<gtk::Widget>()),
        ] {
            let row = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(12).build();
            row.append(&gtk::Label::builder().label(label).width_chars(16).xalign(0.0).build());
            row.append(&child);
            widget.append(&row);
        }
        widget.insert_child_after(&init_row, widget.first_child().as_ref());

        output_button.connect_clicked({
            let output_entry = output_entry.clone();
            move |button| {
                let dialog = gtk::FileDialog::builder().title("Choose output ISO path").build();
                let root = button.root().and_downcast::<gtk::Window>();
                let output_entry = output_entry.clone();
                dialog.save(root.as_ref(), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            output_entry.set_text(&path.to_string_lossy());
                        }
                    }
                });
            }
        });

        let packages: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let packages_entry = gtk::Entry::builder().placeholder_text("package name").hexpand(true).build();
        let packages_add_button = gtk::Button::with_label("Add");
        let packages_status = gtk::Label::builder().xalign(0.0).build();
        let packages_list = gtk::ListBox::builder().selection_mode(gtk::SelectionMode::None).build();

        let packages_input_row = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(6).build();
        packages_input_row.append(&packages_entry);
        packages_input_row.append(&packages_add_button);

        let packages_label_row = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(12).build();
        packages_label_row.append(&gtk::Label::builder().label("Extra packages").width_chars(16).xalign(0.0).build());
        packages_label_row.append(&packages_input_row);

        widget.append(&packages_label_row);
        widget.append(&packages_status);
        widget.append(&packages_list);

        let add_package: Rc<dyn Fn()> = {
            let packages_entry = packages_entry.clone();
            let packages_status = packages_status.clone();
            let packages_list = packages_list.clone();
            let packages = packages.clone();
            Rc::new(move || {
                let name = packages_entry.text().trim().to_string();
                if name.is_empty() {
                    return;
                }
                match package_exists(&name) {
                    Ok(PackageCheck::Exists) => {
                        packages.borrow_mut().push(name.clone());
                        packages_list.append(&package_row(&name, &packages, &packages_list));
                        packages_entry.set_text("");
                        packages_status.set_label("");
                    }
                    Ok(PackageCheck::NotFound) => {
                        packages_status.set_label(&format!("Package '{name}' not found"));
                    }
                    Err(e) => {
                        packages_status.set_label(&format!("Could not check package: {e}"));
                    }
                }
            })
        };

        packages_add_button.connect_clicked({
            let add_package = add_package.clone();
            move |_| add_package()
        });
        packages_entry.connect_activate({
            let add_package = add_package.clone();
            move |_| add_package()
        });

        Self {
            widget,
            arch_dropdown,
            init_dropdown,
            init_row,
            variant_dropdown,
            kernel_entry,
            output_entry,
            packages,
        }
    }

    pub fn set_capabilities(&self, caps: Capabilities) {
        self.init_row.set_visible(caps.supports_init);
    }

    pub fn collect(&self) -> (OptionsFields, Vec<(String, String)>) {
        let output_text = self.output_entry.text().to_string();
        let mut errors = Vec::new();
        if output_text.trim().is_empty() {
            errors.push(("output".to_string(), "Output ISO path is required".to_string()));
        }

        let kernel_text = self.kernel_entry.text().to_string();
        let fields = OptionsFields {
            arch: ARCHES[self.arch_dropdown.selected() as usize].to_string(),
            init: if self.init_row.is_visible() {
                Some(if self.init_dropdown.selected() == 1 { Init::Dinit } else { Init::Runit })
            } else {
                None
            },
            variant: VARIANTS[self.variant_dropdown.selected() as usize].to_string(),
            kernel: if kernel_text.trim().is_empty() { None } else { Some(kernel_text) },
            packages: self.packages.borrow().clone(),
            output: PathBuf::from(output_text),
        };

        (fields, errors)
    }
}

fn package_row(name: &str, packages: &Rc<RefCell<Vec<String>>>, packages_list: &gtk::ListBox) -> gtk::ListBoxRow {
    let row_box = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(6).build();
    row_box.append(&gtk::Label::builder().label(name).hexpand(true).xalign(0.0).build());
    let remove_button = gtk::Button::with_label("Remove");
    row_box.append(&remove_button);

    let row = gtk::ListBoxRow::builder().child(&row_box).build();

    remove_button.connect_clicked({
        let packages = packages.clone();
        let packages_list = packages_list.clone();
        let row = row.clone();
        let name = name.to_string();
        move |_| {
            packages.borrow_mut().retain(|p| p != &name);
            packages_list.remove(&row);
        }
    });

    row
}
