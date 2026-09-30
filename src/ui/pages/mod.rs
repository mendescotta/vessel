//! Wizard pages plus the small row helpers they share. Every helper keeps its
//! widget and the shared profile in sync both ways: user edits go through
//! `state::update`, and a `Change::Replace` (open / new / snapshot) reloads
//! the widget from the new profile.

pub mod boot;
pub mod desktop;
pub mod output;
pub mod repos;
pub mod review;
pub mod start;
pub mod system;

use std::rc::Rc;

use adw::prelude::*;

use crate::profile::validate::{Issue, Severity};
use crate::profile::Profile;
use crate::ui::state::{self, Change, Shared};

/// Switches the wizard to the page with this name.
pub type Navigate = Rc<dyn Fn(&str)>;

pub const START: &str = "start";
pub const SYSTEM: &str = "system";
pub const BOOT: &str = "boot";
pub const DESKTOP: &str = "desktop";
pub const REPOS: &str = "repos";
pub const REVIEW: &str = "review";
pub const OUTPUT: &str = "output";

/// (name, sidebar title) in wizard order.
pub const ORDER: &[(&str, &str)] = &[
    (START, "Start"),
    (SYSTEM, "System"),
    (BOOT, "Boot"),
    (DESKTOP, "Desktop"),
    (REPOS, "Repos & packages"),
    (REVIEW, "Review"),
    (OUTPUT, "Output & build"),
];

/// A list of the current issues for `fields` (all fields when empty), kept up to date.
pub fn issues_group(s: &Shared, fields: &'static [&'static str]) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();
    let list = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(4).build();
    group.add(&list);

    let render = {
        let group = group.clone();
        let list = list.clone();
        move |issues: &[Issue]| {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            let mut any = false;
            for issue in issues.iter().filter(|i| fields.is_empty() || fields.contains(&i.field)) {
                any = true;
                let (icon, class) = match issue.severity {
                    Severity::Error => ("dialog-error-symbolic", "error"),
                    Severity::Warning => ("dialog-warning-symbolic", "warning"),
                };
                let row = gtk::Box::builder().spacing(8).build();
                row.append(&gtk::Image::from_icon_name(icon));
                row.append(&gtk::Label::builder().label(&issue.message).wrap(true).xalign(0.0).build());
                row.add_css_class(class);
                list.append(&row);
            }
            group.set_visible(any);
        }
    };
    render(&s.borrow().issues);
    state::on_change(s, move |_, issues, _| render(issues));
    group
}

/// A combo row over an axis enum.
pub fn enum_row<T: Copy + PartialEq + 'static>(
    s: &Shared,
    title: &str,
    all: &'static [T],
    label: fn(T) -> &'static str,
    get: fn(&Profile) -> T,
    set: fn(&mut Profile, T),
) -> adw::ComboRow {
    let labels: Vec<&str> = all.iter().map(|v| label(*v)).collect();
    let row = adw::ComboRow::builder().title(title).model(&gtk::StringList::new(&labels)).build();
    let index_of = move |v: T| all.iter().position(|x| *x == v).unwrap_or(0) as u32;
    row.set_selected(index_of(get(&s.borrow().profile)));

    let s2 = s.clone();
    row.connect_selected_notify(move |row| {
        if let Some(v) = all.get(row.selected() as usize) {
            let v = *v;
            state::update(&s2, |p| set(p, v));
        }
    });
    let row2 = row.clone();
    state::on_change(s, move |p, _, change| {
        if change == Change::Replace && row2.selected() != index_of(get(p)) {
            row2.set_selected(index_of(get(p)));
        }
    });
    row
}

/// A text entry row bound to a string field.
pub fn text_row(s: &Shared, title: &str, get: fn(&Profile) -> String, set: fn(&mut Profile, String)) -> adw::EntryRow {
    let row = adw::EntryRow::builder().title(title).text(get(&s.borrow().profile)).build();
    let s2 = s.clone();
    row.connect_changed(move |row| {
        let text = row.text().to_string();
        state::update(&s2, |p| set(p, text));
    });
    let row2 = row.clone();
    state::on_change(s, move |p, _, change| {
        if change == Change::Replace && row2.text() != get(p) {
            row2.set_text(&get(p));
        }
    });
    row
}

/// A switch row bound to a boolean view of the profile (a flag or list membership).
pub fn switch_row(
    s: &Shared,
    title: &str,
    subtitle: Option<&str>,
    get: impl Fn(&Profile) -> bool + 'static,
    set: impl Fn(&mut Profile, bool) + 'static,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder().title(title).build();
    if let Some(sub) = subtitle {
        row.set_subtitle(sub);
    }
    row.set_active(get(&s.borrow().profile));
    let get = Rc::new(get);
    let s2 = s.clone();
    row.connect_active_notify(move |row| {
        let on = row.is_active();
        state::update(&s2, |p| set(p, on));
    });
    let row2 = row.clone();
    state::on_change(s, move |p, _, _| {
        // Membership can change from elsewhere (e.g. "add required repos"), so sync on every change.
        if row2.is_active() != get(p) {
            row2.set_active(get(p));
        }
    });
    row
}

/// Adds or removes `item` in `list`, keeping order.
pub fn toggle_in<T: PartialEq>(list: &mut Vec<T>, item: T, on: bool) {
    let present = list.contains(&item);
    if on && !present {
        list.push(item);
    } else if !on && present {
        list.retain(|x| *x != item);
    }
}

/// A scrolling preferences page with a title/description header group.
pub fn page(title: &str, description: &str) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::new();
    let header = adw::PreferencesGroup::builder().title(title).description(description).build();
    page.add(&header);
    page
}

pub fn show_error(widget: &impl IsA<gtk::Widget>, heading: &str, body: &str) {
    let dialog = adw::AlertDialog::builder().heading(heading).body(body).build();
    dialog.add_response("ok", "OK");
    dialog.present(Some(widget));
}
