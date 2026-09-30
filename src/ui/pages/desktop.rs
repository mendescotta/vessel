use adw::prelude::*;

use super::{enum_row, issues_group, page, switch_row, toggle_in};
use crate::profile::{Desktop, DisplayManager};
use crate::ui::state::Shared;

pub fn build(s: &Shared) -> gtk::Widget {
    let page = page(
        "Desktop",
        "Choose any number of desktops (none gives a console-only ISO) and one display manager to log in to them.",
    );
    page.add(&issues_group(s, &["display_manager"]));

    let desktops = adw::PreferencesGroup::builder().title("Desktops").build();
    for d in Desktop::ALL.iter().copied() {
        desktops.add(&switch_row(
            s,
            d.label(),
            None,
            move |p| p.desktops.contains(&d),
            move |p, on| toggle_in(&mut p.desktops, d, on),
        ));
    }
    page.add(&desktops);

    let dm = adw::PreferencesGroup::builder().title("Login").build();
    dm.add(&enum_row(
        s,
        "Display manager",
        DisplayManager::ALL,
        DisplayManager::label,
        |p| p.display_manager,
        |p, v| p.display_manager = v,
    ));
    page.add(&dm);

    page.upcast()
}
