use adw::prelude::*;
use gtk::gio;

use super::{show_error, Navigate, SYSTEM};
use crate::profile::Profile;
use crate::snapshot::{snapshot, RealHost};
use crate::ui::files::open_profile;
use crate::ui::state::{self, Shared};

pub fn build(s: &Shared, nav: Navigate) -> gtk::Widget {
    let status = adw::StatusPage::builder()
        .icon_name("media-optical-symbolic")
        .title("vessel")
        .description("Compose a Void Linux live ISO: pick an init, userland, bootloaders, desktops and repositories, then generate a standalone build script.")
        .build();

    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .halign(gtk::Align::Center)
        .build();
    let new_btn = gtk::Button::builder().label("New profile").css_classes(["pill", "suggested-action"]).build();
    let open_btn = gtk::Button::builder().label("Open profile…").css_classes(["pill"]).build();
    let seed_btn = gtk::Button::builder().label("Seed from this system").css_classes(["pill"]).build();
    buttons.append(&new_btn);
    buttons.append(&open_btn);
    buttons.append(&seed_btn);
    status.set_child(Some(&buttons));

    {
        let s = s.clone();
        let nav = nav.clone();
        new_btn.connect_clicked(move |_| {
            state::replace(&s, Profile::new_default());
            s.borrow_mut().profile_path = None;
            nav(SYSTEM);
        });
    }
    {
        let s = s.clone();
        let nav = nav.clone();
        open_btn.connect_clicked(move |btn| {
            let nav = nav.clone();
            open_profile(&s, btn, move || nav(SYSTEM));
        });
    }
    {
        let s = s.clone();
        seed_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            btn.set_label("Reading this system…");
            let s = s.clone();
            let nav = nav.clone();
            let btn = btn.clone();
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(|| snapshot(&RealHost)).await;
                btn.set_sensitive(true);
                btn.set_label("Seed from this system");
                let Ok((profile, warnings)) = result else {
                    show_error(&btn, "Snapshot failed", "Reading the system panicked; see the terminal output.");
                    return;
                };
                state::replace(&s, profile);
                s.borrow_mut().profile_path = None;
                nav(SYSTEM);
                if !warnings.is_empty() {
                    show_error(&btn, "Seeded with warnings", &warnings.join("\n"));
                }
            });
        });
    }

    status.upcast()
}
