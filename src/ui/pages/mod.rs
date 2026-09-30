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
use gtk::gio;

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

/// An editable list of names (packages, services, repo URLs). Typing several
/// whitespace-separated names adds them all. With `check_packages`, each entry
/// is looked up with `xbps-query -R` in the background and flagged if missing.
pub fn list_group(
    s: &Shared,
    title: &str,
    description: &str,
    get: fn(&Profile) -> &Vec<String>,
    get_mut: fn(&mut Profile) -> &mut Vec<String>,
    check_packages: bool,
) -> adw::PreferencesGroup {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::backend::packages::{package_exists, PackageCheck};

    let group = adw::PreferencesGroup::builder().title(title).description(description).build();
    let add = adw::EntryRow::builder().title("Add…").show_apply_button(true).build();
    group.add(&add);

    let rows: Rc<RefCell<Vec<adw::ActionRow>>> = Rc::default();
    let rendered: Rc<RefCell<Vec<String>>> = Rc::default();
    let lookups: Rc<RefCell<HashMap<String, PackageCheck>>> = Rc::default();

    let render: Rc<dyn Fn(&Profile)> = {
        let (group, rows, rendered, lookups, s) = (group.clone(), rows.clone(), rendered.clone(), lookups.clone(), s.clone());
        Rc::new(move |p: &Profile| {
            let items = get(p);
            if *rendered.borrow() == *items {
                return;
            }
            for row in rows.borrow_mut().drain(..) {
                group.remove(&row);
            }
            for item in items {
                let row = adw::ActionRow::builder().title(item).build();
                let status = gtk::Image::new();
                row.add_prefix(&status);
                if check_packages {
                    flag(&status, lookups.borrow().get(item));
                    if !lookups.borrow().contains_key(item) {
                        let (name, lookups, status) = (item.clone(), lookups.clone(), status.clone());
                        glib::spawn_future_local(async move {
                            let lookup = name.clone();
                            let result = gio::spawn_blocking(move || package_exists(&lookup)).await;
                            if let Ok(Ok(check)) = result {
                                flag(&status, Some(&check));
                                lookups.borrow_mut().insert(name, check);
                            }
                        });
                    }
                }
                let remove = gtk::Button::builder()
                    .icon_name("list-remove-symbolic")
                    .valign(gtk::Align::Center)
                    .css_classes(["flat"])
                    .tooltip_text("Remove")
                    .build();
                let (s, name) = (s.clone(), item.clone());
                remove.connect_clicked(move |_| state::update(&s, |p| get_mut(p).retain(|x| *x != name)));
                row.add_suffix(&remove);
                group.add(&row);
                rows.borrow_mut().push(row);
            }
            *rendered.borrow_mut() = items.clone();
        })
    };

    fn flag(status: &gtk::Image, check: Option<&PackageCheck>) {
        match check {
            Some(PackageCheck::NotFound) => {
                status.set_icon_name(Some("dialog-warning-symbolic"));
                status.set_tooltip_text(Some("Not found in the configured repositories"));
            }
            _ => status.set_icon_name(None),
        }
    }

    render(&s.borrow().profile);
    {
        let render = render.clone();
        state::on_change(s, move |p, _, _| render(p));
    }
    let s = s.clone();
    add.connect_apply(move |entry| {
        let text = entry.text().to_string();
        state::update(&s, |p| {
            let list = get_mut(p);
            for word in text.split_whitespace() {
                if !list.iter().any(|x| x == word) {
                    list.push(word.to_string());
                }
            }
        });
        entry.set_text("");
    });
    group
}

/// A row that picks a file or folder into an optional path field, with a clear button.
pub fn path_row(
    s: &Shared,
    title: &str,
    folder: bool,
    get: fn(&Profile) -> Option<std::path::PathBuf>,
    set: fn(&mut Profile, Option<std::path::PathBuf>),
) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(title).build();
    let subtitle = move |p: &Profile| get(p).map(|x| x.display().to_string()).unwrap_or_else(|| "None".into());
    row.set_subtitle(&subtitle(&s.borrow().profile));

    let choose = gtk::Button::builder().label("Choose…").valign(gtk::Align::Center).build();
    let clear = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .tooltip_text("Clear")
        .build();
    row.add_suffix(&choose);
    row.add_suffix(&clear);

    {
        let s = s.clone();
        let title = title.to_string();
        choose.connect_clicked(move |btn| {
            let dialog = gtk::FileDialog::builder().title(&title).build();
            let window = btn.root().and_downcast::<gtk::Window>();
            let s = s.clone();
            let done = move |res: Result<gio::File, glib::Error>| {
                if let Some(path) = res.ok().and_then(|f| f.path()) {
                    state::update(&s, |p| set(p, Some(path)));
                }
            };
            if folder {
                dialog.select_folder(window.as_ref(), gio::Cancellable::NONE, done);
            } else {
                dialog.open(window.as_ref(), gio::Cancellable::NONE, done);
            }
        });
    }
    {
        let s = s.clone();
        clear.connect_clicked(move |_| state::update(&s, |p| set(p, None)));
    }
    let row2 = row.clone();
    state::on_change(s, move |p, _, _| row2.set_subtitle(&subtitle(p)));
    row
}
