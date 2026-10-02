use std::rc::Rc;

use adw::prelude::*;

use crate::generate::{init::base_packages, userland};
use crate::profile::stack::{select, support};
use crate::profile::validate::required_presets;
use crate::profile::{Init, Profile, Userland};
use crate::ui::state::{self, Shared};

struct Cell {
    init: Init,
    userland: Userland,
    button: gtk::ToggleButton,
    badge: gtk::Label,
    repos: gtk::Label,
}

pub fn group(s: &Shared) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Base system")
        .description("Pick an init (row) and a core userland (column).")
        .build();

    let grid = gtk::Grid::builder().row_spacing(8).column_spacing(8).column_homogeneous(true).margin_top(6).margin_bottom(6).build();
    for (i, u) in Userland::ALL.iter().enumerate() {
        let head = gtk::Label::builder().label(u.label()).css_classes(["heading"]).build();
        grid.attach(&head, i as i32 + 1, 0, 1, 1);
    }

    let mut cells = Vec::new();
    for (r, init) in Init::ALL.iter().copied().enumerate() {
        let name = gtk::Label::builder().label(init.label()).css_classes(["heading"]).halign(gtk::Align::Start).valign(gtk::Align::Center).build();
        grid.attach(&name, 0, r as i32 + 1, 1, 1);
        for (c, userland) in Userland::ALL.iter().copied().enumerate() {
            let badge = gtk::Label::builder().css_classes(["heading"]).build();
            let repos = gtk::Label::builder().css_classes(["caption", "dim-label"]).wrap(true).build();
            let content = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).margin_top(8).margin_bottom(8).build();
            content.append(&badge);
            content.append(&repos);
            let button = gtk::ToggleButton::builder().child(&content).hexpand(true).build();
            grid.attach(&button, c as i32 + 1, r as i32 + 1, 1, 1);
            cells.push(Cell { init, userland, button, badge, repos });
        }
    }
    let cells = Rc::new(cells);
    group.add(&grid);

    let summary = adw::ActionRow::builder().title("This selection installs").subtitle_selectable(true).build();
    group.add(&summary);

    let refresh: Rc<dyn Fn(&Profile)> = {
        let (cells, summary) = (cells.clone(), summary.clone());
        Rc::new(move |p: &Profile| {
            for cell in cells.iter() {
                cell.badge.set_label(support(cell.init, cell.userland).label());
                let mut q = p.clone();
                q.init = cell.init;
                q.userland = cell.userland;
                let needs: Vec<&str> = required_presets(&q).iter().map(|r| r.id()).collect();
                cell.repos.set_label(&if needs.is_empty() { "no extra repos".to_string() } else { format!("+ {}", needs.join(", ")) });
                cell.button.set_active(p.init == cell.init && p.userland == cell.userland);
            }
            let mut pkgs: Vec<&str> = base_packages(p.init).to_vec();
            pkgs.extend(userland::packages(p.userland));
            summary.set_subtitle(&pkgs.join(", "));
        })
    };

    for cell in cells.iter() {
        let (s, refresh, init, userland) = (s.clone(), refresh.clone(), cell.init, cell.userland);
        cell.button.connect_clicked(move |_| {
            state::update(&s, |p| select(p, init, userland));
            refresh(&s.borrow().profile);
        });
    }

    refresh(&s.borrow().profile);
    state::on_change(s, move |p, _, _| refresh(p));
    group
}
