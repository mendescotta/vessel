use std::path::PathBuf;

use adw::prelude::*;
use gtk::gio;

use crate::profile::Profile;
use crate::ui::pages::show_error;
use crate::ui::state::{self, Shared};

fn toml_filter() -> gio::ListStore {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("vessel profiles (*.toml)"));
    filter.add_pattern("*.toml");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    filters
}

pub fn open_profile(s: &Shared, parent: &impl IsA<gtk::Widget>, then: impl Fn() + 'static) {
    let dialog = gtk::FileDialog::builder()
        .title("Open profile")
        .filters(&toml_filter())
        .build();
    let window = parent.root().and_downcast::<gtk::Window>();
    let s = s.clone();
    let parent = parent.clone().upcast::<gtk::Widget>();
    dialog.open(window.as_ref(), gio::Cancellable::NONE, move |res| {
        let Some(path) = res.ok().and_then(|f| f.path()) else {
            return;
        };
        match Profile::load(&path) {
            Ok(p) => {
                state::replace(&s, p);
                s.borrow_mut().profile_path = Some(path);
                then();
            }
            Err(e) => show_error(&parent, "Couldn't open profile", &e.to_string()),
        }
    });
}

pub fn save_profile(s: &Shared, parent: &impl IsA<gtk::Widget>, save_as: bool) {
    let existing = s.borrow().profile_path.clone();
    match existing {
        Some(path) if !save_as => write(s, parent.upcast_ref(), path),
        _ => {
            let name = format!("{}.toml", s.borrow().profile.name);
            let dialog = gtk::FileDialog::builder()
                .title("Save profile")
                .initial_name(&name)
                .filters(&toml_filter())
                .build();
            let window = parent.root().and_downcast::<gtk::Window>();
            let s = s.clone();
            let parent = parent.clone().upcast::<gtk::Widget>();
            dialog.save(window.as_ref(), gio::Cancellable::NONE, move |res| {
                if let Some(path) = res.ok().and_then(|f| f.path()) {
                    write(&s, &parent, path);
                }
            });
        }
    }
}

fn write(s: &Shared, parent: &gtk::Widget, path: PathBuf) {
    let result = s.borrow().profile.save(&path);
    match result {
        Ok(()) => s.borrow_mut().profile_path = Some(path),
        Err(e) => show_error(parent, "Couldn't save profile", &e.to_string()),
    }
}
