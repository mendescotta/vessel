use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

use crate::backend::mklive::BuildOptions;
use crate::ui::pages::build::BuildPage;
use crate::ui::pages::checkout::{CheckoutFields, CheckoutPage};
use crate::ui::pages::options::{OptionsFields, OptionsPage};
use crate::ui::pages::review::ReviewPage;
use crate::ui::pages;

const CHECKOUT: usize = 0;
const OPTIONS: usize = 1;
const REVIEW: usize = 2;
const BUILD: usize = 3;

struct State {
    checkout: CheckoutPage,
    options: OptionsPage,
    review: ReviewPage,
    build: BuildPage,
    checkout_fields: Option<CheckoutFields>,
    options_fields: Option<OptionsFields>,
    stack: gtk::Stack,
    back_button: gtk::Button,
    next_button: gtk::Button,
    current_index: usize,
    window: adw::ApplicationWindow,
}

pub fn build(app: &adw::Application) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("vessel")
        .default_width(880)
        .default_height(620)
        .build();

    let checkout = CheckoutPage::new();
    let options = OptionsPage::new();
    let review = ReviewPage::new();
    let build_page = BuildPage::new();

    let stack = gtk::Stack::new();
    stack.add_titled(&checkout.widget, Some(pages::CHECKOUT), pages::CHECKOUT);
    stack.add_titled(&options.widget, Some(pages::OPTIONS), pages::OPTIONS);
    stack.add_titled(&review.widget, Some(pages::REVIEW), pages::REVIEW);
    stack.add_titled(&build_page.widget, Some(pages::BUILD), pages::BUILD);

    let header_bar = adw::HeaderBar::new();
    let content_toolbar = adw::ToolbarView::new();
    content_toolbar.add_top_bar(&header_bar);
    content_toolbar.set_content(Some(&stack));

    let back_button = gtk::Button::builder().label("Back").build();
    let next_button = gtk::Button::builder().label("Next").css_classes(["suggested-action"]).build();

    let bottom_bar = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .build();
    bottom_bar.append(&back_button);
    bottom_bar.append(&gtk::Box::builder().hexpand(true).build());
    bottom_bar.append(&next_button);
    content_toolbar.add_bottom_bar(&bottom_bar);

    window.set_content(Some(&content_toolbar));

    let state = Rc::new(RefCell::new(State {
        checkout,
        options,
        review,
        build: build_page,
        checkout_fields: None,
        options_fields: None,
        stack,
        back_button: back_button.clone(),
        next_button: next_button.clone(),
        current_index: 0,
        window: window.clone(),
    }));

    update_nav(&state);

    {
        let state = state.clone();
        back_button.connect_clicked(move |_| {
            let mut s = state.borrow_mut();
            if s.current_index > 0 && s.current_index != BUILD {
                s.current_index -= 1;
            }
            drop(s);
            update_nav(&state);
        });
    }

    {
        let state = state.clone();
        next_button.connect_clicked(move |_| on_next(&state));
    }

    window
}

fn update_nav(state: &Rc<RefCell<State>>) {
    let s = state.borrow();
    let title = match s.current_index {
        CHECKOUT => pages::CHECKOUT,
        OPTIONS => pages::OPTIONS,
        REVIEW => pages::REVIEW,
        _ => pages::BUILD,
    };
    s.stack.set_visible_child_name(title);
    s.back_button.set_sensitive(s.current_index > 0 && s.current_index != BUILD);
    s.next_button.set_visible(s.current_index != BUILD);
    s.next_button.set_label(if s.current_index == REVIEW { "Build" } else { "Next" });
}

fn show_errors(state: &Rc<RefCell<State>>, errors: &[(String, String)]) {
    let body = errors.iter().map(|(_, m)| m.as_str()).collect::<Vec<_>>().join("\n");
    let dialog = adw::AlertDialog::builder().heading("Please check your input").body(body).build();
    dialog.add_response("ok", "OK");
    dialog.present(Some(&state.borrow().window));
}

fn on_next(state: &Rc<RefCell<State>>) {
    let current = state.borrow().current_index;

    match current {
        CHECKOUT => {
            let (fields, errors) = state.borrow().checkout.collect();
            if !errors.is_empty() {
                show_errors(state, &errors);
                return;
            }
            state.borrow().options.set_capabilities(fields.capabilities);
            state.borrow_mut().checkout_fields = Some(fields);
        }
        OPTIONS => {
            let (fields, errors) = state.borrow().options.collect();
            if !errors.is_empty() {
                show_errors(state, &errors);
                return;
            }
            state.borrow_mut().options_fields = Some(fields);

            let s = state.borrow();
            let opts = build_options(&s);
            s.review.set_summary(&opts);
        }
        REVIEW => {
            let s = state.borrow();
            let opts = build_options(&s);
            s.build.start(&opts, |_success| {});
        }
        _ => {}
    }

    state.borrow_mut().current_index += 1;
    update_nav(state);
}

fn build_options(state: &State) -> BuildOptions {
    let checkout_fields = state.checkout_fields.as_ref().expect("checkout collected before options");
    let options_fields = state.options_fields.as_ref().expect("options collected before review");
    BuildOptions {
        checkout: checkout_fields.checkout.clone(),
        arch: options_fields.arch.clone(),
        init: options_fields.init,
        variant: options_fields.variant.clone(),
        kernel: options_fields.kernel.clone(),
        packages: options_fields.packages.clone(),
        output: options_fields.output.clone(),
    }
}
