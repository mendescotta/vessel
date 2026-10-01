use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;

use crate::profile::Profile;
use crate::ui::files::{open_profile, save_profile};
use crate::ui::pages::{self, Navigate, ORDER, REVIEW};
use crate::ui::state::{self, Shared};

pub fn build(app: &adw::Application) -> adw::ApplicationWindow {
    let s: Shared = state::new_shared(Profile::new_default());

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("vessel")
        .default_width(1000)
        .default_height(720)
        .build();

    let stack = gtk::Stack::builder().transition_type(gtk::StackTransitionType::Crossfade).build();
    let index = Rc::new(Cell::new(0usize));
    let back = gtk::Button::builder().label("Back").build();
    let next = gtk::Button::builder().label("Next").css_classes(["suggested-action"]).build();

    let refresh_nav: Rc<dyn Fn()> = {
        let (stack, index, back, next, s) = (stack.clone(), index.clone(), back.clone(), next.clone(), s.clone());
        Rc::new(move || {
            if let Some(i) = stack.visible_child_name().and_then(|n| ORDER.iter().position(|(name, _)| *name == n)) {
                index.set(i);
            }
            let i = index.get();
            back.set_sensitive(i > 0);
            next.set_visible(i + 1 < ORDER.len());
            let blocked = ORDER[i].0 == REVIEW && s.borrow().valid.is_none();
            next.set_sensitive(!blocked);
        })
    };
    let nav: Navigate = {
        let stack = stack.clone();
        Rc::new(move |name: &str| stack.set_visible_child_name(name))
    };

    for (name, title) in ORDER {
        let child = match *name {
            pages::START => pages::start::build(&s, nav.clone()),
            pages::SYSTEM => pages::system::build(&s),
            pages::BOOT => pages::boot::build(&s),
            pages::DESKTOP => pages::desktop::build(&s),
            pages::REPOS => pages::repos::build(&s),
            pages::REVIEW => pages::review::build(&s),
            _ => pages::output::build(&s),
        };
        stack.add_titled(&child, Some(name), title);
    }
    {
        let refresh_nav = refresh_nav.clone();
        stack.connect_visible_child_name_notify(move |_| refresh_nav());
    }
    {
        let refresh_nav = refresh_nav.clone();
        state::on_change(&s, move |_, _, _| refresh_nav());
    }
    {
        let (index, nav) = (index.clone(), nav.clone());
        back.connect_clicked(move |_| {
            if index.get() > 0 {
                nav(ORDER[index.get() - 1].0);
            }
        });
    }
    {
        let (index, nav) = (index.clone(), nav.clone());
        next.connect_clicked(move |_| {
            if index.get() + 1 < ORDER.len() {
                nav(ORDER[index.get() + 1].0);
            }
        });
    }

    let sidebar = gtk::StackSidebar::builder().stack(&stack).vexpand(true).build();
    let sidebar_view = adw::ToolbarView::new();
    sidebar_view.add_top_bar(&adw::HeaderBar::builder().show_title(false).build());
    sidebar_view.set_content(Some(&sidebar));

    let header = adw::HeaderBar::new();
    let open_btn = gtk::Button::builder().icon_name("document-open-symbolic").tooltip_text("Open profile").build();
    let save_btn = gtk::Button::builder().icon_name("document-save-symbolic").tooltip_text("Save profile").build();
    let save_as_btn = gtk::Button::builder().icon_name("document-save-as-symbolic").tooltip_text("Save profile as…").build();
    header.pack_start(&open_btn);
    header.pack_end(&save_as_btn);
    header.pack_end(&save_btn);
    {
        let (s, nav) = (s.clone(), nav.clone());
        open_btn.connect_clicked(move |btn| {
            let nav = nav.clone();
            open_profile(&s, btn, move || nav(pages::SYSTEM));
        });
    }
    {
        let s = s.clone();
        save_btn.connect_clicked(move |btn| save_profile(&s, btn, false));
    }
    {
        let s = s.clone();
        save_as_btn.connect_clicked(move |btn| save_profile(&s, btn, true));
    }

    let bottom = gtk::Box::builder()
        .spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .build();
    bottom.append(&back);
    bottom.append(&gtk::Box::builder().hexpand(true).build());
    bottom.append(&next);

    let content_view = adw::ToolbarView::new();
    content_view.add_top_bar(&header);
    content_view.set_content(Some(&stack));
    content_view.add_bottom_bar(&bottom);

    let split = adw::NavigationSplitView::builder()
        .sidebar(&adw::NavigationPage::builder().title("vessel").child(&sidebar_view).build())
        .content(&adw::NavigationPage::builder().title("Profile").child(&content_view).build())
        .build();
    window.set_content(Some(&split));
    refresh_nav();
    window
}
